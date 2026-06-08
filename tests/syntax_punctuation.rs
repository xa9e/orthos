use orthos::{CheckOptions, Checker, Corpus, Profile};
use std::path::PathBuf;

fn checker() -> Checker {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = Corpus::load_dir(root.join("rules")).expect("corpus loads");
    Checker::new(corpus)
}

fn strict_options() -> CheckOptions {
    let mut options = CheckOptions::default();
    options.rule_filter.profile = Profile::Strict;
    options
}

fn comma_rule_fires(text: &str) -> bool {
    checker()
        .check_with_options(text, &strict_options())
        .expect("checker succeeds")
        .issues
        .iter()
        .any(|issue| issue.rule_id == "ru.punctuation.comma_before_subordinator_basic")
}

#[test]
fn does_not_flag_subordinate_marker_at_sentence_start() {
    assert!(!comma_rule_fires("Если он придёт, мы уйдём."));
    assert!(!comma_rule_fires("Когда он пришёл, все замолчали."));
}

#[test]
fn does_not_flag_marker_inside_parentheses() {
    assert!(!comma_rule_fires("Пометка (если возможно) останется."));
    assert!(!comma_rule_fires(
        "Он оставил комментарий [что странно] в черновике."
    ));
}

#[test]
fn does_not_flag_marker_inside_direct_speech_or_quoted_question() {
    assert!(!comma_rule_fires("Он спросил: «что делать дальше?»"));
    assert!(!comma_rule_fires("Она сказала: «если сможешь — приходи»."));
}

#[test]
fn detects_multiword_subordinator_as_single_boundary() {
    let result = checker()
        .check_with_options("Он ушёл потому что устал.", &strict_options())
        .expect("checker succeeds");
    let issue = result
        .issues
        .iter()
        .find(|issue| issue.rule_id == "ru.punctuation.comma_before_subordinator_basic")
        .expect("comma-before-subordinator issue");
    assert_eq!(issue.replacement.as_deref(), Some(", потому"));
}

#[test]
fn quote_boundaries_suppress_marker_scan_until_after_quote() {
    assert!(!comma_rule_fires("Он сказал: «что делать?» — и замолчал."));
    assert!(comma_rule_fires("Он замолчал потому что устал."));
}

#[test]
fn syntax_document_segments_abbreviations_ellipses_and_direct_speech() {
    let text = "В г. Алматы тихо... Он спросил: «Что делать?» Потом ушёл.";
    let spans = orthos::syntax::sentence_spans(text);
    let rendered = spans
        .into_iter()
        .map(|span| text[span.start..span.end].to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        rendered,
        vec![
            "В г. Алматы тихо...".to_string(),
            "Он спросил: «Что делать?»".to_string(),
            "Потом ушёл.".to_string()
        ]
    );
}

#[test]
fn nested_parentheses_and_quotes_are_safe_zones_for_punctuation_rules() {
    let text = "Он оставил заметку (пример: «если получится?» внутри) и ушёл потому что устал.";
    let document = orthos::syntax::SyntaxDocument::new(text);
    assert!(document.is_inside_punctuation_safe_zone(text.find("если").unwrap()));
    assert_eq!(document.sentences().len(), 1);
    assert!(comma_rule_fires(text));
}

#[test]
fn sentence_initial_multiword_marker_does_not_emit_missing_comma() {
    assert!(!comma_rule_fires("Потому что было поздно, он ушёл."));
}

#[test]
fn introductory_phrase_detector_ignores_quoted_sentence_start() {
    let result = checker()
        .check_with_options(
            "Он сказал: «Конечно это риск». Конечно это риск.",
            &strict_options(),
        )
        .expect("checker succeeds");
    let introductory = result
        .issues
        .iter()
        .filter(|issue| issue.rule_id == "ru.punctuation.introductory_phrase_comma_basic")
        .collect::<Vec<_>>();
    assert_eq!(introductory.len(), 1);
    assert_eq!(introductory[0].excerpt, "Конечно это риск.");
}

#[test]
fn introductory_phrase_detector_uses_punctuation_slot_proof() {
    let result = checker()
        .check_with_options("Конечно это риск.", &strict_options())
        .expect("checker succeeds");
    let issue = result
        .issues
        .iter()
        .find(|issue| issue.rule_id == "ru.punctuation.introductory_phrase_comma_basic")
        .expect("introductory comma issue");

    assert_eq!(issue.replacement.as_deref(), Some(", "));
    let proof = issue.proof.as_ref().expect("punctuation slot proof");
    assert!(proof.is_actionable());
    assert!(
        proof
            .facts
            .iter()
            .any(|fact| fact.key == "slot_explanation" && fact.value.contains("introductory"))
    );
}
