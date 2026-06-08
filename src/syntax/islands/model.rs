#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SyntacticIslandKind {
    PlainSentence,
    Parenthetical,
    DirectSpeech,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SyntacticIsland {
    pub id: usize,
    pub span: Span,
    pub start_token: Option<usize>,
    pub end_token: Option<usize>,
    pub kind: SyntacticIslandKind,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SuppressionReason>,
}

impl SyntacticIsland {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty()
    }

    pub fn contains_token(&self, token_index: usize) -> bool {
        matches!((self.start_token, self.end_token), (Some(start), Some(end)) if start <= token_index && token_index <= end)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SyntacticIslandMap {
    islands: Vec<SyntacticIsland>,
}

impl SyntacticIslandMap {
    pub fn from_text_tokens(text: &str, tokens: &[Token<'_>]) -> Self {
        let mut islands = Vec::new();
        let zones = punctuation_safe_zone_records(text);

        for sentence in sentence_spans(text) {
            push_sentence_islands(text, tokens, sentence, &zones, &mut islands);
        }

        islands.sort_by_key(|island| (island.span.start, island.span.end, island.id));
        for (id, island) in islands.iter_mut().enumerate() {
            island.id = id;
        }
        Self { islands }
    }

    pub fn islands(&self) -> &[SyntacticIsland] {
        &self.islands
    }

    pub fn island_for_token(&self, token_index: usize) -> Option<&SyntacticIsland> {
        self.islands.iter().find(|island| island.contains_token(token_index))
    }

    pub fn can_link_tokens(&self, left_token: usize, right_token: usize) -> bool {
        let Some(left) = self.island_for_token(left_token) else {
            return false;
        };
        let Some(right) = self.island_for_token(right_token) else {
            return false;
        };
        left.id == right.id && left.is_actionable() && right.is_actionable()
    }
}

fn push_sentence_islands(
    text: &str,
    tokens: &[Token<'_>],
    sentence: SentenceSpan,
    zones: &[PunctuationSafeZone],
    out: &mut Vec<SyntacticIsland>,
) {
    let mut cursor = sentence.start;
    for zone in zones
        .iter()
        .filter(|zone| zone.span.start < sentence.end && zone.span.end > sentence.start)
    {
        let safe_zone_start = zone.span.start.max(sentence.start);
        let safe_zone_end = zone.span.end.min(sentence.end);
        push_plain_island(tokens, cursor, safe_zone_start, out);
        push_safe_zone_island(tokens, Span::new(safe_zone_start, safe_zone_end), *zone, out);
        cursor = cursor.max(safe_zone_end);
    }
    push_plain_island(tokens, cursor, sentence.end, out);

    let _ = text;
}

fn push_plain_island(tokens: &[Token<'_>], start: usize, end: usize, out: &mut Vec<SyntacticIsland>) {
    if start >= end || !span_has_word_token(tokens, start, end) {
        return;
    }
    let (start_token, end_token) = token_range_for_span(tokens, start, end);
    out.push(SyntacticIsland {
        id: out.len(),
        span: Span::new(start, end),
        start_token,
        end_token,
        kind: SyntacticIslandKind::PlainSentence,
        confidence: SyntaxConfidence::Strong,
        blockers: Vec::new(),
    });
}

fn push_safe_zone_island(
    tokens: &[Token<'_>],
    span: Span,
    zone: PunctuationSafeZone,
    out: &mut Vec<SyntacticIsland>,
) {
    if span.start >= span.end || !span_has_word_token(tokens, span.start, span.end) {
        return;
    }
    let (start_token, end_token) = token_range_for_span(tokens, span.start, span.end);
    let (kind, blocker) = match zone.kind {
        PunctuationSafeZoneKind::Parenthetical => {
            (SyntacticIslandKind::Parenthetical, SuppressionReason::ParenthesisBoundary)
        }
        PunctuationSafeZoneKind::DirectSpeech => {
            (SyntacticIslandKind::DirectSpeech, SuppressionReason::DirectSpeechBoundary)
        }
    };
    out.push(SyntacticIsland {
        id: out.len(),
        span,
        start_token,
        end_token,
        kind,
        confidence: zone.confidence,
        blockers: vec![blocker],
    });
}

fn span_has_word_token(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    tokens.iter().any(|token| {
        token.kind == TokenKind::Word && token.span.end > start && token.span.start < end
    })
}
