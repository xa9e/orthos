#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Check a file or STDIN.
    Check {
        /// Input text file. If omitted, reads STDIN.
        input: Option<PathBuf>,
        /// Directory with YAML rule corpus.
        #[arg(short, long, default_value = "rules")]
        rules: PathBuf,
        /// Optional TSV lexicon for morphology-backed rules.
        #[arg(long)]
        morph_lexicon: Option<PathBuf>,
        /// Output format.
        #[arg(short, long, default_value = "human")]
        format: Format,
        #[command(flatten)]
        selection: RuleSelectionArgs,
        #[command(flatten)]
        suppressions: SuppressionArgs,
        /// Print deterministic per-rule timings to stderr.
        #[arg(long)]
        timings: bool,
        /// Execution strategy. Parallel mode is opt-in and final output remains sorted.
        #[arg(long, value_enum, default_value = "serial")]
        execution_strategy: ExecutionStrategyArg,
    },

    /// Run checks and emit a structured analysis/debug snapshot as JSON.
    Debug {
        /// Input text file. If omitted, reads STDIN.
        input: Option<PathBuf>,
        /// Directory with YAML rule corpus.
        #[arg(short, long, default_value = "rules")]
        rules: PathBuf,
        /// Optional TSV lexicon for morphology-backed rules.
        #[arg(long)]
        morph_lexicon: Option<PathBuf>,
        #[command(flatten)]
        selection: RuleSelectionArgs,
        #[command(flatten)]
        suppressions: SuppressionArgs,
        /// Execution strategy. Parallel mode is opt-in and final output remains sorted.
        #[arg(long, value_enum, default_value = "serial")]
        execution_strategy: ExecutionStrategyArg,
    },
    /// Show the deterministic execution plan without running detectors.
    Plan {
        /// Directory with YAML rule corpus.
        #[arg(short, long, default_value = "rules")]
        rules: PathBuf,
        /// Output format.
        #[arg(short, long, default_value = "human")]
        format: Format,
        #[command(flatten)]
        selection: RuleSelectionArgs,
    },
    /// List rules from corpus.
    ListRules {
        /// Directory with YAML rule corpus.
        #[arg(short, long, default_value = "rules")]
        rules: PathBuf,
        /// Include planned and research rules. Kept for backward compatibility.
        #[arg(long)]
        all: bool,
        #[command(flatten)]
        selection: RuleSelectionArgs,
    },
    /// Validate rule corpus only.
    Validate {
        /// Directory with YAML rule corpus.
        #[arg(short, long, default_value = "rules")]
        rules: PathBuf,
    },
    /// Run executable examples embedded in selected rules.
    TestExamples {
        /// Directory with YAML rule corpus.
        #[arg(short, long, default_value = "rules")]
        rules: PathBuf,
        /// Optional TSV lexicon for morphology-backed rules.
        #[arg(long)]
        morph_lexicon: Option<PathBuf>,
        #[command(flatten)]
        selection: RuleSelectionArgs,
    },
    /// Compile a TSV lexicon into a fast binary cache.
    CompileLexicon {
        /// Input TSV lexicon path.
        #[arg(long)]
        input: PathBuf,
        /// Output binary cache path.
        #[arg(long)]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, clap::Args)]
struct RuleSelectionArgs {
    /// Rule profile: default, strict, typography-only, grammar-research.
    #[arg(long, value_enum, default_value = "default")]
    profile: ProfileArg,
    /// Restrict to domains. Repeat or comma-separate: grammar,punctuation.
    #[arg(long = "domain", value_delimiter = ',', value_parser = parse_domain)]
    domains: Vec<Domain>,
    /// Restrict to severities. Repeat or comma-separate: error,warning,info.
    #[arg(long = "severity", value_delimiter = ',', value_parser = parse_severity)]
    severities: Vec<Severity>,
    /// Restrict to rule ids. Repeat or comma-separate exact ids.
    #[arg(long = "rule-id", value_delimiter = ',')]
    include_rule_ids: Vec<String>,
    /// Exclude rule ids. Repeat or comma-separate exact ids.
    #[arg(long = "exclude-rule", value_delimiter = ',')]
    exclude_rule_ids: Vec<String>,
    /// Override profile by status/default-safety. Values: default-safe, implemented, planned, research.
    #[arg(long = "status", value_delimiter = ',', value_parser = parse_status_filter)]
    statuses: Vec<StatusFilter>,
}

#[derive(Debug, Clone, Default, clap::Args)]
struct SuppressionArgs {
    /// Enable orthos-disable-* directives embedded in text.
    #[arg(long)]
    allow_inline_suppressions: bool,
    /// Suppress a rule for the whole input file. Use exact ids or `all`; repeat or comma-separate.
    #[arg(long = "suppress-rule", value_delimiter = ',')]
    file_rule_ids: Vec<String>,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum ProfileArg {
    Default,
    Strict,
    TypographyOnly,
    GrammarResearch,
}

impl From<ProfileArg> for Profile {
    fn from(value: ProfileArg) -> Self {
        match value {
            ProfileArg::Default => Self::Default,
            ProfileArg::Strict => Self::Strict,
            ProfileArg::TypographyOnly => Self::TypographyOnly,
            ProfileArg::GrammarResearch => Self::GrammarResearch,
        }
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
enum Format {
    Human,
    Json,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum ExecutionStrategyArg {
    Serial,
    DeterministicParallel,
}

impl From<ExecutionStrategyArg> for ExecutionStrategy {
    fn from(value: ExecutionStrategyArg) -> Self {
        match value {
            ExecutionStrategyArg::Serial => Self::Serial,
            ExecutionStrategyArg::DeterministicParallel => Self::DeterministicParallel,
        }
    }
}
