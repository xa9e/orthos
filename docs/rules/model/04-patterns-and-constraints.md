## Pattern model

`pattern` describes the match target in linguistic terms. It does not replace `detector`; it explains what a future detector must identify.

Allowed `pattern.kind` values:

- `surface` — raw character/token adjacency;
- `regex` — regular-expression-level surface pattern;
- `token_sequence` — sequence of normalized tokens;
- `morphological` — token plus POS/features/lemma constraints;
- `syntactic` — constituent or clause configuration;
- `dependency` — typed relation between head/dependent tokens;
- `punctuation_context` — punctuation candidate with left/right linguistic context;
- `word_formation` — morpheme/stem/derivation pattern;
- `stress` — stress-position pattern;
- `lexical_set` — curated list, collocation, idiom, or valency frame.

Optional fields:

- `description` — short human-readable semantics;
- `value` — compact pattern value where useful;
- `captures` — named roles such as `head`, `dependent`, `subject`, `predicate`, `subordinator`, `suffix`.

## Constraint model

`constraints` encode conditions that make a candidate a real error. `exceptions` encode conditions that suppress a candidate.

Allowed `kind` values model reusable linguistic concepts:

### Agreement

`agreement` covers compatibility of case, number, gender, person, animacy, tense, or other relevant features between linked units.

Use cases:

- adjective + noun;
- subject + predicate;
- participle + head noun;
- numeral + noun groups.

Expected capabilities: usually `morphology`; often `syntax` when the linked units are not adjacent.

### Government

`government` covers a head requiring a dependent form, often case or preposition frame.

Use cases:

- preposition + case;
- verb valency;
- adjective/noun government;
- fixed constructions like `согласно` + dative.

Expected capabilities: `morphology`, `syntax`, often `lexicon`.

### Coordination

`coordination` covers homogeneous members and coordinated subjects/predicates/clauses.

Use cases:

- agreement target for coordinated nouns;
- comma rules for homogeneous members;
- repeated conjunctions;
- common element shared by coordinated clauses.

Expected capabilities: `syntax` and sometimes `morphology`.

### Clause boundary

`clause_boundary` marks the start/end of finite or non-finite clauses.

Use cases:

- comma before subordinate clause;
- comma after initial subordinate clause;
- participial and adverbial-participial phrases;
- asyndetic sentence punctuation.

Expected capabilities: `sentence_boundaries`, `tokenization`, and eventually `syntax`.

### Parenthetical expression

`parenthetical_expression` covers inserted material: parenthesized phrases, introductory words, and detached comments.

Use cases:

- suppressing naive comma-before-subordinator matches inside parentheses;
- detecting missing commas around introductory words;
- avoiding punctuation changes inside metadata-like fragments.

Expected capabilities: `syntax` or a conservative span recognizer.

### Direct speech

`direct_speech` covers quoted utterances and colon/dash/quote punctuation patterns.

Use cases:

- suppressing subordinate-marker heuristics inside quoted questions;
- direct speech punctuation;
- boundary detection around author words.

Expected capabilities: quote pairing, punctuation context, and syntax for robust handling.

### Derivation

`derivation` covers source stem, derivational class, morpheme boundary, and productive word-formation pattern.

Use cases:

- prefix-final `з/с`;
- `пре-`/`при-`;
- suffix choice;
- `н/нн` in deverbal adjectives and participles;
- compound adjectives and adverbs.

Expected capabilities: `word_formation`, `morphology`, and `lexicon`.

### Spelling pattern

`spelling_pattern` covers a normative orthographic alternation or grapheme choice.

Use cases:

- `жи/ши`, `ча/ща`, `чу/щу`;
- `-тся/-ться`;
- fused/separate/hyphenated spelling;
- `не`/`ни` with different POS.

Expected capabilities range from `regex` to `morphology` and `word_formation`.

### Stress requirement

`stress_requirement` covers rules where stressed/unstressed position changes spelling.

Use cases:

- `о/е/ё` after sibilants and `ц`;
- some suffix vowel choices;
- dictionary-backed proper stress variants.

Expected capabilities: `stress`; usually also `morphology`, `word_formation`, and `lexicon`.

### Idiom/fixed expression exception

`idiom_fixed_expression_exception` suppresses otherwise valid rules inside stable expressions.

Use cases:

- punctuation suppressed inside fixed expressions;
- idiomatic government;
- lexicalized forms that violate a productive spelling pattern.

Expected capabilities: `lexicon`; sometimes `semantics` or phrase matching.

Additional concept kinds exist for implementation convenience: `morphology`, `syntax`, `lexical`, `style_register`, `named_entity`, `token_context`, and `sentence_boundary`.
