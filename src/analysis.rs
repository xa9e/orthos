//! Per-document analysis cache shared by detectors.
//!
//! The checker builds one `AnalysisContext` per input text. Detectors receive a
//! lightweight `DetectorContext` view and should reuse these cached layers
//! instead of retokenizing the same document independently.

use crate::morph::{MorphAnalysis, MorphAnalyzer};
use crate::syntax::{LinguisticFactStore, MorphosyntaxDocument};
use crate::text::{LineIndex, Token, TokenKind, tokenize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

pub struct AnalysisContext<'a> {
    text: &'a str,
    line_index: LineIndex<'a>,
    token_cache: OnceLock<Vec<Token<'a>>>,
    word_token_cache: OnceLock<Vec<Token<'a>>>,
    morph_analysis_cache: OnceLock<TokenMorphAnalysisCache>,
    morphosyntax_cache: OnceLock<MorphosyntaxDocument<'a>>,
    fact_store_cache: OnceLock<LinguisticFactStore<'a>>,
    morph: &'a dyn MorphAnalyzer,
}

impl std::fmt::Debug for AnalysisContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalysisContext")
            .field("text_bytes", &self.text.len())
            .field("line_count", &self.line_index.line_count())
            .field("tokens_cached", &self.token_cache.get().is_some())
            .field("word_tokens_cached", &self.word_token_cache.get().is_some())
            .field(
                "morph_analyses_cached",
                &self.morph_analysis_cache.get().is_some(),
            )
            .field(
                "morphosyntax_cached",
                &self.morphosyntax_cache.get().is_some(),
            )
            .field("fact_store_cached", &self.fact_store_cache.get().is_some())
            .finish()
    }
}

impl<'a> AnalysisContext<'a> {
    pub fn new(text: &'a str, morph: &'a dyn MorphAnalyzer) -> Self {
        Self {
            text,
            line_index: LineIndex::new(text),
            token_cache: OnceLock::new(),
            word_token_cache: OnceLock::new(),
            morph_analysis_cache: OnceLock::new(),
            morphosyntax_cache: OnceLock::new(),
            fact_store_cache: OnceLock::new(),
            morph,
        }
    }

    pub fn text(&self) -> &'a str {
        self.text
    }

    pub fn line_index(&self) -> &LineIndex<'a> {
        &self.line_index
    }

    pub fn morph(&self) -> &'a dyn MorphAnalyzer {
        self.morph
    }

    pub fn tokens(&self) -> &[Token<'a>] {
        self.token_cache
            .get_or_init(|| tokenize(self.text))
            .as_slice()
    }

    pub fn word_tokens(&self) -> &[Token<'a>] {
        self.word_token_cache
            .get_or_init(|| {
                self.tokens()
                    .iter()
                    .filter(|token| token.kind == TokenKind::Word)
                    .cloned()
                    .collect()
            })
            .as_slice()
    }

    pub fn morph_analyses(&self) -> &TokenMorphAnalysisCache {
        self.morph_analysis_cache
            .get_or_init(|| TokenMorphAnalysisCache::from_tokens(self.tokens(), self.morph))
    }

    pub fn analyses_for_token_index(&self, token_index: usize) -> &[MorphAnalysis] {
        self.morph_analyses().analyses_for_token_index(token_index)
    }

    pub fn morphosyntax(&self) -> &MorphosyntaxDocument<'a> {
        self.morphosyntax_cache.get_or_init(|| {
            MorphosyntaxDocument::from_tokens_with_analyses(self.tokens(), |idx| {
                self.analyses_for_token_index(idx).to_vec()
            })
        })
    }

    pub fn fact_store(&self) -> &LinguisticFactStore<'a> {
        self.fact_store_cache.get_or_init(|| {
            LinguisticFactStore::from_tokens_with_analyses(self.text, self.tokens(), |idx| {
                self.analyses_for_token_index(idx).to_vec()
            })
        })
    }

    pub fn summary(&self) -> AnalysisContextSummary {
        AnalysisContextSummary {
            text_bytes: self.text.len(),
            line_count: self.line_index.line_count(),
            tokens_cached: self.token_cache.get().is_some(),
            word_tokens_cached: self.word_token_cache.get().is_some(),
            morph_analyses_cached: self.morph_analysis_cache.get().is_some(),
            morphosyntax_cached: self.morphosyntax_cache.get().is_some(),
            fact_store_cached: self.fact_store_cache.get().is_some(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TokenMorphAnalysisCache {
    analyses_by_token: Vec<Option<Arc<[MorphAnalysis]>>>,
}

impl TokenMorphAnalysisCache {
    pub fn from_tokens(tokens: &[Token<'_>], morph: &dyn MorphAnalyzer) -> Self {
        let mut analyses_by_token = Vec::with_capacity(tokens.len());
        let mut by_surface: HashMap<&str, Arc<[MorphAnalysis]>> = HashMap::new();

        for token in tokens {
            if !matches!(token.kind, TokenKind::Word | TokenKind::Number) {
                analyses_by_token.push(None);
                continue;
            }

            let analyses = by_surface
                .entry(token.text)
                .or_insert_with(|| Arc::from(morph.analyze(token.text).into_boxed_slice()))
                .clone();
            analyses_by_token.push(Some(analyses));
        }

        Self { analyses_by_token }
    }

    pub fn analyses_for_token_index(&self, token_index: usize) -> &[MorphAnalysis] {
        self.analyses_by_token
            .get(token_index)
            .and_then(Option::as_deref)
            .unwrap_or(&[])
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct AnalysisContextSummary {
    pub text_bytes: usize,
    pub line_count: usize,
    pub tokens_cached: bool,
    pub word_tokens_cached: bool,
    #[serde(default)]
    pub morph_analyses_cached: bool,
    pub morphosyntax_cached: bool,
    #[serde(default)]
    pub fact_store_cached: bool,
}
