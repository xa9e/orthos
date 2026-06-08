# Morphological dictionary integration

`orthos` morphology must stay deterministic, typed, and conservative. Large third-party dictionaries are integration inputs, not code dependencies to vendor blindly. The production path is:

1. import an external dictionary into a neutral record stream;
2. normalize POS/grammemes into `src/morph.rs` typed enums;
3. preserve unsupported tags for audit;
4. emit a compact project-local cache with metadata/provenance;
5. load the cache behind `MorphAnalyzer` without changing detectors.

The current committed lexicon is intentionally tiny. It is a contract fixture, not a real Russian dictionary.

## Core interfaces

Morphology-facing code should depend on these Rust concepts:

- `MorphAnalyzer`:
  - `analyze(&self, token) -> Vec<MorphAnalysis>`;
  - `metadata()` for dictionary/source metadata;
  - `capabilities()` for ambiguity, unknown-token, provenance, id, and stress support.
- `MorphAnalysis`:
  - surface form, lemma, POS, typed features;
  - optional `lemma_id`;
  - optional `paradigm_id`;
  - optional `source_id`;
  - `StressInfo` with `StressAvailability`.
- `MorphFeatures`:
  - raw tags as seen in imported data;
  - normalized tag strings;
  - typed features used by detectors;
  - unrecognized tags for importer diagnostics.
- `DictionaryImporter`:
  - strict importer boundary returning `Result<MorphLexicon, DictionaryImportError>`;
  - backward-compatible `import_tsv` shim for the old demo façade.
- `DictionaryMetadata`:
  - source id, name, version, format, license status, attribution, entry count, stress flag.

Detectors must not inspect external dictionary files directly. If a detector needs linguistic semantics, add an explicit typed field or helper instead of matching raw OpenCorpora/pymorphy/AOT tags inline.

## Implemented import boundaries

Implemented now:

- `ProjectTsvDictionaryImporter`:
  - extended project TSV;
  - strict malformed-line errors;
  - provenance, lemma ids, paradigm ids, stress column.
- `OpenCorporaXmlDictionaryImporter`:
  - tiny OpenCorpora-like fixture XML only;
  - supports `<lemma id="...">`, `<l t="...">`, `<f t="...">`, inline `<g v="..."/>` tags;
  - maps POS from grammemes and preserves unsupported grammemes.
- `OpenCorporaCsvDictionaryImporter`:
  - tiny CSV slice: `form,lemma,pos,grammemes,lemma_id,paradigm_id,source_id,stress`.
- `PymorphyExportDictionaryImporter`:
  - tiny tab-separated export slice: `word,normal_form,tag,lemma_id,paradigm_id,source_id,stress`;
  - comma/space/pipe-separated tags are normalized into the same `MorphFeatures` model.
- `StressTsvImporter`:
  - stress-record boundary separate from morphology records;
  - attach by surface form plus optional `lemma_id`/`paradigm_id`.

Not implemented now:

- full OpenCorpora XML streaming parser;
- direct compiled pymorphy dictionary parser;
- AOT parser;
- large generated cache format;
- dictionary download or network workflow.

## Conversion script

`scripts/import_morph/convert_fixture.py` converts tiny fixture snapshots to project TSV.

Supported inputs:

- `--source-format project-tsv`;
- `--source-format opencorpora-xml`;
- `--source-format opencorpora-csv`;
- `--source-format pymorphy-tsv`.

Properties:

- deterministic sorted output;
- explicit `--encoding`, defaulting to `utf-8-sig`;
- no network access;
- no raw dictionary vendoring;
- stdout reports only the record count and output path;
- generated output is UTF-8 project TSV.

Example:

```bash
python3 scripts/import_morph/convert_fixture.py \
  --input testdata/fixtures/morph/opencorpora.xml \
  --output /tmp/morph.tsv \
  --source-format opencorpora-xml \
  --source-id fixture.opencorpora.xml
```

License caveat: this script does not make third-party data redistributable. Before committing generated data, verify the dictionary-data license separately from analyzer-code licenses and record attribution in `DictionaryMetadata`/docs.

## OpenCorpora dictionaries

OpenCorpora remains the most plausible first-class production source because it exports structured XML dictionaries and already underpins many Russian morphology tools. The current importer is only a fixture parser; a production importer should be a streaming parser, not a `String`/line hack.

Recommended production boundary:

1. parse XML lexemes and grammemes;
2. assign a stable `source_id`, source version/revision/date, and attribution;
3. map OpenCorpora POS tags:
   - `NOUN` -> `PartOfSpeech::Noun`;
   - `ADJF`, `ADJS` -> `PartOfSpeech::Adjective` plus `AdjectiveForm::Full/Short`;
   - `COMP` -> `PartOfSpeech::Comparative`;
   - `VERB`, `INFN` -> `PartOfSpeech::Verb` plus `VerbForm` where known;
   - `PRTF`, `PRTS` -> `PartOfSpeech::Participle` plus form/voice where known;
   - `GRND` -> `PartOfSpeech::Gerund`;
   - `NUMR`, `ADVB`, `NPRO`, `PRED`, `PREP`, `CONJ`, `PRCL`, `INTJ` -> corresponding typed POS variants.
4. map grammemes:
   - cases: `nomn`, `gent`, `datv`, `accs`, `ablt`, `loct`, plus `gen2`, `loc2`, `voct` where present;
   - number: `sing`, `plur`;
   - gender: `masc`, `femn`, `neut`, `ms-f`;
   - animacy: `anim`, `inan`;
   - aspect: `perf`, `impf`;
   - tense/person: `past`, `pres`, `futr`, `1per`, `2per`, `3per`;
   - voice/form/mood and lexical grammemes as the typed model grows.
5. preserve every unsupported grammeme in `unrecognized_tags` and publish counts in import logs.

Do not flatten ambiguity away. If OpenCorpora provides several analyses for a surface token, return all analyses unless a separate disambiguation layer is explicitly added later. Grammar diagnostics must remain conservative: unknown or ambiguous analysis suppresses unsafe warnings.

## pymorphy2 / pymorphy3 dictionary format

`pymorphy2` and `pymorphy3` use compiled dictionary packages derived from OpenCorpora-like sources. Their internal representation is attractive for runtime efficiency but is not the canonical source for this Rust project yet.

Useful architecture facts:

- compiled dictionaries store paradigms, suffix tables, grammeme tables, and DAWG-like word lookup structures;
- paradigm ids are useful: `MorphAnalysis.paradigm_id` exists so a future importer can retain a stable compact reference;
- lemma ids are useful for deduplicating lexeme-level metadata and for later inflection/generation;
- dictionary packages may be installable separately from analyzer code, so importer metadata must distinguish analyzer package license from dictionary data license.

Recommended integration path:

- first production importer: OpenCorpora XML -> project TSV/JSON cache;
- later importer: pymorphy compiled dictionary -> neutral records only if the format is stable enough and licensing is checked;
- never require Python at runtime for the Rust checker;
- allow offline conversion tools in `scripts/` if they produce a compact cache committed only when redistribution is legally clear.

The implemented `PymorphyExportDictionaryImporter` is intentionally not a compiled-dictionary reader. It only locks down a tiny exported fixture format.

## AOT-style dictionaries

AOT-style morphology is historically important and still appears in downstream projects. Treat it as a possible import source, not as a default dependency.

Risks:

- original download locations and mirrors may be stale;
- file formats can be old and sparsely documented;
- downstream projects may relicense code while dictionary data has separate conditions;
- dictionary freshness is questionable compared with OpenCorpora-derived sources.

Required before integration:

1. identify exact source package and version;
2. identify dictionary-data license separately from wrapper/analyzer code license;
3. verify redistribution of derived wordform tables;
4. write a tiny fixture parser for the specific format slice being supported;
5. record every assumption in `DictionaryMetadata` and importer docs.

Until this is done, AOT support should remain a stub/import-boundary topic. Do not vendor AOT dictionaries.

## Small curated project dictionaries

Small dictionaries are allowed and should remain boring:

- committed under `data/lexicon/` only when hand-curated or fixture-sized;
- human-readable TSV for review;
- no generated megafiles;
- source id required;
- optional lemma/paradigm ids allowed;
- optional stress column allowed.

Current extended TSV columns:

1. `form` — surface form, normalized to lowercase for lookup;
2. `lemma`;
3. `pos`;
4. `features` — pipe-separated tags;
5. optional `lemma_id`;
6. optional `paradigm_id`;
7. optional `source_id`;
8. optional stress marker/stressed form.

The old four-column TSV remains valid. The extra columns make fixture-based importer tests useful without introducing a real dictionary.

## Future stress dictionaries

Stress must be modeled as a capability, not assumed globally.

Rules:

- `StressAvailability::Unknown`: analyzer did not say whether stress is available;
- `StressAvailability::Unavailable`: analyzer/source has no stress data;
- `StressAvailability::Available`: analysis has stress support, optionally with a stressed form;
- detectors that require stress must check capability and per-analysis stress state;
- stress dictionaries need independent metadata because their sources and licenses may differ from morphology dictionaries.

The current `StressTsvImporter` attaches stress by normalized surface form plus optional `lemma_id` and `paradigm_id`. That is deliberately conservative: ids make homograph enrichment safer than a raw surface-only join.

## Licensing policy

Licensing is deliberately represented as data. Do not pretend every resource is vendorable just because code around it is open source.

For each dictionary source record:

- source URL/name;
- version/revision/date;
- dictionary data license;
- analyzer code license if an analyzer is used during conversion;
- redistribution status for derived caches;
- attribution text;
- whether local generation is required instead of committing the result.

`LicenseStatus::Unknown` must block vendoring of generated large caches. `LicenseStatus::LocalGenerationOnly` is appropriate for fixture importers and local experiments where redistribution has not been reviewed.

## Importer test policy

Importer tests must use tiny fixtures only:

- one to five lexemes/forms are enough;
- cover ids/provenance/stress columns;
- cover unsupported tag preservation;
- cover strict malformed-line errors;
- cover conservative ambiguity behavior.

Large dictionary tests belong in external benchmark jobs, not in this repository.
