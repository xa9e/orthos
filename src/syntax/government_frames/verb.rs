fn verb_government_frames_from_tokens<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    islands: &SyntacticIslandMap,
    clause_boundaries: &ClauseBoundaryMap,
) -> Vec<GovernmentFrame<'a>> {
    let registry = crate::morph::VerbGovernmentRegistry::russian_seed();
    let mut frames = Vec::new();
    for governor_index in 0..tokens.len() {
        let Some(governor) = tokens.get(governor_index) else {
            continue;
        };
        if governor.kind != TokenKind::Word {
            continue;
        }
        let governor_analyses = ambiguity.analyses_for_token(governor_index).to_vec();
        if !can_govern_as_verb(governor.text, &governor_analyses, &registry) {
            continue;
        }
        frames.extend(direct_verb_government_frames(
            tokens,
            ambiguity,
            islands,
            clause_boundaries,
            governor_index,
            &governor_analyses,
            &registry,
        ));
        frames.extend(prepositional_verb_government_frames(
            tokens,
            ambiguity,
            islands,
            clause_boundaries,
            governor_index,
            &governor_analyses,
            &registry,
        ));
    }
    frames
}

fn direct_verb_government_frames<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    islands: &SyntacticIslandMap,
    clause_boundaries: &ClauseBoundaryMap,
    governor_index: usize,
    governor_analyses: &[crate::morph::MorphAnalysis],
    registry: &crate::morph::VerbGovernmentRegistry,
) -> Vec<GovernmentFrame<'a>> {
    let Some(governor) = tokens.get(governor_index) else {
        return Vec::new();
    };
    let entries = direct_entries_for_verb_governor(governor.text, governor_analyses, registry);
    if entries.is_empty() {
        return Vec::new();
    }
    let Some(group_start) = following_word_after_plain_gap(tokens, governor_index) else {
        return Vec::new();
    };
    let mut frames = Vec::new();
    for entry in entries {
        let expected_cases = sorted_cases(entry.allowed_cases.iter().copied());
        for modifier_count in 0..=2 {
            let Some(group) = nominal_group_from_start(tokens, group_start, modifier_count) else {
                continue;
            };
            frames.push(case_government_frame(
                tokens,
                ambiguity,
                islands,
                clause_boundaries,
                governor_index,
                governor_analyses,
                group,
                expected_cases.clone(),
                GovernmentFrameSource::VerbValencySeed,
                Some(GovernmentFrameModelRef::from_verb_government(entry)),
            ));
        }
    }
    frames
}

fn prepositional_verb_government_frames<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    islands: &SyntacticIslandMap,
    clause_boundaries: &ClauseBoundaryMap,
    governor_index: usize,
    governor_analyses: &[crate::morph::MorphAnalysis],
    registry: &crate::morph::VerbGovernmentRegistry,
) -> Vec<GovernmentFrame<'a>> {
    let Some(governor) = tokens.get(governor_index) else {
        return Vec::new();
    };
    let entries = prepositional_entries_for_verb_governor(governor.text, governor_analyses, registry);
    if entries.is_empty() {
        return Vec::new();
    }
    let Some(preposition_index) = following_word_after_plain_gap(tokens, governor_index) else {
        return Vec::new();
    };
    let Some(preposition) = tokens.get(preposition_index) else {
        return Vec::new();
    };
    let preposition_text = lower_ru(preposition.text);
    let Some(group_start) = following_word_after_plain_gap(tokens, preposition_index) else {
        return Vec::new();
    };

    let mut frames = Vec::new();
    for entry in entries {
        if entry.preposition.as_deref() != Some(preposition_text.as_str()) {
            continue;
        }
        let expected_cases = sorted_cases(entry.allowed_cases.iter().copied());
        for modifier_count in 0..=2 {
            let Some(group) = nominal_group_from_start(tokens, group_start, modifier_count) else {
                continue;
            };
            frames.push(case_government_frame(
                tokens,
                ambiguity,
                islands,
                clause_boundaries,
                governor_index,
                governor_analyses,
                group,
                expected_cases.clone(),
                GovernmentFrameSource::VerbPrepositionalValencySeed,
                Some(GovernmentFrameModelRef::from_verb_government(entry)),
            ));
        }
    }
    frames
}
