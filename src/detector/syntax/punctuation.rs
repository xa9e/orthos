fn missing_comma_before_subordinator_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    markers: &[String],
    message: &str,
) -> Result<Vec<Issue>> {
    let marker_set: HashSet<String> = markers.iter().map(|s| lower_ru(s)).collect();
    let document = SyntaxDocument::with_clause_markers(ctx.text, &marker_set);
    let mut out = Vec::new();

    for boundary in document.clause_boundaries() {
        if boundary.kind != ClauseBoundaryKind::BeforeMarker {
            continue;
        }
        if !boundary.confidence.is_actionable() {
            continue;
        }

        let token = &document.tokens()[boundary.marker.start_token];
        out.push(mk_issue(
            rule,
            ctx,
            Span::new(token.span.start, token.span.end),
            message.to_owned(),
            Some(format!(", {}", token.text)),
        ));
    }
    Ok(out)
}

fn introductory_phrase_comma_detector(rule: &Rule, ctx: &DetectorContext<'_>, phrases: &[String], message: &str) -> Result<Vec<Issue>> {
    let phrase_set: HashSet<String> = phrases.iter().map(|s| lower_ru(s)).collect();
    let single_word_phrases = phrase_set
        .iter()
        .filter(|phrase| phrase.chars().all(|ch| ch.is_alphabetic() || ch == 'ё' || ch == 'Ё'))
        .cloned()
        .collect::<HashSet<_>>();
    let mut out = introductory_phrase_slots(rule, ctx, &single_word_phrases, message);
    out.extend(introductory_phrase_surface_fallback(
        rule,
        ctx,
        &phrase_set,
        &single_word_phrases,
        message,
    ));
    out.sort_by_key(|issue| (issue.span.start, issue.span.end, issue.rule_id.clone()));
    out.dedup_by_key(|issue| (issue.span.start, issue.span.end, issue.rule_id.clone()));
    Ok(out)
}

fn introductory_phrase_slots(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    single_word_phrases: &HashSet<String>,
    message: &str,
) -> Vec<Issue> {
    let starts = sentence_starts(ctx.text).into_iter().collect::<HashSet<_>>();
    ctx.fact_store()
        .punctuation_slots()
        .iter()
        .filter(|slot| single_word_phrases.contains(&lower_ru(slot.left_token.text)))
        .filter(|slot| slot.after_introductory_candidate)
        .filter(|slot| starts.contains(&slot.left_token.span.start))
        .filter(|slot| slot.missing_expected_mark(PunctuationMark::Comma))
        .map(|slot| {
            let mut issue = mk_issue(
                rule,
                ctx,
                slot.span,
                message.to_owned(),
                Some(", ".to_owned()),
            );
            issue.proof = Some(punctuation_slot_proof(slot, PunctuationMark::Comma));
            issue
        })
        .collect()
}

fn introductory_phrase_surface_fallback(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    phrase_set: &HashSet<String>,
    fact_store_phrases: &HashSet<String>,
    message: &str,
) -> Vec<Issue> {
    let starts = sentence_starts(ctx.text);
    let mut out = Vec::new();

    for start in starts {
        if is_inside_punctuation_safe_zone(ctx.text, start) {
            continue;
        }
        let tail = &ctx.text[start..];
        let tail_lower = lower_ru(tail);
        for phrase in phrase_set {
            if fact_store_phrases.contains(phrase) {
                continue;
            }
            if !tail_lower.starts_with(phrase) {
                continue;
            }
            let phrase_end = start + phrase.len();
            if !is_boundary_after(ctx.text, phrase_end) {
                continue;
            }
            let Some((next_idx, next_ch)) = next_non_ws_char(ctx.text, phrase_end) else { continue; };
            if next_ch != ',' {
                out.push(mk_issue(
                    rule,
                    ctx,
                    Span::new(next_idx, next_idx + next_ch.len_utf8()),
                    message.to_owned(),
                    Some(format!(", {next_ch}")),
                ));
            }
        }
    }
    out
}

fn unbalanced_quotes_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut out = Vec::new();
    for (idx, ch) in ctx.text.char_indices() {
        match ch {
            '«' => stack.push(('«', idx)),
            '»' => {
                if let Some(('«', _)) = stack.last() {
                    stack.pop();
                } else {
                    out.push(mk_issue(rule, ctx, Span::new(idx, idx + ch.len_utf8()), message.to_owned(), None));
                }
            }
            _ => {}
        }
    }
    for (_, idx) in stack {
        out.push(mk_issue(rule, ctx, Span::new(idx, idx + '«'.len_utf8()), message.to_owned(), None));
    }
    Ok(out)
}

fn unpaired_delimiters_detector(rule: &Rule, ctx: &DetectorContext<'_>, pairs: &[String], message: &str) -> Result<Vec<Issue>> {
    let delimiter_pairs = parse_delimiter_pairs(pairs);
    let open_to_close: HashMap<char, char> = delimiter_pairs.iter().copied().collect();
    let close_to_open: HashMap<char, char> = delimiter_pairs.iter().map(|(o, c)| (*c, *o)).collect();
    let mut stack: Vec<(char, usize)> = Vec::new();
    let mut out = Vec::new();

    for (idx, ch) in ctx.text.char_indices() {
        if open_to_close.contains_key(&ch) {
            stack.push((ch, idx));
            continue;
        }
        if let Some(expected_open) = close_to_open.get(&ch) {
            match stack.pop() {
                Some((open, _)) if &open == expected_open => {}
                _ => out.push(mk_issue(rule, ctx, Span::new(idx, idx + ch.len_utf8()), message.to_owned(), None)),
            }
        }
    }

    for (open, idx) in stack {
        out.push(mk_issue(rule, ctx, Span::new(idx, idx + open.len_utf8()), message.to_owned(), None));
    }
    Ok(out)
}


fn coordination_comma_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    Ok(ctx
        .fact_store()
        .punctuation_slots()
        .iter()
        .filter(|slot| slot.missing_expected_mark(PunctuationMark::Comma))
        .filter(|slot| {
            slot.evidence
                .iter()
                .any(|item| item.kind == crate::syntax::PunctuationSlotEvidenceKind::Coordination)
        })
        .map(|slot| {
            let mut issue = mk_issue(rule, ctx, slot.span, message.to_owned(), Some(", ".to_owned()));
            issue.proof = Some(punctuation_slot_proof(slot, PunctuationMark::Comma));
            issue
        })
        .collect())
}
