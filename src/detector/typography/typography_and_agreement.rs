fn space_after_opening_punctuation_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    for (idx, ch) in ctx.text.char_indices() {
        if !matches!(ch, '(' | '[' | '{' | '«') {
            continue;
        }
        let after_idx = idx + ch.len_utf8();
        let Some((space_idx, next_ch)) = char_after(ctx.text, after_idx) else { continue; };
        if next_ch == ' ' || next_ch == '\t' {
            out.push(mk_issue(
                rule,
                ctx,
                Span::new(space_idx, space_idx + next_ch.len_utf8()),
                message.to_owned(),
                Some(String::new()),
            ));
        }
    }
    Ok(out)
}

fn space_before_closing_quote_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    for (idx, ch) in ctx.text.char_indices() {
        if ch != '»' {
            continue;
        }
        let Some((space_idx, prev_ch)) = char_before(ctx.text, idx) else { continue; };
        if prev_ch == ' ' || prev_ch == '\t' {
            out.push(mk_issue(rule, ctx, Span::new(space_idx, idx), message.to_owned(), Some(String::new())));
        }
    }
    Ok(out)
}

fn hyphen_instead_of_dash_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    let needle = " - ";
    let mut search_from = 0usize;
    while let Some(local) = ctx.text[search_from..].find(needle) {
        let start = search_from + local;
        out.push(mk_issue(rule, ctx, Span::new(start, start + needle.len()), message.to_owned(), Some(" — ".to_string())));
        search_from = start + needle.len();
    }
    Ok(out)
}

fn dash_spacing_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    for (idx, ch) in ctx.text.char_indices() {
        if ch != '—' {
            continue;
        }
        let left_ok = char_before(ctx.text, idx).is_some_and(|(_, c)| c.is_whitespace());
        let right_idx = idx + ch.len_utf8();
        let right_ok = char_after(ctx.text, right_idx).is_some_and(|(_, c)| c.is_whitespace());
        if !left_ok || !right_ok {
            out.push(mk_issue(rule, ctx, Span::new(idx, right_idx), message.to_owned(), Some(" — ".to_string())));
        }
    }
    Ok(out)
}

fn multiple_punctuation_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let chars: Vec<(usize, char)> = ctx.text.char_indices().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let (start, ch) = chars[i];
        if ch != '!' && ch != '?' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < chars.len() && chars[j].1 == ch {
            j += 1;
        }
        if j - i > 1 {
            let end = if j < chars.len() { chars[j].0 } else { ctx.text.len() };
            out.push(mk_issue(rule, ctx, Span::new(start, end), message.to_owned(), Some(ch.to_string())));
        }
        i = j;
    }
    Ok(out)
}

fn number_unit_spacing_detector(rule: &Rule, ctx: &DetectorContext<'_>, units: &[String], message: &str) -> Result<Vec<Issue>> {
    let units: HashSet<String> = units.iter().map(|s| lower_ru(s)).collect();
    let tokens = ctx.tokens();
    let mut out = Vec::new();

    for pair in tokens.windows(2) {
        if pair[0].kind == TokenKind::Number && pair[1].kind == TokenKind::Word {
            let unit = normalize_word(pair[1].text);
            if units.contains(&unit) && pair[0].span.end == pair[1].span.start {
                out.push(mk_issue(
                    rule,
                    ctx,
                    Span::new(pair[1].span.start, pair[1].span.end),
                    message.to_owned(),
                    Some(format!(" {}", pair[1].text)),
                ));
            }
        }
    }

    Ok(out)
}
