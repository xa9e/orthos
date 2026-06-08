#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::{Domain, Examples, Level, RuleStatus, Severity};
    use crate::morph::{MorphAnalysis, MorphAnalyzer, MorphLexicon};
    use crate::syntax::DiagnosticProofKind;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct CountingMorphAnalyzer {
        inner: MorphLexicon,
        calls: AtomicUsize,
    }

    impl CountingMorphAnalyzer {
        fn demo() -> Self {
            Self {
                inner: MorphLexicon::demo(),
                calls: AtomicUsize::new(0),
            }
        }

        fn calls(&self) -> usize {
            self.calls.load(Ordering::SeqCst)
        }
    }

    impl MorphAnalyzer for CountingMorphAnalyzer {
        fn analyze(&self, token: &str) -> Vec<MorphAnalysis> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.inner.analyze(token)
        }
    }

    fn rule(detector: Detector) -> Rule {
        Rule {
            id: "T".to_string(),
            title: "T".to_string(),
            domain: Domain::Grammar,
            level: Level::Basic,
            status: RuleStatus::Implemented,
            severity: Severity::Warning,
            source_refs: Vec::new(),
            rationale: None,
            explanation: None,
            suggestion: None,
            requires: Vec::new(),
            tags: Vec::new(),
            rule_family: None,
            confidence: None,
            false_positive_risk: None,
            pattern: None,
            constraints: Vec::new(),
            exceptions: Vec::new(),
            evidence: Vec::new(),
            supersedes: Vec::new(),
            related_rules: Vec::new(),
            implementation_notes: None,
            examples: Examples::default(),
            detector,
        }
    }

    fn ctx<'a>(analysis: &'a AnalysisContext<'a>) -> DetectorContext<'a> {
        DetectorContext::new(analysis)
    }

    #[test]
    fn analysis_context_caches_token_views() {
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new("Привет, мир", &morph);

        assert!(!analysis.summary().tokens_cached);
        let first = analysis.tokens().as_ptr();
        let second = analysis.tokens().as_ptr();
        assert_eq!(first, second);
        assert!(analysis.summary().tokens_cached);

        let words = analysis.word_tokens();
        assert_eq!(words.len(), 2);
        assert!(analysis.summary().word_tokens_cached);
        assert!(!analysis.summary().morphosyntax_cached);

        let first_morphosyntax = analysis.morphosyntax() as *const _;
        let second_morphosyntax = analysis.morphosyntax() as *const _;
        assert_eq!(first_morphosyntax, second_morphosyntax);
        assert!(analysis.summary().morphosyntax_cached);
    }

    #[test]
    fn analysis_context_reuses_morph_analysis_per_surface_token() {
        let morph = CountingMorphAnalyzer::demo();
        let analysis = AnalysisContext::new("согласно приказа согласно приказа", &morph);

        let _ = analysis.fact_store();
        let after_fact_store = morph.calls();
        let _ = analysis.morphosyntax();

        assert_eq!(after_fact_store, 2);
        assert_eq!(morph.calls(), after_fact_store);
    }

    #[test]
    fn detects_tsya_contexts() {
        let r = rule(Detector::TsyaHeuristic {
            infinitive_triggers: vec!["надо".into()],
            finite_subjects: vec!["он".into()],
            message: "x".into(),
        });
        let text = "надо учится";
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new(text, &morph);
        assert_eq!(run_detector(&r, &ctx(&analysis)).unwrap().len(), 1);
    }

    #[test]
    fn detects_basic_pol_compound_spacing() {
        let r = rule(Detector::PolCompoundHyphenMissing { message: "x".into() });
        let text = "пол яблока и пол чайной ложки";
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new(text, &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].replacement.as_deref(), Some("пол-яблока"));
    }

    #[test]
    fn detects_basic_preposition_government() {
        let r = rule(Detector::PrepositionGovernmentBasic { message: "x".into() });
        let text = "согласно приказа";
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new(text, &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn detects_preposition_nominal_group_government_through_fact_store() {
        let r = rule(Detector::PrepositionNominalGroupGovernmentBasic { message: "x".into() });
        let text = "согласно новому приказа";
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new(text, &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].proof.as_ref().map(|proof| proof.kind),
            Some(DiagnosticProofKind::GovernmentConflict)
        );
        assert!(analysis.summary().fact_store_cached);
    }

    #[test]
    fn detects_verb_government_through_fact_store() {
        let r = rule(Detector::VerbGovernmentBasic { message: "x".into() });
        let text = "ждать ответу";
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new(text, &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].proof.as_ref().map(|proof| proof.kind),
            Some(DiagnosticProofKind::GovernmentConflict)
        );
        assert!(analysis.summary().fact_store_cached);
    }

    #[test]
    fn agreement_detector_emits_diagnostic_proof() {
        let r = rule(Detector::NominalGroupModifierAgreementBasic { message: "x".into() });
        let text = "красивый новые дома";
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new(text, &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert!(!issues.is_empty());
        assert_eq!(
            issues[0].proof.as_ref().map(|proof| proof.kind),
            Some(DiagnosticProofKind::AgreementConflict)
        );
    }


    #[test]
    fn coordination_comma_detector_uses_punctuation_slots() {
        let r = rule(Detector::CoordinationCommaBasic { message: "x".into() });
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new("быстрый точный анализатор", &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].proof.as_ref().map(|proof| proof.kind),
            Some(DiagnosticProofKind::BoundarySuppression)
        );
        assert!(analysis.summary().fact_store_cached);
    }

    #[test]
    fn document_abbreviation_detector_uses_document_context() {
        let r = rule(Detector::DocumentAbbreviationExpansion { message: "x".into() });
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new("ООН обновила доклад. ООН опубликовала приложение.", &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].proof.as_ref().map(|proof| proof.kind),
            Some(DiagnosticProofKind::DocumentAbbreviation)
        );
    }

    #[test]
    fn document_style_detector_uses_document_context() {
        let r = rule(Detector::DocumentStyleConsistency { message: "x".into() });
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new("# План\n- первый пункт\n* второй пункт", &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(
            issues[0].proof.as_ref().map(|proof| proof.kind),
            Some(DiagnosticProofKind::DocumentConsistency)
        );
    }

    #[test]
    fn detects_clitic_hyphen_missing_from_model() {
        let r = rule(Detector::CliticHyphenMissing {
            group: "emphatic_to_pronoun_seed".into(),
            message: "x".into(),
        });
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new("он то понял", &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].replacement.as_deref(), Some("он-то"));
    }

    #[test]
    fn detects_negated_verbal_form_spacing() {
        let r = rule(Detector::NegatedVerbSpacingBasic { message: "x".into() });
        let morph = MorphLexicon::demo();
        let analysis = AnalysisContext::new("он нехочет спорить", &morph);
        let issues = run_detector(&r, &ctx(&analysis)).unwrap();

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].replacement.as_deref(), Some("не хочет"));
    }

}
