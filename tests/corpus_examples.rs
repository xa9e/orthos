use orthos::{CheckOptions, Checker, Corpus, Profile};
use std::path::PathBuf;

#[test]
fn implemented_rule_examples_are_executable_specs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = Corpus::load_dir(root.join("rules")).expect("corpus loads");
    let checker = Checker::new(corpus);
    let mut options = CheckOptions::default();
    options.rule_filter.profile = Profile::Strict;

    let mut failures = Vec::new();
    for rule in checker.selected_rules(&options) {
        for text in &rule.examples.invalid {
            let issues = checker.check_with_options(text, &options).unwrap().issues;
            if !issues.iter().any(|issue| issue.rule_id == rule.id) {
                failures.push(format!(
                    "{} did not detect invalid example: {}",
                    rule.id, text
                ));
            }
        }
        for text in &rule.examples.valid {
            let issues = checker.check_with_options(text, &options).unwrap().issues;
            if issues.iter().any(|issue| issue.rule_id == rule.id) {
                failures.push(format!("{} fired on valid example: {}", rule.id, text));
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
