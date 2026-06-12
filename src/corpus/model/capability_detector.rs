impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Tokenization => "tokenization",
            Self::SentenceBoundaries => "sentence_boundaries",
            Self::Regex => "regex",
            Self::Lexicon => "lexicon",
            Self::Morphology => "morphology",
            Self::Syntax => "syntax",
            Self::Semantics => "semantics",
            Self::NamedEntities => "named_entities",
            Self::WordFormation => "word_formation",
            Self::Stress => "stress",
            Self::Benchmark => "benchmark",
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Detector {
    Regex {
        pattern: String,
        message: String,
        #[serde(default)]
        replacement: Option<String>,
    },
    RepeatedWord {
        message: String,
    },
    MultipleSpaces {
        message: String,
        replacement: String,
    },
    NoWhitespaceBeforePunctuation {
        punctuation: String,
        message: String,
    },
    MissingWhitespaceAfterPunctuation {
        message: String,
    },
    MissingSentenceTerminal {
        message: String,
        replacement: String,
    },
    SentenceInitialLowercase {
        message: String,
    },
    ParticleHyphenMissing {
        bases: Vec<String>,
        particles: Vec<String>,
        message: String,
    },
    CliticHyphenMissing {
        group: String,
        message: String,
    },
    SeparateParticleHyphenated {
        particles: Vec<String>,
        message: String,
    },
    MissingCommaBeforeSubordinator {
        markers: Vec<String>,
        message: String,
    },
    IntroductoryPhraseComma {
        phrases: Vec<String>,
        message: String,
    },
    CoordinationCommaBasic {
        message: String,
    },
    PhraseologicalCoordinationComma {
        message: String,
    },
    UnbalancedQuotes {
        message: String,
    },
    UnpairedDelimiters {
        #[serde(default)]
        pairs: Vec<String>,
        message: String,
    },
    MixedAlphabetWord {
        message: String,
    },
    ConfusableTokens {
        forms: Vec<String>,
        message: String,
    },
    PhraseMap {
        forms: Vec<String>,
        message: String,
    },
    ZhiShiChaShcha {
        #[serde(default)]
        exceptions: Vec<String>,
        message: String,
    },
    TsyaHeuristic {
        infinitive_triggers: Vec<String>,
        finite_subjects: Vec<String>,
        message: String,
    },
    NegatedVerbSpacingBasic {
        message: String,
    },
    AdjNounAgreementDemo {
        message: String,
    },
    NominalGroupModifierAgreementBasic {
        message: String,
    },
    SubjectPredicateAgreementBasic {
        message: String,
    },
    PrepositionGovernmentBasic {
        message: String,
    },
    PrepositionNominalGroupGovernmentBasic {
        message: String,
    },
    VerbGovernmentBasic {
        message: String,
    },
    NumeralNounAgreementBasic {
        message: String,
    },
    NumeralNominalGroupAgreementBasic {
        message: String,
    },
    CompoundNumeralNominalGroupAgreementBasic {
        message: String,
    },
    TypedCompoundNumeralNominalGroupAgreementBasic {
        message: String,
    },
    SpaceAfterOpeningPunctuation {
        message: String,
    },
    SpaceBeforeClosingQuote {
        message: String,
    },
    HyphenInsteadOfDash {
        message: String,
    },
    DashSpacing {
        message: String,
    },
    MultiplePunctuation {
        message: String,
    },
    NumberUnitSpacing {
        units: Vec<String>,
        message: String,
    },
    PolCompoundHyphenMissing {
        message: String,
    },
    PrefixFinalZSAssimilation {
        message: String,
    },
    DocumentAbbreviationExpansion {
        message: String,
    },
    DocumentStyleConsistency {
        message: String,
    },
    Manual {
        note: String,
    },
}

impl Detector {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Regex { .. } => "regex",
            Self::RepeatedWord { .. } => "repeated_word",
            Self::MultipleSpaces { .. } => "multiple_spaces",
            Self::NoWhitespaceBeforePunctuation { .. } => "no_whitespace_before_punctuation",
            Self::MissingWhitespaceAfterPunctuation { .. } => "missing_whitespace_after_punctuation",
            Self::MissingSentenceTerminal { .. } => "missing_sentence_terminal",
            Self::SentenceInitialLowercase { .. } => "sentence_initial_lowercase",
            Self::ParticleHyphenMissing { .. } => "particle_hyphen_missing",
            Self::CliticHyphenMissing { .. } => "clitic_hyphen_missing",
            Self::SeparateParticleHyphenated { .. } => "separate_particle_hyphenated",
            Self::MissingCommaBeforeSubordinator { .. } => "missing_comma_before_subordinator",
            Self::IntroductoryPhraseComma { .. } => "introductory_phrase_comma",
            Self::CoordinationCommaBasic { .. } => "coordination_comma_basic",
            Self::PhraseologicalCoordinationComma { .. } => "phraseological_coordination_comma",
            Self::UnbalancedQuotes { .. } => "unbalanced_quotes",
            Self::UnpairedDelimiters { .. } => "unpaired_delimiters",
            Self::MixedAlphabetWord { .. } => "mixed_alphabet_word",
            Self::ConfusableTokens { .. } => "confusable_tokens",
            Self::PhraseMap { .. } => "phrase_map",
            Self::ZhiShiChaShcha { .. } => "zhi_shi_cha_shcha",
            Self::TsyaHeuristic { .. } => "tsya_heuristic",
            Self::NegatedVerbSpacingBasic { .. } => "negated_verb_spacing_basic",
            Self::AdjNounAgreementDemo { .. } => "adj_noun_agreement_demo",
            Self::NominalGroupModifierAgreementBasic { .. } => "nominal_group_modifier_agreement_basic",
            Self::SubjectPredicateAgreementBasic { .. } => "subject_predicate_agreement_basic",
            Self::PrepositionGovernmentBasic { .. } => "preposition_government_basic",
            Self::PrepositionNominalGroupGovernmentBasic { .. } => "preposition_nominal_group_government_basic",
            Self::VerbGovernmentBasic { .. } => "verb_government_basic",
            Self::NumeralNounAgreementBasic { .. } => "numeral_noun_agreement_basic",
            Self::NumeralNominalGroupAgreementBasic { .. } => "numeral_nominal_group_agreement_basic",
            Self::CompoundNumeralNominalGroupAgreementBasic { .. } => {
                "compound_numeral_nominal_group_agreement_basic"
            }
            Self::TypedCompoundNumeralNominalGroupAgreementBasic { .. } => {
                "typed_compound_numeral_nominal_group_agreement_basic"
            }
            Self::SpaceAfterOpeningPunctuation { .. } => "space_after_opening_punctuation",
            Self::SpaceBeforeClosingQuote { .. } => "space_before_closing_quote",
            Self::HyphenInsteadOfDash { .. } => "hyphen_instead_of_dash",
            Self::DashSpacing { .. } => "dash_spacing",
            Self::MultiplePunctuation { .. } => "multiple_punctuation",
            Self::NumberUnitSpacing { .. } => "number_unit_spacing",
            Self::PolCompoundHyphenMissing { .. } => "pol_compound_hyphen_missing",
            Self::PrefixFinalZSAssimilation { .. } => "prefix_final_z_s_assimilation",
            Self::DocumentAbbreviationExpansion { .. } => "document_abbreviation_expansion",
            Self::DocumentStyleConsistency { .. } => "document_style_consistency",
            Self::Manual { .. } => "manual",
        }
    }
}
