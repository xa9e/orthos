# Evaluation methodology

The evaluation pipeline measures **deterministic diagnostics** emitted by `orthos` on normalized Russian correction datasets. It does not train models and it does not score generated corrections.

## Current benchmark contract

Input is normalized JSONL produced by `scripts/import/`:

```json
{
  "id": "dataset/local-id",
  "dataset": "rlc-annotated|lorugec|ruspellgold|custom",
  "source_file": "original member or local path",
  "input": "source sentence expected to contain an issue",
  "correction": "single corrected target when available",
  "targets": ["one or more corrected targets"],
  "edits": [],
  "labels": [],
  "metadata": {}
}
```

Dataset-specific data must stay in `metadata`, `labels`, or `edits`; new importers must not add ad-hoc top-level keys. That stability matters because benchmark scripts, CI fixtures, and downstream reports consume the same shape.

`data/dataset-registry.json` records the local archive names, importer commands, output paths, licensing status, and limitations for the supported datasets. The registry is operational metadata, not a scoring definition and not a license grant.

## What the benchmark can measure now

`scripts/eval/benchmark_jsonl.py` runs the checker on `input` sentences and on corrected targets, then reports:

- configuration: benchmark mode, input/output path, checker command, profile, rules path, morphology lexicon, extra checker args;
- example, token, and expected dirty/clean counts;
- diagnostics on source inputs;
- diagnostics on corrected targets;
- diagnostics per 1,000 sentences;
- diagnostics per 1,000 tokens;
- corrected-target false-positive sentence rate;
- corrected-target false-positive diagnostics per 1,000 target tokens;
- examples with unexpected diagnostics on corrected targets;
- a **recall proxy**: dirty source examples with at least one diagnostic divided by dirty source examples;
- a **precision proxy**: detected dirty source examples divided by detected dirty source examples plus corrected-target false-positive sentences;
- **F0.5 proxy** over the two proxy values;
- coarse label-domain hit rate for labels that can be mapped to `orthography`, `punctuation`, `grammar`, or `style`;
- full issues by rule id;
- issues by dataset label;
- detected records by dataset label;
- runtime summary: total seconds, checker seconds, checker invocation count, average checker milliseconds.

These numbers are suitable for regression tracking and for answering: “Did this rule change make the checker noisier or less responsive on known examples?”

## What the benchmark cannot measure yet

The current diagnostics are not a full GEC scorer. Specifically:

- it does not compute exact M2/ERRANT precision, recall, or F-scores;
- it does not align checker spans to source edit spans across all datasets;
- it does not verify that a diagnostic’s suggested replacement equals the dataset correction;
- it does not handle multi-reference scoring beyond choosing the first available target for corrected-target false-positive checks;
- it does not distinguish “right sentence, wrong rule id” from “right broad domain” except through the weak label-domain hit rate;
- it does not estimate severity, user-visible usefulness, or explanation quality;
- it does not resolve whether a corrected-target diagnostic indicates a noisy checker, an imperfect reference, or a valid issue outside the dataset annotation scope.

The proxy metrics are intentionally named as proxies. Reporting them as canonical GEC quality would be grade-A metric laundering.

## Dataset-specific caveats

### RLC-GEC / rlc-annotated

RLC sentence rows include corrected text and annotations. `annotations.csv` has token-boundary `start`/`end` fields, but checker diagnostics currently use textual positions. Exact span scoring therefore needs a future alignment layer.

`rlc_test.csv` contains edit rows; the importer groups them into sentence-pair records to avoid duplicate benchmark examples.

### LORuGEC

The workbook source preserves rule metadata and grammar sections. Those labels are useful for coarse grouping, but spreadsheet rows do not provide source spans. The M2 files do provide token spans, but they carry less rule metadata than the workbook. Choose the source deliberately:

- `--source xlsx` for metadata-rich rule and section analysis;
- `--source m2` for token-span-oriented future scoring work.

No explicit redistribution license was present in the referenced archive metadata, so full raw files and generated full JSONL dumps should remain local-only unless maintainers publish clear terms.

### RuSpellGold

RuSpellGold is mostly useful for spelling/orthography false-positive and recall smoke checks. It has source/correction pairs and domains, but no detailed labels or edit spans. The importer assigns the coarse label `spelling`.

The dataset metadata indicates Apache-2.0, while the README also warns that underlying source-text copyrights remain with original authors or publishers. Keep full normalized dumps out of the repository by default.

### Generic pair files

`gec_pairs_to_jsonl.py` is for tiny ad-hoc TSV/CSV files. It has no native labels, spans, domains, or provenance beyond the configured columns. Use dataset-specific importers when available.

## Avoiding overfitting

Rules must not be shaped to memorize benchmark sentences. Practical guardrails:

- inspect false positives on corrected targets before celebrating source hits;
- inspect `unexpected_target_diagnostic_examples`, not only aggregate rates;
- prefer broad linguistic conditions over literal sentence fragments;
- keep fixture examples tiny and representative, not exhaustive;
- add counterexamples when adding rules;
- review top corrected-target rule ids after every detector/rule change;
- do not tune thresholds against one dataset without checking at least one other dataset family;
- separate `implemented`, `planned`, and `research` rules in the corpus so aspirational taxonomy does not pollute release metrics.

## CI fixture policy

CI should run only tiny committed fixtures under `testdata/fixtures/import/`. Full third-party archives and generated JSONL files stay out of Git by default:

```text
data/eval/
reports/eval/
```

Recommended CI checks:

```bash
python -m pytest tests/test_importers.py
python -m compileall -q scripts tests
cargo fmt -- --check
cargo test
cargo run -- validate --rules rules
cargo run -- test-examples --rules rules
```

When `pytest` is unavailable, run the importer scripts directly against fixtures and then run the benchmark with a fake checker or compiled local binary.

## Interpreting proxy metrics

A good release should usually have:

- stable or improving source recall proxy;
- low corrected-target diagnostics per 1,000 tokens;
- low false-positive target sentence rate;
- no sudden spike in one rule id on corrected targets;
- runtime movement explained by a deliberate detector change.

A bad release often looks like a recall jump paired with a corrected-target false-positive spike. That means the checker got louder, not necessarily smarter.
