#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageModelDebugSnapshot {
    pub verb_government: VerbGovernmentInventoryDebug,
}

impl LanguageModelDebugSnapshot {
    fn from_options(options: &DebugOptions) -> Self {
        let registry = crate::morph::VerbGovernmentRegistry::russian_seed();
        let fixtures = crate::morph::VerbGovernmentFixtureSet::russian_seed();
        let false_positives = crate::morph::VerbGovernmentFalsePositiveFixtureSet::russian_seed();
        let entries_without_fixture = registry
            .entries()
            .into_iter()
            .map(crate::morph::VerbGovernmentKey::from_government)
            .filter(|key| !fixtures.contains_key(key))
            .take(options.limits.max_language_model_entries)
            .collect::<Vec<_>>();
        let entries = registry
            .entries()
            .into_iter()
            .take(options.limits.max_language_model_entries)
            .cloned()
            .collect::<Vec<_>>();
        Self {
            verb_government: VerbGovernmentInventoryDebug {
                entry_count: registry.len(),
                fixture_count: fixtures.len(),
                false_positive_fixture_count: false_positives.len(),
                entries_without_fixture,
                entries,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerbGovernmentInventoryDebug {
    pub entry_count: usize,
    pub fixture_count: usize,
    pub false_positive_fixture_count: usize,
    pub entries_without_fixture: Vec<crate::morph::VerbGovernmentKey>,
    pub entries: Vec<crate::morph::VerbGovernment>,
}
