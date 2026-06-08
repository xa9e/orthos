#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuantityRelationKind {
    NumeralNoun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantityConflict {
    pub numeral_class: NumeralGovernmentClass,
    pub observed_cases: BTreeSet<Case>,
    pub observed_numbers: BTreeSet<Number>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuantityAgreementCheck {
    pub relation: QuantityRelationKind,
    pub compatibility: MorphCompatibility,
    pub conflict: Option<QuantityConflict>,
    pub unknown_reason: Option<String>,
}

impl QuantityAgreementCheck {
    pub fn compatible(relation: QuantityRelationKind) -> Self {
        Self {
            relation,
            compatibility: MorphCompatibility::Compatible,
            conflict: None,
            unknown_reason: None,
        }
    }

    pub fn unknown(relation: QuantityRelationKind, reason: impl Into<String>) -> Self {
        Self {
            relation,
            compatibility: MorphCompatibility::Unknown,
            conflict: None,
            unknown_reason: Some(reason.into()),
        }
    }

    pub fn incompatible(relation: QuantityRelationKind, conflict: QuantityConflict) -> Self {
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

pub fn numeral_noun_agreement_check(
    numeral_analyses: &[MorphAnalysis],
    noun_analyses: &[MorphAnalysis],
) -> QuantityAgreementCheck {
    let numerals = numeral_analyses
        .iter()
        .filter(|analysis| analysis.pos == PartOfSpeech::Numeral)
        .collect::<Vec<_>>();
    let nouns = noun_analyses
        .iter()
        .filter(|analysis| analysis.pos == PartOfSpeech::Noun)
        .collect::<Vec<_>>();

    if numerals.is_empty() {
        return QuantityAgreementCheck::unknown(QuantityRelationKind::NumeralNoun, "no numeral analysis");
    }
    if nouns.is_empty() {
        return QuantityAgreementCheck::unknown(QuantityRelationKind::NumeralNoun, "no noun analysis");
    }

    let mut saw_incompatible = false;
    let mut saw_unknown = false;
    let mut conflict_class = NumeralGovernmentClass::Unknown;
    let mut observed_cases = BTreeSet::new();
    let mut observed_numbers = BTreeSet::new();

    for numeral in numerals {
        let class = numeral_government_class(numeral);
        if matches!(class, NumeralGovernmentClass::Unknown) {
            saw_unknown = true;
            continue;
        }

        for noun in &nouns {
            match numeral_noun_compatibility(numeral, noun) {
                NumeralNounCompatibility::Compatible => {
                    return QuantityAgreementCheck::compatible(QuantityRelationKind::NumeralNoun);
                }
                NumeralNounCompatibility::Incompatible => {
                    saw_incompatible = true;
                    conflict_class = class;
                    if let Some(case) = noun.features.case {
                        observed_cases.insert(case);
                    }
                    if let Some(number) = noun.features.number {
                        observed_numbers.insert(number);
                    }
                }
                NumeralNounCompatibility::Unknown => saw_unknown = true,
                NumeralNounCompatibility::Unsupported => {}
            }
        }
    }

    if saw_unknown {
        return QuantityAgreementCheck::unknown(QuantityRelationKind::NumeralNoun, "ambiguous quantity relation");
    }
    if saw_incompatible {
        return QuantityAgreementCheck::incompatible(
            QuantityRelationKind::NumeralNoun,
            QuantityConflict {
                numeral_class: conflict_class,
                observed_cases,
                observed_numbers,
            },
        );
    }

    QuantityAgreementCheck::unknown(QuantityRelationKind::NumeralNoun, "unsupported quantity relation")
}
