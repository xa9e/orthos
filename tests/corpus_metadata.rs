use orthos::Corpus;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_rules_dir(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("orthos-{test_name}-{nanos}"));
    fs::create_dir_all(&dir).expect("create temp rules dir");
    dir
}

fn write_rules(dir: &Path, yaml: &str) {
    fs::write(dir.join("rules.yaml"), yaml).expect("write rule fixture");
}

fn load_error(dir: &Path) -> String {
    Corpus::load_dir(dir)
        .expect_err("fixture must be rejected")
        .to_string()
}

// Large metadata regression suites are sharded by validation concern.

include!("corpus_metadata/identity_and_sources.rs");
include!("corpus_metadata/extended_metadata.rs");
include!("corpus_metadata/examples_and_confidence.rs");
include!("corpus_metadata/rule_ids_patterns_sources.rs");
