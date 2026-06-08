#[test]
fn inline_and_file_suppressions_are_applied_after_detection() {
    let mut inline_options = CheckOptions::default();
    inline_options.suppressions.inline_enabled = true;
    let inline_result = checker()
        .check_with_options(
            "Привет , мир. # orthos-disable-line ru.punctuation.no_space_before_mark",
            &inline_options,
        )
        .expect("check succeeds");
    assert!(inline_result.issues.is_empty());

    let file_options = CheckOptions {
        suppressions: SuppressionOptions {
            inline_enabled: false,
            file_rule_ids: ["ru.punctuation.no_space_before_mark".to_owned()]
                .into_iter()
                .collect(),
        },
        ..Default::default()
    };
    let file_result = checker()
        .check_with_options("Привет , мир.", &file_options)
        .expect("check succeeds");
    assert!(file_result.issues.is_empty());
}

#[test]
fn issue_order_is_deterministic_by_span_then_rule_id() {
    let result = checker()
        .check_with_options("Привет ,мир. Это это ошибка", &strict_options())
        .expect("check succeeds");

    let order: Vec<_> = result
        .issues
        .iter()
        .map(|issue| (issue.span.start, issue.span.end, issue.rule_id.as_str()))
        .collect();
    let mut sorted = order.clone();
    sorted.sort_unstable();

    assert_eq!(order, sorted);
}

#[test]
fn execution_plan_summary_is_serializable_and_counts_skips() {
    let lint = Checker::with_capabilities(
        corpus(),
        CapabilityRegistry::new([Capability::Tokenization]),
    );
    let mut options = CheckOptions::default();
    options
        .rule_filter
        .include_rule_ids
        .insert("ru.punctuation.comma_before_subordinator_basic".to_owned());

    let summary = lint.execution_plan_summary(&options);
    assert_eq!(summary.selected_rule_count, 0);
    assert_eq!(summary.skipped_rule_count, 1);

    let value = serde_json::to_value(&summary).expect("plan summary serializes");
    assert_eq!(value["selected_rule_count"], 0);
    assert_eq!(value["skipped_rule_count"], 1);
}

#[test]
fn deterministic_parallel_strategy_preserves_issue_order() {
    let lint = checker();
    let mut serial = strict_options();
    serial.execution_strategy = ExecutionStrategy::Serial;

    let mut parallel = strict_options();
    parallel.execution_strategy = ExecutionStrategy::DeterministicParallel;

    let text = "Привет ,мир. Это это ошибка. Открылась свободная вакансия. красивый машина";
    let serial_result = lint
        .check_with_options(text, &serial)
        .expect("serial check succeeds");
    let parallel_result = lint
        .check_with_options(text, &parallel)
        .expect("parallel check succeeds");

    assert_eq!(parallel_result.issues, serial_result.issues);
    assert_eq!(
        parallel_result.execution_plan.selected_rule_count,
        serial_result.execution_plan.selected_rule_count
    );
}

#[test]
fn next_line_and_global_inline_suppressions_work() {
    let mut options = CheckOptions::default();
    options.suppressions.inline_enabled = true;

    let next_line_result = checker()
        .check_with_options(
            "# orthos-disable-next-line
Привет , мир.",
            &options,
        )
        .expect("check succeeds");
    assert!(next_line_result.issues.is_empty());

    let file_result = checker()
        .check_with_options(
            "# orthos-disable-file
Привет , мир. Это это ошибка.",
            &options,
        )
        .expect("check succeeds");
    assert!(file_result.issues.is_empty());
}

#[test]
fn json_issue_contract_keeps_required_fields() {
    let result = checker()
        .check_with_options("Привет , мир.", &CheckOptions::default())
        .expect("check succeeds");
    let first = result.issues.first().expect("one diagnostic");
    let value = serde_json::to_value(first).expect("issue serializes");
    let Value::Object(map) = value else {
        panic!("issue JSON must be an object");
    };

    for key in [
        "rule_id",
        "severity",
        "message",
        "span",
        "start",
        "end",
        "replacement",
        "source_refs",
        "excerpt",
    ] {
        assert!(map.contains_key(key), "missing JSON key: {key}");
    }
}

#[test]
fn strict_demo_contract_keeps_representative_rule_ids_stable() {
    let result = checker()
        .check_with_options(include_str!("../../examples/bad.txt"), &strict_options())
        .expect("check succeeds");
    let rule_ids = result
        .issues
        .iter()
        .map(|issue| issue.rule_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        rule_ids,
        vec![
            "ru.orthography.sentence_initial_capital",
            "ru.punctuation.no_space_before_mark",
            "ru.punctuation.no_space_before_mark",
            "ru.punctuation.comma_before_subordinator_basic",
            "ru.orthography.particle_to_libo_nibud_hyphen",
            "ru.orthography.ne_common_confusables",
            "ru.orthography.ne_with_common_verbs",
            "ru.grammar.repeated_word",
            "ru.typography.multiple_spaces",
            "ru.punctuation.unbalanced_guillemets",
            "ru.punctuation.unpaired_delimiters",
            "ru.punctuation.sentence_terminal",
        ]
    );
}
