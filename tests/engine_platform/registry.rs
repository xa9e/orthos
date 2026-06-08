#[test]
fn detector_registry_has_stable_sorted_metadata() {
    let registry = default_detector_registry();
    let kinds = registry.kinds();
    let mut sorted = kinds.clone();
    sorted.sort_unstable();

    assert_eq!(kinds, sorted);
    assert!(kinds.contains(&"regex"));
    assert!(kinds.contains(&"phrase_map"));
    assert!(kinds.contains(&"manual"));
    assert!(registry
        .metadata("missing_comma_before_subordinator")
        .expect("registered detector")
        .capabilities
        .contains(&Capability::Syntax));
}

#[test]
fn detector_registry_contracts_validate() {
    let registry = default_detector_registry();
    registry
        .validate()
        .expect("default registry metadata is complete");

    for metadata in registry.all_metadata() {
        assert!(!metadata.kind.trim().is_empty());
        assert!(!metadata.description.trim().is_empty());
        assert!(metadata.deterministic);
    }
}

#[test]
fn empty_registry_reports_unknown_detector_in_plan() {
    let lint = Checker::with_detector_registry(corpus(), DetectorRegistry::new());
    let mut options = CheckOptions::default();
    options
        .rule_filter
        .include_rule_ids
        .insert("ru.punctuation.no_space_before_mark".to_owned());

    let plan = lint.execution_plan(&options);
    assert!(plan.rules.is_empty());
    assert_eq!(plan.skipped_rules.len(), 1);
    assert_eq!(
        plan.skipped_rules[0].reason,
        SkippedRuleReason::UnknownDetectorKind("no_whitespace_before_punctuation".to_owned())
    );
}
