#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Case {
    Nominative,
    Genitive,
    Dative,
    Accusative,
    Instrumental,
    Prepositional,
    Locative,
    Partitive,
    Vocative,
}

impl Case {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "nom" | "nomn" | "nominative" | "имен" => Self::Nominative,
            "gen" | "gent" | "genitive" | "род" => Self::Genitive,
            "dat" | "datv" | "dative" | "дат" => Self::Dative,
            "acc" | "accs" | "accusative" | "вин" => Self::Accusative,
            "ins" | "ablt" | "inst" | "instrumental" | "твор" => Self::Instrumental,
            "prep" | "loct" | "prepositional" | "предл" => Self::Prepositional,
            "loc" | "loc2" | "locative" | "местн" => Self::Locative,
            "part" | "gen2" | "partitive" | "парт" => Self::Partitive,
            "voct" | "voc" | "vocative" | "зв" => Self::Vocative,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Number {
    Singular,
    Plural,
}

impl Number {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "sing" | "sg" | "singular" | "ед" => Self::Singular,
            "plur" | "pl" | "plural" | "мн" => Self::Plural,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Gender {
    Masculine,
    Feminine,
    Neuter,
    Common,
}

impl Gender {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "masc" | "m" | "masculine" | "муж" => Self::Masculine,
            "fem" | "femn" | "f" | "feminine" | "жен" => Self::Feminine,
            "neut" | "n" | "neutral" | "neuter" | "сред" => Self::Neuter,
            "common" | "ms_f" | "общ" => Self::Common,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Animacy {
    Animate,
    Inanimate,
}

impl Animacy {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "anim" | "animate" | "од" => Self::Animate,
            "inan" | "inanimate" | "неод" => Self::Inanimate,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Aspect {
    Perfective,
    Imperfective,
}

impl Aspect {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "perf" | "perfective" | "сов" => Self::Perfective,
            "impf" | "imperf" | "imperfective" | "несов" => Self::Imperfective,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Tense {
    Past,
    Present,
    Future,
}

impl Tense {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "past" | "pst" | "прош" => Self::Past,
            "pres" | "present" | "наст" => Self::Present,
            "futr" | "future" | "буд" => Self::Future,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Person {
    First,
    Second,
    Third,
}

impl Person {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "1" | "1p" | "1per" | "first" => Self::First,
            "2" | "2p" | "2per" | "second" => Self::Second,
            "3" | "3p" | "3per" | "third" => Self::Third,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum AdjectiveForm {
    Full,
    Short,
}

impl AdjectiveForm {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "full" | "plen" | "adjf" | "полн" => Self::Full,
            "short" | "brev" | "adjs" | "кратк" => Self::Short,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Degree {
    Positive,
    Comparative,
    Superlative,
}

impl Degree {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "pos" | "positive" => Self::Positive,
            "comp" | "comparative" | "cmp" | "сравн" => Self::Comparative,
            "supr" | "superlative" | "sup" | "прев" => Self::Superlative,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum VerbForm {
    Infinitive,
    Finite,
    Participle,
    Gerund,
}

impl VerbForm {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "inf" | "infn" | "infinitive" | "инф" => Self::Infinitive,
            "finite" | "fin" | "личн" => Self::Finite,
            "part" | "participle" | "prtf" | "prts" | "прич" => Self::Participle,
            "gerund" | "grnd" | "деепр" => Self::Gerund,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Mood {
    Indicative,
    Imperative,
}

impl Mood {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "ind" | "indicative" | "изъяв" => Self::Indicative,
            "imp" | "impr" | "imperative" | "повел" => Self::Imperative,
            _ => return None,
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Voice {
    Active,
    Passive,
}

impl Voice {
    pub fn parse(raw: &str) -> Option<Self> {
        Some(match normalize_tag(raw).as_str() {
            "act" | "actv" | "active" | "действ" => Self::Active,
            "pass" | "pssv" | "passive" | "страд" => Self::Passive,
            _ => return None,
        })
    }
}
