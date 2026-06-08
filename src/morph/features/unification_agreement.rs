#[derive(Debug, Default)]
struct AgreementFeatureBuckets {
    cases: BTreeSet<Case>,
    numbers: BTreeSet<Number>,
    genders: BTreeSet<Gender>,
    persons: BTreeSet<Person>,
}

impl AgreementFeatureBuckets {
    fn from_adj_noun_side(analyses: &[&MorphAnalysis]) -> Self {
        let mut out = Self::default();
        for analysis in analyses {
            out.insert_case(analysis.features.case);
            out.insert_number(analysis.features.number);
            out.insert_gender(analysis.features.gender);
        }
        out
    }

    fn from_subject_side(analyses: &[&MorphAnalysis]) -> Self {
        let mut out = Self::default();
        for analysis in analyses {
            let signature = subject_agreement_signature(analysis);
            out.insert_number(signature.number);
            out.insert_gender(signature.gender);
            out.insert_person(signature.person);
        }
        out
    }

    fn from_predicate_side(analyses: &[&MorphAnalysis]) -> Self {
        let mut out = Self::default();
        for analysis in analyses {
            let signature = predicate_agreement_signature(analysis);
            out.insert_number(signature.number);
            out.insert_gender(signature.gender);
            out.insert_person(signature.person);
        }
        out
    }

    fn insert_case(&mut self, value: Option<Case>) {
        if let Some(value) = value {
            self.cases.insert(value);
        }
    }

    fn insert_number(&mut self, value: Option<Number>) {
        if let Some(value) = value {
            self.numbers.insert(value);
        }
    }

    fn insert_gender(&mut self, value: Option<Gender>) {
        if let Some(value) = value {
            self.genders.insert(value);
        }
    }

    fn insert_person(&mut self, value: Option<Person>) {
        if let Some(value) = value {
            self.persons.insert(value);
        }
    }
}

pub fn feature_unification_agreement_check(
    relation: AgreementRelationKind,
    left_analyses: &[MorphAnalysis],
    right_analyses: &[MorphAnalysis],
) -> AgreementCheck {
    let unification = match relation {
        AgreementRelationKind::AdjectiveNoun => adjective_noun_feature_unification(left_analyses, right_analyses),
        AgreementRelationKind::SubjectPredicate => subject_predicate_feature_unification(left_analyses, right_analyses),
    };
    let compatibility = unification.compatibility();
    AgreementCheck {
        relation,
        compatibility,
        conflicts: unification.conflicts(),
        unknown_features: unification.unknown_features(),
        unification: Some(unification),
    }
}

pub fn adjective_noun_feature_unification(
    left_analyses: &[MorphAnalysis],
    right_analyses: &[MorphAnalysis],
) -> FeatureUnification {
    let left_candidates = left_analyses
        .iter()
        .filter(|analysis| can_unify_as_nominal_modifier(analysis))
        .collect::<Vec<_>>();
    let right_candidates = right_analyses
        .iter()
        .filter(|analysis| analysis.pos == PartOfSpeech::Noun)
        .collect::<Vec<_>>();
    let mut unification = FeatureUnification::new(AgreementRelationKind::AdjectiveNoun);

    if left_candidates.is_empty() || right_candidates.is_empty() {
        return unification.with_step(unknown_step(
            AgreementFeatureKind::Case,
            "missing modifier or noun candidate",
        ));
    }
    if left_analyses.iter().any(|analysis| !can_unify_as_nominal_modifier(analysis))
        || right_analyses.iter().any(|analysis| analysis.pos != PartOfSpeech::Noun)
    {
        return unification.with_step(unknown_step(
            AgreementFeatureKind::Case,
            "role ambiguity outside modifier-head relation",
        ));
    }

    let left = AgreementFeatureBuckets::from_adj_noun_side(&left_candidates);
    let right = AgreementFeatureBuckets::from_adj_noun_side(&right_candidates);
    unification = unification
        .with_step(unify_typed_sets(
            AgreementFeatureKind::Case,
            &left.cases,
            &right.cases,
            false,
        ))
        .with_step(unify_typed_sets(
            AgreementFeatureKind::Number,
            &left.numbers,
            &right.numbers,
            false,
        ));

    if should_unify_gender_for_nominal_agreement(&left.numbers, &right.numbers) {
        unification = unification.with_step(unify_gender_sets(&left.genders, &right.genders));
    } else {
        unification = unification.with_step(not_applicable_step(
            AgreementFeatureKind::Gender,
            "plural nominal agreement does not require gender",
        ));
    }
    if unification.compatibility() == MorphCompatibility::Compatible
        && !any_compatible_adj_noun_pair(&left_candidates, &right_candidates)
    {
        unification = unification.with_step(FeatureUnificationStep {
            feature: AgreementFeatureKind::Case,
            left_values: Vec::new(),
            right_values: Vec::new(),
            intersection: Vec::new(),
            status: FeatureUnificationStatus::Conflict,
            note: Some("no compatible modifier-noun pair across individual analyses".to_owned()),
        });
    }
    unification
}

pub fn subject_predicate_feature_unification(
    left_analyses: &[MorphAnalysis],
    right_analyses: &[MorphAnalysis],
) -> FeatureUnification {
    let subject_candidates = left_analyses
        .iter()
        .filter(|analysis| can_be_unified_subject(analysis))
        .collect::<Vec<_>>();
    let predicate_candidates = right_analyses
        .iter()
        .filter(|analysis| can_be_unified_predicate(analysis))
        .collect::<Vec<_>>();
    let mut unification = FeatureUnification::new(AgreementRelationKind::SubjectPredicate);

    if subject_candidates.is_empty() || predicate_candidates.is_empty() {
        return unification.with_step(unknown_step(
            AgreementFeatureKind::Number,
            "missing subject or predicate candidate",
        ));
    }
    if left_analyses.iter().any(|analysis| !can_be_unified_subject(analysis))
        || right_analyses.iter().any(|analysis| !can_be_unified_predicate(analysis))
    {
        return unification.with_step(unknown_step(
            AgreementFeatureKind::Number,
            "role ambiguity outside subject-predicate relation",
        ));
    }

    let subject = AgreementFeatureBuckets::from_subject_side(&subject_candidates);
    let predicate = AgreementFeatureBuckets::from_predicate_side(&predicate_candidates);
    unification = unification.with_step(unify_typed_sets(
        AgreementFeatureKind::Number,
        &subject.numbers,
        &predicate.numbers,
        false,
    ));

    if !predicate.persons.is_empty() {
        unification = unification.with_step(unify_typed_sets(
            AgreementFeatureKind::Person,
            &subject.persons,
            &predicate.persons,
            false,
        ));
    }

    if predicate_gender_is_relevant(&predicate_candidates) {
        unification = unification.with_step(unify_gender_sets(&subject.genders, &predicate.genders));
    } else {
        unification = unification.with_step(not_applicable_step(
            AgreementFeatureKind::Gender,
            "predicate form does not require gender agreement",
        ));
    }
    unification
}

fn any_compatible_adj_noun_pair(
    adj: &[&MorphAnalysis],
    noun: &[&MorphAnalysis],
) -> bool {
    adj.iter().any(|a| noun.iter().any(|n| can_agree_as_adj_noun(a, n)))
}
