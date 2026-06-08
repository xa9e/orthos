# Evaluation datasets

The evaluation layer normalizes external Russian GEC/spellcheck datasets into one local JSONL schema and keeps full third-party corpora out of the repository unless redistribution is explicitly safe.

## Dataset registry

Machine-readable dataset metadata lives in:

```text
data/dataset-registry.json
```

The registry records, per dataset:

- dataset name;
- expected local archive name;
- source URL/name;
- license and usage status;
- importer command template;
- normalized output path;
- recommended benchmark mode;
- observed external counts from the prior attached archives;
- fixture counts and known limitations.

The registry is intentionally JSON, not YAML, so orchestration scripts can read it with Python stdlib only. It is **not** a downloader and it must not be treated as a license grant.

Run a registered local archive end to end:

```bash
python scripts/eval/run_benchmark.py \
  --dataset lorugec \
  --archive /datasets/LORuGEC-main.zip \
  --checker-bin target/debug/orthos \
  --rules rules \
  --limit 100 \
  --report-dir reports/eval
```

If the three known archives are in `~/Downloads`, they can be used directly:

```bash
for item in \
  "lorugec:$HOME/Downloads/LORuGEC-main.zip" \
  "rlc-annotated:$HOME/Downloads/rlc-annotated-main.zip" \
  "ruspellgold:$HOME/Downloads/RuSpellGold.zip"
do
  dataset=${item%%:*}
  archive=${item#*:}
  python scripts/eval/run_benchmark.py --dataset "$dataset" --archive "$archive" \
    --checker-bin target/debug/orthos --rules rules --profile strict --limit 100
done
```

Dry-run the commands without importing or benchmarking:

```bash
python scripts/eval/run_benchmark.py \
  --dataset rlc-annotated \
  --archive /datasets/rlc-annotated-main.zip \
  --dry-run
```

## Normalized JSONL schema

Every importer writes one JSON object per line:

```json
{
  "id": "dataset-local-id",
  "dataset": "rlc-annotated|lorugec|ruspellgold|...",
  "source_file": "relative/original/file/path",
  "input": "incorrect or source sentence",
  "correction": "single corrected sentence if available",
  "targets": ["one or more corrected references if available"],
  "edits": [],
  "labels": [],
  "metadata": {}
}
```

Rules:

- `input` is required.
- `correction` may be empty when only labels/spans exist.
- `targets` contains available corrected references.
- `labels` preserves source labels/rules where available.
- `metadata` stores dataset-specific fields. Do not add dataset-specific top-level keys.

## Repository policy

Do **not** vendor full third-party datasets into this repository by default.

Preferred workflow:

```text
external archive -> scripts/import/* -> local data/eval/*.jsonl -> scripts/eval/benchmark_jsonl.py
```

Generated evaluation outputs are ignored by `.gitignore`:

```text
data/eval/
reports/eval/
```

Tiny fixtures used by importer tests live under:

```text
testdata/fixtures/import/
```

The committed fixture suite also builds temporary ZIP archives in tests, so archive-aware import paths are checked without committing raw full corpora.

## Snapshot note

This submitted project snapshot contains only tiny committed fixtures under `testdata/fixtures/import/`. The full third-party archives named below were **not** attached in this run. Record counts in this document and `data/dataset-registry.json` are retained from prior archive inspection recorded in the merged project. When fresh archives are supplied, rerun importers and update observed counts instead of trusting stale metadata.

## Concrete datasets

### RLC-GEC / `rlc-annotated`

Expected local archive:

```text
rlc-annotated-main.zip
```

Source files expected by the importer:

```text
LICENSE
README.md
documents.csv
sentences.csv
annotations.csv
rlc_test.csv
```

License / redistribution:

- The prior archive contained an MIT License file.
- Redistribution is allowed under MIT terms, but the default project policy still avoids committing the full corpus.

Schema and counts in the previously inspected archive:

- `documents.csv`: 2,004 documents.
- `sentences.csv`: 31,519 sentence rows with `text`, `corrected`, `status`, document linkage.
- `annotations.csv`: 41,410 edit annotations with `tag`, `quote`, `correction`, token-boundary `start`/`end`, and `annotation_source`.
- `rlc_test.csv`: 519 edit rows; grouped importer output has 216 unique sentence pairs.

Importer behavior:

- The importer accepts either an unpacked directory or a local ZIP archive.
- Main sentence records are emitted from `sentences.csv` and enriched with grouped annotations from `annotations.csv` plus document metadata.
- By default, unchanged sentence pairs are skipped; use `--keep-uncorrected` to keep clean controls.
- `--part all` appends grouped `rlc_test.csv` pairs.
- Missing expected CSV members fail with explicit file-member errors.

Command:

```bash
python scripts/import/rlc_annotated.py \
  --input /path/to/rlc-annotated-main.zip \
  --output data/eval/gec/rlc-annotated.jsonl \
  --part all
```

Observed default output on the previously attached archive with `--part all`: **16,512** normalized records.

### LORuGEC / `lorugec`

Expected local archive:

```text
LORuGEC-main.zip
```

Source files expected by the importer:

```text
README.md
LORuGEC.xlsx
LORuGEC.val.m2
LORuGEC.test.m2
```

License / redistribution:

- The referenced README asks users to cite the BEA 2025 paper.
- No explicit OSI-style license file was present in the prior inspected archive. Treat redistribution as unclear until maintainers publish explicit terms.
- Do not commit the full workbook or M2 files.

Schema and counts in the previously inspected archive:

- `LORuGEC.xlsx`: 960 data rows after the header.
- Workbook columns include rule name, rule definition, source URL, grammar section, base-model difficulty, incorrect sentence, corrected sentence, whether both sentences match, and `Prompt/Query` marker.
- `Prompt` rows map to validation: 348 rows.
- `Query` rows map to test: 612 rows.
- Grammar sections in the workbook: Spelling, Punctuation, Grammar, Semantics.
- `LORuGEC.val.m2`: 348 M2 blocks.
- `LORuGEC.test.m2`: 612 M2 blocks.

Importer behavior:

- The importer accepts either an unpacked directory or a local ZIP archive.
- Default mode reads `LORuGEC.xlsx` using stdlib XML parsing, not `pandas` or `openpyxl`, to preserve rule metadata without extra dependencies.
- XLSX source and correction sentences are detokenized before writing JSONL, so punctuation spacing artifacts such as `слово , слово .` do not dominate benchmark diagnostics.
- `--source m2` is available as a fallback when only M2 files are present.
- Spreadsheet rows without explicit spans create a pair-level edit note. M2 rows preserve span and correction fields.

Command:

```bash
python scripts/import/lorugec.py \
  --input /path/to/LORuGEC-main.zip \
  --output data/eval/gec/lorugec.jsonl
```

Fallback command:

```bash
python scripts/import/lorugec.py \
  --input /path/to/LORuGEC-main.zip \
  --source m2 \
  --output data/eval/gec/lorugec-m2.jsonl
```

Observed default output on the previously attached archive: **960** normalized records.

### RuSpellGold / `ruspellgold`

Expected local archive:

```text
RuSpellGold.zip
```

Source files expected by the importer:

```text
README.md
RuSpellGold.py
data/complete_test/test.json
data/aranea/split.json
data/literature/split.json
data/news/split.json
data/social_media/split.json
data/strategic_documents/split.json
```

License / redistribution:

- Dataset card declares `apache-2.0`.
- README also notes that copyright of texts from source publications/resources remains with original authors or publishers.
- Redistribution is broadly allowed under Apache-2.0 for the dataset package, but the source-text copyright caveat means the project should still avoid committing full normalized dumps by default.

Schema and counts in the previously inspected archive:

- Complete test JSONL: 1,711 sentence pairs.
- Domain shards: aranea 756, literature 260, news 245, social_media 200, strategic_documents 250.
- Fields: `source`, `correction`, `domain`.
- No labels, spans, or rule annotations are present; importer assigns the coarse label `spelling`.

Importer behavior:

- The importer accepts either an unpacked directory or a local ZIP archive.
- Default mode uses `data/complete_test/test.json`.
- Identical source/correction pairs are skipped by default; use `--keep-identical` to keep all clean controls.
- `--split domains` concatenates domain shards instead of the complete duplicate.

Command:

```bash
python scripts/import/ruspellgold.py \
  --input /path/to/RuSpellGold.zip \
  --output data/eval/spelling/ruspellgold.jsonl
```

Observed default output on the previously attached archive: **1,213** normalized records.

## Generic pair importer

For ad-hoc TSV/CSV sentence pairs without useful metadata:

```bash
python scripts/import/gec_pairs_to_jsonl.py input.tsv data/eval/gec/custom.jsonl \
  --dataset custom \
  --delimiter $'\t' \
  --source-col 0 \
  --target-col 1 \
  --label grammar
```

Prefer dataset-specific importers when source labels, spans, domains, or rule metadata exist.

## Candidate future datasets

Keep RULEC-GEC, EACL 2024 multi-reference Russian GEC, AI Forever spellcheck/punctuation, and MultiGEC 2025 as evaluation candidates, not as blindly copied rule text.
