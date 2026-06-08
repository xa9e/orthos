#[cfg(test)]
mod diagnostic_ledger_tests {
    use super::*;
    use crate::morph::MorphLexicon;

    #[test]
    fn diagnostic_ledger_keeps_emitted_and_suppressed_candidates() {
        let lexicon = MorphLexicon::parse_tsv(coordination_fixture_tsv());
        let facts = LinguisticFactStore::new(
            "умный быстрый анализатор. Он сказал: «Конечно анализатор работает». ООН решила вопрос. ООН обновила доклад.",
            &lexicon,
        );
        let ledger = facts.diagnostic_ledger();

        assert!(ledger.entries_by_kind(DiagnosticLedgerKind::Punctuation).any(|entry| {
            entry.status == DiagnosticLedgerStatus::EmittedCandidate
                && entry.proof.conflict.as_ref().is_some_and(|conflict| {
                    conflict.kind == "missing_punctuation_mark"
                })
        }));
        assert!(ledger.suppressed().any(|entry| {
            entry.kind == DiagnosticLedgerKind::Punctuation
                && entry.proof.blockers.contains(&SuppressionReason::DirectSpeechBoundary)
        }));
        assert!(ledger.entries_by_kind(DiagnosticLedgerKind::DocumentAbbreviation).any(|entry| {
            entry.status == DiagnosticLedgerStatus::EmittedCandidate
                && entry.proof.kind == DiagnosticProofKind::DocumentAbbreviation
        }));
        assert!(facts.summary().diagnostic_ledger_entries >= ledger.entries().len());
    }

    #[test]
    fn punctuation_slots_find_asyndetic_modifier_coordination() {
        let lexicon = MorphLexicon::parse_tsv(coordination_fixture_tsv());
        let facts = LinguisticFactStore::new("умный быстрый анализатор", &lexicon);
        let slot = facts
            .punctuation_slots()
            .iter()
            .find(|slot| slot.left_token.text == "умный" && slot.right_token.text == "быстрый")
            .unwrap();

        assert!(slot.missing_expected_mark(PunctuationMark::Comma));
        assert!(slot.evidence.iter().any(|item| {
            item.kind == PunctuationSlotEvidenceKind::Coordination
                && item.mark == Some(PunctuationMark::Comma)
        }));
    }



    fn coordination_fixture_tsv() -> &'static str {
        "умный\tумный\tADJ\tgender=masc|number=sing|case=nom|adj_form=full|degree=pos\n\
         быстрый\tбыстрый\tADJ\tgender=masc|number=sing|case=nom|adj_form=full|degree=pos\n\
         анализатор\tанализатор\tNOUN\tgender=masc|number=sing|case=nom|animacy=inan\n\
         работает\tработать\tVERB\tnumber=sing|person=3|tense=pres|verb_form=finite|aspect=impf\n"
    }
}
