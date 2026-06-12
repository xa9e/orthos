// Fixed expressions of the form `и X и Y` / `ни X ни Y`.
//
// The norm (Lopatin; Rozental §13 п.4) writes these without an internal
// comma, unlike ordinary repeated conjunctions: «и день и ночь»,
// «ни слуху ни духу». The inventory is a curated seed keyed by ё-folded
// lowercase forms so spelling variants keep matching.

/// `(conjunction, first member, second member)`, all in lookup-key form.
const PHRASEOLOGICAL_REPEATED_PAIRS: &[(&str, &str, &str)] = &[
    ("и", "так", "сяк"),
    ("и", "так", "эдак"),
    ("и", "так", "этак"),
    ("и", "день", "ночь"),
    ("и", "стар", "млад"),
    ("и", "смех", "грех"),
    ("и", "смех", "горе"),
    ("и", "холод", "голод"),
    ("и", "туда", "сюда"),
    ("ни", "слуху", "духу"),
    ("ни", "свет", "заря"),
    ("ни", "рыба", "мясо"),
    ("ни", "ответа", "привета"),
    ("ни", "конца", "края"),
    ("ни", "конца", "краю"),
    ("ни", "да", "нет"),
    ("ни", "дать", "взять"),
    ("ни", "себе", "людям"),
    ("ни", "жив", "мертв"),
    ("ни", "взад", "вперед"),
    ("ни", "сном", "духом"),
    ("ни", "пуха", "пера"),
    ("ни", "много", "мало"),
    ("ни", "больше", "меньше"),
    ("ни", "тпру", "ну"),
    ("ни", "встать", "сесть"),
    ("ни", "туда", "сюда"),
];

fn is_phraseological_repeated_pair(conjunction: &str, first: &str, second: &str) -> bool {
    PHRASEOLOGICAL_REPEATED_PAIRS
        .iter()
        .any(|(conj, a, b)| *conj == conjunction && *a == first && *b == second)
}

/// A comma wrongly written inside a phraseological repeated-conjunction pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhraseologicalPairComma {
    /// Token index of the offending comma.
    pub comma_token_index: usize,
    /// Span of the comma itself.
    pub comma_span: Span,
    /// Span of the whole `и X, и Y` expression.
    pub pair_span: Span,
}

/// Finds commas inside known `и X, и Y` / `ни X, ни Y` phraseological pairs.
pub fn phraseological_pair_commas(tokens: &[Token<'_>]) -> Vec<PhraseologicalPairComma> {
    let significant = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| token.kind != TokenKind::Whitespace)
        .map(|(idx, _)| idx)
        .collect::<Vec<_>>();

    let mut out = Vec::new();
    for window in significant.windows(5) {
        let [conj1, first, comma, conj2, second] = window else {
            continue;
        };
        let conj1_token = &tokens[*conj1];
        let comma_token = &tokens[*comma];
        if conj1_token.kind != TokenKind::Word
            || comma_token.kind != TokenKind::Punctuation
            || comma_token.text != ","
        {
            continue;
        }
        let conj_key = morph_lookup_key(conj1_token.text);
        if morph_lookup_key(tokens[*conj2].text) != conj_key {
            continue;
        }
        if tokens[*first].kind != TokenKind::Word || tokens[*second].kind != TokenKind::Word {
            continue;
        }
        if is_phraseological_repeated_pair(
            &conj_key,
            &morph_lookup_key(tokens[*first].text),
            &morph_lookup_key(tokens[*second].text),
        ) {
            out.push(PhraseologicalPairComma {
                comma_token_index: *comma,
                comma_span: comma_token.span,
                pair_span: Span::new(conj1_token.span.start, tokens[*second].span.end),
            });
        }
    }
    out
}
