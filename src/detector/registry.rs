#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DetectorMetadata {
    pub kind: &'static str,
    pub capabilities: &'static [Capability],
    pub deterministic: bool,
    pub description: &'static str,
}

pub trait DetectorRunner: Send + Sync {
    fn metadata(&self) -> &'static DetectorMetadata;
    fn run(&self, rule: &Rule, ctx: &DetectorContext<'_>) -> Result<Vec<Issue>>;
}

pub struct DetectorRegistry {
    runners: BTreeMap<&'static str, Box<dyn DetectorRunner>>,
}

impl DetectorMetadata {
    pub fn validate(&self) -> Result<()> {
        if self.kind.trim().is_empty() {
            anyhow::bail!("detector metadata has empty kind");
        }
        if self.description.trim().is_empty() {
            anyhow::bail!("detector `{}` has empty description", self.kind);
        }

        let mut seen = BTreeSet::new();
        let mut previous = None;
        for capability in self.capabilities {
            if let Some(previous) = previous
                && previous > *capability
            {
                anyhow::bail!(
                    "detector `{}` capabilities must be sorted for stable metadata",
                    self.kind
                );
            }
            if !seen.insert(*capability) {
                anyhow::bail!(
                    "detector `{}` declares duplicate capability `{}`",
                    self.kind,
                    capability
                );
            }
            previous = Some(*capability);
        }

        Ok(())
    }
}

impl std::fmt::Debug for DetectorRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetectorRegistry")
            .field("kinds", &self.kinds())
            .finish()
    }
}

impl DetectorRegistry {
    pub fn new() -> Self {
        Self {
            runners: BTreeMap::new(),
        }
    }

    pub fn register<R>(&mut self, runner: R)
    where
        R: DetectorRunner + 'static,
    {
        let kind = runner.metadata().kind;
        let previous = self.runners.insert(kind, Box::new(runner));
        assert!(previous.is_none(), "duplicate detector runner registration for `{kind}`");
    }

    pub fn contains_kind(&self, kind: &str) -> bool {
        self.runners.contains_key(kind)
    }

    pub fn metadata(&self, kind: &str) -> Option<&'static DetectorMetadata> {
        self.runners.get(kind).map(|runner| runner.metadata())
    }

    pub fn all_metadata(&self) -> Vec<&'static DetectorMetadata> {
        self.runners.values().map(|runner| runner.metadata()).collect()
    }

    pub fn validate(&self) -> Result<()> {
        for (kind, runner) in &self.runners {
            let metadata = runner.metadata();
            if metadata.kind != *kind {
                anyhow::bail!(
                    "detector registry key `{}` does not match metadata kind `{}`",
                    kind,
                    metadata.kind
                );
            }
            metadata.validate()?;
        }
        Ok(())
    }

    pub fn kinds(&self) -> Vec<&'static str> {
        self.runners.keys().copied().collect()
    }

    pub fn run(&self, rule: &Rule, ctx: &DetectorContext<'_>) -> Result<Vec<Issue>> {
        let kind = rule.detector.kind();
        let Some(runner) = self.runners.get(kind) else {
            anyhow::bail!("unknown detector kind `{kind}` for rule `{}`", rule.id);
        };
        runner.run(rule, ctx)
    }
}

impl Default for DetectorRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register(RegexRunner);
        registry.register(RepeatedWordRunner);
        registry.register(MultipleSpacesRunner);
        registry.register(NoWhitespaceBeforePunctuationRunner);
        registry.register(MissingWhitespaceAfterPunctuationRunner);
        registry.register(MissingSentenceTerminalRunner);
        registry.register(SentenceInitialLowercaseRunner);
        registry.register(ParticleHyphenMissingRunner);
        registry.register(CliticHyphenMissingRunner);
        registry.register(SeparateParticleHyphenatedRunner);
        registry.register(MissingCommaBeforeSubordinatorRunner);
        registry.register(IntroductoryPhraseCommaRunner);
        registry.register(CoordinationCommaBasicRunner);
        registry.register(DocumentAbbreviationExpansionRunner);
        registry.register(DocumentStyleConsistencyRunner);
        registry.register(UnbalancedQuotesRunner);
        registry.register(UnpairedDelimitersRunner);
        registry.register(MixedAlphabetWordRunner);
        registry.register(ConfusableTokensRunner);
        registry.register(PhraseMapRunner);
        registry.register(ZhiShiChaShchaRunner);
        registry.register(TsyaHeuristicRunner);
        registry.register(NegatedVerbSpacingBasicRunner);
        registry.register(AdjNounAgreementDemoRunner);
        registry.register(NominalGroupModifierAgreementBasicRunner);
        registry.register(SubjectPredicateAgreementBasicRunner);
        registry.register(PrepositionGovernmentBasicRunner);
        registry.register(PrepositionNominalGroupGovernmentBasicRunner);
        registry.register(VerbGovernmentBasicRunner);
        registry.register(NumeralNounAgreementBasicRunner);
        registry.register(NumeralNominalGroupAgreementBasicRunner);
        registry.register(CompoundNumeralNominalGroupAgreementBasicRunner);
        registry.register(TypedCompoundNumeralNominalGroupAgreementBasicRunner);
        registry.register(SpaceAfterOpeningPunctuationRunner);
        registry.register(SpaceBeforeClosingQuoteRunner);
        registry.register(HyphenInsteadOfDashRunner);
        registry.register(DashSpacingRunner);
        registry.register(MultiplePunctuationRunner);
        registry.register(NumberUnitSpacingRunner);
        registry.register(PolCompoundHyphenMissingRunner);
        registry.register(PrefixFinalZSAssimilationRunner);
        registry.register(ManualRunner);
        registry
    }
}

pub fn default_detector_registry() -> &'static DetectorRegistry {
    &DEFAULT_DETECTOR_REGISTRY
}

pub struct DetectorContext<'a> {
    pub text: &'a str,
    pub line_index: &'a LineIndex<'a>,
    pub morph: &'a dyn MorphAnalyzer,
    pub analysis: &'a AnalysisContext<'a>,
}

impl<'a> DetectorContext<'a> {
    pub fn new(analysis: &'a AnalysisContext<'a>) -> Self {
        Self {
            text: analysis.text(),
            line_index: analysis.line_index(),
            morph: analysis.morph(),
            analysis,
        }
    }

    pub fn tokens(&self) -> &[Token<'a>] {
        self.analysis.tokens()
    }

    pub fn word_tokens(&self) -> &[Token<'a>] {
        self.analysis.word_tokens()
    }

    pub fn morphosyntax(&self) -> &MorphosyntaxDocument<'a> {
        self.analysis.morphosyntax()
    }

    pub fn fact_store(&self) -> &LinguisticFactStore<'a> {
        self.analysis.fact_store()
    }
}
