#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FeatureUnificationStatus {
    Compatible,
    Conflict,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureUnificationStep {
    pub feature: AgreementFeatureKind,
    pub left_values: Vec<String>,
    pub right_values: Vec<String>,
    pub intersection: Vec<String>,
    pub status: FeatureUnificationStatus,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureUnification {
    pub relation: AgreementRelationKind,
    pub steps: Vec<FeatureUnificationStep>,
}

impl FeatureUnification {
    pub fn new(relation: AgreementRelationKind) -> Self {
        Self {
            relation,
            steps: Vec::new(),
        }
    }

    pub fn with_step(mut self, step: FeatureUnificationStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn compatibility(&self) -> MorphCompatibility {
        if self
            .steps
            .iter()
            .any(|step| step.status == FeatureUnificationStatus::Conflict)
        {
            MorphCompatibility::Incompatible
        } else if self
            .steps
            .iter()
            .any(|step| step.status == FeatureUnificationStatus::Unknown)
        {
            MorphCompatibility::Unknown
        } else {
            MorphCompatibility::Compatible
        }
    }

    pub fn conflicts(&self) -> Vec<AgreementConflict> {
        self.steps
            .iter()
            .filter(|step| step.status == FeatureUnificationStatus::Conflict)
            .map(|step| AgreementConflict {
                feature: step.feature,
                left: describe_values(&step.left_values),
                right: describe_values(&step.right_values),
            })
            .collect()
    }

    pub fn unknown_features(&self) -> Vec<AgreementFeatureKind> {
        let mut out = self
            .steps
            .iter()
            .filter(|step| step.status == FeatureUnificationStatus::Unknown)
            .map(|step| step.feature)
            .collect::<Vec<_>>();
        out.sort_unstable();
        out.dedup();
        out
    }

    pub fn summary(&self) -> String {
        self.steps
            .iter()
            .map(|step| {
                format!(
                    "{:?}: {} ∩ {} = {} => {:?}",
                    step.feature,
                    describe_values(&step.left_values),
                    describe_values(&step.right_values),
                    describe_values(&step.intersection),
                    step.status
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}
