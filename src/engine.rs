use crate::analysis::AnalysisContext;
use crate::corpus::{Capability, Corpus, Domain, Rule, RuleStatus, Severity};
use crate::debug::{DebugOptions, DebugReport, EngineDebugSnapshot, RuleExecutionDebug};
use crate::detector::{DetectorContext, DetectorRegistry};
use crate::issue::Issue;
use crate::morph::{MorphAnalyzer, MorphLexicon};
use crate::text::LineIndex;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::sync::Arc;
use std::time::Instant;

// The implementation is split into domain-oriented source shards to keep files small
// while preserving the original module API and private helper visibility.

include!("engine/checker.rs");
include!("engine/builder_capabilities.rs");
include!("engine/planning_results.rs");
include!("engine/options.rs");
include!("engine/suppression.rs");
include!("engine/tests.rs");
