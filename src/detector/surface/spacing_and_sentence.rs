fn mk_issue(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    span: Span,
    message: String,
    replacement: Option<String>,
) -> Issue {
    Issue {
        rule_id: rule.id.clone(),
        severity: rule.severity,
        message,
        span,
        start: ctx.line_index.position(span.start),
        end: ctx.line_index.position(span.end),
        replacement,
        suggestion: rule.suggestion.clone(),
        explanation: rule.explanation.clone(),
        source_refs: rule.source_refs.clone(),
        excerpt: excerpt(ctx.text, span),
        proof: None,
    }
}

fn regex_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    pattern: &str,
    message: &str,
    replacement: Option<&str>,
) -> Result<Vec<Issue>> {
    let re = Regex::new(pattern).with_context(|| format!("bad regex in rule `{}`", rule.id))?;
    Ok(re
        .find_iter(ctx.text)
        .map(|m| {
            mk_issue(
                rule,
                ctx,
                Span::new(m.start(), m.end()),
                message.to_owned(),
                replacement.map(str::to_owned),
            )
        })
        .collect())
}

fn multiple_spaces_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str, replacement: &str) -> Result<Vec<Issue>> {
    Ok(MULTI_SPACE_RE
        .find_iter(ctx.text)
        .map(|m| mk_issue(rule, ctx, Span::new(m.start(), m.end()), message.to_owned(), Some(replacement.to_owned())))
        .collect())
}

fn no_whitespace_before_punctuation_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    punctuation: &str,
    message: &str,
) -> Result<Vec<Issue>> {
    let tokens = ctx.tokens();
    let punct: HashSet<char> = punctuation.chars().collect();
    let mut out = Vec::new();

    for window in tokens.windows(2) {
        let [ws, mark] = window else { continue; };
        if ws.kind == TokenKind::Whitespace
            && mark.kind == TokenKind::Punctuation
            && mark.text.chars().all(|c| punct.contains(&c))
            && !ws.text.contains('\n')
        {
            out.push(mk_issue(
                rule,
                ctx,
                Span::new(ws.span.start, mark.span.end),
                message.to_owned(),
                Some(mark.text.to_owned()),
            ));
        }
    }

    Ok(out)
}

fn missing_whitespace_after_punctuation_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    for (idx, ch) in ctx.text.char_indices() {
        if !is_punctuation_requiring_space_after(ch) {
            continue;
        }
        let after_idx = idx + ch.len_utf8();
        let Some((next_idx, next_ch)) = char_after(ctx.text, after_idx) else { continue; };
        if next_ch.is_whitespace() || is_closing_punctuation(next_ch) || is_punctuation_requiring_space_after(next_ch) {
            continue;
        }
        if ch == '.' && (decimal_context(ctx.text, idx, next_idx) || abbreviation_context(ctx.text, idx, next_idx)) {
            continue;
        }
        if next_ch.is_alphanumeric() || has_cyrillic(&next_ch.to_string()) {
            out.push(mk_issue(
                rule,
                ctx,
                Span::new(next_idx, next_idx + next_ch.len_utf8()),
                message.to_owned(),
                Some(format!(" {next_ch}")),
            ));
        }
    }
    Ok(out)
}

fn repeated_word_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let tokens = ctx.tokens();
    let mut out = Vec::new();
    let mut prev_word: Option<&Token<'_>> = None;

    for token in tokens {
        if token.kind != TokenKind::Word {
            if !matches!(token.kind, TokenKind::Whitespace) {
                prev_word = None;
            }
            continue;
        }

        if let Some(prev) = prev_word
            && lower_ru(prev.text) == lower_ru(token.text)
        {
            out.push(mk_issue(
                rule,
                ctx,
                Span::new(prev.span.start, token.span.end),
                message.to_owned(),
                Some(token.text.to_owned()),
            ));
        }
        prev_word = Some(token);
    }

    Ok(out)
}

fn missing_sentence_terminal_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str, replacement: &str) -> Result<Vec<Issue>> {
    let trimmed = ctx.text.trim_end();
    if trimmed.is_empty() || trimmed.chars().last().is_some_and(|c| matches!(c, '.' | '!' | '?' | '…')) {
        return Ok(Vec::new());
    }
    let span = Span::new(trimmed.len(), trimmed.len());
    Ok(vec![mk_issue(rule, ctx, span, message.to_owned(), Some(replacement.to_owned()))])
}

fn sentence_initial_lowercase_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    let tokens = ctx.tokens();

    for sentence_start in sentence_starts(ctx.text) {
        let Some(token) = tokens
            .iter()
            .find(|token| token.span.start >= sentence_start && token.kind == TokenKind::Word)
        else {
            continue;
        };

        let Some(first_char) = token.text.chars().next() else {
            continue;
        };
        if !first_char.is_lowercase() {
            continue;
        }

        let replacement: String = first_char.to_uppercase().collect();
        out.push(mk_issue(
            rule,
            ctx,
            Span::new(token.span.start, token.span.start + first_char.len_utf8()),
            message.to_owned(),
            Some(replacement),
        ));
    }

    Ok(out)
}

fn particle_hyphen_missing_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    bases: &[String],
    particles: &[String],
    message: &str,
) -> Result<Vec<Issue>> {
    let tokens = ctx.tokens();
    let base_set: HashSet<String> = bases.iter().map(|s| lower_ru(s)).collect();
    let particle_set: HashSet<String> = particles.iter().map(|s| lower_ru(s)).collect();
    let mut out = Vec::new();

    for i in 0..tokens.len().saturating_sub(2) {
        let base = &tokens[i];
        let ws = &tokens[i + 1];
        let particle = &tokens[i + 2];
        if base.kind == TokenKind::Word
            && ws.kind == TokenKind::Whitespace
            && !ws.text.contains('\n')
            && particle.kind == TokenKind::Word
            && base_set.contains(&lower_ru(base.text))
            && particle_set.contains(&lower_ru(particle.text))
        {
            out.push(mk_issue(
                rule,
                ctx,
                Span::new(base.span.start, particle.span.end),
                message.to_owned(),
                Some(format!("{}-{}", base.text, particle.text)),
            ));
        }
    }

    Ok(out)
}

fn separate_particle_hyphenated_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    Ok(HYPHENATED_PARTICLE_RE
        .captures_iter(ctx.text)
        .filter_map(|caps| {
            let whole = caps.get(0)?;
            let stem = caps.get(1)?.as_str();
            let particle = caps.get(2)?.as_str();
            Some(mk_issue(
                rule,
                ctx,
                Span::new(whole.start(), whole.end()),
                message.to_owned(),
                Some(format!("{} {}", stem, particle)),
            ))
        })
        .collect())
}
