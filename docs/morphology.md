# Morphology architecture

`orthos` treats morphology as an explicit typed capability. The engine is deterministic and conservative: a missing, incomplete, or ambiguous analysis suppresses morphology-backed diagnostics instead of inventing a warning.

See also: `docs/morph-dictionaries.md` for dictionary-source integration, fixture formats, conversion scripts, and licensing policy.

## Public boundary

Rule code should depend on:

- `MorphAnalyzer` — analyzer trait:
  - `analyze(&self, token) -> Vec<MorphAnalysis>`;
  - `metadata()` for dictionary provenance;
  - `capabilities()` for analyzer behavior.
- `MorphAnalysis` — one possible analysis of one surface token:
  - form;
  - lemma;
  - typed POS;
  - typed features;
  - optional `lemma_id`;
  - optional `paradigm_id`;
  - optional `source_id`;
  - `StressInfo`.
- typed feature enums such as `PartOfSpeech`, `Case`, `Number`, `Gender`, `Animacy`, `Aspect`, `Tense`, `Person`, `AdjectiveForm`, `Degree`, `VerbForm`, `Mood`, `Voice`;
- compatibility helpers such as:
  - `case_compatibility`;
  - `number_compatibility`;
  - `gender_compatibility`;
  - `has_compatible_case_number_gender`;
  - `can_agree_as_adj_noun`;
  - `confidently_reject_adj_noun_agreement`;
  - `subject_predicate_compatibility`;
  - `numeral_government_compatibility`;
  - `animacy_aware_accusative_compatibility`.

The current concrete analyzer is `MorphLexicon`, an in-memory lexicon. It is still tiny and exists to make detector contracts testable. It is not a production Russian morphology dictionary.

## ё/е lookup folding

Russian text routinely drops `ё`, so `MorphLexicon` buckets entries under a
ё-folded lowercase key (`колеса` and `колёса` share one bucket).

Lookup policy:

- a token written without `ё` returns the whole bucket — both spellings are a
  genuine ambiguity and conservative rules must treat them as such;
- a token with an explicit `ё` is trusted as a disambiguator: if the bucket
  contains exact-spelling analyses, only those are returned;
- a `ё` token with no exact entry still falls back to the folded bucket.

This removes a classic false-positive class (е-spelled plural subjects such as
«Колеса стучали» being analyzed only as genitive singular) without losing the
precision that explicit `ё` provides. Binary lexicon caches are keyed the same
way; the cache magic was bumped (`RLM2`/`RLI3`) so stale caches regenerate.

## Analyzer capabilities

`AnalyzerCapabilities` documents what the analyzer can safely provide:

- lexical lookup;
- lemma availability;
- lemma ids;
- paradigm ids;
- provenance/source ids;
- stress availability;
- ambiguity policy;
- unknown-token behavior.

The built-in TSV lexicon uses:

- `AmbiguityPolicy::ConservativeDiagnostics`;
- `UnknownTokenBehavior::NoAnalysis`;
- `StressAvailability::Unavailable` unless an imported fixture supplies stress data.

Detectors must check capabilities before relying on optional data such as stress. A detector that cannot prove compatibility must return no diagnostic. That rule is boring, and it prevents a lot of garbage warnings.

## Implemented lexicon/importer layer

Implemented in this iteration:

- `ProjectTsvDictionaryImporter` for project TSV fixtures and the built-in demo lexicon;
- `OpenCorporaXmlDictionaryImporter` for tiny OpenCorpora-like XML fixtures;
- `OpenCorporaCsvDictionaryImporter` for tiny OpenCorpora-like CSV fixtures;
- `PymorphyExportDictionaryImporter` for tiny tab-separated pymorphy-style exported fixtures;
- `StressTsvImporter` and `MorphLexicon::attach_stress_records` for future stress/provenance enrichment boundaries;
- `scripts/import_morph/convert_fixture.py` for deterministic offline conversion of tiny fixture snapshots into project TSV.

Not implemented:

- full OpenCorpora XML parsing at production scale;
- direct reading of compiled pymorphy DAWG/paradigm packages;
- AOT dictionary import;
- real stress dictionary integration;
- disambiguation.

The fixture importers intentionally parse a narrow, documented slice. They are boundary tests, not a free pass to dump gigabytes of dictionary data into git.

## Project TSV format

Path for the demo lexicon: `data/lexicon/demo_morph.tsv`.

Required columns:

1. `form` — lowercased by the loader;
2. `lemma`;
3. `pos` — normalized into `PartOfSpeech`;
4. `features` — pipe-separated tags, for example `gender=masc|number=sing|case=nom`.

Optional columns:

5. `lemma_id`;
6. `paradigm_id`;
7. `source_id`;
8. stress marker or stressed form.

The parser accepts `key=value` tags and common OpenCorpora-like aliases (`NOUN`, `ADJF`, `ADJS`, `nomn`, `gent`, `sing`, `plur`, `masc`, `femn`, `neut`, `anim`, `inan`, etc.). Raw tags are preserved in `MorphFeatures::raw_tags`; normalized spellings go to `normalized_tags`; unsupported tags go to `unrecognized_tags` for importer diagnostics, not detector logic.

## Fixture formats

Tiny fixtures live under `testdata/fixtures/morph/`.

Implemented fixture slices:

- `project.tsv` — project TSV with ids/provenance/stress;
- `opencorpora.xml` — `<lemma>`, `<l t="...">`, `<f t="...">`, and inline `<g v="..."/>` tags;
- `opencorpora.csv` — `form,lemma,pos,grammemes,lemma_id,paradigm_id,source_id,stress`;
- `pymorphy.tsv` — `word, normal_form, tag, lemma_id, paradigm_id, source_id, stress`, tab-separated;
- `stress.tsv` — either two columns (`form`, `stress`) or five columns (`form`, `lemma_id`, `paradigm_id`, `source_id`, `stress`).

All fixture importers preserve ambiguity. Multiple analyses for one surface form remain multiple analyses.

## Agreement model

Adjective–noun agreement is intentionally two-stage:

- pair-level compatibility checks whether two concrete analyses conflict in case/number/gender;
- detector-level rejection requires a complete, unambiguous adjective/participle analysis on the left and a complete, unambiguous noun analysis on the right.

This means:

- compatible analysis ⇒ no diagnostic;
- unknown token ⇒ no diagnostic;
- missing case/number/gender where needed ⇒ no diagnostic;
- multiple competing signatures ⇒ no diagnostic;
- one complete adjective signature + one complete noun signature + no compatibility ⇒ diagnostic.

Plural agreement ignores gender. Singular agreement compares gender, with `Gender::Common` treated as compatible with masculine/feminine/neuter.

Low-level helpers return `MorphCompatibility` so callers can distinguish:

- `Compatible`;
- `Incompatible`;
- `Unknown`.

For legacy detector behavior, `has_compatible_case_number_gender` still treats `Unknown` as permissive and only rejects definite conflicts.

## Subject–predicate placeholder model

`subject_predicate_compatibility` is implemented as a conservative primitive, not a production syntax detector.

Current behavior:

- supports noun/pronoun/numeral subjects and verb/participle/predicative predicates;
- compares number when both sides expose number;
- treats noun subjects as third person for finite-person comparison;
- checks gender only for singular past-tense predicate forms;
- returns `Unknown` instead of guessing when person, number, tense, or gender evidence is missing.

This is enough for future detector experiments to share a typed contract. It is not enough to diagnose all Russian subject–predicate agreement errors.

## Numeral government model

`numeral_government_class` recognizes a tiny deterministic class layer:

- `One`;
- `Paucal` for `2–4`-like government;
- `Many` for `5+`-like government;
- `Collective`;
- `Ordinal`;
- `Unknown`.

The class may come from `num_class=...`, known lemmas in tiny fixtures, or typed grammemes such as `Ordinal`/`Collective`.

`numeral_government_compatibility` currently models only safe fixture-level cases:

- ordinal/`one` behave like agreement checks;
- paucal nominative/accusative numerals expect genitive singular nouns;
- many/collective nominative/accusative numerals expect genitive plural nouns;
- oblique numeral cases and missing class evidence return `Unknown`.

The older `numeral_noun_compatibility` is retained as a compatibility shim.

## Animacy-aware accusative helper

`animacy_aware_accusative_compatibility` answers whether one concrete analysis can satisfy an accusative syntactic slot under a conservative shadow-case model:

- explicit accusative ⇒ `Compatible`;
- animate masculine singular or animate plural genitive shadow ⇒ `Compatible`;
- inanimate nominative shadow ⇒ `Compatible`;
- unsupported known cases ⇒ `Incompatible`;
- feminine/neuter animate singular genitive and incomplete evidence ⇒ `Unknown`.

This helper is intentionally narrow. It prevents detectors from treating every genitive-looking form as accusative-compatible.

## Government lookup

`PrepositionGovernment` stores allowed cases for one preposition. `PrepositionGovernmentRegistry` returns:

- `Compatible` when a known entry allows the case;
- `Incompatible` when the preposition is known but none of its entries allow the case;
- `Unknown` when the preposition is not in the registry.

No production preposition-government diagnostics are added in this iteration.

## Swapping analyzers

`Checker::with_morph_analyzer(corpus, analyzer)` accepts any analyzer implementing `MorphAnalyzer`. That is the intended extension point for:

- the current TSV `MorphLexicon`;
- an OpenCorpora-derived in-memory lexicon;
- a compact cache generated from pymorphy dictionaries;
- an adapter over an external analyzer process;
- a test double that returns controlled ambiguity.

Detectors must not read analyzer storage directly. If a detector needs new linguistic semantics, add typed fields or explicit helper functions rather than checking untyped raw tags.
