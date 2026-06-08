# Diagnostic ledger, punctuation slots, and document context

This layer makes diagnostics auditable. The engine should not only emit issues; it should also preserve candidates that were suppressed because the linguistic proof was unsafe.

## DiagnosticLedger

`DiagnosticLedger` is a document-local record of diagnostic candidates derived from facts:

- agreement graph conflicts;
- government frame conflicts;
- quantity conflicts;
- punctuation slots with missing expected marks;
- document-context style and abbreviation candidates.

Each entry stores:

- `kind` — agreement, government, quantity, punctuation, document style, document abbreviation;
- `status` — emitted candidate or suppressed;
- `span`;
- `DiagnosticProof`.

This matters because false positives are not debugged only by looking at emitted issues. The useful question is often: *why did the engine stay silent here?* The ledger gives a machine-readable answer.

## Coordination-backed PunctuationSlot

`PunctuationSlot` now uses two sources for comma expectation inside homogeneous-member contexts:

1. already extracted `CoordinationGroup` facts;
2. conservative asyndetic modifier pairs such as `умный быстрый анализатор`, where both neighboring words look like compatible modifiers and a nominal head follows.

This is intentionally narrow. It does not pretend to solve all homogeneous-member punctuation. It creates a safe place for future punctuation rules to inspect boundary evidence instead of doing left/right token hacks.

## Document-context detectors

Two strict seed detectors consume `DocumentContext`:

- `document_abbreviation_expansion` flags repeated abbreviations whose first mention has no detected expansion;
- `document_style_consistency` flags mixed list markers and mixed heading punctuation.

Both are style-level rules. They are useful for document analysis and UI/debug experiments, but they are deliberately tagged `strict-only` because domain-specific texts can have valid exceptions.

## Test doctrine

This layer has multi-fixture tests for:

- emitted and suppressed ledger entries;
- coordination punctuation slots;
- abbreviation detection;
- document style detection;
- fact-store summaries.

The tests are part of the design contract. New fact layers should show at least one positive path and one suppression/ambiguity path.
