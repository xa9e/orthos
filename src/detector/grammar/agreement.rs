fn adj_noun_agreement_demo_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    agreement_graph_detector(
        rule,
        ctx,
        message,
        AgreementGraphEdgeKind::ModifierHead,
        demo_adjacent_modifier_head_edge,
    )
}

fn subject_predicate_agreement_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    agreement_graph_detector(
        rule,
        ctx,
        message,
        AgreementGraphEdgeKind::SubjectPredicate,
        |_, _| true,
    )
}

fn nominal_group_modifier_agreement_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    agreement_graph_detector(
        rule,
        ctx,
        message,
        AgreementGraphEdgeKind::ModifierHead,
        nominal_group_modifier_edge,
    )
}

fn agreement_graph_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
    kind: AgreementGraphEdgeKind,
    should_report: impl Fn(&DetectorContext<'_>, &crate::syntax::AgreementGraphEdge<'_>) -> bool,
) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    let mut reported = BTreeSet::new();

    for edge in ctx.fact_store().agreement_graph().conflicts_by_kind(kind) {
        if !should_report(ctx, edge) {
            continue;
        }
        if !reported.insert((edge.span.start, edge.span.end, edge.kind)) {
            continue;
        }
        let mut issue = mk_issue(rule, ctx, edge.span, message.to_owned(), None);
        issue.proof = Some(agreement_edge_proof(edge));
        issue.replacement = match edge.kind {
            AgreementGraphEdgeKind::SubjectPredicate => subject_predicate_replacement(ctx, edge),
            AgreementGraphEdgeKind::ModifierHead => modifier_head_replacement(ctx, edge),
            _ => None,
        };
        out.push(issue);
    }

    Ok(out)
}

fn demo_adjacent_modifier_head_edge(
    ctx: &DetectorContext<'_>,
    edge: &crate::syntax::AgreementGraphEdge<'_>,
) -> bool {
    let words = word_tokens_in_span(ctx.tokens(), edge.span);
    if words.len() != 2 {
        return false;
    }
    let [left, right] = words.as_slice() else {
        return false;
    };
    if left.span != edge.left.token.span || right.span != edge.right.token.span {
        return false;
    }
    !word_before_span_blocks_demo_agreement(ctx.tokens(), edge.span, ctx.morph)
}

fn nominal_group_modifier_edge(
    ctx: &DetectorContext<'_>,
    edge: &crate::syntax::AgreementGraphEdge<'_>,
) -> bool {
    let words = word_tokens_in_span(ctx.tokens(), edge.span);
    words.len() > 2 && !word_before_span_blocks_demo_agreement(ctx.tokens(), edge.span, ctx.morph)
}

fn word_before_span_blocks_demo_agreement(
    tokens: &[Token<'_>],
    span: Span,
    morph: &dyn MorphAnalyzer,
) -> bool {
    tokens
        .iter()
        .take_while(|token| token.span.start < span.start)
        .filter(|token| token.kind == TokenKind::Word)
        .last()
        .is_some_and(|token| {
            token_is_unambiguous_numeral(token, morph)
                || token_has_unambiguous_pos(token, morph, PartOfSpeech::Preposition)
        })
}

fn modifier_head_replacement(
    ctx: &DetectorContext<'_>,
    edge: &crate::syntax::AgreementGraphEdge<'_>,
) -> Option<String> {
    let expected = expected_modifier_form(&edge.right.analyses)?;
    let mut candidates = BTreeSet::new();

    for observed in &edge.left.analyses {
        if !observed.pos.can_modify_noun() {
            continue;
        }
        for candidate in ctx.morph.analyses_for_lemma(&observed.lemma) {
            if !analysis_matches_modifier_target(&candidate, observed, expected) {
                continue;
            }
            candidates.insert(match_case(edge.left.token.text, &candidate.form));
        }
    }

    if candidates.len() == 1 {
        Some(replace_token_inside_span(
            ctx.text,
            edge.span,
            edge.left.token.span,
            &candidates.into_iter().next()?,
        ))
    } else {
        None
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct ExpectedModifierForm {
    case: crate::morph::Case,
    number: crate::morph::Number,
    gender: Option<crate::morph::Gender>,
}

fn expected_modifier_form(heads: &[crate::morph::MorphAnalysis]) -> Option<ExpectedModifierForm> {
    let case = unique_value(heads.iter().map(|analysis| analysis.features.case))?;
    let number = unique_value(heads.iter().map(|analysis| analysis.features.number))?;
    let gender = match number {
        crate::morph::Number::Singular => {
            Some(unique_value(heads.iter().map(|analysis| analysis.features.gender))?)
        }
        crate::morph::Number::Plural => None,
    };
    Some(ExpectedModifierForm {
        case,
        number,
        gender,
    })
}

fn analysis_matches_modifier_target(
    candidate: &crate::morph::MorphAnalysis,
    observed: &crate::morph::MorphAnalysis,
    expected: ExpectedModifierForm,
) -> bool {
    candidate.lemma == observed.lemma
        && candidate.pos == observed.pos
        && candidate.pos.can_modify_noun()
        && candidate.features.case == Some(expected.case)
        && candidate.features.number == Some(expected.number)
        && expected
            .gender
            .is_none_or(|gender| candidate.features.gender == Some(gender))
        && same_if_known_for_agreement(candidate.features.adjective_form, observed.features.adjective_form)
        && same_if_known_for_agreement(candidate.features.degree, observed.features.degree)
        && same_dictionary_ref_for_agreement(candidate.lemma_id.as_ref(), observed.lemma_id.as_ref())
        && same_dictionary_ref_for_agreement(candidate.paradigm_id.as_ref(), observed.paradigm_id.as_ref())
        && lower_ru(&candidate.form) != lower_ru(&observed.form)
}

fn subject_predicate_replacement(
    ctx: &DetectorContext<'_>,
    edge: &crate::syntax::AgreementGraphEdge<'_>,
) -> Option<String> {
    let expected = expected_subject_predicate_form(&edge.left.analyses)?;
    let predicate = edge
        .right
        .analyses
        .iter()
        .find(|analysis| is_past_finite_verb(analysis))?;
    let candidate = past_tense_predicate_candidates(edge.right.token.text, expected)
        .into_iter()
        .find(|candidate| {
            ctx.morph.analyze(candidate).iter().any(|analysis| {
                analysis.lemma == predicate.lemma
                    && edge.left.analyses.iter().any(|subject| {
                        crate::morph::subject_predicate_agreement_check(subject, analysis)
                            .compatibility
                            == crate::morph::MorphCompatibility::Compatible
                    })
            })
        })?;

    Some(replace_token_inside_span(
        ctx.text,
        edge.span,
        edge.right.token.span,
        &candidate,
    ))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct ExpectedPredicateForm {
    number: crate::morph::Number,
    gender: Option<crate::morph::Gender>,
}

fn expected_subject_predicate_form(
    subjects: &[crate::morph::MorphAnalysis],
) -> Option<ExpectedPredicateForm> {
    let number = unique_value(subjects.iter().map(|analysis| analysis.features.number))?;
    let gender = match number {
        crate::morph::Number::Singular => {
            Some(unique_value(subjects.iter().map(|analysis| analysis.features.gender))?)
        }
        crate::morph::Number::Plural => None,
    };
    Some(ExpectedPredicateForm { number, gender })
}

fn unique_value<T>(values: impl Iterator<Item = Option<T>>) -> Option<T>
where
    T: Copy + Eq,
{
    let mut found = None;
    for value in values {
        let value = value?;
        if found.is_some_and(|existing| existing != value) {
            return None;
        }
        found = Some(value);
    }
    found
}

fn same_if_known_for_agreement<T: Eq>(left: Option<T>, right: Option<T>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left == right,
        _ => true,
    }
}

fn same_dictionary_ref_for_agreement<T: Eq>(left: Option<&T>, right: Option<&T>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => left == right,
        _ => true,
    }
}

fn is_past_finite_verb(analysis: &crate::morph::MorphAnalysis) -> bool {
    analysis.pos == crate::morph::PartOfSpeech::Verb
        && analysis.features.tense == Some(crate::morph::Tense::Past)
        && !matches!(
            analysis.features.verb_form,
            Some(crate::morph::VerbForm::Infinitive)
        )
}

fn past_tense_predicate_candidates(
    surface: &str,
    expected: ExpectedPredicateForm,
) -> Vec<String> {
    let suffix = match (expected.number, expected.gender) {
        (crate::morph::Number::Plural, _) => "ли",
        (crate::morph::Number::Singular, Some(crate::morph::Gender::Feminine)) => "ла",
        (crate::morph::Number::Singular, Some(crate::morph::Gender::Neuter)) => "ло",
        (crate::morph::Number::Singular, Some(crate::morph::Gender::Masculine)) => "л",
        (crate::morph::Number::Singular, _) => return Vec::new(),
    };
    let mut candidates = BTreeSet::new();
    push_past_suffix_candidate(surface, suffix, &mut candidates);
    candidates.into_iter().collect()
}

fn push_past_suffix_candidate(surface: &str, suffix: &str, candidates: &mut BTreeSet<String>) {
    for source_suffix in ["ёл", "ел", "ла", "ло", "ли", "л"] {
        let lower = lower_ru(surface);
        if !lower.ends_with(source_suffix) {
            continue;
        }
        let replacement_suffix = match (source_suffix, suffix) {
            ("ёл" | "ел", "ла") => "ла",
            ("ёл" | "ел", "ло") => "ло",
            ("ёл" | "ел", "ли") => "ли",
            ("ла" | "ло" | "ли", "л") => "л",
            (_, value) => value,
        };
        let candidate = replace_suffix(surface, source_suffix, replacement_suffix);
        if lower_ru(&candidate) != lower {
            candidates.insert(candidate);
        }
    }
}

fn replace_token_inside_span(
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
