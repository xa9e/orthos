fn clitic_hyphen_missing_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    group: &str,
    message: &str,
) -> Result<Vec<Issue>> {
    let Some(group) = CliticHyphenGroup::parse(group) else {
        anyhow::bail!("unknown clitic hyphen group `{}` in rule `{}`", group, rule.id);
    };
    let mut out = Vec::new();

    for window in ctx.tokens().windows(3) {
        let [base, ws, particle] = window else { continue; };
        if base.kind != TokenKind::Word
            || ws.kind != TokenKind::Whitespace
            || ws.text.contains('\n')
            || particle.kind != TokenKind::Word
        {
            continue;
        }
        let Some(suggestion) = RussianCliticModel::suggest_missing_hyphen(base.text, particle.text, group) else {
            continue;
        };
        out.push(mk_issue(
            rule,
            ctx,
            Span::new(base.span.start, particle.span.end),
            message.to_owned(),
            Some(suggestion.replacement),
        ));
    }

    Ok(out)
}

fn negated_verb_spacing_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    let mut out = Vec::new();

    for token in ctx.word_tokens() {
        let Some(suggestion) = split_negated_verb_candidate(token.text, ctx.morph) else {
            continue;
        };
        out.push(mk_issue(
            rule,
            ctx,
            token.span,
            message.to_owned(),
            Some(suggestion.replacement),
        ));
    }

    Ok(out)
}
