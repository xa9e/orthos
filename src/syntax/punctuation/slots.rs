pub fn punctuation_slots_from_facts<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    islands: &SyntacticIslandMap,
    clauses: &[ClauseCandidate<'a>],
    coordination_groups: &[CoordinationGroup<'a>],
) -> Vec<PunctuationSlot<'a>> {
    let word_indices = tokens
        .iter()
        .enumerate()
        .filter_map(|(idx, token)| (token.kind == TokenKind::Word).then_some(idx))
        .collect::<Vec<_>>();
    let mut out = Vec::new();
    for pair in word_indices.windows(2) {
        let left_idx = pair[0];
        let right_idx = pair[1];
        if let Some(slot) = punctuation_slot_for_pair(
            out.len(),
            tokens,
            ambiguity,
            islands,
            clauses,
            coordination_groups,
            left_idx,
            right_idx,
        ) {
            out.push(slot);
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn punctuation_slot_for_pair<'a>(
    id: usize,
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    islands: &SyntacticIslandMap,
    clauses: &[ClauseCandidate<'a>],
    coordination_groups: &[CoordinationGroup<'a>],
    left_idx: usize,
    right_idx: usize,
) -> Option<PunctuationSlot<'a>> {
    let left = tokens.get(left_idx)?.clone();
    let right = tokens.get(right_idx)?.clone();
    let span = Span::new(left.span.end, right.span.start);
    let left_island = islands.island_for_token(left_idx);
    let right_island = islands.island_for_token(right_idx);
    let inside_quotes = token_pair_has_island_kind(left_island, right_island, SyntacticIslandKind::DirectSpeech);
    let inside_parentheses = token_pair_has_island_kind(left_island, right_island, SyntacticIslandKind::Parenthetical);
    let mut blockers = slot_blockers(left_idx, right_idx, islands);
    let between_clauses = between_clause_candidates(left_idx, right_idx, clauses, islands);
    let after_introductory_candidate = is_introductory_candidate(left.text);
    let existing_marks = punctuation_marks_between(tokens, left_idx, right_idx);
    let in_coordination = coordination_groups.iter().any(|group| {
        group.members.windows(2).any(|pair| {
            pair[0].token_index == left_idx && pair[1].token_index == right_idx
        })
    });
    let coordination_candidate = in_coordination
        || asyndetic_coordination_candidate(tokens, ambiguity, left_idx, right_idx, islands);

    if inside_quotes {
        blockers.push(SuppressionReason::DirectSpeechBoundary);
    }
    if inside_parentheses {
        blockers.push(SuppressionReason::ParenthesisBoundary);
    }
    blockers.sort_unstable();
    blockers.dedup();

    let mut expected_marks = Vec::new();
    if between_clauses || after_introductory_candidate || coordination_candidate {
        expected_marks.push(PunctuationMark::Comma);
    }
    if subject_predicate_dash_candidate(left.text, right.text) {
        expected_marks.push(PunctuationMark::Dash);
    }
    expected_marks.sort_unstable();
    expected_marks.dedup();

    let forbidden_marks = forbidden_marks_for_slot(&blockers, &existing_marks, coordination_candidate);
    let boundary_strength = boundary_strength_for_slot(&blockers, between_clauses, after_introductory_candidate, coordination_candidate);
    let explanation = slot_explanation(boundary_strength, between_clauses, after_introductory_candidate, coordination_candidate);
    let evidence = punctuation_slot_evidence(
        span,
        boundary_strength,
        between_clauses,
        after_introductory_candidate,
        coordination_candidate,
        &existing_marks,
        &expected_marks,
        &forbidden_marks,
        &blockers,
    );

    Some(PunctuationSlot {
        id,
        span,
        left_token_index: left_idx,
        right_token_index: right_idx,
        left_token: left,
        right_token: right,
        boundary_strength,
        inside_quotes,
        inside_parentheses,
        between_clauses,
        after_introductory_candidate,
        existing_marks,
        expected_marks,
        forbidden_marks,
        explanation,
        evidence,
        blockers,
    })
}

fn token_pair_has_island_kind(
    left: Option<&SyntacticIsland>,
    right: Option<&SyntacticIsland>,
    kind: SyntacticIslandKind,
) -> bool {
    left.is_some_and(|island| island.kind == kind) || right.is_some_and(|island| island.kind == kind)
}

fn slot_blockers(left_idx: usize, right_idx: usize, islands: &SyntacticIslandMap) -> Vec<SuppressionReason> {
    if islands.can_link_tokens(left_idx, right_idx) {
        Vec::new()
    } else {
        vec![SuppressionReason::UnsafeBoundary]
    }
}

fn between_clause_candidates(
    left_idx: usize,
    right_idx: usize,
    clauses: &[ClauseCandidate<'_>],
    islands: &SyntacticIslandMap,
) -> bool {
    if !islands.can_link_tokens(left_idx, right_idx) {
        return false;
    }
    let left_clause = clauses.iter().find(|clause| clause_contains_token(clause, left_idx));
    let right_clause = clauses.iter().find(|clause| clause_contains_token(clause, right_idx));
    match (left_clause, right_clause) {
        (Some(left), Some(right)) => left.id != right.id,
        _ => false,
    }
}

fn clause_contains_token(clause: &ClauseCandidate<'_>, token_index: usize) -> bool {
    matches!((clause.start_token, clause.end_token), (Some(start), Some(end)) if start <= token_index && token_index <= end)
}

fn punctuation_marks_between(tokens: &[Token<'_>], left_idx: usize, right_idx: usize) -> Vec<PunctuationMark> {
    let mut marks = tokens[(left_idx + 1)..right_idx]
        .iter()
        .filter(|token| token.kind == TokenKind::Punctuation)
        .filter_map(|token| punctuation_mark_from_text(token.text))
        .collect::<Vec<_>>();
    marks.sort_unstable();
    marks.dedup();
    marks
}

fn punctuation_mark_from_text(value: &str) -> Option<PunctuationMark> {
    Some(match value {
        "," => PunctuationMark::Comma,
        "—" | "-" | "–" => PunctuationMark::Dash,
        ":" => PunctuationMark::Colon,
        ";" => PunctuationMark::Semicolon,
        "." => PunctuationMark::Period,
        _ => return None,
    })
}

fn forbidden_marks_for_slot(
    blockers: &[SuppressionReason],
    existing_marks: &[PunctuationMark],
    in_coordination: bool,
) -> Vec<PunctuationMark> {
    let mut forbidden = Vec::new();
    if !blockers.is_empty() {
        forbidden.extend([PunctuationMark::Comma, PunctuationMark::Dash, PunctuationMark::Colon]);
    }
    if in_coordination && existing_marks.contains(&PunctuationMark::Comma) {
        forbidden.push(PunctuationMark::Colon);
    }
    forbidden.sort_unstable();
    forbidden.dedup();
    forbidden
}

fn boundary_strength_for_slot(
    blockers: &[SuppressionReason],
    between_clauses: bool,
    after_introductory_candidate: bool,
    in_coordination: bool,
) -> PunctuationBoundaryStrength {
    if !blockers.is_empty() {
        PunctuationBoundaryStrength::Unsafe
    } else if between_clauses || after_introductory_candidate || in_coordination {
        PunctuationBoundaryStrength::Strong
    } else {
        PunctuationBoundaryStrength::Weak
    }
}

fn slot_explanation(
    strength: PunctuationBoundaryStrength,
    between_clauses: bool,
    after_introductory_candidate: bool,
    in_coordination: bool,
) -> String {
    if between_clauses {
        "slot is between clause candidates".to_owned()
    } else if after_introductory_candidate {
        "slot follows an introductory-word candidate".to_owned()
    } else if in_coordination {
        "slot is inside a coordination group".to_owned()
    } else {
        format!("slot has {strength:?} boundary evidence")
    }
}

fn is_introductory_candidate(value: &str) -> bool {
    matches!(
        lower_ru(value).as_str(),
        "безусловно"
            | "вероятно"
            | "возможно"
            | "итак"
            | "кстати"
            | "конечно"
            | "например"
            | "однако"
            | "следовательно"
            | "значит"
            | "по-видимому"
            | "во-первых"
            | "во-вторых"
    )
}

fn subject_predicate_dash_candidate(left: &str, right: &str) -> bool {
    left.chars().next().is_some_and(|ch| ch.is_uppercase()) && right.chars().next().is_some_and(|ch| ch.is_uppercase())
}
