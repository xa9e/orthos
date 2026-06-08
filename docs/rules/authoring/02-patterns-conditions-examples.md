## `pattern`

`pattern` describes the linguistic target a detector should find:

```yaml
pattern:
  kind: dependency
  description: Typed subject-predicate dependency with finite predicate morphology.
  captures: [subject, predicate]
```

Use `value` for compact literal/regex/list-like values only when it adds clarity. Use `captures` for stable roles (`head`, `dependent`, `subject`, `predicate`, `marker`, `left_context`, `right_context`).

Allowed `pattern.kind` values:

- `surface`
- `regex`
- `token_sequence`
- `morphological`
- `syntactic`
- `dependency`
- `punctuation_context`
- `word_formation`
- `stress`
- `lexical_set`

The detector may be a temporary seed. The pattern should describe the target model, not just today’s hack.

## `constraints`

`constraints` say when a candidate is really an error:

```yaml
constraints:
  - kind: agreement
    features: [case, number, gender]
    description: Dependent adjective must agree with the head noun.
```

Good constraints are reusable and analyzer-oriented. Use `features` for machine-readable feature names; use `description` to explain human context. A constraint must contain at least `value`, `description`, or `features`.

## `exceptions`

`exceptions` suppress false positives:

```yaml
exceptions:
  - kind: idiom_fixed_expression_exception
    value: curated_fixed_expression_id
```

Exceptions are not shameful. Russian spelling and punctuation are full of lexicalized forms, idioms, authorial punctuation, names, quotations, and registers. Document them instead of burying them in detector code.

## `evidence`

`evidence` records why the rule exists or why it is worth implementing:

```yaml
evidence:
  - kind: normative_source
    source_ref: gramota_lopatin_full
  - kind: donor_taxonomy
    source_ref: lorugec_dataset
    note: Taxonomy signal only; no raw rows used.
```

When `evidence.source_ref` is present, it must match a declared `source_refs[].id`. Use `note` for expert review or design rationale that does not point to a specific source.

Allowed evidence kinds:

- `normative_source`
- `corpus_attestation`
- `donor_taxonomy`
- `benchmark`
- `lexicon`
- `morphology`
- `syntax`
- `stress_dictionary`
- `expert_review`

## `examples.valid` and `examples.invalid`

Examples are executable specs, not decoration:

```yaml
examples:
  valid:
    - Согласно приказу, встреча перенесена.
  invalid:
    - Согласно приказа, встреча перенесена.
```

Rules:

- implemented rules must include at least one valid and one invalid example;
- examples must be short, original, and license-safe;
- do not duplicate examples within or across `valid` and `invalid`;
- valid examples must not trigger the same rule;
- invalid examples must trigger the same rule when `cargo run -- test-examples --rules rules` is available;
- keep dataset examples out unless licensing and task ownership explicitly allow them.

For planned/research rules, examples still document intended behavior, but they are not a substitute for a detector.
