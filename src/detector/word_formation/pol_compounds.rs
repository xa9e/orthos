fn pol_compound_hyphen_missing_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    let mut out = Vec::new();

    for window in ctx.tokens().windows(3) {
        let [prefix, ws, head] = window else { continue; };
        if !is_separate_pol_candidate(prefix, ws, head) {
            continue;
        }
        out.push(mk_issue(
            rule,
            ctx,
            Span::new(prefix.span.start, head.span.end),
            message.to_owned(),
            Some(format!("{}-{}", prefix.text, head.text)),
        ));
    }

    Ok(out)
}

fn is_separate_pol_candidate(prefix: &Token<'_>, ws: &Token<'_>, head: &Token<'_>) -> bool {
    prefix.kind == TokenKind::Word
        && lower_ru(prefix.text) == "пол"
        && ws.kind == TokenKind::Whitespace
        && !ws.text.contains('\n')
        && head.kind == TokenKind::Word
        && starts_hyphenated_pol_head(head.text)
}

fn starts_hyphenated_pol_head(value: &str) -> bool {
    let Some(first) = value.chars().next() else {
        return false;
    };
    let mut lowered = first.to_lowercase();
    let Some(first_lower) = lowered.next() else {
        return false;
    };

    first.is_uppercase()
        || matches!(
            first_lower,
            'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я' | 'л'
        )
}

fn prefix_final_z_s_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let mut out = Vec::new();

    for token in ctx.word_tokens() {
        let Some(suggestion) = prefix_final_z_s_suggestion(token.text) else {
            continue;
        };
        out.push(mk_issue(
            rule,
            ctx,
            token.span,
            message.to_owned(),
            Some(match_case(token.text, &suggestion.replacement)),
        ));
    }

    Ok(out)
}
