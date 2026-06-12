//! Dataset-backed regression contract.
//!
//! Every record in `testdata/fixtures/eval/gec_dataset_regressions.jsonl` is a
//! curated example from a real GEC dataset and carries explicit expectations:
//!
//! - the corrected target must stay silent for the guarded model-backed rules
//!   (precision side);
//! - the erroneous input must trigger the listed rules, when the engine claims
//!   to detect that error class (recall side).
//!
//! Token morphology for these records lives in
//! `testdata/fixtures/eval/dataset_regressions_morph.tsv` so that curating a
//! record and curating its morphology stay one reviewable unit.

use orthos::{CheckOptions, Checker, Corpus, MorphLexicon, Profile};
use pretty_assertions::assert_eq;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Rules guarded against false positives on every corrected target unless a
/// record overrides the set via `expectations.correction_silent_rules`.
const DEFAULT_GUARDED_RULES: &[&str] = &[
    "ru.grammar.adj_noun_agreement_demo",
    "ru.grammar.nominal_group_modifier_agreement_basic",
    "ru.grammar.subject_predicate_agreement_basic",
    "ru.grammar.preposition_nominal_group_government_basic",
];

#[derive(Debug, Deserialize)]
struct DatasetRegression {
    id: String,
    input: String,
    correction: String,
    #[serde(default)]
    expectations: Expectations,
}

#[derive(Debug, Default, Deserialize)]
struct Expectations {
    /// Overrides the default guarded rule set for the corrected target.
    #[serde(default)]
    correction_silent_rules: Option<Vec<String>>,
    /// Rules that must fire on the erroneous input (recall claims).
    #[serde(default)]
    input_must_trigger: Vec<String>,
    /// Rules that must stay silent even on the erroneous input.
    #[serde(default)]
    input_must_not_trigger: Vec<String>,
}

fn checker() -> Checker {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = Corpus::load_dir(root.join("rules")).expect("corpus loads");
    let morphology = include_str!("../testdata/fixtures/eval/dataset_regressions_morph.tsv");
    Checker::with_morph_lexicon(corpus, MorphLexicon::parse_tsv(morphology))
}

fn strict_options() -> CheckOptions {
    let mut options = CheckOptions::default();
    options.rule_filter.profile = Profile::Strict;
    options
}

fn load_regressions() -> Vec<DatasetRegression> {
    include_str!("../testdata/fixtures/eval/gec_dataset_regressions.jsonl")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("dataset regression fixture record parses"))
        .collect()
}

fn rule_ids_for(checker: &Checker, text: &str) -> BTreeSet<String> {
    checker
        .check_with_options(text, &strict_options())
        .unwrap()
        .issues
        .into_iter()
        .map(|issue| issue.rule_id)
        .collect()
}

#[test]
fn corrected_dataset_targets_stay_silent_for_guarded_rules() {
    let checker = checker();

    for record in load_regressions() {
        let fired = rule_ids_for(&checker, &record.correction);
        let guarded: Vec<String> = match &record.expectations.correction_silent_rules {
            Some(rules) => rules.clone(),
            None => DEFAULT_GUARDED_RULES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        };
        let violations = guarded
            .iter()
            .filter(|rule| fired.contains(*rule))
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            violations,
            Vec::<String>::new(),
            "{}: corrected target must stay silent",
            record.id
        );
    }
}

#[test]
fn erroneous_dataset_inputs_trigger_claimed_rules() {
    let checker = checker();

    for record in load_regressions() {
        if record.expectations.input_must_trigger.is_empty()
            && record.expectations.input_must_not_trigger.is_empty()
        {
            continue;
        }
        let fired = rule_ids_for(&checker, &record.input);
        let missing = record
            .expectations
            .input_must_trigger
            .iter()
            .filter(|rule| !fired.contains(*rule))
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            missing,
            Vec::<String>::new(),
            "{}: claimed rules must fire on erroneous input (fired: {:?})",
            record.id,
            fired
        );
        let unexpected = record
            .expectations
            .input_must_not_trigger
            .iter()
            .filter(|rule| fired.contains(*rule))
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            unexpected,
            Vec::<String>::new(),
            "{}: rules listed as silent fired on erroneous input",
            record.id
        );
    }
}

#[test]
fn governed_nominal_group_regression_still_catches_bad_head_case() {
    let issues = checker()
        .check_with_options(
            "Согласно новому приказа, встреча перенесена.",
            &strict_options(),
        )
        .unwrap()
        .issues;

    assert!(
        issues.iter().any(|issue| {
            issue.rule_id == "ru.grammar.preposition_nominal_group_government_basic"
        })
    );
}
