fn document_terms(tokens: &[Token<'_>], sentences: &[SentenceSpan]) -> Vec<DocumentTerm> {
    let mut by_term = std::collections::BTreeMap::<String, Vec<DocumentMention>>::new();
    for (token_index, token) in tokens.iter().enumerate() {
        if token.kind != TokenKind::Word {
            continue;
        }
        let canonical = normalize_document_term(token.text);
        if !is_document_term_candidate(&canonical) {
            continue;
        }
        by_term.entry(canonical).or_default().push(DocumentMention {
            span: token.span,
            sentence_index: sentence_index_for_span(token.span, sentences),
            token_index,
            is_first_mention: false,
        });
    }

    let mut terms = by_term
        .into_iter()
        .map(|(canonical, mut mentions)| {
            if let Some(first) = mentions.first_mut() {
                first.is_first_mention = true;
            }
            DocumentTerm {
                canonical,
                first_span: mentions.first().map(|mention| mention.span).unwrap_or(Span::new(0, 0)),
                frequency: mentions.len(),
                mentions,
            }
        })
        .collect::<Vec<_>>();
    terms.sort_by_key(|term| (std::cmp::Reverse(term.frequency), term.first_span.start, term.canonical.clone()));
    terms
}

fn normalize_document_term(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphabetic() || *ch == '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_document_term_candidate(value: &str) -> bool {
    value.chars().count() >= 4 && !document_stop_words().contains(&value)
}

fn document_stop_words() -> &'static [&'static str] {
    &["который", "которая", "которые", "потому", "если", "чтобы", "этого", "этот", "эта", "это", "были", "было", "есть", "будет", "можно", "нужно"]
}

fn sentence_index_for_span(span: Span, sentences: &[SentenceSpan]) -> Option<usize> {
    sentences
        .iter()
        .position(|sentence| sentence.start <= span.start && span.end <= sentence.end)
}

fn cross_sentence_facts_from_terms_and_abbreviations(
    terms: &[DocumentTerm],
    abbreviations: &[DocumentAbbreviation],
) -> Vec<CrossSentenceFact> {
    let mut out = terms
        .iter()
        .filter_map(cross_sentence_repeated_term_fact)
        .collect::<Vec<_>>();
    out.extend(
        abbreviations
            .iter()
            .filter_map(cross_sentence_abbreviation_fact),
    );
    out
}

fn cross_sentence_repeated_term_fact(term: &DocumentTerm) -> Option<CrossSentenceFact> {
    let mut sentence_indices = term
        .mentions
        .iter()
        .filter_map(|mention| mention.sentence_index)
        .collect::<Vec<_>>();
    sentence_indices.sort_unstable();
    sentence_indices.dedup();
    (sentence_indices.len() > 1).then(|| CrossSentenceFact {
        key: "repeated_term_across_sentences".to_owned(),
        value: term.canonical.clone(),
        spans: term.mentions.iter().map(|mention| mention.span).collect(),
    })
}

fn cross_sentence_abbreviation_fact(abbr: &DocumentAbbreviation) -> Option<CrossSentenceFact> {
    let mut sentence_indices = abbr
        .mentions
        .iter()
        .filter_map(|mention| mention.sentence_index)
        .collect::<Vec<_>>();
    sentence_indices.sort_unstable();
    sentence_indices.dedup();
    (sentence_indices.len() > 1).then(|| CrossSentenceFact {
        key: "repeated_abbreviation_across_sentences".to_owned(),
        value: abbr.short.clone(),
        spans: abbr.mentions.iter().map(|mention| mention.span).collect(),
    })
}
