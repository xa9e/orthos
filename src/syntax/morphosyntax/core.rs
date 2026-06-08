#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MorphosyntacticRelationKind {
    AttributiveAgreement,
    SubjectPredicateAgreement,
    PrepositionCaseGovernment,
    NumeralCaseNumberGovernment,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MorphosyntacticRole {
    Modifier,
    Head,
    Subject,
    Predicate,
    CaseGovernor,
    GovernedNominal,
    Quantifier,
    QuantifiedHead,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MorphosyntacticTerm<'a> {
    pub token_index: usize,
    pub token: Token<'a>,
    pub role: MorphosyntacticRole,
    pub analyses: Vec<crate::morph::MorphAnalysis>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MorphosyntacticConstraint {
    Agreement(crate::morph::AgreementCheck),
    Government(crate::morph::GovernmentCheck),
    Quantity(crate::morph::QuantityAgreementCheck),
    Unknown { reason: String },
}

impl MorphosyntacticConstraint {
    pub fn compatibility(&self) -> crate::morph::MorphCompatibility {
        match self {
            Self::Agreement(check) => check.compatibility,
            Self::Government(check) => check.compatibility,
            Self::Quantity(check) => check.compatibility,
            Self::Unknown { .. } => crate::morph::MorphCompatibility::Unknown,
        }
    }

    pub fn is_confident_rejection(&self) -> bool {
        match self {
            Self::Agreement(check) => check.is_confident_rejection(),
            Self::Government(check) => check.is_confident_rejection(),
            Self::Quantity(check) => check.is_confident_rejection(),
            Self::Unknown { .. } => false,
        }
    }

    pub fn explanation_label(&self) -> &'static str {
        match self {
            Self::Agreement(check) if check.is_confident_rejection() => "agreement-conflict",
            Self::Agreement(_) => "agreement",
            Self::Government(check) if check.is_confident_rejection() => "government-conflict",
            Self::Government(_) => "government",
            Self::Quantity(check) if check.is_confident_rejection() => "quantity-conflict",
            Self::Quantity(_) => "quantity",
            Self::Unknown { .. } => "unknown",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MorphosyntacticRelation<'a> {
    pub kind: MorphosyntacticRelationKind,
    pub governor: MorphosyntacticTerm<'a>,
    pub dependent: MorphosyntacticTerm<'a>,
    pub span: Span,
    pub confidence: SyntaxConfidence,
    pub blockers: Vec<SyntaxRelationBlocker>,
    pub constraint: MorphosyntacticConstraint,
}

impl<'a> MorphosyntacticRelation<'a> {
    pub fn is_actionable(&self) -> bool {
        self.confidence.is_actionable() && self.blockers.is_empty()
    }

    pub fn is_confident_rejection(&self) -> bool {
        self.is_actionable() && self.constraint.is_confident_rejection()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MorphosyntaxDocument<'a> {
    tokens: Vec<Token<'a>>,
    relations: Vec<MorphosyntacticRelation<'a>>,
}

impl<'a> MorphosyntaxDocument<'a> {
    pub fn new(text: &'a str, morph: &dyn crate::morph::MorphAnalyzer) -> Self {
        let tokens = tokenize(text);
        Self::from_tokens(&tokens, morph)
    }

    pub fn from_tokens(tokens: &[Token<'a>], morph: &dyn crate::morph::MorphAnalyzer) -> Self {
        let preposition_registry = crate::morph::PrepositionGovernmentRegistry::russian_seed();
        Self::from_tokens_with_preposition_registry(tokens, morph, &preposition_registry)
    }

    pub fn from_tokens_with_analyses(
        tokens: &[Token<'a>],
        analyses_for_token: impl Fn(usize) -> Vec<crate::morph::MorphAnalysis>,
    ) -> Self {
        let preposition_registry = crate::morph::PrepositionGovernmentRegistry::russian_seed();
        Self::from_tokens_with_preposition_registry_and_analyses(
            tokens,
            &preposition_registry,
            analyses_for_token,
        )
    }

    pub fn from_tokens_with_preposition_registry(
        tokens: &[Token<'a>],
        morph: &dyn crate::morph::MorphAnalyzer,
        preposition_registry: &crate::morph::PrepositionGovernmentRegistry,
    ) -> Self {
        Self::from_tokens_with_preposition_registry_and_analyses(
            tokens,
            preposition_registry,
            |token_index| morph.analyze(tokens[token_index].text),
        )
    }

    pub fn from_tokens_with_preposition_registry_and_analyses(
        tokens: &[Token<'a>],
        preposition_registry: &crate::morph::PrepositionGovernmentRegistry,
        analyses_for_token: impl Fn(usize) -> Vec<crate::morph::MorphAnalysis>,
    ) -> Self {
        let mut relations = Vec::new();
        relations.extend(attributive_relations(tokens, &analyses_for_token));
        relations.extend(subject_predicate_relations(tokens, &analyses_for_token));
        relations.extend(preposition_government_relations(
            tokens,
            &analyses_for_token,
            preposition_registry,
        ));
        relations.extend(numeral_government_relations(tokens, &analyses_for_token));
        dedup_relations(&mut relations);
        Self {
            tokens: tokens.to_vec(),
            relations,
        }
    }

    pub fn tokens(&self) -> &[Token<'a>] {
        &self.tokens
    }

    pub fn relations(&self) -> &[MorphosyntacticRelation<'a>] {
        &self.relations
    }

    pub fn conflicts(&self) -> impl Iterator<Item = &MorphosyntacticRelation<'a>> {
        self.relations.iter().filter(|relation| relation.is_confident_rejection())
    }

    pub fn relations_by_kind(
        &self,
        kind: MorphosyntacticRelationKind,
    ) -> impl Iterator<Item = &MorphosyntacticRelation<'a>> {
        self.relations.iter().filter(move |relation| relation.kind == kind)
    }
}
