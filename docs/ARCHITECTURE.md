# Orthos architecture

Orthos is a deterministic Russian proofreading engine built around explicit rule, morphology, syntax, and diagnostic-proof layers.

## Hard goals

- Prefer deterministic, explainable rules over ML.
- Treat the rule corpus as source code.
- Make every implemented rule testable through examples.
- Keep morphology and syntax behind explicit interfaces.
- Keep module boundaries narrow enough for independent feature work.

## Current module boundaries

```text
src/
  corpus.rs      YAML schema, corpus loading, rule/source validation
  detector.rs    detector dispatch and detector implementations
  engine.rs      orchestration: corpus + morphology + detector context
  issue.rs       diagnostic model
  morph.rs       demo morphology façade and agreement primitive
  text.rs        tokenization, spans, line/column, Unicode helpers

rules/
  00-basic.yaml              original base rules and sources
  05-donor-surface.yaml      ru-guard typography/punctuation donors
  10-orthography.yaml        original orthography seed
  15-donor-orthography.yaml  ru-guard orthography donors
  20-punctuation.yaml        original punctuation seed
  30-grammar.yaml            original grammar roadmap
  35-donor-grammar-style.yaml rulang/ru-guard grammar/style donors
  90-roadmap-ruslint.yaml    research backlog distilled from ruslint

data/
  lexicon/demo_morph.tsv     tiny demo lexicon; not production morphology

tests/
  basic.rs                   smoke/regression tests
  corpus_examples.rs         executable rule examples
```

## Detector strategy

The schema is still a typed `Detector` enum. That is intentional for the current stage: bad YAML should fail early. New detector families must be added with:

1. a detector variant in `src/corpus.rs`;
2. one thin dispatch arm in `src/detector.rs`;
3. the implementation function or module;
4. at least one YAML rule;
5. tests in `tests/basic.rs` or generated through `examples`.

When detector count grows past roughly 40, split `src/detector.rs` into:

```text
src/detectors/
  mod.rs
  context.rs
  registry.rs
  surface.rs
  lexical.rs
  morphology.rs
  syntax.rs
```

Do not create a single god-engine. That is how these projects rot.

## Rule lifecycle

Each rule has:

- `implemented` — executable and tested;
- `planned` — clear enough, blocked by a known capability;
- `research` — linguistically nontrivial or needs corpus work.

Implemented rules should normally include:

- `source_refs`;
- `examples.valid`;
- `examples.invalid`;
- `requires`;
- `explanation` for non-obvious or risky rules;
- `suggestion` if correction is possible.

## Capability flags

`requires` is the capability contract. It tells the engine what linguistic layer a rule needs.

Current values:

- `tokenization`
- `sentence_boundaries`
- `regex`
- `lexicon`
- `morphology`
- `syntax`
- `semantics`
- `named_entities`
- `word_formation`
- `stress`
- `benchmark`

## Reused rule families

The current rule corpus includes:

- additional surface detectors;
- `-тся/-ться` heuristic;
- demo morphology and adjective-noun agreement;
- explanations/suggestions/examples discipline.

The research backlog includes:

- roadmap categories for hard orthography, grammar, punctuation;
- explicit future capabilities.

Additional grammar/style seeds include:

- phrase-level seed rules for pleonasms, government, numerals;
- the idea that detector logic should become trait/registry-based later.

## Non-goals for this fork

- No neural correction model.
- No automatic downloading of large datasets.
- No production-grade morphology yet.
- No claim that phrase seeds are linguistically complete.

Phrase seeds are scaffolding. They must migrate to morphology/syntax-backed rules as the language model matures.
