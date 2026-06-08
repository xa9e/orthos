fn asyndetic_coordination_candidate(
    tokens: &[Token<'_>],
    ambiguity: &AmbiguityModel<'_>,
    left_idx: usize,
    right_idx: usize,
    islands: &SyntacticIslandMap,
) -> bool {
    if !islands.can_link_tokens(left_idx, right_idx) {
        return false;
    }
    if punctuation_marks_between(tokens, left_idx, right_idx).into_iter().any(|mark| mark != PunctuationMark::Comma) {
        return false;
    }
    let left = ambiguity.analyses_for_token(left_idx);
    let right = ambiguity.analyses_for_token(right_idx);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    modifier_coordination_pair(left, right) && following_nominal_head(tokens, ambiguity, right_idx)
}

fn modifier_coordination_pair(
    left: &[crate::morph::MorphAnalysis],
    right: &[crate::morph::MorphAnalysis],
) -> bool {
    if !left.iter().all(|analysis| analysis.pos.can_modify_noun()) {
        return false;
    }
    if !right.iter().all(|analysis| analysis.pos.can_modify_noun()) {
        return false;
    }
    shared_optional_feature(left.iter().filter_map(|a| a.features.case), right.iter().filter_map(|a| a.features.case))
        && shared_optional_feature(left.iter().filter_map(|a| a.features.number), right.iter().filter_map(|a| a.features.number))
}

fn shared_optional_feature<T>(left: impl Iterator<Item = T>, right: impl Iterator<Item = T>) -> bool
where
    T: Ord,
{
    let left = left.collect::<BTreeSet<_>>();
    let right = right.collect::<BTreeSet<_>>();
    !left.is_empty() && !right.is_empty() && left.intersection(&right).next().is_some()
}

fn following_nominal_head(
    tokens: &[Token<'_>],
    ambiguity: &AmbiguityModel<'_>,
    after: usize,
) -> bool {
    for (idx, token) in tokens
        .iter()
        .enumerate()
        .take(tokens.len().min(after + 6))
        .skip(after + 1)
    {
        if token.kind == TokenKind::Punctuation {
            return false;
        }
        if token.kind != TokenKind::Word {
            continue;
        }
        return ambiguity
            .analyses_for_token(idx)
            .iter()
            .any(|analysis| analysis.pos == crate::morph::PartOfSpeech::Noun);
    }
    false
}
