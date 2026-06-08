use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use walkdir::WalkDir;

// The implementation is split into domain-oriented source shards to keep files small
// while preserving the original module API and private helper visibility.

include!("corpus/loading.rs");
include!("corpus/rules/rule_model_validation.rs");
include!("corpus/rules/capability_inference.rs");
include!("corpus/rules/display_parse.rs");
include!("corpus/model/supporting_models.rs");
include!("corpus/model/enums.rs");
include!("corpus/model/capability_detector.rs");
