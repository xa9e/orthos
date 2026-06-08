#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ClauseBoundaryMap {
    boundaries: Vec<ClauseBoundary>,
}

impl ClauseBoundaryMap {
    pub fn from_text_tokens(text: &str, tokens: &[Token<'_>]) -> Self {
        let marker_set = default_clause_marker_set();
        let boundaries = sentence_spans(text)
            .into_iter()
            .flat_map(|sentence| clause_boundaries_for_sentence(text, tokens, &marker_set, sentence))
            .collect();
        Self { boundaries }
    }

    pub fn boundaries(&self) -> &[ClauseBoundary] {
        &self.boundaries
    }

    pub fn blockers_between_tokens(
        &self,
        left_token: usize,
        right_token: usize,
    ) -> Vec<SuppressionReason> {
        if left_token == right_token {
            return Vec::new();
        }
        let (start, end) = if left_token < right_token {
            (left_token, right_token)
        } else {
            (right_token, left_token)
        };

        let mut blockers = self
            .boundaries
            .iter()
            .filter(|boundary| boundary_is_link_blocker(boundary))
            .filter(|boundary| start < boundary.marker.start_token && boundary.marker.start_token <= end)
            .map(|_| SuppressionReason::ClauseBoundary)
            .collect::<Vec<_>>();
        blockers.sort_unstable();
        blockers.dedup();
        blockers
    }
}

fn boundary_is_link_blocker(boundary: &ClauseBoundary) -> bool {
    matches!(
        boundary.kind,
        ClauseBoundaryKind::BeforeMarker | ClauseBoundaryKind::PunctuatedBeforeMarker
    ) && boundary.confidence.is_actionable()
}
