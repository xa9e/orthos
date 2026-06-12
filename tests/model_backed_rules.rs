use orthos::{
    CheckOptions, Checker, Corpus, DebugOptions, DiagnosticProof, DiagnosticProofKind,
    MorphLexicon, Profile, VerbGovernmentFalsePositiveFixtureSet, VerbGovernmentFixtureSet,
};
use pretty_assertions::assert_eq;
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

fn strict_debug_options() -> CheckOptions {
    let mut options = strict_options();
    options.debug = DebugOptions::enabled();
    options
}

#[test]
fn grammar_model_rules_return_machine_readable_proofs() {
    let issues = checker()
        .check_with_options("Согласно приказа. Два дом стояли рядом.", &strict_options())
        .unwrap()
        .issues;

    let government = issues
        .iter()
        .find(|i| i.rule_id == "ru.grammar.preposition_government_basic")
        .expect("preposition government issue");
    assert_eq!(
        government.proof.as_ref().expect("government proof").kind,
        DiagnosticProofKind::GovernmentConflict
    );

    let quantity = issues
        .iter()
        .find(|i| i.rule_id == "ru.grammar.numeral_noun_agreement")
        .expect("quantity government issue");
    assert_eq!(
        quantity.proof.as_ref().expect("quantity proof").kind,
        DiagnosticProofKind::QuantityConflict
    );
}

#[test]
fn preposition_government_rules_split_adjacent_and_group_frames() {
    let adjacent = checker()
        .check_with_options("Согласно приказа, встреча перенесена.", &strict_options())
        .unwrap()
        .issues;
    assert!(
        adjacent
            .iter()
            .any(|i| i.rule_id == "ru.grammar.preposition_government_basic")
    );
    assert!(
        !adjacent
            .iter()
            .any(|i| i.rule_id == "ru.grammar.preposition_nominal_group_government_basic")
    );

    let grouped = checker()
        .check_with_options(
            "Согласно новому приказа, встреча перенесена.",
            &strict_options(),
        )
        .unwrap()
        .issues;
    assert!(
        grouped
            .iter()
            .any(|i| i.rule_id == "ru.grammar.preposition_nominal_group_government_basic")
    );
}

#[test]
fn quantity_rules_use_maximal_compound_numeral_frames() {
    let issues = checker()
        .check_with_options(
            "Сто двадцать два новых дом стояли рядом.",
            &strict_options(),
        )
        .unwrap()
        .issues;

    let typed = issues
        .iter()
        .find(|i| i.rule_id == "ru.grammar.typed_compound_numeral_nominal_group_agreement_basic")
        .expect("typed quantity issue");
    assert_eq!(
        typed.replacement.as_deref(),
        Some("Сто двадцать два новых дома")
    );
    assert!(
        !issues
            .iter()
            .any(|i| { i.rule_id == "ru.grammar.compound_numeral_nominal_group_agreement_basic" })
    );
}

fn checker_with_morph(tsv: &str) -> Checker {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = Corpus::load_dir(root.join("rules")).expect("corpus loads");
    Checker::with_morph_lexicon(corpus, MorphLexicon::parse_tsv(tsv))
}

#[test]
fn verb_government_rule_uses_prepositional_valency_frames() {
    let morph = "говорит\tговорить\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite|aspect=impf\n\
                 задаче\tзадача\tNOUN\tgender=fem|number=sing|case=prep|animacy=inan\n\
                 задачу\tзадача\tNOUN\tgender=fem|number=sing|case=acc|animacy=inan\n";
    let issues = checker_with_morph(morph)
        .check_with_options(
            "Он говорит о задаче. Он говорит о задачу.",
            &strict_options(),
        )
        .unwrap()
        .issues;

    let verb_government = issues
        .iter()
        .filter(|issue| issue.rule_id == "ru.grammar.verb_government_basic")
        .collect::<Vec<_>>();
    assert_eq!(verb_government.len(), 1);
    assert!(verb_government[0].excerpt.contains("говорит о задачу"));
    assert_eq!(
        verb_government[0].replacement.as_deref(),
        Some("говорит о задаче")
    );
    let proof = verb_government[0].proof.as_ref().expect("government proof");
    assert_eq!(proof.kind, DiagnosticProofKind::GovernmentConflict);
    assert!(
        proof
            .facts
            .iter()
            .any(|fact| { fact.key == "model_lemma" && fact.value == "говорить" })
    );
    assert!(proof.facts.iter().any(|fact| {
        fact.key == "model_complement_kind" && fact.value == "PrepositionalObject"
    }));
    assert!(
        proof
            .facts
            .iter()
            .any(|fact| { fact.key == "model_preposition" && fact.value == "о" })
    );
    assert!(proof.facts.iter().any(|fact| {
        fact.key == "model_source_id" && fact.value == "project.verb_government_seed"
    }));
}

#[test]
fn verb_government_rule_handles_dative_direct_object_seed() {
    let morph = "помогает\tпомогать\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite|aspect=impf\n\
                 проекту\tпроект\tNOUN\tgender=masc|number=sing|case=dat|animacy=inan\n\
                 проектом\tпроект\tNOUN\tgender=masc|number=sing|case=ins|animacy=inan\n";
    let issues = checker_with_morph(morph)
        .check_with_options(
            "Он помогает проекту. Он помогает проектом.",
            &strict_options(),
        )
        .unwrap()
        .issues;

    let verb_government = issues
        .iter()
        .filter(|issue| issue.rule_id == "ru.grammar.verb_government_basic")
        .collect::<Vec<_>>();
    assert_eq!(verb_government.len(), 1);
    assert!(verb_government[0].excerpt.contains("помогает проектом"));
    assert_eq!(
        verb_government[0].replacement.as_deref(),
        Some("помогает проекту")
    );
}

const VERB_GOVERNMENT_FIXTURE_MORPH: &str =
    include_str!("../data/grammar/verb_government.fixture_morph.tsv");

fn assert_proof_fact(proof: &DiagnosticProof, key: &str, expected: &str) {
    assert!(
        proof
            .facts
            .iter()
            .any(|fact| fact.key == key && fact.value == expected),
        "missing proof fact {key}={expected:?}; facts: {:?}",
        proof.facts
    );
}

#[test]
fn verb_government_seed_fixtures_are_enforced_by_checker() {
    let fixtures = VerbGovernmentFixtureSet::russian_seed();
    let checker = checker_with_morph(VERB_GOVERNMENT_FIXTURE_MORPH);

    for fixture in fixtures.fixtures() {
        let valid_issues = checker
            .check_with_options(&fixture.valid_text, &strict_options())
            .unwrap()
            .issues
            .into_iter()
            .filter(|issue| issue.rule_id == "ru.grammar.verb_government_basic")
            .collect::<Vec<_>>();
        assert!(
            valid_issues.is_empty(),
            "valid verb-government fixture should not trigger {:?}: {:?}",
            fixture,
            valid_issues
        );

        let invalid_issues = checker
            .check_with_options(&fixture.invalid_text, &strict_options())
            .unwrap()
            .issues
            .into_iter()
            .filter(|issue| issue.rule_id == "ru.grammar.verb_government_basic")
            .collect::<Vec<_>>();
        assert_eq!(
            invalid_issues.len(),
            1,
            "invalid verb-government fixture should trigger once: {:?}",
            fixture
        );
        let issue = &invalid_issues[0];
        assert!(issue.excerpt.contains(&fixture.invalid_excerpt));
        let proof = issue.proof.as_ref().expect("verb-government proof");
        assert_eq!(proof.kind, DiagnosticProofKind::GovernmentConflict);
        assert_proof_fact(proof, "model_lemma", &fixture.key.lemma);
        assert_proof_fact(
            proof,
            "model_complement_kind",
            &format!("{:?}", fixture.key.complement_kind),
        );
        if let Some(preposition) = &fixture.key.preposition {
            assert_proof_fact(proof, "model_preposition", preposition);
        }
    }
}

#[test]
fn verb_government_false_positive_fixtures_do_not_emit_issues() {
    let fixtures = VerbGovernmentFalsePositiveFixtureSet::russian_seed();
    let checker = checker_with_morph(VERB_GOVERNMENT_FIXTURE_MORPH);

    for fixture in fixtures.fixtures() {
        let result = checker
            .check_with_options(&fixture.text, &strict_debug_options())
            .unwrap();
        let verb_government_issues = result
            .issues
            .iter()
            .filter(|issue| issue.rule_id == "ru.grammar.verb_government_basic")
            .collect::<Vec<_>>();
        assert!(
            verb_government_issues.is_empty(),
            "false-positive fixture must not emit verb-government issue: {:?}; got {:?}",
            fixture,
            verb_government_issues
        );

        let Some(expected_blocker) = &fixture.expected_blocker else {
            continue;
        };
        let debug = result.debug.as_ref().expect("debug report");
        let fact_store = debug
            .analysis
            .fact_store
            .as_ref()
            .expect("fact store snapshot");
        let frame = fact_store
            .government_frames
            .iter()
            .find(|frame| {
                frame.model_ref.as_ref().is_some_and(|model_ref| {
                    model_ref.lemma == fixture.key.lemma
                        && model_ref.complement_kind == fixture.key.complement_kind
                        && model_ref.preposition == fixture.key.preposition
                })
            })
            .expect("blocked model-backed frame should be visible in debug");
        assert!(
            frame
                .blockers
                .iter()
                .any(|blocker| blocker == expected_blocker),
            "fixture {:?} expected blocker {expected_blocker}, got {:?}",
            fixture,
            frame.blockers
        );
        assert!(!frame.conflict, "blocked frame must not be actionable");
    }
}
