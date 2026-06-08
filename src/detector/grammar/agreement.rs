fn adj_noun_agreement_demo_detector(rule: &Rule, ctx: &DetectorContext<'_>, message: &str) -> Result<Vec<Issue>> {
    agreement_graph_detector(rule, ctx, message, AgreementGraphEdgeKind::ModifierHead)
}

fn subject_predicate_agreement_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    agreement_graph_detector(rule, ctx, message, AgreementGraphEdgeKind::SubjectPredicate)
}

fn nominal_group_modifier_agreement_basic_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
) -> Result<Vec<Issue>> {
    agreement_graph_detector(rule, ctx, message, AgreementGraphEdgeKind::ModifierHead)
}

fn agreement_graph_detector(
    rule: &Rule,
    ctx: &DetectorContext<'_>,
    message: &str,
    kind: AgreementGraphEdgeKind,
) -> Result<Vec<Issue>> {
    let mut out = Vec::new();
    let mut reported = BTreeSet::new();

    for edge in ctx.fact_store().agreement_graph().conflicts_by_kind(kind) {
        if !reported.insert((edge.span.start, edge.span.end, edge.kind)) {
            continue;
        }
        let mut issue = mk_issue(rule, ctx, edge.span, message.to_owned(), None);
        issue.proof = Some(agreement_edge_proof(edge));
        out.push(issue);
    }

    Ok(out)
}
