//! Conservative document-level syntax primitives for punctuation-aware rules.
//!
//! This module is intentionally shallow: it does not pretend to parse Russian.
//! It centralizes reusable scanning, segmentation and suppression logic so
//! punctuation detectors do not grow into one-off regex soup.

use crate::issue::Span;
use crate::text::{
    Token, TokenKind, is_sentence_boundary, lower_ru, next_non_ws, previous_non_ws, tokenize,
};
use std::collections::{BTreeSet, HashSet};

// The implementation is split into domain-oriented source shards to keep files small
// while preserving the original module API and private helper visibility.

include!("syntax/model.rs");
include!("syntax/sentences/windows.rs");
include!("syntax/relations/model.rs");
include!("syntax/proof/model.rs");
include!("syntax/ambiguity/model.rs");
include!("syntax/islands/model.rs");
include!("syntax/nominal_groups/model.rs");
include!("syntax/quantities/model.rs");
include!("syntax/morphosyntax/core.rs");
include!("syntax/morphosyntax/agreement.rs");
include!("syntax/morphosyntax/builders.rs");
include!("syntax/agreement_graph/model.rs");
include!("syntax/coordination/model.rs");
include!("syntax/coordination/inference.rs");
include!("syntax/coordination/builders.rs");
include!("syntax/punctuation/slot_model.rs");
include!("syntax/punctuation/slot_evidence.rs");
include!("syntax/punctuation/coordination_slots.rs");
include!("syntax/punctuation/slots.rs");
include!("syntax/document_context/model.rs");
include!("syntax/document_context/terms.rs");
include!("syntax/document_context/abbreviations.rs");
include!("syntax/document_context/glossary.rs");
include!("syntax/document_context/structure.rs");
include!("syntax/government_frames/model.rs");
include!("syntax/government_frames/verb.rs");
include!("syntax/government_frames/verb_case.rs");
include!("syntax/proof/builders.rs");
include!("syntax/diagnostic_ledger/model.rs");
include!("syntax/clauses/candidates.rs");
include!("syntax/constructions/model.rs");
include!("syntax/rule_contracts/model.rs");
include!("syntax/facts/store.rs");
include!("syntax/punctuation/safe_zones.rs");
include!("syntax/clauses/boundaries.rs");
include!("syntax/sentences/segmentation_helpers.rs");
include!("syntax/clauses/markers.rs");
include!("syntax/clauses/link_map.rs");
include!("syntax/tests.rs");
include!("syntax/tests_linguistic_layers.rs");
include!("syntax/tests_verb_government.rs");
include!("syntax/tests_diagnostic_ledger.rs");
