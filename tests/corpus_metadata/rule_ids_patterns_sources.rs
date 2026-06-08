#[test]
fn rejects_invalid_rule_id_naming_convention() {
    let dir = temp_rules_dir("invalid-rule-id-shape");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.BadSlug
    title: Invalid id fixture
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    detector:
      kind: manual
      note: Model-only fixture.
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("components must contain only ASCII lowercase"), "unexpected error: {err}");
}

#[test]
fn rejects_pattern_kind_without_required_capability() {
    let cases = [
        ("morphological", "morphology"),
        ("syntactic", "syntax"),
        ("dependency", "syntax"),
        ("word_formation", "word_formation"),
    ];

    for (pattern_kind, missing_capability) in cases {
        let dir = temp_rules_dir(&format!("missing-{missing_capability}-{pattern_kind}"));
        write_rules(
            &dir,
            &format!(
                r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
rules:
  - id: ru.fixture.{pattern_kind}_capability
    title: Pattern capability fixture
    domain: grammar
    level: advanced
    status: research
    severity: warning
    source_refs: [fixture_source]
    requires: [tokenization]
    pattern:
      kind: {pattern_kind}
      description: Capability implication fixture.
    detector:
      kind: manual
      note: Model-only fixture.
"#
            ),
        );

        let err = load_error(&dir);
        assert!(
            err.contains(&format!("missing requires: {missing_capability}")),
            "unexpected error for {pattern_kind}: {err}"
        );
    }
}

#[test]
fn accepts_source_metadata_fields() {
    let dir = temp_rules_dir("source-metadata-fields");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
    source_type: normative_reference
    url: https://example.invalid/source
    bibliographic_pointer: Fixture book, section 1
    license_note: Citation pointer only; no source text copied.
    year: "2026"
    authority: normative
    note: Fixture note.
rules:
  - id: ru.fixture.source_metadata
    title: Source metadata fixture
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    detector:
      kind: manual
      note: Model-only fixture.
"#,
    );

    let corpus = Corpus::load_dir(&dir).expect("source metadata fixture loads");
    assert_eq!(corpus.sources.len(), 1);
}

#[test]
fn rejects_empty_source_metadata_fields() {
    let dir = temp_rules_dir("empty-source-metadata-field");
    write_rules(
        &dir,
        r#"
version: 1
source_refs:
  - id: fixture_source
    title: Fixture source
    year: ""
rules:
  - id: ru.fixture.empty_source_metadata
    title: Empty source metadata fixture
    domain: style
    level: basic
    status: planned
    severity: info
    source_refs: [fixture_source]
    requires: [tokenization]
    detector:
      kind: manual
      note: Model-only fixture.
"#,
    );

    let err = load_error(&dir);
    assert!(err.contains("source.year must not be empty"), "unexpected error: {err}");
}
