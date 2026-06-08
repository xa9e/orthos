#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NegationSpacingSuggestion {
    pub replacement: String,
    pub lexical_base: String,
}

pub fn split_negated_verb_candidate(
    token: &str,
    analyzer: &dyn MorphAnalyzer,
) -> Option<NegationSpacingSuggestion> {
    let normalized = lower_ru(token);
    let base = normalized.strip_prefix("не")?;
    if base.chars().count() < 2 || is_lexicalized_ne_verb(&normalized) {
        return None;
    }

    if analyzer
        .analyze(&normalized)
        .iter()
        .any(|analysis| is_verbal_pos(analysis.pos))
    {
        return None;
    }

    analyzer
        .analyze(base)
        .iter()
        .any(|analysis| is_verbal_pos(analysis.pos))
        .then(|| NegationSpacingSuggestion {
            replacement: format!("не {}", match_case_after_ne(token, base)),
            lexical_base: base.to_owned(),
        })
}

fn is_verbal_pos(pos: PartOfSpeech) -> bool {
    matches!(pos, PartOfSpeech::Verb | PartOfSpeech::Gerund | PartOfSpeech::Participle)
}

fn is_lexicalized_ne_verb(value: &str) -> bool {
    matches!(
        value,
        "ненавидеть"
            | "ненавижу"
            | "ненавидишь"
            | "негодовать"
            | "нездоровиться"
            | "несдобровать"
            | "недоумевать"
            | "неистовствовать"
    )
}

fn match_case_after_ne(original: &str, lower_base: &str) -> String {
    let Some(prefix_end) = original.char_indices().nth(2).map(|(idx, _)| idx) else {
        return lower_base.to_owned();
    };
    let original_base = &original[prefix_end..];
    if original_base.chars().next().is_some_and(char::is_uppercase) {
        uppercase_first(lower_base)
    } else {
        lower_base.to_owned()
    }
}
