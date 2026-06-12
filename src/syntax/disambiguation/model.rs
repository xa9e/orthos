// Contextual morphological disambiguation: typed trace model.
//
// The constraint engine removes readings that are impossible in their local
// context. Every removal is recorded here so the debug layer can prove why a
// reading disappeared, and so a regression can pin the exact elimination.
// Invariant: a token never loses its last reading.

/// Which constraint eliminated a reading.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisambiguationConstraint {
    /// A reliable preposition restricts the case of the next nominal token.
    PrepositionCaseGovernment,
    /// A finite verb or gerund reading cannot directly follow a preposition.
    PrepositionVerbExclusion,
    /// A reading is kept only if it agrees with at least one reading of a
    /// reliable adjacent modifier/head partner.
    ModifierHeadAgreement,
}

impl DisambiguationConstraint {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PrepositionCaseGovernment => "preposition_case_government",
            Self::PrepositionVerbExclusion => "preposition_verb_exclusion",
            Self::ModifierHeadAgreement => "modifier_head_agreement",
        }
    }
}

/// Machine-readable proof for one eliminated reading.
#[derive(Debug, Clone, Eq, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReadingElimination {
    pub token_index: usize,
    pub form: String,
    pub eliminated_lemma: String,
    pub eliminated_pos: crate::morph::PartOfSpeech,
    /// Compact `key=value|...` rendering of the eliminated reading's features.
    pub eliminated_features: String,
    pub constraint: DisambiguationConstraint,
    /// Token that licensed the elimination (e.g. the governing preposition).
    pub evidence_token_index: usize,
    pub evidence_form: String,
    pub explanation: String,
}

/// Full trace of a disambiguation run over one document.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DisambiguationTrace {
    pub eliminations: Vec<ReadingElimination>,
    /// Fixpoint passes executed (constraints can enable each other).
    pub passes: usize,
}

impl DisambiguationTrace {
    pub fn is_empty(&self) -> bool {
        self.eliminations.is_empty()
    }

    pub fn eliminations_for_token(&self, token_index: usize) -> Vec<&ReadingElimination> {
        self.eliminations
            .iter()
            .filter(|item| item.token_index == token_index)
            .collect()
    }
}
