#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofConfidence {
    Certain,
    Strong,
    Weak,
    Ambiguous,
}

impl From<SyntaxConfidence> for ProofConfidence {
    fn from(value: SyntaxConfidence) -> Self {
        match value {
            SyntaxConfidence::Certain => Self::Certain,
            SyntaxConfidence::Strong => Self::Strong,
            SyntaxConfidence::Weak => Self::Weak,
            SyntaxConfidence::Ambiguous => Self::Ambiguous,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionReason {
    InlineSuppression,
    FileSuppression,
    UnsafeBoundary,
    QuoteBoundary,
    ParenthesisBoundary,
    DirectSpeechBoundary,
    ClauseBoundary,
    SentenceBoundary,
    AmbiguousMorphology,
    UnknownMorphology,
    InsufficientMorphology,
    ConflictNotProven,
    NonActionableConfidence,
    PunctuationBetween,
    NewlineBetween,
    Coordination,
    UnsupportedRuleCapability,
    EmptySpan,
    Unknown,
}

impl From<SyntaxRelationBlocker> for SuppressionReason {
    fn from(value: SyntaxRelationBlocker) -> Self {
        match value {
            SyntaxRelationBlocker::PunctuationBetween => Self::PunctuationBetween,
            SyntaxRelationBlocker::SentenceBoundaryBetween => Self::SentenceBoundary,
            SyntaxRelationBlocker::NewlineBetween => Self::NewlineBetween,
            SyntaxRelationBlocker::AmbiguousMorphology => Self::AmbiguousMorphology,
            SyntaxRelationBlocker::UnknownMorphology => Self::UnknownMorphology,
            SyntaxRelationBlocker::Coordination => Self::Coordination,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticProofKind {
    AgreementConflict,
    GovernmentConflict,
    QuantityConflict,
    BoundarySuppression,
    IslandSuppression,
    AmbiguitySuppression,
    CapabilityContract,
    DocumentConsistency,
    DocumentAbbreviation,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DiagnosticFact {
    pub key: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

impl DiagnosticFact {
    pub fn new(key: impl Into<String>, value: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DiagnosticAssumption {
    pub key: String,
    pub value: String,
    pub confidence: ProofConfidence,
}

impl DiagnosticAssumption {
    pub fn new(key: impl Into<String>, value: impl Into<String>, confidence: ProofConfidence) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            confidence,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DiagnosticConflict {
    pub kind: String,
    pub expected: Vec<String>,
    pub observed: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SuppressedAlternative {
    pub label: String,
    pub reason: SuppressionReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DiagnosticProof {
    pub kind: DiagnosticProofKind,
    pub facts: Vec<DiagnosticFact>,
    pub assumptions: Vec<DiagnosticAssumption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict: Option<DiagnosticConflict>,
    pub confidence: ProofConfidence,
    pub blockers: Vec<SuppressionReason>,
    pub suppressed_alternatives: Vec<SuppressedAlternative>,
}

impl DiagnosticProof {
    pub fn new(kind: DiagnosticProofKind, confidence: SyntaxConfidence) -> Self {
        Self {
            kind,
            facts: Vec::new(),
            assumptions: Vec::new(),
            conflict: None,
            confidence: confidence.into(),
            blockers: Vec::new(),
            suppressed_alternatives: Vec::new(),
        }
    }

    pub fn with_fact(mut self, fact: DiagnosticFact) -> Self {
        self.facts.push(fact);
        self
    }

    pub fn with_assumption(mut self, assumption: DiagnosticAssumption) -> Self {
        self.assumptions.push(assumption);
        self
    }

    pub fn with_conflict(mut self, conflict: DiagnosticConflict) -> Self {
        self.conflict = Some(conflict);
        self
    }

    pub fn with_blocker(mut self, blocker: SuppressionReason) -> Self {
        self.blockers.push(blocker);
        self.blockers.sort_unstable();
        self.blockers.dedup();
        self
    }

    pub fn with_blockers(mut self, blockers: impl IntoIterator<Item = SuppressionReason>) -> Self {
        self.blockers.extend(blockers);
        self.blockers.sort_unstable();
        self.blockers.dedup();
        self
    }

    pub fn with_suppressed_alternative(mut self, alternative: SuppressedAlternative) -> Self {
        self.suppressed_alternatives.push(alternative);
        self
    }

    pub fn is_actionable(&self) -> bool {
        matches!(self.confidence, ProofConfidence::Certain | ProofConfidence::Strong)
            && self.blockers.is_empty()
            && self.conflict.is_some()
    }
}
