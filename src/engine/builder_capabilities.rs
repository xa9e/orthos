#[derive(Debug, Clone)]
pub struct CheckerBuilder {
    corpus: Corpus,
    morph: Arc<dyn MorphAnalyzer>,
    detectors: Arc<DetectorRegistry>,
    capabilities: CapabilityRegistry,
}

impl CheckerBuilder {
    pub fn new(corpus: Corpus) -> Self {
        Self {
            corpus,
            morph: Arc::new(MorphLexicon::demo()),
            detectors: Arc::new(DetectorRegistry::default()),
            capabilities: CapabilityRegistry::default(),
        }
    }

    pub fn with_morph_lexicon(self, morph: MorphLexicon) -> Self {
        self.with_morph_analyzer(morph)
    }

    pub fn with_morph_analyzer<A>(mut self, morph: A) -> Self
    where
        A: MorphAnalyzer + 'static,
    {
        self.morph = Arc::new(morph);
        self
    }

    pub fn with_detector_registry(mut self, detectors: DetectorRegistry) -> Self {
        self.detectors = Arc::new(detectors);
        self
    }

    pub fn with_capabilities(mut self, capabilities: CapabilityRegistry) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn build(self) -> Checker {
        Checker::with_components(self.corpus, self.morph, self.detectors, self.capabilities)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CapabilityRegistry {
    available: BTreeSet<Capability>,
}

impl CapabilityRegistry {
    pub fn new(capabilities: impl IntoIterator<Item = Capability>) -> Self {
        Self {
            available: capabilities.into_iter().collect(),
        }
    }

    pub fn default_engine() -> Self {
        Self::new([
            Capability::Tokenization,
            Capability::SentenceBoundaries,
            Capability::Regex,
            Capability::Lexicon,
            Capability::Morphology,
            Capability::Syntax,
            Capability::WordFormation,
        ])
    }

    pub fn contains(&self, capability: Capability) -> bool {
        self.available.contains(&capability)
    }

    pub fn capabilities(&self) -> impl Iterator<Item = Capability> + '_ {
        self.available.iter().copied()
    }

    pub fn supports_rule(&self, rule: &Rule) -> bool {
        self.missing_for_rule(rule).is_empty()
    }

    pub fn missing_for_rule(&self, rule: &Rule) -> Vec<Capability> {
        rule.requires
            .iter()
            .copied()
            .filter(|capability| !self.available.contains(capability))
            .collect()
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::default_engine()
    }
}
