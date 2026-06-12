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
            let Some(analyses) = self.entries.get_mut(&morph_lookup_key(&record.form)) else {
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
        let Some(bucket) = self.entries.get(&morph_lookup_key(token)) else {
            return Vec::new();
        };
        // Entries are bucketed under the ё-folded key, so a bucket can mix
        // analyses for spellings that differ only in ё (колеса/колёса).
        // An explicit ё in the token is trusted as a disambiguator; a token
        // without ё keeps the whole bucket as genuine ambiguity.
        let lowered = lower_ru(token);
        if lowered.contains('ё') {
            let exact = bucket
                .iter()
                .filter(|analysis| lower_ru(&analysis.form) == lowered)
                .cloned()
                .collect::<Vec<_>>();
            if !exact.is_empty() {
                return exact;
            }
        }
        bucket.clone()
    }

    fn analyses_for_lemma(&self, lemma: &str) -> Vec<MorphAnalysis> {
        let mut out = self
            .entries
            .values()
            .flat_map(|analyses| analyses.iter())
            .filter(|analysis| analysis.lemma == lemma)
            .cloned()
            .collect::<Vec<_>>();
        out.sort_by(|left, right| {
            left.form
                .cmp(&right.form)
                .then_with(|| left.pos.cmp(&right.pos))
                .then_with(|| left.features.raw_tags.cmp(&right.features.raw_tags))
        });
        out.dedup_by(|left, right| {
            left.form == right.form
                && left.pos == right.pos
                && left.features.raw_tags == right.features.raw_tags
        });
        out
    }

    fn metadata(&self) -> Vec<DictionaryMetadata> {
        self.metadata.clone()
    }

    fn capabilities(&self) -> AnalyzerCapabilities {
        self.capabilities.clone()
    }
}
