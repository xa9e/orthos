#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ConstructionPatternKind {
    PrepositionNominalGroup,
    NumeralNominalGroup,
    ModifierHead,
    SubjectPredicate,
    Clause,
    Parenthetical,
    DirectSpeech,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConstructionPattern {
    pub kind: ConstructionPatternKind,
    pub required_capabilities: Vec<crate::corpus::Capability>,
    pub description: &'static str,
}

impl ConstructionPattern {
    pub fn seed(kind: ConstructionPatternKind) -> Self {
        let (required_capabilities, description) = match kind {
            ConstructionPatternKind::PrepositionNominalGroup => (
                vec![
                    crate::corpus::Capability::Tokenization,
                    crate::corpus::Capability::Morphology,
                    crate::corpus::Capability::Syntax,
                ],
                "preposition plus governed nominal group",
            ),
            ConstructionPatternKind::NumeralNominalGroup => (
                vec![
                    crate::corpus::Capability::Tokenization,
                    crate::corpus::Capability::Morphology,
                    crate::corpus::Capability::Syntax,
                ],
                "cardinal numeral plus counted nominal group",
            ),
            ConstructionPatternKind::ModifierHead => (
                vec![crate::corpus::Capability::Morphology, crate::corpus::Capability::Syntax],
                "modifier plus nominal head",
            ),
            ConstructionPatternKind::SubjectPredicate => (
                vec![crate::corpus::Capability::Morphology, crate::corpus::Capability::Syntax],
                "subject candidate plus predicate",
            ),
            ConstructionPatternKind::Clause => (
                vec![crate::corpus::Capability::SentenceBoundaries, crate::corpus::Capability::Syntax],
                "minimal safe clause candidate",
            ),
            ConstructionPatternKind::Parenthetical => (
                vec![crate::corpus::Capability::Tokenization, crate::corpus::Capability::Syntax],
                "parenthetical syntactic island",
            ),
            ConstructionPatternKind::DirectSpeech => (
                vec![crate::corpus::Capability::Tokenization, crate::corpus::Capability::Syntax],
                "direct-speech syntactic island",
            ),
        };
        Self {
            kind,
            required_capabilities,
            description,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConstructionPatternMatch<'a> {
    pub kind: ConstructionPatternKind,
    pub span: Span,
    pub head_token: Option<Token<'a>>,
    pub dependent_token: Option<Token<'a>>,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SuppressionReason>,
}

impl<'a> ConstructionPatternMatch<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty()
    }
}

pub fn construction_matches_from_facts<'a>(
    morphosyntax: &MorphosyntaxDocument<'a>,
    clauses: &[ClauseCandidate<'a>],
    nominal_groups: &[NominalGroupCandidate<'a>],
    islands: &SyntacticIslandMap,
) -> Vec<ConstructionPatternMatch<'a>> {
    let mut out = morphosyntax
        .relations()
        .iter()
        .filter_map(construction_match_from_relation)
        .collect::<Vec<_>>();

    out.extend(clauses.iter().map(construction_match_from_clause));
    out.extend(nominal_groups.iter().map(construction_match_from_nominal_group));
    out.extend(islands.islands().iter().filter_map(construction_match_from_island));
    out.sort_by_key(|item| (item.span.start, item.span.end, item.kind));
    out
}

fn construction_match_from_relation<'a>(
    relation: &MorphosyntacticRelation<'a>,
) -> Option<ConstructionPatternMatch<'a>> {
    let kind = match relation.kind {
        MorphosyntacticRelationKind::AttributiveAgreement => ConstructionPatternKind::ModifierHead,
        MorphosyntacticRelationKind::SubjectPredicateAgreement => ConstructionPatternKind::SubjectPredicate,
        MorphosyntacticRelationKind::PrepositionCaseGovernment => {
            ConstructionPatternKind::PrepositionNominalGroup
        }
        MorphosyntacticRelationKind::NumeralCaseNumberGovernment => ConstructionPatternKind::NumeralNominalGroup,
    };
    Some(ConstructionPatternMatch {
        kind,
        span: relation.span,
        head_token: Some(relation.governor.token.clone()),
        dependent_token: Some(relation.dependent.token.clone()),
        confidence: relation.confidence,
        blockers: relation.blockers.iter().copied().map(SuppressionReason::from).collect(),
    })
}

fn construction_match_from_clause<'a>(clause: &ClauseCandidate<'a>) -> ConstructionPatternMatch<'a> {
    ConstructionPatternMatch {
        kind: ConstructionPatternKind::Clause,
        span: clause.span,
        head_token: clause.predicate.as_ref().map(|term| term.token.clone()),
        dependent_token: clause.subject_candidate.as_ref().map(|term| term.token.clone()),
        confidence: clause.confidence,
        blockers: clause.blockers.clone(),
    }
}

fn construction_match_from_nominal_group<'a>(group: &NominalGroupCandidate<'a>) -> ConstructionPatternMatch<'a> {
    ConstructionPatternMatch {
        kind: ConstructionPatternKind::ModifierHead,
        span: group.span,
        head_token: Some(group.head.clone()),
        dependent_token: group.modifiers.first().cloned(),
        confidence: group.confidence,
        blockers: group.blockers.iter().copied().map(SuppressionReason::from).collect(),
    }
}

fn construction_match_from_island<'a>(island: &SyntacticIsland) -> Option<ConstructionPatternMatch<'a>> {
    let kind = match island.kind {
        SyntacticIslandKind::Parenthetical => ConstructionPatternKind::Parenthetical,
        SyntacticIslandKind::DirectSpeech => ConstructionPatternKind::DirectSpeech,
        SyntacticIslandKind::PlainSentence => return None,
    };
    Some(ConstructionPatternMatch {
        kind,
        span: island.span,
        head_token: None,
        dependent_token: None,
        confidence: island.confidence,
        blockers: island.blockers.clone(),
    })
}
