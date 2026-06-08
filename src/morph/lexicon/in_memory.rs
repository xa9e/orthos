#[derive(Debug, Default, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MorphLexicon {
    entries: HashMap<String, Vec<MorphAnalysis>>,
    metadata: Vec<DictionaryMetadata>,
    capabilities: AnalyzerCapabilities,
}

impl MorphLexicon {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn demo() -> Self {
        let metadata = DictionaryMetadata::curated_project("project.demo_morph", "Built-in demo morphology TSV");
        Self::parse_tsv_with_metadata(include_str!("../../../data/lexicon/demo_morph.tsv"), metadata)
            .unwrap_or_else(|_| Self::parse_tsv(include_str!("../../../data/lexicon/demo_morph.tsv")))
    }

    pub fn load_tsv(path: impl AsRef<Path>) -> Result<Self, io::Error> {
        let content = fs::read_to_string(path)?;
        Ok(Self::parse_tsv(&content))
    }

    pub fn parse_tsv(content: &str) -> Self {
        let metadata = DictionaryMetadata::curated_project("project.inline_tsv", "Inline project morphology TSV");
        parse_project_tsv(content, metadata, false).unwrap_or_default()
    }

    pub fn parse_tsv_with_metadata(
        content: &str,
        metadata: DictionaryMetadata,
    ) -> Result<Self, DictionaryImportError> {
        parse_project_tsv(content, metadata, true)
    }

    pub fn len(&self) -> usize {
        self.entries.values().map(Vec::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn metadata(&self) -> &[DictionaryMetadata] {
        &self.metadata
    }

    pub fn capabilities(&self) -> &AnalyzerCapabilities {
        &self.capabilities
    }

    pub fn analyze(&self, token: &str) -> Vec<MorphAnalysis> {
        <Self as MorphAnalyzer>::analyze(self, token)
    }

    pub fn attach_stress_records(&mut self, records: &[StressRecord]) -> usize {
        let mut attached = 0;
        for record in records {
            let Some(analyses) = self.entries.get_mut(&record.form) else {
                continue;
            };
            for analysis in analyses {
                if record.matches_analysis(analysis) {
                    analysis.stress = record.stress.clone();
                    if analysis.source_id.is_none() {
                        analysis.source_id = record.source_id.clone();
                    }
                    attached += 1;
                }
            }
        }

        if attached > 0 {
            for metadata in &mut self.metadata {
                metadata.has_stress = true;
            }
            self.capabilities = AnalyzerCapabilities::project_lexicon(&self.metadata);
        }

        attached
    }
}

impl MorphAnalyzer for MorphLexicon {
    fn analyze(&self, token: &str) -> Vec<MorphAnalysis> {
        self.entries
            .get(&lower_ru(token))
            .cloned()
            .unwrap_or_default()
    }

    fn metadata(&self) -> Vec<DictionaryMetadata> {
        self.metadata.clone()
    }

    fn capabilities(&self) -> AnalyzerCapabilities {
        self.capabilities.clone()
    }
}
