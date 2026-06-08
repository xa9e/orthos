#[cfg(test)]
mod tests {
    use super::*;

    fn first(lexicon: &MorphLexicon, token: &str) -> MorphAnalysis {
        lexicon
            .analyze(token)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("missing analysis for {token}"))
    }

    #[test]
    fn parses_demo_lexicon_into_typed_features() {
        let lexicon = MorphLexicon::parse_tsv(
            "красивый\tкрасивый\tADJ\tgender=masc|number=sing|case=nom|animacy=inan\n\
             дом\tдом\tNOUN\tgender=masc|number=sing|case=nom|animacy=inan\n",
        );
        let adj = first(&lexicon, "Красивый");
        let noun = first(&lexicon, "дом");

        assert_eq!(adj.pos, PartOfSpeech::Adjective);
        assert_eq!(adj.features.gender, Some(Gender::Masculine));
        assert_eq!(noun.features.case, Some(Case::Nominative));
        assert!(can_agree_as_adj_noun(&adj, &noun));
    }

    #[test]
    fn rejects_incompatible_complete_adj_noun_pair() {
        let lexicon = MorphLexicon::parse_tsv(
            "красивый\tкрасивый\tADJ\tgender=masc|number=sing|case=nom\n\
             машина\tмашина\tNOUN\tgender=fem|number=sing|case=nom\n",
        );
        assert!(!can_agree_as_adj_noun(
            &first(&lexicon, "красивый"),
            &first(&lexicon, "машина")
        ));
    }

    #[test]
    fn unknown_or_missing_features_are_not_confident_rejections() {
        let lexicon = MorphLexicon::parse_tsv(
            "красивый\tкрасивый\tADJ\tgender=masc|number=sing\n\
             машина\tмашина\tNOUN\tgender=fem|number=sing|case=nom\n",
        );
        let adj = lexicon.analyze("красивый");
        let noun = lexicon.analyze("машина");
        assert!(!confidently_reject_adj_noun_agreement(&adj, &noun));
        assert!(!confidently_reject_adj_noun_agreement(&[], &noun));
    }

    #[test]
    fn ambiguity_is_safe_by_default() {
        let lexicon = MorphLexicon::parse_tsv(
            "новые\tновый\tADJ\tnumber=plur|case=nom\n\
             дома\tдом\tNOUN\tgender=masc|number=sing|case=gen\n\
             дома\tдом\tNOUN\tgender=masc|number=plur|case=nom\n",
        );
        let adj = lexicon.analyze("новые");
        let noun = lexicon.analyze("дома");
        assert!(!confidently_reject_adj_noun_agreement(&adj, &noun));
    }

    #[test]
    fn preposition_government_check_reports_confident_case_conflict() {
        let registry = PrepositionGovernmentRegistry::russian_seed();
        let lexicon = MorphLexicon::parse_tsv(
            "приказа\tприказ\tNOUN\tgender=masc|number=sing|case=gen\n\
             приказу\tприказ\tNOUN\tgender=masc|number=sing|case=dat\n",
        );

        let bad = preposition_government_check(&registry, "согласно", &lexicon.analyze("приказа"));
        let good =
            preposition_government_check(&registry, "согласно", &lexicon.analyze("приказу"));

        assert!(bad.is_confident_rejection());
        assert_eq!(good.compatibility, MorphCompatibility::Compatible);
    }

    #[test]
    fn clitic_model_suggests_hyphens_for_seed_groups() {
        let indefinite = RussianCliticModel::suggest_missing_hyphen(
            "Кто",
            "то",
            CliticHyphenGroup::IndefinitePronominal,
        )
        .expect("indefinite clitic suggestion");
        assert_eq!(indefinite.replacement, "Кто-то");

        let emphatic = RussianCliticModel::suggest_missing_hyphen(
            "я",
            "то",
            CliticHyphenGroup::EmphaticToPronounSeed,
        )
        .expect("emphatic clitic suggestion");
        assert_eq!(emphatic.replacement, "я-то");
        assert!(RussianCliticModel::suggest_missing_hyphen(
            "дом",
            "то",
            CliticHyphenGroup::EmphaticToPronounSeed,
        )
        .is_none());
    }

    #[test]
    fn negation_spacing_model_uses_verbal_analyses_conservatively() {
        let lexicon = MorphLexicon::parse_tsv(
            "знаю\tзнать\tVERB\tnumber=sing|person=1|tense=pres|verb_form=finite\n\
             ненавидит\tненавидеть\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite\n",
        );
        let suggestion = split_negated_verb_candidate("незнаю", &lexicon)
            .expect("не + known verb should be split");
        assert_eq!(suggestion.replacement, "не знаю");
        assert!(split_negated_verb_candidate("ненавидит", &lexicon).is_none());
        assert!(split_negated_verb_candidate("неведомо", &lexicon).is_none());
    }


    #[test]
    fn verb_government_seed_registry_loads_direct_and_prepositional_entries() {
        let registry = VerbGovernmentRegistry::russian_seed();

        assert!(registry.len() >= 28);
        assert_eq!(
            registry.direct_cases_for_lemma("проверять"),
            [Case::Accusative].into_iter().collect()
        );
        let entries = registry.prepositional_entries_for_lemma("говорить");
        assert!(entries.iter().any(|entry| {
            entry.preposition.as_deref() == Some("о")
                && entry.allowed_cases.contains(&Case::Prepositional)
        }));
    }

    #[test]
    fn verb_government_seed_parser_rejects_bad_prepositional_rows() {
        let seed = "говорить\tprepositional_object\t\tprep\tproject.test\tmissing preposition\n";
        let error = VerbGovernmentRegistry::parse_seed_tsv(seed).unwrap_err();

        assert!(error.message.contains("must specify preposition"));
    }

    #[test]
    fn verb_government_seed_parser_rejects_unknown_cases() {
        let seed = "говорить\tprepositional_object\tо\tweird\tproject.test\tbad case\n";
        let error = VerbGovernmentRegistry::parse_seed_tsv(seed).unwrap_err();

        assert!(error.message.contains("unknown case"));
    }

    #[test]
    fn verb_government_seed_parser_rejects_duplicate_keys() {
        let seed = "говорить\tprepositional_object\tо\tprep\tproject.test\tnote\n\
                    говорить\tprepositional_object\tо\tprep\tproject.test\tnote\n";
        let error = VerbGovernmentRegistry::parse_seed_tsv(seed).unwrap_err();

        assert!(error.message.contains("duplicate"));
    }

    #[test]
    fn verb_government_fixtures_cover_builtin_seed_rows() {
        let registry = VerbGovernmentRegistry::russian_seed();
        let fixtures = VerbGovernmentFixtureSet::russian_seed();

        assert_eq!(fixtures.len(), registry.len());
        for entry in registry.entries() {
            let key = VerbGovernmentKey::from_government(entry);
            assert!(
                fixtures.contains_key(&key),
                "missing verb-government fixture for {:?}",
                key
            );
        }
    }

    #[test]
    fn verb_government_fixture_parser_rejects_duplicate_keys() {
        let fixtures = concat!(
            "ждать\tdirect_object\t\tждать ответа\tждать ответу\tждать ответу\n",
            "ждать\tdirect_object\t\tждать ответа\tждать ответу\tждать ответу\n",
        );
        let error = VerbGovernmentFixtureSet::parse_tsv(fixtures).unwrap_err();

        assert!(error.message.contains("duplicate fixture"));
    }

    #[test]
    fn verb_government_fixture_parser_rejects_bad_excerpt() {
        let fixtures = "ждать\tdirect_object\t\tждать ответа\tждать ответу\tдругая строка\n";
        let error = VerbGovernmentFixtureSet::parse_tsv(fixtures).unwrap_err();

        assert!(error.message.contains("invalid_excerpt"));
    }


    #[test]
    fn verb_government_false_positive_fixtures_load_builtin_rows() {
        let fixtures = VerbGovernmentFalsePositiveFixtureSet::russian_seed();

        assert!(!fixtures.is_empty());
        assert!(fixtures.fixtures().iter().any(|fixture| {
            fixture.id == "direct_speech_prepositional"
                && fixture.key.lemma == "говорить"
                && fixture.expected_blocker.as_deref() == Some("DirectSpeechBoundary")
        }));
    }

    #[test]
    fn verb_government_false_positive_parser_rejects_duplicate_ids() {
        let fixtures = concat!(
            "fp1	ждать	direct_object		ждать ответу	ждать ответу		reason
",
            "fp1	ждать	direct_object		ждать ответу	ждать ответу		reason
",
        );
        let error = VerbGovernmentFalsePositiveFixtureSet::parse_tsv(fixtures).unwrap_err();

        assert!(error.message.contains("duplicate false-positive fixture id"));
    }

    #[test]
    fn verb_government_false_positive_parser_rejects_bad_excerpt() {
        let fixtures = "fp1	ждать	direct_object		ждать ответу	другая строка		reason
";
        let error = VerbGovernmentFalsePositiveFixtureSet::parse_tsv(fixtures).unwrap_err();

        assert!(error.message.contains("forbidden_excerpt"));
    }


    #[test]
    fn verb_government_false_positive_parser_rejects_unknown_blocker() {
        let fixtures = "fp1	ждать	direct_object		ждать ответу	ждать ответу	MagicBoundary	reason
";
        let error = VerbGovernmentFalsePositiveFixtureSet::parse_tsv(fixtures).unwrap_err();

        assert!(error.message.contains("unknown expected_blocker"));
    }

    #[test]
    fn verb_government_false_positive_fixtures_include_clause_boundaries() {
        let fixtures = VerbGovernmentFalsePositiveFixtureSet::russian_seed();

        assert!(fixtures.fixtures().iter().any(|fixture| {
            fixture.id == "clause_marker_direct"
                && fixture.key.lemma == "проверять"
                && fixture.expected_blocker.as_deref() == Some("ClauseBoundary")
        }));
    }

}
