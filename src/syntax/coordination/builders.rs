pub fn coordination_groups_from_facts<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    islands: &SyntacticIslandMap,
) -> Vec<CoordinationGroup<'a>> {
    let mut out = Vec::new();
    for island in islands.islands() {
        collect_coordination_groups_for_island(tokens, ambiguity, island, islands, &mut out);
    }
    out.sort_by_key(|group| (group.span.start, group.span.end, group.id));
    for (id, group) in out.iter_mut().enumerate() {
        group.id = id;
    }
    out
}

fn collect_coordination_groups_for_island<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    island: &SyntacticIsland,
    islands: &SyntacticIslandMap,
    out: &mut Vec<CoordinationGroup<'a>>,
) {
    let Some(mut idx) = island.start_token else { return; };
    let Some(end) = island.end_token else { return; };

    while idx <= end {
        let Some(first_member) = next_member_token(tokens, idx, end) else { break; };
        let mut members = vec![member_for_token(tokens, ambiguity, first_member)];
        let mut connectors = Vec::new();
        let mut cursor = first_member;

        while let Some((connector, member_idx)) = next_coordination_link(tokens, cursor, end) {
            connectors.push(connector);
            members.push(member_for_token(tokens, ambiguity, member_idx));
            cursor = member_idx;
        }

        if members.len() > 1 {
            let blockers = coordination_blockers(&members, island, islands);
            let confidence = if blockers.is_empty() {
                SyntaxConfidence::Strong
            } else {
                SyntaxConfidence::Ambiguous
            };
            let head_context = following_word_token(tokens, cursor, end);
            let kind = infer_coordination_kind(&members, head_context.as_ref());
            let agreement_number = infer_coordination_agreement_number(kind, &members);
            let shared_cases = shared_coordination_cases(&members);
            let (last_token_index, last_token_end) = members
                .last()
                .map(|member| (member.token_index, member.token.span.end))
                .unwrap_or((members[0].token_index, members[0].token.span.end));
            out.push(CoordinationGroup {
                id: out.len(),
                kind,
                span: Span::new(members[0].token.span.start, last_token_end),
                start_token: members[0].token_index,
                end_token: last_token_index,
                island_id: Some(island.id),
                members,
                connectors,
                head_context,
                agreement_number,
                shared_cases,
                confidence,
                blockers,
            });
            idx = cursor.saturating_add(1);
        } else {
            idx = first_member.saturating_add(1);
        }
    }
}

fn next_member_token(tokens: &[Token<'_>], start: usize, end: usize) -> Option<usize> {
    (start..=end).find(|idx| {
        tokens
            .get(*idx)
            .is_some_and(|token| token.kind == TokenKind::Word && !is_coordination_conjunction(token.text))
    })
}

fn next_coordination_link<'a>(
    tokens: &[Token<'a>],
    current_member: usize,
    end: usize,
) -> Option<(CoordinationConnector<'a>, usize)> {
    let connector_idx = next_non_ws_index(tokens, current_member + 1, end)?;
    let connector_token = tokens.get(connector_idx)?;
    let connector_kind = if connector_token.kind == TokenKind::Punctuation && connector_token.text == "," {
        CoordinationConnectorKind::Comma
    } else if connector_token.kind == TokenKind::Word && is_coordination_conjunction(connector_token.text) {
        repeated_or_single_conjunction(tokens, connector_idx, current_member)
    } else {
        return None;
    };
    let member_idx = next_non_ws_index(tokens, connector_idx + 1, end)?;
    let member_token = tokens.get(member_idx)?;
    (member_token.kind == TokenKind::Word && !is_coordination_conjunction(member_token.text)).then(|| {
        (
            CoordinationConnector {
                kind: connector_kind,
                token_index: connector_idx,
                token: connector_token.clone(),
            },
            member_idx,
        )
    })
}

fn repeated_or_single_conjunction(
    tokens: &[Token<'_>],
    connector_idx: usize,
    current_member: usize,
) -> CoordinationConnectorKind {
    let prev_connector = previous_non_ws_index(tokens, current_member)
        .and_then(|idx| tokens.get(idx))
        .is_some_and(|token| token.kind == TokenKind::Word && is_coordination_conjunction(token.text));
    if prev_connector || lower_ru(tokens[connector_idx].text) == "ни" {
        CoordinationConnectorKind::RepeatedConjunction
    } else {
        CoordinationConnectorKind::SingleConjunction
    }
}

fn next_non_ws_index(tokens: &[Token<'_>], start: usize, end: usize) -> Option<usize> {
    (start..=end).find(|idx| tokens.get(*idx).is_some_and(|token| token.kind != TokenKind::Whitespace))
}

fn previous_non_ws_index(tokens: &[Token<'_>], before: usize) -> Option<usize> {
    tokens[..before]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, token)| token.kind != TokenKind::Whitespace)
        .map(|(idx, _)| idx)
}

fn following_word_token<'a>(tokens: &[Token<'a>], after: usize, end: usize) -> Option<Token<'a>> {
    ((after + 1)..=end)
        .find_map(|idx| tokens.get(idx).filter(|token| token.kind == TokenKind::Word).cloned())
}

fn member_for_token<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    token_index: usize,
) -> CoordinationMember<'a> {
    CoordinationMember {
        token_index,
        token: tokens[token_index].clone(),
        analyses: ambiguity.analyses_for_token(token_index).to_vec(),
    }
}

fn coordination_blockers(
    members: &[CoordinationMember<'_>],
    island: &SyntacticIsland,
    islands: &SyntacticIslandMap,
) -> Vec<SuppressionReason> {
    let mut blockers = island.blockers.clone();
    for pair in members.windows(2) {
        if !islands.can_link_tokens(pair[0].token_index, pair[1].token_index) {
            blockers.push(SuppressionReason::UnsafeBoundary);
        }
    }
    blockers.sort_unstable();
    blockers.dedup();
    blockers
}
