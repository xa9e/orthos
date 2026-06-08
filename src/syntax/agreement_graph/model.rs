#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum AgreementGraphEdgeKind {
    ModifierHead,
    SubjectPredicate,
    NumeralHead,
    Apposition,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AgreementGraphEdge<'a> {
    pub kind: AgreementGraphEdgeKind,
    pub left: MorphosyntacticTerm<'a>,
    pub right: MorphosyntacticTerm<'a>,
    pub span: Span,
    pub constraint: MorphosyntacticConstraint,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SuppressionReason>,
}

impl<'a> AgreementGraphEdge<'a> {
    pub fn is_conflict(&self) -> bool {
        self.confidence.is_actionable()
            && self.blockers.is_empty()
            && self.constraint.is_confident_rejection()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AgreementGraph<'a> {
    edges: Vec<AgreementGraphEdge<'a>>,
}

impl<'a> AgreementGraph<'a> {
    pub fn from_facts(
        morphosyntax: &MorphosyntaxDocument<'a>,
        clauses: &[ClauseCandidate<'a>],
        nominal_groups: &[NominalGroupCandidate<'a>],
        ambiguity: &AmbiguityModel<'a>,
        islands: &SyntacticIslandMap,
    ) -> Self {
        let mut edges = Vec::new();
        edges.extend(
            morphosyntax
                .relations()
                .iter()
                .filter_map(|relation| agreement_edge_from_relation(relation, islands)),
        );
        edges.extend(
            clauses
                .iter()
                .filter_map(|clause| agreement_edge_from_clause(clause, islands)),
        );
        edges.extend(
            nominal_groups
                .iter()
                .flat_map(|group| agreement_edges_from_nominal_group(group, ambiguity, islands)),
        );
        dedup_agreement_edges(&mut edges);
        Self { edges }
    }

    pub fn from_morphosyntax(document: &MorphosyntaxDocument<'a>) -> Self {
        let edges = document
            .relations()
            .iter()
            .filter_map(|relation| agreement_edge_from_relation_without_islands(relation))
            .collect();
        Self { edges }
    }

    pub fn edges(&self) -> &[AgreementGraphEdge<'a>] {
        &self.edges
    }

    pub fn conflicts(&self) -> impl Iterator<Item = &AgreementGraphEdge<'a>> {
        self.edges.iter().filter(|edge| edge.is_conflict())
    }

    pub fn edges_for_token(&self, token_index: usize) -> impl Iterator<Item = &AgreementGraphEdge<'a>> {
        self.edges.iter().filter(move |edge| {
            edge.left.token_index == token_index || edge.right.token_index == token_index
        })
    }

    pub fn conflicts_by_kind(
        &self,
        kind: AgreementGraphEdgeKind,
    ) -> impl Iterator<Item = &AgreementGraphEdge<'a>> {
        self.conflicts().filter(move |edge| edge.kind == kind)
    }
}

fn agreement_edge_from_relation<'a>(
    relation: &MorphosyntacticRelation<'a>,
    islands: &SyntacticIslandMap,
) -> Option<AgreementGraphEdge<'a>> {
    let mut edge = agreement_edge_from_relation_without_islands(relation)?;
    edge.blockers.extend(boundary_blockers_for_link(
        relation.governor.token_index,
        relation.dependent.token_index,
        islands,
    ));
    finalize_edge_safety(&mut edge);
    Some(edge)
}

fn agreement_edge_from_relation_without_islands<'a>(
    relation: &MorphosyntacticRelation<'a>,
) -> Option<AgreementGraphEdge<'a>> {
    let kind = match relation.kind {
        MorphosyntacticRelationKind::AttributiveAgreement => AgreementGraphEdgeKind::ModifierHead,
        MorphosyntacticRelationKind::SubjectPredicateAgreement => AgreementGraphEdgeKind::SubjectPredicate,
        MorphosyntacticRelationKind::NumeralCaseNumberGovernment => AgreementGraphEdgeKind::NumeralHead,
        MorphosyntacticRelationKind::PrepositionCaseGovernment => return None,
    };
    Some(AgreementGraphEdge {
        kind,
        left: relation.governor.clone(),
        right: relation.dependent.clone(),
        span: relation.span,
        constraint: relation.constraint.clone(),
        confidence: relation.confidence,
        blockers: relation.blockers.iter().copied().map(SuppressionReason::from).collect(),
    })
}

fn agreement_edge_from_clause<'a>(
    clause: &ClauseCandidate<'a>,
    islands: &SyntacticIslandMap,
) -> Option<AgreementGraphEdge<'a>> {
    let subject = clause.subject_candidate.as_ref()?;
    let predicate = clause.predicate.as_ref()?;
    let mut blockers = clause.blockers.clone();
    blockers.extend(boundary_blockers_for_link(subject.token_index, predicate.token_index, islands));
    blockers.sort_unstable();
    blockers.dedup();

    let constraint = MorphosyntacticConstraint::Agreement(agreement_check_for_analyses(
        crate::morph::AgreementRelationKind::SubjectPredicate,
        &subject.analyses,
        &predicate.analyses,
    ));
    let mut edge = AgreementGraphEdge {
        kind: AgreementGraphEdgeKind::SubjectPredicate,
        left: clause_term_to_morphosyntactic_term(subject, MorphosyntacticRole::Subject),
        right: clause_term_to_morphosyntactic_term(predicate, MorphosyntacticRole::Predicate),
        span: Span::new(subject.token.span.start, predicate.token.span.end),
        constraint,
        confidence: clause.confidence,
        blockers,
    };
    finalize_edge_safety(&mut edge);
    Some(edge)
}

fn agreement_edges_from_nominal_group<'a>(
    group: &NominalGroupCandidate<'a>,
    ambiguity: &AmbiguityModel<'a>,
    islands: &SyntacticIslandMap,
) -> Vec<AgreementGraphEdge<'a>> {
    if !nominal_group_roles_are_plausible(group, &|idx| ambiguity.analyses_for_token(idx).to_vec()) {
        return Vec::new();
    }
    let head_analyses = ambiguity.analyses_for_token(group.end_token).to_vec();
    if head_analyses.is_empty() {
        return Vec::new();
    }

    group
        .modifiers
        .iter()
        .filter_map(|modifier| {
            let modifier_index = token_index_by_span_in_ambiguity(ambiguity, modifier.span)?;
            let modifier_analyses = ambiguity.analyses_for_token(modifier_index).to_vec();
            if modifier_analyses.is_empty() {
                return None;
            }
            let mut blockers = group
                .blockers
                .iter()
                .copied()
                .map(SuppressionReason::from)
                .collect::<Vec<_>>();
            blockers.extend(boundary_blockers_for_link(modifier_index, group.end_token, islands));
            blockers.sort_unstable();
            blockers.dedup();

            let constraint = MorphosyntacticConstraint::Agreement(agreement_check_for_analyses(
                crate::morph::AgreementRelationKind::AdjectiveNoun,
                &modifier_analyses,
                &head_analyses,
            ));
            let mut edge = AgreementGraphEdge {
                kind: AgreementGraphEdgeKind::ModifierHead,
                left: MorphosyntacticTerm {
                    token_index: modifier_index,
                    token: modifier.clone(),
                    role: MorphosyntacticRole::Modifier,
                    analyses: modifier_analyses,
                },
                right: MorphosyntacticTerm {
                    token_index: group.end_token,
                    token: group.head.clone(),
                    role: MorphosyntacticRole::Head,
                    analyses: head_analyses.clone(),
                },
                span: Span::new(modifier.span.start, group.head.span.end),
                constraint,
                confidence: group.confidence,
                blockers,
            };
            finalize_edge_safety(&mut edge);
            Some(edge)
        })
        .collect()
}

fn clause_term_to_morphosyntactic_term<'a>(
    term: &ClauseTerm<'a>,
    role: MorphosyntacticRole,
) -> MorphosyntacticTerm<'a> {
    MorphosyntacticTerm {
        token_index: term.token_index,
        token: term.token.clone(),
        role,
        analyses: term.analyses.clone(),
    }
}

fn boundary_blockers_for_link(
    left_token: usize,
    right_token: usize,
    islands: &SyntacticIslandMap,
) -> Vec<SuppressionReason> {
    if islands.can_link_tokens(left_token, right_token) {
        return Vec::new();
    }
    let left_kind = islands.island_for_token(left_token).map(|island| island.kind);
    let right_kind = islands.island_for_token(right_token).map(|island| island.kind);
    match (left_kind, right_kind) {
        (Some(SyntacticIslandKind::DirectSpeech), _) | (_, Some(SyntacticIslandKind::DirectSpeech)) => {
            vec![SuppressionReason::DirectSpeechBoundary]
        }
        (Some(SyntacticIslandKind::Parenthetical), _) | (_, Some(SyntacticIslandKind::Parenthetical)) => {
            vec![SuppressionReason::ParenthesisBoundary]
        }
        _ => vec![SuppressionReason::UnsafeBoundary],
    }
}

fn finalize_edge_safety(edge: &mut AgreementGraphEdge<'_>) {
    edge.blockers.sort_unstable();
    edge.blockers.dedup();
    if !edge.blockers.is_empty() && edge.confidence.is_actionable() {
        edge.confidence = SyntaxConfidence::Ambiguous;
    }
}

fn token_index_by_span_in_ambiguity(ambiguity: &AmbiguityModel<'_>, span: Span) -> Option<usize> {
    ambiguity
        .token_ambiguities()
        .iter()
        .find(|item| item.token.span == span)
        .map(|item| item.token_index)
}

fn dedup_agreement_edges(edges: &mut Vec<AgreementGraphEdge<'_>>) {
    let mut seen = BTreeSet::new();
    edges.retain(|edge| {
        seen.insert((
            edge.kind,
            edge.left.token.span.start,
            edge.left.token.span.end,
            edge.right.token.span.start,
            edge.right.token.span.end,
            edge.span.start,
            edge.span.end,
        ))
    });
}
