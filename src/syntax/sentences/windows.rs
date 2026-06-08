pub fn sentence_spans(text: &str) -> Vec<SentenceSpan> {
    let Some((mut start, _)) = text.char_indices().find(|(_, ch)| !ch.is_whitespace()) else {
        return Vec::new();
    };

    let safe_zones = sentence_segmentation_safe_zones(text);
    let mut spans = Vec::new();
    let mut cursor = start;

    while cursor < text.len() {
        let Some((local_idx, ch)) = text[cursor..].char_indices().next() else {
            break;
        };
        let idx = cursor + local_idx;
        let next_cursor = idx + ch.len_utf8();

        if !is_sentence_boundary(ch) {
            cursor = next_cursor;
            continue;
        }

        let terminal_end = sentence_terminal_cluster_end(text, idx);
        if !is_sentence_split_candidate(text, start, idx, terminal_end, &safe_zones) {
            cursor = terminal_end;
            continue;
        }

        let end = include_closing_boundary_punctuation(text, terminal_end);
        if start < end {
            spans.push(SentenceSpan { start, end });
        }

        start = text[end..]
            .char_indices()
            .find(|(_, next)| !next.is_whitespace())
            .map(|(local, _)| end + local)
            .unwrap_or(text.len());
        cursor = start;
    }

    if start < text.len() {
        spans.push(SentenceSpan { start, end: text.len() });
    }

    spans.sort_by_key(|span| (span.start, span.end));
    spans.dedup();
    spans
}

pub fn sentence_span_at(text: &str, byte_index: usize) -> Option<SentenceSpan> {
    sentence_spans(text).into_iter().find(|span| span.contains(byte_index))
}

pub fn token_window<'t, 's>(tokens: &'s [Token<'t>], idx: usize) -> Option<TokenWindow<'t, 's>> {
    let current = tokens.get(idx)?;
    Some(TokenWindow {
        previous: previous_non_ws(tokens, idx),
        current,
        next: next_non_ws(tokens, idx),
    })
}

pub fn previous_non_space_token(tokens: &[Token<'_>], idx: usize) -> Option<usize> {
    let end = idx.min(tokens.len());
    tokens[..end]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, token)| token.kind != TokenKind::Whitespace)
        .map(|(token_idx, _)| token_idx)
}

pub fn next_non_space_token(tokens: &[Token<'_>], idx: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(idx.saturating_add(1))
        .find(|(_, token)| token.kind != TokenKind::Whitespace)
        .map(|(token_idx, _)| token_idx)
}

pub fn previous_word_token(tokens: &[Token<'_>], idx: usize) -> Option<usize> {
    let end = idx.min(tokens.len());
    tokens[..end]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, token)| token.kind == TokenKind::Word)
        .map(|(token_idx, _)| token_idx)
}

pub fn next_word_token(tokens: &[Token<'_>], idx: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(idx.saturating_add(1))
        .find(|(_, token)| token.kind == TokenKind::Word)
        .map(|(token_idx, _)| token_idx)
}

pub fn punctuation_context_before(tokens: &[Token<'_>], idx: usize) -> PunctuationContext {
    let Some(prev_idx) = previous_non_space_token(tokens, idx) else {
        return PunctuationContext {
            has_comma_before: false,
            preceded_by_sentence_terminal: false,
            preceded_by_opening_delimiter: false,
            preceded_by_clause_separator: false,
        };
    };
    let prev = &tokens[prev_idx];
    let last = last_char(prev.text);

    PunctuationContext {
        has_comma_before: has_comma_before_marker(tokens, idx),
        preceded_by_sentence_terminal: last.is_some_and(is_sentence_boundary),
        preceded_by_opening_delimiter: last.is_some_and(is_opening_boundary_punctuation),
        preceded_by_clause_separator: last.is_some_and(is_clause_separator),
    }
}

pub fn has_comma_before_marker(tokens: &[Token<'_>], idx: usize) -> bool {
    let mut cursor = idx;
    while let Some(prev_idx) = previous_non_space_token(tokens, cursor) {
        let prev = &tokens[prev_idx];
        let Some(ch) = last_char(prev.text) else {
            return false;
        };
        if is_closing_boundary_punctuation(ch) {
            cursor = prev_idx;
            continue;
        }
        return ch == ',';
    }
    false
}

pub fn is_sentence_initial(tokens: &[Token<'_>], idx: usize) -> bool {
    let mut cursor = idx;
    while let Some(prev_idx) = previous_non_space_token(tokens, cursor) {
        let prev = &tokens[prev_idx];
        let Some(ch) = last_char(prev.text) else {
            return false;
        };
        if is_opening_boundary_punctuation(ch) {
            cursor = prev_idx;
            continue;
        }
        return is_sentence_boundary(ch);
    }
    true
}
