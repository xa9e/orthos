pub fn agreement_edge_proof(edge: &AgreementGraphEdge<'_>) -> DiagnosticProof {
    let kind = match edge.kind {
        AgreementGraphEdgeKind::NumeralHead => DiagnosticProofKind::QuantityConflict,
        _ => DiagnosticProofKind::AgreementConflict,
    };
    let mut proof = DiagnosticProof::new(kind, edge.confidence)
        .with_fact(DiagnosticFact::new(
            "left_token",
            edge.left.token.text,
            Some(edge.left.token.span),
        ))
        .with_fact(DiagnosticFact::new(
            "right_token",
            edge.right.token.text,
            Some(edge.right.token.span),
        ))
        .with_fact(DiagnosticFact::new(
            "edge_kind",
            format!("{:?}", edge.kind),
            Some(edge.span),
        ))
        .with_blockers(edge.blockers.clone());

    if let MorphosyntacticConstraint::Agreement(check) = &edge.constraint
        && let Some(unification) = &check.unification
    {
        proof = proof.with_fact(DiagnosticFact::new(
            "feature_unification",
            unification.summary(),
            Some(edge.span),
        ));
    }
    if let Some(conflict) = diagnostic_conflict_from_constraint(&edge.constraint, edge.span) {
        proof = proof.with_conflict(conflict);
    }
    proof
}

pub fn government_frame_proof(frame: &GovernmentFrame<'_>) -> DiagnosticProof {
    let proof_kind = if frame.kind == GovernmentFrameKind::Numeral {
        DiagnosticProofKind::QuantityConflict
    } else {
        DiagnosticProofKind::GovernmentConflict
    };
    let conflict_kind = if frame.kind == GovernmentFrameKind::Numeral {
        "quantity_government"
    } else {
        "case_government"
    };
    let mut proof = DiagnosticProof::new(proof_kind, frame.confidence)
        .with_fact(DiagnosticFact::new(
            "governor",
            frame.governor.token.text,
            Some(frame.governor.token.span),
        ))
        .with_fact(DiagnosticFact::new(
            "dependent",
            frame.dependent.token.text,
            Some(frame.dependent.token.span),
        ))
        .with_fact(DiagnosticFact::new(
            "frame_kind",
            format!("{:?}", frame.kind),
            Some(frame.span),
        ))
        .with_fact(DiagnosticFact::new(
            "frame_source",
            format!("{:?}", frame.source),
            Some(frame.span),
        ));

    if let Some(model_ref) = &frame.model_ref {
        proof = proof
            .with_fact(DiagnosticFact::new(
                "model_lemma",
                model_ref.lemma.clone(),
                Some(frame.span),
            ))
            .with_fact(DiagnosticFact::new(
                "model_complement_kind",
                format!("{:?}", model_ref.complement_kind),
                Some(frame.span),
            ))
            .with_fact(DiagnosticFact::new(
                "model_preposition",
                model_ref.preposition.clone().unwrap_or_default(),
                Some(frame.span),
            ));
        if let Some(source_id) = &model_ref.source_id {
            proof = proof.with_fact(DiagnosticFact::new(
                "model_source_id",
                source_id.to_string(),
                Some(frame.span),
            ));
        }
    }

    proof
        .with_conflict(DiagnosticConflict {
            kind: conflict_kind.to_owned(),
            expected: frame.expected_cases.iter().map(|case| format!("{case:?}")).collect(),
            observed: frame.observed_cases.iter().map(|case| format!("{case:?}")).collect(),
            span: frame.span,
        })
        .with_blockers(frame.blockers.clone())
}


pub fn punctuation_slot_proof(slot: &PunctuationSlot<'_>, expected: PunctuationMark) -> DiagnosticProof {
    let mut proof = DiagnosticProof::new(DiagnosticProofKind::BoundarySuppression, slot_confidence(slot))
        .with_fact(DiagnosticFact::new(
            "left_token",
            slot.left_token.text,
            Some(slot.left_token.span),
        ))
        .with_fact(DiagnosticFact::new(
            "right_token",
            slot.right_token.text,
            Some(slot.right_token.span),
        ))
        .with_fact(DiagnosticFact::new(
            "boundary_strength",
            format!("{:?}", slot.boundary_strength),
            Some(slot.span),
        ))
        .with_fact(DiagnosticFact::new(
            "slot_explanation",
            slot.explanation.clone(),
            Some(slot.span),
        ))
        .with_fact(DiagnosticFact::new(
            "expected_mark",
            format!("{expected:?}"),
            Some(slot.span),
        ))
        .with_blockers(slot.blockers.clone());

    for evidence in &slot.evidence {
        proof = proof.with_fact(DiagnosticFact::new(
            format!("punctuation_evidence::{:?}", evidence.kind),
            evidence.message.clone(),
            evidence.span,
        ));
    }

    if slot.missing_expected_mark(expected) {
        proof = proof.with_conflict(DiagnosticConflict {
            kind: "missing_punctuation_mark".to_owned(),
            expected: vec![format!("{expected:?}")],
            observed: slot.existing_marks.iter().map(|mark| format!("{mark:?}")).collect(),
            span: slot.span,
        });
    }

    for mark in &slot.forbidden_marks {
        proof = proof.with_suppressed_alternative(SuppressedAlternative {
            label: format!("forbidden {mark:?}"),
            reason: SuppressionReason::UnsafeBoundary,
            span: Some(slot.span),
        });
    }
    proof
}

fn slot_confidence(slot: &PunctuationSlot<'_>) -> SyntaxConfidence {
    match slot.boundary_strength {
        PunctuationBoundaryStrength::Strong => SyntaxConfidence::Strong,
        PunctuationBoundaryStrength::Medium => SyntaxConfidence::Weak,
        PunctuationBoundaryStrength::Weak | PunctuationBoundaryStrength::None => SyntaxConfidence::Weak,
        PunctuationBoundaryStrength::Unsafe => SyntaxConfidence::Ambiguous,
    }
}

fn diagnostic_conflict_from_constraint(
    constraint: &MorphosyntacticConstraint,
    span: Span,
) -> Option<DiagnosticConflict> {
    match constraint {
        MorphosyntacticConstraint::Agreement(check) => agreement_conflict(check, span),
        MorphosyntacticConstraint::Quantity(check) => quantity_conflict(check, span),
        MorphosyntacticConstraint::Government(check) => government_conflict(check, span),
        MorphosyntacticConstraint::Unknown { .. } => None,
    }
}

fn agreement_conflict(
    check: &crate::morph::AgreementCheck,
    span: Span,
) -> Option<DiagnosticConflict> {
    check.is_confident_rejection().then(|| DiagnosticConflict {
        kind: format!("{:?}", check.relation),
        expected: check.conflicts.iter().map(|conflict| conflict.left.clone()).collect(),
        observed: check.conflicts.iter().map(|conflict| conflict.right.clone()).collect(),
        span,
    })
}

fn quantity_conflict(
    check: &crate::morph::QuantityAgreementCheck,
    span: Span,
) -> Option<DiagnosticConflict> {
    check.conflict.as_ref().map(|conflict| DiagnosticConflict {
        kind: format!("{:?}", check.relation),
        expected: vec![format!("{:?}", conflict.numeral_class)],
        observed: conflict
            .observed_cases
            .iter()
            .map(|case| format!("{case:?}"))
            .chain(conflict.observed_numbers.iter().map(|number| format!("{number:?}")))
            .collect(),
        span,
    })
}

fn government_conflict(
    check: &crate::morph::GovernmentCheck,
    span: Span,
) -> Option<DiagnosticConflict> {
    check.conflict.as_ref().map(|conflict| DiagnosticConflict {
        kind: format!("{:?}", check.relation),
        expected: conflict.expected.cases.iter().map(|case| format!("{case:?}")).collect(),
        observed: conflict.observed_cases.iter().map(|case| format!("{case:?}")).collect(),
        span,
    })
}
pub fn document_abbreviation_proof(
    abbreviation: &DocumentAbbreviation,
    reason: SuppressionReason,
) -> DiagnosticProof {
    let mut proof = DiagnosticProof::new(DiagnosticProofKind::DocumentAbbreviation, SyntaxConfidence::Strong)
        .with_fact(DiagnosticFact::new(
            "abbreviation",
            abbreviation.short.clone(),
            Some(abbreviation.first_span),
        ))
        .with_fact(DiagnosticFact::new(
            "frequency",
            abbreviation.frequency.to_string(),
            Some(abbreviation.first_span),
        ));
    if let Some(expansion) = &abbreviation.expansion {
        proof = proof.with_fact(DiagnosticFact::new(
            "expansion",
            expansion.clone(),
            Some(abbreviation.first_span),
        ));
    } else {
        proof = proof.with_conflict(DiagnosticConflict {
            kind: "missing_first_mention_expansion".to_owned(),
            expected: vec!["expanded first mention".to_owned()],
            observed: vec!["bare repeated abbreviation".to_owned()],
            span: abbreviation.first_span,
        });
    }
    proof.with_suppressed_alternative(SuppressedAlternative {
        label: "known or domain-obvious abbreviation".to_owned(),
        reason,
        span: Some(abbreviation.first_span),
    })
}

pub fn document_style_proof(candidate: &DocumentStyleCandidate<'_>) -> DiagnosticProof {
    let mut proof = DiagnosticProof::new(DiagnosticProofKind::DocumentConsistency, SyntaxConfidence::Strong)
        .with_fact(DiagnosticFact::new("style_check", candidate.key, Some(candidate.span)));

    for heading in &candidate.headings {
        proof = proof.with_fact(DiagnosticFact::new(
            "heading",
            format!("{}::{:?}", heading.text, heading.style),
            Some(heading.span),
        ));
    }
    for item in &candidate.list_items {
        proof = proof.with_fact(DiagnosticFact::new(
            "list_item_marker",
            format!("{}::{:?}", item.marker, item.marker_kind),
            Some(item.span),
        ));
    }

    proof.with_conflict(DiagnosticConflict {
        kind: candidate.key.to_owned(),
        expected: vec!["consistent document style".to_owned()],
        observed: vec![candidate.key.to_owned()],
        span: candidate.span,
    })
    .with_suppressed_alternative(SuppressedAlternative {
        label: "intentional local formatting".to_owned(),
        reason: candidate.reason,
        span: Some(candidate.span),
    })
}
