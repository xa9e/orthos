use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use orthos::text::{TokenKind, lower_ru, tokenize};
use orthos::{
    CheckOptions, Checker, Corpus, DebugOptions, Domain, ExecutionPlanSummary, ExecutionStrategy,
    MorphLexicon, Profile, Rule, Severity, SkippedRuleReason, StatusFilter, SuppressionOptions,
};
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

// The implementation is split into domain-oriented source shards to keep files small
// while preserving the original module API and private helper visibility.

include!("cli/args.rs");
include!("cli/run.rs");
include!("cli/commands/rules.rs");
include!("cli/io/output.rs");
include!("cli/options.rs");
