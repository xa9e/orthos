#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum PartOfSpeech {
    Noun,
    Adjective,
    Verb,
    Participle,
    Gerund,
    Pronoun,
    Numeral,
    Adverb,
    Comparative,
    Predicative,
    Preposition,
    Conjunction,
    Particle,
    Interjection,
    Punctuation,
    Other,
}

impl PartOfSpeech {
    pub fn parse(raw: &str) -> Self {
        match normalize_tag(raw).as_str() {
            "noun" | "n" | "s" | "сущ" | "substantive" => Self::Noun,
            "adj" | "adjf" | "adjs" | "adjective" | "a" | "полн_прил" | "кратк_прил" | "прил" => Self::Adjective,
            "verb" | "infn" | "v" | "гл" => Self::Verb,
            "participle" | "part" | "prtf" | "prts" | "прич" => Self::Participle,
            "gerund" | "grnd" | "деепр" => Self::Gerund,
            "pron" | "npro" | "pronoun" | "мест" => Self::Pronoun,
            "num" | "numr" | "numeral" | "числ" => Self::Numeral,
            "adv" | "advb" | "adverb" | "нар" => Self::Adverb,
            "comp" | "comparative" | "сравн" => Self::Comparative,
            "pred" | "predicative" | "предик" => Self::Predicative,
            "prep" | "preposition" | "предл" => Self::Preposition,
            "conj" | "conjunction" | "союз" => Self::Conjunction,
            "partcl" | "prcl" | "particle" | "част" => Self::Particle,
            "intj" | "interjection" | "межд" => Self::Interjection,
            "punct" | "punctuation" => Self::Punctuation,
            _ => Self::Other,
        }
    }

    pub fn can_modify_noun(&self) -> bool {
        matches!(self, Self::Adjective | Self::Participle)
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Noun),
            1 => Some(Self::Adjective),
            2 => Some(Self::Verb),
            3 => Some(Self::Participle),
            4 => Some(Self::Gerund),
            5 => Some(Self::Pronoun),
            6 => Some(Self::Numeral),
            7 => Some(Self::Adverb),
            8 => Some(Self::Comparative),
            9 => Some(Self::Predicative),
            10 => Some(Self::Preposition),
            11 => Some(Self::Conjunction),
            12 => Some(Self::Particle),
            13 => Some(Self::Interjection),
            14 => Some(Self::Punctuation),
            15 => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Grammeme {
    ProperName,
    CommonNoun,
    Qualitative,
    Relative,
    Possessive,
    Ordinal,
    Cardinal,
    Collective,
    Transitive,
    Intransitive,
    Reflexive,
    Impersonal,
    Indeclinable,
    Abbreviation,
}

impl Grammeme {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "proper" | "proper_name" | "name" | "имя" | "propn" => Self::ProperName,
            "common" | "common_noun" => Self::CommonNoun,
            "qual" | "qualitative" | "кач" => Self::Qualitative,
            "rel" | "relative" | "относ" => Self::Relative,
            "poss" | "possessive" | "притяж" => Self::Possessive,
            "ordinal" | "ord" | "порядк" => Self::Ordinal,
            "cardinal" | "card" | "колич" => Self::Cardinal,
            "collective" | "coll" | "собират" => Self::Collective,
            "trans" | "tran" | "переход" => Self::Transitive,
            "intr" | "intransitive" | "непереход" => Self::Intransitive,
            "reflexive" | "refl" | "возвр" => Self::Reflexive,
            "impersonal" | "imprs" | "безл" => Self::Impersonal,
            "indecl" | "indeclinable" | "0" => Self::Indeclinable,
            "abbr" | "abbreviation" | "сокр" => Self::Abbreviation,
            _ => return None,
        })
    }
}
