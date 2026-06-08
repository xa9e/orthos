#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MorphCompatibility {
    Compatible,
    Incompatible,
    Unknown,
}

impl MorphCompatibility {
    pub fn permits_conservative_match(self) -> bool {
        !matches!(self, Self::Incompatible)
    }
}

pub fn case_compatibility(left: Option<Case>, right: Option<Case>) -> MorphCompatibility {
    optional_value_compatibility(left, right)
}

pub fn number_compatibility(left: Option<Number>, right: Option<Number>) -> MorphCompatibility {
    optional_value_compatibility(left, right)
}

pub fn gender_compatibility(
    left_number: Option<Number>,
    right_number: Option<Number>,
    left_gender: Option<Gender>,
    right_gender: Option<Gender>,
) -> MorphCompatibility {
    if matches!(left_number, Some(Number::Plural)) || matches!(right_number, Some(Number::Plural)) {
        return MorphCompatibility::Compatible;
    }
    match (left_gender, right_gender) {
        (Some(Gender::Common), Some(_)) | (Some(_), Some(Gender::Common)) => MorphCompatibility::Compatible,
        (Some(left), Some(right)) if left == right => MorphCompatibility::Compatible,
        (Some(_), Some(_)) => MorphCompatibility::Incompatible,
        _ => MorphCompatibility::Unknown,
    }
}

pub fn has_compatible_case_number_gender(left: &MorphAnalysis, right: &MorphAnalysis) -> bool {
    let left = left.agreement_signature();
    let right = right.agreement_signature();

    case_compatibility(left.case, right.case).permits_conservative_match()
        && number_compatibility(left.number, right.number).permits_conservative_match()
        && gender_compatibility(left.number, right.number, left.gender, right.gender)
            .permits_conservative_match()
}

pub fn can_agree_as_adj_noun(adj: &MorphAnalysis, noun: &MorphAnalysis) -> bool {
    if !adj.pos.can_modify_noun() || noun.pos != PartOfSpeech::Noun {
        return true;
    }
    if matches!(adj.features.degree, Some(Degree::Comparative)) {
        return true;
    }
    if matches!(adj.features.adjective_form, Some(AdjectiveForm::Short)) {
        return false;
    }
    has_compatible_case_number_gender(adj, noun)
}

/// Conservative detector helper: true means the adjacent pair is confidently
/// incompatible; false covers compatible, unknown, and dangerous ambiguity.
pub fn confidently_reject_adj_noun_agreement(
    left_analyses: &[MorphAnalysis],
    right_analyses: &[MorphAnalysis],
) -> bool {
    let adj: Vec<&MorphAnalysis> = left_analyses
        .iter()
        .filter(|analysis| analysis.pos.can_modify_noun())
        .collect();
    let noun: Vec<&MorphAnalysis> = right_analyses
        .iter()
        .filter(|analysis| analysis.pos == PartOfSpeech::Noun)
        .collect();

    if adj.is_empty() || noun.is_empty() {
        return false;
    }

    let left_has_other_pos = left_analyses
        .iter()
        .any(|analysis| !analysis.pos.can_modify_noun());
    let right_has_other_pos = right_analyses
        .iter()
        .any(|analysis| analysis.pos != PartOfSpeech::Noun);
    if left_has_other_pos || right_has_other_pos {
        return false;
    }

    if adj.iter().any(|analysis| !analysis.agreement_signature().is_complete_for_adj_noun()) {
        return false;
    }
    if noun.iter().any(|analysis| !analysis.agreement_signature().is_complete_for_adj_noun()) {
        return false;
    }

    let adj_signatures = unique_signatures(&adj);
    let noun_signatures = unique_signatures(&noun);
    if adj_signatures.len() != 1 || noun_signatures.len() != 1 {
        return false;
    }

    !adj
        .iter()
        .any(|adj| noun.iter().any(|noun| can_agree_as_adj_noun(adj, noun)))
}
