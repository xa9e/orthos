#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NominalGroupCandidate<'a> {
    pub start_token: usize,
    pub end_token: usize,
    pub modifiers: Vec<Token<'a>>,
    pub head: Token<'a>,
    pub span: Span,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SyntaxRelationBlocker>,
}

impl<'a> NominalGroupCandidate<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum NominalGroupGovernorKind {
    Preposition,
    Numeral,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GovernedNominalGroupCandidate<'a> {
    pub kind: NominalGroupGovernorKind,
    pub governor_token: usize,
    pub governor: Token<'a>,
    pub group: NominalGroupCandidate<'a>,
    pub span: Span,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SyntaxRelationBlocker>,
}

impl<'a> GovernedNominalGroupCandidate<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty() && self.group.is_actionable()
    }
}

pub fn short_nominal_group_candidates<'a>(
    tokens: &[Token<'a>],
    max_modifiers: usize,
) -> Vec<NominalGroupCandidate<'a>> {
    let mut out = Vec::new();
    for start in 0..tokens.len() {
        if tokens[start].kind != TokenKind::Word {
            continue;
        }
        for modifier_count in 1..=max_modifiers {
            if let Some(group) = nominal_group_from_start(tokens, start, modifier_count) {
                out.push(group);
            }
        }
    }
    out
}

pub fn preposition_governed_nominal_group_candidates<'a>(
    tokens: &[Token<'a>],
    prepositions: &BTreeSet<String>,
    max_modifiers: usize,
) -> Vec<GovernedNominalGroupCandidate<'a>> {
    governed_nominal_group_candidates(
        tokens,
        NominalGroupGovernorKind::Preposition,
        |token| prepositions.contains(&lower_ru(token.text)),
        max_modifiers,
    )
}

pub fn numeral_governed_nominal_group_candidates<'a>(
    tokens: &[Token<'a>],
    max_modifiers: usize,
) -> Vec<GovernedNominalGroupCandidate<'a>> {
    governed_nominal_group_candidates(
        tokens,
        NominalGroupGovernorKind::Numeral,
        |token| token.kind == TokenKind::Word,
        max_modifiers,
    )
}

pub fn nominal_group_roles_are_plausible(
    group: &NominalGroupCandidate<'_>,
    analyses_for_token: &impl Fn(usize) -> Vec<crate::morph::MorphAnalysis>,
) -> bool {
    analyses_can_head_nominal_group(&analyses_for_token(group.end_token))
        && group
            .modifiers
            .iter()
            .enumerate()
            .all(|(offset, _)| analyses_can_modify_nominal_group(&analyses_for_token(group.start_token + offset * 2)))
}

fn analyses_can_head_nominal_group(analyses: &[crate::morph::MorphAnalysis]) -> bool {
    !analyses.is_empty()
        && analyses.iter().all(|analysis| {
            matches!(
                analysis.pos,
                crate::morph::PartOfSpeech::Noun
                    | crate::morph::PartOfSpeech::Pronoun
                    | crate::morph::PartOfSpeech::Numeral
            )
        })
}

fn analyses_can_modify_nominal_group(analyses: &[crate::morph::MorphAnalysis]) -> bool {
    !analyses.is_empty()
        && analyses.iter().all(|analysis| {
            matches!(
                analysis.pos,
                crate::morph::PartOfSpeech::Adjective
                    | crate::morph::PartOfSpeech::Participle
                    | crate::morph::PartOfSpeech::Numeral
            )
        })
}

fn governed_nominal_group_candidates<'a>(
    tokens: &[Token<'a>],
    kind: NominalGroupGovernorKind,
    governor_filter: impl Fn(&Token<'_>) -> bool,
    max_modifiers: usize,
) -> Vec<GovernedNominalGroupCandidate<'a>> {
    let mut out = Vec::new();
    for governor_index in 0..tokens.len() {
        let governor = &tokens[governor_index];
        if governor.kind != TokenKind::Word || !governor_filter(governor) {
            continue;
        }
        let Some(group_start) = following_word_after_plain_gap(tokens, governor_index) else {
            continue;
        };
        for modifier_count in 0..=max_modifiers {
            let Some(group) = nominal_group_from_start(tokens, group_start, modifier_count) else {
                continue;
            };
            let blockers = merge_blockers(gap_blockers_after(tokens, governor_index), group.blockers.clone());
            let confidence = confidence_from_blockers(&blockers);
            out.push(GovernedNominalGroupCandidate {
                kind,
                governor_token: governor_index,
                governor: governor.clone(),
                span: Span::new(governor.span.start, group.span.end),
                group,
                confidence,
                blockers,
            });
        }
    }
    out
}

fn nominal_group_from_start<'a>(
    tokens: &[Token<'a>],
    start: usize,
    modifier_count: usize,
) -> Option<NominalGroupCandidate<'a>> {
    let mut modifiers = Vec::with_capacity(modifier_count);
    let mut blockers = Vec::new();
    let mut current = start;

    for _ in 0..modifier_count {
        if tokens.get(current)?.kind != TokenKind::Word {
            return None;
        }
        modifiers.push(tokens[current].clone());
        blockers.extend(gap_blockers_after(tokens, current));
        current = following_word_after_plain_gap(tokens, current)?;
    }

    let head = tokens.get(current)?;
    if head.kind != TokenKind::Word {
        return None;
    }

    let span = Span::new(tokens[start].span.start, head.span.end);
    Some(NominalGroupCandidate {
        start_token: start,
        end_token: current,
        modifiers,
        head: head.clone(),
        span,
        confidence: confidence_from_blockers(&blockers),
        blockers,
    })
}

fn following_word_after_plain_gap(tokens: &[Token<'_>], index: usize) -> Option<usize> {
    let gap_index = index.checked_add(1)?;
    let word_index = index.checked_add(2)?;
    if tokens.get(gap_index)?.kind != TokenKind::Whitespace {
        return None;
    }
    if tokens.get(word_index)?.kind != TokenKind::Word {
        return None;
    }
    Some(word_index)
}

fn gap_blockers_after(tokens: &[Token<'_>], index: usize) -> Vec<SyntaxRelationBlocker> {
    tokens
        .get(index + 1)
        .filter(|token| token.kind == TokenKind::Whitespace)
        .map(|token| whitespace_blockers(token.text))
        .unwrap_or_default()
}

fn merge_blockers(
    mut left: Vec<SyntaxRelationBlocker>,
    right: Vec<SyntaxRelationBlocker>,
) -> Vec<SyntaxRelationBlocker> {
    left.extend(right);
    left.sort_unstable();
    left.dedup();
    left
}

fn confidence_from_blockers(blockers: &[SyntaxRelationBlocker]) -> SyntaxConfidence {
    if blockers.is_empty() {
        SyntaxConfidence::Strong
    } else {
        SyntaxConfidence::Ambiguous
    }
}
