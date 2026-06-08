# Corpus design

## Why YAML rules, not hard-coded Rust

Rules are linguistic data. The engine should be deployable independently of the corpus, and the corpus must be reviewable by linguists without rebuilding the binary.

## Rule ID convention

Format:

```text
ru.<domain-or-subsystem>.<phenomenon>.<scope>
```

Examples:

```text
ru.punctuation.no_space_before_mark
ru.orthography.particle_to_libo_nibud_hyphen
ru.grammar.subject_predicate_agreement_basic
ru.punctuation.kak_predicative_no_comma
```

ID rules enforced by metadata validation:

- must start with `ru.`;
- dot-separated components must be non-empty;
- only ASCII lowercase letters, digits, `_`, and `.` are allowed;
- ids must be unique across all loaded YAML files.

## Rule lifecycle

- `implemented`: executable by the current engine and backed by examples that pass.
- `planned`: linguistically clear, but requires a missing detector, lexicon, or data source.
- `research`: depends on unresolved ambiguity, syntax, morphology, stress, semantics, or benchmark evidence.

A rule must not be marked `implemented` while using a `manual` detector. Implemented rules must have at least one invalid example.

## Required metadata

Each rule should declare:

- `id`;
- `title`;
- `domain`;
- `level`;
- `status`;
- `severity`;
- `source_refs`;
- `requires`;
- `detector`;
- safe minimal examples.

`source_refs` must resolve to corpus-level `source_refs`. New source ids should be added once and reused.

## `requires` discipline

`requires` describes the detector capability needed for a responsible implementation, not merely what the current placeholder detector does.

Typical capability mapping:

- typography and spacing: `tokenization` or `regex`;
- closed lexical phrase maps: `tokenization`, `lexicon`;
- agreement, government, forms, part of speech: `morphology`;
- clauses, dependencies, homogeneous members, detached phrases: `syntax`;
- prefixes, suffixes, roots, derivation, compound spelling: `word_formation`;
- `о/ё/е`, suffixes dependent on stress: `stress`;
- collocation, pleonasm risk, replacement ranking: `semantics`, often `benchmark`.

The metadata validator now rejects rules whose text clearly names morphology, syntax, word formation, or stress blockers but omits the corresponding `requires` capability.

## Detector strategy

1. Surface detectors: spaces, punctuation adjacency, duplicated tokens, quotes.
2. Lexical detectors: limited dictionaries and closed lists.
3. Morphological detectors: POS, lemma, case, number, gender, tense, aspect.
4. Word-formation detectors: morpheme boundaries, producing stem, suffix/prefix classes.
5. Syntactic detectors: dependency tree, clause boundaries, homogeneous members.
6. Hybrid rankers: rule candidate generation plus statistical ranking to suppress false positives.

## Why not just regex

Russian has free word order, rich morphology, homonymy, ellipsis, stress-sensitive spelling, and many clause-boundary effects. Regex is acceptable for typography and some low-risk orthography. It is a bad fit for agreement, government, participial clauses, most comma rules, and word formation. Pretending otherwise just manufactures false positives with confidence.
