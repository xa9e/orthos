#[cfg(test)]
mod verb_government_syntax_tests {
    use super::*;
    use crate::morph::MorphLexicon;

    #[test]
    fn government_frames_include_prepositional_verb_valency() {
        let lexicon = MorphLexicon::parse_tsv(
            "говорит\tговорить\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite|aspect=impf\n\
             задачу\tзадача\tNOUN\tgender=fem|number=sing|case=acc|animacy=inan\n",
        );
        let facts = LinguisticFactStore::new("говорит о задачу", &lexicon);

        let frame = facts
            .government_frames()
            .iter()
            .find(|frame| {
                frame.kind == GovernmentFrameKind::Verb
                    && frame.source == GovernmentFrameSource::VerbPrepositionalValencySeed
                    && frame.governor.token.text == "говорит"
                    && frame.dependent.token.text == "задачу"
            })
            .expect("prepositional verb-government frame");

        assert_eq!(frame.expected_cases, vec![crate::morph::Case::Prepositional]);
        assert_eq!(frame.observed_cases, vec![crate::morph::Case::Accusative]);
        assert!(frame.is_conflict());
        let model_ref = frame.model_ref.as_ref().expect("model reference");
        assert_eq!(model_ref.lemma, "говорить");
        assert_eq!(
            model_ref.complement_kind,
            crate::morph::VerbGovernmentComplementKind::PrepositionalObject
        );
        assert_eq!(model_ref.preposition.as_deref(), Some("о"));
        assert_eq!(
            model_ref.source_id.as_ref().map(|source| source.as_str()),
            Some("project.verb_government_seed")
        );
    }

    #[test]
    fn verb_government_uses_lemma_for_direct_object_seed() {
        let lexicon = MorphLexicon::parse_tsv(
            "проверяет\tпроверять\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite|aspect=impf\n\
             проекту\tпроект\tNOUN\tgender=masc|number=sing|case=dat|animacy=inan\n",
        );
        let facts = LinguisticFactStore::new("проверяет проекту", &lexicon);

        let frame = facts
            .government_frames()
            .iter()
            .find(|frame| {
                frame.kind == GovernmentFrameKind::Verb
                    && frame.source == GovernmentFrameSource::VerbValencySeed
                    && frame.governor.token.text == "проверяет"
                    && frame.dependent.token.text == "проекту"
            })
            .expect("direct verb-government frame");

        assert_eq!(frame.expected_cases, vec![crate::morph::Case::Accusative]);
        assert_eq!(frame.observed_cases, vec![crate::morph::Case::Dative]);
        assert!(frame.is_conflict());
        let model_ref = frame.model_ref.as_ref().expect("model reference");
        assert_eq!(model_ref.lemma, "проверять");
        assert_eq!(
            model_ref.complement_kind,
            crate::morph::VerbGovernmentComplementKind::DirectObject
        );
        assert_eq!(model_ref.preposition, None);
    }

    #[test]
    fn clause_boundary_map_blocks_links_across_subordinator() {
        let tokens = tokenize("проверять что проекту");
        let map = ClauseBoundaryMap::from_text_tokens("проверять что проекту", &tokens);

        let blockers = map.blockers_between_tokens(0, 4);

        assert_eq!(blockers, vec![SuppressionReason::ClauseBoundary]);
        assert!(map.boundaries().iter().any(|boundary| {
            boundary.marker.canonical == "что" && boundary.kind == ClauseBoundaryKind::BeforeMarker
        }));
    }

    #[test]
    fn verb_government_frame_is_blocked_across_clause_boundary() {
        let lexicon = MorphLexicon::parse_tsv(
            "проверять\tпроверять\tVERB\tverb_form=infinitive|aspect=impf\n\
             что\tчто\tCONJ\tindecl\n\
             проекту\tпроект\tNOUN\tgender=masc|number=sing|case=dat|animacy=inan\n",
        );
        let facts = LinguisticFactStore::new("проверять что проекту", &lexicon);

        let frame = facts
            .government_frames()
            .iter()
            .find(|frame| {
                frame.kind == GovernmentFrameKind::Verb
                    && frame.source == GovernmentFrameSource::VerbValencySeed
                    && frame.governor.token.text == "проверять"
                    && frame.dependent.token.text == "проекту"
            })
            .expect("blocked direct verb-government frame");

        assert_eq!(frame.compatibility, crate::morph::MorphCompatibility::Incompatible);
        assert!(frame.blockers.contains(&SuppressionReason::ClauseBoundary));
        assert!(!frame.is_conflict());
    }

}
