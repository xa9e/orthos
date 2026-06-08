#[derive(Debug, Clone)]
pub struct ExecutionPlan<'a> {
    pub rules: Vec<&'a Rule>,
    pub skipped_rules: Vec<SkippedRule>,
}

impl<'a> ExecutionPlan<'a> {
    pub fn selected_rule_count(&self) -> usize {
        self.rules.len()
    }

    pub fn skipped_rule_count(&self) -> usize {
        self.skipped_rules.len()
    }

    pub fn summary(&self) -> ExecutionPlanSummary {
        ExecutionPlanSummary {
            selected_rule_count: self.selected_rule_count(),
            skipped_rule_count: self.skipped_rule_count(),
            selected_rules: self
                .rules
                .iter()
                .map(|rule| PlannedRuleSummary {
                    rule_id: rule.id.clone(),
                    detector_kind: rule.detector.kind().to_owned(),
                    domain: rule.domain,
                    severity: rule.severity,
                    status: rule.status,
                    required_capabilities: rule.requires.clone(),
                })
                .collect(),
            skipped_rules: self.skipped_rules.clone(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct ExecutionPlanSummary {
    pub selected_rule_count: usize,
    pub skipped_rule_count: usize,
    pub selected_rules: Vec<PlannedRuleSummary>,
    pub skipped_rules: Vec<SkippedRule>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct PlannedRuleSummary {
    pub rule_id: String,
    pub detector_kind: String,
    pub domain: Domain,
    pub severity: Severity,
    pub status: RuleStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct SkippedRule {
    pub rule_id: String,
    pub reason: SkippedRuleReason,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", content = "detail", rename_all = "snake_case")]
pub enum SkippedRuleReason {
    UnknownDetectorKind(String),
    MissingCapabilities(Vec<Capability>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckResult {
    pub issues: Vec<Issue>,
    pub execution_plan: ExecutionPlanSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timings: Option<EngineTimings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebugReport>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EngineTimings {
    pub rules: Vec<RuleTiming>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuleTiming {
    pub rule_id: String,
    pub elapsed_micros: u64,
    pub issue_count: usize,
}

#[derive(Debug)]
struct RuleExecutionOutput {
    issues: Vec<Issue>,
    timing: Option<RuleTiming>,
    debug: RuleExecutionDebug,
}

fn flatten_rule_outputs(
    outputs: Vec<RuleExecutionOutput>,
) -> (Vec<Issue>, Vec<RuleTiming>, Vec<RuleExecutionDebug>) {
    let mut issues = Vec::new();
    let mut timings = Vec::new();
    let mut debug = Vec::new();

    for output in outputs {
        issues.extend(output.issues);
        if let Some(timing) = output.timing {
            timings.push(timing);
        }
        debug.push(output.debug);
    }

    (issues, timings, debug)
}
