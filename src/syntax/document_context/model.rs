#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum HeadingStyle {
    MarkdownAtx,
    PlainShortLine,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ListMarkerKind {
    Dash,
    Asterisk,
    Bullet,
    Numbered,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct HeadingCandidate {
    pub line_number: usize,
    pub span: Span,
    pub text: String,
    pub style: HeadingStyle,
    pub has_terminal_punctuation: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ListItemCandidate {
    pub line_number: usize,
    pub span: Span,
    pub marker: String,
    pub marker_kind: ListMarkerKind,
    pub body: String,
    pub has_terminal_punctuation: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DocumentMention {
    pub span: Span,
    pub sentence_index: Option<usize>,
    pub token_index: usize,
    pub is_first_mention: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DocumentTerm {
    pub canonical: String,
    pub first_span: Span,
    pub frequency: usize,
    pub mentions: Vec<DocumentMention>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CrossSentenceFact {
    pub key: String,
    pub value: String,
    pub spans: Vec<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DocumentAbbreviation {
    pub short: String,
    pub expansion: Option<String>,
    pub first_span: Span,
    pub frequency: usize,
    pub mentions: Vec<DocumentMention>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GlossaryEntry {
    pub term: String,
    pub definition: String,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct DocumentStyleProfile {
    pub paragraph_count: usize,
    pub heading_count: usize,
    pub list_item_count: usize,
    pub mixed_heading_punctuation: bool,
    pub mixed_list_markers: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DocumentContext<'a> {
    pub text: &'a str,
    pub terms: Vec<DocumentTerm>,
    pub repeated_terms: Vec<DocumentTerm>,
    pub headings: Vec<HeadingCandidate>,
    pub list_items: Vec<ListItemCandidate>,
    pub abbreviations: Vec<DocumentAbbreviation>,
    pub glossary_entries: Vec<GlossaryEntry>,
    pub style: DocumentStyleProfile,
    pub cross_sentence_facts: Vec<CrossSentenceFact>,
}

impl<'a> DocumentContext<'a> {
    pub fn new(text: &'a str, tokens: &[Token<'a>]) -> Self {
        let sentence_spans = sentence_spans(text);
        let headings = heading_candidates(text);
        let list_items = list_item_candidates(text);
        let terms = document_terms(tokens, &sentence_spans);
        let repeated_terms = terms
            .iter()
            .filter(|term| term.frequency > 1)
            .cloned()
            .collect::<Vec<_>>();
        let abbreviations = document_abbreviations(text, tokens, &sentence_spans);
        let glossary_entries = glossary_entries_from_document(text, &headings, &list_items);
        let cross_sentence_facts = cross_sentence_facts_from_terms_and_abbreviations(&terms, &abbreviations);
        let style = document_style_profile(text, &headings, &list_items);
        Self {
            text,
            terms,
            repeated_terms,
            headings,
            list_items,
            abbreviations,
            glossary_entries,
            style,
            cross_sentence_facts,
        }
    }
}
