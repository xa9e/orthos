use super::{checker, strict_options};

#[test]
fn detects_basic_numeral_noun_agreement() {
    let issues = checker()
        .check_with_options(
            "Два дом стояли рядом. Пять домов снесли.",
            &strict_options(),
        )
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.numeral_noun_agreement")
    );
}

#[test]
fn detects_hyphenated_preposition_and_koe_seed_rules() {
    let issues = checker()
        .check_with_options("Из за дождя кое кто опоздал.", &strict_options())
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.orthography.compound_preposition_hyphen_seed")
    );
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.orthography.koe_indefinite_hyphen_seed")
    );
}

#[test]
fn detects_model_backed_clitic_and_negation_rules() {
    let issues = checker()
        .check_with_options(
            "Я то понял. Ну ка проверь. Он нехочет спорить.",
            &strict_options(),
        )
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.orthography.emphatic_to_particle_hyphen_seed")
    );
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.orthography.ka_particle_hyphen_seed")
    );
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.orthography.ne_with_common_verbs")
    );
}

#[test]
fn detects_short_nominal_group_government_and_quantity() {
    let issues = checker()
        .check_with_options(
            "Согласно новому приказа, встреча перенесена. Два новых дом стояли рядом.",
            &strict_options(),
        )
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.preposition_nominal_group_government_basic")
    );
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.numeral_nominal_group_agreement_basic")
    );
}

#[test]
fn detects_non_adjacent_modifier_agreement_in_nominal_group() {
    let issues = checker()
        .check_with_options("Новый важному приказу присвоили номер.", &strict_options())
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| i.rule_id == "ru.grammar.nominal_group_modifier_agreement_basic")
    );
}

#[test]
fn detects_compound_numeral_nominal_group_agreement() {
    let issues = checker()
        .check_with_options(
            "Двадцать два новых дом стояли рядом. Двадцать пять домов снесли.",
            &strict_options(),
        )
        .unwrap()
        .issues;
    assert!(
        issues
            .iter()
            .any(|i| { i.rule_id == "ru.grammar.compound_numeral_nominal_group_agreement_basic" })
    );
}

#[test]
fn detects_typed_compound_numeral_component_agreement() {
    let issues = checker()
        .check_with_options(
            "Сто двадцать два новых дом стояли рядом. Сто двадцать пять домов снесли.",
            &strict_options(),
        )
        .unwrap()
        .issues;
    assert!(issues.iter().any(|i| {
        i.rule_id == "ru.grammar.typed_compound_numeral_nominal_group_agreement_basic"
    }));
}
