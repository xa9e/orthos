#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumeralGovernmentClass {
    One,
    Paucal,
    Many,
    Collective,
    Ordinal,
    Unknown,
}

pub fn numeral_government_class(numeral: &MorphAnalysis) -> NumeralGovernmentClass {
    if numeral.has_grammeme(Grammeme::Ordinal) || has_normalized_tag(&numeral.features, "num_class=ordinal") {
        return NumeralGovernmentClass::Ordinal;
    }
    if numeral.has_grammeme(Grammeme::Collective) || has_normalized_tag(&numeral.features, "num_class=collective") {
        return NumeralGovernmentClass::Collective;
    }
    if has_normalized_tag(&numeral.features, "num_class=one") {
        return NumeralGovernmentClass::One;
    }
    if has_normalized_tag(&numeral.features, "num_class=paucal") {
        return NumeralGovernmentClass::Paucal;
    }
    if has_normalized_tag(&numeral.features, "num_class=many") {
        return NumeralGovernmentClass::Many;
    }

    match lower_ru(&numeral.lemma).as_str() {
        "один" | "одна" | "одно" => NumeralGovernmentClass::One,
        "два" | "две" | "три" | "четыре" => NumeralGovernmentClass::Paucal,
        "пять" | "шесть" | "семь" | "восемь" | "девять" | "десять" => NumeralGovernmentClass::Many,
        _ => NumeralGovernmentClass::Unknown,
    }
}

pub fn numeral_government_compatibility(
    numeral: &MorphAnalysis,
    noun: &MorphAnalysis,
) -> MorphCompatibility {
    if numeral.pos != PartOfSpeech::Numeral || noun.pos != PartOfSpeech::Noun {
        return MorphCompatibility::Unknown;
    }

    match numeral_government_class(numeral) {
        NumeralGovernmentClass::Unknown => MorphCompatibility::Unknown,
        NumeralGovernmentClass::Ordinal | NumeralGovernmentClass::One => {
            agreement_compatibility(numeral.agreement_signature(), noun.agreement_signature())
        }
        NumeralGovernmentClass::Paucal => governed_case_number_compatibility(
            numeral.features.case,
            noun.features.case,
            noun.features.number,
            Case::Genitive,
            Number::Singular,
        ),
        NumeralGovernmentClass::Many | NumeralGovernmentClass::Collective => governed_case_number_compatibility(
            numeral.features.case,
            noun.features.case,
            noun.features.number,
            Case::Genitive,
            Number::Plural,
        ),
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NumeralNounCompatibility {
    Compatible,
    Incompatible,
    Unknown,
    Unsupported,
}

pub fn numeral_noun_compatibility(
    numeral: &MorphAnalysis,
    noun: &MorphAnalysis,
) -> NumeralNounCompatibility {
    if numeral.pos != PartOfSpeech::Numeral || noun.pos != PartOfSpeech::Noun {
        return NumeralNounCompatibility::Unsupported;
    }

    match numeral_government_compatibility(numeral, noun) {
        MorphCompatibility::Compatible => NumeralNounCompatibility::Compatible,
        MorphCompatibility::Incompatible => NumeralNounCompatibility::Incompatible,
        MorphCompatibility::Unknown => NumeralNounCompatibility::Unknown,
    }
}

pub fn animacy_aware_accusative_compatibility(analysis: &MorphAnalysis) -> MorphCompatibility {
    match analysis.features.case {
        Some(Case::Accusative) => MorphCompatibility::Compatible,
        Some(Case::Genitive) => match (analysis.features.animacy, analysis.features.number, analysis.features.gender) {
            (Some(Animacy::Animate), Some(Number::Plural), _) => MorphCompatibility::Compatible,
            (Some(Animacy::Animate), Some(Number::Singular), Some(Gender::Masculine)) => {
                MorphCompatibility::Compatible
            }
            (Some(Animacy::Inanimate), _, _) => MorphCompatibility::Incompatible,
            (Some(Animacy::Animate), Some(Number::Singular), Some(_)) => MorphCompatibility::Unknown,
            _ => MorphCompatibility::Unknown,
        },
        Some(Case::Nominative) => match analysis.features.animacy {
            Some(Animacy::Inanimate) => MorphCompatibility::Compatible,
            Some(Animacy::Animate) => MorphCompatibility::Incompatible,
            None => MorphCompatibility::Unknown,
        },
        Some(_) => MorphCompatibility::Incompatible,
        None => MorphCompatibility::Unknown,
    }
}
