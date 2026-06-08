#[test]
fn accepts_extended_rule_model_metadata() {
    let dir = temp_rules_dir("extended-rule-model-metadata");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.base
    title: Base fixture rule
    domain: grammar
    level: advanced
    status: research
    severity: warning
    source_refs: [fixture_source]
    requires: [morphology, lexicon]
    rule_family: morphology_dependent
    confidence: medium
    false_positive_risk: high
    pattern:
      kind: morphological
      description: Morphological agreement signature fixture.
      captures: [head, dependent]
    constraints:
      - kind: agreement
        description: Dependent form must agree with the head token.
        features: [case, number, gender]
    exceptions:
      - kind: idiom_fixed_expression_exception
        value: fixture idiom
    evidence:
      - kind: normative_source
        source_ref: fixture_source
    implementation_notes: Requires disambiguated morphology before implementation.
    detector:
      kind: manual
      note: Model-only fixture.
  - id: ru.fixture.related
    title: Related fixture rule
    domain: grammar
    level: advanced
    status: research
    severity: warning
    source_refs: [fixture_source]
    requires: [syntax, morphology]
    related_rules: [ru.fixture.base]
    supersedes: [ru.fixture.base]
    constraints:
      - kind: government
        description: The governor constrains dependent case.
    detector:
      kind: manual
      note: Model-only fixture.
"#,
    );

    let corpus = Corpus::load_dir(&dir).expect("extended metadata fixture loads");
    assert_eq!(corpus.rules.len(), 2);
}

#[test]
fn rejects_duplicate_related_rules() {
    let dir = temp_rules_dir("duplicate-related-rules");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.base
    title: Base fixture rule
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    detector:
      kind: manual
      note: Surface-only fixture.
  - id: ru.fixture.duplicate_related
    title: Duplicate related fixture
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    related_rules: [ru.fixture.base, ru.fixture.base]
    detector:
      kind: manual
      note: Surface-only fixture.
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("duplicate related_rules"), "unexpected error: {err}");
}

#[test]
fn rejects_unknown_related_rules() {
    let dir = temp_rules_dir("unknown-related-rules");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.unknown_related
    title: Unknown related fixture
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    related_rules: [ru.fixture.missing]
    detector:
      kind: manual
      note: Surface-only fixture.
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("unknown related rule"), "unexpected error: {err}");
}

#[test]
fn rejects_unknown_evidence_source_refs() {
    let dir = temp_rules_dir("unknown-evidence-source-ref");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.unknown_evidence_source
    title: Unknown evidence source fixture
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    evidence:
      - kind: normative_source
        source_ref: missing_source
    detector:
      kind: manual
      note: Surface-only fixture.
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("evidence references unknown source"), "unexpected error: {err}");
}

#[test]
fn rejects_invalid_confidence_values() {
    let dir = temp_rules_dir("invalid-confidence-value");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.invalid_confidence
    title: Invalid confidence fixture
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    confidence: maybe
    detector:
      kind: manual
      note: Surface-only fixture.
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("invalid YAML schema"), "unexpected error: {err}");
}
