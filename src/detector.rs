use crate::analysis::AnalysisContext;
use crate::corpus::{Capability, Detector, Rule};
use crate::issue::{Issue, Span};
use crate::morph::{
    CliticHyphenGroup, MorphAnalyzer, PartOfSpeech, RussianCliticModel,
    prefix_final_z_s_suggestion, split_negated_verb_candidate,
};
use crate::syntax::{
    AgreementGraphEdgeKind, ClauseBoundaryKind, GovernmentFrame, GovernmentFrameKind,
    LinguisticFactStore, MorphosyntaxDocument, PunctuationMark, SyntaxDocument,
    agreement_edge_proof, document_abbreviation_candidates, document_abbreviation_proof,
    document_style_candidates, document_style_proof, government_frame_proof,
    is_inside_punctuation_safe_zone, punctuation_slot_proof, sentence_spans,
};
use crate::text::{
    LineIndex, Token, TokenKind, char_after, char_before, excerpt, has_cyrillic, has_latin,
    is_punctuation_requiring_space_after, lower_ru, next_non_ws_char, normalize_word,
    uppercase_first,
};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

static MULTI_SPACE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^\S\r\n]{2,}").unwrap());
static HYPHENATED_PARTICLE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?iu)\b([а-яё]+)-(же|бы|ли)\b").unwrap());
static DEFAULT_DETECTOR_REGISTRY: Lazy<DetectorRegistry> = Lazy::new(DetectorRegistry::default);

// The implementation is split into domain-oriented source shards to keep files small
// while preserving the original module API and private helper visibility.

include!("detector/registry.rs");
include!("detector/dispatch.rs");
include!("detector/dispatch/grammar.rs");
include!("detector/dispatch/orthography.rs");
include!("detector/surface/spacing_and_sentence.rs");
include!("detector/syntax/punctuation.rs");
include!("detector/syntax/document.rs");
include!("detector/grammar/agreement.rs");
include!("detector/grammar/government.rs");
include!("detector/grammar/quantity.rs");
include!("detector/orthography/token_maps.rs");
include!("detector/orthography/clitics.rs");
include!("detector/typography/typography_and_agreement.rs");
include!("detector/word_formation/pol_compounds.rs");
include!("detector/support/helpers.rs");
include!("detector/tests.rs");
