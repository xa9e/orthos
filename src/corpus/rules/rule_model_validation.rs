#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rule {
    pub id: String,
    pub title: String,
    pub domain: Domain,
    pub level: Level,
    pub status: RuleStatus,
    pub severity: Severity,
    #[serde(default)]
    pub source_refs: Vec<String>,
    #[serde(default)]
    pub rationale: Option<String>,
    #[serde(default)]
    pub explanation: Option<String>,
    #[serde(default)]
    pub suggestion: Option<String>,
    #[serde(default)]
    pub requires: Vec<Capability>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub rule_family: Option<RuleFamily>,
    #[serde(default)]
    pub confidence: Option<Confidence>,
    #[serde(default)]
    pub false_positive_risk: Option<FalsePositiveRisk>,
    #[serde(default)]
    pub pattern: Option<RulePattern>,
    #[serde(default)]
    pub constraints: Vec<RuleCondition>,
    #[serde(default)]
    pub exceptions: Vec<RuleCondition>,
    #[serde(default)]
    pub evidence: Vec<RuleEvidence>,
    #[serde(default)]
    pub supersedes: Vec<String>,
    #[serde(default)]
    pub related_rules: Vec<String>,
    #[serde(default)]
    pub implementation_notes: Option<String>,
    #[serde(default)]
    pub examples: Examples,
    pub detector: Detector,
}

impl Rule {
    fn validate(&self) -> Result<()> {
        self.validate_id()?;
        if self.title.trim().is_empty() {
            anyhow::bail!("empty title");
        }
        self.validate_source_refs()?;
        self.validate_rule_links()?;
        self.validate_extended_metadata()?;
        self.examples.validate()?;
        self.validate_implemented_contract()?;
        self.validate_declared_capabilities()?;
        Ok(())
    }

    fn validate_id(&self) -> Result<()> {
        validate_rule_id_shape("rule id", &self.id)
    }

    fn validate_implemented_contract(&self) -> Result<()> {
        if self.status != RuleStatus::Implemented {
            return Ok(());
        }
        if matches!(self.detector, Detector::Manual { .. }) {
            anyhow::bail!("implemented rule cannot use manual detector");
        }
        if self.examples.invalid.is_empty() {
            anyhow::bail!("implemented rule must have at least one invalid example");
        }
        if self.examples.valid.is_empty() {
            anyhow::bail!("implemented rule must have at least one valid example");
        }
        if self.confidence.is_none() {
            anyhow::bail!("implemented rule must declare confidence");
        }
        if self.false_positive_risk.is_none() {
            anyhow::bail!("implemented rule must declare false_positive_risk");
        }
        Ok(())
    }

    fn validate_source_refs(&self) -> Result<()> {
        if self.source_refs.is_empty() {
            anyhow::bail!("rule must declare at least one source_ref");
        }
        let mut seen = HashSet::new();
        for source_id in &self.source_refs {
            if source_id.trim().is_empty() {
                anyhow::bail!("rule contains an empty source_ref");
            }
            if !seen.insert(source_id.as_str()) {
                anyhow::bail!("rule contains duplicate source_ref `{source_id}`");
            }
        }
        Ok(())
    }

    fn validate_rule_links(&self) -> Result<()> {
        validate_rule_ref_list(&self.id, "related_rules", &self.related_rules)?;
        validate_rule_ref_list(&self.id, "supersedes", &self.supersedes)?;
        if self.related_rules.iter().any(|rule_id| rule_id == &self.id) {
            anyhow::bail!("related_rules cannot contain the rule itself");
        }
        if self.supersedes.iter().any(|rule_id| rule_id == &self.id) {
            anyhow::bail!("supersedes cannot contain the rule itself");
        }
        Ok(())
    }

    fn validate_extended_metadata(&self) -> Result<()> {
        validate_optional_non_empty("implementation_notes", self.implementation_notes.as_deref())?;
        validate_string_list("tags", &self.tags)?;
        if let Some(pattern) = &self.pattern {
            pattern.validate()?;
        }
        validate_conditions("constraints", &self.constraints)?;
        validate_conditions("exceptions", &self.exceptions)?;
        for (index, evidence) in self.evidence.iter().enumerate() {
            evidence.validate(index)?;
        }
        Ok(())
    }

    fn validate_declared_capabilities(&self) -> Result<()> {
        let declared: HashSet<Capability> = self.requires.iter().copied().collect();
        let mut missing = Vec::new();

        for capability in self.capabilities_implied_by_detector() {
            if !declared.contains(&capability) {
                missing.push(capability_label(capability));
            }
        }

        for capability in self.capabilities_implied_by_extended_metadata() {
            if !declared.contains(&capability) {
                missing.push(capability_label(capability));
            }
        }

        for (capability, needles) in CAPABILITY_TEXT_MARKERS {
            if contains_any_marker(&self.capability_text(), needles) && !declared.contains(capability) {
                missing.push(capability_label(*capability));
            }
        }

        missing.sort_unstable();
        missing.dedup();
        if !missing.is_empty() {
            anyhow::bail!("rule text implies missing requires: {}", missing.join(", "));
        }

        Ok(())
    }

    fn capabilities_implied_by_extended_metadata(&self) -> Vec<Capability> {
        let mut out = Vec::new();
        if let Some(rule_family) = self.rule_family {
            extend_unique(&mut out, capabilities_for_rule_family(rule_family));
        }
        if let Some(pattern) = &self.pattern {
            extend_unique(&mut out, capabilities_for_pattern_kind(pattern.kind));
        }
        for condition in self.constraints.iter().chain(self.exceptions.iter()) {
            extend_unique(&mut out, capabilities_for_linguistic_concept(condition.kind));
        }
        out
    }

    fn capabilities_implied_by_detector(&self) -> Vec<Capability> {
        match &self.detector {
            Detector::AdjNounAgreementDemo { .. } => vec![Capability::Morphology],
            Detector::NominalGroupModifierAgreementBasic { .. }
            | Detector::SubjectPredicateAgreementBasic { .. } => {
                vec![Capability::Tokenization, Capability::Morphology, Capability::Syntax]
            }
            Detector::TsyaHeuristic { .. }
            | Detector::NegatedVerbSpacingBasic { .. } => {
                vec![Capability::Tokenization, Capability::Morphology]
            }
            Detector::PhraseMap { .. } | Detector::ConfusableTokens { .. } => {
                vec![Capability::Tokenization, Capability::Lexicon]
            }
            Detector::ParticleHyphenMissing { .. } => vec![Capability::Tokenization, Capability::Lexicon],
            Detector::CliticHyphenMissing { .. } => {
                vec![Capability::Tokenization, Capability::WordFormation]
            }
            Detector::PolCompoundHyphenMissing { .. } => {
                vec![Capability::Tokenization, Capability::WordFormation]
            }
            Detector::PrefixFinalZSAssimilation { .. } => {
                vec![Capability::Tokenization, Capability::Lexicon, Capability::WordFormation]
            }
            Detector::RepeatedWord { .. }
            | Detector::NoWhitespaceBeforePunctuation { .. }
            | Detector::MissingWhitespaceAfterPunctuation { .. }
            | Detector::UnbalancedQuotes { .. }
            | Detector::UnpairedDelimiters { .. }
            | Detector::SpaceAfterOpeningPunctuation { .. }
            | Detector::SpaceBeforeClosingQuote { .. }
            | Detector::HyphenInsteadOfDash { .. }
            | Detector::DashSpacing { .. }
            | Detector::MultiplePunctuation { .. }
            | Detector::NumberUnitSpacing { .. }
            | Detector::IntroductoryPhraseComma { .. }
            | Detector::CoordinationCommaBasic { .. }
            | Detector::PhraseologicalCoordinationComma { .. }
            | Detector::MissingCommaBeforeSubordinator { .. }
            | Detector::MixedAlphabetWord { .. }
            | Detector::ZhiShiChaShcha { .. } => vec![Capability::Tokenization],
            Detector::MultipleSpaces { .. }
            | Detector::Regex { .. }
            | Detector::SeparateParticleHyphenated { .. } => vec![Capability::Regex],
           Detector::SentenceInitialLowercase { .. } | Detector::MissingSentenceTerminal { .. } => {
               vec![Capability::SentenceBoundaries]
           }
            Detector::PrepositionGovernmentBasic { .. }
            | Detector::PrepositionNominalGroupGovernmentBasic { .. }
            | Detector::NumeralNounAgreementBasic { .. }
            | Detector::NumeralNominalGroupAgreementBasic { .. }
            | Detector::CompoundNumeralNominalGroupAgreementBasic { .. }
            | Detector::TypedCompoundNumeralNominalGroupAgreementBasic { .. }
            | Detector::VerbGovernmentBasic { .. } => {
                vec![Capability::Tokenization, Capability::Morphology, Capability::Syntax]
            }
            Detector::DocumentAbbreviationExpansion { .. }
            | Detector::DocumentStyleConsistency { .. } => {
                vec![Capability::Tokenization, Capability::Syntax]
            }
            Detector::Manual { .. } => Vec::new(),
        }
    }

    fn capability_text(&self) -> String {
        let mut parts = vec![self.id.as_str(), self.title.as_str()];
        if let Some(value) = self.rationale.as_deref() {
            parts.push(value);
        }
        if let Some(value) = self.explanation.as_deref() {
            parts.push(value);
        }
        if let Some(value) = self.suggestion.as_deref() {
            parts.push(value);
        }
        match &self.detector {
            Detector::Manual { note } => parts.push(note),
            Detector::Regex { message, .. }
            | Detector::RepeatedWord { message }
            | Detector::MultipleSpaces { message, .. }
            | Detector::NoWhitespaceBeforePunctuation { message, .. }
            | Detector::MissingWhitespaceAfterPunctuation { message }
            | Detector::MissingSentenceTerminal { message, .. }
            | Detector::SentenceInitialLowercase { message }
            | Detector::ParticleHyphenMissing { message, .. }
            | Detector::CliticHyphenMissing { message, .. }
            | Detector::SeparateParticleHyphenated { message, .. }
            | Detector::MissingCommaBeforeSubordinator { message, .. }
            | Detector::IntroductoryPhraseComma { message, .. }
            | Detector::CoordinationCommaBasic { message }
            | Detector::PhraseologicalCoordinationComma { message }
            | Detector::UnbalancedQuotes { message }
            | Detector::UnpairedDelimiters { message, .. }
            | Detector::MixedAlphabetWord { message }
            | Detector::ConfusableTokens { message, .. }
            | Detector::PhraseMap { message, .. }
            | Detector::ZhiShiChaShcha { message, .. }
            | Detector::TsyaHeuristic { message, .. }
            | Detector::NegatedVerbSpacingBasic { message }
            | Detector::AdjNounAgreementDemo { message }
            | Detector::NominalGroupModifierAgreementBasic { message }
            | Detector::SubjectPredicateAgreementBasic { message }
            | Detector::PrepositionGovernmentBasic { message }
            | Detector::PrepositionNominalGroupGovernmentBasic { message }
            | Detector::NumeralNounAgreementBasic { message }
            | Detector::NumeralNominalGroupAgreementBasic { message }
            | Detector::CompoundNumeralNominalGroupAgreementBasic { message }
            | Detector::TypedCompoundNumeralNominalGroupAgreementBasic { message }
            | Detector::SpaceAfterOpeningPunctuation { message }
            | Detector::SpaceBeforeClosingQuote { message }
            | Detector::HyphenInsteadOfDash { message }
            | Detector::DashSpacing { message }
            | Detector::MultiplePunctuation { message }
            | Detector::NumberUnitSpacing { message, .. }
            | Detector::PolCompoundHyphenMissing { message }
            | Detector::PrefixFinalZSAssimilation { message }
            | Detector::DocumentAbbreviationExpansion { message }
            | Detector::DocumentStyleConsistency { message }
            | Detector::VerbGovernmentBasic { message } => parts.push(message),
        }
        parts.extend(self.tags.iter().map(String::as_str));
        parts.join(" ").to_lowercase()
    }
}
