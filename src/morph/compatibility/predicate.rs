#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubjectPredicateRole {
    Subject,
    Predicate,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SubjectAgreementSignature {
    pub person: Option<Person>,
    pub number: Option<Number>,
    pub gender: Option<Gender>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct PredicateAgreementSignature {
    pub person: Option<Person>,
    pub number: Option<Number>,
    pub gender: Option<Gender>,
    pub tense: Option<Tense>,
}

pub fn subject_agreement_signature(analysis: &MorphAnalysis) -> SubjectAgreementSignature {
    let person = match analysis.pos {
        PartOfSpeech::Noun => Some(Person::Third),
        PartOfSpeech::Pronoun | PartOfSpeech::Numeral => analysis.features.person,
        _ => None,
    };
    SubjectAgreementSignature {
        person,
        number: analysis.features.number,
        gender: analysis.features.gender,
    }
}

pub fn predicate_agreement_signature(analysis: &MorphAnalysis) -> PredicateAgreementSignature {
    PredicateAgreementSignature {
        person: analysis.features.person,
        number: analysis.features.number,
        gender: analysis.features.gender,
        tense: analysis.features.tense,
    }
}

pub fn subject_predicate_compatibility(
    subject: &MorphAnalysis,
    predicate: &MorphAnalysis,
) -> MorphCompatibility {
    if !matches!(subject.pos, PartOfSpeech::Noun | PartOfSpeech::Pronoun | PartOfSpeech::Numeral)
        || !matches!(
            predicate.pos,
            PartOfSpeech::Verb | PartOfSpeech::Participle | PartOfSpeech::Predicative
        )
    {
        return MorphCompatibility::Unknown;
    }

    let subject = subject_agreement_signature(subject);
    let predicate = predicate_agreement_signature(predicate);
    let mut saw_unknown = false;

    match number_compatibility(subject.number, predicate.number) {
        MorphCompatibility::Incompatible => return MorphCompatibility::Incompatible,
        MorphCompatibility::Unknown => saw_unknown = true,
        MorphCompatibility::Compatible => {}
    }

    if predicate.person.is_some() {
        match optional_value_compatibility(subject.person, predicate.person) {
            MorphCompatibility::Incompatible => return MorphCompatibility::Incompatible,
            MorphCompatibility::Unknown => saw_unknown = true,
            MorphCompatibility::Compatible => {}
        }
    }

    let needs_gender = matches!(predicate.tense, Some(Tense::Past))
        && matches!(predicate.number, Some(Number::Singular));
    if needs_gender {
        match gender_compatibility(subject.number, predicate.number, subject.gender, predicate.gender) {
            MorphCompatibility::Incompatible => return MorphCompatibility::Incompatible,
            MorphCompatibility::Unknown => saw_unknown = true,
            MorphCompatibility::Compatible => {}
        }
    }

    if saw_unknown {
        MorphCompatibility::Unknown
    } else {
        MorphCompatibility::Compatible
    }
}
