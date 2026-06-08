#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PunctuationMark {
    Comma,
    Dash,
    Colon,
    Semicolon,
    Period,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PunctuationBoundaryStrength {
    None,
    Weak,
    Medium,
    Strong,
    Unsafe,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PunctuationSlotEvidenceKind {
    Boundary,
    ExistingMark,
    ExpectedMark,
    ForbiddenMark,
    ClauseBoundary,
    IntroductoryCandidate,
    Coordination,
    IslandBlocker,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PunctuationSlotEvidence {
    pub kind: PunctuationSlotEvidenceKind,
    pub mark: Option<PunctuationMark>,
    pub message: String,
    pub span: Option<Span>,
}

impl PunctuationSlotEvidence {
    pub fn new(
        kind: PunctuationSlotEvidenceKind,
        mark: Option<PunctuationMark>,
        message: impl Into<String>,
        span: Option<Span>,
    ) -> Self {
        Self {
            kind,
            mark,
            message: message.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PunctuationSlot<'a> {
    pub id: usize,
    pub span: Span,
    pub left_token_index: usize,
    pub right_token_index: usize,
    pub left_token: Token<'a>,
    pub right_token: Token<'a>,
    pub boundary_strength: PunctuationBoundaryStrength,
    pub inside_quotes: bool,
    pub inside_parentheses: bool,
    pub between_clauses: bool,
    pub after_introductory_candidate: bool,
    pub existing_marks: Vec<PunctuationMark>,
    pub expected_marks: Vec<PunctuationMark>,
    pub forbidden_marks: Vec<PunctuationMark>,
    pub explanation: String,
    pub evidence: Vec<PunctuationSlotEvidence>,
    pub blockers: Vec<SuppressionReason>,
}

impl PunctuationSlot<'_> {
    pub fn allows(&self, mark: PunctuationMark) -> bool {
        !self.forbidden_marks.contains(&mark)
    }

    pub fn expects(&self, mark: PunctuationMark) -> bool {
        self.expected_marks.contains(&mark)
    }

    pub fn has_existing_mark(&self, mark: PunctuationMark) -> bool {
        self.existing_marks.contains(&mark)
    }

    pub fn missing_expected_mark(&self, mark: PunctuationMark) -> bool {
        self.expects(mark) && self.allows(mark) && !self.has_existing_mark(mark) && self.blockers.is_empty()
    }
}
