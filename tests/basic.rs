use orthos::{CheckOptions, Checker, Corpus, Profile};
use pretty_assertions::assert_eq;
use std::path::PathBuf;

#[path = "basic/grammar_models.rs"]
mod grammar_models;

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

#[test]
fn corpus_loads() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = Corpus::load_dir(root.join("rules")).expect("corpus loads");
    assert!(corpus.rules.len() >= 45);
    assert!(corpus.implemented_rules().count() >= 25);
}

#[test]
fn detects_space_before_punctuation() {
    let issues = checker().check("Привет , мир !").unwrap();
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.punctuation.no_space_before_mark")
    );
}

#[test]
fn detects_missing_space_after_punctuation() {
    let issues = checker().check("Привет,мир.").unwrap();
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.punctuation.missing_space_after_mark")
    );
}

#[test]
fn detects_repeated_word() {
    let issues = checker().check("Это это ошибка.").unwrap();
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.repeated_word")
    );
}

#[test]
fn detects_missing_hyphen_particle() {
    let issues = checker().check("Кто то пришёл.").unwrap();
    let issue = issues
        .iter()
        .find(|i| i.rule_id == "ru.orthography.particle_to_libo_nibud_hyphen")
        .expect("missing particle issue");
    assert_eq!(issue.replacement.as_deref(), Some("Кто-то"));
}

#[test]
fn detects_pol_compound_missing_hyphen() {
    let issues = checker()
        .check("Пол лимона осталось. Пол чайной ложки хватит.")
        .unwrap();
    let issue = issues
        .iter()
        .find(|i| i.rule_id == "ru.orthography.pol_hyphen_missing_basic")
        .expect("missing pol-compound hyphen issue");
    assert_eq!(issue.replacement.as_deref(), Some("Пол-лимона"));
    assert_eq!(
        issues
            .iter()
            .filter(|i| i.rule_id == "ru.orthography.pol_hyphen_missing_basic")
            .count(),
        1
    );
}

#[test]
fn detects_heuristic_missing_comma_before_subordinator() {
    let issues = checker()
        .check_with_options("Я знаю что он придёт.", &strict_options())
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.punctuation.comma_before_subordinator_basic")
    );
}

#[test]
fn detects_donor_orthography_rules() {
    let issues = checker()
        .check("Мaма сказала: я незнаю, но надо учится.")
        .unwrap();
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.orthography.mixed_alphabet_word")
    );
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.orthography.ne_common_confusables")
    );
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.orthography.tsya_tsya_context_heuristic")
    );
}

#[test]
fn detects_demo_morph_agreement() {
    let issues = checker()
        .check_with_options("красивый машина", &strict_options())
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.adj_noun_agreement_demo")
    );
}

#[test]
fn detects_phrase_seed_rules() {
    let issues = checker()
        .check_with_options(
            "Открылась свободная вакансия. Согласно приказа, оба стороны согласились.",
            &strict_options(),
        )
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.style.pleonasm_phrase_seed")
    );
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.preposition_case_phrase_seed")
    );
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.numeral_phrase_seed")
    );
}

#[test]
fn detects_prefix_final_z_s_assimilation() {
    let issues = checker()
        .check("Он хочет разсказать и потом расбить лёд.")
        .unwrap();
    let replacements: Vec<_> = issues
        .iter()
        .filter(|i| i.rule_id == "ru.orthography.prefix_final_s_z")
        .map(|i| i.replacement.as_deref())
        .collect();

    assert!(replacements.contains(&Some("рассказать")));
    assert!(replacements.contains(&Some("разбить")));
}

#[test]
fn detects_basic_preposition_government() {
    let issues = checker()
        .check_with_options(
            "Согласно приказа, встреча перенесена. К дому пришёл курьер.",
            &strict_options(),
        )
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.preposition_government_basic")
    );
}

#[test]
fn detects_basic_subject_predicate_agreement() {
    let issues = checker()
        .check_with_options("Девочка пришёл. Дети пришли.", &strict_options())
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.subject_predicate_agreement_basic")
    );
}
