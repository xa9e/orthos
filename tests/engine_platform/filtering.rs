#[test]
fn default_and_strict_profiles_have_different_execution_sets() {
    let default_checker = checker();
    let default_options = CheckOptions::default();
    let strict_options = strict_options();

    let default_rules: Vec<_> = default_checker
        .selected_rules(&default_options)
        .map(|rule| rule.id.as_str())
        .collect();
    let strict_rules: Vec<_> = default_checker
        .selected_rules(&strict_options)
        .map(|rule| rule.id.as_str())
        .collect();

    assert!(!default_rules.contains(&"ru.grammar.adj_noun_agreement_demo"));
    assert!(strict_rules.contains(&"ru.grammar.adj_noun_agreement_demo"));
    assert!(strict_rules.len() > default_rules.len());
}

#[test]
fn include_rule_id_and_exclude_rule_are_exact_and_exclude_wins() {
    let mut options = CheckOptions::default();
    options
        .rule_filter
        .include_rule_ids
        .insert("ru.punctuation.no_space_before_mark".to_owned());

    let result = checker()
        .check_with_options("Привет , мир. Это это ошибка.", &options)
        .expect("check succeeds");
    assert_eq!(result.issues.len(), 1);
    assert_eq!(
        result.issues[0].rule_id,
        "ru.punctuation.no_space_before_mark"
    );

    options
        .rule_filter
        .exclude_rule_ids
        .insert("ru.punctuation.no_space_before_mark".to_owned());
    let result = checker()
        .check_with_options("Привет , мир.", &options)
        .expect("check succeeds");
    assert!(result.issues.is_empty());
}

#[test]
fn domain_severity_and_status_filters_gate_execution() {
    let lint = checker();

    let mut domain_options = strict_options();
    domain_options
        .rule_filter
        .domains
        .insert(Domain::Typography);
    let domain_rules: Vec<_> = lint.selected_rules(&domain_options).collect();
    assert!(!domain_rules.is_empty());
    assert!(domain_rules
        .iter()
        .all(|rule| rule.domain == Domain::Typography));

    let mut severity_options = strict_options();
    severity_options
        .rule_filter
        .severities
        .insert(Severity::Error);
    assert!(lint
        .selected_rules(&severity_options)
        .all(|rule| rule.severity == Severity::Error));

    let mut status_options = CheckOptions::default();
    status_options
        .rule_filter
        .statuses
        .insert(StatusFilter::Planned);
    let status_rules: Vec<_> = lint.selected_rules(&status_options).collect();
    assert!(!status_rules.is_empty());
    assert!(status_rules
        .iter()
        .all(|rule| rule.status == orthos::RuleStatus::Planned));
}

#[test]
fn capability_registry_skips_rules_missing_runtime_support() {
    let lint = Checker::with_capabilities(
        corpus(),
        CapabilityRegistry::new([Capability::Tokenization]),
    );
    let mut options = CheckOptions::default();
    options
        .rule_filter
        .include_rule_ids
        .insert("ru.punctuation.comma_before_subordinator_basic".to_owned());

    let plan = lint.execution_plan(&options);
    assert!(plan.rules.is_empty());
    assert_eq!(plan.skipped_rules.len(), 1);
    assert_eq!(
        &plan.skipped_rules[0].reason,
        &SkippedRuleReason::MissingCapabilities(vec![Capability::Syntax])
    );
}
