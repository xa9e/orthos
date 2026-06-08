#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::tokenize;
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn sentence_spans_skip_leading_whitespace_and_keep_final_fragment() {
        let spans = sentence_spans("  Привет. Пока");
        assert_eq!(
            spans,
            vec![SentenceSpan { start: 2, end: 15 }, SentenceSpan { start: 16, end: 24 }]
        );
    }

    #[test]
    fn sentence_spans_handle_abbreviations_and_ellipses() {
        let text = "В г. Алматы тихо... Потом начался дождь. См. также приложение.";
        let rendered = sentence_spans(text)
            .into_iter()
            .map(|span| text[span.start..span.end].to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            rendered,
            vec![
                "В г. Алматы тихо...".to_string(),
                "Потом начался дождь.".to_string(),
                "См. также приложение.".to_string()
            ]
        );
    }

    #[test]
    fn sentence_spans_do_not_split_inside_nested_safe_zone_fragment() {
        let text = "Он оставил заметку (пример: «что делать?» внутри) и ушёл.";
        let rendered = sentence_spans(text)
            .into_iter()
            .map(|span| text[span.start..span.end].to_string())
            .collect::<Vec<_>>();
        assert_eq!(rendered, vec![text.to_string()]);
    }

    #[test]
    fn sentence_spans_split_after_closed_direct_speech_when_sentence_ends() {
        let text = "Он спросил: «Что делать?» Потом ушёл.";
        let rendered = sentence_spans(text)
            .into_iter()
            .map(|span| text[span.start..span.end].to_string())
            .collect::<Vec<_>>();
        assert_eq!(rendered, vec!["Он спросил: «Что делать?»".to_string(), "Потом ушёл.".to_string()]);
    }

    #[test]
    fn detects_marker_as_sentence_initial_after_opening_quote() {
        let tokens = tokenize("«Если придёшь, позвони».");
        let marker_idx = tokens.iter().position(|token| token.text == "Если").unwrap();
        assert!(is_sentence_initial(&tokens, marker_idx));
    }

    #[test]
    fn suppresses_inside_guillemets_and_parentheses() {
        let quoted = "Он спросил: «что делать?»";
        let parenthesized = "Пометка (если возможно).";
        let plain = "Я знаю что делать.";
        assert!(is_inside_quotes_or_parentheses(quoted, quoted.find("что").unwrap()));
        assert!(is_inside_quotes_or_parentheses(parenthesized, parenthesized.find("если").unwrap()));
        assert!(!is_inside_quotes_or_parentheses(plain, plain.find("что").unwrap()));
    }

    #[test]
    fn treats_potomu_chto_as_one_marker() {
        let tokens = tokenize("Он ушёл потому что устал.");
        let markers = HashSet::from(["потому что".to_string(), "что".to_string()]);
        let start_idx = tokens.iter().position(|token| token.text == "потому").unwrap();
        let what_idx = tokens.iter().position(|token| token.text == "что").unwrap();
        let matched = find_clause_marker(&tokens, start_idx, &markers).unwrap();
        assert_eq!(matched.canonical, "потому что");
        assert_eq!(matched.kind, ClauseMarkerKind::MultiwordSubordinator);
        assert!(find_clause_marker(&tokens, what_idx, &markers).is_none());
    }

    #[test]
    fn generic_multiword_markers_require_plain_whitespace() {
        let markers = HashSet::from(["так как".to_string(), "как".to_string()]);
        let compact = tokenize("Он ушёл так как устал.");
        let start_idx = compact.iter().position(|token| token.text == "так").unwrap();
        let marker = find_clause_marker(&compact, start_idx, &markers).unwrap();
        assert_eq!(marker.canonical, "так как");

        let punctuated = tokenize("Он ушёл так, как хотел.");
        let as_idx = punctuated.iter().position(|token| token.text == "как").unwrap();
        assert_eq!(find_clause_marker(&punctuated, as_idx, &markers).unwrap().canonical, "как");
    }

    #[test]
    fn extracts_parenthetical_and_direct_speech_spans_as_safe_zones() {
        let text = "Он сказал: «если сможешь — приходи» (если нет — напиши).";
        let quote_idx = text.find("сможешь").unwrap();
        let parenthetical_idx = text.rfind("если").unwrap();
        assert_eq!(direct_speech_spans(text).len(), 1);
        assert_eq!(parenthetical_spans(text).len(), 1);
        assert!(is_inside_punctuation_safe_zone(text, quote_idx));
        assert!(is_inside_punctuation_safe_zone(text, parenthetical_idx));

        let ascii_quotes = "\"если сможешь — приходи\"";
        let ascii_quote_idx = ascii_quotes.find("сможешь").unwrap();
        assert_eq!(direct_speech_spans(ascii_quotes).len(), 1);
        assert!(is_inside_punctuation_safe_zone(ascii_quotes, ascii_quote_idx));
    }

    #[test]
    fn document_api_caches_tokens_sentences_safe_zones_and_clause_boundaries() {
        let markers = HashSet::from(["что".to_string(), "потому что".to_string()]);
        let document = SyntaxDocument::with_clause_markers("Я знаю что он ушёл потому что устал.", &markers);
        assert_eq!(document.sentences().len(), 1);
        assert!(!document.tokens().is_empty());
        assert_eq!(document.safe_zones().len(), 0);
        let boundaries = document.clause_boundaries().collect::<Vec<_>>();
        assert_eq!(boundaries.len(), 2);
        assert!(boundaries.iter().all(|boundary| boundary.kind == ClauseBoundaryKind::BeforeMarker));
    }

    #[test]
    fn classifies_clause_boundary_for_actionable_marker() {
        let text = "Я знаю что он придёт.";
        let tokens = tokenize(text);
        let markers = HashSet::from(["что".to_string()]);
        let marker_idx = tokens.iter().position(|token| token.text == "что").unwrap();
        let marker = find_clause_marker(&tokens, marker_idx, &markers).unwrap();
        let boundary = clause_boundary_for_marker(text, &tokens, marker);
        assert_eq!(boundary.kind, ClauseBoundaryKind::BeforeMarker);
        assert!(boundary.confidence.is_actionable());
    }

    #[test]
    fn weak_relative_markers_are_not_actionable_for_comma_insertion() {
        let text = "Я видел дом который построили вчера.";
        let tokens = tokenize(text);
        let markers = HashSet::from(["который".to_string()]);
        let marker_idx = tokens.iter().position(|token| token.text == "который").unwrap();
        let marker = find_clause_marker(&tokens, marker_idx, &markers).unwrap();
        assert_eq!(marker.confidence, SyntaxConfidence::Weak);
        assert!(!should_report_missing_comma_before_clause_marker(text, &tokens, &marker));
    }

    #[test]
    fn extracts_preposition_government_candidates() {
        let tokens = tokenize("Согласно приказа, встреча перенесена.");
        let prepositions = BTreeSet::from(["согласно".to_string()]);
        let candidates = preposition_government_candidates(&tokens, &prepositions);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].kind, SyntaxRelationKind::PrepositionGovernment);
        assert_eq!(candidates[0].left.text, "Согласно");
        assert_eq!(candidates[0].right.text, "приказа");
    }


    #[test]
    fn extracts_numeral_noun_candidates() {
        let tokens = tokenize("Два дома стоят рядом.");
        let candidates = adjacent_numeral_noun_candidates(&tokens);

        assert!(candidates
            .iter()
            .any(|candidate| candidate.kind == SyntaxRelationKind::NumeralNoun
                && candidate.left.text == "Два"
                && candidate.right.text == "дома"));
    }

    #[test]
    fn extracts_typed_nominal_group_candidates_with_multiple_modifiers() {
        let tokens = tokenize("Новый важный приказ вступил в силу.");
        let groups = short_nominal_group_candidates(&tokens, 3);

        assert!(groups.iter().any(|group| {
            group.modifiers.iter().map(|token| token.text).collect::<Vec<_>>() == vec!["Новый", "важный"]
                && group.head.text == "приказ"
                && group.is_actionable()
        }));
    }

    #[test]
    fn extracts_short_nominal_group_relation_candidates() {
        let prep_tokens = tokenize("Согласно новому приказа, встреча перенесена.");
        let prepositions = BTreeSet::from(["согласно".to_string()]);
        let prep_candidates = preposition_nominal_group_candidates(&prep_tokens, &prepositions);
        assert!(prep_candidates.iter().any(|candidate| {
            candidate.kind == SyntaxRelationKind::PrepositionNominalGroup
                && candidate.left.text == "Согласно"
                && candidate.right.text == "приказа"
        }));

        let quantity_tokens = tokenize("Два новых дома стояли рядом.");
        let quantity_candidates = numeral_nominal_group_candidates(&quantity_tokens);
        assert!(quantity_candidates.iter().any(|candidate| {
            candidate.kind == SyntaxRelationKind::NumeralNominalGroup
                && candidate.left.text == "Два"
                && candidate.right.text == "дома"
        }));
    }


    #[test]
    fn extracts_compound_numeral_nominal_group_candidates() {
        let tokens = tokenize("Двадцать два новых дома стояли рядом.");
        let candidates = compound_numeral_nominal_group_candidates(&tokens, 3, 3);

        assert!(candidates.iter().any(|candidate| {
            candidate
                .numeral_phrase
                .tokens
                .iter()
                .map(|token| token.text)
                .collect::<Vec<_>>()
                == vec!["Двадцать", "два"]
                && candidate.group.modifiers.iter().map(|token| token.text).collect::<Vec<_>>()
                    == vec!["новых"]
                && candidate.group.head.text == "дома"
                && candidate.is_actionable()
        }));
    }

    #[test]
    fn extracts_typed_numeral_component_slots() {
        let tokens = tokenize("Сто двадцать два новых дома стояли рядом.");
        let candidates = compound_numeral_nominal_group_candidates(&tokens, 3, 3);

        let candidate = candidates
            .iter()
            .find(|candidate| {
                candidate
                    .numeral_phrase
                    .tokens
                    .iter()
                    .map(|token| token.text)
                    .collect::<Vec<_>>()
                    == vec!["Сто", "двадцать", "два"]
                    && candidate.group.head.text == "дома"
            })
            .expect("three-component numeral phrase candidate");

        let classes = candidate
            .numeral_phrase
            .components
            .iter()
            .map(|slot| slot.component_class)
            .collect::<Vec<_>>();
        assert_eq!(
            classes,
            vec![
                NumeralComponentClass::Hundred,
                NumeralComponentClass::Decade,
                NumeralComponentClass::UnitPaucal
            ]
        );
        assert_eq!(
            candidate.numeral_phrase.governing_component().unwrap().token.text,
            "два"
        );
    }


    #[test]
    fn morphosyntax_document_exposes_preposition_case_government_conflict() {
        let lexicon = crate::morph::MorphLexicon::demo();
        let document = MorphosyntaxDocument::new("Согласно приказа, встреча перенесена.", &lexicon);

        let relation = document
            .relations_by_kind(MorphosyntacticRelationKind::PrepositionCaseGovernment)
            .find(|relation| relation.governor.token.text == "Согласно" && relation.dependent.token.text == "приказа")
            .expect("preposition government morphosyntactic relation");

        assert_eq!(relation.governor.role, MorphosyntacticRole::CaseGovernor);
        assert_eq!(relation.dependent.role, MorphosyntacticRole::GovernedNominal);
        assert!(relation.is_confident_rejection());
        assert_eq!(relation.constraint.explanation_label(), "government-conflict");
    }

    #[test]
    fn morphosyntax_document_uses_governing_component_inside_compound_numeral_phrase() {
        let lexicon = crate::morph::MorphLexicon::demo();
        let document = MorphosyntaxDocument::new("Сто двадцать два новых домов стояли рядом.", &lexicon);

        let relation = document
            .relations_by_kind(MorphosyntacticRelationKind::NumeralCaseNumberGovernment)
            .find(|relation| relation.governor.token.text == "два" && relation.dependent.token.text == "домов")
            .expect("compound numeral morphosyntactic relation");

        assert_eq!(relation.governor.role, MorphosyntacticRole::Quantifier);
        assert_eq!(relation.dependent.role, MorphosyntacticRole::QuantifiedHead);
        assert_eq!(relation.span.start, 0);
        assert_eq!(relation.constraint.explanation_label(), "quantity-conflict");
        assert!(relation.is_confident_rejection());
    }

}
