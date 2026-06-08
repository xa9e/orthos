use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Issue {
    pub rule_id: String,
    pub severity: crate::corpus::Severity,
    pub message: String,
    pub span: Span,
    pub start: Position,
    pub end: Position,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_refs: Vec<String>,
    pub excerpt: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof: Option<crate::syntax::DiagnosticProof>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}
