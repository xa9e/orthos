#[cfg(test)]
mod derivation_tests {
    use super::*;

    #[test]
    fn seed_model_segments_prefixed_verbs() {
        let model = RussianDerivationModel::seed();
        let parses = model.analyze_word("рассказать");
        let best = parses.first().expect("parse exists");

        assert!(best.signature().contains("Prefix:рас"));
        assert!(best.signature().contains("Root:сказ"));
        assert_ne!(best.confidence, DerivationConfidence::Low);
    }

    #[test]
    fn z_s_prefix_assimilation_suggests_expected_variant() {
        let suggestion = prefix_final_z_s_suggestion("разсказать").expect("suggestion exists");
        assert_eq!(suggestion.replacement, "рассказать");

        let suggestion = prefix_final_z_s_suggestion("расбить").expect("suggestion exists");
        assert_eq!(suggestion.replacement, "разбить");
    }

    #[test]
    fn z_s_prefix_assimilation_ignores_unmodelled_borrowed_roots() {
        assert!(prefix_final_z_s_suggestion("расист").is_none());
        assert!(prefix_final_z_s_suggestion("рассказать").is_none());
    }
}
