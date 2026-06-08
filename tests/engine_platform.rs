use orthos::{
    Capability, CapabilityRegistry, CheckOptions, Checker, Corpus, DetectorRegistry, Domain,
    ExecutionStrategy, Profile, Severity, SkippedRuleReason, StatusFilter, SuppressionOptions,
    default_detector_registry,
};
use pretty_assertions::assert_eq;
use serde_json::Value;
use std::path::PathBuf;

fn corpus() -> Corpus {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Corpus::load_dir(root.join("rules")).expect("corpus loads")
}

fn checker() -> Checker {
    Checker::new(corpus())
}

fn strict_options() -> CheckOptions {
    let mut options = CheckOptions::default();
    options.rule_filter.profile = Profile::Strict;
    options
}

// Engine platform tests are sharded by subsystem while sharing fixture helpers.

include!("engine_platform/registry.rs");
include!("engine_platform/filtering.rs");
include!("engine_platform/suppression_and_contracts.rs");
