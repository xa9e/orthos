use crate::issue::{Position, Span};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub span: Span,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TokenKind {
    Word,
    Number,
    Whitespace,
    Punctuation,
    Other,
}

pub fn tokenize(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut iter = text.char_indices().peekable();

    while let Some((start, ch)) = iter.next() {
        let kind = classify(ch);
        let mut end = start + ch.len_utf8();

        while let Some(&(idx, next)) = iter.peek() {
            if !same_group(kind, next) {
                break;
            }
            iter.next();
            end = idx + next.len_utf8();
        }

        tokens.push(Token {
            kind,
            text: &text[start..end],
            span: Span::new(start, end),
        });
    }

    tokens
}

pub fn word_tokens(text: &str) -> Vec<Token<'_>> {
    tokenize(text)
        .into_iter()
        .filter(|token| token.kind == TokenKind::Word)
        .collect()
}

fn classify(ch: char) -> TokenKind {
    if ch.is_whitespace() {
        TokenKind::Whitespace
    } else if ch.is_ascii_digit() || matches!(ch, '０'..='９') {
        TokenKind::Number
    } else if ch.is_alphabetic() || is_word_joiner(ch) {
        TokenKind::Word
    } else if is_ru_punctuation(ch) || ch.is_ascii_punctuation() {
        TokenKind::Punctuation
    } else {
        TokenKind::Other
    }
}

fn same_group(kind: TokenKind, ch: char) -> bool {
    match kind {
        TokenKind::Word => ch.is_alphabetic() || is_word_joiner(ch),
        TokenKind::Number => ch.is_ascii_digit() || matches!(ch, '０'..='９'),
        TokenKind::Whitespace => ch.is_whitespace(),
        TokenKind::Punctuation => is_ru_punctuation(ch) || ch.is_ascii_punctuation(),
        TokenKind::Other => classify(ch) == TokenKind::Other,
    }
}

fn is_word_joiner(ch: char) -> bool {
    matches!(ch, '-' | '‑' | '–' | '\'')
}

fn is_ru_punctuation(ch: char) -> bool {
    matches!(ch, '«' | '»' | '—' | '–' | '…' | '“' | '”' | '„')
}

#[derive(Debug, Clone)]
pub struct LineIndex<'a> {
    text: &'a str,
    starts: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub fn new(text: &'a str) -> Self {
        let mut starts = vec![0];
        for (idx, ch) in text.char_indices() {
            if ch == '\n' {
                starts.push(idx + 1);
            }
        }
        Self { text, starts }
    }

    pub fn position(&self, byte_offset: usize) -> Position {
        let safe_offset = floor_char_boundary(self.text, byte_offset.min(self.text.len()));
        let line_zero = match self.starts.binary_search(&safe_offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.starts[line_zero];
        Position {
            line: line_zero + 1,
            column: self.text[line_start..safe_offset].chars().count() + 1,
        }
    }

    pub fn line_count(&self) -> usize {
        self.starts.len()
    }
}

pub fn excerpt(text: &str, span: Span) -> String {
    let start = floor_char_boundary(text, span.start.min(text.len()));
    let end = ceil_char_boundary(text, span.end.min(text.len()));
    let before = text[..start]
        .rfind(is_excerpt_boundary)
        .map(|i| i + 1)
        .unwrap_or(0);
    let after = text[end..]
        .find(is_excerpt_boundary)
        .map(|i| end + i + 1)
        .unwrap_or(text.len());
    text[before..after].trim().to_owned()
}

fn is_excerpt_boundary(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?' | '\n')
}

pub fn previous_non_ws<'t, 's>(tokens: &'s [Token<'t>], idx: usize) -> Option<&'s Token<'t>> {
    tokens[..idx]
        .iter()
        .rev()
        .find(|t| t.kind != TokenKind::Whitespace)
}

pub fn next_non_ws<'t, 's>(tokens: &'s [Token<'t>], idx: usize) -> Option<&'s Token<'t>> {
    tokens[idx + 1..]
        .iter()
        .find(|t| t.kind != TokenKind::Whitespace)
}

pub fn lower_ru(s: &str) -> String {
    s.chars().flat_map(char::to_lowercase).collect::<String>()
}

/// Folds `ё` to `е`. Russian text routinely omits `ё`, so dictionary lookup
/// must treat the two spellings as one key space.
pub fn fold_yo(s: &str) -> String {
    s.replace('ё', "е")
}

/// Canonical lexicon lookup key: lowercased with `ё` folded to `е`.
pub fn morph_lookup_key(s: &str) -> String {
    fold_yo(&lower_ru(s))
}

pub fn normalize_word(word: &str) -> String {
    word.chars()
        .filter(|ch| ch.is_alphabetic() || *ch == '-')
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn first_char_is_lowercase(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_lowercase())
}

pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().collect::<String>() + chars.as_str()
}

pub fn uppercase_first(value: &str) -> String {
    capitalize_first(value)
}

pub fn is_cyrillic(ch: char) -> bool {
    matches!(ch,
        '\u{0400}'..='\u{04FF}' |
        '\u{0500}'..='\u{052F}' |
        '\u{2DE0}'..='\u{2DFF}' |
        '\u{A640}'..='\u{A69F}'
    )
}

pub fn has_cyrillic(value: &str) -> bool {
    value.chars().any(is_cyrillic)
}

pub fn has_latin(value: &str) -> bool {
    value.chars().any(|ch| ch.is_ascii_alphabetic())
}

pub fn starts_with_lowercase_letter(value: &str) -> bool {
    value.chars().next().is_some_and(|ch| ch.is_lowercase())
}

pub fn char_before(text: &str, byte_index: usize) -> Option<(usize, char)> {
    let idx = floor_char_boundary(text, byte_index.min(text.len()));
    text[..idx].char_indices().next_back()
}

pub fn char_after(text: &str, byte_index: usize) -> Option<(usize, char)> {
    if byte_index > text.len() || !text.is_char_boundary(byte_index) {
        return None;
    }
    text[byte_index..]
        .char_indices()
        .next()
        .map(|(idx, ch)| (byte_index + idx, ch))
}

pub fn prev_non_ws_char(text: &str, byte_index: usize) -> Option<(usize, char)> {
    let idx = floor_char_boundary(text, byte_index.min(text.len()));
    text[..idx]
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_whitespace())
}

pub fn next_non_ws_char(text: &str, byte_index: usize) -> Option<(usize, char)> {
    if byte_index > text.len() || !text.is_char_boundary(byte_index) {
        return None;
    }
    text[byte_index..]
        .char_indices()
        .find(|(_, ch)| !ch.is_whitespace())
        .map(|(idx, ch)| (byte_index + idx, ch))
}

pub fn is_sentence_boundary(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?' | '…' | '\n')
}

pub fn is_punctuation_requiring_space_after(ch: char) -> bool {
    matches!(ch, ',' | ';' | ':' | '!' | '?' | '…' | '.')
}

pub fn is_space_forbidden_before(ch: char) -> bool {
    matches!(
        ch,
        ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '»' | '.'
    )
}

pub fn floor_char_boundary(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

pub fn ceil_char_boundary(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    while idx < text.len() && !text.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_columns_are_not_byte_columns() {
        let idx = LineIndex::new("Я ё");
        assert_eq!(idx.position("Я ".len()).column, 3);
    }

    #[test]
    fn detects_mixed_alphabet_words() {
        assert!(has_cyrillic("мaма"));
        assert!(has_latin("мaма"));
    }
}
