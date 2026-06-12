# orthos

Rule-first Russian proofreading engine in Rust.

This is not a regex toy. The project is already building a transparent
language-model layer for Russian: tokenization, morphology, agreement,
government, quantity frames, punctuation safe zones, suppressions, debug
snapshots, dataset importers, and regression fixtures.

## What It Already Catches

Run the strict profile to see the current research-grade surface:

```bash
cargo run -- check examples/bad.txt --rules rules --profile strict
```

Representative checks that already work:

```text
Согласно новому приказа, встреча перенесена.
→ preposition + nominal-group government conflict

Два новых дом стояли рядом.
→ numeral + short nominal-group agreement conflict

Сто двадцать два новых дом стояли рядом.
→ typed compound-numeral component selects the governing numeral part

Новый важному приказу присвоили номер.
→ non-adjacent modifier/head agreement conflict inside a nominal group

Девочка пришёл.
→ subject/predicate agreement conflict

Он говорит о задачу.
→ verb valency frame: prepositional object must use the modeled case

Он помогает проектом.
→ verb valency frame: direct-object seed expects another case

Он хочет разсказать и потом расбить лёд.
→ word-formation-backed prefix-final з/с assimilation

Из за дождя кое кто опоздал. Я то понял. Ну ка проверь.
→ model-backed hyphenation for compound prepositions, кое-, -то, -ка

Он ушёл потому что устал. Конечно это риск.
→ punctuation with clause-marker and introductory-phrase safe-zone logic

Мaма сказала: я незнаю, но надо учится.
→ mixed alphabet, common не-confusable, and -тся/-ться heuristic

Пол лимона осталось.
→ conservative пол- hyphenation before л/vowel/proper-name contexts
```

The engine also keeps negative fixtures for false-positive risk. For example,
corrected targets from LORuGEC, RLC, and RuSpellGold are stored as regression
fixtures so model-backed grammar rules do not keep firing on normal sentences
like `В один из летних вечеров...`, `От него целый год...`, or
`к потере слуха и зрения`.

## Philosophy

The long-term product direction is a serious Russian grammar/proofreading
checker with minimal ML and maximal explicit linguistic structure. Regex is
allowed only for surface phenomena. Grammar rules should be backed by reusable
models:

- typed morphology and feature unification;
- preposition and verb government frames;
- nominal-group and quantity models;
- clause boundaries and syntactic safe zones;
- debug snapshots with machine-readable proofs;
- dataset-based regression tests.

## Install on Arch Linux

```bash
sudo pacman -S --needed rust cargo jq parallel python-pytest
```

## Run

```bash
cargo run -- check examples/bad.txt --rules rules
cargo run -- check examples/bad.txt --rules rules --profile strict
cargo run -- check examples/bad.txt --rules rules --format json | jq
```

Use a custom morphology TSV:

```bash
cargo run -- check input.txt --rules rules --morph-lexicon data/lexicon/demo_morph.tsv
```

Inspect the debug layer:

```bash
cargo run -- debug input.txt --rules rules --profile strict | jq
```

Validate the rule corpus:

```bash
cargo run -- validate --rules rules
```

Run embedded rule examples as executable specs:

```bash
cargo run -- test-examples --rules rules
```

Batch check with GNU Parallel:

```bash
find texts -type f -name '*.txt' -print0 | \
  parallel -0 'cargo run --quiet -- check {} --rules rules --format json > {.}.lint.json'
```

## Profiles and Filtering

Default checks run conservative implemented rules only. `strict` enables all
implemented rules, including style, heuristic, and advanced demo/model-backed
checks.

```bash
cargo run -- check examples/bad.txt --rules rules --profile default
cargo run -- check examples/bad.txt --rules rules --profile strict
cargo run -- check examples/bad.txt --rules rules --profile typography-only
cargo run -- list-rules --rules rules --all --status research
```

Targeted filters:

```bash
cargo run -- check examples/bad.txt --rules rules --domain punctuation --severity error
cargo run -- check examples/bad.txt --rules rules --rule-id ru.punctuation.no_space_before_mark
cargo run -- check examples/bad.txt --rules rules --exclude-rule ru.typography.hyphen_instead_of_dash
```

Suppressions are off by default. Enable inline directives explicitly:

```bash
cargo run -- check input.txt --rules rules --allow-inline-suppressions
cargo run -- check input.txt --rules rules --suppress-rule ru.punctuation.no_space_before_mark
```

See `docs/CLI_USAGE.md` for the JSON contract, suppression syntax, timing mode,
and batch examples.

## Evaluation Datasets

Local-only importers currently support:

- RLC annotated / RLC-GEC;
- LORuGEC;
- RuSpellGold.

The repository does not vendor full raw corpora by default. Use local archives
from your machine:

```bash
python scripts/eval/run_benchmark.py \
  --dataset lorugec \
  --archive "$HOME/Downloads/LORuGEC-main.zip" \
  --checker-bin target/debug/orthos \
  --rules rules \
  --profile strict \
  --limit 100
```

The checked-in fixture `testdata/fixtures/eval/gec_dataset_regressions.jsonl`
keeps a tiny, stable slice of real dataset-derived false-positive cases for
normal development.

## Full Morphology Cache

The repository includes `data/lexicon/demo_morph.tsv`, a small built-in lexicon
used by tests and examples. For broader morphology-backed checks, download the
OpenCorpora-derived cache from the GitHub release:

```bash
mkdir -p data/lexicon
curl -L -o data/lexicon/opencorpora.bincache.zst \
  https://github.com/xa9e/orthos/releases/download/morph-cache-v1/opencorpora.bincache.zst
curl -L -o data/lexicon/opencorpora.bincache.idx.zst \
  https://github.com/xa9e/orthos/releases/download/morph-cache-v1/opencorpora.bincache.idx.zst

zstd -d data/lexicon/opencorpora.bincache.zst -o data/lexicon/opencorpora.bincache
zstd -d data/lexicon/opencorpora.bincache.idx.zst -o data/lexicon/opencorpora.bincache.idx
```

After that, `orthos` automatically uses `data/lexicon/opencorpora.bincache`
when no `--morph-lexicon` is passed.

The cache is derived from the OpenCorpora morphological dictionary. OpenCorpora
data is distributed under CC BY-SA 3.0; the project code is licensed under
AGPL-3.0-or-later.

## Test

```bash
cargo fmt --check
cargo check
cargo test
cargo run -- validate --rules rules
cargo run -- test-examples --rules rules
python3 scripts/dev/project_health.py
python3 -m pytest -q tests/test_importers.py tests/dev_tools/test_project_health.py
git diff --check
```

## Current Detector Families

- whitespace, punctuation spacing, sentence-final punctuation, lowercase starts;
- repeated words and repeated punctuation;
- mixed Cyrillic/Latin words;
- common spelling and confusable-token seeds;
- clitic and compound-preposition hyphenation;
- `не + verb/gerund/participle` spacing;
- `-тся/-ться` context heuristic;
- `пол-` hyphenation in conservative contexts;
- prefix-final `з/с` assimilation through a seed word-formation model;
- missing comma before frequent subordinators with safe-zone suppression;
- introductory phrase comma at sentence start;
- coordination comma slots for homogeneous modifier series;
- adjective/noun and nominal-group modifier agreement;
- subject/predicate agreement;
- preposition case government;
- preposition + nominal-group government;
- numeral/noun and numeral + nominal-group quantity agreement;
- compound and typed-component compound numeral agreement;
- verb government from data-backed valency seeds;
- phrase-map style, pleonasm, and government seeds;
- debug proof generation for model-backed diagnostics.

## Development Notes

Read `IDEA.md` first. The project direction is rule-first and debug-first:
every serious grammar rule should become a small inspectable language model, not
a one-off detector.

Useful docs:

- `docs/DATASETS.md`
- `docs/CLI_USAGE.md`
- `docs/ARCHITECTURE.md`
- `docs/language-model/`
- `docs/debug-layer.md`

## License

The project is licensed under the GNU Affero General Public License,
version 3 or any later version (AGPL-3.0-or-later). See `LICENSE` for the full
text. External datasets keep their own licenses and are never vendored into
this repository; see `docs/DATASETS.md`.

## Git Bundle Workflow

Create a portable repository bundle:

```bash
git bundle create orthos.bundle --all
git bundle verify orthos.bundle
```

Restore it:

```bash
git clone orthos.bundle orthos
cd orthos
```

For a source-only archive without Git history:

```bash
git archive --format=tar HEAD | xz -T0 > orthos-source.tar.xz
```
