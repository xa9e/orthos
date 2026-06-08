use orthos::{CheckOptions, Checker, Corpus, DebugOptions, MorphLexicon, Profile};
use pretty_assertions::assert_eq;
use std::path::PathBuf;

fn checker_with_morph(tsv: &str) -> Checker {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = Corpus::load_dir(root.join("rules")).expect("corpus loads");
    Checker::with_morph_lexicon(corpus, MorphLexicon::parse_tsv(tsv))
}

fn strict_options() -> CheckOptions {
    let mut options = CheckOptions::default();
    options.rule_filter.profile = Profile::Strict;
    options
}

#[test]
fn debug_report_is_absent_by_default() {
    let result = checker_with_morph("проекту\tпроект\tNOUN\tgender=masc|number=sing|case=dat\n")
        .check_with_options("проекту", &strict_options())
        .unwrap();

    assert!(result.debug.is_none());
}

#[test]
fn debug_report_exposes_language_model_inventory_and_fact_store_frames() {
    let morph = "говорит\tговорить\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite|aspect=impf\n\
                 задачу\tзадача\tNOUN\tgender=fem|number=sing|case=acc|animacy=inan\n";
    let mut options = strict_options();
    options.debug = DebugOptions::enabled();
    let result = checker_with_morph(morph)
        .check_with_options("Он говорит о задачу.", &options)
        .unwrap();
    let debug = result.debug.expect("debug report");

    assert_eq!(debug.schema_version, 4);
    assert!(debug.analysis.summary_after.fact_store_cached);
    let verb_government_inventory = &debug
        .language_model
        .as_ref()
        .expect("language model snapshot")
        .verb_government;
    assert_eq!(
        verb_government_inventory.entry_count,
        verb_government_inventory.fixture_count
    );
    assert!(verb_government_inventory.false_positive_fixture_count > 0);
    assert!(verb_government_inventory.entries_without_fixture.is_empty());
    assert!(
        verb_government_inventory
            .entries
            .iter()
            .any(|entry| entry.lemma == "говорить" && entry.preposition.as_deref() == Some("о"))
    );

    let fact_store = debug.analysis.fact_store.expect("fact store snapshot");
    let frame = fact_store
        .government_frames
        .iter()
        .find(|frame| {
            frame.source == "VerbPrepositionalValencySeed"
                && frame.governor == "говорит"
                && frame.dependent == "задачу"
        })
        .expect("prepositional verb-government debug frame");
    assert_eq!(frame.confidence, "Strong");
    assert!(frame.blockers.is_empty());
    assert!(frame.conflict);
    let model_ref = frame.model_ref.as_ref().expect("debug model reference");
    assert_eq!(model_ref.lemma, "говорить");
    assert_eq!(
        model_ref.complement_kind,
        orthos::VerbGovernmentComplementKind::PrepositionalObject
    );
    assert_eq!(model_ref.preposition.as_deref(), Some("о"));
    assert_eq!(
        model_ref.source_id.as_ref().map(|source| source.as_str()),
        Some("project.verb_government_seed")
    );
}

#[test]
fn debug_report_records_raw_suppressed_and_emitted_rule_counts() {
    let morph = "говорит\tговорить\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite|aspect=impf\n\
                 задачу\tзадача\tNOUN\tgender=fem|number=sing|case=acc|animacy=inan\n";
    let mut options = strict_options();
    options.debug = DebugOptions::enabled();
    options
        .suppressions
        .file_rule_ids
        .insert("ru.grammar.verb_government_basic".to_owned());
    options
        .suppressions
        .file_rule_ids
        .insert("ru.grammar.preposition_government_basic".to_owned());
    let result = checker_with_morph(morph)
        .check_with_options("Он говорит о задачу.", &options)
        .unwrap();
    let debug = result.debug.expect("debug report");

    let rule = debug
        .engine
        .rule_outputs
        .iter()
        .find(|item| item.rule_id == "ru.grammar.verb_government_basic")
        .expect("verb government rule output");
    assert_eq!(rule.raw_issue_count, 1);
    assert_eq!(rule.suppressed_issue_count, 1);
    assert_eq!(rule.emitted_issue_count, 0);
    let preposition_rule = debug
        .engine
        .rule_outputs
        .iter()
        .find(|item| item.rule_id == "ru.grammar.preposition_government_basic")
        .expect("preposition government rule output");
    assert_eq!(preposition_rule.raw_issue_count, 1);
    assert_eq!(preposition_rule.suppressed_issue_count, 1);
    assert_eq!(preposition_rule.emitted_issue_count, 0);
    assert!(result.issues.is_empty());
}

#[test]
fn debug_report_exposes_clause_boundary_blocked_government_frames() {
    let morph = "проверять\tпроверять\tVERB\tverb_form=infinitive|aspect=impf\n\
                 что\tчто\tCONJ\tindecl\n\
                 проекту\tпроект\tNOUN\tgender=masc|number=sing|case=dat|animacy=inan\n";
    let mut options = strict_options();
    options.debug = DebugOptions::enabled();
    let result = checker_with_morph(morph)
        .check_with_options("проверять что проекту", &options)
        .unwrap();
    assert!(
        result
            .issues
            .iter()
            .all(|issue| issue.rule_id != "ru.grammar.verb_government_basic")
    );

    let debug = result.debug.expect("debug report");
    let fact_store = debug.analysis.fact_store.expect("fact store snapshot");
    assert!(
        fact_store
            .clause_boundaries
            .iter()
            .any(|boundary| { boundary.marker == "что" && boundary.kind == "BeforeMarker" })
    );
    let frame = fact_store
        .government_frames
        .iter()
        .find(|frame| frame.governor == "проверять" && frame.dependent == "проекту")
        .expect("blocked verb-government frame");
    assert!(
        frame
            .blockers
            .iter()
            .any(|blocker| blocker == "ClauseBoundary")
    );
    assert!(!frame.conflict);
}
