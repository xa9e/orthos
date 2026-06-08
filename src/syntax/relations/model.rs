#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SyntaxRelationKind {
    ModifierHead,
    SubjectPredicate,
    PrepositionGovernment,
    PrepositionNominalGroup,
    NumeralNoun,
    NumeralNominalGroup,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SyntaxRelationBlocker {
    PunctuationBetween,
    SentenceBoundaryBetween,
    NewlineBetween,
    AmbiguousMorphology,
    UnknownMorphology,
    Coordination,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SyntaxRelationCandidate<'a> {
    pub kind: SyntaxRelationKind,
    pub left: Token<'a>,
    pub right: Token<'a>,
    pub span: Span,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SyntaxRelationBlocker>,
}

impl<'a> SyntaxRelationCandidate<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty()
    }
}

pub fn preposition_government_candidates<'a>(
    tokens: &[Token<'a>],
    prepositions: &BTreeSet<String>,
) -> Vec<SyntaxRelationCandidate<'a>> {
    let is_known_preposition = |token: &Token<'_>| prepositions.contains(&lower_ru(token.text));
    adjacent_filtered_word_relation_candidates(
        tokens,
        SyntaxRelationKind::PrepositionGovernment,
        is_known_preposition,
    )
}

pub fn preposition_nominal_group_candidates<'a>(
    tokens: &[Token<'a>],
    prepositions: &BTreeSet<String>,
) -> Vec<SyntaxRelationCandidate<'a>> {
    preposition_governed_nominal_group_candidates(tokens, prepositions, 3)
        .into_iter()
        .map(|candidate| governed_group_relation_candidate(
            SyntaxRelationKind::PrepositionNominalGroup,
            candidate.governor,
            candidate.group.head,
            candidate.span,
            candidate.confidence,
            candidate.blockers,
        ))
        .collect()
}

pub fn adjacent_numeral_noun_candidates<'a>(
    tokens: &[Token<'a>],
) -> Vec<SyntaxRelationCandidate<'a>> {
    adjacent_word_relation_candidates(tokens, SyntaxRelationKind::NumeralNoun)
}

pub fn numeral_nominal_group_candidates<'a>(
    tokens: &[Token<'a>],
) -> Vec<SyntaxRelationCandidate<'a>> {
    numeral_governed_nominal_group_candidates(tokens, 3)
        .into_iter()
        .map(|candidate| governed_group_relation_candidate(
            SyntaxRelationKind::NumeralNominalGroup,
            candidate.governor,
            candidate.group.head,
            candidate.span,
            candidate.confidence,
            candidate.blockers,
        ))
        .collect()
}

pub fn adjacent_subject_predicate_candidates<'a>(
    tokens: &[Token<'a>],
) -> Vec<SyntaxRelationCandidate<'a>> {
    adjacent_word_relation_candidates(tokens, SyntaxRelationKind::SubjectPredicate)
}

pub fn adjacent_modifier_head_candidates<'a>(
    tokens: &[Token<'a>],
) -> Vec<SyntaxRelationCandidate<'a>> {
    adjacent_word_relation_candidates(tokens, SyntaxRelationKind::ModifierHead)
}

fn adjacent_word_relation_candidates<'a>(
    tokens: &[Token<'a>],
    kind: SyntaxRelationKind,
) -> Vec<SyntaxRelationCandidate<'a>> {
    adjacent_filtered_word_relation_candidates(tokens, kind, |_| true)
}

fn adjacent_filtered_word_relation_candidates<'a>(
    tokens: &[Token<'a>],
    kind: SyntaxRelationKind,
    left_filter: impl Fn(&Token<'_>) -> bool,
) -> Vec<SyntaxRelationCandidate<'a>> {
    let mut out = Vec::new();

    for window in tokens.windows(3) {
        let [left, gap, right] = window else {
            continue;
        };
        if left.kind != TokenKind::Word
            || right.kind != TokenKind::Word
            || gap.kind != TokenKind::Whitespace
            || !left_filter(left)
        {
            continue;
        }
        let blockers = whitespace_blockers(gap.text);
        out.push(relation_candidate(kind, left, right, blockers));
    }

    out
}

#[allow(dead_code)]
fn three_word_relation_candidates<'a>(
    tokens: &[Token<'a>],
    kind: SyntaxRelationKind,
    left_filter: impl Fn(&Token<'_>) -> bool,
) -> Vec<SyntaxRelationCandidate<'a>> {
    let mut out = Vec::new();

    for window in tokens.windows(5) {
        let [left, first_gap, middle, second_gap, right] = window else {
            continue;
        };
        if left.kind != TokenKind::Word
            || first_gap.kind != TokenKind::Whitespace
            || middle.kind != TokenKind::Word
            || second_gap.kind != TokenKind::Whitespace
            || right.kind != TokenKind::Word
            || !left_filter(left)
        {
            continue;
        }
        let blockers = [first_gap.text, second_gap.text]
            .into_iter()
            .flat_map(whitespace_blockers)
            .collect::<Vec<_>>();
        out.push(relation_candidate(kind, left, right, blockers));
    }

    out
}


fn governed_group_relation_candidate<'a>(
    kind: SyntaxRelationKind,
    left: Token<'a>,
    right: Token<'a>,
    span: Span,
    confidence: SyntaxConfidence,
    blockers: Vec<SyntaxRelationBlocker>,
) -> SyntaxRelationCandidate<'a> {
    SyntaxRelationCandidate {
        kind,
        left,
        right,
        span,
        confidence,
        blockers,
    }
}

fn relation_candidate<'a>(
    kind: SyntaxRelationKind,
    left: &Token<'a>,
    right: &Token<'a>,
    blockers: Vec<SyntaxRelationBlocker>,
) -> SyntaxRelationCandidate<'a> {
    let confidence = if blockers.is_empty() {
        SyntaxConfidence::Strong
    } else {
        SyntaxConfidence::Ambiguous
    };
    SyntaxRelationCandidate {
        kind,
        left: left.clone(),
        right: right.clone(),
        span: Span::new(left.span.start, right.span.end),
        confidence,
        blockers,
    }
}

fn whitespace_blockers(value: &str) -> Vec<SyntaxRelationBlocker> {
    let mut blockers = Vec::new();
    if value.contains('\n') || value.contains('\r') {
        blockers.push(SyntaxRelationBlocker::NewlineBetween);
    }
    blockers
}
