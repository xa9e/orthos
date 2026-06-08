#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClauseBoundaryDebugEntry {
    pub marker: String,
    pub kind: String,
    pub confidence: String,
    pub span: Span,
    pub start_token: usize,
    pub end_token: usize,
}

impl ClauseBoundaryDebugEntry {
    fn from_boundary(boundary: &crate::syntax::ClauseBoundary) -> Self {
        Self {
            marker: boundary.marker.canonical.clone(),
            kind: format!("{:?}", boundary.kind),
            confidence: format!("{:?}", boundary.confidence),
            span: boundary.boundary_span,
            start_token: boundary.marker.start_token,
            end_token: boundary.marker.end_token,
        }
    }
}
