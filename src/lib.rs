//! Rule-based Russian proofreading core.
//!
//! The library deliberately separates the rule corpus from the engine. Rules are
//! loaded from YAML, validated, then executed by detector implementations.

pub mod analysis;
pub mod corpus;
pub mod debug;
pub mod detector;
pub mod engine;
pub mod issue;
pub mod morph;
pub mod syntax;
pub mod text;

pub use analysis::{AnalysisContext, AnalysisContextSummary, TokenMorphAnalysisCache};
pub use corpus::{
    Capability, Confidence, Corpus, Detector, Domain, EvidenceKind, FalsePositiveRisk, Level,
    LinguisticConcept, PatternKind, Rule, RuleCondition, RuleEvidence, RuleFamily, RulePattern,
    RuleStatus, Severity, SourceAuthority, SourceRef, SourceType,
};
pub use debug::{
    AnalysisDebugSnapshot, DebugLimits, DebugOptions, DebugReport, EngineDebugSnapshot,
    FactStoreDebugSnapshot, GovernmentFrameDebugEntry, GovernmentFrameModelRefDebug,
    LanguageModelDebugSnapshot, RuleExecutionDebug, TokenDebugEntry, TokenMorphDebugEntry,
    VerbGovernmentInventoryDebug,
};
pub use detector::{DetectorMetadata, DetectorRegistry, DetectorRunner, default_detector_registry};
pub use engine::{
    CapabilityRegistry, CheckOptions, CheckResult, Checker, CheckerBuilder, EngineTimings,
    ExecutionPlan, ExecutionPlanSummary, ExecutionStrategy, PlannedRuleSummary, Profile,
    RuleFilter, RuleTiming, SkippedRule, SkippedRuleReason, StatusFilter, SuppressionOptions,
    check_text,
};
pub use issue::{Issue, Position, Span};
pub use morph::{
    AdjectiveForm, AgreementCheck, AgreementConflict, AgreementFeatureKind, AgreementRelationKind,
    AgreementSignature, AmbiguityPolicy, Analysis, AnalyzerCapabilities, Animacy, Aspect, Case,
    CliticHyphenGroup, CliticHyphenSuggestion, CliticPosition, Degree, DerivationConfidence,
    DictionaryFormat, DictionaryImportError, DictionaryImporter, DictionaryMetadata,
    FeatureConstraintSet, Gender, GovernmentCheck, GovernmentConflict, GovernmentRelationKind,
    Grammeme, LemmaId, LicenseStatus, Mood, MorphAnalysis, MorphAnalyzer, MorphCompatibility,
    MorphFeatures, MorphLexicon, MorphemeEntry, MorphemeInventory, MorphemeKind,
    MorphemeProductivity, MorphemeSegment, NegationSpacingSuggestion, Number,
    NumeralGovernmentClass, NumeralNounCompatibility, OpenCorporaCsvDictionaryImporter,
    OpenCorporaXmlDictionaryImporter, ParadigmId, PartOfSpeech, Person,
    PredicateAgreementSignature, PrefixAssimilationSuggestion, PrepositionGovernment,
    PrepositionGovernmentRegistry, ProjectTsvDictionaryImporter, PymorphyExportDictionaryImporter,
    RussianCliticModel, RussianDerivationModel, SourceId, StressAvailability, StressInfo,
    StressRecord, StressTsvImporter, SubjectAgreementSignature, SubjectPredicateRole, Tense,
    UnknownTokenBehavior, VerbForm, VerbGovernment, VerbGovernmentComplementKind,
    VerbGovernmentFalsePositiveFixture, VerbGovernmentFalsePositiveFixtureSet,
    VerbGovernmentFixture, VerbGovernmentFixtureSet, VerbGovernmentKey, VerbGovernmentRegistry,
    VerbGovernmentSeedError, Voice, WordFormationParse, adj_noun_agreement_check,
    animacy_aware_accusative_compatibility, can_agree_as_adj_noun, case_compatibility,
    confidently_reject_adj_noun_agreement, confidently_reject_subject_predicate_agreement,
    gender_compatibility, has_compatible_case_number_gender, number_compatibility,
    numeral_government_class, numeral_government_compatibility, numeral_noun_agreement_check,
    numeral_noun_compatibility, predicate_agreement_signature, prefix_final_z_s_suggestion,
    preposition_government_check, split_negated_verb_candidate, subject_agreement_signature,
    subject_predicate_agreement_check, subject_predicate_compatibility,
};
pub use syntax::{
    AgreementGraph, AgreementGraphEdge, AgreementGraphEdgeKind, AmbiguityClass, AmbiguityModel,
    ClauseBoundary, ClauseBoundaryKind, ClauseBoundaryMap, ClauseCandidate, ClauseTerm,
    ClauseTermRole, ConfidencePolicy, ConstructionPattern, ConstructionPatternKind,
    ConstructionPatternMatch, DiagnosticAssumption, DiagnosticConflict, DiagnosticFact,
    DiagnosticProof, DiagnosticProofKind, GovernmentFrame, GovernmentFrameKind,
    GovernmentFrameModelRef, GovernmentFrameSource, LinguisticFactStore, MorphosyntacticConstraint,
    MorphosyntacticRelation, MorphosyntacticRelationKind, MorphosyntacticRole, MorphosyntacticTerm,
    MorphosyntaxDocument, ProofConfidence, ProofSignalKind, RuleCapabilityContract,
    SuppressedAlternative, SuppressionReason, SyntacticIsland, SyntacticIslandKind,
    SyntacticIslandMap, SyntaxConfidence, TokenAmbiguity,
};
