pub trait MorphAnalyzer: std::fmt::Debug + Send + Sync {
    fn analyze(&self, token: &str) -> Vec<MorphAnalysis>;

    fn analyses_for_lemma(&self, _lemma: &str) -> Vec<MorphAnalysis> {
        Vec::new()
    }

    fn metadata(&self) -> Vec<DictionaryMetadata> {
        Vec::new()
    }

    fn capabilities(&self) -> AnalyzerCapabilities {
        AnalyzerCapabilities::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SourceId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SourceId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct LemmaId(String);

impl LemmaId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for LemmaId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for LemmaId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct ParadigmId(String);

impl ParadigmId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ParadigmId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ParadigmId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum DictionaryFormat {
    ProjectTsv,
    OpenCorporaXml,
    OpenCorporaCsv,
    PymorphyDictionary,
    PymorphyExport,
    AotDictionary,
    StressDictionary,
    StressTsv,
    Other,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum LicenseStatus {
    Unknown,
    Redistributable,
    Restricted,
    LocalGenerationOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct DictionaryMetadata {
    pub source_id: SourceId,
    pub name: String,
    pub version: Option<String>,
    pub format: DictionaryFormat,
    pub license: LicenseStatus,
    pub attribution: Option<String>,
    pub entry_count: Option<usize>,
    pub has_stress: bool,
}

impl DictionaryMetadata {
    pub fn new(source_id: impl Into<SourceId>, name: impl Into<String>, format: DictionaryFormat) -> Self {
        Self {
            source_id: source_id.into(),
            name: name.into(),
            version: None,
            format,
            license: LicenseStatus::Unknown,
            attribution: None,
            entry_count: None,
            has_stress: false,
        }
    }

    pub fn curated_project(source_id: impl Into<SourceId>, name: impl Into<String>) -> Self {
        Self {
            license: LicenseStatus::Redistributable,
            ..Self::new(source_id, name, DictionaryFormat::ProjectTsv)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum UnknownTokenBehavior {
    NoAnalysis,
    SurfaceAsOther,
    HeuristicGuess,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum AmbiguityPolicy {
    ReturnAllAnalyses,
    PreferDictionaryOrder,
    ConservativeDiagnostics,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum StressAvailability {
    #[default]
    Unknown,
    Unavailable,
    Available,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AnalyzerCapabilities {
    pub lexical_lookup: bool,
    pub returns_lemmas: bool,
    pub returns_lemma_ids: bool,
    pub returns_paradigm_ids: bool,
    pub returns_provenance: bool,
    pub stress: StressAvailability,
    pub ambiguity_policy: AmbiguityPolicy,
    pub unknown_token_behavior: UnknownTokenBehavior,
}

impl Default for AnalyzerCapabilities {
    fn default() -> Self {
        Self {
            lexical_lookup: false,
            returns_lemmas: false,
            returns_lemma_ids: false,
            returns_paradigm_ids: false,
            returns_provenance: false,
            stress: StressAvailability::Unknown,
            ambiguity_policy: AmbiguityPolicy::ReturnAllAnalyses,
            unknown_token_behavior: UnknownTokenBehavior::NoAnalysis,
        }
    }
}

impl AnalyzerCapabilities {
    pub fn project_lexicon(metadata: &[DictionaryMetadata]) -> Self {
        Self {
            lexical_lookup: true,
            returns_lemmas: true,
            returns_lemma_ids: true,
            returns_paradigm_ids: true,
            returns_provenance: true,
            stress: if metadata.iter().any(|item| item.has_stress) {
                StressAvailability::Available
            } else {
                StressAvailability::Unavailable
            },
            ambiguity_policy: AmbiguityPolicy::ConservativeDiagnostics,
            unknown_token_behavior: UnknownTokenBehavior::NoAnalysis,
        }
    }
}
