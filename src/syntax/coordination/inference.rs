fn infer_coordination_kind(
    members: &[CoordinationMember<'_>],
    head_context: Option<&Token<'_>>,
) -> CoordinationGroupKind {
    if members.iter().all(member_can_modify_noun) && head_context.is_some() {
        return CoordinationGroupKind::ModifierSeries;
    }
    if members.iter().all(member_can_be_nominal_subject) {
        return CoordinationGroupKind::NominalSubject;
    }
    if members.iter().all(member_can_be_predicate) {
        return CoordinationGroupKind::PredicateSeries;
    }
    CoordinationGroupKind::Unknown
}


fn infer_coordination_agreement_number(
    kind: CoordinationGroupKind,
    members: &[CoordinationMember<'_>],
) -> Option<crate::morph::Number> {
    if members.len() < 2 {
        return None;
    }
    match kind {
        CoordinationGroupKind::NominalSubject => Some(crate::morph::Number::Plural),
        CoordinationGroupKind::ModifierSeries | CoordinationGroupKind::PredicateSeries => member_common_number(members),
        CoordinationGroupKind::Unknown => None,
    }
}

fn member_common_number(members: &[CoordinationMember<'_>]) -> Option<crate::morph::Number> {
    let mut intersection: Option<BTreeSet<crate::morph::Number>> = None;
    for member in members {
        let values = member
            .analyses
            .iter()
            .filter_map(|analysis| analysis.features.number)
            .collect::<BTreeSet<_>>();
        if values.is_empty() {
            return None;
        }
        intersection = Some(match intersection {
            Some(current) => current.intersection(&values).copied().collect(),
            None => values,
        });
    }
    intersection.and_then(|values| {
        if values.len() == 1 {
            values.iter().next().copied()
        } else {
            None
        }
    })
}

fn shared_coordination_cases(members: &[CoordinationMember<'_>]) -> Vec<crate::morph::Case> {
    let mut intersection: Option<BTreeSet<crate::morph::Case>> = None;
    for member in members {
        let values = member
            .analyses
            .iter()
            .filter_map(|analysis| analysis.features.case)
            .collect::<BTreeSet<_>>();
        if values.is_empty() {
            return Vec::new();
        }
        intersection = Some(match intersection {
            Some(current) => current.intersection(&values).copied().collect(),
            None => values,
        });
    }
    intersection
        .unwrap_or_default()
        .into_iter()
        .collect::<Vec<_>>()
}

fn member_can_modify_noun(member: &CoordinationMember<'_>) -> bool {
    !member.analyses.is_empty() && member.analyses.iter().all(|analysis| analysis.pos.can_modify_noun())
}

fn member_can_be_nominal_subject(member: &CoordinationMember<'_>) -> bool {
    member.analyses.iter().any(|analysis| {
        matches!(analysis.pos, crate::morph::PartOfSpeech::Noun | crate::morph::PartOfSpeech::Pronoun)
            && !matches!(analysis.features.case, Some(case) if case != crate::morph::Case::Nominative)
    })
}

fn member_can_be_predicate(member: &CoordinationMember<'_>) -> bool {
    member
        .analyses
        .iter()
        .any(|analysis| matches!(analysis.pos, crate::morph::PartOfSpeech::Verb | crate::morph::PartOfSpeech::Predicative))
}

fn is_coordination_conjunction(value: &str) -> bool {
    matches!(lower_ru(value).as_str(), "и" | "или" | "либо" | "ни" | "да" | "а" | "но")
}
