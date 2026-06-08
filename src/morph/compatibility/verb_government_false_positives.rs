#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct VerbGovernmentFalsePositiveFixture {
    pub id: String,
    pub key: VerbGovernmentKey,
    pub text: String,
    pub forbidden_excerpt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_blocker: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct VerbGovernmentFalsePositiveFixtureSet {
    fixtures: Vec<VerbGovernmentFalsePositiveFixture>,
}

impl VerbGovernmentFalsePositiveFixtureSet {
    pub fn russian_seed() -> Self {
        Self::parse_tsv(include_str!(
            "../../../data/grammar/verb_government.false_positive.tsv"
        ))
        .expect("built-in verb government false-positive fixtures must be valid")
    }

    pub fn parse_tsv(content: &str) -> Result<Self, VerbGovernmentSeedError> {
        let mut fixtures = Vec::new();
        let mut ids = BTreeSet::new();
        for (index, raw_line) in content.lines().enumerate() {
            let line_number = index + 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fixture = parse_verb_government_false_positive_line(line_number, raw_line)?;
            if !ids.insert(fixture.id.clone()) {
                return Err(VerbGovernmentSeedError::new(
                    line_number,
                    "duplicate false-positive fixture id",
                ));
            }
            fixtures.push(fixture);
        }
        Ok(Self { fixtures })
    }

    pub fn fixtures(&self) -> &[VerbGovernmentFalsePositiveFixture] {
        &self.fixtures
    }

    pub fn len(&self) -> usize {
        self.fixtures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty()
    }
}

fn parse_verb_government_false_positive_line(
    line_number: usize,
    line: &str,
) -> Result<VerbGovernmentFalsePositiveFixture, VerbGovernmentSeedError> {
    let columns = line.split('\t').collect::<Vec<_>>();
    if columns.len() != 8 {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            format!(
                "expected 8 tab-separated false-positive fixture columns, got {}",
                columns.len()
            ),
        ));
    }

    let id = required_column(line_number, columns[0], "id")?;
    let lemma = required_column(line_number, columns[1], "lemma")?;
    let complement_kind = VerbGovernmentComplementKind::parse(columns[2]).ok_or_else(|| {
        VerbGovernmentSeedError::new(
            line_number,
            format!("unknown complement_kind {:?}", columns[2]),
        )
    })?;
    let preposition = optional_column(columns[3]).map(|value| lower_ru(&value));
    let text = required_column(line_number, columns[4], "text")?;
    let forbidden_excerpt = required_column(line_number, columns[5], "forbidden_excerpt")?;
    let expected_blocker = optional_column(columns[6]);
    if let Some(blocker) = expected_blocker.as_deref() {
        if !is_known_false_positive_blocker(blocker) {
            return Err(VerbGovernmentSeedError::new(
                line_number,
                format!("unknown expected_blocker {blocker:?}"),
            ));
        }
    }
    let reason = required_column(line_number, columns[7], "reason")?;

    if complement_kind == VerbGovernmentComplementKind::DirectObject && preposition.is_some() {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            "direct_object false-positive fixtures must not specify preposition",
        ));
    }
    if complement_kind == VerbGovernmentComplementKind::PrepositionalObject && preposition.is_none() {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            "prepositional_object false-positive fixtures must specify preposition",
        ));
    }
    if !text.contains(forbidden_excerpt) {
        return Err(VerbGovernmentSeedError::new(
            line_number,
            "forbidden_excerpt must be contained in text",
        ));
    }

    Ok(VerbGovernmentFalsePositiveFixture {
        id: id.to_owned(),
        key: VerbGovernmentKey::new(lemma, complement_kind, preposition),
        text: text.to_owned(),
        forbidden_excerpt: forbidden_excerpt.to_owned(),
        expected_blocker,
        reason: reason.to_owned(),
    })
}


fn is_known_false_positive_blocker(value: &str) -> bool {
    matches!(
        value,
        "DirectSpeechBoundary" | "ParenthesisBoundary" | "UnsafeBoundary" | "ClauseBoundary"
    )
}
