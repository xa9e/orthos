#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GovernmentRelationKind {
    PrepositionNominal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernmentConflict {
    pub expected: FeatureConstraintSet,
    pub observed_cases: BTreeSet<Case>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernmentCheck {
    pub relation: GovernmentRelationKind,
    pub compatibility: MorphCompatibility,
    pub conflict: Option<GovernmentConflict>,
    pub unknown_reason: Option<String>,
}

impl GovernmentCheck {
    pub fn compatible(relation: GovernmentRelationKind) -> Self {
        Self {
            relation,
            compatibility: MorphCompatibility::Compatible,
            conflict: None,
            unknown_reason: None,
        }
    }

    pub fn unknown(relation: GovernmentRelationKind, reason: impl Into<String>) -> Self {
        Self {
            relation,
            compatibility: MorphCompatibility::Unknown,
            conflict: None,
            unknown_reason: Some(reason.into()),
        }
    }

    pub fn incompatible(relation: GovernmentRelationKind, conflict: GovernmentConflict) -> Self {
        Self {
            relation,
            compatibility: MorphCompatibility::Incompatible,
            conflict: Some(conflict),
            unknown_reason: None,
        }
    }

    pub fn is_confident_rejection(&self) -> bool {
        self.compatibility == MorphCompatibility::Incompatible && self.conflict.is_some()
    }
}

pub fn preposition_government_check(
    registry: &PrepositionGovernmentRegistry,
    preposition: &str,
    nominal_analyses: &[MorphAnalysis],
) -> GovernmentCheck {
    let entries = registry.lookup(preposition);
    if entries.is_empty() {
        return GovernmentCheck::unknown(GovernmentRelationKind::PrepositionNominal, "unknown governor");
    }

    let nominals = nominal_analyses
        .iter()
        .filter(|analysis| can_be_governed_nominal(analysis))
        .collect::<Vec<_>>();
    if nominals.is_empty() {
        return GovernmentCheck::unknown(GovernmentRelationKind::PrepositionNominal, "no nominal analysis");
    }

    let expected = entries.iter().fold(FeatureConstraintSet::new(), |acc, entry| {
        acc.require_cases(entry.allowed_cases.iter().copied())
    });
    let mut observed_cases = BTreeSet::new();
    let mut has_unknown_case = false;

    for analysis in nominals {
        match analysis.features.case {
            Some(case) if expected.allows_case(case) == MorphCompatibility::Compatible => {
                return GovernmentCheck::compatible(GovernmentRelationKind::PrepositionNominal);
            }
            Some(case) => {
                observed_cases.insert(case);
            }
            None => has_unknown_case = true,
        }
    }

    if has_unknown_case || observed_cases.is_empty() {
        return GovernmentCheck::unknown(GovernmentRelationKind::PrepositionNominal, "unknown governed case");
    }

    GovernmentCheck::incompatible(
        GovernmentRelationKind::PrepositionNominal,
        GovernmentConflict {
            expected,
            observed_cases,
        },
    )
}

fn can_be_governed_nominal(analysis: &MorphAnalysis) -> bool {
    matches!(
        analysis.pos,
        PartOfSpeech::Noun
            | PartOfSpeech::Pronoun
            | PartOfSpeech::Adjective
            | PartOfSpeech::Numeral
            | PartOfSpeech::Participle
    )
}
