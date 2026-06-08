fn mixed_alphabet_word_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    Ok(ctx.word_tokens()
        .iter()
        .filter(|token| has_cyrillic(token.text) && has_latin(token.text))
        .map(|token| mk_issue(rule, ctx, token.span, message.to_owned(), None))
        .collect())
}

fn confusable_tokens_detector(rule: &Rule, ctx: &DetectorContext<'_>, forms: &[String], message: &str) -> Result<Vec<Issue>> {
    token_map_detector(rule, ctx, forms, message, true)
}

fn phrase_map_detector(rule: &Rule, ctx: &DetectorContext<'_>, forms: &[String], message: &str) -> Result<Vec<Issue>> {
    let replacements = parse_replacement_map(forms);
    let text_lower = lower_ru(ctx.text);
    let mut out = Vec::new();
    for (bad, good) in replacements {
        let mut search_from = 0usize;
        while let Some(local) = text_lower[search_from..].find(&bad) {
            let start = search_from + local;
            let end = start + bad.len();
            if phrase_boundary(ctx.text, start, end) {
                out.push(mk_issue(rule, ctx, Span::new(start, end), message.to_owned(), Some(good.clone())));
            }
            search_from = end;
        }
    }
    Ok(out)
}

fn token_map_detector(rule: &Rule, ctx: &DetectorContext<'_>, forms: &[String], message: &str, match_case_flag: bool) -> Result<Vec<Issue>> {
    let replacements = parse_replacement_map(forms);
    let mut out = Vec::new();
    for token in ctx.word_tokens() {
        let normalized = normalize_word(token.text);
        if let Some(replacement) = replacements.get(&normalized) {
            let fixed = if match_case_flag { match_case(token.text, replacement) } else { replacement.to_owned() };
            out.push(mk_issue(rule, ctx, token.span, message.to_owned(), Some(fixed)));
        }
    }
    Ok(out)
}

fn zhi_shi_cha_shcha_detector(rule: &Rule, ctx: &DetectorContext<'_>, exceptions: &[String], message: &str) -> Result<Vec<Issue>> {
    let exception_set: HashSet<String> = exceptions.iter().map(|s| lower_ru(s)).collect();
    let replacements = [("жы", "жи"), ("шы", "ши"), ("чя", "ча"), ("щя", "ща"), ("чю", "чу"), ("щю", "щу")];
    let mut out = Vec::new();

    for token in ctx.word_tokens() {
        let normalized = normalize_word(token.text);
        if exception_set.contains(&normalized) {
            continue;
        }
        let mut fixed = lower_ru(token.text);
        let mut changed = false;
        for (bad, good) in replacements {
            if fixed.contains(bad) {
                fixed = fixed.replace(bad, good);
                changed = true;
            }
        }
        if changed {
            out.push(mk_issue(rule, ctx, token.span, message.to_owned(), Some(match_case(token.text, &fixed))));
        }
    }

    Ok(out)
}

fn tsya_heuristic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    infinitive_triggers: &[String],
    finite_subjects: &[String],
    message: &str,
) -> Result<Vec<Issue>> {
    let infinitive_triggers: HashSet<String> = infinitive_triggers.iter().map(|s| lower_ru(s)).collect();
    let finite_subjects: HashSet<String> = finite_subjects.iter().map(|s| lower_ru(s)).collect();
    let words = ctx.word_tokens();
    let mut out = Vec::new();

    for pair in words.windows(2) {
        let prev = normalize_word(pair[0].text);
        let current = normalize_word(pair[1].text);
        if infinitive_triggers.contains(&prev) && current.ends_with("тся") {
            let fixed = replace_suffix(pair[1].text, "тся", "ться");
            out.push(mk_issue(rule, ctx, pair[1].span, message.to_owned(), Some(fixed)));
        } else if finite_subjects.contains(&prev) && current.ends_with("ться") {
            let fixed = replace_suffix(pair[1].text, "ться", "тся");
            out.push(mk_issue(rule, ctx, pair[1].span, message.to_owned(), Some(fixed)));
        }
    }

    Ok(out)
}
