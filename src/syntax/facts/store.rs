#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LinguisticFactStoreSummary {
    pub tokens: usize,
    pub ambiguous_tokens: usize,
    pub islands: usize,
    pub nominal_groups: usize,
    #[serde(default)]
    pub clause_boundaries: usize,
    pub clauses: usize,
    pub agreement_edges: usize,
    pub coordination_groups: usize,
    pub punctuation_slots: usize,
    pub government_frames: usize,
    pub constructions: usize,
    pub document_terms: usize,
    pub document_abbreviations: usize,
    pub glossary_entries: usize,
    pub diagnostic_ledger_entries: usize,
    pub suppressed_diagnostics: usize,
}

pub struct LinguisticFactStore<'a> {
    tokens: Vec<Token<'a>>,
    ambiguity: AmbiguityModel<'a>,
    islands: SyntacticIslandMap,
    clause_boundaries: ClauseBoundaryMap,
    nominal_groups: Vec<NominalGroupCandidate<'a>>,
    clauses: Vec<ClauseCandidate<'a>>,
    morphosyntax: MorphosyntaxDocument<'a>,
    agreement_graph: AgreementGraph<'a>,
    coordination_groups: Vec<CoordinationGroup<'a>>,
    punctuation_slots: Vec<PunctuationSlot<'a>>,
    government_frames: Vec<GovernmentFrame<'a>>,
    constructions: Vec<ConstructionPatternMatch<'a>>,
    document_context: DocumentContext<'a>,
    diagnostic_ledger: DiagnosticLedger,
}

impl<'a> LinguisticFactStore<'a> {
    pub fn from_tokens(
        text: &'a str,
        tokens: &[Token<'a>],
        morph: &dyn crate::morph::MorphAnalyzer,
    ) -> Self {
        Self::from_tokens_with_analyses(text, tokens, |token_index| {
            morph.analyze(tokens[token_index].text)
        })
    }

    pub fn from_tokens_with_analyses(
        text: &'a str,
        tokens: &[Token<'a>],
        analyses_for_token: impl Fn(usize) -> Vec<crate::morph::MorphAnalysis>,
    ) -> Self {
        let ambiguity = AmbiguityModel::from_tokens_with_analyses(tokens, &analyses_for_token);
        let islands = SyntacticIslandMap::from_text_tokens(text, tokens);
        let clause_boundaries = ClauseBoundaryMap::from_text_tokens(text, tokens);
        let nominal_groups = short_nominal_group_candidates(tokens, 3);
        let clauses = clause_candidates_from_islands(tokens, &ambiguity, &islands);
        let morphosyntax = MorphosyntaxDocument::from_tokens_with_analyses(tokens, &analyses_for_token);
        let agreement_graph = AgreementGraph::from_facts(
            &morphosyntax,
            &clauses,
            &nominal_groups,
            &ambiguity,
            &islands,
        );
        let coordination_groups = coordination_groups_from_facts(tokens, &ambiguity, &islands);
        let punctuation_slots = punctuation_slots_from_facts(tokens, &ambiguity, &islands, &clauses, &coordination_groups);
        let government_frames = government_frames_from_facts(
            tokens,
            &ambiguity,
            &morphosyntax,
            &islands,
            &clause_boundaries,
        );
        let document_context = DocumentContext::new(text, tokens);
        let constructions = construction_matches_from_facts(
            &morphosyntax,
            &clauses,
            &nominal_groups,
            &islands,
        );
        let diagnostic_ledger = DiagnosticLedger::from_facts(
            &agreement_graph,
            &government_frames,
            &punctuation_slots,
            &document_context,
        );

        Self {
            tokens: tokens.to_vec(),
            ambiguity,
            islands,
            clause_boundaries,
            nominal_groups,
            clauses,
            morphosyntax,
            agreement_graph,
            coordination_groups,
            punctuation_slots,
            government_frames,
            constructions,
            document_context,
            diagnostic_ledger,
        }
    }

    pub fn new(text: &'a str, morph: &dyn crate::morph::MorphAnalyzer) -> Self {
        let tokens = tokenize(text);
        Self::from_tokens(text, &tokens, morph)
    }

    pub fn tokens(&self) -> &[Token<'a>] {
        &self.tokens
    }

    pub fn ambiguity(&self) -> &AmbiguityModel<'a> {
        &self.ambiguity
    }

    pub fn islands(&self) -> &SyntacticIslandMap {
        &self.islands
    }

    pub fn clause_boundaries(&self) -> &ClauseBoundaryMap {
        &self.clause_boundaries
    }

    pub fn nominal_groups(&self) -> &[NominalGroupCandidate<'a>] {
        &self.nominal_groups
    }

    pub fn clauses(&self) -> &[ClauseCandidate<'a>] {
        &self.clauses
    }

    pub fn morphosyntax(&self) -> &MorphosyntaxDocument<'a> {
        &self.morphosyntax
    }

    pub fn agreement_graph(&self) -> &AgreementGraph<'a> {
        &self.agreement_graph
    }

    pub fn coordination_groups(&self) -> &[CoordinationGroup<'a>] {
        &self.coordination_groups
    }

    pub fn punctuation_slots(&self) -> &[PunctuationSlot<'a>] {
        &self.punctuation_slots
    }

    pub fn government_frames(&self) -> &[GovernmentFrame<'a>] {
        &self.government_frames
    }

    pub fn constructions(&self) -> &[ConstructionPatternMatch<'a>] {
        &self.constructions
    }

    pub fn document_context(&self) -> &DocumentContext<'a> {
        &self.document_context
    }

    pub fn diagnostic_ledger(&self) -> &DiagnosticLedger {
        &self.diagnostic_ledger
    }

    pub fn summary(&self) -> LinguisticFactStoreSummary {
        LinguisticFactStoreSummary {
            tokens: self.tokens.len(),
            ambiguous_tokens: self
                .ambiguity
                .token_ambiguities()
                .iter()
                .filter(|item| item.class.is_ambiguous())
                .count(),
            islands: self.islands.islands().len(),
            clause_boundaries: self.clause_boundaries.boundaries().len(),
            nominal_groups: self.nominal_groups.len(),
            clauses: self.clauses.len(),
            agreement_edges: self.agreement_graph.edges().len(),
            coordination_groups: self.coordination_groups.len(),
            punctuation_slots: self.punctuation_slots.len(),
            government_frames: self.government_frames.len(),
            constructions: self.constructions.len(),
            document_terms: self.document_context.terms.len(),
            document_abbreviations: self.document_context.abbreviations.len(),
            glossary_entries: self.document_context.glossary_entries.len(),
            diagnostic_ledger_entries: self.diagnostic_ledger.entries().len(),
            suppressed_diagnostics: self.diagnostic_ledger.suppressed().count(),
        }
    }
}
