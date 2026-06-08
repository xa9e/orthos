impl Rule {
    /// Conservative profile gate used by the default CLI/editor profile.
    ///
    /// This deliberately excludes research/planned rules, style-only rules,
    /// advanced demo rules, and heuristic punctuation rules unless the caller
    /// opts into a stricter or research profile.
    pub fn is_default_safe(&self) -> bool {
        if self.status != RuleStatus::Implemented {
            return false;
        }
        if matches!(self.domain, Domain::Style) {
            return false;
        }
        if matches!(self.level, Level::Advanced | Level::Expert | Level::Heuristic) {
            return false;
        }
        !self.tags.iter().any(|tag| {
            matches!(
                tag.as_str(),
                "demo" | "experimental" | "research" | "roadmap" | "strict-only"
            )
        })
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Orthography => "orthography",
            Self::Punctuation => "punctuation",
            Self::Grammar => "grammar",
            Self::Style => "style",
            Self::Typography => "typography",
        })
    }
}

impl FromStr for Domain {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.trim().replace('-', "_").as_str() {
            "orthography" => Ok(Self::Orthography),
            "punctuation" => Ok(Self::Punctuation),
            "grammar" => Ok(Self::Grammar),
            "style" => Ok(Self::Style),
            "typography" => Ok(Self::Typography),
            other => Err(format!("unknown domain `{other}`")),
        }
    }
}

impl fmt::Display for RuleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Implemented => "implemented",
            Self::Planned => "planned",
            Self::Research => "research",
        })
    }
}

impl FromStr for RuleStatus {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.trim().replace('-', "_").as_str() {
            "implemented" => Ok(Self::Implemented),
            "planned" => Ok(Self::Planned),
            "research" => Ok(Self::Research),
            other => Err(format!("unknown rule status `{other}`")),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        })
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value.trim().replace('-', "_").as_str() {
            "info" => Ok(Self::Info),
            "warning" => Ok(Self::Warning),
            "error" => Ok(Self::Error),
            other => Err(format!("unknown severity `{other}`")),
        }
    }
}
