#[derive(Debug, Clone, Default)]
pub struct CheckOptions {
    pub rule_filter: RuleFilter,
    pub suppressions: SuppressionOptions,
    pub collect_timings: bool,
    pub execution_strategy: ExecutionStrategy,
    pub debug: DebugOptions,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStrategy {
    #[default]
    Serial,
    DeterministicParallel,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum Profile {
    #[default]
    Default,
    Strict,
    TypographyOnly,
    GrammarResearch,
}

impl Profile {
    pub fn matches(self, rule: &Rule) -> bool {
        match self {
            Self::Default => rule.is_default_safe(),
            Self::Strict => rule.status == RuleStatus::Implemented,
            Self::TypographyOnly => {
                rule.status == RuleStatus::Implemented && rule.domain == Domain::Typography
            }
            Self::GrammarResearch => {
                rule.status == RuleStatus::Research && rule.domain == Domain::Grammar
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RuleFilter {
    pub profile: Profile,
    pub domains: BTreeSet<Domain>,
    pub severities: BTreeSet<Severity>,
    pub include_rule_ids: BTreeSet<String>,
    pub exclude_rule_ids: BTreeSet<String>,
    pub statuses: BTreeSet<StatusFilter>,
}

impl RuleFilter {
    pub fn matches(&self, rule: &Rule) -> bool {
        if self.exclude_rule_ids.contains(&rule.id) {
            return false;
        }

        let selected_by_id = !self.include_rule_ids.is_empty();
        if selected_by_id && !self.include_rule_ids.contains(&rule.id) {
            return false;
        }

        if !self.domains.is_empty() && !self.domains.contains(&rule.domain) {
            return false;
        }
        if !self.severities.is_empty() && !self.severities.contains(&rule.severity) {
            return false;
        }

        if !self.statuses.is_empty() {
            return self.statuses.iter().any(|status| status.matches(rule));
        }

        selected_by_id || self.profile.matches(rule)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum StatusFilter {
    DefaultSafe,
    Implemented,
    Planned,
    Research,
}

impl StatusFilter {
    pub fn matches(self, rule: &Rule) -> bool {
        match self {
            Self::DefaultSafe => rule.is_default_safe(),
            Self::Implemented => rule.status == RuleStatus::Implemented,
            Self::Planned => rule.status == RuleStatus::Planned,
            Self::Research => rule.status == RuleStatus::Research,
        }
    }
}
