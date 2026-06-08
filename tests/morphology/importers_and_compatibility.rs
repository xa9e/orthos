#[test]
fn low_level_compatibility_primitives_separate_unknown_from_incompatible() {
    assert_eq!(
        case_compatibility(Some(Case::Nominative), Some(Case::Genitive)),
        MorphCompatibility::Incompatible
    );
    assert_eq!(
        number_compatibility(Some(Number::Singular), None),
        MorphCompatibility::Unknown
    );
    assert_eq!(
        gender_compatibility(
            Some(Number::Plural),
            Some(Number::Plural),
            Some(Gender::Masculine),
            Some(Gender::Feminine),
        ),
        MorphCompatibility::Compatible
    );
}

#[test]
fn preposition_government_model_reports_known_case_sets() {
    let mut registry = PrepositionGovernmentRegistry::default();
    registry.insert(PrepositionGovernment::new("без", [Case::Genitive]));

    assert_eq!(
        registry.allows_case("БЕЗ", Case::Genitive),
        MorphCompatibility::Compatible
    );
    assert_eq!(
        registry.allows_case("без", Case::Dative),
        MorphCompatibility::Incompatible
    );
    assert_eq!(
        registry.allows_case("около", Case::Genitive),
        MorphCompatibility::Unknown
    );
}

#[test]
fn project_tsv_fixture_roundtrip_preserves_ids_provenance_and_stress() {
    let importer = ProjectTsvDictionaryImporter::curated("fixture.project", "Tiny project fixture");
    let lexicon = importer
        .import_lexicon(&morph_fixture("project.tsv"))
        .expect("project fixture imports");

    assert_eq!(lexicon.len(), 2);
    let zamok = &lexicon.analyze("замок")[0];
    assert_eq!(zamok.lemma_id.as_ref().map(|id| id.as_str()), Some("lemma:zamok"));
    assert_eq!(zamok.paradigm_id.as_ref().map(|id| id.as_str()), Some("paradigm:castle"));
    assert_eq!(zamok.source_id.as_ref().map(|id| id.as_str()), Some("fixture.project"));
    assert_eq!(zamok.stress.stressed_form.as_deref(), Some("за́мок"));
}

#[test]
fn opencorpora_xml_fixture_importer_preserves_lemma_paradigm_and_unknown_tags() {
    let importer = OpenCorporaXmlDictionaryImporter::fixture("fixture.opencorpora.xml", "Tiny OpenCorpora XML fixture");
    let lexicon = importer
        .import_lexicon(&morph_fixture("opencorpora.xml"))
        .expect("OpenCorpora XML fixture imports");

    assert_eq!(lexicon.len(), 3);
    let cat = &lexicon.analyze("кота")[0];
    assert_eq!(cat.lemma, "кот");
    assert_eq!(cat.pos, PartOfSpeech::Noun);
    assert_eq!(cat.features.case, Some(Case::Genitive));
    assert_eq!(cat.features.animacy, Some(orthos::Animacy::Animate));
    assert_eq!(cat.lemma_id.as_ref().map(|id| id.as_str()), Some("opencorpora:1"));
    assert_eq!(cat.paradigm_id.as_ref().map(|id| id.as_str()), Some("opencorpora:paradigm:1"));

    let new = &lexicon.analyze("новые")[0];
    assert_eq!(new.pos, PartOfSpeech::Adjective);
    assert!(new.features.unrecognized_tags.contains("FixUnknown"));
}

#[test]
fn opencorpora_csv_fixture_importer_preserves_stress() {
    let importer = OpenCorporaCsvDictionaryImporter::fixture("fixture.opencorpora.csv", "Tiny OpenCorpora CSV fixture");
    let lexicon = importer
        .import_lexicon(&morph_fixture("opencorpora.csv"))
        .expect("OpenCorpora CSV fixture imports");

    assert_eq!(lexicon.len(), 2);
    let analysis = &lexicon.analyze("мира")[0];
    assert_eq!(analysis.features.case, Some(Case::Genitive));
    assert_eq!(analysis.stress.availability, StressAvailability::Available);
    assert_eq!(analysis.stress.stressed_form.as_deref(), Some("ми́ра"));
}

#[test]
fn pymorphy_export_fixture_importer_keeps_ambiguity_and_provenance() {
    let importer = PymorphyExportDictionaryImporter::fixture("fixture.pymorphy", "Tiny pymorphy export fixture");
    let lexicon = importer
        .import_lexicon(&morph_fixture("pymorphy.tsv"))
        .expect("pymorphy fixture imports");

    let analyses = lexicon.analyze("стали");
    assert_eq!(analyses.len(), 2);
    assert!(analyses.iter().any(|analysis| analysis.pos == PartOfSpeech::Noun));
    assert!(analyses.iter().any(|analysis| analysis.pos == PartOfSpeech::Verb));
    assert!(analyses
        .iter()
        .any(|analysis| analysis.lemma_id.as_ref().map(|id| id.as_str()) == Some("pymorphy:lemma:2")));
    assert!(!confidently_reject_adj_noun_agreement(&analyses, &analyses));
}

#[test]
fn stress_tsv_enrichment_attaches_to_matching_dictionary_ids_only() {
    let importer = ProjectTsvDictionaryImporter::curated("fixture.project", "Tiny project fixture");
    let mut lexicon = importer
        .import_lexicon(&morph_fixture("project.tsv"))
        .expect("project fixture imports");
    let stress = StressTsvImporter::new("fixture.stress")
        .import_records(&morph_fixture("stress.tsv"))
        .expect("stress fixture imports");

    assert_eq!(lexicon.attach_stress_records(&stress), 1);
    let zamka = &lexicon.analyze("замка")[0];
    assert_eq!(zamka.stress.availability, StressAvailability::Available);
    assert_eq!(zamka.stress.stressed_form.as_deref(), Some("замка́"));
}

#[test]
fn subject_predicate_helper_is_conservative_and_typed() {
    let lexicon = lexicon(
        "мальчик\tмальчик\tNOUN\tgender=masc|number=sing|case=nom|animacy=anim\n\
         пришла\tприйти\tVERB\tgender=fem|number=sing|tense=past\n\
         пришёл\tприйти\tVERB\tgender=masc|number=sing|tense=past\n\
         пришли\tприйти\tVERB\tnumber=plur|tense=past\n",
    );
    let subject = &lexicon.analyze("мальчик")[0];

    assert_eq!(
        subject_predicate_compatibility(subject, &lexicon.analyze("пришёл")[0]),
        MorphCompatibility::Compatible
    );
    assert_eq!(
        subject_predicate_compatibility(subject, &lexicon.analyze("пришла")[0]),
        MorphCompatibility::Incompatible
    );
    assert_eq!(
        subject_predicate_compatibility(subject, &lexicon.analyze("пришли")[0]),
        MorphCompatibility::Incompatible
    );
}

#[test]
fn numeral_government_helper_handles_modeled_classes_without_guessing() {
    let lexicon = lexicon(
        "два\tдва\tNUMR\tnum_class=paucal|case=nomn\n\
         дома\tдом\tNOUN\tgender=masc|number=sing|case=gent|animacy=inan\n\
         дом\tдом\tNOUN\tgender=masc|number=sing|case=nomn|animacy=inan\n\
         много\tмного\tNUMR\tcase=nomn\n",
    );
    let two = &lexicon.analyze("два")[0];
    let houses = &lexicon.analyze("дома")[0];
    let house = &lexicon.analyze("дом")[0];
    let unknown = &lexicon.analyze("много")[0];

    assert_eq!(numeral_government_class(two), NumeralGovernmentClass::Paucal);
    assert_eq!(numeral_government_compatibility(two, houses), MorphCompatibility::Compatible);
    assert_eq!(numeral_government_compatibility(two, house), MorphCompatibility::Incompatible);
    assert_eq!(numeral_government_compatibility(unknown, houses), MorphCompatibility::Unknown);
}

#[test]
fn animacy_aware_accusative_helper_models_shadow_cases_conservatively() {
    let lexicon = lexicon(
        "кота\tкот\tNOUN\tgender=masc|number=sing|case=gent|animacy=anim\n\
         стол\tстол\tNOUN\tgender=masc|number=sing|case=nomn|animacy=inan\n\
         кошки\tкошка\tNOUN\tgender=fem|number=sing|case=gent|animacy=anim\n\
         дому\tдом\tNOUN\tgender=masc|number=sing|case=datv|animacy=inan\n",
    );

    assert_eq!(
        animacy_aware_accusative_compatibility(&lexicon.analyze("кота")[0]),
        MorphCompatibility::Compatible
    );
    assert_eq!(
        animacy_aware_accusative_compatibility(&lexicon.analyze("стол")[0]),
        MorphCompatibility::Compatible
    );
    assert_eq!(
        animacy_aware_accusative_compatibility(&lexicon.analyze("кошки")[0]),
        MorphCompatibility::Unknown
    );
    assert_eq!(
        animacy_aware_accusative_compatibility(&lexicon.analyze("дому")[0]),
        MorphCompatibility::Incompatible
    );
}


#[test]
fn quantity_agreement_check_reports_confident_numeral_noun_conflict() {
    let lexicon = lexicon(
        "два	два	NUMR	num_class=paucal|case=nomn
         дом	дом	NOUN	gender=masc|number=sing|case=nomn|animacy=inan
         дома	дом	NOUN	gender=masc|number=sing|case=gent|animacy=inan
",
    );

    let rejected = numeral_noun_agreement_check(&lexicon.analyze("два"), &lexicon.analyze("дом"));
    assert_eq!(rejected.compatibility, MorphCompatibility::Incompatible);
    assert!(rejected.conflict.is_some());

    let accepted = numeral_noun_agreement_check(&lexicon.analyze("два"), &lexicon.analyze("дома"));
    assert_eq!(accepted.compatibility, MorphCompatibility::Compatible);
}
