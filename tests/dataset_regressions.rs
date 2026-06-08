use orthos::{CheckOptions, Checker, Corpus, MorphLexicon, Profile};
use pretty_assertions::assert_eq;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct DatasetRegression {
    id: String,
    correction: String,
}

fn checker() -> Checker {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus = Corpus::load_dir(root.join("rules")).expect("corpus loads");
    Checker::with_morph_lexicon(corpus, MorphLexicon::parse_tsv(fixture_morphology()))
}

fn strict_options() -> CheckOptions {
    let mut options = CheckOptions::default();
    options.rule_filter.profile = Profile::Strict;
    options
}

fn load_regressions() -> Vec<DatasetRegression> {
    include_str!("../testdata/fixtures/eval/gec_dataset_regressions.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).expect("dataset regression fixture record parses"))
        .collect()
}

#[test]
fn corrected_dataset_fixture_targets_do_not_trigger_model_backed_false_positives() {
    let checker = checker();
    let guarded_rules = [
        "ru.grammar.adj_noun_agreement_demo",
        "ru.grammar.nominal_group_modifier_agreement_basic",
        "ru.grammar.preposition_nominal_group_government_basic",
    ];

    for record in load_regressions() {
        let issues = checker
            .check_with_options(&record.correction, &strict_options())
            .unwrap()
            .issues
            .into_iter()
            .filter(|issue| guarded_rules.contains(&issue.rule_id.as_str()))
            .map(|issue| issue.rule_id)
            .collect::<Vec<_>>();
        assert_eq!(issues, Vec::<String>::new(), "{}", record.id);
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

fn fixture_morphology() -> &'static str {
    "так\tтак\tADV\t\n\
     эдак\tэдак\tADV\t\n\
     у\tу\tPREP\t\n\
     него\tон\tPRON\tgender=masc|number=sing|case=gen\n\
     все\tвесь\tADJ\tnumber=plur|case=nom|adj_form=full\n\
     от\tот\tPREP\t\n\
     целый\tцелый\tADJ\tgender=masc|number=sing|case=nom|adj_form=full\n\
     год\tгод\tNOUN\tgender=masc|number=sing|case=nom|animacy=inan\n\
     слуху\tслух\tNOUN\tgender=masc|number=sing|case=dat|animacy=inan\n\
     духу\tдух\tNOUN\tgender=masc|number=sing|case=dat|animacy=inan\n\
     к\tк\tPREP\t\n\
     потере\tпотеря\tNOUN\tgender=fem|number=sing|case=dat|animacy=inan\n\
     слуха\tслух\tNOUN\tgender=masc|number=sing|case=gen|animacy=inan\n\
     зрения\tзрение\tNOUN\tgender=neut|number=sing|case=gen|animacy=inan\n\
     один\tодин\tNUMR\tgender=masc|number=sing|case=nom\n\
     из\tиз\tPREP\t\n\
     летних\tлетний\tADJ\tnumber=plur|case=gen|adj_form=full\n\
     вечеров\tвечер\tNOUN\tgender=masc|number=plur|case=gen|animacy=inan\n\
     прозвучал\tпрозвучать\tVERB\tgender=masc|number=sing|tense=past|verb_form=finite\n\
     тревожный\tтревожный\tADJ\tgender=masc|number=sing|case=nom|adj_form=full\n\
     звонок\tзвонок\tNOUN\tgender=masc|number=sing|case=nom|animacy=inan\n\
     сайтов\tсайт\tNOUN\tgender=masc|number=plur|case=gen|animacy=inan\n\
     авиакомпаний\tавиакомпания\tNOUN\tgender=fem|number=plur|case=gen|animacy=inan\n\
     поисках\tпоиск\tNOUN\tgender=masc|number=plur|case=prep|animacy=inan\n\
     скидок\tскидка\tNOUN\tgender=fem|number=plur|case=gen|animacy=inan\n\
     новому\tновый\tADJ\tgender=masc|number=sing|case=dat|adj_form=full\n\
     приказа\tприказ\tNOUN\tgender=masc|number=sing|case=gen|animacy=inan\n"
}
