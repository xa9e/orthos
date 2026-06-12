# Contextual morphological disambiguation

Most Russian word forms are morphologically ambiguous, and most model-backed
rules refuse to act on ambiguous tokens. Without contextual disambiguation
the engine is precise but mute. This layer removes readings that are
impossible in their local context — in the constraint-grammar tradition:
deterministic, rule-based, and fully explainable.

## Position in the pipeline

`LinguisticFactStore::from_tokens_with_analyses` runs
`disambiguate_readings` before any other fact layer is built. Every
downstream model (ambiguity classes, agreement graph, coordination,
punctuation slots, government frames) sees the reduced reading sets.

## Safety invariants

- a token never loses its last reading;
- the licensing token must be reliable before it can eliminate readings of a
  neighbour (e.g. a preposition in *all* of its readings);
- if no consistent combination exists at all — a likely real grammar error —
  the engine does not prune, so the error stays visible to detectors;
- every elimination is recorded as a typed `ReadingElimination` proof with
  the constraint id, the evidence token, and a human-readable explanation;
- constraints run to a bounded fixpoint (`MAX_DISAMBIGUATION_PASSES`),
  because one elimination can make a neighbour reliable enough to license
  the next one.

## Implemented constraints

| Constraint | Effect |
| --- | --- |
| `preposition_verb_exclusion` | finite verb / gerund readings cannot directly follow a preposition («из стали» loses the «стать» reading) |
| `preposition_case_government` | a preposition from the government registry intersects the case set of the adjacent nominal («у дома» keeps only the genitive reading) |
| `modifier_head_agreement` | adjacent full-form modifier + noun keep only readings participating in at least one agreeing combination («новые дома» resolves both tokens to nominative plural) |

## Observability

- `LinguisticFactStoreSummary.eliminated_readings` counts eliminations;
- the debug snapshot (`orthos debug`) carries the eliminations under
  `fact_store.disambiguation`, capped by `max_fact_items`;
- `LinguisticFactStore::disambiguation()` exposes the trace to rule code.

## Growth path

- extend the preposition seed with multi-case prepositions (`в`, `на`, `с`,
  `по`, `за`) — the single largest recall lever for this layer;
- number/numeral constraints (paucal forms after «два/три/четыре»);
- pronoun and particle homonymy (`то`, `это`, `всё`) via clause context;
- cross-token windows beyond direct adjacency once the relation layer
  (roadmap A3) exists.
