#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    NormativeReference,
    CorpusAnnotationStandard,
    Dictionary,
    DonorProject,
    DatasetTaxonomy,
    OnlineReference,
    EditorialPractice,
    ProjectInternal,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceAuthority {
    Normative,
    Descriptive,
    CorpusDerived,
    ProjectInternal,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleFamily {
    Orthography,
    Punctuation,
    Grammar,
    MorphologyDependent,
    SyntaxDependent,
    WordFormation,
    StressDependent,
    LexicalStyle,
    Typography,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FalsePositiveRisk {
    Low,
    Medium,
    High,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    Surface,
    Regex,
    TokenSequence,
    Morphological,
    Syntactic,
    Dependency,
    PunctuationContext,
    WordFormation,
    Stress,
    LexicalSet,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LinguisticConcept {
    Agreement,
    Government,
    Coordination,
    ClauseBoundary,
    ParentheticalExpression,
    DirectSpeech,
    Derivation,
    SpellingPattern,
    StressRequirement,
    IdiomFixedExpressionException,
    Morphology,
    Syntax,
    Lexical,
    StyleRegister,
    NamedEntity,
    TokenContext,
    SentenceBoundary,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    NormativeSource,
    CorpusAttestation,
    DonorTaxonomy,
    Benchmark,
    Lexicon,
    Morphology,
    Syntax,
    StressDictionary,
    ExpertReview,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Orthography,
    Punctuation,
    Grammar,
    Style,
    Typography,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    Basic,
    School,
    Intermediate,
    Advanced,
    Expert,
    Heuristic,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleStatus {
    Implemented,
    Planned,
    Research,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Tokenization,
    SentenceBoundaries,
    Regex,
    Lexicon,
    Morphology,
    Syntax,
    Semantics,
    NamedEntities,
    WordFormation,
    Stress,
    Benchmark,
}
