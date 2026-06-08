# Taxonomy of Russian proofreading rules

The corpus is organized by linguistic subsystem, not by source order. A source may motivate several rules, but the YAML rule id must identify the checked phenomenon and implementation scope.

For authoring mechanics, schema fields, validation policy, and source discipline, see `docs/rule-authoring-guide.md`. This taxonomy answers “where does the phenomenon belong?”; the authoring guide answers “how must the YAML rule be written?”.

## 0. Technical typography and segmentation

Surface-level checks. These may be deterministic and mostly regex/tokenization-backed.

- whitespace normalization;
- whitespace before and after punctuation;
- paired brackets and quotes;
- dash/hyphen distinction;
- dash spacing;
- repeated expressive punctuation;
- number-unit spacing;
- mixed-script tokens and confusable glyphs.

## 1. Basic orthography

Mostly normative spelling checks. Some are surface-safe; most become reliable only with morphology, lexicon, or word formation.

- sentence-initial capital letter;
- `не`/`ни` with verbs, nouns, adjectives, participles;
- fused vs separate function words: `чтобы` / `что бы`, `также` / `так же`, `зато` / `за то`, `причём` / `при чём`, `оттого` / `от того`;
- hyphenated particles: `-то`, `-либо`, `-нибудь`, `-таки`;
- separate particles: `же`, `бы`, `ли`;
- `-тся` / `-ться`;
- separating `ъ` and `ь`;
- `жи/ши`, `ча/ща`, `чу/щу` as low-risk surface checks.

## 2. Word formation and school-level orthography

Rules here must declare `requires: [word_formation]` whenever morpheme boundaries, productive derivation, or source stems are needed. Regex-only implementation is usually a trap.

- prefix-final `з/с`;
- `ы/и` after prefixes;
- `пре-` / `при-`;
- `пол-` compounds;
- written equivalents of complex words with digits or letters;
- `о/е/ё` after sibilants and `ц`;
- `н/нн` in adjectives, participles, and deverbal adjectives;
- participial suffix vowels;
- suffixes `-ек-/-ик-`, `-иц-/-ец-`, `-еньк-/-оньк-`, `-инск-/-енск-`;
- compound adjectives;
- fused, separate, and hyphenated adverbs.

## 3. Punctuation

Punctuation is split into surface punctuation and syntax-heavy punctuation. Syntax-heavy rules must normally declare `syntax`; participial/adverbial-participial rules also need `morphology`.

- simple punctuation adjacency;
- sentence terminal punctuation;
- comma before frequent subordinators as a heuristic;
- introductory words and phrases;
- homogeneous members and repeated conjunctions;
- homogeneous subordinate clauses;
- common element in a compound sentence;
- junction of two conjunctions;
- `как`-constructions: comparison, introductory use, role/application, predicative use;
- participial clauses;
- adverbial-participial clauses;
- detached definitions near personal pronouns;
- nondecomposable/idiomatic expressions where comma is suppressed;
- dash between subject and predicate;
- dash in asyndetic complex sentences;
- dash with apposition.

## 4. Grammar

Grammar rules are parser- and morphology-heavy by default. Most should remain `research` until the engine has a real morphological analyzer, disambiguation, and typed dependencies.

- adjective-noun agreement;
- subject-predicate agreement;
- participle agreement with the head noun;
- government by verbs, prepositions, nouns, and adjectives;
- quantitative numeral declension;
- `полтора` / `полторы` / `полтораста` agreement and declension;
- numeral-noun agreement;
- pronoun reference;
- coordination;
- tense/aspect compatibility.

## 5. Style and semantics

These checks are rarely safe as hard errors. The default severity should be `info` or `warning`, backed by curated lists and benchmarked false-positive rates.

- pleonasms;
- lexical collocation;
- paronyms;
- bureaucratic clichés;
- register mismatch;
- ambiguity and garden-path constructions.

## LORuGEC label mapping

`LORuGEC.xlsx` was inspected for taxonomy only. The `.m2` files do not expose useful error types: all edits are labeled `None`, so no dataset examples or raw rows are vendored.

Useful LORuGEC sections map as follows:

- `Spelling` -> orthography plus word-formation rules: `не`, fused/separate words, prefixes, suffixes, `н/нн`, `пол-`, adverbs, complex adjectives.
- `Punctuation` -> syntax-heavy punctuation rules: homogeneous members, conjunction junctions, `как`, introductory constructions, detached definitions, participial/deeprichastie clauses, dashes.
- `Grammar` -> agreement, government, numeral declension, `полтора/полторы/полтораста`.
- `Semantics` -> style/semantics: lexical collocation and pleonasms.

The LORuGEC mapping is diagnostic, not an implementation shortcut: labels identify hard phenomena, not ready detectors.


## Normalized rule-family layer

The taxonomy above remains source-facing and pedagogical. The formal schema in `docs/rule-model.md` adds a second layer: `rule_family` describes implementation architecture.

Use the split this way:

- surface typography stays `domain: typography` and usually `rule_family: typography`;
- visible spelling errors stay `domain: orthography`, but may use `rule_family: morphology_dependent`, `word_formation`, or `stress_dependent`;
- punctuation rules that need clause structure should use `rule_family: syntax_dependent`, not plain `punctuation`;
- grammar rules should explicitly mark `morphology_dependent` or `syntax_dependent` when the detector cannot be adjacency-based;
- style rules with curated lexical sets should use `rule_family: lexical_style` and conservative severity.

The first annotated corpus seeds are deliberately small: subordinator punctuation, subject-predicate agreement, and stress-dependent spelling after sibilants. They are examples of the model, not an attempt to reclassify all 107 rules in one pass.

## Planned/research family backlog

Source-backed future families, refined from existing Russian-rule taxonomy and LORuGEC-style labels:

- `morphology_dependent`: `-тся/-ться`, agreement, participles, numeral-noun forms, `полтора/полторы/полтораста`;
- `syntax_dependent`: subordinate clauses, homogeneous members, detached clauses, direct speech, apposition, dash in complex sentences;
- `word_formation`: prefixes, suffixes, `н/нн`, compound words, hyphenated adverbs/adjectives;
- `stress_dependent`: `о/е/ё` after sibilants and `ц`, suffix-vowel families that need stress;
- `lexical_style`: paronyms, pleonasms, collocation, bureaucratic clichés, idiom exceptions.

This layer should prevent the classic trap: labeling a hard syntactic or morphological phenomenon as a simple regex rule just because the visible typo is short.
