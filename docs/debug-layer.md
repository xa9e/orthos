# Debug layer

`orthos debug` is the machine-readable inspection path for deterministic grammar diagnostics.
It is intentionally separate from normal human output: ordinary `check` stays compact,
while `debug` emits the full `CheckResult` plus a structured `debug` snapshot.

## CLI

```bash
cargo run -- debug examples/bad.txt --rules rules --profile strict > debug.json
```

Useful slices:

```bash
jq '.debug.engine.rule_outputs[] | select(.raw_issue_count > 0)' debug.json
jq '.debug.analysis.fact_store.government_frames[] | select(.kind == "Verb")' debug.json
jq '.debug.language_model.verb_government.entries[] | select(.lemma == "говорить")' debug.json
```

## Snapshot shape

The current schema version is `4`.

The snapshot contains:

- `analysis.summary_before` and `analysis.summary_after`;
- optional token and morphology dumps;
- `fact_store.summary` (including `eliminated_readings`);
- `fact_store.disambiguation` with one proof per eliminated morphological reading;
- `fact_store.clause_boundaries` with marker/kind/confidence spans;
- government frames with governor/dependent/cases/confidence/blockers/conflict status;
- verb-government frame `model_ref` when a frame was produced from the language model;
- `engine.execution_plan`;
- per-rule raw/suppressed/emitted issue counts;
- verb-government model inventory.

`model_ref` is intentionally source-facing. For a verb-government frame it records:

- `lemma`: the registry lemma that matched the governor;
- `complement_kind`: direct or prepositional complement;
- `preposition`: the preposition required by the seed row, if any;
- `source_id`: the seed source identifier;
- `note`: the short seed-row explanation.

This lets a developer compare three layers without guessing:

1. `debug.language_model.verb_government.entries`: what the model knows;
2. `debug.analysis.fact_store.government_frames[].model_ref`: which model row fired;
3. `debug.engine.rule_outputs`: which rule emitted or suppressed the final issue.

The same source trace is also copied into `DiagnosticProof.facts` for emitted issues as `model_lemma`, `model_complement_kind`, `model_preposition`, and `model_source_id`.

## Development contract

A new model-backed detector should add tests at two levels:

1. fact/model tests: the expected linguistic fact exists in `LinguisticFactStore`;
2. rule/debug tests: the rule emits or suppresses the expected issue and debug explains why.

Do not use ad-hoc `println!` debugging as the contract. If a fact matters, expose it in the debug snapshot or in a proof object.


## Language-model fixture coverage

Debug schema version `2` added positive fixture coverage for verb government:

```bash
jq '.debug.language_model.verb_government | {entry_count, fixture_count, false_positive_fixture_count, entries_without_fixture}' debug.json
```

Expected healthy state for the built-in seed is:

```json
{
  "entry_count": 29,
  "fixture_count": 29,
  "false_positive_fixture_count": 8,
  "entries_without_fixture": []
}
```

If `entries_without_fixture` is non-empty, a model row was added without a regression fixture. Do not paper over that with a detector tweak; add the fixture and morphology row first.


## False-positive fixture visibility

Debug schema version `3` added `false_positive_fixture_count` for verb government.
This does not list every negative fixture by default; it gives a cheap
health signal that the model has a silence contract. The fixtures themselves live
in `data/grammar/verb_government.false_positive.tsv` and are enforced by the
model-backed Rust regression.

For island-based false positives, debug should still expose the candidate frame
with a blocker such as `DirectSpeechBoundary` or `ParenthesisBoundary`. That is
deliberate: the system can show why it did not emit, instead of making the
candidate disappear without explanation.


## Clause-boundary link safety

Debug schema version `4` adds `debug.analysis.fact_store.clause_boundaries`.
This is a reusable syntax signal, not a verb-government one-off. Verb-government
frames that cross an actionable subordinate marker such as `что` must stay visible
in `government_frames`, but with `ClauseBoundary` in `blockers` and `conflict: false`.

Useful slice:

```bash
jq '.debug.analysis.fact_store | {clause_boundaries, government_frames: [.government_frames[] | select(.blockers[]? == "ClauseBoundary")]}' debug.json
```

This protects the conservative policy from `IDEA.md`: model-backed grammar rules
should skip unsafe clause links instead of issuing confident diagnostics across
shallow syntax boundaries.
