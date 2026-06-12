fn numeral_noun_agreement_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    quantity_government_frame_detector(
        rule,
        ctx,
        message,
        QuantityFrameShape::AdjacentSingleNumeral,
    )
}

fn numeral_nominal_group_agreement_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    quantity_government_frame_detector(
        rule,
        ctx,
        message,
        QuantityFrameShape::SingleNumeralGroup,
    )
}

fn compound_numeral_nominal_group_agreement_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    quantity_government_frame_detector(
        rule,
        ctx,
        message,
        QuantityFrameShape::CompoundNumeral(2),
    )
}

fn typed_compound_numeral_nominal_group_agreement_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    quantity_government_frame_detector(
        rule,
        ctx,
        message,
        QuantityFrameShape::CompoundNumeral(3),
    )
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum QuantityFrameShape {
    AdjacentSingleNumeral,
    SingleNumeralGroup,
    CompoundNumeral(usize),
}

fn quantity_government_frame_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
    shape: QuantityFrameShape,
) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    let mut reported = BTreeSet::new();

    for frame in ctx
        .fact_store()
        .government_frames()
        .iter()
        .filter(|frame| frame.kind == GovernmentFrameKind::Numeral)
    {
        if !frame.is_conflict() {
            continue;
        }
        if !quantity_frame_matches_shape(frame, ctx.tokens(), ctx.morph, shape) {
            continue;
        }
        if !reported.insert((frame.span.start, frame.span.end, frame.governor.token.span.start)) {
            continue;
        }
        let mut issue = mk_issue(rule, ctx, frame.span, message.to_owned(), None);
        issue.proof = Some(government_frame_proof(frame));
        issue.replacement = government_frame_replacement(ctx, frame);
        out.push(issue);
    }

    Ok(out)
}

fn quantity_frame_matches_shape<'a>(
    frame: &GovernmentFrame<'a>,
    tokens: &[Token<'a>],
    morph: &dyn MorphAnalyzer,
    shape: QuantityFrameShape,
) -> bool {
    let words = word_tokens_in_span(tokens, frame.span);
    let Some(last) = words.last() else { return false; };
    if last.span != frame.dependent.token.span {
        return false;
    }

    let Some(numeral_prefix_len) = governing_numeral_prefix_len(&words, frame, morph) else {
        return false;
    };
    if words
        .get(numeral_prefix_len)
        .is_some_and(|token| token_is_unambiguous_numeral(token, morph))
    {
        return false;
    }
    if matches!(shape, QuantityFrameShape::CompoundNumeral(_))
        && preceding_word_is_unambiguous_numeral(tokens, frame.span, morph)
    {
        return false;
    }

    match shape {
        QuantityFrameShape::AdjacentSingleNumeral => numeral_prefix_len == 1 && words.len() == 2,
        QuantityFrameShape::SingleNumeralGroup => numeral_prefix_len == 1 && words.len() > 2,
        QuantityFrameShape::CompoundNumeral(expected_len) => numeral_prefix_len == expected_len,
    }
}

fn governing_numeral_prefix_len<'a>(
    words: &[&Token<'a>],
    frame: &GovernmentFrame<'a>,
    morph: &dyn MorphAnalyzer,
) -> Option<usize> {
    let governor_position = words
        .iter()
        .position(|token| token.span == frame.governor.token.span)?;
    let prefix = &words[..=governor_position];
    prefix
        .iter()
        .all(|token| token_is_unambiguous_numeral(token, morph))
        .then_some(prefix.len())
}

fn preceding_word_is_unambiguous_numeral<'a>(
    tokens: &[Token<'a>],
    span: Span,
    morph: &dyn MorphAnalyzer,
) -> bool {
    let words_in_span = word_tokens_in_span(tokens, span);
    let Some(&first_word) = words_in_span.first() else {
        return false;
    };
    let first_start = first_word.span.start;
    tokens
        .iter()
        .take_while(|token| token.span.start < first_start)
        .filter(|token| token.kind == TokenKind::Word)
        .last()
        .is_some_and(|token| token_is_unambiguous_numeral(token, morph))
}
