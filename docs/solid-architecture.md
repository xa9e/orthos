# SOLID architecture notes

The current project should be treated as a layered rule engine, not as a pile of regex checks.

## Layer boundaries

```text
Corpus YAML
  -> corpus schema / validation
  -> analysis context
  -> morphology provider
  -> shallow syntax document
  -> detector registry
  -> engine execution plan
  -> diagnostics / evaluation
```

## Module responsibilities

### `corpus`

Owns YAML shape and validation. It must not run detectors, tokenize text, or know about CLI behavior.

### `analysis`

Owns reusable per-document caches. It is the place for cheap deterministic layers shared by detectors.

Current caches:

- line index;
- token stream;
- word token stream;
- morphology provider reference.

Future caches can be added here when they are document-wide and reusable.

### `detector`

Owns detector runners and detector registry contracts. It should not own document analysis state.

A detector runner should be small, deterministic, and explicit about required capabilities.

### `engine`

Owns rule selection, execution planning, suppression, execution strategy, and result aggregation. It should not know the internals of a specific linguistic rule.

### `morph`

Owns morphology abstractions, dictionary import boundaries, and grammatical compatibility primitives.

Large dictionaries must stay outside the repository unless licensing and size are explicitly approved.

### `syntax`

Owns shallow deterministic syntax primitives used by punctuation and grammar rules. It is not a full parser and must stay conservative.

### `scripts/import` and `scripts/eval`

Own external corpus ingestion and benchmark orchestration. They normalize data into small, explicit JSONL contracts.

## Refactoring rules for future work

1. Do not add a detector by extending a central `match` when a `DetectorRunner` can be registered instead.
2. Do not make detectors retokenize text directly; use `DetectorContext` / `AnalysisContext` caches.
3. Do not put linguistic taxonomy decisions inside CLI code.
4. Do not let dataset importers mutate rule YAML.
5. Do not commit raw external datasets by default.
6. New morphology providers must implement `MorphAnalyzer` and document their ambiguity behavior.
7. New syntax-based rules must go through `SyntaxDocument` or a documented extension of it.
8. Every implemented rule needs valid and invalid examples.
9. Every source-derived rule needs source metadata and a licensing note when relevant.
10. Evaluation metrics must clearly distinguish proxy metrics from true GEC metrics.
