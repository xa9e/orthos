# Sources

This project stores formalized summaries and executable examples, not copied textbook chapters.

## Primary normative and corpus sources

- Грамота.ру: В. В. Лопатин (ред.), *Правила русской орфографии и пунктуации. Полный академический справочник*.
- Грамота.ру: *Правила русской орфографии и пунктуации (1956)*.
- Грамота.ру: *Справочник по пунктуации*.
- НКРЯ: morphological and syntactic annotation standards.
- OpenCorpora dictionary as a candidate morphology lexicon source where licensing and packaging are appropriate.

- Грамота.ру: краткий теоретический курс по морфемике и словообразованию; used for terminology and decomposition concepts, not copied prose.
- *Русская грамматика* (1980), том 1; long-term academic reference for morphology and word formation.
- *Русская грамматика* (1980), том 2; long-term academic reference for syntactic links, including agreement, government, and adjunction.
- RusGram case overview; used as a pointer for the distinction between governed, construction-conditioned, and freely attached case uses.
- Universal Dependencies Russian SynTagRus; human-corrected morphology/syntax corpus pointer for future agreement and dependency benchmarks.

## Donor and diagnostic sources

- `ru-guard` donor material: surface detector ideas, small examples, and demo morphology concepts.
- `ruslint` donor material: long-term roadmap categories for morphology-heavy rules.
- `rulang` donor material: phrase-map seed rules and registry-oriented corpus architecture ideas.
- LORuGEC: inspected only for rule-label and taxonomy signals. The raw dataset is not redistributed here.

## Web reference pointers used in metadata

- `orthographia.ru` and `old-rozental.ru` are used as source pointers where labels or rule families are easier to identify there.
- These references are navigational. The corpus paraphrases the rule phenomenon and stores only short examples.

## Licensing and redistribution discipline

Many textbooks, school manuals, dictionaries, and corpora are not automatically redistributable. Use them for human study and citation, not for bulk copying.

For open-source distribution:

- store project-authored rule formulations;
- keep examples minimal and original;
- link to sources instead of copying long rule text;
- do not vendor external corpora unless the license clearly allows it and the project intentionally accepts the size/maintenance cost;
- record implementation blockers in `requires` instead of pretending a regex can solve morphology, syntax, stress, or word formation.
