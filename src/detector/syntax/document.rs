fn document_abbreviation_expansion_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    Ok(document_abbreviation_candidates(ctx.fact_store().document_context())
        .into_iter()
        .map(|candidate| {
            let span = candidate.abbreviation.first_span;
            let mut issue = mk_issue(rule, ctx, span, message.to_owned(), None);
            issue.proof = Some(document_abbreviation_proof(
                candidate.abbreviation,
                candidate.reason,
            ));
            issue
        })
        .collect())
}

fn document_style_consistency_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    Ok(document_style_candidates(ctx.fact_store().document_context())
        .into_iter()
        .map(|candidate| {
            let mut issue = mk_issue(rule, ctx, candidate.span, message.to_owned(), None);
            issue.proof = Some(document_style_proof(&candidate));
            issue
        })
        .collect())
}
