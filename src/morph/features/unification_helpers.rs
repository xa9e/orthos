fn unify_typed_sets<T>(
    feature: AgreementFeatureKind,
    left: &BTreeSet<T>,
    right: &BTreeSet<T>,
    empty_is_compatible: bool,
) -> FeatureUnificationStep
where
    T: Ord + std::fmt::Debug,
{
    let left_values = describe_typed_set(left);
    let right_values = describe_typed_set(right);
    let intersection = left.intersection(right).map(|value| format!("{value:?}")).collect::<Vec<_>>();
    let status = if left.is_empty() || right.is_empty() {
        if empty_is_compatible {
            FeatureUnificationStatus::Compatible
        } else {
            FeatureUnificationStatus::Unknown
        }
    } else if intersection.is_empty() {
        FeatureUnificationStatus::Conflict
    } else {
        FeatureUnificationStatus::Compatible
    };
    FeatureUnificationStep {
        feature,
        left_values,
        right_values,
        intersection,
        status,
        note: None,
    }
}

fn unify_gender_sets(
    left: &BTreeSet<Gender>,
    right: &BTreeSet<Gender>,
) -> FeatureUnificationStep {
    let left_values = describe_typed_set(left);
    let right_values = describe_typed_set(right);
    let mut intersection = left
        .intersection(right)
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>();
    if intersection.is_empty() && (!left.is_empty() && !right.is_empty()) {
        if left.contains(&Gender::Common) {
            intersection.extend(right.iter().map(|value| format!("{value:?}")));
        } else if right.contains(&Gender::Common) {
            intersection.extend(left.iter().map(|value| format!("{value:?}")));
        }
        intersection.sort();
        intersection.dedup();
    }
    let status = if left.is_empty() || right.is_empty() {
        FeatureUnificationStatus::Unknown
    } else if intersection.is_empty() {
        FeatureUnificationStatus::Conflict
    } else {
        FeatureUnificationStatus::Compatible
    };
    FeatureUnificationStep {
        feature: AgreementFeatureKind::Gender,
        left_values,
        right_values,
        intersection,
        status,
        note: None,
    }
}

fn unknown_step(feature: AgreementFeatureKind, note: &str) -> FeatureUnificationStep {
    FeatureUnificationStep {
        feature,
        left_values: Vec::new(),
        right_values: Vec::new(),
        intersection: Vec::new(),
        status: FeatureUnificationStatus::Unknown,
        note: Some(note.to_owned()),
    }
}

fn not_applicable_step(feature: AgreementFeatureKind, note: &str) -> FeatureUnificationStep {
    FeatureUnificationStep {
        feature,
        left_values: Vec::new(),
        right_values: Vec::new(),
        intersection: Vec::new(),
        status: FeatureUnificationStatus::NotApplicable,
        note: Some(note.to_owned()),
    }
}

fn describe_typed_set<T>(values: &BTreeSet<T>) -> Vec<String>
where
    T: std::fmt::Debug,
{
    values.iter().map(|value| format!("{value:?}")).collect()
}

fn describe_values(values: &[String]) -> String {
    if values.is_empty() {
        "∅".to_owned()
    } else {
        values.join(" | ")
    }
}

fn can_unify_as_nominal_modifier(analysis: &MorphAnalysis) -> bool {
    analysis.pos.can_modify_noun()
        && !matches!(analysis.features.degree, Some(Degree::Comparative))
        && !matches!(analysis.features.adjective_form, Some(AdjectiveForm::Short))
}

fn can_be_unified_subject(analysis: &MorphAnalysis) -> bool {
    matches!(analysis.pos, PartOfSpeech::Noun | PartOfSpeech::Pronoun | PartOfSpeech::Numeral)
        && !matches!(analysis.features.case, Some(case) if case != Case::Nominative)
}

fn can_be_unified_predicate(analysis: &MorphAnalysis) -> bool {
    matches!(analysis.pos, PartOfSpeech::Verb | PartOfSpeech::Participle | PartOfSpeech::Predicative)
}

fn should_unify_gender_for_nominal_agreement(
    left_numbers: &BTreeSet<Number>,
    right_numbers: &BTreeSet<Number>,
) -> bool {
    left_numbers.is_empty()
        || right_numbers.is_empty()
        || left_numbers.contains(&Number::Singular)
        || right_numbers.contains(&Number::Singular)
}

fn predicate_gender_is_relevant(predicate_candidates: &[&MorphAnalysis]) -> bool {
    predicate_candidates.iter().any(|analysis| {
        matches!(analysis.features.tense, Some(Tense::Past))
            && matches!(analysis.features.number, Some(Number::Singular) | None)
    })
}
