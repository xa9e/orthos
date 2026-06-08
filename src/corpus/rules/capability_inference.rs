const CAPABILITY_TEXT_MARKERS: &[(Capability, &[&str])] = &[
    (
        Capability::Morphology,
        &[
            "морфолог",
            "падеж",
            "род ",
            "числител",
            "склон",
            "спряж",
            "часть речи",
            "лемм",
            "словоформ",
            "инфинитив",
            "причаст",
            "деепричаст",
        ],
    ),
    (
        Capability::Syntax,
        &[
            "синтак",
            "dependency",
            "зависим",
            "подлежащ",
            "сказуем",
            "придаточ",
            "однород",
            "оборот",
            "clause",
            "parser",
            "координац",
            "управлен",
            "союз",
        ],
    ),
    (
        Capability::WordFormation,
        &[
            "словообраз",
            "морфем",
            "приставк",
            "суффикс",
            "корн",
            "производящ",
            "производн",
            "основ",
        ],
    ),
    (Capability::Stress, &["ударен", "ударён", "ударение"]),
];

fn contains_any_marker(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn capability_label(capability: Capability) -> &'static str {
    match capability {
        Capability::Tokenization => "tokenization",
        Capability::SentenceBoundaries => "sentence_boundaries",
        Capability::Regex => "regex",
        Capability::Lexicon => "lexicon",
        Capability::Morphology => "morphology",
        Capability::Syntax => "syntax",
        Capability::Semantics => "semantics",
        Capability::NamedEntities => "named_entities",
        Capability::WordFormation => "word_formation",
        Capability::Stress => "stress",
        Capability::Benchmark => "benchmark",
    }
}

fn validate_rule_ref_list(owner_id: &str, field: &str, values: &[String]) -> Result<()> {
    let mut seen = HashSet::new();
    for value in values {
        validate_rule_id_reference(field, value)?;
        if !seen.insert(value.as_str()) {
            anyhow::bail!("rule `{owner_id}` contains duplicate {field} `{value}`");
        }
    }
    Ok(())
}

fn validate_rule_id_reference(field: &str, value: &str) -> Result<()> {
    validate_rule_id_shape(&format!("{field} rule id"), value)
}

fn validate_rule_id_shape(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("{field} must not be empty");
    }

    let components: Vec<&str> = value.split('.').collect();
    if components.len() < 3 || components.first() != Some(&"ru") {
        anyhow::bail!("{field} `{value}` must use `ru.<namespace>.<snake_case_slug>`");
    }

    for component in components.iter().skip(1) {
        validate_rule_id_component(field, value, component)?;
    }

    Ok(())
}

fn validate_rule_id_component(field: &str, value: &str, component: &str) -> Result<()> {
    if component.is_empty() {
        anyhow::bail!("{field} `{value}` must not contain empty dot-separated components");
    }
    let Some(first) = component.chars().next() else {
        anyhow::bail!("{field} `{value}` must not contain empty dot-separated components");
    };
    if !component
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        anyhow::bail!(
            "{field} `{value}` components must contain only ASCII lowercase letters, digits, and `_`"
        );
    }
    if !first.is_ascii_lowercase() {
        anyhow::bail!("{field} `{value}` components must start with an ASCII lowercase letter");
    }
    if component.ends_with('_') || component.contains("__") {
        anyhow::bail!("{field} `{value}` components must use compact snake_case without empty parts");
    }
    Ok(())
}

fn validate_optional_non_empty(field: &str, value: Option<&str>) -> Result<()> {
    if matches!(value, Some(value) if value.trim().is_empty()) {
        anyhow::bail!("{field} must not be empty");
    }
    Ok(())
}

fn validate_string_list(field: &str, values: &[String]) -> Result<()> {
    let mut seen = HashSet::new();
    for value in values {
        if value.trim().is_empty() {
            anyhow::bail!("{field} contains an empty value");
        }
        if !seen.insert(value.as_str()) {
            anyhow::bail!("{field} contains duplicate value `{value}`");
        }
    }
    Ok(())
}

fn validate_conditions(field: &str, values: &[RuleCondition]) -> Result<()> {
    for (index, value) in values.iter().enumerate() {
        value.validate(field, index)?;
    }
    Ok(())
}

fn extend_unique(out: &mut Vec<Capability>, capabilities: &[Capability]) {
    for capability in capabilities {
        if !out.contains(capability) {
            out.push(*capability);
        }
    }
}

fn capabilities_for_rule_family(rule_family: RuleFamily) -> &'static [Capability] {
    match rule_family {
        RuleFamily::Orthography | RuleFamily::Punctuation | RuleFamily::Typography => {
            &[Capability::Tokenization]
        }
        RuleFamily::Grammar | RuleFamily::MorphologyDependent => &[Capability::Morphology],
        RuleFamily::SyntaxDependent => &[Capability::Syntax],
        RuleFamily::WordFormation => &[Capability::WordFormation],
        RuleFamily::StressDependent => &[Capability::Stress],
        RuleFamily::LexicalStyle => &[Capability::Lexicon],
    }
}

fn capabilities_for_pattern_kind(kind: PatternKind) -> &'static [Capability] {
    match kind {
        PatternKind::Surface | PatternKind::TokenSequence | PatternKind::PunctuationContext => {
            &[Capability::Tokenization]
        }
        PatternKind::Regex => &[Capability::Regex],
        PatternKind::Morphological => &[Capability::Morphology],
        PatternKind::Syntactic | PatternKind::Dependency => &[Capability::Syntax],
        PatternKind::WordFormation => &[Capability::WordFormation],
        PatternKind::Stress => &[Capability::Stress],
        PatternKind::LexicalSet => &[Capability::Lexicon],
    }
}

fn capabilities_for_linguistic_concept(kind: LinguisticConcept) -> &'static [Capability] {
    match kind {
        LinguisticConcept::Agreement | LinguisticConcept::Morphology => &[Capability::Morphology],
        LinguisticConcept::Government => &[Capability::Syntax, Capability::Morphology],
        LinguisticConcept::Coordination
        | LinguisticConcept::ClauseBoundary
        | LinguisticConcept::ParentheticalExpression
        | LinguisticConcept::DirectSpeech
        | LinguisticConcept::Syntax => &[Capability::Syntax],
        LinguisticConcept::Derivation => &[Capability::WordFormation],
        LinguisticConcept::SpellingPattern
        | LinguisticConcept::TokenContext
        | LinguisticConcept::SentenceBoundary => &[Capability::Tokenization],
        LinguisticConcept::StressRequirement => &[Capability::Stress],
        LinguisticConcept::IdiomFixedExpressionException
        | LinguisticConcept::Lexical
        | LinguisticConcept::StyleRegister
        | LinguisticConcept::NamedEntity => &[Capability::Lexicon],
    }
}
