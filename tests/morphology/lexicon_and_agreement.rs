#[test]
fn typed_lexicon_preserves_demo_tsv_path() {
    let lexicon = MorphLexicon::demo();
    let analyses = lexicon.analyze("КРАСИВЫЙ");

    assert_eq!(analyses.len(), 1);
    assert_eq!(analyses[0].pos, PartOfSpeech::Adjective);
    assert_eq!(analyses[0].features.case, Some(Case::Nominative));
    assert_eq!(analyses[0].features.number, Some(Number::Singular));
    assert_eq!(analyses[0].features.gender, Some(Gender::Masculine));
}

#[test]
fn compact_cache_can_load_requested_forms_only() {
    let lexicon = lexicon(
        "дом\tдом\tNOUN\tgender=masc|number=sing|case=nom\n\
         машина\tмашина\tNOUN\tgender=fem|number=sing|case=nom\n",
    );
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "orthos-filtered-cache-{}-{nanos}.bincache",
        std::process::id()
    ));
    lexicon.save_cache(&path).expect("cache writes");
    let mut index_os = path.as_os_str().to_os_string();
    index_os.push(".idx");
    let index_path = PathBuf::from(index_os);

    let forms = ["дом".to_owned()].into_iter().collect();
    let filtered = MorphLexicon::load_cache_for_forms(&path, &forms).expect("filtered cache loads");
    let filtered_again =
        MorphLexicon::load_cache_for_forms(&path, &forms).expect("indexed filtered cache loads");
    let index_magic = fs::read(&index_path).expect("index reads");

    assert!(index_path.exists());
    assert_eq!(&index_magic[..4], b"RLI2");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered_again.len(), 1);
    assert_eq!(filtered.analyze("дом").len(), 1);
    assert!(filtered.analyze("машина").is_empty());
    assert_eq!(filtered.metadata()[0].entry_count, Some(1));

    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&index_path);
}

#[test]
fn compatible_adj_noun_analyses_are_accepted() {
    let lexicon = lexicon(
        "красивый\tкрасивый\tADJ\tgender=masc|number=sing|case=nom\n\
         дом\tдом\tNOUN\tgender=masc|number=sing|case=nom\n",
    );
    let adj = &lexicon.analyze("красивый")[0];
    let noun = &lexicon.analyze("дом")[0];

    assert!(can_agree_as_adj_noun(adj, noun));
    assert!(!confidently_reject_adj_noun_agreement(
        &lexicon.analyze("красивый"),
        &lexicon.analyze("дом")
    ));
}

#[test]
fn incompatible_adj_noun_analyses_are_confidently_rejected() {
    let lexicon = lexicon(
        "красивый\tкрасивый\tADJ\tgender=masc|number=sing|case=nom\n\
         машина\tмашина\tNOUN\tgender=fem|number=sing|case=nom\n",
    );

    assert!(confidently_reject_adj_noun_agreement(
        &lexicon.analyze("красивый"),
        &lexicon.analyze("машина")
    ));
}

#[test]
fn ambiguous_analyses_do_not_trigger_demo_agreement_rule() {
    let lexicon = lexicon(
        "новые\tновый\tADJ\tnumber=plur|case=nom\n\
         дома\tдом\tNOUN\tgender=masc|number=sing|case=gen\n\
         дома\tдом\tNOUN\tgender=masc|number=plur|case=nom\n",
    );
    let checker = Checker::with_morph_lexicon(corpus(), lexicon);
    let mut options = CheckOptions::default();
    options.rule_filter.profile = orthos::Profile::Strict;
    let issues = checker
        .check_with_options("новые дома", &options)
        .expect("check succeeds")
        .issues;

    assert!(!issues
        .iter()
        .any(|issue| issue.rule_id == "ru.grammar.adj_noun_agreement_demo"));
}

#[test]
fn unknown_tokens_do_not_trigger_demo_agreement_rule() {
    let checker = Checker::with_morph_lexicon(corpus(), MorphLexicon::empty());
    let mut options = CheckOptions::default();
    options.rule_filter.profile = orthos::Profile::Strict;
    let issues = checker
        .check_with_options("красивый машина", &options)
        .expect("check succeeds")
        .issues;

    assert!(!issues
        .iter()
        .any(|issue| issue.rule_id == "ru.grammar.adj_noun_agreement_demo"));
}

#[test]
fn current_yaml_regression_for_demo_agreement_still_fires_in_strict_profile() {
    let checker = Checker::new(corpus());
    let mut options = CheckOptions::default();
    options.rule_filter.profile = orthos::Profile::Strict;
    let issues = checker
        .check_with_options("красивый машина", &options)
        .expect("check succeeds")
        .issues;

    assert!(issues
        .iter()
        .any(|issue| issue.rule_id == "ru.grammar.adj_noun_agreement_demo"));
}

#[test]
fn feature_normalization_accepts_opencorpora_aliases() {
    let features = MorphFeatures::parse("femn|plur|gent|ADJF|@Name|unknown-tag|form=plen");

    assert_eq!(features.gender, Some(Gender::Feminine));
    assert_eq!(features.number, Some(Number::Plural));
    assert_eq!(features.case, Some(Case::Genitive));
    assert_eq!(features.adjective_form, Some(AdjectiveForm::Full));
    assert!(features.grammemes.contains(&Grammeme::ProperName));
    assert!(features.normalized_tags.contains("unknown_tag"));
    assert!(!features.unrecognized_tags.contains("ADJF"));
    assert!(features.unrecognized_tags.contains("unknown-tag"));
}

#[test]
fn project_tsv_importer_parses_fixture_metadata_ids_and_stress() {
    let importer = ProjectTsvDictionaryImporter::curated("fixture.project", "Tiny project fixture");
    let lexicon = importer
        .import_lexicon(
            "замок\tзамок\tNOUN\tgender=masc|number=sing|case=nom\tlemma:zamok\tparadigm:1\tfixture.project\tза́мок\n",
        )
        .expect("fixture imports");

    let metadata = lexicon.metadata();
    assert_eq!(metadata.len(), 1);
    assert_eq!(metadata[0].source_id.as_str(), "fixture.project");
    assert_eq!(metadata[0].entry_count, Some(1));
    assert!(metadata[0].has_stress);

    let capabilities = lexicon.capabilities();
    assert!(capabilities.lexical_lookup);
    assert!(capabilities.returns_lemma_ids);
    assert!(capabilities.returns_paradigm_ids);
    assert!(capabilities.returns_provenance);
    assert_eq!(capabilities.ambiguity_policy, AmbiguityPolicy::ConservativeDiagnostics);
    assert_eq!(capabilities.unknown_token_behavior, UnknownTokenBehavior::NoAnalysis);
    assert_eq!(capabilities.stress, StressAvailability::Available);

    let analysis = &lexicon.analyze("ЗАМОК")[0];
    assert_eq!(analysis.lemma_id.as_ref().map(|id| id.as_str()), Some("lemma:zamok"));
    assert_eq!(analysis.paradigm_id.as_ref().map(|id| id.as_str()), Some("paradigm:1"));
    assert_eq!(analysis.source_id.as_ref().map(|id| id.as_str()), Some("fixture.project"));
    assert_eq!(analysis.stress.availability, StressAvailability::Available);
    assert_eq!(analysis.stress.stressed_form.as_deref(), Some("за́мок"));
}

#[test]
fn strict_importer_rejects_malformed_fixture_line() {
    let importer = ProjectTsvDictionaryImporter::curated("fixture.bad", "Malformed fixture");
    let error = importer
        .import_lexicon("только\tдве\n")
        .expect_err("strict importer rejects short rows");

    assert_eq!(error.line, Some(1));
    assert!(error.message.contains("expected at least 4 TSV columns"));
}


#[test]
fn derivation_model_returns_morpheme_segments() {
    let model = orthos::RussianDerivationModel::seed();
    let parses = model.analyze_word("писательница");
    let best = parses.first().expect("parse exists");

    assert!(best.root().is_some_and(|root| root.form == "пис"));
    assert!(best.suffixes().any(|suffix| suffix.form == "тель"));
    assert!(best.suffixes().any(|suffix| suffix.form == "ниц"));
}

#[test]
fn public_prefix_assimilation_helper_is_conservative() {
    let misspelled = orthos::prefix_final_z_s_suggestion("безсмертный").expect("suggestion");
    assert_eq!(misspelled.replacement, "бессмертный");
    assert!(orthos::prefix_final_z_s_suggestion("расист").is_none());
}

#[test]
fn subject_predicate_agreement_check_reports_past_gender_conflict() {
    let lexicon = lexicon(
        "девочка\tдевочка\tNOUN\tgender=fem|number=sing|case=nom\n\
         пришёл\tприйти\tVERB\tgender=masc|number=sing|tense=past|verb_form=finite\n",
    );
    let check = subject_predicate_agreement_check(&lexicon.analyze("девочка")[0], &lexicon.analyze("пришёл")[0]);

    assert_eq!(check.compatibility, MorphCompatibility::Incompatible);
    assert!(check.conflicts.iter().any(|conflict| conflict.feature == AgreementFeatureKind::Gender));
}

#[test]
fn subject_predicate_detector_fires_only_for_confident_adjacent_pairs() {
    let checker = Checker::new(corpus());
    let mut options = CheckOptions::default();
    options.rule_filter.profile = orthos::Profile::Strict;
    let issues = checker
        .check_with_options("Девочка пришёл. Дети пришли.", &options)
        .expect("check succeeds")
        .issues;

    assert_eq!(
        issues
            .iter()
            .filter(|issue| issue.rule_id == "ru.grammar.subject_predicate_agreement_basic")
            .count(),
        1
    );
}
