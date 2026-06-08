# Verb government model

The verb-government seed lives in:

```text
data/grammar/verb_government.seed.tsv
```

The Rust entry point is `VerbGovernmentRegistry::russian_seed()`.

## Data schema

```text
lemma	complement_kind	preposition	cases	source_id	note
```

Supported `complement_kind` values:

- `direct_object`: `ждать ответа`, `управлять проектом`, `помогать проекту`;
- `prepositional_object`: `говорить о задаче`, `надеяться на успех`.

For `direct_object`, `preposition` must be empty.
For `prepositional_object`, `preposition` is mandatory.

## Runtime path

1. Morphology supplies verb lemmas for the governor token.
2. `VerbGovernmentRegistry` finds matching valency entries.
3. Syntax builds `GovernmentFrameKind::Verb` frames.
4. `ru.grammar.verb_government_basic` consumes conflicting frames.
5. Debug exposes both the inventory and the built frames.
6. Verb-government frames carry `model_ref`, pointing back to the seed row that produced them.

## Safety policy

The model must remain conservative:

- no issue without nominal morphology for the dependent head;
- no issue across unsafe syntactic islands;
- no issue across actionable clause boundaries such as unpunctuated `что`;
- no prepositional-object shortcut through the generic preposition detector;
- no direct-object fallback when the registry only knows a prepositional complement.

## Validation

Run:

```bash
python scripts/morph/validate_language_model_data.py
cargo test verb_government
cargo run -- debug examples/bad.txt --rules rules --profile strict > debug.json
```

## Debug traceability

Every verb-government frame produced from the registry should be traceable back to a seed row. The `model_ref` attached to the frame stores `lemma`, `complement_kind`, optional `preposition`, `source_id`, and `note`. If a future detector needs a seed fact but cannot expose it through this reference, the debug contract is incomplete. Emitted verb-government issues should also copy the same source trace into `DiagnosticProof.facts`, so a normal JSON issue is useful even without a full debug snapshot.


## Fixture coverage contract

The model has a second checked-in data table:

```text
data/grammar/verb_government.fixtures.tsv
```

Each seed row must have exactly one fixture key with:

- `valid_text`: should not emit `ru.grammar.verb_government_basic`;
- `invalid_text`: should emit exactly one `ru.grammar.verb_government_basic` issue;
- `invalid_excerpt`: should be contained in the emitted issue excerpt.

The compact morphology for these fixtures lives in:

```text
data/grammar/verb_government.fixture_morph.tsv
```

Run the aggregate data validator before touching model data:

```bash
python scripts/morph/validate_language_model_data.py
```

Debug schema exposes coverage through `debug.language_model.verb_government.fixture_count`, `false_positive_fixture_count`, and `entries_without_fixture`. New seed rows with missing positive fixtures should be treated as incomplete work, not as a harmless TODO.


## False-positive fixture contract

The model also has a negative fixture table:

```text
data/grammar/verb_government.false_positive.tsv
```

These rows protect the conservative boundary of the model. A false-positive
fixture must emit zero `ru.grammar.verb_government_basic` issues. If it names an
`expected_blocker`, debug must still show the model-backed frame and the blocker,
for example `DirectSpeechBoundary` for direct speech or `ParenthesisBoundary` for
parentheticals. Empty `expected_blocker` means the syntax path should not build a
frame at all, commonly because the token gap is not a plain whitespace link.


## Clause-boundary false positives

The false-positive table now includes subordinate-marker cases such as:

```text
проверять что проекту
говорить о что задачу
```

These examples are intentionally synthetic boundary probes. They are not meant as
polished Russian prose; they prove that the verb-government model does not link a
governor to a dependent through an actionable clause marker. The expected debug
shape is a visible model-backed frame with `ClauseBoundary` in `blockers` and no
emitted `ru.grammar.verb_government_basic` issue.

The boundary itself is produced by `ClauseBoundaryMap`, which lives in the syntax
layer and is intended to be reused by future agreement/government/punctuation
rules. Do not reimplement this as local detector punctuation logic.
