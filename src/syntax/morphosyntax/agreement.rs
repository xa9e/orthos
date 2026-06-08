fn agreement_check_for_analyses(
    relation: crate::morph::AgreementRelationKind,
    left_analyses: &[crate::morph::MorphAnalysis],
    right_analyses: &[crate::morph::MorphAnalysis],
) -> crate::morph::AgreementCheck {
    if left_analyses.is_empty() || right_analyses.is_empty() {
        return agreement_unknown(relation);
    }
    crate::morph::feature_unification_agreement_check(relation, left_analyses, right_analyses)
}

fn agreement_unknown(relation: crate::morph::AgreementRelationKind) -> crate::morph::AgreementCheck {
    crate::morph::AgreementCheck {
        relation,
        compatibility: crate::morph::MorphCompatibility::Unknown,
        conflicts: Vec::new(),
        unknown_features: Vec::new(),
        unification: None,
    }
}
