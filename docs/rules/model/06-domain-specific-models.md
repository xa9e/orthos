## Domain-specific normalized models

### Orthography rules

Minimum model:

- `domain: orthography`;
- `rule_family`: `orthography`, `morphology_dependent`, `word_formation`, or `stress_dependent`;
- `pattern.kind`: `surface`, `regex`, `token_sequence`, `morphological`, `word_formation`, or `stress`;
- constraints such as `spelling_pattern`, `derivation`, `stress_requirement`, or `idiom_fixed_expression_exception`;
- explicit `requires` matching the selected family.

Good orthography rules distinguish **surface symptoms** from **linguistic causes**. A regex for `незнаю` can be implemented, but the normalized model must not pretend it solves all `не` + verb cases.

### Punctuation rules

Minimum model:

- `domain: punctuation`;
- `rule_family`: `punctuation` or `syntax_dependent`;
- `pattern.kind`: usually `punctuation_context`, `token_sequence`, `syntactic`, or `dependency`;
- constraints such as `clause_boundary`, `coordination`, `parenthetical_expression`, or `direct_speech`;
- high `false_positive_risk` unless the detector is purely typographic.

Russian punctuation is often syntax. Treating every comma rule as a token heuristic is a bug farm.

### Grammar rules

Minimum model:

- `domain: grammar`;
- `rule_family`: `grammar`, `morphology_dependent`, or `syntax_dependent`;
- `pattern.kind`: `morphological`, `syntactic`, or `dependency`;
- constraints such as `agreement`, `government`, or `coordination`;
- `requires` normally includes `morphology`, often `syntax`, sometimes `lexicon`.

Most grammar rules should stay `research` until morphology, ambiguity handling, and dependencies are testable.

### Morphology-dependent rules

Minimum model:

- `rule_family: morphology_dependent`;
- `pattern.kind: morphological`;
- constraints name POS/features/lemma/signature requirements;
- `requires: [morphology]`, plus tokenization/lexicon if needed.

Examples: `-тся/-ться`, adjective-noun agreement, participle agreement, numeral inflection.

### Syntax-dependent rules

Minimum model:

- `rule_family: syntax_dependent`;
- `pattern.kind`: `syntactic`, `dependency`, or `punctuation_context`;
- constraints include `clause_boundary`, `coordination`, `agreement`, `government`, `parenthetical_expression`, or `direct_speech`;
- `requires: [syntax]`, plus `morphology` when feature compatibility is checked.

Examples: subject-predicate agreement, comma around clauses, homogeneous members, direct speech punctuation.

### Word-formation rules

Minimum model:

- `rule_family: word_formation`;
- `pattern.kind: word_formation`;
- constraints include `derivation` and optionally `spelling_pattern`;
- `requires: [word_formation]`, usually with `morphology` and `lexicon`.

Examples: prefix-final `з/с`, `пре-`/`при-`, `пол-`, suffix alternations, compound words.

### Stress-dependent rules

Minimum model:

- `rule_family: stress_dependent`;
- `pattern.kind: stress` or `word_formation`;
- constraints include `stress_requirement`;
- `requires: [stress]`, usually with `word_formation`, `morphology`, and `lexicon`.

Examples: `о/е/ё` after sibilants and `ц`, suffix vowels whose norm depends on stress.

### Lexical/style rules

Minimum model:

- `domain: style` or a spelling/grammar domain with `rule_family: lexical_style`;
- `pattern.kind: lexical_set`;
- constraints include `lexical`, `style_register`, `government`, or `idiom_fixed_expression_exception`;
- `severity: info` or `warning` unless evidence is strong;
- `false_positive_risk: medium` or `high` by default.

Examples: paronyms, pleonasms, register mismatch, bureaucratic clichés, lexical collocation.

## Promotion policy

A rule should move from `research` to `planned` when:

- the phenomenon is normalized in this model;
- required analyzer capabilities are known;
- exceptions and false-positive risks are explicitly listed;
- at least one valid and one invalid executable example exists.

A rule should move from `planned` to `implemented` only when:

- the detector is executable;
- invalid examples fire;
- valid examples do not fire;
- the default profile decision is explicit;
- high-risk behavior is gated behind strict/research profiles or benchmarked.
