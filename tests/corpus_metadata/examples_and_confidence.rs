#[test]
fn rejects_empty_executable_examples() {
    let dir = temp_rules_dir("empty-executable-example");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.empty_example
    title: Empty example fixture
    domain: style
    level: basic
    status: implemented
    severity: warning
    source_refs: [fixture_source]
    requires: [regex]
    detector:
      kind: regex
      pattern: bad
      message: Bad fixture.
    examples:
      invalid: [""]
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("examples.invalid contains an empty value"), "unexpected error: {err}");
}

#[test]
fn rejects_implemented_rules_without_confidence_metadata() {
    let dir = temp_rules_dir("implemented-without-confidence");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.no_confidence
    title: Missing confidence fixture
    domain: style
    level: basic
    status: implemented
    severity: warning
    source_refs: [fixture_source]
    requires: [regex]
    false_positive_risk: low
    detector:
      kind: regex
      pattern: bad
      message: Bad fixture.
    examples:
      valid: ["good"]
      invalid: ["bad"]
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("implemented rule must declare confidence"), "unexpected error: {err}");
}

#[test]
fn rejects_implemented_rules_without_false_positive_risk_metadata() {
    let dir = temp_rules_dir("implemented-without-risk");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.no_risk
    title: Missing risk fixture
    domain: style
    level: basic
    status: implemented
    severity: warning
    source_refs: [fixture_source]
    requires: [regex]
    confidence: high
    detector:
      kind: regex
      pattern: bad
      message: Bad fixture.
    examples:
      valid: ["good"]
      invalid: ["bad"]
"#,
    );

    let err = load_error(&dir);
    assert!(
        err.contains("implemented rule must declare false_positive_risk"),
        "unexpected error: {err}"
    );
}

#[test]
fn rejects_implemented_rules_without_valid_examples() {
    let dir = temp_rules_dir("implemented-without-valid");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.no_valid_example
    title: Missing valid example fixture
    domain: style
    level: basic
    status: implemented
    severity: warning
    source_refs: [fixture_source]
    requires: [regex]
    confidence: high
    false_positive_risk: low
    detector:
      kind: regex
      pattern: bad
      message: Bad fixture.
    examples:
      invalid: ["bad"]
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("implemented rule must have at least one valid example"), "unexpected error: {err}");
}

#[test]
fn rejects_duplicate_examples_across_valid_and_invalid_sets() {
    let dir = temp_rules_dir("duplicate-cross-examples");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.duplicate_cross_example
    title: Duplicate cross example fixture
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [regex]
    detector:
      kind: manual
      note: Model-only fixture.
    examples:
      valid: ["same text"]
      invalid: ["same text"]
"#,
    );

    let err = load_error(&dir);
    assert!(
        err.contains("examples.valid and examples.invalid contain duplicate example"),
        "unexpected error: {err}"
    );
}
