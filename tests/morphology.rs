use orthos::{
    AdjectiveForm, AgreementFeatureKind, AmbiguityPolicy, Case, CheckOptions, Checker, Corpus,
    DictionaryImporter, Gender, Grammeme, MorphCompatibility, MorphFeatures, MorphLexicon, Number,
    NumeralGovernmentClass, OpenCorporaCsvDictionaryImporter, OpenCorporaXmlDictionaryImporter,
    PartOfSpeech, PrepositionGovernment, PrepositionGovernmentRegistry,
    ProjectTsvDictionaryImporter, PymorphyExportDictionaryImporter, StressAvailability,
    StressTsvImporter, UnknownTokenBehavior, animacy_aware_accusative_compatibility,
    can_agree_as_adj_noun, case_compatibility, confidently_reject_adj_noun_agreement,
    gender_compatibility, number_compatibility, numeral_government_class,
    numeral_government_compatibility, numeral_noun_agreement_check,
    subject_predicate_agreement_check, subject_predicate_compatibility,
};
use pretty_assertions::assert_eq;
use std::{fs, path::PathBuf};

fn corpus() -> Corpus {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Corpus::load_dir(root.join("rules")).expect("corpus loads")
}

fn lexicon(tsv: &str) -> MorphLexicon {
    MorphLexicon::parse_tsv(tsv)
}

fn morph_fixture(name: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join("testdata/fixtures/morph").join(name))
        .expect("morph fixture reads")
}

// Morphology tests are sharded by subsystem while sharing fixture helpers.

include!("morphology/lexicon_and_agreement.rs");
include!("morphology/importers_and_compatibility.rs");
