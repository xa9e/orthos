fn document_abbreviations(
    text: &str,
    tokens: &[Token<'_>],
    sentences: &[SentenceSpan],
) -> Vec<DocumentAbbreviation> {
    let mut by_short = std::collections::BTreeMap::<String, Vec<DocumentMention>>::new();
    for (token_index, token) in tokens.iter().enumerate() {
        if token.kind != TokenKind::Word {
            continue;
        }
        let Some(short) = normalize_abbreviation(token.text) else { continue; };
        by_short.entry(short).or_default().push(DocumentMention {
            span: token.span,
            sentence_index: sentence_index_for_span(token.span, sentences),
            token_index,
            is_first_mention: false,
        });
    }

    let mut out = by_short
        .into_iter()
        .map(|(short, mut mentions)| {
            if let Some(first) = mentions.first_mut() {
                first.is_first_mention = true;
            }
            let first_span = mentions.first().map(|mention| mention.span).unwrap_or(Span::new(0, 0));
            DocumentAbbreviation {
                expansion: expansion_before_parenthetical_abbreviation(text, first_span),
                short,
                first_span,
                frequency: mentions.len(),
                mentions,
            }
        })
        .collect::<Vec<_>>();
    out.sort_by_key(|abbr| (std::cmp::Reverse(abbr.frequency), abbr.first_span.start, abbr.short.clone()));
    out
}

fn normalize_abbreviation(value: &str) -> Option<String> {
    let cleaned = value
        .chars()
        .filter(|ch| ch.is_alphabetic())
        .collect::<String>();
    let length = cleaned.chars().count();
    if !(2..=10).contains(&length) {
        return None;
    }
    let has_lowercase = cleaned.chars().any(|ch| ch.is_lowercase());
    (!has_lowercase).then(|| cleaned.to_uppercase())
}

fn expansion_before_parenthetical_abbreviation(text: &str, abbr_span: Span) -> Option<String> {
    let open = text[..abbr_span.start].rfind('(')?;
    if text[open..abbr_span.start].contains(')') {
        return None;
    }
    let before = text[..open].trim_end();
    let start = before
        .rfind(is_abbreviation_context_boundary)
        .map(|idx| idx + 1)
        .unwrap_or(0);
    let candidate = before[start..]
        .trim()
        .trim_matches(|ch: char| matches!(ch, ':' | ';' | ',' | '—' | '-'))
        .trim();
    (candidate.chars().count() >= 6).then(|| candidate.to_owned())
}

fn is_abbreviation_context_boundary(ch: char) -> bool {
    matches!(ch, '.' | '!' | '?' | '\n')
}
