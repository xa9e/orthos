#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ProofSignalKind {
    AgreementConflict,
    GovernmentConflict,
    QuantityConflict,
    BoundarySuppression,
    AmbiguitySuppression,
    DocumentConsistency,
    DocumentAbbreviation,
    DiagnosticLedger,
    SurfaceMatch,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ConfidencePolicy {
    EmitOnlyWhenAllAnalysesConflict,
    EmitWhenSingleSafeAnalysisConflicts,
    SurfaceDeterministic,
    InformationalOnly,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuleCapabilityContract {
    pub requires: BTreeSet<crate::corpus::Capability>,
    pub produces: BTreeSet<ProofSignalKind>,
    pub confidence_policy: ConfidencePolicy,
}

impl RuleCapabilityContract {
    pub fn from_rule(rule: &crate::corpus::Rule) -> Self {
        let requires = rule.requires.iter().copied().collect::<BTreeSet<_>>();
        let produces = proof_signals_for_detector(&rule.detector);
        let confidence_policy = if requires.contains(&crate::corpus::Capability::Morphology) {
            ConfidencePolicy::EmitOnlyWhenAllAnalysesConflict
        } else if requires.contains(&crate::corpus::Capability::Regex) {
            ConfidencePolicy::SurfaceDeterministic
        } else {
            ConfidencePolicy::InformationalOnly
        };
        Self {
            requires,
            produces,
            confidence_policy,
        }
    }

    pub fn requires(&self, capability: crate::corpus::Capability) -> bool {
        self.requires.contains(&capability)
    }

    pub fn produces(&self, signal: ProofSignalKind) -> bool {
        self.produces.contains(&signal)
    }
}

fn proof_signals_for_detector(detector: &crate::corpus::Detector) -> BTreeSet<ProofSignalKind> {
    let mut out = BTreeSet::new();
    match detector {
        crate::corpus::Detector::AdjNounAgreementDemo { .. }
        | crate::corpus::Detector::NominalGroupModifierAgreementBasic { .. }
        | crate::corpus::Detector::SubjectPredicateAgreementBasic { .. } => {
            out.insert(ProofSignalKind::AgreementConflict);
        }
        crate::corpus::Detector::PrepositionGovernmentBasic { .. }
        | crate::corpus::Detector::PrepositionNominalGroupGovernmentBasic { .. }
        | crate::corpus::Detector::VerbGovernmentBasic { .. } => {
            out.insert(ProofSignalKind::GovernmentConflict);
        }
        crate::corpus::Detector::NumeralNounAgreementBasic { .. }
        | crate::corpus::Detector::NumeralNominalGroupAgreementBasic { .. }
        | crate::corpus::Detector::CompoundNumeralNominalGroupAgreementBasic { .. }
        | crate::corpus::Detector::TypedCompoundNumeralNominalGroupAgreementBasic { .. } => {
            out.insert(ProofSignalKind::QuantityConflict);
        }
        crate::corpus::Detector::MissingCommaBeforeSubordinator { .. }
        | crate::corpus::Detector::IntroductoryPhraseComma { .. }
        | crate::corpus::Detector::CoordinationCommaBasic { .. } => {
            out.insert(ProofSignalKind::BoundarySuppression);
        }
        crate::corpus::Detector::DocumentAbbreviationExpansion { .. } => {
            out.insert(ProofSignalKind::DocumentAbbreviation);
            out.insert(ProofSignalKind::DiagnosticLedger);
        }
        crate::corpus::Detector::DocumentStyleConsistency { .. } => {
            out.insert(ProofSignalKind::DocumentConsistency);
            out.insert(ProofSignalKind::DiagnosticLedger);
        }
        _ => {
            out.insert(ProofSignalKind::SurfaceMatch);
        }
    }
    out
}
