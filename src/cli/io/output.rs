fn checker_from_paths_for_text(
    rules: PathBuf,
    morph_lexicon: Option<PathBuf>,
    text: &str,
) -> Result<Checker> {
    let corpus = Corpus::load_dir(&rules)?;
    let forms = morph_forms_for_text(text);
    checker_from_corpus_and_morph(corpus, morph_lexicon, Some(&forms))
}

fn checker_from_corpus_and_morph(
    corpus: Corpus,
    morph_lexicon: Option<PathBuf>,
    requested_forms: Option<&BTreeSet<String>>,
) -> Result<Checker> {
    let Some(path) = morph_lexicon else {
        let default_cache = std::path::Path::new("data/lexicon/opencorpora.bincache");
        let morph = load_default_morph_lexicon(default_cache, requested_forms);
        return Ok(Checker::with_morph_lexicon(corpus, morph));
    };
    let morph = load_morph_lexicon(path, requested_forms)?;
    Ok(Checker::with_morph_lexicon(corpus, morph))
}

fn load_default_morph_lexicon(
    cache_path: &std::path::Path,
    requested_forms: Option<&BTreeSet<String>>,
) -> MorphLexicon {
    if cache_path.exists() {
        match load_morph_lexicon(cache_path.to_path_buf(), requested_forms) {
            Ok(morph) => return morph,
            Err(err) => eprintln!(
                "warning: failed to load default morph cache {} ({err:#}); falling back to bundled demo lexicon",
                cache_path.display()
            ),
        }
    }
    MorphLexicon::demo()
}

fn load_morph_lexicon(
    path: PathBuf,
    requested_forms: Option<&BTreeSet<String>>,
) -> Result<MorphLexicon> {
    if path.extension().is_some_and(|ext| ext == "bincache") {
        let loaded = if let Some(forms) = requested_forms {
            MorphLexicon::load_cache_for_forms(&path, forms)
        } else {
            MorphLexicon::load_cache(&path)
        };
        loaded.with_context(|| format!("failed to load morph cache {}", path.display()))
    } else {
        MorphLexicon::load_tsv(&path)
            .with_context(|| format!("failed to load morph lexicon {}", path.display()))
    }
}

fn morph_forms_for_text(text: &str) -> BTreeSet<String> {
    let mut forms = BTreeSet::new();
    for token in tokenize(text) {
        if !matches!(token.kind, TokenKind::Word | TokenKind::Number) {
            continue;
        }

        let normalized = lower_ru(token.text);
        if normalized.is_empty() {
            continue;
        }
        if let Some(base) = normalized.strip_prefix("не")
            && base.chars().count() >= 2
        {
            forms.insert(base.to_owned());
        }
        forms.insert(normalized);
    }
    forms
}

fn read_input(input: Option<PathBuf>) -> Result<String> {
    match input {
        Some(path) => fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display())),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).context("failed to read STDIN")?;
            Ok(buf)
        }
    }
}

fn print_plan_summary(summary: &ExecutionPlanSummary) {
    println!("Selected rules: {}", summary.selected_rule_count);
    println!("Skipped rules: {}", summary.skipped_rule_count);

    if !summary.selected_rules.is_empty() {
        println!("Selected:");
        for rule in &summary.selected_rules {
            println!(
                "  {}\t{}\t{}\t{}\t{}\t{}",
                rule.rule_id,
                rule.domain,
                rule.severity,
                rule.status,
                rule.detector_kind,
                join_capabilities(&rule.required_capabilities)
            );
        }
    }

    if !summary.skipped_rules.is_empty() {
        println!("Skipped:");
        for skipped in &summary.skipped_rules {
            println!(
                "  {}\t{}",
                skipped.rule_id,
                format_skipped_reason(&skipped.reason)
            );
        }
    }
}

fn print_human_issues(issues: &[orthos::Issue]) {
    if issues.is_empty() {
        println!("No issues found.");
        return;
    }

    for issue in issues {
        println!(
            "{}:{}:{} [{}] {}: {}",
            issue.start.line,
            issue.start.column,
            issue.end.column,
            issue.rule_id,
            issue.severity,
            issue.message
        );
        if let Some(replacement) = &issue.replacement {
            println!("  replacement: {replacement}");
        }
        if let Some(suggestion) = &issue.suggestion {
            println!("  suggestion: {suggestion}");
        }
        if let Some(explanation) = &issue.explanation {
            println!("  explanation: {explanation}");
        }
        if !issue.source_refs.is_empty() {
            println!("  source_refs: {}", issue.source_refs.join(", "));
        }
        println!("  excerpt: {}", issue.excerpt);
    }
}

#[cfg(test)]
mod cli_output_tests {
    use super::*;

    #[test]
    fn invalid_default_cache_falls_back_to_demo_lexicon() {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "orthos-invalid-cache-{}-{}.bincache",
            std::process::id(),
            "fallback"
        ));
        fs::write(&path, b"bad").expect("invalid cache fixture is written");

        let lexicon = load_default_morph_lexicon(&path, None);
        let _ = fs::remove_file(&path);

        assert!(!lexicon.analyze("девочка").is_empty());
    }
}
