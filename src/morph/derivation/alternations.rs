#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixAssimilationSuggestion {
    pub actual_prefix: String,
    pub expected_prefix: String,
    pub remainder: String,
    pub replacement: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct AlternatingPrefix {
    voiced: &'static str,
    voiceless: &'static str,
}

const ALTERNATING_PREFIXES: &[AlternatingPrefix] = &[
    AlternatingPrefix { voiced: "через", voiceless: "черес" },
    AlternatingPrefix { voiced: "чрез", voiceless: "чрес" },
    AlternatingPrefix { voiced: "без", voiceless: "бес" },
    AlternatingPrefix { voiced: "воз", voiceless: "вос" },
    AlternatingPrefix { voiced: "из", voiceless: "ис" },
    AlternatingPrefix { voiced: "низ", voiceless: "нис" },
    AlternatingPrefix { voiced: "раз", voiceless: "рас" },
    AlternatingPrefix { voiced: "роз", voiceless: "рос" },
    AlternatingPrefix { voiced: "вз", voiceless: "вс" },
];

pub fn prefix_final_z_s_suggestion(word: &str) -> Option<PrefixAssimilationSuggestion> {
    RussianDerivationModel::seed().prefix_final_z_s_suggestion(word)
}

impl RussianDerivationModel {
    pub fn prefix_final_z_s_suggestion(&self, word: &str) -> Option<PrefixAssimilationSuggestion> {
        let lower = lower_ru(word);
        for prefix in ALTERNATING_PREFIXES {
            if let Some(suggestion) = self.check_assimilation_side(&lower, prefix.voiced, prefix.voiceless) {
                return Some(suggestion);
            }
            if let Some(suggestion) = self.check_assimilation_side(&lower, prefix.voiceless, prefix.voiced) {
                return Some(suggestion);
            }
        }
        None
    }

    fn check_assimilation_side(
        &self,
        word: &str,
        actual: &str,
        opposite: &str,
    ) -> Option<PrefixAssimilationSuggestion> {
        let remainder = word.strip_prefix(actual)?;
        let first = remainder.chars().next()?;
        let expected = expected_prefix(actual, opposite, first)?;
        if expected == actual || !self.likely_known_base_start(remainder) {
            return None;
        }
        Some(PrefixAssimilationSuggestion {
            actual_prefix: actual.to_string(),
            expected_prefix: expected.to_string(),
            remainder: remainder.to_string(),
            replacement: format!("{expected}{remainder}"),
        })
    }
}

fn expected_prefix<'a>(actual: &'a str, opposite: &'a str, next: char) -> Option<&'a str> {
    if is_voiceless_consonant(next) {
        Some(if ends_with_s(actual) { actual } else { opposite })
    } else if is_voiced_consonant(next) || is_vowel(next) {
        Some(if ends_with_z(actual) { actual } else { opposite })
    } else {
        None
    }
}

fn ends_with_s(value: &str) -> bool {
    value.ends_with('с')
}

fn ends_with_z(value: &str) -> bool {
    value.ends_with('з')
}

fn is_vowel(ch: char) -> bool {
    matches!(ch, 'а' | 'е' | 'ё' | 'и' | 'о' | 'у' | 'ы' | 'э' | 'ю' | 'я')
}

fn is_voiceless_consonant(ch: char) -> bool {
    matches!(ch, 'к' | 'п' | 'с' | 'т' | 'ф' | 'х' | 'ц' | 'ч' | 'ш' | 'щ')
}

fn is_voiced_consonant(ch: char) -> bool {
    matches!(ch, 'б' | 'в' | 'г' | 'д' | 'ж' | 'з' | 'й' | 'л' | 'м' | 'н' | 'р')
}
