#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CoordinationGroupKind {
    ModifierSeries,
    NominalSubject,
    PredicateSeries,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CoordinationConnectorKind {
    Comma,
    SingleConjunction,
    RepeatedConjunction,
    Mixed,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CoordinationMember<'a> {
    pub token_index: usize,
    pub token: Token<'a>,
    pub analyses: Vec<crate::morph::MorphAnalysis>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CoordinationConnector<'a> {
    pub kind: CoordinationConnectorKind,
    pub token_index: usize,
    pub token: Token<'a>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CoordinationGroup<'a> {
    pub id: usize,
    pub kind: CoordinationGroupKind,
    pub span: Span,
    pub start_token: usize,
    pub end_token: usize,
    pub island_id: Option<usize>,
    pub members: Vec<CoordinationMember<'a>>,
    pub connectors: Vec<CoordinationConnector<'a>>,
    pub head_context: Option<Token<'a>>,
    pub agreement_number: Option<crate::morph::Number>,
    pub shared_cases: Vec<crate::morph::Case>,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SuppressionReason>,
}

impl<'a> CoordinationGroup<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty() && self.members.len() > 1
    }
}
