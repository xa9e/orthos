# Algorithmic Russian word-formation model

## Why this layer exists

The checker must not become a graveyard of isolated regular expressions. Many
Russian orthographic and grammatical rules depend on word structure: prefix,
root, derivational suffix, inflectional suffix, ending, postfix, interfix and
productive alternations. A spelling detector should be able to ask: "is this a
productive prefix boundary?" instead of guessing from the first three letters.

The first implementation is deliberately small, but it establishes the right
shape:

- typed morpheme kinds;
- project-authored seed inventory;
- declarative macro-based inventory definitions;
- candidate morphemic parses with confidence and score;
- conservative helpers for orthographic rules;
- a detector using the model for final `з/с` in prefixes.

## Linguistic base assumptions

The model follows a conservative school/academic decomposition vocabulary:
морфема, приставка, корень, суффикс, окончание, постфикс, интерфикс. It treats
word formation as structured derivation, not only as surface spelling. This is
compatible with the project rule corpus: a rule can declare `word_formation` and
then require concrete detector capabilities.

The seed inventory is not a copied dictionary. It is a small project-authored
starter set designed to prove architecture and support tests. Real coverage
must come later from licensed dictionaries, curated morphemic resources, and
reviewed project additions.

## Current algorithm

`RussianDerivationModel::analyze_word` normalizes a word, then searches:

1. known endings, longest first;
2. possible prefix chains, currently up to two prefixes;
3. known root at the start of the remaining base;
4. suffix chains that consume the rest of the base;
5. fallback unknown-root parse if nothing was recognized.

The output is a ranked list of `WordFormationParse` values. Each parse contains
ordered `MorphemeSegment`s, a score and a confidence level. Unknown-root parses
exist so the API is total, but detectors should not use them for aggressive
corrections.

## Why macros are used

The inventory is defined through a small macro DSL:

```rust
morpheme_inventory!(
    MorphemeKind::Prefix,
    MorphemeProductivity::Productive,
    {
        "раз" => ["z_s_pair", "separation_or_intensity"],
        "рас" => ["z_s_pair", "separation_or_intensity"]
    }
)
```

This is intentional. A language checker will accumulate hundreds or thousands
of small linguistic facts. Hand-written constructors would add noise; a macro
keeps the facts declarative while preserving static typing and zero runtime
parsing for the built-in seed layer.

The likely next step is code generation: keep reviewed TSV/YAML inventories in
`data/morphemes`, validate them, and generate Rust static arrays during build or
release packaging. For now, static macro data is safer and simpler.

## First consuming rule: prefix-final з/с

The detector for `ru.orthography.prefix_final_s_z` checks productive prefixes
such as `без-/бес-`, `из-/ис-`, `раз-/рас-`, `воз-/вос-`. It does not blindly
rewrite any word starting with `рас` or `раз`. It only suggests a correction when
the remaining base looks known to the seed morpheme inventory.

Examples:

- `разсказать` → `рассказать`;
- `расбить` → `разбить`;
- `безсмертный` → `бессмертный`;
- `расист` is ignored because the seed model does not treat `ист` as a known
  derivational base in this context.

That last example is the point: conservative abstention beats confident garbage.

## Known limitations

- Root inventory is tiny.
- Alternations such as `пис/пиш`, `лаг/лож`, `бер/бир` are not modeled yet.
- The model does not yet use part of speech from the morph lexicon.
- It does not model stress, so rules like `о/ё/е` after sibilants remain blocked.
- It does not distinguish synchronically productive segmentation from historical
  etymology.
- It cannot replace a real morphologic dictionary such as an OpenCorpora-derived
  lexicon.

## Roadmap

1. Add explicit allomorph groups: root alternants, беглые гласные, consonant
   alternations.
2. Add derivational templates: `prefix* + root + suffix* + ending + postfix?` is
   only the first template family.
3. Connect parses with `MorphAnalysis` so derivation can be filtered by POS and
   grammatical features.
4. Add agreement abstractions over typed feature unification, not ad-hoc pair
   checks.
5. Add import/validation tooling for `data/morphemes` and later generated Rust
   arrays.
6. Benchmark detectors by rule family: word-formation rules need separate false
   positive accounting.
