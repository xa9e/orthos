#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Examples {
    #[serde(default)]
    pub valid: Vec<String>,
    #[serde(default)]
    pub invalid: Vec<String>,
}

impl Examples {
    fn validate(&self) -> Result<()> {
        validate_string_list("examples.valid", &self.valid)?;
        validate_string_list("examples.invalid", &self.invalid)?;

        let valid: HashSet<&str> = self.valid.iter().map(String::as_str).collect();
        for example in &self.invalid {
            if valid.contains(example.as_str()) {
                anyhow::bail!(
                    "examples.valid and examples.invalid contain duplicate example `{example}`"
                );
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RulePattern {
    pub kind: PatternKind,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub captures: Vec<String>,
}

impl RulePattern {
    fn validate(&self) -> Result<()> {
        validate_optional_non_empty("pattern.value", self.value.as_deref())?;
        validate_optional_non_empty("pattern.description", self.description.as_deref())?;
        validate_string_list("pattern.captures", &self.captures)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleCondition {
    pub kind: LinguisticConcept,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

impl RuleCondition {
    fn validate(&self, field: &str, index: usize) -> Result<()> {
        let value_field = format!("{field}[{index}].value");
        let description_field = format!("{field}[{index}].description");
        let features_field = format!("{field}[{index}].features");
        validate_optional_non_empty(&value_field, self.value.as_deref())?;
        validate_optional_non_empty(&description_field, self.description.as_deref())?;
        validate_string_list(&features_field, &self.features)?;
        if self.value.is_none() && self.description.is_none() && self.features.is_empty() {
            anyhow::bail!("{field}[{index}] must include value, description, or features");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleEvidence {
    pub kind: EvidenceKind,
    #[serde(default)]
    pub source_ref: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

impl RuleEvidence {
    fn validate(&self, index: usize) -> Result<()> {
        validate_optional_non_empty("evidence.source_ref", self.source_ref.as_deref())?;
        validate_optional_non_empty("evidence.note", self.note.as_deref())?;
        if self.source_ref.is_none() && self.note.is_none() {
            anyhow::bail!("evidence[{index}] must include source_ref or note");
        }
        Ok(())
    }
}
