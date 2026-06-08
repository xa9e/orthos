fn attributive_relations<'a>(
    tokens: &[Token<'a>],
    analyses_for_token: &impl Fn(usize) -> Vec<crate::morph::MorphAnalysis>,
) -> Vec<MorphosyntacticRelation<'a>> {
    adjacent_modifier_head_candidates(tokens)
        .into_iter()
        .filter_map(|candidate| {
            let left_index = token_index_by_span(tokens, candidate.left.span)?;
            let right_index = token_index_by_span(tokens, candidate.right.span)?;
            let left_analyses = analyses_for_token(left_index);
            let right_analyses = analyses_for_token(right_index);
            let constraint = MorphosyntacticConstraint::Agreement(agreement_check_for_analyses(
                crate::morph::AgreementRelationKind::AdjectiveNoun,
                &left_analyses,
                &right_analyses,
            ));
            Some(morphosyntactic_relation(
                MorphosyntacticRelationKind::AttributiveAgreement,
                MorphosyntacticRole::Modifier,
                left_index,
                candidate.left,
                left_analyses,
                MorphosyntacticRole::Head,
                right_index,
                candidate.right,
                right_analyses,
                candidate.span,
                candidate.confidence,
                candidate.blockers,
                constraint,
            ))
        })
        .collect()
}

fn subject_predicate_relations<'a>(
    tokens: &[Token<'a>],
    analyses_for_token: &impl Fn(usize) -> Vec<crate::morph::MorphAnalysis>,
) -> Vec<MorphosyntacticRelation<'a>> {
    adjacent_subject_predicate_candidates(tokens)
        .into_iter()
        .filter_map(|candidate| {
            let left_index = token_index_by_span(tokens, candidate.left.span)?;
            let right_index = token_index_by_span(tokens, candidate.right.span)?;
            let left_analyses = analyses_for_token(left_index);
            let right_analyses = analyses_for_token(right_index);
            let constraint = MorphosyntacticConstraint::Agreement(agreement_check_for_analyses(
                crate::morph::AgreementRelationKind::SubjectPredicate,
                &left_analyses,
                &right_analyses,
            ));
            Some(morphosyntactic_relation(
                MorphosyntacticRelationKind::SubjectPredicateAgreement,
                MorphosyntacticRole::Subject,
                left_index,
                candidate.left,
                left_analyses,
                MorphosyntacticRole::Predicate,
                right_index,
                candidate.right,
                right_analyses,
                candidate.span,
                candidate.confidence,
                candidate.blockers,
                constraint,
            ))
        })
        .collect()
}

fn preposition_government_relations<'a>(
    tokens: &[Token<'a>],
    analyses_for_token: &impl Fn(usize) -> Vec<crate::morph::MorphAnalysis>,
    preposition_registry: &crate::morph::PrepositionGovernmentRegistry,
) -> Vec<MorphosyntacticRelation<'a>> {
    preposition_governed_nominal_group_candidates(tokens, &preposition_registry.prepositions(), 3)
        .into_iter()
        .filter_map(|candidate| {
            if !nominal_group_roles_are_plausible(&candidate.group, analyses_for_token) {
                return None;
            }
            let left_index = candidate.governor_token;
            let right_index = candidate.group.end_token;
            let left_analyses = analyses_for_token(left_index);
            let right_analyses = analyses_for_token(right_index);
            let constraint = MorphosyntacticConstraint::Government(crate::morph::preposition_government_check(
                preposition_registry,
                candidate.governor.text,
                &right_analyses,
            ));
            Some(morphosyntactic_relation(
                MorphosyntacticRelationKind::PrepositionCaseGovernment,
                MorphosyntacticRole::CaseGovernor,
                left_index,
                candidate.governor,
                left_analyses,
                MorphosyntacticRole::GovernedNominal,
                right_index,
                candidate.group.head,
                right_analyses,
                candidate.span,
                candidate.confidence,
                candidate.blockers,
                constraint,
            ))
        })
        .collect()
}

fn numeral_government_relations<'a>(
    tokens: &[Token<'a>],
    analyses_for_token: &impl Fn(usize) -> Vec<crate::morph::MorphAnalysis>,
) -> Vec<MorphosyntacticRelation<'a>> {
    let mut relations = Vec::new();

    let compound_candidates = compound_numeral_nominal_group_candidates(tokens, 3, 3);
    let subsumed_pairs: BTreeSet<(usize, usize)> = compound_candidates
        .iter()
        .filter(|c| c.is_actionable())
        .filter_map(|c| {
            c.numeral_phrase.governing_component().map(|g| (g.token_index, c.group.end_token))
        })
        .collect();

    for candidate in numeral_nominal_group_candidates(tokens) {
        let Some(left_index) = token_index_by_span(tokens, candidate.left.span) else {
            continue;
        };
        let Some(right_index) = token_index_by_span(tokens, candidate.right.span) else {
            continue;
        };
        if subsumed_pairs.contains(&(left_index, right_index)) {
            continue;
        }
        let left_analyses = analyses_for_token(left_index);
        let right_analyses = analyses_for_token(right_index);
        let constraint = MorphosyntacticConstraint::Quantity(crate::morph::numeral_noun_agreement_check(
            &left_analyses,
            &right_analyses,
        ));
        relations.push(morphosyntactic_relation(
            MorphosyntacticRelationKind::NumeralCaseNumberGovernment,
            MorphosyntacticRole::Quantifier,
            left_index,
            candidate.left,
            left_analyses,
            MorphosyntacticRole::QuantifiedHead,
            right_index,
            candidate.right,
            right_analyses,
            candidate.span,
            candidate.confidence,
            candidate.blockers,
            constraint,
        ));
    }

    for candidate in &compound_candidates {
        if !candidate.is_actionable() {
            continue;
        }
        if subsumed_by_longer_compound(candidate, &compound_candidates) {
            continue;
        }
        let Some(governing_component) = candidate.numeral_phrase.governing_component() else {
            continue;
        };
        let left_index = governing_component.token_index;
        let right_index = candidate.group.end_token;
        let left_analyses = analyses_for_token(left_index);
        let right_analyses = analyses_for_token(right_index);
        let constraint = MorphosyntacticConstraint::Quantity(crate::morph::numeral_noun_agreement_check(
            &left_analyses,
            &right_analyses,
        ));
        relations.push(morphosyntactic_relation(
            MorphosyntacticRelationKind::NumeralCaseNumberGovernment,
            MorphosyntacticRole::Quantifier,
            left_index,
            governing_component.token.clone(),
            left_analyses,
            MorphosyntacticRole::QuantifiedHead,
            right_index,
            candidate.group.head.clone(),
            right_analyses,
            candidate.span,
            candidate.confidence,
            candidate.blockers.clone(),
            constraint,
        ));
    }

    relations
}

#[allow(clippy::too_many_arguments)]
fn morphosyntactic_relation<'a>(
    kind: MorphosyntacticRelationKind,
    governor_role: MorphosyntacticRole,
    governor_index: usize,
    governor_token: Token<'a>,
    governor_analyses: Vec<crate::morph::MorphAnalysis>,
    dependent_role: MorphosyntacticRole,
    dependent_index: usize,
    dependent_token: Token<'a>,
    dependent_analyses: Vec<crate::morph::MorphAnalysis>,
    span: Span,
    confidence: SyntaxConfidence,
    blockers: Vec<SyntaxRelationBlocker>,
    constraint: MorphosyntacticConstraint,
) -> MorphosyntacticRelation<'a> {
    MorphosyntacticRelation {
        kind,
        governor: MorphosyntacticTerm {
            token_index: governor_index,
            token: governor_token,
            role: governor_role,
            analyses: governor_analyses,
        },
        dependent: MorphosyntacticTerm {
            token_index: dependent_index,
            token: dependent_token,
            role: dependent_role,
            analyses: dependent_analyses,
        },
        span,
        confidence: downgrade_confidence_for_unknown_constraint(confidence, &constraint),
        blockers,
        constraint,
    }
}

fn downgrade_confidence_for_unknown_constraint(
    confidence: SyntaxConfidence,
    constraint: &MorphosyntacticConstraint,
) -> SyntaxConfidence {
    if matches!(constraint.compatibility(), crate::morph::MorphCompatibility::Unknown)
        && confidence.is_actionable()
    {
        SyntaxConfidence::Weak
    } else {
        confidence
    }
}

fn token_index_by_span(tokens: &[Token<'_>], span: Span) -> Option<usize> {
    tokens.iter().position(|token| token.span == span)
}

fn dedup_relations(relations: &mut Vec<MorphosyntacticRelation<'_>>) {
    let mut seen = BTreeSet::new();
    relations.retain(|relation| {
        seen.insert((
            relation.kind,
            relation.governor.token.span.start,
            relation.governor.token.span.end,
            relation.dependent.token.span.start,
            relation.dependent.token.span.end,
            relation.span.start,
            relation.span.end,
        ))
    });
}

fn subsumed_by_longer_compound<'a>(
    candidate: &QuantifiedNominalGroupCandidate<'a>,
    all: &[QuantifiedNominalGroupCandidate<'a>],
) -> bool {
    all.iter().any(|other| {
        other.is_actionable()
            && other.numeral_phrase.components.len() > candidate.numeral_phrase.components.len()
            && other.numeral_phrase.start_token <= candidate.numeral_phrase.start_token
            && other.numeral_phrase.end_token >= candidate.numeral_phrase.end_token
            && other.group.end_token == candidate.group.end_token
    })
}
