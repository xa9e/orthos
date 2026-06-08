#[cfg(test)]
mod linguistic_layer_tests {
    use super::*;
    use crate::morph::MorphLexicon;
    use crate::text::tokenize;

    #[test]
    fn syntactic_islands_do_not_cross_direct_speech() {
        let text = "Девочка пришла: «мальчик пришёл».";
        let tokens = tokenize(text);
        let islands = SyntacticIslandMap::from_text_tokens(text, &tokens);

        assert!(islands
            .islands()
            .iter()
            .any(|island| island.kind == SyntacticIslandKind::PlainSentence && island.is_actionable()));
        assert!(islands.islands().iter().any(|island| {
            island.kind == SyntacticIslandKind::DirectSpeech
                && island.blockers.contains(&SuppressionReason::DirectSpeechBoundary)
        }));
    }

    #[test]
    fn ambiguity_model_keeps_multiple_analyses_explicit() {
        let lexicon = MorphLexicon::demo();
        let tokens = tokenize("Дома стоят.");
        let ambiguity = AmbiguityModel::from_tokens(&tokens, &lexicon);
        let token_index = tokens.iter().position(|token| token.text == "Дома").unwrap();
        let item = ambiguity.for_token_index(token_index).unwrap();

        assert_eq!(item.class, AmbiguityClass::AmbiguousSamePartOfSpeech);
        assert!(item.blockers.contains(&SuppressionReason::AmbiguousMorphology));
    }

    #[test]
    fn fact_store_exposes_clauses_agreement_graph_and_government_frames() {
        let lexicon = MorphLexicon::demo();
        let facts = LinguisticFactStore::new("Согласно приказа. Девочка пришла.", &lexicon);

        assert!(!facts.nominal_groups().is_empty());
        assert!(facts.government_frames().iter().any(|frame| {
            frame.kind == GovernmentFrameKind::Preposition
                && frame.governor.token.text == "Согласно"
                && frame.dependent.token.text == "приказа"
                && frame.is_conflict()
        }));
        assert!(facts.clauses().iter().any(|clause| {
            clause
                .subject_candidate
                .as_ref()
                .is_some_and(|term| term.token.text == "Девочка")
                && clause
                    .predicate
                    .as_ref()
                    .is_some_and(|term| term.token.text == "пришла")
                && clause.is_actionable()
        }));
        assert!(facts.agreement_graph().edges().iter().any(|edge| {
            edge.kind == AgreementGraphEdgeKind::SubjectPredicate
                && edge.left.token.text == "Девочка"
                && edge.right.token.text == "пришла"
        }));
    }

    #[test]
    fn government_frames_include_seed_verb_valency() {
        let lexicon = MorphLexicon::demo();
        let facts = LinguisticFactStore::new("Ждать ответу нельзя.", &lexicon);

        assert!(facts.government_frames().iter().any(|frame| {
            frame.kind == GovernmentFrameKind::Verb
                && frame.source == GovernmentFrameSource::VerbValencySeed
                && frame.governor.token.text == "Ждать"
                && frame.dependent.token.text == "ответу"
                && frame.expected_cases == vec![crate::morph::Case::Genitive]
                && frame.observed_cases == vec![crate::morph::Case::Dative]
                && frame.is_conflict()
        }));
    }

    #[test]
    fn agreement_graph_exposes_nominal_group_edges_with_proofs() {
        let lexicon = MorphLexicon::demo();
        let facts = LinguisticFactStore::new("красивый новые дома", &lexicon);
        let edge = facts
            .agreement_graph()
            .conflicts_by_kind(AgreementGraphEdgeKind::ModifierHead)
            .find(|edge| edge.left.token.text == "красивый" && edge.right.token.text == "дома")
            .unwrap();
        let proof = agreement_edge_proof(edge);

        assert_eq!(proof.kind, DiagnosticProofKind::AgreementConflict);
        assert!(proof.is_actionable());
    }

    #[test]
    fn diagnostic_proof_is_machine_readable_and_actionable_only_with_conflict() {
        let proof = DiagnosticProof::new(DiagnosticProofKind::GovernmentConflict, SyntaxConfidence::Strong)
            .with_fact(DiagnosticFact::new("governor", "согласно", Some(Span::new(0, 8))));
        assert!(!proof.is_actionable());

        let blocked = DiagnosticProof::new(DiagnosticProofKind::AmbiguitySuppression, SyntaxConfidence::Strong)
            .with_blocker(SuppressionReason::AmbiguousMorphology);
        assert!(!blocked.is_actionable());
    }

    #[test]
    fn feature_unification_uses_intersections_across_multiple_fixtures() {
        let lexicon = MorphLexicon::parse_tsv(
            "синем\tсиний\tADJ\tgender=masc|number=sing|case=dat|adj_form=full|degree=pos\n\
             синем\tсиний\tADJ\tgender=masc|number=sing|case=loc|adj_form=full|degree=pos\n\
             дому\tдом\tNOUN\tgender=masc|number=sing|case=dat|animacy=inan\n\
             новые\tновый\tADJ\tnumber=plur|case=nom|adj_form=full|degree=pos\n\
             дом\tдом\tNOUN\tgender=masc|number=sing|case=nom|animacy=inan\n",
        );

        let compatible = crate::morph::adjective_noun_feature_unification(
            &lexicon.analyze("синем"),
            &lexicon.analyze("дому"),
        );
        let case_step = compatible
            .steps
            .iter()
            .find(|step| step.feature == crate::morph::AgreementFeatureKind::Case)
            .unwrap();
        assert_eq!(compatible.compatibility(), crate::morph::MorphCompatibility::Compatible);
        assert_eq!(case_step.intersection, vec!["Dative".to_owned()]);

        let conflict = crate::morph::adjective_noun_feature_unification(
            &lexicon.analyze("новые"),
            &lexicon.analyze("дом"),
        );
        assert_eq!(conflict.compatibility(), crate::morph::MorphCompatibility::Incompatible);
        assert!(conflict
            .steps
            .iter()
            .any(|step| step.feature == crate::morph::AgreementFeatureKind::Number
                && step.status == crate::morph::FeatureUnificationStatus::Conflict));
    }

    #[test]
    fn coordination_groups_are_extracted_from_multiple_fixtures() {
        let lexicon = MorphLexicon::parse_tsv(coordination_fixture_tsv());
        let modifier = LinguisticFactStore::new("умный, быстрый и надёжный анализатор", &lexicon);
        assert!(modifier.coordination_groups().iter().any(|group| {
            group.kind == CoordinationGroupKind::ModifierSeries
                && group.members.len() == 3
                && group.connectors.len() == 2
                && group.is_actionable()
        }));

        let subject = LinguisticFactStore::new("мама и папа пришли", &lexicon);
        assert!(subject.coordination_groups().iter().any(|group| {
            group.kind == CoordinationGroupKind::NominalSubject
                && group.members.len() == 2
                && group.agreement_number == Some(crate::morph::Number::Plural)
                && group.shared_cases == vec![crate::morph::Case::Nominative]
        }));

        let repeated = LinguisticFactStore::new("ни рыба ни мясо", &lexicon);
        assert!(repeated.coordination_groups().iter().any(|group| {
            group.connectors
                .iter()
                .any(|connector| connector.kind == CoordinationConnectorKind::RepeatedConjunction)
        }));
    }

    #[test]
    fn punctuation_slots_document_expected_marks_and_suppression_fixtures() {
        let lexicon = MorphLexicon::parse_tsv(coordination_fixture_tsv());
        let introductory = LinguisticFactStore::new("Конечно анализатор работает.", &lexicon);
        assert!(introductory.punctuation_slots().iter().any(|slot| {
            slot.left_token.text == "Конечно"
                && slot.after_introductory_candidate
                && slot.expects(PunctuationMark::Comma)
        }));

        let coordinated = LinguisticFactStore::new("умный, быстрый анализатор", &lexicon);
        assert!(coordinated.punctuation_slots().iter().any(|slot| {
            slot.left_token.text == "умный"
                && slot.right_token.text == "быстрый"
                && slot.existing_marks.contains(&PunctuationMark::Comma)
        }));

        let quoted = LinguisticFactStore::new("Он сказал: «конечно анализатор работает».", &lexicon);
        assert!(quoted.punctuation_slots().iter().any(|slot| {
            slot.inside_quotes && slot.blockers.contains(&SuppressionReason::DirectSpeechBoundary)
        }));

        let proof_slot = introductory
            .punctuation_slots()
            .iter()
            .find(|slot| slot.left_token.text == "Конечно")
            .unwrap();
        assert!(proof_slot.missing_expected_mark(PunctuationMark::Comma));
        assert!(proof_slot.evidence.iter().any(|item| {
            item.kind == PunctuationSlotEvidenceKind::IntroductoryCandidate
                && item.mark == Some(PunctuationMark::Comma)
        }));
    }

    #[test]
    fn document_context_tracks_headings_lists_and_cross_sentence_terms() {
        let lexicon = MorphLexicon::parse_tsv(coordination_fixture_tsv());
        let facts = LinguisticFactStore::new(
            "# Глоссарий\n- анализатор — модуль проверки\n* парсер: модуль разбора\n\nОрганизация Объединённых Наций (ООН) выпустила документ. Анализатор точен. Анализатор стабилен. ООН обновила стиль.",
            &lexicon,
        );
        let document = facts.document_context();

        assert_eq!(document.style.heading_count, 1);
        assert_eq!(document.style.list_item_count, 2);
        assert!(document.style.mixed_list_markers);
        assert!(document.repeated_terms.iter().any(|term| term.canonical == "анализатор"));
        assert!(document.cross_sentence_facts.iter().any(|fact| {
            fact.key == "repeated_term_across_sentences" && fact.value == "анализатор"
        }));
        assert!(document.abbreviations.iter().any(|abbr| {
            abbr.short == "ООН"
                && abbr.frequency == 2
                && abbr.expansion.as_deref() == Some("Организация Объединённых Наций")
        }));
        assert!(document.cross_sentence_facts.iter().any(|fact| {
            fact.key == "repeated_abbreviation_across_sentences" && fact.value == "ООН"
        }));
        assert!(document.glossary_entries.iter().any(|entry| {
            entry.term == "анализатор" && entry.definition == "модуль проверки"
        }));
        assert!(document.glossary_entries.iter().any(|entry| {
            entry.term == "парсер" && entry.definition == "модуль разбора"
        }));
    }


    #[test]
    fn fact_store_summary_is_a_debuggable_document_profile() {
        let lexicon = MorphLexicon::parse_tsv(coordination_fixture_tsv());
        let facts = LinguisticFactStore::new(
            "# Глоссарий\n- анализатор — модуль проверки\n\nМама и папа пришли. Анализатор работает.",
            &lexicon,
        );
        let summary = facts.summary();

        assert!(summary.tokens > 0);
        assert!(summary.islands > 0);
        assert!(summary.coordination_groups > 0);
        assert!(summary.punctuation_slots > 0);
        assert_eq!(summary.glossary_entries, 1);
    }

    fn coordination_fixture_tsv() -> &'static str {
        "умный\tумный\tADJ\tgender=masc|number=sing|case=nom|adj_form=full|degree=pos\n\
         быстрый\tбыстрый\tADJ\tgender=masc|number=sing|case=nom|adj_form=full|degree=pos\n\
         надёжный\tнадёжный\tADJ\tgender=masc|number=sing|case=nom|adj_form=full|degree=pos\n\
         анализатор\tанализатор\tNOUN\tgender=masc|number=sing|case=nom|animacy=inan\n\
         мама\tмама\tNOUN\tgender=fem|number=sing|case=nom|animacy=anim\n\
         папа\tпапа\tNOUN\tgender=masc|number=sing|case=nom|animacy=anim\n\
         пришли\tприйти\tVERB\tnumber=plur|tense=past|verb_form=finite|aspect=perf\n\
         рыба\tрыба\tNOUN\tgender=fem|number=sing|case=nom|animacy=inan\n\
         мясо\tмясо\tNOUN\tgender=neut|number=sing|case=nom|animacy=inan\n\
         работает\tработать\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite|aspect=impf\n\
         проверяет\tпроверять\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite|aspect=impf\n\
         точен\tточный\tADJ\tgender=masc|number=sing|adj_form=short|degree=pos\n\
         стабилен\tстабильный\tADJ\tgender=masc|number=sing|adj_form=short|degree=pos\n"
    }
}
