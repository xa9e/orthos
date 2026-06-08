pub fn check_text(corpus: Corpus, text: &str) -> Result<Vec<Issue>> {
    Checker::new(corpus).check(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::Corpus;
    use std::path::PathBuf;

    fn checker() -> Checker {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let corpus = Corpus::load_dir(root.join("rules")).expect("corpus loads");
        Checker::new(corpus)
    }

    #[test]
    fn default_profile_skips_heuristic_style_and_advanced_rules() {
        let issues = checker()
            .check("Я знаю что он придёт. Открылась свободная вакансия. красивый машина")
            .unwrap();
        assert!(!issues
            .iter()
            .any(|issue| issue.rule_id == "ru.punctuation.comma_before_subordinator_basic"));
        assert!(!issues
            .iter()
            .any(|issue| issue.rule_id == "ru.style.pleonasm_phrase_seed"));
        assert!(!issues
            .iter()
            .any(|issue| issue.rule_id == "ru.grammar.adj_noun_agreement_demo"));
    }

    #[test]
    fn strict_profile_includes_all_implemented_rules() {
        let mut options = CheckOptions::default();
        options.rule_filter.profile = Profile::Strict;
        let result = checker()
            .check_with_options("Я знаю что он придёт. Открылась свободная вакансия. красивый машина", &options)
            .unwrap();
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.rule_id == "ru.punctuation.comma_before_subordinator_basic"));
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.rule_id == "ru.style.pleonasm_phrase_seed"));
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.rule_id == "ru.grammar.adj_noun_agreement_demo"));
    }

    #[test]
    fn inline_suppression_is_opt_in() {
        let text = "Привет , мир. # orthos-disable-line ru.punctuation.no_space_before_mark";
        assert!(checker()
            .check(text)
            .unwrap()
            .iter()
            .any(|issue| issue.rule_id == "ru.punctuation.no_space_before_mark"));

        let mut options = CheckOptions::default();
        options.suppressions.inline_enabled = true;
        let result = checker().check_with_options(text, &options).unwrap();
        assert!(!result
            .issues
            .iter()
            .any(|issue| issue.rule_id == "ru.punctuation.no_space_before_mark"));
    }

    #[test]
    fn file_suppression_filters_requested_rule() {
        let mut options = CheckOptions::default();
        options
            .suppressions
            .file_rule_ids
            .insert("ru.punctuation.no_space_before_mark".to_owned());
        let result = checker()
            .check_with_options("Привет , мир !", &options)
            .unwrap();
        assert!(!result
            .issues
            .iter()
            .any(|issue| issue.rule_id == "ru.punctuation.no_space_before_mark"));
    }
}
