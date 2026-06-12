# Roadmap

Long-term plan for Orthos: a rule-first, deterministic, explainable grammar
checker for Russian. The north star from `IDEA.md`: formalize the language as
deeply as a rule-first system allows, instead of imitating intelligence with a
pile of regexes.

## Where the engine stands

Implemented today:

- token/morphology/syntax fact pipeline (`LinguisticFactStore`): ambiguity
  model, syntactic islands, clause boundary map, nominal groups, agreement
  graph, coordination groups, punctuation slots, government frames,
  document context, diagnostic ledger;
- ~50 detector kinds wired to a YAML rule corpus with executable examples;
- conservative agreement/government rules backed by typed morphology and
  machine-readable proofs;
- ё/е-folded lexicon lookup with explicit-ё disambiguation;
- dataset regression contract (precision + recall claims per curated record);
- benchmark tooling for GEC-style datasets (LORuGEC, RLC, RuSpellGold).

Structural weaknesses to attack, in order of leverage:

1. ambiguity throttles recall: most model-backed rules require unanimous
   analyses, and most Russian word forms are ambiguous;
2. clause segmentation is heuristic and shallow; participial/adverbial
   constructions are not modeled as units;
3. no general dependency-like relation layer: each rule re-derives its own
   token adjacency logic;
4. the bundled lexicon is tiny; the OpenCorpora import path exists but the
   full-dictionary experience is not turnkey;
5. evaluation numbers are proxies; per-domain precision/recall is not yet a
   release gate.

## Design invariants (do not trade away)

- Deterministic, explainable rule engine; ML at most for ranking, never as
  the foundation.
- Every elimination of a hypothesis (morphological reading, clause split,
  diagnostic suppression) leaves a machine-readable trace.
- Conservative by default: prefer silence over confident nonsense.
- Linguistic facts live in declarative inventories (YAML, TSV seeds, typed
  Rust tables), never as ad-hoc string comparisons inside detectors.
- Every model-backed rule ships with true-positive and false-positive
  fixtures, and dataset regressions where a real corpus example exists.

## Workstream A: language model core

A1. **Contextual morphological disambiguation (constraint-grammar style).**
Reduce per-token reading sets using high-precision local constraints
(preposition case government, modifier-head agreement consistency,
impossible POS sequences), with eliminations recorded as proofs and the
last reading never removed. Exit criteria: measurable recall gain on
agreement/government rules on dataset slices without precision loss.

A2. **Clause and construction structure.** Promote clause boundaries from
marker heuristics to a clause model that knows: subordinator type,
participial/adverbial-participial turns, comparative turns, parenthesis-like
insertions. This unlocks the largest family of Russian comma rules
(обособление). Exit: punctuation rules for participial turns with curated
fixtures.

A3. **Dependency-lite relation layer.** A typed, conservative head-dependent
relation builder (subject, predicate, modifier, complement, coordination)
on top of the disambiguated readings — not a full parser, but a shared
substrate so rules stop re-implementing adjacency scans. Exit: subject and
predicate detection across intervening modifiers, used by at least two rules.

A4. **Word formation and morphemics.** Grow the derivational seed inventory
(prefixes, suffixes, alternations) into a segmentation model strong enough
for prefix-assimilation, `пол-` compounds, and hyphenation rules with proofs.

A5. **Quantity and numeral system.** Finish the compound numeral model
(typed components, case percolation, paucal forms) — Russian numerals are a
permanent error source and a showcase for the typed approach.

## Workstream B: rule corpus

- Systematize Lopatin/Rozental punctuation sections into normalized YAML with
  per-rule status, risk, and capability requirements (continue
  `rules/55-punctuation-systematics`).
- Orthography systematics: unstressed vowels (checkable via derivation),
  prefix families, `н/нн`, particles `не/ни`.
- Grammar systematics: government dictionary growth (verb valencies from
  seed lists), agreement edge cases (collective numerals, profession nouns).
- Each batch lands as: rules YAML + executable examples + fixtures + dataset
  regressions where possible.

## Workstream C: data and evaluation

- Make the OpenCorpora import + binary cache a one-command setup; document
  licensing boundaries.
- Grow `gec_dataset_regressions.jsonl` continuously: every fixed noise class
  and every new detection capability gets a record with explicit
  precision/recall expectations.
- Per-domain precision/recall dashboards from benchmark runs; define release
  gates (e.g. target false-positive rate per 1k tokens on corrected targets).
- Add a stress-test corpus of adversarial well-formed Russian (quotes, poetry,
  dialogue, OCR artifacts) for false-positive control.

## Workstream D: engine and platform

- Performance budget: lexicon mmap/lazy loading, fact store reuse across
  rules, sub-100ms checks for editor-sized buffers with the full dictionary.
- Suppression UX: inline directives, profile files, per-project config.
- Stable JSON diagnostic schema with proofs, for editor and CI integrations.

## Workstream E: productization (later, after A1–A3 mature)

- LSP server over the JSON diagnostics.
- WASM build for in-browser checking.
- Editor plugins and a CI action.
- Public benchmark page with honest per-domain metrics.

## Sequencing

1. **M1 — disambiguation core (A1)**: constraint engine + traces + dataset
   evidence.
2. **M2 — full-dictionary DX (C)**: turnkey OpenCorpora setup, benchmark
   gates on real lexicon.
3. **M3 — clause model (A2)**: participial/adverbial обособление with
   fixtures.
4. **M4 — relation layer (A3)** + subject-predicate rules v2.
5. **M5 — corpus sprints (B)** interleaved with noise-class regressions (C).
6. **M6 — platform (D, E)**: LSP + WASM once the linguistic core proves
   itself on benchmarks.

Each milestone must leave the suite green, the docs in sync, and the dataset
regression file strictly larger.
