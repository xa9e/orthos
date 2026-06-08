fn check_options(selection: RuleSelectionArgs, suppressions: SuppressionArgs) -> CheckOptions {
    CheckOptions {
        rule_filter: orthos::RuleFilter {
            profile: selection.profile.into(),
            domains: to_set(selection.domains),
            severities: to_set(selection.severities),
            include_rule_ids: normalized_string_set(selection.include_rule_ids),
            exclude_rule_ids: normalized_string_set(selection.exclude_rule_ids),
            statuses: to_set(selection.statuses),
        },
        suppressions: SuppressionOptions {
            inline_enabled: suppressions.allow_inline_suppressions,
            file_rule_ids: normalized_string_set(suppressions.file_rule_ids),
        },
        collect_timings: false,
        execution_strategy: ExecutionStrategy::default(),
        debug: DebugOptions::default(),
    }
}

fn join_capabilities(capabilities: &[orthos::Capability]) -> String {
    if capabilities.is_empty() {
        return "-".to_owned();
    }
    capabilities
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn profile_visibility(rule: &Rule) -> &'static str {
    if rule.is_default_safe() {
        "default-safe"
    } else if rule.status == orthos::RuleStatus::Implemented {
        "strict"
    } else {
        "non-executable"
    }
}

fn format_skipped_reason(reason: &SkippedRuleReason) -> String {
    match reason {
        SkippedRuleReason::UnknownDetectorKind(kind) => format!("unknown detector kind `{kind}`"),
        SkippedRuleReason::MissingCapabilities(capabilities) => {
            format!("missing capabilities: {}", join_capabilities(capabilities))
        }
    }
}

fn to_set<T: Ord>(values: Vec<T>) -> BTreeSet<T> {
    values.into_iter().collect()
}

fn normalized_string_set(values: Vec<String>) -> BTreeSet<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

fn parse_domain(value: &str) -> std::result::Result<Domain, String> {
    value.parse()
}

fn parse_severity(value: &str) -> std::result::Result<Severity, String> {
    value.parse()
}

fn parse_status_filter(value: &str) -> std::result::Result<StatusFilter, String> {
    match value.trim().replace('_', "-").as_str() {
        "default-safe" => Ok(StatusFilter::DefaultSafe),
        "implemented" => Ok(StatusFilter::Implemented),
        "planned" => Ok(StatusFilter::Planned),
        "research" => Ok(StatusFilter::Research),
        other => Err(format!(
            "unknown status filter `{other}`; expected default-safe, implemented, planned, or research"
        )),
    }
}
