#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerbGovernmentKey {
    pub lemma: String,
    pub complement_kind: VerbGovernmentComplementKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preposition: Option<String>,
}

impl VerbGovernmentKey {
    pub fn new(
        lemma: impl Into<String>,
        complement_kind: VerbGovernmentComplementKind,
        preposition: Option<String>,
    ) -> Self {
        Self {
            lemma: lower_ru(&lemma.into()),
            complement_kind,
            preposition: preposition.map(|value| lower_ru(&value)),
        }
    }

    pub fn from_government(entry: &VerbGovernment) -> Self {
        Self::new(
            entry.lemma.clone(),
            entry.complement_kind,
            entry.preposition.clone(),
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerbGovernmentFixture {
    pub key: VerbGovernmentKey,
    pub valid_text: String,
    pub invalid_text: String,
    pub invalid_excerpt: String,
}

#[derive(Debug, Clone, Default)]
pub struct VerbGovernmentFixtureSet {
    fixtures: Vec<VerbGovernmentFixture>,
    keys: BTreeSet<VerbGovernmentKey>,
}

impl VerbGovernmentFixtureSet {
    pub fn russian_seed() -> Self {
        Self::parse_tsv(include_str!("../../../data/grammar/verb_government.fixtures.tsv"))
            .expect("built-in verb government fixtures must be valid")
    }

    pub fn parse_tsv(content: &str) -> Result<Self, VerbGovernmentSeedError> {
        let mut fixtures = Vec::new();
        let mut keys = BTreeSet::new();
        for (index, raw_line) in content.lines().enumerate() {
            let line_number = index + 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fixture = parse_verb_government_fixture_line(line_number, raw_line)?;
            if !keys.insert(fixture.key.clone()) {
                return Err(VerbGovernmentSeedError::new(
                    line_number,
                    "duplicate fixture lemma/complement/preposition row",
                ));
            }
            fixtures.push(fixture);
        }
        Ok(Self { fixtures, keys })
    }

    pub fn fixtures(&self) -> &[VerbGovernmentFixture] {
        &self.fixtures
    }

    pub fn contains_key(&self, key: &VerbGovernmentKey) -> bool {
        self.keys.contains(key)
    }

    pub fn len(&self) -> usize {
        self.fixtures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty()
    }
}

fn parse_verb_government_fixture_line(
    line_number: usize,
    line: &str,
) -> Result<VerbGovernmentFixture, VerbGovernmentSeedError> {
    let columns = line.split('\t').collect::<Vec<_>>();
    if columns.len() != 6 {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            format!("expected 6 tab-separated fixture columns, got {}", columns.len()),
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
    let valid_text = required_column(line_number, columns[3], "valid_text")?;
    let invalid_text = required_column(line_number, columns[4], "invalid_text")?;
    let invalid_excerpt = required_column(line_number, columns[5], "invalid_excerpt")?;

    if complement_kind == VerbGovernmentComplementKind::DirectObject && preposition.is_some() {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            "direct_object fixtures must not specify preposition",
        ));
    }
    if complement_kind == VerbGovernmentComplementKind::PrepositionalObject && preposition.is_none() {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            "prepositional_object fixtures must specify preposition",
        ));
    }
    if !invalid_text.contains(invalid_excerpt) {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            "invalid_excerpt must be contained in invalid_text",
        ));
    }

    Ok(VerbGovernmentFixture {
        key: VerbGovernmentKey::new(lemma, complement_kind, preposition),
        valid_text: valid_text.to_owned(),
        invalid_text: invalid_text.to_owned(),
        invalid_excerpt: invalid_excerpt.to_owned(),
    })
}
