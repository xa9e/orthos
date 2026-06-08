fn preposition_government_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    government_frame_detector(
        rule,
        ctx,
        message,
        GovernmentFrameKind::Preposition,
        GovernmentSpanShape::Adjacent,
    )
}

fn preposition_nominal_group_government_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    government_frame_detector(
        rule,
        ctx,
        message,
        GovernmentFrameKind::Preposition,
        GovernmentSpanShape::GovernedGroup,
    )
}

fn verb_government_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    government_frame_detector(
        rule,
        ctx,
        message,
        GovernmentFrameKind::Verb,
        GovernmentSpanShape::Any,
    )
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GovernmentSpanShape {
    Adjacent,
    GovernedGroup,
    Any,
}

fn government_frame_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
    kind: GovernmentFrameKind,
    shape: GovernmentSpanShape,
) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    let mut reported = BTreeSet::new();

    for frame in ctx
        .fact_store()
        .government_frames()
        .iter()
        .filter(|frame| frame.kind == kind)
    {
        if !frame.is_conflict() {
            continue;
        }
        if !government_frame_matches_span_shape(frame, ctx.tokens(), shape) {
            continue;
        }
        if !has_nominal_analysis(&frame.dependent.analyses) {
            continue;
        }
        if !reported.insert((frame.span.start, frame.span.end, frame.kind)) {
            continue;
        }
        let mut issue = mk_issue(rule, ctx, frame.span, message.to_owned(), None);
        issue.proof = Some(government_frame_proof(frame));
        out.push(issue);
    }

    Ok(out)
}

fn government_frame_matches_span_shape<'a>(
    frame: &GovernmentFrame<'a>,
    tokens: &[Token<'a>],
    shape: GovernmentSpanShape,
) -> bool {
    let words = word_tokens_in_span(tokens, frame.span);
    let Some(first) = words.first() else { return false; };
    let Some(last) = words.last() else { return false; };
    if last.span != frame.dependent.token.span {
        return false;
    }

    match shape {
        GovernmentSpanShape::Adjacent => {
            words.len() == 2 && first.span == frame.governor.token.span
        }
        GovernmentSpanShape::GovernedGroup => {
            words.len() > 2 && first.span == frame.governor.token.span
        }
        GovernmentSpanShape::Any => words
            .iter()
            .any(|word| word.span == frame.governor.token.span),
    }
}

fn has_nominal_analysis(analyses: &[crate::morph::MorphAnalysis]) -> bool {
    !analyses.is_empty()
        && analyses.iter().all(|analysis| {
            matches!(
                analysis.pos,
                crate::morph::PartOfSpeech::Noun
                    | crate::morph::PartOfSpeech::Pronoun
                    | crate::morph::PartOfSpeech::Adjective
                    | crate::morph::PartOfSpeech::Numeral
                    | crate::morph::PartOfSpeech::Participle
            )
        })
}
