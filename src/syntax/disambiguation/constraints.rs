// Constraint-grammar style contextual disambiguation.
//
// Removes morphological readings that are impossible in their local context,
// before any fact layer or rule sees them. The engine is deliberately strict
// about when it may act:
//
// - the licensing token must be reliable (e.g. a preposition in *all* its
//   readings) before it can eliminate readings of a neighbour;
// - a token never loses its last reading;
// - if no consistent combination exists at all (a real grammar error), the
//   engine does not prune — the error must stay visible to detectors;
// - every elimination is recorded as a typed `ReadingElimination` proof.
//
// Constraints run to a bounded fixpoint because one elimination can make a
// neighbouring token reliable enough to license the next elimination.

const MAX_DISAMBIGUATION_PASSES: usize = 3;

/// Runs all constraints over per-token reading sets (indexed by token index).
pub fn disambiguate_readings(
    tokens: &[Token<'_>],
    readings: &mut [Vec<crate::morph::MorphAnalysis>],
    prepositions: &crate::morph::PrepositionGovernmentRegistry,
) -> DisambiguationTrace {
    let mut trace = DisambiguationTrace::default();
    for _ in 0..MAX_DISAMBIGUATION_PASSES {
        let before = trace.eliminations.len();
        trace.passes += 1;
        apply_preposition_constraints(tokens, readings, prepositions, &mut trace);
        apply_modifier_head_agreement(tokens, readings, &mut trace);
        apply_subject_predicate_agreement(tokens, readings, &mut trace);
        if trace.eliminations.len() == before {
            break;
        }
    }
    trace
}

/// Both preposition-licensed constraints: verb exclusion and case government.
fn apply_preposition_constraints(
    tokens: &[Token<'_>],
    readings: &mut [Vec<crate::morph::MorphAnalysis>],
    prepositions: &crate::morph::PrepositionGovernmentRegistry,
    trace: &mut DisambiguationTrace,
) {
    for prep_idx in 0..tokens.len() {
        if !is_reliable_preposition(tokens, readings, prep_idx) {
            continue;
        }
        let Some(next_idx) = adjacent_word_index(tokens, prep_idx) else {
            continue;
        };

        // A finite verb or gerund can never directly follow a preposition.
        remove_readings(
            readings,
            next_idx,
            tokens,
            trace,
            DisambiguationConstraint::PrepositionVerbExclusion,
            prep_idx,
            |analysis| {
                matches!(
                    analysis.pos,
                    crate::morph::PartOfSpeech::Verb | crate::morph::PartOfSpeech::Gerund
                )
            },
            |_| "finite verb or gerund reading directly after a preposition".to_owned(),
        );

        // A known preposition restricts the case of the adjacent nominal.
        let prep_key = lower_ru(tokens[prep_idx].text);
        let allowed_cases = prepositions
            .lookup(&prep_key)
            .iter()
            .flat_map(|entry| entry.allowed_cases.iter().copied())
            .collect::<BTreeSet<_>>();
        if allowed_cases.is_empty() {
            continue;
        }
        remove_readings(
            readings,
            next_idx,
            tokens,
            trace,
            DisambiguationConstraint::PrepositionCaseGovernment,
            prep_idx,
            |analysis| {
                analysis
                    .features
                    .case
                    .is_some_and(|case| !allowed_cases.contains(&case))
            },
            |analysis| {
                format!(
                    "case {:?} is not governed by preposition \u{ab}{}\u{bb}",
                    analysis.features.case, prep_key
                )
            },
        );
    }
}

/// Keeps only readings that participate in at least one agreeing
/// modifier-head combination, when the partner token is reliable.
fn apply_modifier_head_agreement(
    tokens: &[Token<'_>],
    readings: &mut [Vec<crate::morph::MorphAnalysis>],
    trace: &mut DisambiguationTrace,
) {
    for left_idx in 0..tokens.len() {
        let Some(right_idx) = adjacent_word_index(tokens, left_idx) else {
            continue;
        };
        if tokens[left_idx].kind != TokenKind::Word {
            continue;
        }
        let modifier_reliable = is_reliable_modifier(&readings[left_idx]);
        let head_reliable = is_reliable_noun(&readings[right_idx]);
        if !modifier_reliable && !head_reliable {
            continue;
        }
        let has_agreeing_pair = readings[left_idx].iter().any(|adj| {
            is_full_modifier(adj)
                && readings[right_idx].iter().any(|noun| {
                    noun.pos == crate::morph::PartOfSpeech::Noun
                        && crate::morph::can_agree_as_adj_noun(adj, noun)
                })
        });
        // A real agreement error must stay visible to detectors.
        if !has_agreeing_pair {
            continue;
        }

        if modifier_reliable {
            let modifier_readings = readings[left_idx].clone();
            remove_readings(
                readings,
                right_idx,
                tokens,
                trace,
                DisambiguationConstraint::ModifierHeadAgreement,
                left_idx,
                |analysis| {
                    analysis.pos == crate::morph::PartOfSpeech::Noun
                        && analysis.agreement_signature().is_complete_for_adj_noun()
                        && !modifier_readings
                            .iter()
                            .any(|adj| crate::morph::can_agree_as_adj_noun(adj, analysis))
                },
                |_| "noun reading agrees with no reading of the adjacent modifier".to_owned(),
            );
        }
        if head_reliable {
            let head_readings = readings[right_idx].clone();
            remove_readings(
                readings,
                left_idx,
                tokens,
                trace,
                DisambiguationConstraint::ModifierHeadAgreement,
                right_idx,
                |analysis| {
                    is_full_modifier(analysis)
                        && analysis.agreement_signature().is_complete_for_adj_noun()
                        && !head_readings
                            .iter()
                            .any(|noun| crate::morph::can_agree_as_adj_noun(analysis, noun))
                },
                |_| "modifier reading agrees with no reading of the adjacent noun".to_owned(),
            );
        }
    }
}

/// Prunes local subject-predicate ambiguities after a compatible pair is
/// already licensed by the context. If all predicate readings conflict, the
/// probable grammar error remains visible to detectors.
fn apply_subject_predicate_agreement(
    tokens: &[Token<'_>],
    readings: &mut [Vec<crate::morph::MorphAnalysis>],
    trace: &mut DisambiguationTrace,
) {
    for subject_idx in 0..tokens.len() {
        if tokens[subject_idx].kind != TokenKind::Word {
            continue;
        }
        if previous_adjacent_word_index(tokens, subject_idx).is_some() {
            continue;
        }
        let Some(predicate_idx) = adjacent_word_index(tokens, subject_idx) else {
            continue;
        };

        let subject_reliable = is_reliable_subject(&readings[subject_idx]);
        let predicate_reliable = is_reliable_predicate(&readings[predicate_idx]);
        if !subject_reliable && !predicate_reliable {
            continue;
        }

        let has_compatible_pair = readings[subject_idx].iter().any(|subject| {
            is_subject_reading(subject)
                && readings[predicate_idx].iter().any(|predicate| {
                    is_predicate_reading(predicate)
                        && subject_predicate_pair_is_compatible(subject, predicate)
                })
        });
        if !has_compatible_pair {
            continue;
        }

        if subject_reliable {
            let subject_readings = readings[subject_idx].clone();
            remove_readings(
                readings,
                predicate_idx,
                tokens,
                trace,
                DisambiguationConstraint::SubjectPredicateAgreement,
                subject_idx,
                |analysis| {
                    !is_predicate_reading(analysis)
                        || !subject_readings.iter().any(|subject| {
                            subject_predicate_pair_is_compatible(subject, analysis)
                        })
                },
                |_| "reading is not compatible with the adjacent subject".to_owned(),
            );
        }
        if predicate_reliable {
            let predicate_readings = readings[predicate_idx].clone();
            remove_readings(
                readings,
                subject_idx,
                tokens,
                trace,
                DisambiguationConstraint::SubjectPredicateAgreement,
                predicate_idx,
                |analysis| {
                    !is_subject_reading(analysis)
                        || !predicate_readings.iter().any(|predicate| {
                            subject_predicate_pair_is_compatible(analysis, predicate)
                        })
                },
                |_| "reading is not compatible with the adjacent predicate".to_owned(),
            );
        }
    }
}

/// Removes readings matching `should_remove`, never the last one, and records
/// one `ReadingElimination` per removed reading.
#[allow(clippy::too_many_arguments)]
fn remove_readings(
    readings: &mut [Vec<crate::morph::MorphAnalysis>],
    token_index: usize,
    tokens: &[Token<'_>],
    trace: &mut DisambiguationTrace,
    constraint: DisambiguationConstraint,
    evidence_token_index: usize,
    should_remove: impl Fn(&crate::morph::MorphAnalysis) -> bool,
    explanation: impl Fn(&crate::morph::MorphAnalysis) -> String,
) {
    let bucket = &readings[token_index];
    let removable = bucket
        .iter()
        .filter(|analysis| should_remove(analysis))
        .count();
    if removable == 0 || removable == bucket.len() {
        return;
    }
    let (removed, kept): (Vec<_>, Vec<_>) = bucket
        .clone()
        .into_iter()
        .partition(|analysis| should_remove(analysis));
    for analysis in &removed {
        trace.eliminations.push(ReadingElimination {
            token_index,
            form: tokens[token_index].text.to_owned(),
            eliminated_lemma: analysis.lemma.clone(),
            eliminated_pos: analysis.pos,
            eliminated_features: analysis
                .features
                .raw_tags
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join("|"),
            constraint,
            evidence_token_index,
            evidence_form: tokens[evidence_token_index].text.to_owned(),
            explanation: explanation(analysis),
        });
    }
    readings[token_index] = kept;
}

/// Next word token, adjacent across whitespace only; punctuation breaks the
/// local context.
fn adjacent_word_index(tokens: &[Token<'_>], from: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(from + 1)
        .find(|(_, token)| token.kind != TokenKind::Whitespace)
        .filter(|(_, token)| token.kind == TokenKind::Word)
        .map(|(idx, _)| idx)
}

fn previous_adjacent_word_index(tokens: &[Token<'_>], from: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .take(from)
        .rev()
        .find(|(_, token)| token.kind != TokenKind::Whitespace)
        .filter(|(_, token)| token.kind == TokenKind::Word)
        .map(|(idx, _)| idx)
}

fn is_reliable_preposition(
    tokens: &[Token<'_>],
    readings: &[Vec<crate::morph::MorphAnalysis>],
    idx: usize,
) -> bool {
    tokens[idx].kind == TokenKind::Word
        && !readings[idx].is_empty()
        && readings[idx]
            .iter()
            .all(|analysis| analysis.pos == crate::morph::PartOfSpeech::Preposition)
}

fn is_full_modifier(analysis: &crate::morph::MorphAnalysis) -> bool {
    analysis.pos.can_modify_noun()
        && !matches!(
            analysis.features.adjective_form,
            Some(crate::morph::AdjectiveForm::Short)
        )
        && !matches!(
            analysis.features.degree,
            Some(crate::morph::Degree::Comparative)
        )
}

fn is_reliable_modifier(readings: &[crate::morph::MorphAnalysis]) -> bool {
    !readings.is_empty()
        && readings.iter().all(|analysis| {
            is_full_modifier(analysis) && analysis.agreement_signature().is_complete_for_adj_noun()
        })
}

fn is_reliable_noun(readings: &[crate::morph::MorphAnalysis]) -> bool {
    !readings.is_empty()
        && readings.iter().all(|analysis| {
            analysis.pos == crate::morph::PartOfSpeech::Noun
                && analysis.agreement_signature().is_complete_for_adj_noun()
        })
}

fn is_subject_reading(analysis: &crate::morph::MorphAnalysis) -> bool {
    matches!(
        analysis.pos,
        crate::morph::PartOfSpeech::Noun
            | crate::morph::PartOfSpeech::Pronoun
            | crate::morph::PartOfSpeech::Numeral
    ) && !matches!(analysis.features.case, Some(case) if case != crate::morph::Case::Nominative)
        && analysis.features.number.is_some()
}

fn is_predicate_reading(analysis: &crate::morph::MorphAnalysis) -> bool {
    match analysis.pos {
        crate::morph::PartOfSpeech::Verb => {
            !matches!(analysis.features.verb_form, Some(crate::morph::VerbForm::Infinitive))
                && analysis.features.number.is_some()
        }
        crate::morph::PartOfSpeech::Predicative => true,
        crate::morph::PartOfSpeech::Adjective => {
            matches!(
                analysis.features.adjective_form,
                Some(crate::morph::AdjectiveForm::Short)
            ) && analysis.features.number.is_some()
        }
        _ => false,
    }
}

fn is_reliable_subject(readings: &[crate::morph::MorphAnalysis]) -> bool {
    !readings.is_empty() && readings.iter().all(is_subject_reading)
}

fn is_reliable_predicate(readings: &[crate::morph::MorphAnalysis]) -> bool {
    !readings.is_empty() && readings.iter().all(is_predicate_reading)
}

fn subject_predicate_pair_is_compatible(
    subject: &crate::morph::MorphAnalysis,
    predicate: &crate::morph::MorphAnalysis,
) -> bool {
    is_subject_reading(subject)
        && is_predicate_reading(predicate)
        && crate::morph::subject_predicate_agreement_check(subject, predicate).compatibility
            == crate::morph::MorphCompatibility::Compatible
}
