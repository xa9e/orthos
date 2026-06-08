fn plan_cmd(rules: PathBuf, format: Format, selection: RuleSelectionArgs) -> Result<()> {
    let checker = Checker::new(Corpus::load_dir(&rules)?);
    let options = check_options(selection, SuppressionArgs::default());
    let summary = checker.execution_plan_summary(&options);

    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(&summary)?),
        Format::Human => print_plan_summary(&summary),
    }

    Ok(())
}

fn list_rules_cmd(rules: PathBuf, all: bool, selection: RuleSelectionArgs) -> Result<()> {
    let corpus = Corpus::load_dir(&rules)?;
    let options = check_options(selection, SuppressionArgs::default());
    let mut selected: Vec<&Rule> = corpus
        .rules
        .iter()
        .filter(|rule| list_rule_matches(rule, all, &options.rule_filter))
        .collect();
    selected.sort_by(|left, right| left.id.cmp(&right.id));

    for rule in selected {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            rule.id,
            rule.domain,
            rule.severity,
            rule.status,
            rule.detector.kind(),
            join_capabilities(&rule.requires),
            profile_visibility(rule),
            rule.title
        );
    }
    Ok(())
}


fn list_rule_matches(rule: &Rule, all: bool, filter: &orthos::RuleFilter) -> bool {
    if !all {
        return filter.matches(rule);
    }

    if filter.exclude_rule_ids.contains(&rule.id) {
        return false;
    }
    if !filter.include_rule_ids.is_empty() && !filter.include_rule_ids.contains(&rule.id) {
        return false;
    }
    if !filter.domains.is_empty() && !filter.domains.contains(&rule.domain) {
        return false;
    }
    if !filter.severities.is_empty() && !filter.severities.contains(&rule.severity) {
        return false;
    }
    if !filter.statuses.is_empty() && !filter.statuses.iter().any(|status| status.matches(rule)) {
        return false;
    }

    true
}

fn validate_cmd(rules: PathBuf) -> Result<()> {
    let corpus = Corpus::load_dir(&rules)?;
    let checker = Checker::new(corpus.clone());
    checker.detector_registry().validate()?;
    println!("Corpus OK: {} rules, {} sources", corpus.rules.len(), corpus.sources.len());
    let mut detector_counts: Vec<_> = corpus.rules_by_detector().into_iter().collect();
    detector_counts.sort_by(|left, right| left.0.cmp(right.0));
    for (kind, count) in detector_counts {
        println!("  {kind}: {count}");
    }
    Ok(())
}

fn test_examples_cmd(rules: PathBuf, morph_lexicon: Option<PathBuf>, selection: RuleSelectionArgs) -> Result<()> {
    let corpus = Corpus::load_dir(&rules)?;
    let selection_checker = Checker::new(corpus.clone());
    let mut options = check_options(selection, SuppressionArgs::default());
    if is_unconstrained_default_profile(&options) {
        options.rule_filter.profile = Profile::Strict;
    }
    let selected_rules = selection_checker.selected_rules(&options).cloned().collect::<Vec<_>>();
    let example_text = selected_rules
        .iter()
        .flat_map(|rule| rule.examples.invalid.iter().chain(rule.examples.valid.iter()))
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");
    let requested_forms = morph_forms_for_text(&example_text);
    let checker = if morph_lexicon.is_some() {
        checker_from_corpus_and_morph(corpus, morph_lexicon, Some(&requested_forms))?
    } else {
        Checker::new(corpus)
    };
    let mut failures = Vec::new();

    for rule in selected_rules {
        let rule_options = options_for_single_example_rule(&options, &rule.id);
        for text in &rule.examples.invalid {
            let result = checker.check_with_options(text, &rule_options)?;
            if !result.issues.iter().any(|issue| issue.rule_id == rule.id) {
                failures.push(format!("{}: invalid example was not detected: {}", rule.id, text));
            }
        }
        for text in &rule.examples.valid {
            let result = checker.check_with_options(text, &rule_options)?;
            if result.issues.iter().any(|issue| issue.rule_id == rule.id) {
                failures.push(format!("{}: valid example triggered the rule: {}", rule.id, text));
            }
        }
    }

    if failures.is_empty() {
        println!("All selected rule examples passed.");
        return Ok(());
    }

    for failure in &failures {
        eprintln!("FAIL: {failure}");
    }
    anyhow::bail!("{} embedded example tests failed", failures.len())
}


fn options_for_single_example_rule(options: &CheckOptions, rule_id: &str) -> CheckOptions {
    let mut rule_options = options.clone();
    rule_options.rule_filter.include_rule_ids = BTreeSet::from([rule_id.to_owned()]);
    rule_options
}


fn is_unconstrained_default_profile(options: &CheckOptions) -> bool {
    options.rule_filter.profile == Profile::Default
        && options.rule_filter.domains.is_empty()
        && options.rule_filter.severities.is_empty()
        && options.rule_filter.include_rule_ids.is_empty()
        && options.rule_filter.exclude_rule_ids.is_empty()
        && options.rule_filter.statuses.is_empty()
}


#[cfg(test)]
mod test_examples_command_tests {
    use super::*;

    #[test]
    fn single_rule_example_options_preserve_filters_and_target_one_rule() {
        let mut options = CheckOptions::default();
        options.rule_filter.profile = Profile::Strict;
        options.rule_filter.domains.insert(Domain::Grammar);
        options.rule_filter.include_rule_ids.insert("old.rule".to_owned());
        options.rule_filter.exclude_rule_ids.insert("excluded.rule".to_owned());

        let rule_options = options_for_single_example_rule(&options, "target.rule");

        assert_eq!(
            rule_options.rule_filter.include_rule_ids,
            BTreeSet::from(["target.rule".to_owned()])
        );
        assert_eq!(rule_options.rule_filter.profile, Profile::Strict);
        assert_eq!(rule_options.rule_filter.domains, BTreeSet::from([Domain::Grammar]));
        assert_eq!(
            rule_options.rule_filter.exclude_rule_ids,
            BTreeSet::from(["excluded.rule".to_owned()])
        );
    }
}
