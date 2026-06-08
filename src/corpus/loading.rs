#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorpusFile {
    pub version: u32,
    #[serde(default)]
    pub source_refs: Vec<SourceRef>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceRef {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub source_type: Option<SourceType>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub bibliographic_pointer: Option<String>,
    #[serde(default)]
    pub license_note: Option<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub authority: Option<SourceAuthority>,
    #[serde(default)]
    pub note: Option<String>,
}

impl SourceRef {
    fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            anyhow::bail!("source with empty id");
        }
        if self.title.trim().is_empty() {
            anyhow::bail!("source `{}` has empty title", self.id);
        }
        validate_optional_non_empty("source.url", self.url.as_deref())?;
        validate_optional_non_empty(
            "source.bibliographic_pointer",
            self.bibliographic_pointer.as_deref(),
        )?;
        validate_optional_non_empty("source.license_note", self.license_note.as_deref())?;
        validate_optional_non_empty("source.year", self.year.as_deref())?;
        validate_optional_non_empty("source.note", self.note.as_deref())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Corpus {
    pub version: u32,
    pub sources: Vec<SourceRef>,
    pub rules: Vec<Rule>,
}

impl Corpus {
    pub fn load_dir(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut sources = Vec::new();
        let mut rules = Vec::new();
        let mut seen_rule_ids = HashSet::new();

        let mut yaml_files = Vec::<PathBuf>::new();
        for entry in WalkDir::new(path) {
            let entry = entry.with_context(|| format!("failed to walk {}", path.display()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) else {
                continue;
            };
            if matches!(ext, "yaml" | "yml") {
                yaml_files.push(entry.path().to_path_buf());
            }
        }
        yaml_files.sort();

        for file in yaml_files {
            let raw = fs::read_to_string(&file)
                .with_context(|| format!("failed to read corpus file {}", file.display()))?;
            let parsed: CorpusFile = serde_yaml_ng::from_str(&raw)
                .with_context(|| format!("invalid YAML schema in {}", file.display()))?;
            sources.extend(parsed.source_refs);
            for rule in parsed.rules {
                if !seen_rule_ids.insert(rule.id.clone()) {
                    anyhow::bail!("duplicate rule id `{}` in {}", rule.id, file.display());
                }
                rule.validate()
                    .map_err(|err| anyhow::anyhow!("invalid rule `{}` in {}: {err}", rule.id, file.display()))?;
                rules.push(rule);
            }
        }

        let corpus = Self {
            version: 1,
            sources,
            rules,
        };
        corpus.validate_references()?;
        Ok(corpus)
    }

    pub fn implemented_rules(&self) -> impl Iterator<Item = &Rule> {
        self.rules.iter().filter(|r| r.status == RuleStatus::Implemented)
    }

    pub fn rules_by_detector(&self) -> HashMap<&'static str, usize> {
        let mut out = HashMap::new();
        for rule in &self.rules {
            *out.entry(rule.detector.kind()).or_insert(0) += 1;
        }
        out
    }

    fn validate_references(&self) -> Result<()> {
        let mut seen_sources = HashSet::new();
        for source in &self.sources {
            source.validate()?;
            if !seen_sources.insert(source.id.as_str()) {
                anyhow::bail!("duplicate source id `{}`", source.id);
            }
        }
        let rule_ids: HashSet<&str> = self.rules.iter().map(|rule| rule.id.as_str()).collect();
        for rule in &self.rules {
            for source_id in &rule.source_refs {
                if !seen_sources.contains(source_id.as_str()) {
                    anyhow::bail!("rule `{}` references unknown source `{}`", rule.id, source_id);
                }
            }
            for evidence in &rule.evidence {
                if let Some(source_id) = evidence.source_ref.as_deref()
                    && !seen_sources.contains(source_id)
                {
                    anyhow::bail!(
                        "rule `{}` evidence references unknown source `{}`",
                        rule.id,
                        source_id
                    );
                }
            }
            for related_id in &rule.related_rules {
                if !rule_ids.contains(related_id.as_str()) {
                    anyhow::bail!(
                        "rule `{}` references unknown related rule `{}`",
                        rule.id,
                        related_id
                    );
                }
            }
            for superseded_id in &rule.supersedes {
                if !rule_ids.contains(superseded_id.as_str()) {
                    anyhow::bail!(
                        "rule `{}` supersedes unknown rule `{}`",
                        rule.id,
                        superseded_id
                    );
                }
            }
        }
        Ok(())
    }
}
