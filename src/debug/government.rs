#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernmentFrameDebugEntry {
    pub kind: String,
    pub source: String,
    pub governor: String,
    pub dependent: String,
    pub span: Span,
    pub expected_cases: Vec<crate::morph::Case>,
    pub observed_cases: Vec<crate::morph::Case>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_numbers: Vec<crate::morph::Number>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observed_numbers: Vec<crate::morph::Number>,
    pub compatibility: crate::morph::MorphCompatibility,
    pub confidence: String,
    pub blockers: Vec<String>,
    pub conflict: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_ref: Option<GovernmentFrameModelRefDebug>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernmentFrameModelRefDebug {
    pub lemma: String,
    pub complement_kind: crate::morph::VerbGovernmentComplementKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preposition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<crate::morph::SourceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl GovernmentFrameModelRefDebug {
    fn from_model_ref(model_ref: &GovernmentFrameModelRef) -> Self {
        Self {
            lemma: model_ref.lemma.clone(),
            complement_kind: model_ref.complement_kind,
            preposition: model_ref.preposition.clone(),
            source_id: model_ref.source_id.clone(),
            note: model_ref.note.clone(),
        }
    }
}

impl GovernmentFrameDebugEntry {
    fn from_frame(frame: &GovernmentFrame<'_>) -> Self {
        Self {
            kind: format!("{:?}", frame.kind),
            source: format!("{:?}", frame.source),
            governor: frame.governor.token.text.to_owned(),
            dependent: frame.dependent.token.text.to_owned(),
            span: frame.span,
            expected_cases: frame.expected_cases.clone(),
            observed_cases: frame.observed_cases.clone(),
            expected_numbers: frame.expected_numbers.clone(),
            observed_numbers: frame.observed_numbers.clone(),
            compatibility: frame.compatibility,
            confidence: format!("{:?}", frame.confidence),
            blockers: frame
                .blockers
                .iter()
                .map(|blocker| format!("{blocker:?}"))
                .collect(),
            conflict: frame.is_conflict(),
            model_ref: frame
                .model_ref
                .as_ref()
                .map(GovernmentFrameModelRefDebug::from_model_ref),
        }
    }
}
