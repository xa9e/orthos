fn clause_boundaries_for_sentence(
    text: &str,
    tokens: &[Token<'_>],
    marker_set: &HashSet<String>,
    sentence: SentenceSpan,
) -> Vec<ClauseBoundary> {
    let mut boundaries = Vec::new();

    for idx in 0..tokens.len() {
        let token = &tokens[idx];
        if token.span.start < sentence.start || token.span.start >= sentence.end {
            continue;
        }
        let Some(marker) = find_clause_marker(tokens, idx, marker_set) else {
            continue;
        };
        if marker.start_token != idx || !sentence.contains(marker.span.start) {
            continue;
        }
        boundaries.push(clause_boundary_for_marker_in_sentence(text, tokens, marker, sentence.start));
    }

    boundaries
}

fn clause_boundary_for_marker_in_sentence(
    text: &str,
    tokens: &[Token<'_>],
    marker: ClauseMarker,
    sentence_start: usize,
) -> ClauseBoundary {
    let punctuation = punctuation_context_before(tokens, marker.start_token);
    let (kind, confidence) = if is_inside_punctuation_safe_zone(text, marker.span.start) {
        (ClauseBoundaryKind::SuppressedSafeZone, SyntaxConfidence::Certain)
    } else if is_sentence_initial_in_span(tokens, marker.start_token, sentence_start) {
        (ClauseBoundaryKind::SentenceStartMarker, SyntaxConfidence::Certain)
    } else if punctuation.has_comma_before
        || punctuation.preceded_by_clause_separator
        || punctuation.preceded_by_sentence_terminal
    {
        (ClauseBoundaryKind::PunctuatedBeforeMarker, SyntaxConfidence::Certain)
    } else if previous_word_is_coordinating_or_discourse_marker(tokens, marker.start_token) {
        (ClauseBoundaryKind::Ambiguous, SyntaxConfidence::Weak)
    } else if marker.confidence.is_actionable() {
        (ClauseBoundaryKind::BeforeMarker, marker.confidence)
    } else {
        (ClauseBoundaryKind::Ambiguous, SyntaxConfidence::Ambiguous)
    };

    ClauseBoundary {
        boundary_span: Span::new(marker.span.start, marker.span.start),
        marker,
        kind,
        confidence,
    }
}

fn is_sentence_initial_in_span(tokens: &[Token<'_>], idx: usize, sentence_start: usize) -> bool {
    let mut cursor = idx;
    while let Some(prev_idx) = previous_non_space_token(tokens, cursor) {
        let prev = &tokens[prev_idx];
        if prev.span.start < sentence_start {
            return true;
        }
        let Some(ch) = last_char(prev.text) else {
            return false;
        };
        if is_opening_boundary_punctuation(ch) {
            cursor = prev_idx;
            continue;
        }
        return false;
    }
    true
}

pub fn should_report_missing_comma_before_clause_marker(
    text: &str,
    tokens: &[Token<'_>],
    marker: &ClauseMarker,
) -> bool {
    marker.confidence.is_actionable() && !is_safe_boundary_before_clause_marker(text, tokens, marker)
}

pub fn marker_dependency_edge(marker: &ClauseMarker) -> DependencyEdge {
    DependencyEdge {
        head_token: marker.start_token,
        dependent_token: marker.end_token,
        relation: DependencyRelation::MarkerIntroducesClause,
        confidence: marker.confidence,
    }
}

pub fn syntax_span_for_tokens(
    tokens: &[Token<'_>],
    start_token: usize,
    end_token: usize,
    kind: SyntaxSpanKind,
    confidence: SyntaxConfidence,
) -> Option<SyntaxSpan> {
    let start = tokens.get(start_token)?;
    let end = tokens.get(end_token)?;
    Some(SyntaxSpan {
        span: Span::new(start.span.start, end.span.end),
        start_token: Some(start_token),
        end_token: Some(end_token),
        kind,
        confidence,
    })
}

pub fn is_safe_boundary_before_clause_marker(text: &str, tokens: &[Token<'_>], marker: &ClauseMarkerMatch) -> bool {
    if is_inside_punctuation_safe_zone(text, marker.span.start) || is_sentence_initial(tokens, marker.start_token) {
        return true;
    }

    let punctuation = punctuation_context_before(tokens, marker.start_token);
    if punctuation.has_comma_before || punctuation.preceded_by_clause_separator || punctuation.preceded_by_sentence_terminal {
        return true;
    }

    if let Some(prev_word_idx) = previous_word_token(tokens, marker.start_token) {
        let prev = lower_ru(tokens[prev_word_idx].text);
        if is_coordinating_or_discourse_marker(&prev) {
            return true;
        }
    }

    false
}

pub fn is_opening_boundary_punctuation(ch: char) -> bool {
    matches!(ch, '(' | '[' | '{' | '«' | '“' | '„')
}

pub fn is_closing_boundary_punctuation(ch: char) -> bool {
    matches!(ch, ')' | ']' | '}' | '»' | '”' | '“')
}

pub fn is_clause_separator(ch: char) -> bool {
    matches!(ch, ',' | ';' | ':' | '—' | '-')
}
