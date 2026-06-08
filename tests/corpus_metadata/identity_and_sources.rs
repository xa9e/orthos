#[test]
fn rejects_duplicate_rule_ids() {
    let dir = temp_rules_dir("duplicate-rule-ids");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.same
    title: First fixture rule
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    detector:
      kind: manual
      note: Surface-only fixture.
  - id: ru.fixture.same
    title: Second fixture rule
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    detector:
      kind: manual
      note: Surface-only fixture.
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("duplicate rule id"), "unexpected error: {err}");
}

#[test]
fn rejects_unknown_source_refs() {
    let dir = temp_rules_dir("unknown-source-ref");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.unknown_source
    title: Unknown source fixture
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [missing_source]
    requires: [tokenization]
    detector:
      kind: manual
      note: Surface-only fixture.
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("references unknown source"), "unexpected error: {err}");
}

#[test]
fn rejects_implemented_manual_detectors() {
    let dir = temp_rules_dir("implemented-manual");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.implemented_manual
    title: Implemented manual fixture
    domain: style
    level: basic
    status: implemented
    severity: warning
    source_refs: [fixture_source]
    requires: [tokenization]
    detector:
      kind: manual
      note: Surface-only fixture.
    examples:
      invalid: ["bad"]
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("implemented rule cannot use manual detector"), "unexpected error: {err}");
}

#[test]
fn rejects_implemented_rules_without_invalid_examples() {
    let dir = temp_rules_dir("implemented-without-invalid");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.no_invalid_example
    title: Missing invalid example fixture
    domain: style
    level: basic
    status: implemented
    severity: warning
    source_refs: [fixture_source]
    requires: [regex]
    detector:
      kind: regex
      pattern: "bad"
      message: Bad fixture.
    examples:
      valid: ["good"]
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("implemented rule must have at least one invalid example"), "unexpected error: {err}");
}

#[test]
fn rejects_missing_complex_capability_declarations() {
    let dir = temp_rules_dir("missing-complex-capability");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.syntax_capability
    title: Синтаксическое управление
    domain: grammar
    level: advanced
    status: research
    severity: warning
    source_refs: [fixture_source]
    requires: [morphology]
    detector:
      kind: manual
      note: Нужен синтаксический анализ зависимого слова.
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("missing requires: syntax"), "unexpected error: {err}");
}
