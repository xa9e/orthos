#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerbGovernmentComplementKind {
    DirectObject,
    PrepositionalObject,
}

impl VerbGovernmentComplementKind {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match raw.trim() {
            "direct_object" => Self::DirectObject,
            "prepositional_object" => Self::PrepositionalObject,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerbGovernment {
    pub lemma: String,
    pub complement_kind: VerbGovernmentComplementKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preposition: Option<String>,
    pub allowed_cases: BTreeSet<Case>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<SourceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl VerbGovernment {
    pub fn direct_object(lemma: impl Into<String>, cases: impl IntoIterator<Item = Case>) -> Self {
        Self::new(
            lemma,
            VerbGovernmentComplementKind::DirectObject,
            None,
            cases,
        )
    }

    pub fn prepositional_object(
        lemma: impl Into<String>,
        preposition: impl Into<String>,
        cases: impl IntoIterator<Item = Case>,
    ) -> Self {
        Self::new(
            lemma,
            VerbGovernmentComplementKind::PrepositionalObject,
            Some(preposition.into()),
            cases,
        )
    }

    pub fn new(
        lemma: impl Into<String>,
        complement_kind: VerbGovernmentComplementKind,
        preposition: Option<String>,
        cases: impl IntoIterator<Item = Case>,
    ) -> Self {
        Self {
            lemma: lower_ru(&lemma.into()),
            complement_kind,
            preposition: preposition.map(|value| lower_ru(&value)),
            allowed_cases: cases.into_iter().collect(),
            source_id: None,
            note: None,
        }
    }

    pub fn allows_case(&self, case: Case) -> bool {
        self.allowed_cases.contains(&case)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct VerbGovernmentSeedError {
    pub line: usize,
    pub message: String,
}

impl VerbGovernmentSeedError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for VerbGovernmentSeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "verb government seed line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for VerbGovernmentSeedError {}

#[derive(Debug, Clone, Default)]
pub struct VerbGovernmentRegistry {
    entries_by_lemma: HashMap<String, Vec<VerbGovernment>>,
}

impl VerbGovernmentRegistry {
    pub fn russian_seed() -> Self {
        Self::parse_seed_tsv(include_str!("../../../data/grammar/verb_government.seed.tsv"))
            .expect("built-in verb government seed must be valid")
    }

    pub fn parse_seed_tsv(content: &str) -> Result<Self, VerbGovernmentSeedError> {
        let mut registry = Self::default();
        let mut seen = BTreeSet::new();
        for (index, raw_line) in content.lines().enumerate() {
            let line_number = index + 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let entry = parse_verb_government_seed_line(line_number, raw_line)?;
            let key = (
                entry.lemma.clone(),
                entry.complement_kind,
                entry.preposition.clone().unwrap_or_default(),
            );
            if !seen.insert(key) {
                return Err(VerbGovernmentSeedError::new(
                    line_number,
                    "duplicate lemma/complement/preposition seed row",
                ));
            }
            registry.insert(entry);
        }
        Ok(registry)
    }

    pub fn insert(&mut self, government: VerbGovernment) {
        self.entries_by_lemma
            .entry(government.lemma.clone())
            .or_default()
            .push(government);
    }

    pub fn lookup(&self, lemma: &str) -> &[VerbGovernment] {
        self.entries_by_lemma
            .get(&lower_ru(lemma))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn direct_cases_for_lemma(&self, lemma: &str) -> BTreeSet<Case> {
        self.lookup(lemma)
            .iter()
            .filter(|entry| entry.complement_kind == VerbGovernmentComplementKind::DirectObject)
            .flat_map(|entry| entry.allowed_cases.iter().copied())
            .collect()
    }

    pub fn direct_entries_for_lemma(&self, lemma: &str) -> Vec<&VerbGovernment> {
        self.lookup(lemma)
            .iter()
            .filter(|entry| entry.complement_kind == VerbGovernmentComplementKind::DirectObject)
            .collect()
    }

    pub fn prepositional_entries_for_lemma(&self, lemma: &str) -> Vec<&VerbGovernment> {
        self.lookup(lemma)
            .iter()
            .filter(|entry| entry.complement_kind == VerbGovernmentComplementKind::PrepositionalObject)
            .collect()
    }

    pub fn entries(&self) -> Vec<&VerbGovernment> {
        let mut entries = self
            .entries_by_lemma
            .values()
            .flat_map(|items| items.iter())
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            (
                left.lemma.as_str(),
                left.complement_kind,
                left.preposition.as_deref().unwrap_or(""),
            )
                .cmp(&(
                    right.lemma.as_str(),
                    right.complement_kind,
                    right.preposition.as_deref().unwrap_or(""),
                ))
        });
        entries
    }

    pub fn len(&self) -> usize {
        self.entries_by_lemma.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.entries_by_lemma.is_empty()
    }
}

fn parse_verb_government_seed_line(
    line_number: usize,
    line: &str,
) -> Result<VerbGovernment, VerbGovernmentSeedError> {
    let columns = line.split('\t').collect::<Vec<_>>();
    if columns.len() != 6 {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            format!("expected 6 tab-separated columns, got {}", columns.len()),
        ));
    }

    let lemma = required_column(line_number, columns[0], "lemma")?;
    let complement_kind = VerbGovernmentComplementKind::parse(columns[1]).ok_or_else(|| {
        VerbGovernmentSeedError::new(
            line_number,
            format!("unknown complement_kind {:?}", columns[1]),
        )
    })?;
    let preposition = optional_column(columns[2]).map(|value| lower_ru(&value));
    let cases = parse_case_set(line_number, columns[3])?;
    let source_id = required_column(line_number, columns[4], "source_id")?;
    let note = required_column(line_number, columns[5], "note")?;

    if complement_kind == VerbGovernmentComplementKind::DirectObject && preposition.is_some() {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            "direct_object rows must not specify preposition",
        ));
    }
    if complement_kind == VerbGovernmentComplementKind::PrepositionalObject && preposition.is_none() {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            "prepositional_object rows must specify preposition",
        ));
    }

    let mut government = VerbGovernment::new(lemma, complement_kind, preposition, cases);
    government.source_id = Some(SourceId::new(source_id));
    government.note = Some(note.to_owned());
    Ok(government)
}

fn parse_case_set(
    line_number: usize,
    raw: &str,
) -> Result<BTreeSet<Case>, VerbGovernmentSeedError> {
    let mut cases = BTreeSet::new();
    for raw_case in raw.split('|') {
        let value = raw_case.trim();
        if value.is_empty() {
            continue;
        }
        let Some(case) = Case::parse(value) else {
            return Err(VerbGovernmentSeedError::new(
                line_number,
                format!("unknown case {value:?}"),
            ));
        };
        cases.insert(case);
    }
    if cases.is_empty() {
        return Err(VerbGovernmentSeedError::new(line_number, "empty cases column"));
    }
    Ok(cases)
}

fn required_column<'a>(
    line_number: usize,
    value: &'a str,
    name: &str,
) -> Result<&'a str, VerbGovernmentSeedError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(VerbGovernmentSeedError::new(
            line_number,
            format!("empty {name} column"),
        ))
    } else {
        Ok(trimmed)
    }
}

fn optional_column(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
