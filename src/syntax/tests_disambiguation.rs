#[cfg(test)]
mod disambiguation_tests {
    use super::*;
    use crate::morph::{MorphLexicon, PartOfSpeech, PrepositionGovernmentRegistry};

    fn readings_for(lexicon: &MorphLexicon, tokens: &[Token<'_>]) -> Vec<Vec<crate::morph::MorphAnalysis>> {
        tokens
            .iter()
            .map(|token| {
                if token.kind == TokenKind::Word {
                    lexicon.analyze(token.text)
                } else {
                    Vec::new()
                }
            })
            .collect()
    }

    fn run(text: &str, lexicon_tsv: &str) -> (Vec<Vec<crate::morph::MorphAnalysis>>, DisambiguationTrace) {
        let lexicon = MorphLexicon::parse_tsv(lexicon_tsv);
        let tokens = tokenize(text);
        let mut readings = readings_for(&lexicon, &tokens);
        let trace = disambiguate_readings(
            &tokens,
            &mut readings,
            &PrepositionGovernmentRegistry::russian_seed(),
        );
        (readings, trace)
    }

    const STEEL_TSV: &str = "из\tиз\tPREP\tcase=genitive\n\
         стали\tсталь\tNOUN\tgender=fem|number=sing|case=gen|animacy=inan\n\
         стали\tстать\tVERB\tnumber=plur|tense=past|verb_form=finite\n";

    #[test]
    fn preposition_excludes_verb_reading() {
        let (readings, trace) = run("Из стали куют мечи.", STEEL_TSV);
        // token layout: Из(0) ws(1) стали(2) ...
        assert_eq!(readings[2].len(), 1);
        assert_eq!(readings[2][0].pos, PartOfSpeech::Noun);
        assert!(trace.eliminations.iter().any(|e| {
            e.constraint == DisambiguationConstraint::PrepositionVerbExclusion
                && e.token_index == 2
                && e.evidence_form == "Из"
                && e.eliminated_lemma == "стать"
        }));
    }

    #[test]
    fn preposition_case_government_prunes_disallowed_case() {
        let tsv = "у\tу\tPREP\tcase=genitive\n\
             дома\tдом\tNOUN\tgender=masc|number=sing|case=gen|animacy=inan\n\
             дома\tдом\tNOUN\tgender=masc|number=plur|case=nom|animacy=inan\n";
        let (readings, trace) = run("У дома растёт клён.", tsv);
        assert_eq!(readings[2].len(), 1);
        assert_eq!(
            readings[2][0].features.case,
            Some(crate::morph::Case::Genitive)
        );
        assert!(trace.eliminations.iter().any(|e| {
            e.constraint == DisambiguationConstraint::PrepositionCaseGovernment
                && e.eliminated_features.contains("case=nom")
        }));
    }

    #[test]
    fn modifier_head_agreement_prunes_both_sides() {
        let tsv = "новые\tновый\tADJ\tnumber=plur|case=nom|adj_form=full\n\
             новые\tновый\tADJ\tnumber=plur|case=acc|adj_form=full\n\
             дома\tдом\tNOUN\tgender=masc|number=sing|case=gen|animacy=inan\n\
             дома\tдом\tNOUN\tgender=masc|number=plur|case=nom|animacy=inan\n";
        let (readings, trace) = run("новые дома", tsv);
        assert_eq!(readings[0].len(), 1, "modifier keeps only the agreeing case");
        assert_eq!(readings[0][0].features.case, Some(crate::morph::Case::Nominative));
        assert_eq!(readings[2].len(), 1, "noun keeps only the agreeing reading");
        assert_eq!(readings[2][0].features.number, Some(crate::morph::Number::Plural));
        assert!(trace
            .eliminations
            .iter()
            .all(|e| e.constraint == DisambiguationConstraint::ModifierHeadAgreement));
        assert_eq!(trace.eliminations.len(), 2);
    }

    #[test]
    fn real_agreement_error_is_not_pruned_away() {
        let tsv = "красивая\tкрасивый\tADJ\tgender=fem|number=sing|case=nom|adj_form=full\n\
             дом\tдом\tNOUN\tgender=masc|number=sing|case=nom|animacy=inan\n";
        let (readings, trace) = run("красивая дом", tsv);
        assert!(trace.is_empty(), "no agreeing pair exists, engine must not prune");
        assert_eq!(readings[0].len(), 1);
        assert_eq!(readings[2].len(), 1);
    }

    #[test]
    fn last_reading_is_never_removed() {
        let tsv = "у\tу\tPREP\tcase=genitive\n\
             москва\tмосква\tNOUN\tgender=fem|number=sing|case=nom|animacy=inan\n";
        let (readings, trace) = run("У Москва", tsv);
        assert!(trace.is_empty());
        assert_eq!(readings[2].len(), 1, "sole disallowed reading survives");
    }

    #[test]
    fn punctuation_breaks_preposition_adjacency() {
        let (readings, trace) = run("Из, стали куют.", STEEL_TSV);
        assert!(trace.is_empty());
        assert_eq!(readings[3].len(), 2);
    }

    #[test]
    fn fact_store_exposes_disambiguation_trace() {
        let lexicon = MorphLexicon::parse_tsv(STEEL_TSV);
        let store = LinguisticFactStore::new("Из стали куют мечи.", &lexicon);
        assert_eq!(store.summary().eliminated_readings, 1);
        assert_eq!(store.disambiguation().eliminations_for_token(2).len(), 1);
        let ambiguity = store.ambiguity().for_token_index(2).expect("token tracked");
        assert_eq!(
            ambiguity.class,
            AmbiguityClass::Unambiguous,
            "downstream ambiguity model sees the reduced reading set"
        );
    }
}
