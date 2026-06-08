#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrepositionGovernment {
    pub preposition: String,
    pub allowed_cases: BTreeSet<Case>,
    pub source_id: Option<SourceId>,
    pub note: Option<String>,
}

impl PrepositionGovernment {
    pub fn new(preposition: impl Into<String>, allowed_cases: impl IntoIterator<Item = Case>) -> Self {
        Self {
            preposition: lower_ru(&preposition.into()),
            allowed_cases: allowed_cases.into_iter().collect(),
            source_id: None,
            note: None,
        }
    }

    pub fn allows_case(&self, case: Case) -> bool {
        self.allowed_cases.contains(&case)
    }
}

#[derive(Debug, Clone, Default)]
pub struct PrepositionGovernmentRegistry {
    entries: HashMap<String, Vec<PrepositionGovernment>>,
}

impl PrepositionGovernmentRegistry {
    pub fn insert(&mut self, government: PrepositionGovernment) {
        self.entries
            .entry(government.preposition.clone())
            .or_default()
            .push(government);
    }

    pub fn russian_seed() -> Self {
        let mut registry = Self::default();
        registry.insert_many("без", [Case::Genitive], "negative/source genitive");
        registry.insert_many("для", [Case::Genitive], "purpose genitive");
        registry.insert_many("до", [Case::Genitive], "limit genitive");
        registry.insert_many("из", [Case::Genitive], "source genitive");
        registry.insert_many("от", [Case::Genitive], "source genitive");
        registry.insert_many("у", [Case::Genitive], "possessive/location genitive");
        registry.insert_many("около", [Case::Genitive], "near genitive");
        registry.insert_many("вокруг", [Case::Genitive], "around genitive");
        registry.insert_many("после", [Case::Genitive], "posterior genitive");
        registry.insert_many("против", [Case::Genitive], "opposition genitive");
        registry.insert_many("к", [Case::Dative], "direction dative");
        registry.insert_many("согласно", [Case::Dative], "normative dative");
        registry.insert_many("благодаря", [Case::Dative], "causal dative");
        registry.insert_many("вопреки", [Case::Dative], "concessive dative");
        registry.insert_many("о", [Case::Prepositional], "topic prepositional");
        registry.insert_many("об", [Case::Prepositional], "topic prepositional");
        registry.insert_many("при", [Case::Prepositional], "circumstantial prepositional");
        registry.insert_many("между", [Case::Instrumental], "between instrumental");
        registry.insert_many("над", [Case::Instrumental], "above instrumental");
        registry.insert_many("под", [Case::Instrumental], "under instrumental seed");
        registry
    }

    fn insert_many(
        &mut self,
        preposition: &str,
        cases: impl IntoIterator<Item = Case>,
        note: &str,
    ) {
        let mut government = PrepositionGovernment::new(preposition, cases);
        government.source_id = Some(SourceId::new("project.preposition_government_seed"));
        government.note = Some(note.to_string());
        self.insert(government);
    }

    pub fn prepositions(&self) -> BTreeSet<String> {
        self.entries.keys().cloned().collect()
    }

    pub fn lookup(&self, preposition: &str) -> &[PrepositionGovernment] {
        self.entries
            .get(&lower_ru(preposition))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn allows_case(&self, preposition: &str, case: Case) -> MorphCompatibility {
        let entries = self.lookup(preposition);
        if entries.is_empty() {
            return MorphCompatibility::Unknown;
        }
        if entries.iter().any(|entry| entry.allows_case(case)) {
            MorphCompatibility::Compatible
        } else {
            MorphCompatibility::Incompatible
        }
    }
}

/// Backward-compatible function name from the first demo implementation.
pub fn compatible_adj_noun(adj: &MorphAnalysis, noun: &MorphAnalysis) -> bool {
    can_agree_as_adj_noun(adj, noun)
}

fn has_normalized_tag(features: &MorphFeatures, tag: &str) -> bool {
    features.normalized_tags.contains(&normalize_tag(tag))
}

fn agreement_compatibility(
    left: AgreementSignature,
    right: AgreementSignature,
) -> MorphCompatibility {
    let case = case_compatibility(left.case, right.case);
    let number = number_compatibility(left.number, right.number);
    let gender = gender_compatibility(left.number, right.number, left.gender, right.gender);

    if matches!(case, MorphCompatibility::Incompatible)
        || matches!(number, MorphCompatibility::Incompatible)
        || matches!(gender, MorphCompatibility::Incompatible)
    {
        MorphCompatibility::Incompatible
    } else if matches!(case, MorphCompatibility::Unknown)
        || matches!(number, MorphCompatibility::Unknown)
        || matches!(gender, MorphCompatibility::Unknown)
    {
        MorphCompatibility::Unknown
    } else {
        MorphCompatibility::Compatible
    }
}

fn governed_case_number_compatibility(
    numeral_case: Option<Case>,
    noun_case: Option<Case>,
    noun_number: Option<Number>,
    expected_case: Case,
    expected_number: Number,
) -> MorphCompatibility {
    match numeral_case {
        Some(Case::Nominative | Case::Accusative) => {}
        Some(_) => return MorphCompatibility::Unknown,
        None => return MorphCompatibility::Unknown,
    }

    match (noun_case, noun_number) {
        (Some(case), Some(number)) if case == expected_case && number == expected_number => {
            MorphCompatibility::Compatible
        }
        (Some(_), Some(_)) => MorphCompatibility::Incompatible,
        _ => MorphCompatibility::Unknown,
    }
}

fn unique_signatures(analyses: &[&MorphAnalysis]) -> BTreeSet<AgreementSignature> {
    analyses
        .iter()
        .map(|analysis| analysis.agreement_signature())
        .collect()
}

fn optional_value_compatibility<T: Eq>(left: Option<T>, right: Option<T>) -> MorphCompatibility {
    match (left, right) {
        (Some(left), Some(right)) if left == right => MorphCompatibility::Compatible,
        (Some(_), Some(_)) => MorphCompatibility::Incompatible,
        _ => MorphCompatibility::Unknown,
    }
}

#[allow(dead_code)]
fn gender_compatible(
    left_number: Option<Number>,
    right_number: Option<Number>,
    left_gender: Option<Gender>,
    right_gender: Option<Gender>,
) -> bool {
    gender_compatibility(left_number, right_number, left_gender, right_gender).permits_conservative_match()
}

fn normalize_tag(raw: &str) -> String {
    raw.trim()
        .trim_start_matches('@')
        .to_lowercase()
        .replace('-', "_")
}
