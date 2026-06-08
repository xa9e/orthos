#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CliticPosition {
    Prefix,
    Suffix,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CliticHyphenGroup {
    IndefinitePronominal,
    EmphaticToPronounSeed,
    ImperativeKaSeed,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CliticHyphenSuggestion {
    pub replacement: String,
    pub group: CliticHyphenGroup,
    pub position: CliticPosition,
}

pub struct RussianCliticModel;

impl CliticHyphenGroup {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match raw {
            "indefinite_pronominal" => Self::IndefinitePronominal,
            "emphatic_to_pronoun_seed" => Self::EmphaticToPronounSeed,
            "imperative_ka_seed" => Self::ImperativeKaSeed,
            _ => return None,
        })
    }
}

impl RussianCliticModel {
    pub fn suggest_missing_hyphen(
        base: &str,
        particle: &str,
        group: CliticHyphenGroup,
    ) -> Option<CliticHyphenSuggestion> {
        let base_norm = lower_ru(base);
        let particle_norm = lower_ru(particle);
        let matches_group = match group {
            CliticHyphenGroup::IndefinitePronominal => {
                is_indefinite_base(&base_norm) && matches!(particle_norm.as_str(), "то" | "либо" | "нибудь")
            }
            CliticHyphenGroup::EmphaticToPronounSeed => {
                is_emphatic_to_base(&base_norm) && particle_norm == "то"
            }
            CliticHyphenGroup::ImperativeKaSeed => {
                is_ka_base(&base_norm) && particle_norm == "ка"
            }
        };

        matches_group.then(|| CliticHyphenSuggestion {
            replacement: format!("{}-{}", base, particle),
            group,
            position: CliticPosition::Suffix,
        })
    }
}

fn is_indefinite_base(value: &str) -> bool {
    matches!(
        value,
        "кто"
            | "что"
            | "чей"
            | "какой"
            | "какая"
            | "какое"
            | "какие"
            | "как"
            | "где"
            | "куда"
            | "откуда"
            | "когда"
            | "почему"
            | "зачем"
            | "сколько"
            | "так"
            | "там"
            | "туда"
            | "оттуда"
            | "тогда"
            | "столько"
    )
}

fn is_emphatic_to_base(value: &str) -> bool {
    matches!(
        value,
        "я" | "ты" | "он" | "она" | "оно" | "мы" | "вы" | "они" | "это" | "тот"
    )
}

fn is_ka_base(value: &str) -> bool {
    matches!(
        value,
        "ну" | "давай" | "давайте" | "скажи" | "скажите" | "глянь" | "гляньте" | "подумай"
            | "постой" | "постойте" | "поди"
    )
}
