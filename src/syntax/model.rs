#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SyntaxConfidence {
    Certain,
    Strong,
    Weak,
    Ambiguous,
}

impl SyntaxConfidence {
    pub fn is_actionable(self) -> bool {
        matches!(self, Self::Certain | Self::Strong)
    }

    pub fn min(self, other: Self) -> Self {
        if self <= other {
            self
        } else {
            other
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SentenceSpan {
    pub start: usize,
    pub end: usize,
}

impl SentenceSpan {
    pub fn as_span(self) -> Span {
        Span::new(self.start, self.end)
    }

    pub fn contains(self, byte_index: usize) -> bool {
        self.start <= byte_index && byte_index < self.end
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SyntaxDocument<'text> {
    text: &'text str,
    tokens: Vec<Token<'text>>,
    sentences: Vec<SentenceSyntax>,
    parentheticals: Vec<ParentheticalSpan>,
    direct_speech: Vec<DirectSpeechSpan>,
    safe_zones: Vec<PunctuationSafeZone>,
}

impl<'text> SyntaxDocument<'text> {
    pub fn new(text: &'text str) -> Self {
        let marker_set = HashSet::new();
        Self::with_clause_markers(text, &marker_set)
    }

    pub fn with_clause_markers(text: &'text str, marker_set: &HashSet<String>) -> Self {
        let tokens = tokenize(text);
        let parentheticals = parenthetical_spans(text);
        let direct_speech = direct_speech_spans(text);
        let safe_zones = punctuation_safe_zone_records(text);
        let sentence_spans = sentence_spans(text);

        let sentences = sentence_spans
            .into_iter()
            .map(|span| {
                let (start_token, end_token) = token_range_for_span(&tokens, span.start, span.end);
                let boundaries = clause_boundaries_for_sentence(text, &tokens, marker_set, span);
                let edges = boundaries.iter().map(|boundary| marker_dependency_edge(&boundary.marker)).collect();
                let fragments = boundaries
                    .iter()
                    .filter_map(|boundary| {
                        syntax_span_for_tokens(
                            &tokens,
                            boundary.marker.start_token,
                            boundary.marker.end_token,
                            SyntaxSpanKind::ClauseFragment,
                            boundary.confidence,
                        )
                    })
                    .collect();

                SentenceSyntax {
                    span,
                    start_token,
                    end_token,
                    clause_graph: ClauseGraph {
                        boundaries,
                        edges,
                        fragments,
                    },
                    confidence: SyntaxConfidence::Certain,
                }
            })
            .collect();

        Self {
            text,
            tokens,
            sentences,
            parentheticals,
            direct_speech,
            safe_zones,
        }
    }

    pub fn text(&self) -> &'text str {
        self.text
    }

    pub fn tokens(&self) -> &[Token<'text>] {
        &self.tokens
    }

    pub fn sentences(&self) -> &[SentenceSyntax] {
        &self.sentences
    }

    pub fn parentheticals(&self) -> &[ParentheticalSpan] {
        &self.parentheticals
    }

    pub fn direct_speech(&self) -> &[DirectSpeechSpan] {
        &self.direct_speech
    }

    pub fn safe_zones(&self) -> &[PunctuationSafeZone] {
        &self.safe_zones
    }

    pub fn sentence_at(&self, byte_index: usize) -> Option<&SentenceSyntax> {
        self.sentences.iter().find(|sentence| sentence.span.contains(byte_index))
    }

    pub fn clause_boundaries(&self) -> impl Iterator<Item = &ClauseBoundary> {
        self.sentences
            .iter()
            .flat_map(|sentence| sentence.clause_graph.boundaries.iter())
    }

    pub fn is_inside_punctuation_safe_zone(&self, byte_index: usize) -> bool {
        self.safe_zones
            .iter()
            .any(|zone| zone.span.start < byte_index && byte_index < zone.span.end)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SentenceSyntax {
    pub span: SentenceSpan,
    pub start_token: Option<usize>,
    pub end_token: Option<usize>,
    pub clause_graph: ClauseGraph,
    pub confidence: SyntaxConfidence,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ClauseGraph {
    pub boundaries: Vec<ClauseBoundary>,
    pub edges: Vec<DependencyEdge>,
    pub fragments: Vec<SyntaxSpan>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct TokenWindow<'t, 's> {
    pub previous: Option<&'s Token<'t>>,
    pub current: &'s Token<'t>,
    pub next: Option<&'s Token<'t>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PunctuationContext {
    pub has_comma_before: bool,
    pub preceded_by_sentence_terminal: bool,
    pub preceded_by_opening_delimiter: bool,
    pub preceded_by_clause_separator: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClauseMarkerKind {
    Subordinator,
    MultiwordSubordinator,
    RelativePronoun,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClauseMarker {
    pub start_token: usize,
    pub end_token: usize,
    pub span: Span,
    pub canonical: String,
    pub kind: ClauseMarkerKind,
    pub confidence: SyntaxConfidence,
}

pub type ClauseMarkerMatch = ClauseMarker;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClauseBoundaryKind {
    BeforeMarker,
    SentenceStartMarker,
    PunctuatedBeforeMarker,
    SuppressedSafeZone,
    Ambiguous,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClauseBoundary {
    pub marker: ClauseMarker,
    pub boundary_span: Span,
    pub kind: ClauseBoundaryKind,
    pub confidence: SyntaxConfidence,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DependencyRelation {
    MarkerIntroducesClause,
    QuoteContainsSpeech,
    ParentheticalContainsFragment,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DependencyEdge {
    pub head_token: usize,
    pub dependent_token: usize,
    pub relation: DependencyRelation,
    pub confidence: SyntaxConfidence,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SyntaxSpanKind {
    Sentence,
    ClauseFragment,
    Parenthetical,
    DirectSpeech,
    PunctuationSafeZone,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SyntaxSpan {
    pub span: Span,
    pub start_token: Option<usize>,
    pub end_token: Option<usize>,
    pub kind: SyntaxSpanKind,
    pub confidence: SyntaxConfidence,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PunctuationSafeZoneKind {
    Parenthetical,
    DirectSpeech,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct PunctuationSafeZone {
    pub span: Span,
    pub inner_span: Span,
    pub kind: PunctuationSafeZoneKind,
    pub confidence: SyntaxConfidence,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DelimiterPair {
    pub open: char,
    pub close: char,
}

impl DelimiterPair {
    pub const fn new(open: char, close: char) -> Self {
        Self { open, close }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ParentheticalSpan {
    pub span: Span,
    pub inner_span: Span,
    pub delimiter: DelimiterPair,
    pub confidence: SyntaxConfidence,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DirectSpeechSpan {
    pub quote_span: Span,
    pub inner_span: Span,
    pub opening_quote: char,
    pub closing_quote: char,
    pub confidence: SyntaxConfidence,
}
