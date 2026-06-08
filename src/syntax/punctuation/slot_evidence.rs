#[allow(clippy::too_many_arguments)]
fn punctuation_slot_evidence(
    span: Span,
    strength: PunctuationBoundaryStrength,
    between_clauses: bool,
    after_introductory_candidate: bool,
    in_coordination: bool,
    existing_marks: &[PunctuationMark],
    expected_marks: &[PunctuationMark],
    forbidden_marks: &[PunctuationMark],
    blockers: &[SuppressionReason],
) -> Vec<PunctuationSlotEvidence> {
    let mut evidence = vec![PunctuationSlotEvidence::new(
        PunctuationSlotEvidenceKind::Boundary,
        None,
        format!("boundary strength: {strength:?}"),
        Some(span),
    )];
    if between_clauses {
        evidence.push(PunctuationSlotEvidence::new(
            PunctuationSlotEvidenceKind::ClauseBoundary,
            Some(PunctuationMark::Comma),
            "slot lies between two clause candidates",
            Some(span),
        ));
    }
    if after_introductory_candidate {
        evidence.push(PunctuationSlotEvidence::new(
            PunctuationSlotEvidenceKind::IntroductoryCandidate,
            Some(PunctuationMark::Comma),
            "slot follows an introductory-word candidate",
            Some(span),
        ));
    }
    if in_coordination {
        evidence.push(PunctuationSlotEvidence::new(
            PunctuationSlotEvidenceKind::Coordination,
            Some(PunctuationMark::Comma),
            "slot is inside a coordination group",
            Some(span),
        ));
    }
    for mark in existing_marks {
        evidence.push(PunctuationSlotEvidence::new(
            PunctuationSlotEvidenceKind::ExistingMark,
            Some(*mark),
            format!("existing mark: {mark:?}"),
            Some(span),
        ));
    }
    for mark in expected_marks {
        evidence.push(PunctuationSlotEvidence::new(
            PunctuationSlotEvidenceKind::ExpectedMark,
            Some(*mark),
            format!("expected mark: {mark:?}"),
            Some(span),
        ));
    }
    for mark in forbidden_marks {
        evidence.push(PunctuationSlotEvidence::new(
            PunctuationSlotEvidenceKind::ForbiddenMark,
            Some(*mark),
            format!("forbidden mark: {mark:?}"),
            Some(span),
        ));
    }
    for blocker in blockers {
        evidence.push(PunctuationSlotEvidence::new(
            PunctuationSlotEvidenceKind::IslandBlocker,
            None,
            format!("blocked by {blocker:?}"),
            Some(span),
        ));
    }
    evidence
}
