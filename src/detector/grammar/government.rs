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
        issue.replacement = government_frame_replacement(ctx, frame);
        out.push(issue);
    }

    Ok(out)
}

fn government_frame_replacement(
    ctx: &DetectorContext<'_>,
    frame: &GovernmentFrame<'_>,
) -> Option<String> {
    if frame.expected_cases.is_empty() {
        return None;
    }
    let replacement = governed_token_replacement(ctx, frame)?;
    Some(replace_governed_token_inside_span(
        ctx.text,
        frame.span,
        frame.dependent.token.span,
        &replacement,
    ))
}

fn governed_token_replacement(
    ctx: &DetectorContext<'_>,
    frame: &GovernmentFrame<'_>,
) -> Option<String> {
    let mut candidates = BTreeSet::new();
    for observed in &frame.dependent.analyses {
        for candidate in ctx.morph.analyses_for_lemma(&observed.lemma) {
            if !analysis_matches_government_target(
                &candidate,
                observed,
                &frame.expected_cases,
                &frame.expected_numbers,
            ) {
                continue;
            }
            candidates.insert(match_case(frame.dependent.token.text, &candidate.form));
        }
    }

    if candidates.len() == 1 {
        candidates.into_iter().next()
    } else {
        None
    }
}

fn analysis_matches_government_target(
    candidate: &crate::morph::MorphAnalysis,
    observed: &crate::morph::MorphAnalysis,
    expected_cases: &[crate::morph::Case],
    expected_numbers: &[crate::morph::Number],
) -> bool {
    candidate.lemma == observed.lemma
        && candidate.pos == observed.pos
        && candidate
            .features
            .case
            .is_some_and(|case| expected_cases.contains(&case))
        && target_number_matches(candidate.features.number, observed.features.number, expected_numbers)
        && same_if_known(candidate.features.gender, observed.features.gender)
        && same_if_known(candidate.features.animacy, observed.features.animacy)
        && same_if_known(candidate.features.adjective_form, observed.features.adjective_form)
        && same_if_known(candidate.features.degree, observed.features.degree)
        && same_dictionary_ref(candidate.lemma_id.as_ref(), observed.lemma_id.as_ref())
        && same_dictionary_ref(candidate.paradigm_id.as_ref(), observed.paradigm_id.as_ref())
        && lower_ru(&candidate.form) != lower_ru(&observed.form)
}

fn target_number_matches(
    candidate: Option<crate::morph::Number>,
    observed: Option<crate::morph::Number>,
    expected_numbers: &[crate::morph::Number],
) -> bool {
    if expected_numbers.is_empty() {
        return same_if_known(candidate, observed);
    }
    candidate.is_some_and(|number| expected_numbers.contains(&number))
}

fn same_if_known<T: Eq>(left: Option<T>, right: Option<T>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left == right,
        _ => true,
    }
}

fn same_dictionary_ref<T: Eq>(left: Option<&T>, right: Option<&T>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left == right,
        _ => true,
    }
}

fn replace_governed_token_inside_span(
    text: &str,
    span: Span,
    token_span: Span,
    replacement: &str,
) -> String {
    format!(
        "{}{}{}",
        &text[span.start..token_span.start],
        replacement,
        &text[token_span.end..span.end]
    )
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
