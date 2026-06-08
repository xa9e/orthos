fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Check {
            input,
            rules,
            morph_lexicon,
            format,
            selection,
            suppressions,
            timings,
            execution_strategy,
        } => check_cmd(
            input,
            rules,
            morph_lexicon,
            format,
            selection,
            suppressions,
            timings,
            execution_strategy.into(),
        ),
        Command::Debug {
            input,
            rules,
            morph_lexicon,
            selection,
            suppressions,
            execution_strategy,
        } => debug_cmd(
            input,
            rules,
            morph_lexicon,
            selection,
            suppressions,
            execution_strategy.into(),
        ),
        Command::Plan { rules, format, selection } => plan_cmd(rules, format, selection),
        Command::ListRules { rules, all, selection } => list_rules_cmd(rules, all, selection),
        Command::Validate { rules } => validate_cmd(rules),
        Command::TestExamples {
            rules,
            morph_lexicon,
            selection,
        } => test_examples_cmd(rules, morph_lexicon, selection),
        Command::CompileLexicon { input, output } => compile_lexicon_cmd(input, output),
    }
}

#[allow(clippy::too_many_arguments)]
fn check_cmd(
    input: Option<PathBuf>,
    rules: PathBuf,
    morph_lexicon: Option<PathBuf>,
    format: Format,
    selection: RuleSelectionArgs,
    suppressions: SuppressionArgs,
    timings: bool,
    execution_strategy: ExecutionStrategy,
) -> Result<()> {
    let text = read_input(input)?;
    let checker = checker_from_paths_for_text(rules, morph_lexicon, &text)?;
    let mut options = check_options(selection, suppressions);
    options.collect_timings = timings;
    options.execution_strategy = execution_strategy;
    let result = checker.check_with_options(&text, &options)?;

    match format {
        Format::Json => println!("{}", serde_json::to_string_pretty(&result.issues)?),
        Format::Human => print_human_issues(&result.issues),
    }

    if let Some(timings) = result.timings {
        eprintln!("Rule timings (microseconds, deterministic rule-id order):");
        for timing in timings.rules {
            eprintln!(
                "  {}\t{}\t{} issues",
                timing.rule_id, timing.elapsed_micros, timing.issue_count
            );
        }
    }

    Ok(())
}

fn debug_cmd(
    input: Option<PathBuf>,
    rules: PathBuf,
    morph_lexicon: Option<PathBuf>,
    selection: RuleSelectionArgs,
    suppressions: SuppressionArgs,
    execution_strategy: ExecutionStrategy,
) -> Result<()> {
    let text = read_input(input)?;
    let checker = checker_from_paths_for_text(rules, morph_lexicon, &text)?;
    let mut options = check_options(selection, suppressions);
    options.collect_timings = true;
    options.execution_strategy = execution_strategy;
    options.debug = DebugOptions::enabled();
    let result = checker.check_with_options(&text, &options)?;

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn compile_lexicon_cmd(input: PathBuf, output: PathBuf) -> Result<()> {
    eprintln!("Loading TSV lexicon from {}...", input.display());
    let lexicon = MorphLexicon::load_tsv(&input)?;
    eprintln!("Loaded {} entries, writing cache to {}...", lexicon.len(), output.display());
    lexicon.save_cache(&output)?;
    eprintln!("Done.");
    Ok(())
}
