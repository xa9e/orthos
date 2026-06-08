## Core fields

### `domain`

A broad user-facing subsystem:

- `typography` — surface text mechanics: spaces, quote pairing, dash/hyphen shape;
- `orthography` — spelling norms;
- `punctuation` — punctuation norms;
- `grammar` — agreement, government, declension, syntactic compatibility;
- `style` — lexical/style/semantic advice.

`domain` is intentionally coarse. Use `rule_family` for implementation architecture.

### `level`

Complexity and expected analyzer depth:

- `basic` — deterministic surface rule;
- `school` — common normative rule, may still require morphology;
- `intermediate` — needs limited context or lists;
- `advanced` — morphology/syntax/lexicon-heavy;
- `expert` — semantic, register, valency, or hard exception logic;
- `heuristic` — intentionally incomplete implementation with known false positives.

### `status`

- `implemented` — executable detector exists, invalid examples must fire;
- `planned` — rule is linguistically clear but implementation is missing/incomplete;
- `research` — linguistic, architectural, or data evidence is not yet strong enough.

### `severity`

- `error` — high-confidence normative error;
- `warning` — likely issue or incomplete detector;
- `info` — style, ambiguity, or weak evidence.

## Rule families

`rule_family` describes the implementation family, not just the visible error category.

Allowed values:

- `typography` — byte/char/token surface checks;
- `orthography` — spelling rules that can be checked with regex, tokens, lexicon, morphology, or word formation;
- `punctuation` — punctuation placement or punctuation shape;
- `grammar` — agreement, government, valency, declension, compatibility;
- `morphology_dependent` — requires POS/features/lemma/form analysis;
- `syntax_dependent` — requires clause/dependency/constituent structure;
- `word_formation` — requires morphemes, stems, prefixes, suffixes, derivation class;
- `stress_dependent` — requires stress position or stress dictionary;
- `lexical_style` — lexical compatibility, register, paronyms, clichés, idioms.

A rule can still have `domain: orthography` and `rule_family: stress_dependent`; this is exactly the point. For example, `о/е/ё` after sibilants is visible as spelling but architecturally depends on stress and word formation.

## Confidence and false-positive risk

`confidence` is the expected correctness of the rule model plus available detector strategy:

- `high` — deterministic or near-deterministic with bounded exceptions;
- `medium` — good norm, but implementation has non-trivial ambiguity;
- `low` — research/planned rule where analyzers or evidence are immature.

`false_positive_risk` is independent:

- `low` — accidental firing should be rare;
- `medium` — context can flip the judgment;
- `high` — the rule needs disambiguation, lexicon, syntax, benchmark gating, or exception lists.

A rule may have high confidence but high false-positive risk: the norm is stable, yet the current detector cannot safely recognize context. That is common in Russian punctuation.
