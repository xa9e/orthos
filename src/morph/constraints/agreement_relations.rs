#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AgreementRelationKind {
    AdjectiveNoun,
    SubjectPredicate,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AgreementFeatureKind {
    Case,
    Number,
    Gender,
    Person,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgreementConflict {
    pub feature: AgreementFeatureKind,
    pub left: String,
    pub right: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgreementCheck {
    pub relation: AgreementRelationKind,
    pub compatibility: MorphCompatibility,
    pub conflicts: Vec<AgreementConflict>,
    pub unknown_features: Vec<AgreementFeatureKind>,
    pub unification: Option<FeatureUnification>,
}

impl AgreementCheck {
    pub fn compatible(relation: AgreementRelationKind) -> Self {
        Self {
            relation,
            compatibility: MorphCompatibility::Compatible,
            conflicts: Vec::new(),
            unknown_features: Vec::new(),
            unification: None,
        }
    }

    pub fn is_confident_rejection(&self) -> bool {
        self.compatibility == MorphCompatibility::Incompatible && !self.conflicts.is_empty()
    }
}

pub fn adj_noun_agreement_check(adj: &MorphAnalysis, noun: &MorphAnalysis) -> AgreementCheck {
    let mut check = AgreementCheck::compatible(AgreementRelationKind::AdjectiveNoun);
    if !adj.pos.can_modify_noun() || noun.pos != PartOfSpeech::Noun {
        check.compatibility = MorphCompatibility::Unknown;
        return check;
    }

    compare_feature(&mut check, AgreementFeatureKind::Case, adj.features.case, noun.features.case);
    compare_feature(
        &mut check,
        AgreementFeatureKind::Number,
        adj.features.number,
        noun.features.number,
    );
    compare_gender(
        &mut check,
        adj.features.number,
        noun.features.number,
        adj.features.gender,
        noun.features.gender,
    );
    finalize_agreement_check(&mut check);
    check
}

pub fn subject_predicate_agreement_check(
    subject: &MorphAnalysis,
    predicate: &MorphAnalysis,
) -> AgreementCheck {
    let mut check = AgreementCheck::compatible(AgreementRelationKind::SubjectPredicate);
    let subject_signature = subject_agreement_signature(subject);
    let predicate_signature = predicate_agreement_signature(predicate);

    if subject_signature.person.is_none() && subject.pos != PartOfSpeech::Noun {
        check.unknown_features.push(AgreementFeatureKind::Person);
    }
    compare_feature(
        &mut check,
        AgreementFeatureKind::Number,
        subject_signature.number,
        predicate_signature.number,
    );
    if predicate_signature.person.is_some() {
        compare_feature(
            &mut check,
            AgreementFeatureKind::Person,
            subject_signature.person,
            predicate_signature.person,
        );
    }
    if predicate_needs_gender(predicate_signature) {
        compare_gender(
            &mut check,
            subject_signature.number,
            predicate_signature.number,
            subject_signature.gender,
            predicate_signature.gender,
        );
    }

    finalize_agreement_check(&mut check);
    check
}

pub fn confidently_reject_subject_predicate_agreement(
    subject_analyses: &[MorphAnalysis],
    predicate_analyses: &[MorphAnalysis],
) -> bool {
    let subjects: Vec<&MorphAnalysis> = subject_analyses.iter().filter(|analysis| can_be_subject(analysis)).collect();
    let predicates: Vec<&MorphAnalysis> = predicate_analyses.iter().filter(|analysis| can_be_predicate(analysis)).collect();

    if subjects.is_empty() || predicates.is_empty() || has_non_subject(subject_analyses) || has_non_predicate(predicate_analyses) {
        return false;
    }
    if subjects.len() != 1 || predicates.len() != 1 {
        return false;
    }

    subject_predicate_agreement_check(subjects[0], predicates[0]).is_confident_rejection()
}

fn compare_feature<T>(
    check: &mut AgreementCheck,
    feature: AgreementFeatureKind,
    left: Option<T>,
    right: Option<T>,
) where
    T: Eq + std::fmt::Debug,
{
    match optional_value_compatibility(left.as_ref(), right.as_ref()) {
        MorphCompatibility::Compatible => {}
        MorphCompatibility::Incompatible => check.conflicts.push(AgreementConflict {
            feature,
            left: format_value(left),
            right: format_value(right),
        }),
        MorphCompatibility::Unknown => push_unknown(check, feature),
    }
}

fn compare_gender(
    check: &mut AgreementCheck,
    left_number: Option<Number>,
    right_number: Option<Number>,
    left_gender: Option<Gender>,
    right_gender: Option<Gender>,
) {
    match gender_compatibility(left_number, right_number, left_gender, right_gender) {
        MorphCompatibility::Compatible => {}
        MorphCompatibility::Incompatible => check.conflicts.push(AgreementConflict {
            feature: AgreementFeatureKind::Gender,
            left: format_value(left_gender),
            right: format_value(right_gender),
        }),
        MorphCompatibility::Unknown => push_unknown(check, AgreementFeatureKind::Gender),
    }
}

fn finalize_agreement_check(check: &mut AgreementCheck) {
    check.compatibility = if !check.conflicts.is_empty() {
        MorphCompatibility::Incompatible
    } else if !check.unknown_features.is_empty() {
        MorphCompatibility::Unknown
    } else {
        MorphCompatibility::Compatible
    };
}

fn push_unknown(check: &mut AgreementCheck, feature: AgreementFeatureKind) {
    if !check.unknown_features.contains(&feature) {
        check.unknown_features.push(feature);
    }
}

fn format_value<T: std::fmt::Debug>(value: Option<T>) -> String {
    value.map(|item| format!("{item:?}")).unwrap_or_else(|| "None".to_string())
}

fn predicate_needs_gender(predicate: PredicateAgreementSignature) -> bool {
    matches!(predicate.tense, Some(Tense::Past)) && matches!(predicate.number, Some(Number::Singular))
}

fn can_be_subject(analysis: &MorphAnalysis) -> bool {
    matches!(analysis.pos, PartOfSpeech::Noun | PartOfSpeech::Pronoun | PartOfSpeech::Numeral)
        && !matches!(analysis.features.case, Some(case) if case != Case::Nominative)
}

fn can_be_predicate(analysis: &MorphAnalysis) -> bool {
    matches!(analysis.pos, PartOfSpeech::Verb | PartOfSpeech::Participle | PartOfSpeech::Predicative)
}

fn has_non_subject(analyses: &[MorphAnalysis]) -> bool {
    analyses.iter().any(|analysis| !can_be_subject(analysis))
}

fn has_non_predicate(analyses: &[MorphAnalysis]) -> bool {
    analyses.iter().any(|analysis| !can_be_predicate(analysis))
}
