#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ClauseTermRole {
    SubjectCandidate,
    Predicate,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClauseTerm<'a> {
    pub token_index: usize,
    pub token: Token<'a>,
    pub role: ClauseTermRole,
    pub analyses: Vec<crate::morph::MorphAnalysis>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClauseCandidate<'a> {
    pub id: usize,
    pub span: Span,
    pub start_token: Option<usize>,
    pub end_token: Option<usize>,
    pub island_id: Option<usize>,
    pub predicate: Option<ClauseTerm<'a>>,
    pub subject_candidate: Option<ClauseTerm<'a>>,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SuppressionReason>,
}

impl<'a> ClauseCandidate<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty()
    }
}

pub fn clause_candidates_from_islands<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    islands: &SyntacticIslandMap,
) -> Vec<ClauseCandidate<'a>> {
    let mut clauses = Vec::new();
    for island in islands.islands() {
        clauses.push(clause_candidate_for_island(clauses.len(), tokens, ambiguity, island));
    }
    clauses
}

fn clause_candidate_for_island<'a>(
    id: usize,
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    island: &SyntacticIsland,
) -> ClauseCandidate<'a> {
    let predicate = find_predicate(tokens, ambiguity, island);
    let subject_candidate = predicate
        .as_ref()
        .and_then(|predicate| find_subject_before_predicate(tokens, ambiguity, island, predicate.token_index));

    let mut blockers = island.blockers.clone();
    if predicate.is_none() {
        blockers.push(SuppressionReason::ConflictNotProven);
    }
    if subject_candidate.is_none() {
        blockers.push(SuppressionReason::InsufficientMorphology);
    }
    if predicate
        .as_ref()
        .is_some_and(|term| !ambiguity_for_term_is_safe(ambiguity, term.token_index))
    {
        blockers.push(SuppressionReason::AmbiguousMorphology);
    }
    if subject_candidate
        .as_ref()
        .is_some_and(|term| !ambiguity_for_term_is_safe(ambiguity, term.token_index))
    {
        blockers.push(SuppressionReason::AmbiguousMorphology);
    }
    blockers.sort_unstable();
    blockers.dedup();

    ClauseCandidate {
        id,
        span: island.span,
        start_token: island.start_token,
        end_token: island.end_token,
        island_id: Some(island.id),
        predicate,
        subject_candidate,
        confidence: if blockers.is_empty() {
            SyntaxConfidence::Strong
        } else {
            SyntaxConfidence::Weak
        },
        blockers,
    }
}

fn find_predicate<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    island: &SyntacticIsland,
) -> Option<ClauseTerm<'a>> {
    island_token_indices(island)
        .into_iter()
        .filter(|idx| tokens.get(*idx).is_some_and(|token| token.kind == TokenKind::Word))
        .find_map(|idx| {
            let analyses = ambiguity.analyses_for_token(idx);
            analyses.iter().any(is_predicate_analysis).then(|| ClauseTerm {
                token_index: idx,
                token: tokens[idx].clone(),
                role: ClauseTermRole::Predicate,
                analyses: analyses.to_vec(),
            })
        })
}

fn find_subject_before_predicate<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    island: &SyntacticIsland,
    predicate_index: usize,
) -> Option<ClauseTerm<'a>> {
    island_token_indices(island)
        .into_iter()
        .filter(|idx| *idx < predicate_index)
        .rev()
        .find_map(|idx| {
            let token = tokens.get(idx)?;
            if token.kind != TokenKind::Word {
                return None;
            }
            let analyses = ambiguity.analyses_for_token(idx);
            analyses.iter().any(is_subject_analysis).then(|| ClauseTerm {
                token_index: idx,
                token: token.clone(),
                role: ClauseTermRole::SubjectCandidate,
                analyses: analyses.to_vec(),
            })
        })
}

fn island_token_indices(island: &SyntacticIsland) -> Vec<usize> {
    match (island.start_token, island.end_token) {
        (Some(start), Some(end)) if start <= end => (start..=end).collect(),
        _ => Vec::new(),
    }
}

fn is_predicate_analysis(analysis: &crate::morph::MorphAnalysis) -> bool {
    match analysis.pos {
        crate::morph::PartOfSpeech::Verb => {
            !matches!(analysis.features.verb_form, Some(crate::morph::VerbForm::Infinitive))
        }
        crate::morph::PartOfSpeech::Predicative => true,
        crate::morph::PartOfSpeech::Adjective => {
            matches!(analysis.features.adjective_form, Some(crate::morph::AdjectiveForm::Short))
        }
        _ => false,
    }
}

fn is_subject_analysis(analysis: &crate::morph::MorphAnalysis) -> bool {
    matches!(analysis.pos, crate::morph::PartOfSpeech::Noun | crate::morph::PartOfSpeech::Pronoun)
        && !matches!(analysis.features.case, Some(case) if case != crate::morph::Case::Nominative)
}

fn ambiguity_for_term_is_safe(ambiguity: &AmbiguityModel<'_>, token_index: usize) -> bool {
    ambiguity
        .for_token_index(token_index)
        .is_some_and(TokenAmbiguity::is_safe_for_confident_diagnostic)
}
