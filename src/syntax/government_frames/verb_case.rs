#[allow(clippy::too_many_arguments)]
fn case_government_frame<'a>(
    tokens: &[Token<'a>],
    ambiguity: &AmbiguityModel<'a>,
    islands: &SyntacticIslandMap,
    clause_boundaries: &ClauseBoundaryMap,
    governor_index: usize,
    governor_analyses: &[crate::morph::MorphAnalysis],
    group: NominalGroupCandidate<'a>,
    expected_cases: Vec<crate::morph::Case>,
    source: GovernmentFrameSource,
    model_ref: Option<GovernmentFrameModelRef>,
) -> GovernmentFrame<'a> {
    let governor = tokens[governor_index].clone();
    let dependent_analyses = ambiguity.analyses_for_token(group.end_token).to_vec();
    let (compatibility, observed_cases, mut blockers) = case_government_result(
        &expected_cases,
        &dependent_analyses,
    );
    blockers.extend(
        group
            .blockers
            .iter()
            .copied()
            .map(SuppressionReason::from),
    );
    blockers.extend(government_boundary_blockers_for_link(
        governor_index,
        group.end_token,
        islands,
        clause_boundaries,
    ));
    blockers.sort_unstable();
    blockers.dedup();
    GovernmentFrame {
        kind: GovernmentFrameKind::Verb,
        source,
        governor: MorphosyntacticTerm {
            token_index: governor_index,
            token: governor,
            role: MorphosyntacticRole::CaseGovernor,
            analyses: governor_analyses.to_vec(),
        },
        dependent: MorphosyntacticTerm {
            token_index: group.end_token,
            token: group.head.clone(),
            role: MorphosyntacticRole::GovernedNominal,
            analyses: dependent_analyses,
        },
        span: Span::new(tokens[governor_index].span.start, group.span.end),
        expected_cases,
        observed_cases,
        expected_numbers: Vec::new(),
        observed_numbers: Vec::new(),
        compatibility,
        confidence: confidence_for_case_frame(&blockers),
        blockers,
        model_ref,
    }
}

fn can_govern_as_verb(
    surface: &str,
    analyses: &[crate::morph::MorphAnalysis],
    registry: &crate::morph::VerbGovernmentRegistry,
) -> bool {
    !verb_lemmas(surface, analyses, registry).is_empty()
}

fn direct_entries_for_verb_governor<'a>(
    surface: &str,
    analyses: &[crate::morph::MorphAnalysis],
    registry: &'a crate::morph::VerbGovernmentRegistry,
) -> Vec<&'a crate::morph::VerbGovernment> {
    let mut entries = verb_lemmas(surface, analyses, registry)
        .into_iter()
        .flat_map(|lemma| registry.direct_entries_for_lemma(&lemma))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.lemma.cmp(&right.lemma));
    entries.dedup_by(|left, right| {
        left.lemma == right.lemma
            && left.complement_kind == right.complement_kind
            && left.preposition == right.preposition
            && left.allowed_cases == right.allowed_cases
    });
    entries
}

fn prepositional_entries_for_verb_governor<'a>(
    surface: &str,
    analyses: &[crate::morph::MorphAnalysis],
    registry: &'a crate::morph::VerbGovernmentRegistry,
) -> Vec<&'a crate::morph::VerbGovernment> {
    let mut entries = verb_lemmas(surface, analyses, registry)
        .into_iter()
        .flat_map(|lemma| registry.prepositional_entries_for_lemma(&lemma))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        (left.lemma.as_str(), left.preposition.as_deref().unwrap_or(""))
            .cmp(&(right.lemma.as_str(), right.preposition.as_deref().unwrap_or("")))
    });
    entries.dedup_by(|left, right| {
        left.lemma == right.lemma
            && left.complement_kind == right.complement_kind
            && left.preposition == right.preposition
            && left.allowed_cases == right.allowed_cases
    });
    entries
}

fn verb_lemmas(
    surface: &str,
    analyses: &[crate::morph::MorphAnalysis],
    registry: &crate::morph::VerbGovernmentRegistry,
) -> Vec<String> {
    let mut lemmas = analyses
        .iter()
        .filter(|analysis| analysis.pos == crate::morph::PartOfSpeech::Verb)
        .map(|analysis| lower_ru(&analysis.lemma))
        .filter(|lemma| !registry.lookup(lemma).is_empty())
        .collect::<BTreeSet<_>>();
    if lemmas.is_empty() {
        let fallback = lower_ru(surface);
        if !registry.lookup(&fallback).is_empty() {
            lemmas.insert(fallback);
        }
    }
    lemmas.into_iter().collect()
}

fn sorted_cases(cases: impl IntoIterator<Item = crate::morph::Case>) -> Vec<crate::morph::Case> {
    cases.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

fn case_government_result(
    expected_cases: &[crate::morph::Case],
    dependent_analyses: &[crate::morph::MorphAnalysis],
) -> (crate::morph::MorphCompatibility, Vec<crate::morph::Case>, Vec<SuppressionReason>) {
    let nominals = dependent_analyses
        .iter()
        .filter(|analysis| can_be_case_governed(analysis))
        .collect::<Vec<_>>();
    if nominals.is_empty() {
        return (
            crate::morph::MorphCompatibility::Unknown,
            Vec::new(),
            vec![SuppressionReason::UnknownMorphology],
        );
    }

    let mut observed_cases = BTreeSet::new();
    let mut saw_unknown_case = false;
    for analysis in nominals {
        match analysis.features.case {
            Some(case) if expected_cases.contains(&case) => {
                return (crate::morph::MorphCompatibility::Compatible, Vec::new(), Vec::new());
            }
            Some(case) => {
                observed_cases.insert(case);
            }
            None => saw_unknown_case = true,
        }
    }

    if saw_unknown_case || observed_cases.is_empty() {
        return (
            crate::morph::MorphCompatibility::Unknown,
            observed_cases.into_iter().collect(),
            vec![SuppressionReason::InsufficientMorphology],
        );
    }
    (
        crate::morph::MorphCompatibility::Incompatible,
        observed_cases.into_iter().collect(),
        Vec::new(),
    )
}

fn can_be_case_governed(analysis: &crate::morph::MorphAnalysis) -> bool {
    matches!(
        analysis.pos,
        crate::morph::PartOfSpeech::Noun
            | crate::morph::PartOfSpeech::Pronoun
            | crate::morph::PartOfSpeech::Adjective
            | crate::morph::PartOfSpeech::Numeral
            | crate::morph::PartOfSpeech::Participle
    )
}

fn confidence_for_case_frame(blockers: &[SuppressionReason]) -> SyntaxConfidence {
    if blockers.is_empty() {
        SyntaxConfidence::Strong
    } else {
        SyntaxConfidence::Ambiguous
    }
}
