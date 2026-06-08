# Roadmap toward a serious Russian grammar checker

## Phase 1: deterministic surface checker

- Implement typography and simple punctuation rules.
- Build stable YAML schema.
- Add golden tests for every rule.
- Add JSON output compatible with editor integrations.

## Phase 2: morphology

- Import or bind a Russian morphological dictionary/analyzer.
- Store tags in a compact internal format.
- Add ambiguity-preserving analyses; do not pick one parse too early.
- Implement `не` + verb, `тся/ться`, cases after prepositions, adjective-noun agreement.

## Phase 3: syntax

- Train or integrate a dependency parser.
- Support clause boundaries.
- Implement participial/adverbial-participial punctuation.
- Implement subject-predicate and coordination rules.

## Phase 4: ranking and false-positive control

- Generate candidates with rules.
- Rank candidates using context features and corpus statistics.
- Add a suppression layer for quotations, poetry, chats, code, URLs, names, dialectal spelling.

## Phase 5: productization

- Language Server Protocol.
- WASM build.
- batch CLI.
- editor plugins.
- benchmark suite with precision/recall, latency and memory budgets.
