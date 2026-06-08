fn decimal_context(text: &str, dot_idx: usize, next_idx: usize) -> bool {
    let prev_digit = char_before(text, dot_idx).is_some_and(|(_, ch)| ch.is_ascii_digit());
    let next_digit = char_after(text, next_idx).is_some_and(|(_, ch)| ch.is_ascii_digit());
    prev_digit && next_digit
}

fn abbreviation_context(text: &str, dot_idx: usize, next_idx: usize) -> bool {
    let prev_letter = char_before(text, dot_idx).is_some_and(|(_, ch)| ch.is_alphabetic());
    let next_upper = char_after(text, next_idx).is_some_and(|(_, ch)| ch.is_uppercase());
    prev_letter && next_upper
}

fn is_closing_punctuation(ch: char) -> bool {
    matches!(ch, ')' | ']' | '}' | '»' | '”')
}

fn parse_delimiter_pairs(raw_pairs: &[String]) -> Vec<(char, char)> {
    if raw_pairs.is_empty() {
        return vec!['(', '[', '{', '«']
            .into_iter()
            .zip([')', ']', '}', '»'])
            .collect();
    }
    raw_pairs
        .iter()
        .filter_map(|pair| {
            let mut chars = pair.chars().filter(|ch| !ch.is_whitespace());
            Some((chars.next()?, chars.next()?))
        })
        .collect()
}

fn parse_replacement_map(values: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for value in values {
        if let Some((bad, good)) = value.split_once("=>") {
            map.insert(lower_ru(bad.trim()), good.trim().to_string());
        }
    }
    map
}

fn match_case(original: &str, replacement: &str) -> String {
    if original.chars().all(|ch| !ch.is_lowercase()) {
        replacement.to_uppercase()
    } else if original.chars().next().is_some_and(|ch| ch.is_uppercase()) {
        uppercase_first(replacement)
    } else {
        replacement.to_string()
    }
}

fn replace_suffix(word: &str, suffix: &str, replacement: &str) -> String {
    let lower = lower_ru(word);
    if !lower.ends_with(suffix) {
        return word.to_string();
    }
    let stem_bytes = word.len() - suffix.len();
    format!("{}{}", &word[..stem_bytes], replacement)
}

fn sentence_starts(text: &str) -> Vec<usize> {
    sentence_spans(text).into_iter().map(|span| span.start).collect()
}

fn is_boundary_after(text: &str, byte_idx: usize) -> bool {
    if byte_idx >= text.len() {
        return true;
    }
    char_after(text, byte_idx).is_some_and(|(_, ch)| !ch.is_alphabetic())
}

fn phrase_boundary(text: &str, start: usize, end: usize) -> bool {
    let left_ok = start == 0
        || char_before(text, start).is_some_and(|(_, ch)| !ch.is_alphabetic());
    let right_ok = end >= text.len()
        || char_after(text, end).is_some_and(|(_, ch)| !ch.is_alphabetic());
    left_ok && right_ok
}


fn word_tokens_in_span<'a>(tokens: &'a [Token<'a>], span: Span) -> Vec<&'a Token<'a>> {
    tokens
        .iter()
        .filter(|token| {
            token.kind == TokenKind::Word
                && token.span.start >= span.start
                && token.span.end <= span.end
        })
        .collect()
}

fn token_has_unambiguous_pos(
    token: &Token<'_>,
    morph: &dyn MorphAnalyzer,
    pos: PartOfSpeech,
) -> bool {
    let analyses = morph.analyze(token.text);
    !analyses.is_empty() && analyses.iter().all(|analysis| analysis.pos == pos)
}

fn token_is_unambiguous_numeral(token: &Token<'_>, morph: &dyn MorphAnalyzer) -> bool {
    token_has_unambiguous_pos(token, morph, PartOfSpeech::Numeral)
}
