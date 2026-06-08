use crate::analysis::{AnalysisContext, AnalysisContextSummary};
use crate::engine::{ExecutionPlanSummary, ExecutionStrategy};
use crate::issue::Span;
use crate::morph::MorphAnalysis;
use crate::syntax::{GovernmentFrame, GovernmentFrameModelRef, LinguisticFactStore};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DebugOptions {
    pub enabled: bool,
    pub include_tokens: bool,
    pub include_morphology: bool,
    pub include_fact_store: bool,
    pub include_language_model: bool,
    pub limits: DebugLimits,
}

impl DebugOptions {
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            include_tokens: true,
            include_morphology: true,
            include_fact_store: true,
            include_language_model: true,
            limits: DebugLimits::default(),
        }
    }
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            include_tokens: false,
            include_morphology: false,
            include_fact_store: false,
            include_language_model: false,
            limits: DebugLimits::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DebugLimits {
    pub max_tokens: usize,
    pub max_analyses_per_token: usize,
    pub max_fact_items: usize,
    pub max_language_model_entries: usize,
}

impl Default for DebugLimits {
    fn default() -> Self {
        Self {
            max_tokens: 256,
            max_analyses_per_token: 8,
            max_fact_items: 256,
            max_language_model_entries: 256,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugReport {
    pub schema_version: u32,
    pub analysis: AnalysisDebugSnapshot,
    pub engine: EngineDebugSnapshot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_model: Option<LanguageModelDebugSnapshot>,
}

impl DebugReport {
    pub fn from_analysis(
        analysis: &AnalysisContext<'_>,
        summary_before: AnalysisContextSummary,
        engine: EngineDebugSnapshot,
        options: &DebugOptions,
    ) -> Option<Self> {
        if !options.enabled {
            return None;
        }
        Some(Self {
            schema_version: 4,
            analysis: AnalysisDebugSnapshot::from_analysis(analysis, summary_before, options),
            engine,
            language_model: options
                .include_language_model
                .then(|| LanguageModelDebugSnapshot::from_options(options)),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDebugSnapshot {
    pub summary_before: AnalysisContextSummary,
    pub summary_after: AnalysisContextSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<TokenDebugEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphology: Vec<TokenMorphDebugEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_store: Option<FactStoreDebugSnapshot>,
}

impl AnalysisDebugSnapshot {
    fn from_analysis(
        analysis: &AnalysisContext<'_>,
        summary_before: AnalysisContextSummary,
        options: &DebugOptions,
    ) -> Self {
        let tokens = options
            .include_tokens
            .then(|| token_debug_entries(analysis, options))
            .unwrap_or_default();
        let morphology = options
            .include_morphology
            .then(|| morphology_debug_entries(analysis, options))
            .unwrap_or_default();
        let fact_store = options
            .include_fact_store
            .then(|| FactStoreDebugSnapshot::from_store(analysis.fact_store(), options));
        Self {
            summary_before,
            summary_after: analysis.summary(),
            tokens,
            morphology,
            fact_store,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDebugEntry {
    pub index: usize,
    pub kind: String,
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMorphDebugEntry {
    pub token_index: usize,
    pub token: String,
    pub analyses: Vec<MorphAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactStoreDebugSnapshot {
    pub summary: crate::syntax::LinguisticFactStoreSummary,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clause_boundaries: Vec<ClauseBoundaryDebugEntry>,
    pub government_frames: Vec<GovernmentFrameDebugEntry>,
}

impl FactStoreDebugSnapshot {
    fn from_store(store: &LinguisticFactStore<'_>, options: &DebugOptions) -> Self {
        Self {
            summary: store.summary(),
            clause_boundaries: store
                .clause_boundaries()
                .boundaries()
                .iter()
                .take(options.limits.max_fact_items)
                .map(ClauseBoundaryDebugEntry::from_boundary)
                .collect(),
            government_frames: store
                .government_frames()
                .iter()
                .take(options.limits.max_fact_items)
                .map(GovernmentFrameDebugEntry::from_frame)
                .collect(),
        }
    }
}

include!("debug/clause.rs");
include!("debug/government.rs");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineDebugSnapshot {
    pub execution_strategy: ExecutionStrategy,
    pub execution_plan: ExecutionPlanSummary,
    pub rule_outputs: Vec<RuleExecutionDebug>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuleExecutionDebug {
    pub rule_id: String,
    pub detector_kind: String,
    pub raw_issue_count: usize,
    pub suppressed_issue_count: usize,
    pub emitted_issue_count: usize,
}

include!("debug/language_model.rs");

fn token_debug_entries(
    analysis: &AnalysisContext<'_>,
    options: &DebugOptions,
) -> Vec<TokenDebugEntry> {
    analysis
        .tokens()
        .iter()
        .take(options.limits.max_tokens)
        .enumerate()
        .map(|(index, token)| TokenDebugEntry {
            index,
            kind: format!("{:?}", token.kind),
            text: token.text.to_owned(),
            span: token.span,
        })
        .collect()
}

fn morphology_debug_entries(
    analysis: &AnalysisContext<'_>,
    options: &DebugOptions,
) -> Vec<TokenMorphDebugEntry> {
    analysis
        .tokens()
        .iter()
        .take(options.limits.max_tokens)
        .enumerate()
        .filter_map(|(token_index, token)| {
            let analyses = analysis
                .analyses_for_token_index(token_index)
                .iter()
                .take(options.limits.max_analyses_per_token)
                .cloned()
                .collect::<Vec<_>>();
            (!analyses.is_empty()).then(|| TokenMorphDebugEntry {
                token_index,
                token: token.text.to_owned(),
                analyses,
            })
        })
        .collect()
}
