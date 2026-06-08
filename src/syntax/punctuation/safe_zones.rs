pub fn parenthetical_spans(text: &str) -> Vec<ParentheticalSpan> {
    let pairs = [
        DelimiterPair::new('(', ')'),
        DelimiterPair::new('[', ']'),
        DelimiterPair::new('{', '}'),
    ];
    delimiter_spans(text, &pairs)
        .into_iter()
        .map(|(span, inner_span, delimiter)| ParentheticalSpan {
            span,
            inner_span,
            delimiter,
            confidence: SyntaxConfidence::Certain,
        })
        .collect()
}

pub fn direct_speech_spans(text: &str) -> Vec<DirectSpeechSpan> {
    let pairs = [
        DelimiterPair::new('«', '»'),
        DelimiterPair::new('„', '“'),
        DelimiterPair::new('“', '”'),
        DelimiterPair::new('"', '"'),
    ];
    delimiter_spans(text, &pairs)
        .into_iter()
        .map(|(span, inner_span, delimiter)| DirectSpeechSpan {
            quote_span: span,
            inner_span,
            opening_quote: delimiter.open,
            closing_quote: delimiter.close,
            confidence: SyntaxConfidence::Strong,
        })
        .collect()
}

pub fn punctuation_safe_zone_records(text: &str) -> Vec<PunctuationSafeZone> {
    let mut zones: Vec<PunctuationSafeZone> = parenthetical_spans(text)
        .into_iter()
        .map(|span| PunctuationSafeZone {
            span: span.span,
            inner_span: span.inner_span,
            kind: PunctuationSafeZoneKind::Parenthetical,
            confidence: span.confidence,
        })
        .collect();

    zones.extend(direct_speech_spans(text).into_iter().map(|span| PunctuationSafeZone {
        span: span.quote_span,
        inner_span: span.inner_span,
        kind: PunctuationSafeZoneKind::DirectSpeech,
        confidence: span.confidence,
    }));

    zones.sort_by_key(|zone| (zone.span.start, zone.span.end));
    zones
}

pub fn punctuation_safe_zones(text: &str) -> Vec<SyntaxSpan> {
    punctuation_safe_zone_records(text)
        .into_iter()
        .map(|zone| SyntaxSpan {
            span: zone.span,
            start_token: None,
            end_token: None,
            kind: SyntaxSpanKind::PunctuationSafeZone,
            confidence: zone.confidence,
        })
        .collect()
}

pub fn is_inside_punctuation_safe_zone(text: &str, byte_index: usize) -> bool {
    punctuation_safe_zone_records(text)
        .into_iter()
        .any(|zone| zone.span.start < byte_index && byte_index < zone.span.end)
}

pub fn is_inside_quotes_or_parentheses(text: &str, byte_index: usize) -> bool {
    is_inside_punctuation_safe_zone(text, byte_index)
}

pub fn find_clause_marker(
    tokens: &[Token<'_>],
    idx: usize,
    marker_set: &HashSet<String>,
) -> Option<ClauseMarkerMatch> {
    let token = tokens.get(idx)?;
    if token.kind != TokenKind::Word {
        return None;
    }

    if let Some(marker) = find_multiword_clause_marker(tokens, idx, marker_set) {
        return Some(marker);
    }

    let current = lower_ru(token.text);
    if is_non_initial_part_of_multiword_clause_marker(tokens, idx, marker_set) {
        return None;
    }

    if marker_set.contains(&current) {
        return Some(ClauseMarker {
            start_token: idx,
            end_token: idx,
            span: token.span,
            canonical: current.clone(),
            kind: marker_kind_for(&current),
            confidence: marker_confidence_for(&current),
        });
    }

    None
}

pub fn clause_boundaries(text: &str, tokens: &[Token<'_>], marker_set: &HashSet<String>) -> Vec<ClauseBoundary> {
    let mut boundaries = Vec::new();

    for idx in 0..tokens.len() {
        let Some(marker) = find_clause_marker(tokens, idx, marker_set) else {
            continue;
        };
        if marker.start_token != idx {
            continue;
        }
        boundaries.push(clause_boundary_for_marker(text, tokens, marker));
    }

    boundaries
}

pub fn clause_boundary_for_marker(text: &str, tokens: &[Token<'_>], marker: ClauseMarker) -> ClauseBoundary {
    let punctuation = punctuation_context_before(tokens, marker.start_token);
    let (kind, confidence) = if is_inside_punctuation_safe_zone(text, marker.span.start) {
        (ClauseBoundaryKind::SuppressedSafeZone, SyntaxConfidence::Certain)
    } else if is_sentence_initial(tokens, marker.start_token) {
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
