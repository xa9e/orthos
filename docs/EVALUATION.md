# Evaluation pipeline

This project evaluates deterministic diagnostics first. It does **not** optimize generative correction metrics yet.

## Pipeline

```text
third-party archive or extracted directory
  -> scripts/import/{rlc_annotated,lorugec,ruspellgold}.py
  -> normalized JSONL
  -> scripts/eval/benchmark_jsonl.py
  -> report with diagnostic rates, corrected-target false positives, rule/label breakdowns, examples, and runtime/configuration metadata
```

Registered datasets are described in `data/dataset-registry.json`; see `docs/DATASETS.md`.

## Import examples

```bash
mkdir -p data/eval/gec data/eval/spelling reports/eval

python scripts/import/rlc_annotated.py \
  --input /datasets/rlc-annotated-main.zip \
  --output data/eval/gec/rlc-annotated.jsonl \
  --part all

python scripts/import/lorugec.py \
  --input /datasets/LORuGEC-main.zip \
  --output data/eval/gec/lorugec.jsonl

python scripts/import/ruspellgold.py \
  --input /datasets/RuSpellGold.zip \
  --output data/eval/spelling/ruspellgold.jsonl
```

All dataset-specific importers accept either a ZIP archive or an extracted directory.

## One-command registered run

`run_benchmark.py` reads `data/dataset-registry.json`, imports a local archive, then invokes the benchmark.

```bash
python scripts/eval/run_benchmark.py \
  --dataset ruspellgold \
  --archive /datasets/RuSpellGold.zip \
  --checker-bin target/debug/orthos \
  --rules rules \
  --profile default \
  --limit 500
```

Use `--dry-run` to inspect commands without execution:

```bash
python scripts/eval/run_benchmark.py \
  --dataset lorugec \
  --archive /datasets/LORuGEC-main.zip \
  --dry-run
```

## Benchmark command

Accurate per-record smoke run:

```bash
python scripts/eval/benchmark_jsonl.py \
  --input data/eval/gec/lorugec.jsonl \
  --rules rules \
  --profile default \
  --mode per-record \
  --limit 100 \
  --output reports/eval/lorugec-smoke.json
```

Faster exploratory batch run:

```bash
python scripts/eval/benchmark_jsonl.py \
  --input data/eval/gec/rlc-annotated.jsonl \
  --rules rules \
  --profile default \
  --mode batch \
  --output reports/eval/rlc-annotated-batch.json
```

`batch` mode runs the checker once for all source sentences and once for corrected targets. It is faster, but global/line-sensitive detectors can differ from `per-record`. Use `per-record` for release gates and regression decisions.

When a compiled binary exists, avoid `cargo run` overhead:

```bash
cargo build
python scripts/eval/benchmark_jsonl.py \
  --input data/eval/spelling/ruspellgold.jsonl \
  --checker-bin target/debug/orthos \
  --rules rules \
  --profile default \
  --mode per-record
```

When you want the benchmark to use the exact cargo contract without compiling first, pass the wrapper:

```bash
python scripts/eval/benchmark_jsonl.py \
  --input data/eval/gec/lorugec.jsonl \
  --checker-bin scripts/eval/run_cargo_checker.py \
  --rules rules \
  --profile default \
  --mode per-record \
  --limit 50
```

The wrapper invokes:

```bash
cargo run --quiet -- check <file> --rules rules --format json --profile default
```

If `cargo` is unavailable, use a compiled binary or a fake checker for tests; the wrapper exits with a clear error instead of pretending the benchmark ran.

With the demo morphology lexicon:

```bash
python scripts/eval/benchmark_jsonl.py \
  --input data/eval/gec/rlc-annotated.jsonl \
  --checker-bin target/debug/orthos \
  --rules rules \
  --morph-lexicon data/lexicon/demo_morph.tsv \
  --profile strict \
  --mode per-record \
  --limit 500
```

Forward extra checker arguments with repeated `--checker-arg=...` values:

```bash
python scripts/eval/benchmark_jsonl.py \
  --input data/eval/gec/rlc-annotated.jsonl \
  --checker-bin target/debug/orthos \
  --rules rules \
  --checker-arg=--status=implemented \
  --checker-arg=--domain=orthography
```

## Report fields

The benchmark reports:

- configuration block: input path, output path, mode, limit, rules path, profile, morphology lexicon, checker command, extra checker args;
- number of examples, expected dirty examples, expected clean examples, and token counts;
- diagnostics on source inputs;
- diagnostics per 1,000 source sentences and per 1,000 source tokens;
- diagnostics on corrected targets;
- diagnostics per 1,000 corrected-target sentences and per 1,000 corrected-target tokens;
- corrected-target false-positive sentence count and rate;
- false-positive diagnostics per 1,000 target tokens;
- source detection precision, recall, and F0.5 proxies;
- full issues by rule id plus top rule ids;
- issues and detected records grouped by dataset label;
- examples with unexpected diagnostics on corrected targets;
- label-domain hit rate for labels that can be mapped to rough rule domains;
- runtime summary with checker invocation count and average checker time.

The precision/recall/F0.5 fields are proxies, not exact GEC metrics. The label-domain hit rate is intentionally coarse: it maps labels to domains such as `orthography`, `punctuation`, `grammar`, and `style`, then checks whether any fired rule belongs to the same broad domain. See `docs/evaluation-methodology.md` for the ugly caveats before using these numbers in public quality claims.

## Dataset regression fixtures

Curated dataset records live in `testdata/fixtures/eval/gec_dataset_regressions.jsonl`
and are executed by `tests/dataset_regressions.rs` (`cargo test --test dataset_regressions`).
Each record keeps the benchmark JSONL schema (`input`, `correction`, `targets`,
`edits`, `labels`) and adds an `expectations` object:

- `correction_silent_rules` — rules that must not fire on the corrected target
  (overrides the default guarded set; precision side);
- `input_must_trigger` — rules that must fire on the erroneous input
  (recall claims);
- `input_must_not_trigger` — rules that must stay silent even on the erroneous
  input.

Token morphology for these records is curated next to them in
`testdata/fixtures/eval/dataset_regressions_morph.tsv`. Adding a record means
adding its morphology in the same change, so a regression stays reproducible
without external dictionaries.

## Smoke tests

Importer and benchmark smoke tests use only tiny local fixtures:

```bash
python -m unittest tests.test_importers -v
```

If `pytest` is installed:

```bash
python -m pytest tests/test_importers.py
```

## Arch Linux setup

```bash
sudo pacman -S --needed rust cargo python jq parallel
```

No Python package installation is required for the current importers or benchmark script.

## GNU Parallel examples

Import all local archives in parallel:

```bash
parallel --halt now,fail=1 ::: \
  'python scripts/import/rlc_annotated.py --input /datasets/rlc-annotated-main.zip --output data/eval/gec/rlc-annotated.jsonl --part all' \
  'python scripts/import/lorugec.py --input /datasets/LORuGEC-main.zip --output data/eval/gec/lorugec.jsonl' \
  'python scripts/import/ruspellgold.py --input /datasets/RuSpellGold.zip --output data/eval/spelling/ruspellgold.jsonl'
```

Run small per-dataset benchmark samples in parallel after `cargo build`:

```bash
parallel --halt now,fail=1 ::: \
  'python scripts/eval/benchmark_jsonl.py --input data/eval/gec/rlc-annotated.jsonl --checker-bin target/debug/orthos --rules rules --profile default --limit 200 --output reports/eval/rlc-smoke.json' \
  'python scripts/eval/benchmark_jsonl.py --input data/eval/gec/lorugec.jsonl --checker-bin target/debug/orthos --rules rules --profile default --limit 200 --output reports/eval/lorugec-smoke.json' \
  'python scripts/eval/benchmark_jsonl.py --input data/eval/spelling/ruspellgold.jsonl --checker-bin target/debug/orthos --rules rules --profile default --limit 200 --output reports/eval/ruspellgold-smoke.json'
```

## Current caveats

- Full corpora are generated locally; they are not committed.
- LORuGEC redistribution terms are unclear in the prior inspected archive. Treat generated JSONL as local-only until license is explicit.
- RuSpellGold includes source-text copyright caveats despite Apache-2.0 dataset metadata.
- The checker is detection-first. Do not report correction F-scores from this pipeline as if they were full GEC quality.
- Benchmark reports examples with target diagnostics, but it cannot prove whether the reference is wrong or the checker is noisy without manual inspection.
