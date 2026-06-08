#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FeatureConstraintSet {
    pub cases: BTreeSet<Case>,
    pub numbers: BTreeSet<Number>,
    pub genders: BTreeSet<Gender>,
}

impl FeatureConstraintSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn require_case(mut self, case: Case) -> Self {
        self.cases.insert(case);
        self
    }

    pub fn require_cases(mut self, cases: impl IntoIterator<Item = Case>) -> Self {
        self.cases.extend(cases);
        self
    }

    pub fn allows_case(&self, case: Case) -> MorphCompatibility {
        if self.cases.is_empty() {
            MorphCompatibility::Unknown
        } else if self.cases.contains(&case) {
            MorphCompatibility::Compatible
        } else {
            MorphCompatibility::Incompatible
        }
    }

    pub fn describe_cases(&self) -> String {
        self.cases
            .iter()
            .map(|case| format!("{case:?}"))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
