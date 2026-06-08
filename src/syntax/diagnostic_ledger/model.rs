#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLedgerStatus {
    EmittedCandidate,
    Suppressed,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLedgerKind {
    Agreement,
    Government,
    Quantity,
    Punctuation,
    DocumentStyle,
    DocumentAbbreviation,
    Unknown,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DiagnosticLedgerEntry {
    pub id: usize,
    pub kind: DiagnosticLedgerKind,
    pub status: DiagnosticLedgerStatus,
    pub span: Span,
    pub proof: DiagnosticProof,
}

#[derive(Debug, Clone, Eq, PartialEq, Default, serde::Deserialize, serde::Serialize)]
pub struct DiagnosticLedger {
    entries: Vec<DiagnosticLedgerEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DocumentAbbreviationCandidate<'a> {
    pub abbreviation: &'a DocumentAbbreviation,
    pub reason: SuppressionReason,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DocumentStyleCandidate<'a> {
    pub key: &'static str,
    pub span: Span,
    pub reason: SuppressionReason,
    pub headings: Vec<&'a HeadingCandidate>,
    pub list_items: Vec<&'a ListItemCandidate>,
}

impl DiagnosticLedger {
    pub fn from_facts(
        agreement_graph: &AgreementGraph<'_>,
        government_frames: &[GovernmentFrame<'_>],
        punctuation_slots: &[PunctuationSlot<'_>],
        document_context: &DocumentContext<'_>,
    ) -> Self {
        let mut entries = Vec::new();
        collect_agreement_entries(agreement_graph, &mut entries);
        collect_government_entries(government_frames, &mut entries);
        collect_punctuation_entries(punctuation_slots, &mut entries);
        collect_document_entries(document_context, &mut entries);
        for (id, entry) in entries.iter_mut().enumerate() {
            entry.id = id;
        }
        Self { entries }
    }

    pub fn entries(&self) -> &[DiagnosticLedgerEntry] {
        &self.entries
    }

    pub fn suppressed(&self) -> impl Iterator<Item = &DiagnosticLedgerEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.status == DiagnosticLedgerStatus::Suppressed)
    }

    pub fn emitted_candidates(&self) -> impl Iterator<Item = &DiagnosticLedgerEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.status == DiagnosticLedgerStatus::EmittedCandidate)
    }

    pub fn entries_by_kind(
        &self,
        kind: DiagnosticLedgerKind,
    ) -> impl Iterator<Item = &DiagnosticLedgerEntry> {
        self.entries.iter().filter(move |entry| entry.kind == kind)
    }
}

pub fn document_abbreviation_candidates<'a>(
    context: &'a DocumentContext<'a>,
) -> Vec<DocumentAbbreviationCandidate<'a>> {
    context
        .abbreviations
        .iter()
        .filter(|abbr| abbr.frequency > 1 && abbr.expansion.is_none())
        .map(|abbreviation| DocumentAbbreviationCandidate {
            abbreviation,
            reason: SuppressionReason::ConflictNotProven,
        })
        .collect()
}

pub fn document_style_candidates<'a>(
    context: &'a DocumentContext<'a>,
) -> Vec<DocumentStyleCandidate<'a>> {
    let mut out = Vec::new();
    if context.style.mixed_list_markers {
        out.push(DocumentStyleCandidate {
            key: "mixed_list_markers",
            span: span_covering_list_items(&context.list_items),
            reason: SuppressionReason::ConflictNotProven,
            headings: Vec::new(),
            list_items: context.list_items.iter().collect(),
        });
    }
    if context.style.mixed_heading_punctuation {
        out.push(DocumentStyleCandidate {
            key: "mixed_heading_punctuation",
            span: span_covering_headings(&context.headings),
            reason: SuppressionReason::ConflictNotProven,
            headings: context.headings.iter().collect(),
            list_items: Vec::new(),
        });
    }
    out
}

fn collect_agreement_entries(
    agreement_graph: &AgreementGraph<'_>,
    entries: &mut Vec<DiagnosticLedgerEntry>,
) {
    for edge in agreement_graph.edges() {
        let proof = agreement_edge_proof(edge);
        if proof.conflict.is_none() && proof.blockers.is_empty() {
            continue;
        }
        let kind = match edge.kind {
            AgreementGraphEdgeKind::NumeralHead => DiagnosticLedgerKind::Quantity,
            _ => DiagnosticLedgerKind::Agreement,
        };
        push_ledger_entry(entries, kind, edge.span, proof);
    }
}

fn collect_government_entries(
    frames: &[GovernmentFrame<'_>],
    entries: &mut Vec<DiagnosticLedgerEntry>,
) {
    for frame in frames {
        if frame.compatibility != crate::morph::MorphCompatibility::Incompatible && frame.blockers.is_empty() {
            continue;
        }
        let kind = match frame.kind {
            GovernmentFrameKind::Numeral => DiagnosticLedgerKind::Quantity,
            GovernmentFrameKind::Preposition | GovernmentFrameKind::Verb => DiagnosticLedgerKind::Government,
            GovernmentFrameKind::Unknown => DiagnosticLedgerKind::Unknown,
        };
        push_ledger_entry(entries, kind, frame.span, government_frame_proof(frame));
    }
}

fn collect_punctuation_entries(
    slots: &[PunctuationSlot<'_>],
    entries: &mut Vec<DiagnosticLedgerEntry>,
) {
    for slot in slots {
        for mark in &slot.expected_marks {
            if slot.has_existing_mark(*mark) {
                continue;
            }
            push_ledger_entry(
                entries,
                DiagnosticLedgerKind::Punctuation,
                slot.span,
                punctuation_slot_proof(slot, *mark),
            );
        }
    }
}

fn collect_document_entries(
    context: &DocumentContext<'_>,
    entries: &mut Vec<DiagnosticLedgerEntry>,
) {
    for candidate in document_abbreviation_candidates(context) {
        let proof = document_abbreviation_proof(candidate.abbreviation, candidate.reason);
        push_ledger_entry(
            entries,
            DiagnosticLedgerKind::DocumentAbbreviation,
            candidate.abbreviation.first_span,
            proof,
        );
    }
    for candidate in document_style_candidates(context) {
        let proof = document_style_proof(&candidate);
        push_ledger_entry(entries, DiagnosticLedgerKind::DocumentStyle, candidate.span, proof);
    }
}

fn push_ledger_entry(
    entries: &mut Vec<DiagnosticLedgerEntry>,
    kind: DiagnosticLedgerKind,
    span: Span,
    proof: DiagnosticProof,
) {
    let status = if proof.is_actionable() {
        DiagnosticLedgerStatus::EmittedCandidate
    } else {
        DiagnosticLedgerStatus::Suppressed
    };
    entries.push(DiagnosticLedgerEntry {
        id: entries.len(),
        kind,
        status,
        span,
        proof,
    });
}

fn span_covering_list_items(items: &[ListItemCandidate]) -> Span {
    span_covering(items.iter().map(|item| item.span))
}

fn span_covering_headings(headings: &[HeadingCandidate]) -> Span {
    span_covering(headings.iter().map(|heading| heading.span))
}

fn span_covering(mut spans: impl Iterator<Item = Span>) -> Span {
    let Some(first) = spans.next() else { return Span::new(0, 0); };
    spans.fold(first, |acc, span| Span::new(acc.start.min(span.start), acc.end.max(span.end)))
}
