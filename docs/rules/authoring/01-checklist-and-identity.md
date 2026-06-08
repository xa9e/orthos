# Rule authoring guide

This guide is the working contract for adding deterministic Russian-proofreading rules to `orthos`. The rule corpus is source-grounded metadata plus executable examples. A rule is not acceptable just because a regex can be written for it.

## Authoring checklist

Before opening a change, verify that the rule has:

- a stable id using `ru.<namespace>.<snake_case_slug>`;
- `domain`, `rule_family`, `level`, `status`, and `severity` chosen independently;
- `requires` matching every analyzer capability implied by the detector, pattern, constraints, and prose;
- `pattern` describing the linguistic match target, not just the current detector implementation;
- `constraints` and `exceptions` for conditions that create or suppress the issue;
- `evidence` and `source_refs` pointing to declared corpus sources;
- `examples.valid` and `examples.invalid` that are short, original, and executable;
- `confidence` and `false_positive_risk`, mandatory for implemented rules and strongly recommended for all planned/research rules;
- no copied source paragraphs, raw dataset rows, or benchmark examples whose license is unclear.

## Stable rule id

Use this shape:

```text
ru.<namespace>.<snake_case_slug>
```

Examples:

```text
ru.typography.multiple_spaces
ru.orthography.ne_verbs_full
ru.punctuation.introductory_phrase_comma_basic
ru.grammar.preposition_case_phrase_seed
ru.syntax.subject_predicate_agreement_roadmap
```

Rules:

- prefix is always `ru`;
- each dot-separated component after `ru` starts with an ASCII lowercase letter;
- components use compact `snake_case`: lowercase ASCII letters, digits, and single underscores;
- no empty components, trailing underscores, uppercase letters, spaces, hyphens, or source names;
- the id names the checked phenomenon and scope, not the book/dataset where it was noticed.

`namespace` is normally the user-visible domain (`orthography`, `punctuation`, `grammar`, `style`, `typography`) but may be an architectural namespace such as `syntax` when the phenomenon is deliberately tracked that way. Do not rename an existing id unless the old rule is superseded explicitly.

## `domain`

`domain` is the broad user-facing category:

- `typography` — spaces, quotes, brackets, dash/hyphen shape, technical text mechanics;
- `orthography` — spelling norms;
- `punctuation` — punctuation placement or punctuation shape;
- `grammar` — agreement, government, declension, syntactic compatibility;
- `style` — lexical, register, semantic, cliché, or editorial advice.

Do not use `domain` to hide analyzer complexity. A visible spelling error can still require morphology, syntax, stress, or word formation.

## `rule_family`

`rule_family` describes implementation architecture:

- `typography` — deterministic surface text mechanics;
- `orthography` — spelling checks that do not need a more specific analyzer family;
- `punctuation` — punctuation checks that are mostly surface/contextual;
- `grammar` — grammar checks with bounded morphology/syntax needs;
- `morphology_dependent` — POS/features/lemma/form analysis is central;
- `syntax_dependent` — clause, dependency, or constituent structure is central;
- `word_formation` — morpheme/stem/prefix/suffix/derivation logic is central;
- `stress_dependent` — stress position or stress dictionaries are central;
- `lexical_style` — curated lexical sets, collocations, idioms, register, paronyms.

When uncertain, choose the family that explains what would make false positives hard to control. That usually beats the tempting but wrong “regex rule” label.

## `level`

Use `level` to document analyzer depth and expected complexity:

- `basic` — surface-deterministic checks;
- `school` — common school norm, often needs small lists or morphology;
- `intermediate` — limited context and exceptions;
- `advanced` — morphology, syntax, lexicon, stress, or word formation;
- `expert` — semantic/register/valency-heavy logic;
- `heuristic` — intentionally incomplete detector with known false positives.

`level` is not severity. A basic typography rule can be an `error`; an expert rule can be `info`.

## `status`

- `implemented` — executable detector exists and embedded examples must be runnable specs;
- `planned` — the rule is linguistically stable, but the detector is missing or incomplete;
- `research` — the rule, analyzer strategy, evidence, or false-positive policy is not mature.

Implemented rules must not use `detector.kind: manual`. Keep hard Russian grammar and punctuation rules in `research` until analyzer prerequisites are real, not aspirational.

## `severity`

- `error` — high-confidence normative issue;
- `warning` — likely issue, incomplete detector, or context-dependent norm;
- `info` — style, ambiguity, weak evidence, or advisory signal.

When the norm is stable but the current detector is crude, lower severity or raise `false_positive_risk`; do not pretend the detector is better than it is.

## `requires`

Declare every capability that the detector or model needs:

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

Validation enforces several deterministic implications:

- `pattern.kind: morphological` requires `morphology`;
- `pattern.kind: syntactic` or `dependency` requires `syntax`;
- `pattern.kind: word_formation` requires `word_formation`;
- morphology, syntax, word-formation, stress, lexicon, and detector-specific cues in metadata must be reflected in `requires`.

Prefer over-declaring real prerequisites to under-declaring them. Under-declared rules become execution-planning landmines.
