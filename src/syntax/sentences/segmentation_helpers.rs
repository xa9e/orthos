fn token_range_for_span(tokens: &[Token<'_>], start: usize, end: usize) -> (Option<usize>, Option<usize>) {
    let start_token = tokens
        .iter()
        .position(|token| token.span.end > start && token.span.start < end);
    let end_token = tokens
        .iter()
        .enumerate()
        .rev()
        .find(|(_, token)| token.span.end > start && token.span.start < end)
        .map(|(idx, _)| idx);
    (start_token, end_token)
}

fn delimiter_spans(text: &str, pairs: &[DelimiterPair]) -> Vec<(Span, Span, DelimiterPair)> {
    let mut stack: Vec<(DelimiterPair, usize)> = Vec::new();
    let mut spans = Vec::new();

    for (idx, ch) in text.char_indices() {
        if pairs.iter().any(|pair| pair.close == ch) {
            if stack.last().is_some_and(|(pair, _)| pair.close == ch) {
                let (pair, start) = stack.pop().expect("checked stack top");
                let open_end = start + pair.open.len_utf8();
                let close_start = idx;
                let close_end = idx + ch.len_utf8();
                if open_end <= close_start {
                    spans.push((Span::new(start, close_end), Span::new(open_end, close_start), pair));
                }
                continue;
            }

            if !pairs.iter().any(|pair| pair.open == ch) {
                continue;
            }
        }

        if let Some(pair) = pairs.iter().find(|pair| pair.open == ch) {
            stack.push((*pair, idx));
        }
    }

    spans.sort_by_key(|(span, _, _)| (span.start, span.end));
    spans
}

fn include_closing_boundary_punctuation(text: &str, byte_index: usize) -> usize {
    let mut end = byte_index;
    for (_, ch) in text[byte_index..].char_indices() {
        if !is_closing_boundary_punctuation(ch) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

fn sentence_segmentation_safe_zones(text: &str) -> Vec<PunctuationSafeZone> {
    punctuation_safe_zone_records(text)
}

fn sentence_terminal_cluster_end(text: &str, byte_index: usize) -> usize {
    let mut end = byte_index;
    for (local, ch) in text[byte_index..].char_indices() {
        if !matches!(ch, '.' | '!' | '?' | '…' | '\n') {
            break;
        }
        end = byte_index + local + ch.len_utf8();
    }
    end
}

fn is_sentence_split_candidate(
    text: &str,
    sentence_start: usize,
    terminal_start: usize,
    terminal_end: usize,
    safe_zones: &[PunctuationSafeZone],
) -> bool {
    let Some((_, ch)) = char_at(text, terminal_start) else {
        return false;
    };

    if ch == '.' && terminal_end == terminal_start + ch.len_utf8() && is_probable_abbreviation_period(text, sentence_start, terminal_start) {
        return false;
    }

    let boundary_end = include_closing_boundary_punctuation(text, terminal_end);
    if let Some(zone) = safe_zones
        .iter()
        .find(|zone| zone.span.start < terminal_start && terminal_start < zone.span.end)
    {
        if boundary_end < zone.span.end {
            return false;
        }

        if matches!(zone.kind, PunctuationSafeZoneKind::Parenthetical) && !looks_like_sentence_continues_after(text, zone.span.end) {
            return true;
        }

        return !looks_like_sentence_continues_after(text, boundary_end);
    }

    true
}

fn looks_like_sentence_continues_after(text: &str, byte_index: usize) -> bool {
    let Some((_, ch)) = next_non_ws_char_from(text, byte_index) else {
        return false;
    };
    matches!(ch, '—' | '-' | ',' | ';' | ':') || ch.is_lowercase()
}

fn is_probable_abbreviation_period(text: &str, sentence_start: usize, period_idx: usize) -> bool {
    if is_decimal_separator_period(text, period_idx) {
        return true;
    }

    let prefix = lower_ru(text[sentence_start..=period_idx].trim_end());
    if ["т.е.", "т. е.", "т.к.", "т. к.", "т.п.", "т. п.", "т.д.", "т. д."].iter().any(|abbr| prefix.ends_with(*abbr)) {
        return true;
    }

    let Some(word_start) = word_start_before(text, period_idx) else {
        return false;
    };
    let word = &text[word_start..period_idx];
    let normalized = lower_ru(word);

    if is_known_abbreviation_word(&normalized) {
        return true;
    }

    if word.chars().count() == 1 && word.chars().all(|ch| ch.is_uppercase()) {
        return next_non_ws_char_from(text, period_idx + 1).is_some_and(|(_, next)| next.is_uppercase());
    }

    false
}

fn is_decimal_separator_period(text: &str, period_idx: usize) -> bool {
    previous_char(text, period_idx).is_some_and(|(_, ch)| ch.is_ascii_digit())
        && next_char(text, period_idx + 1).is_some_and(|(_, ch)| ch.is_ascii_digit())
}

fn is_known_abbreviation_word(word: &str) -> bool {
    matches!(
        word,
        "г" | "ул"
            | "д"
            | "кв"
            | "стр"
            | "рис"
            | "см"
            | "им"
            | "тов"
            | "проф"
            | "доц"
            | "акад"
            | "др"
            | "пр"
            | "т"
            | "н"
            | "э"
            | "руб"
            | "коп"
            | "тыс"
            | "млн"
            | "млрд"
            | "зам"
            | "нач"
            | "ред"
    )
}
