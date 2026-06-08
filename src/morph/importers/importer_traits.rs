#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryImportError {
    pub line: Option<usize>,
    pub message: String,
}

impl DictionaryImportError {
    pub fn new(line: Option<usize>, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for DictionaryImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "dictionary import error at line {line}: {}", self.message),
            None => write!(f, "dictionary import error: {}", self.message),
        }
    }
}

impl std::error::Error for DictionaryImportError {}

/// Stable dictionary import boundary.
///
/// Importers must convert external tags to typed `MorphFeatures`, preserve
/// unsupported tags in `unrecognized_tags`, and attach source/provenance ids.
pub trait DictionaryImporter: std::fmt::Debug + Send + Sync {
    fn metadata(&self) -> DictionaryMetadata;

    fn import_lexicon(&self, content: &str) -> Result<MorphLexicon, DictionaryImportError>;

    /// Backward-compatible helper retained from the demo façade.
    fn import_tsv(&self, content: &str) -> MorphLexicon {
        self.import_lexicon(content).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectTsvDictionaryImporter {
    metadata: DictionaryMetadata,
}

impl ProjectTsvDictionaryImporter {
    pub fn new(metadata: DictionaryMetadata) -> Self {
        Self { metadata }
    }

    pub fn curated(source_id: impl Into<SourceId>, name: impl Into<String>) -> Self {
        Self::new(DictionaryMetadata::curated_project(source_id, name))
    }
}


#[derive(Debug, Clone)]
pub struct OpenCorporaXmlDictionaryImporter {
    metadata: DictionaryMetadata,
}

impl OpenCorporaXmlDictionaryImporter {
    pub fn new(metadata: DictionaryMetadata) -> Self {
        Self { metadata }
    }

    pub fn fixture(source_id: impl Into<SourceId>, name: impl Into<String>) -> Self {
        let mut metadata = DictionaryMetadata::new(source_id, name, DictionaryFormat::OpenCorporaXml);
        metadata.license = LicenseStatus::LocalGenerationOnly;
        Self::new(metadata)
    }
}

impl DictionaryImporter for OpenCorporaXmlDictionaryImporter {
    fn metadata(&self) -> DictionaryMetadata {
        self.metadata.clone()
    }

    fn import_lexicon(&self, content: &str) -> Result<MorphLexicon, DictionaryImportError> {
        parse_opencorpora_xml_fixture(content, self.metadata.clone())
    }
}

#[derive(Debug, Clone)]
pub struct OpenCorporaCsvDictionaryImporter {
    metadata: DictionaryMetadata,
}

impl OpenCorporaCsvDictionaryImporter {
    pub fn new(metadata: DictionaryMetadata) -> Self {
        Self { metadata }
    }

    pub fn fixture(source_id: impl Into<SourceId>, name: impl Into<String>) -> Self {
        let mut metadata = DictionaryMetadata::new(source_id, name, DictionaryFormat::OpenCorporaCsv);
        metadata.license = LicenseStatus::LocalGenerationOnly;
        Self::new(metadata)
    }
}

impl DictionaryImporter for OpenCorporaCsvDictionaryImporter {
    fn metadata(&self) -> DictionaryMetadata {
        self.metadata.clone()
    }

    fn import_lexicon(&self, content: &str) -> Result<MorphLexicon, DictionaryImportError> {
        parse_delimited_dictionary_fixture(content, self.metadata.clone(), FixtureDialect::OpenCorporaCsv)
    }
}

#[derive(Debug, Clone)]
pub struct PymorphyExportDictionaryImporter {
    metadata: DictionaryMetadata,
}

impl PymorphyExportDictionaryImporter {
    pub fn new(metadata: DictionaryMetadata) -> Self {
        Self { metadata }
    }

    pub fn fixture(source_id: impl Into<SourceId>, name: impl Into<String>) -> Self {
        let mut metadata = DictionaryMetadata::new(source_id, name, DictionaryFormat::PymorphyExport);
        metadata.license = LicenseStatus::LocalGenerationOnly;
        Self::new(metadata)
    }
}

impl DictionaryImporter for PymorphyExportDictionaryImporter {
    fn metadata(&self) -> DictionaryMetadata {
        self.metadata.clone()
    }

    fn import_lexicon(&self, content: &str) -> Result<MorphLexicon, DictionaryImportError> {
        parse_delimited_dictionary_fixture(content, self.metadata.clone(), FixtureDialect::PymorphyExport)
    }
}

impl DictionaryImporter for ProjectTsvDictionaryImporter {
    fn metadata(&self) -> DictionaryMetadata {
        self.metadata.clone()
    }

    fn import_lexicon(&self, content: &str) -> Result<MorphLexicon, DictionaryImportError> {
        parse_project_tsv(content, self.metadata.clone(), true)
    }
}
