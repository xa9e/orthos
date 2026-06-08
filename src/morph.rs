use crate::text::{lower_ru, uppercase_first};
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

// Stable analyzer boundary for rule code.
//
// Implementations may be a tiny in-memory lexicon, an OpenCorpora-derived
// dictionary, or an adapter over an external process. Detectors should depend
// on this trait and on typed `MorphAnalysis` values, not on storage internals.

// The implementation is split into domain-oriented source shards to keep files small
// while preserving the original module API and private helper visibility.

include!("morph/core/analyzer_and_ids.rs");
include!("morph/features/feature_set.rs");
include!("morph/features/pos_and_grammeme.rs");
include!("morph/features/categories.rs");
include!("morph/features/agreement_signature.rs");
include!("morph/features/unification_model.rs");
include!("morph/features/unification_agreement.rs");
include!("morph/features/unification_helpers.rs");
include!("morph/clitics/hyphenation.rs");
include!("morph/derivation/morpheme_model.rs");
include!("morph/derivation/inventory_macros.rs");
include!("morph/derivation/seed_inventory.rs");
include!("morph/derivation/segmenter.rs");
include!("morph/derivation/alternations.rs");
include!("morph/negation/verb_spacing.rs");
include!("morph/lexicon/in_memory.rs");
include!("morph/lexicon/cache.rs");
include!("morph/lexicon/cache_index.rs");
include!("morph/importers/importer_traits.rs");
include!("morph/importers/project_and_xml.rs");
include!("morph/importers/delimited_and_helpers.rs");
include!("morph/stress/records.rs");
include!("morph/compatibility/agreement.rs");
include!("morph/compatibility/predicate.rs");
include!("morph/compatibility/numerals.rs");
include!("morph/compatibility/preposition_government.rs");
include!("morph/compatibility/verb_government.rs");
include!("morph/compatibility/verb_government_fixtures.rs");
include!("morph/compatibility/verb_government_false_positives.rs");
include!("morph/constraints/feature_constraints.rs");
include!("morph/constraints/agreement_relations.rs");
include!("morph/constraints/government_relations.rs");
include!("morph/constraints/quantity_relations.rs");
include!("morph/derivation/tests.rs");
include!("morph/tests.rs");
