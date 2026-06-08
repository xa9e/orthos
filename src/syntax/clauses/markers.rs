fn find_multiword_clause_marker(
    tokens: &[Token<'_>],
    idx: usize,
    marker_set: &HashSet<String>,
) -> Option<ClauseMarker> {
    let current = lower_ru(tokens.get(idx)?.text);
    let mut best: Option<ClauseMarker> = None;

    for marker in marker_set.iter().filter(|marker| marker.contains(' ')) {
        let parts = marker.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 2 || parts.first().is_none_or(|first| *first != current.as_str()) {
            continue;
        }

        let mut end_token = idx;
        let mut ok = true;
        for expected in parts.iter().skip(1) {
            let Some(next_idx) = next_word_token(tokens, end_token) else {
                ok = false;
                break;
            };
            if !only_whitespace_between(tokens, end_token, next_idx) || lower_ru(tokens[next_idx].text) != *expected {
                ok = false;
                break;
            }
            end_token = next_idx;
        }

        if !ok {
            continue;
        }

        let candidate = ClauseMarker {
            start_token: idx,
            end_token,
            span: Span::new(tokens[idx].span.start, tokens[end_token].span.end),
            canonical: marker.clone(),
            kind: ClauseMarkerKind::MultiwordSubordinator,
            confidence: marker_confidence_for(marker),
        };

        if best
            .as_ref()
            .is_none_or(|existing| {
                candidate.canonical.split_whitespace().count()
                    > existing.canonical.split_whitespace().count()
            })
        {
            best = Some(candidate);
        }
    }

    best
}

fn is_non_initial_part_of_multiword_clause_marker(
    tokens: &[Token<'_>],
    idx: usize,
    marker_set: &HashSet<String>,
) -> bool {
    let current = lower_ru(tokens[idx].text);

    marker_set.iter().filter(|marker| marker.contains(' ')).any(|marker| {
        let parts = marker.split_whitespace().collect::<Vec<_>>();
        parts.iter().enumerate().skip(1).any(|(part_idx, part)| {
            if *part != current.as_str() {
                return false;
            }

            let mut cursor = idx;
            for expected in parts[..part_idx].iter().rev() {
                let Some(prev_idx) = previous_word_token(tokens, cursor) else {
                    return false;
                };
                if !only_whitespace_between(tokens, prev_idx, cursor) || lower_ru(tokens[prev_idx].text) != *expected {
                    return false;
                }
                cursor = prev_idx;
            }
            true
        })
    })
}

fn only_whitespace_between(tokens: &[Token<'_>], left_idx: usize, right_idx: usize) -> bool {
    left_idx < right_idx && tokens[left_idx + 1..right_idx].iter().all(|token| token.kind == TokenKind::Whitespace)
}

fn marker_kind_for(word: &str) -> ClauseMarkerKind {
    match word {
        "который" | "которая" | "которое" | "которые" => ClauseMarkerKind::RelativePronoun,
        "что" | "чтобы" | "если" | "когда" | "пока" | "поскольку" | "хотя" => ClauseMarkerKind::Subordinator,
        _ => ClauseMarkerKind::Unknown,
    }
}

fn marker_confidence_for(marker: &str) -> SyntaxConfidence {
    match marker {
        "потому что" | "так как" | "для того чтобы" => SyntaxConfidence::Strong,
        "что" | "чтобы" | "если" | "поскольку" | "хотя" => SyntaxConfidence::Strong,
        "когда" | "пока" | "который" | "которая" | "которое" | "которые" => SyntaxConfidence::Weak,
        _ if marker.contains(' ') => SyntaxConfidence::Weak,
        _ => SyntaxConfidence::Ambiguous,
    }
}


pub fn default_clause_marker_set() -> HashSet<String> {
    [
        "что",
        "чтобы",
        "если",
        "когда",
        "пока",
        "поскольку",
        "хотя",
        "который",
        "которая",
        "которое",
        "которые",
        "потому что",
        "так как",
        "для того чтобы",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn previous_word_is_coordinating_or_discourse_marker(tokens: &[Token<'_>], idx: usize) -> bool {
    previous_word_token(tokens, idx)
        .map(|prev_idx| lower_ru(tokens[prev_idx].text))
        .is_some_and(|word| is_coordinating_or_discourse_marker(&word))
}

fn is_coordinating_or_discourse_marker(word: &str) -> bool {
    matches!(word, "и" | "а" | "но" | "или" | "либо" | "да" | "ну")
}

fn last_char(value: &str) -> Option<char> {
    value.chars().next_back()
}

fn char_at(text: &str, byte_index: usize) -> Option<(usize, char)> {
    if byte_index > text.len() || !text.is_char_boundary(byte_index) {
        return None;
    }
    text[byte_index..].char_indices().next().map(|(idx, ch)| (byte_index + idx, ch))
}

fn previous_char(text: &str, byte_index: usize) -> Option<(usize, char)> {
    if byte_index > text.len() {
        return None;
    }
    let mut idx = byte_index.min(text.len());
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    text[..idx].char_indices().next_back()
}

fn next_char(text: &str, byte_index: usize) -> Option<(usize, char)> {
    char_at(text, byte_index)
}

fn next_non_ws_char_from(text: &str, byte_index: usize) -> Option<(usize, char)> {
    if byte_index > text.len() || !text.is_char_boundary(byte_index) {
        return None;
    }
    text[byte_index..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(idx, ch)| (byte_index + idx, ch))
}

fn word_start_before(text: &str, byte_index: usize) -> Option<usize> {
    let mut start = None;
    for (idx, ch) in text[..byte_index].char_indices().rev() {
        if ch.is_alphabetic() {
            start = Some(idx);
            continue;
        }
        break;
    }
    start
}
