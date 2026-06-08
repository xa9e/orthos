#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum GovernmentFrameKind {
    Preposition,
    Numeral,
    Verb,
    Unknown,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum GovernmentFrameSource {
    MorphosyntaxRelation,
    VerbValencySeed,
    VerbPrepositionalValencySeed,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GovernmentFrameModelRef {
    pub lemma: String,
    pub complement_kind: crate::morph::VerbGovernmentComplementKind,
    pub preposition: Option<String>,
    pub source_id: Option<crate::morph::SourceId>,
    pub note: Option<String>,
}

impl GovernmentFrameModelRef {
    pub fn from_verb_government(entry: &crate::morph::VerbGovernment) -> Self {
        Self {
            lemma: entry.lemma.clone(),
            complement_kind: entry.complement_kind,
            preposition: entry.preposition.clone(),
            source_id: entry.source_id.clone(),
            note: entry.note.clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GovernmentFrame<'a> {
    pub kind: GovernmentFrameKind,
    pub source: GovernmentFrameSource,
    pub governor: MorphosyntacticTerm<'a>,
    pub dependent: MorphosyntacticTerm<'a>,
    pub span: Span,
    pub expected_cases: Vec<crate::morph::Case>,
    pub observed_cases: Vec<crate::morph::Case>,
    pub compatibility: crate::morph::MorphCompatibility,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SuppressionReason>,
    pub model_ref: Option<GovernmentFrameModelRef>,
}

impl<'a> GovernmentFrame<'a> {
    pub fn is_conflict(&self) -> bool {
        self.confidence.is_actionable()
            && self.blockers.is_empty()
            && self.compatibility == crate::morph::MorphCompatibility::Incompatible
    }
}

pub fn government_frames_from_facts<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    morphosyntax: &MorphosyntaxDocument<'a>,
    islands: &SyntacticIslandMap,
    clause_boundaries: &ClauseBoundaryMap,
) -> Vec<GovernmentFrame<'a>> {
    let mut frames = morphosyntax
        .relations()
        .iter()
        .filter_map(|relation| government_frame_from_relation(relation, islands, clause_boundaries))
        .collect::<Vec<_>>();
    frames.extend(verb_government_frames_from_tokens(
        tokens,
        ambiguity,
        islands,
        clause_boundaries,
    ));
    dedup_government_frames(&mut frames);
    frames
}

pub fn government_frames_from_morphosyntax<'a>(
    document: &MorphosyntaxDocument<'a>,
) -> Vec<GovernmentFrame<'a>> {
    document
        .relations()
        .iter()
        .filter_map(government_frame_from_relation_without_islands)
        .collect()
}

fn government_frame_from_relation<'a>(
    relation: &MorphosyntacticRelation<'a>,
    islands: &SyntacticIslandMap,
    clause_boundaries: &ClauseBoundaryMap,
) -> Option<GovernmentFrame<'a>> {
    let mut frame = government_frame_from_relation_without_islands(relation)?;
    frame.blockers.extend(government_boundary_blockers_for_link(
        relation.governor.token_index,
        relation.dependent.token_index,
        islands,
        clause_boundaries,
    ));
    finalize_government_frame_safety(&mut frame);
    Some(frame)
}

fn government_frame_from_relation_without_islands<'a>(
    relation: &MorphosyntacticRelation<'a>,
) -> Option<GovernmentFrame<'a>> {
    let kind = match relation.kind {
        MorphosyntacticRelationKind::PrepositionCaseGovernment => GovernmentFrameKind::Preposition,
        MorphosyntacticRelationKind::NumeralCaseNumberGovernment => GovernmentFrameKind::Numeral,
        MorphosyntacticRelationKind::AttributiveAgreement
        | MorphosyntacticRelationKind::SubjectPredicateAgreement => return None,
    };
    Some(GovernmentFrame {
        kind,
        source: GovernmentFrameSource::MorphosyntaxRelation,
        governor: relation.governor.clone(),
        dependent: relation.dependent.clone(),
        span: relation.span,
        expected_cases: expected_cases_for_constraint(&relation.constraint),
        observed_cases: observed_cases_for_constraint(&relation.constraint),
        compatibility: relation.constraint.compatibility(),
        confidence: relation.confidence,
        blockers: relation.blockers.iter().copied().map(SuppressionReason::from).collect(),
        model_ref: None,
    })
}

fn expected_cases_for_constraint(
    constraint: &MorphosyntacticConstraint,
) -> Vec<crate::morph::Case> {
    let mut cases = match constraint {
        MorphosyntacticConstraint::Government(check) => check
            .conflict
            .as_ref()
            .map(|conflict| conflict.expected.cases.iter().copied().collect())
            .unwrap_or_default(),
        MorphosyntacticConstraint::Quantity(check) => check
            .conflict
            .as_ref()
            .map(|conflict| expected_cases_for_numeral_class(conflict.numeral_class))
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    cases.sort_unstable();
    cases.dedup();
    cases
}

fn observed_cases_for_constraint(
    constraint: &MorphosyntacticConstraint,
) -> Vec<crate::morph::Case> {
    let mut cases = match constraint {
        MorphosyntacticConstraint::Government(check) => check
            .conflict
            .as_ref()
            .map(|conflict| conflict.observed_cases.iter().copied().collect())
            .unwrap_or_default(),
        MorphosyntacticConstraint::Quantity(check) => check
            .conflict
            .as_ref()
            .map(|conflict| conflict.observed_cases.iter().copied().collect())
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    cases.sort_unstable();
    cases.dedup();
    cases
}

fn expected_cases_for_numeral_class(
    class: crate::morph::NumeralGovernmentClass,
) -> Vec<crate::morph::Case> {
    match class {
        crate::morph::NumeralGovernmentClass::One => vec![crate::morph::Case::Nominative],
        crate::morph::NumeralGovernmentClass::Paucal
        | crate::morph::NumeralGovernmentClass::Many
        | crate::morph::NumeralGovernmentClass::Collective => vec![crate::morph::Case::Genitive],
        crate::morph::NumeralGovernmentClass::Ordinal
        | crate::morph::NumeralGovernmentClass::Unknown => Vec::new(),
    }
}

fn government_boundary_blockers_for_link(
    governor_token: usize,
    dependent_token: usize,
    islands: &SyntacticIslandMap,
    clause_boundaries: &ClauseBoundaryMap,
) -> Vec<SuppressionReason> {
    let mut blockers = clause_boundaries.blockers_between_tokens(governor_token, dependent_token);
    if islands.can_link_tokens(governor_token, dependent_token) {
        blockers.sort_unstable();
        blockers.dedup();
        return blockers;
    }
    let governor_kind = islands.island_for_token(governor_token).map(|island| island.kind);
    let dependent_kind = islands.island_for_token(dependent_token).map(|island| island.kind);
    blockers.extend(match (governor_kind, dependent_kind) {
        (Some(SyntacticIslandKind::DirectSpeech), _) | (_, Some(SyntacticIslandKind::DirectSpeech)) => {
            vec![SuppressionReason::DirectSpeechBoundary]
        }
        (Some(SyntacticIslandKind::Parenthetical), _) | (_, Some(SyntacticIslandKind::Parenthetical)) => {
            vec![SuppressionReason::ParenthesisBoundary]
        }
        _ => vec![SuppressionReason::UnsafeBoundary],
    });
    blockers.sort_unstable();
    blockers.dedup();
    blockers
}

fn finalize_government_frame_safety(frame: &mut GovernmentFrame<'_>) {
    frame.blockers.sort_unstable();
    frame.blockers.dedup();
    if !frame.blockers.is_empty() && frame.confidence.is_actionable() {
        frame.confidence = SyntaxConfidence::Ambiguous;
    }
}

fn dedup_government_frames(frames: &mut Vec<GovernmentFrame<'_>>) {
    let mut seen = BTreeSet::new();
    frames.retain(|frame| {
        seen.insert((
            frame.kind,
            frame.source,
            frame.governor.token.span.start,
            frame.governor.token.span.end,
            frame.dependent.token.span.start,
            frame.dependent.token.span.end,
            frame.span.start,
            frame.span.end,
            frame.expected_cases.clone(),
        ))
    });
}
