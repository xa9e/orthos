# CLI usage and integration notes

`orthos` keeps command-line behavior deterministic enough for editors, CI, and batch processing. Rule ids are stable; JSON issue output is sorted by byte span, then rule id.

## Basic commands

```bash
cargo run -- check examples/bad.txt --rules rules
cargo run -- check examples/bad.txt --rules rules --format json
cargo run -- validate --rules rules
cargo run -- test-examples --rules rules
```

Read from STDIN by omitting the input path:

```bash
cat examples/bad.txt | cargo run -- check --rules rules --format json
```

## Plan/debug command

Use `plan` to inspect selected and skipped rules without running detectors:

```bash
cargo run -- plan --rules rules
cargo run -- plan --rules rules --profile strict
cargo run -- plan --rules rules --profile strict --format json
cargo run -- plan --rules rules --status research --format json
```

The JSON plan summary includes selected/skipped counts, selected rule metadata, required capabilities, and machine-readable skipped-rule reasons.

## Profiles

Profiles are a first-stage rule gate. Additional domain, severity, status, and rule-id filters are applied after the selected profile.

- `default` — implemented rules considered conservative for normal use. It excludes planned/research rules, style-only checks, advanced demo rules, and heuristic checks unless explicitly requested.
- `strict` — all implemented rules, including style, advanced demo, typography, and heuristic checks.
- `typography-only` — implemented typography-domain rules only.
- `grammar-research` — research grammar rules only. Current research/manual rules may document future work and produce no issues until a detector exists.

Examples:

```bash
cargo run -- check examples/bad.txt --rules rules --profile default
cargo run -- check examples/bad.txt --rules rules --profile strict
cargo run -- check examples/bad.txt --rules rules --profile typography-only
cargo run -- list-rules --rules rules --profile grammar-research
```

## Rule filtering

Use filters for editor integrations, CI policy, or targeted regression tests.

```bash
cargo run -- check examples/bad.txt --rules rules --domain punctuation
cargo run -- check examples/bad.txt --rules rules --severity error
cargo run -- check examples/bad.txt --rules rules --rule-id ru.punctuation.no_space_before_mark
cargo run -- check examples/bad.txt --rules rules --exclude-rule ru.typography.hyphen_instead_of_dash
cargo run -- list-rules --rules rules --status default-safe
cargo run -- list-rules --rules rules --all --status research
```

Filters accept repeated or comma-separated values:

```bash
cargo run -- check examples/bad.txt --rules rules \
  --profile strict \
  --domain punctuation,orthography \
  --severity error,warning
```

Status filters override the profile gate:

- `default-safe`
- `implemented`
- `planned`
- `research`

Exact rule-id includes are explicit selections. Exact excludes always win.

## Rule inventory

`list-rules` prints tab-separated fields:

```text
rule_id domain severity status detector_kind required_capabilities profile_visibility title
```

Example:

```bash
cargo run -- list-rules --rules rules --all
cargo run -- list-rules --rules rules --profile strict --domain typography
```

`required_capabilities` is `-` when the rule declares none. `profile_visibility` is `default-safe`, `strict`, or `non-executable`.

## Suppressions

Suppressions are disabled unless the caller opts in. This prevents accidental text content from changing diagnostics.

Whole-file command-line suppression:

```bash
cargo run -- check input.txt --rules rules \
  --suppress-rule ru.punctuation.no_space_before_mark
```

Suppress every diagnostic for one run:

```bash
cargo run -- check input.txt --rules rules --suppress-rule all
```

Inline suppressions require `--allow-inline-suppressions`:

```text
Привет , мир. # orthos-disable-line ru.punctuation.no_space_before_mark
Я знаю что он придёт. # orthos-disable-line
# orthos-disable-next-line ru.typography.multiple_spaces
Слишком  много пробелов.
# orthos-disable-file ru.style.pleonasm_phrase_seed
```

Supported directives:

- `orthos-disable-line [rule-id[,rule-id...]]`
- `orthos-disable-next-line [rule-id[,rule-id...]]`
- `orthos-disable-file [rule-id[,rule-id...]]`

Without rule ids, a directive suppresses all rules for its scope. Use this sparingly; rule-specific suppressions are better for editor and CI auditability.

## Execution strategy

Default execution is serial:

```bash
cargo run -- check examples/bad.txt --rules rules --execution-strategy serial
```

Opt-in deterministic parallel execution uses scoped standard-library threads and preserves final issue ordering:

```bash
cargo run -- check examples/bad.txt --rules rules \
  --profile strict \
  --execution-strategy deterministic-parallel
```

For small files, parallel mode can be slower. It exists as a deterministic platform seam, not as magic speed powder.

## Stable JSON contract

`--format json` for `check` writes a stable array of issue objects to stdout. Timings, when enabled, go to stderr so JSON consumers do not need a second schema.

Each issue contains:

- `rule_id` — stable rule identifier.
- `severity` — `info`, `warning`, or `error`.
- `message` — detector message for users.
- `span.start` and `span.end` — UTF-8 byte offsets, half-open range.
- `start.line`, `start.column`, `end.line`, `end.column` — 1-based Unicode scalar columns suitable for editor diagnostics.
- `replacement` — exact replacement when the detector can provide one.
- `suggestion` — broader rule-level suggestion when available.
- `explanation` — rule-level explanation when available.
- `source_refs` — stable source reference ids from the corpus when available.
- `excerpt` — nearby human-readable context.

Output sorting is deterministic: byte start, byte end, rule id.

## Timing/performance mode

```bash
cargo run -- check examples/bad.txt --rules rules --profile strict --timings
cargo run -- check examples/bad.txt --rules rules --format json --timings > issues.json
```

Timing output is per-rule microseconds and issue count, sorted by rule id on stderr. It is useful for quick regressions, not statistically rigorous benchmark claims.

## GNU Parallel batch usage

Arch Linux setup:

```bash
sudo pacman -S --needed rust cargo jq parallel
```

Batch JSON per text file:

```bash
find corpus -type f -name '*.txt' -print0 \
  | parallel -0 'cargo run --quiet -- check {} --rules rules --format json > {.}.rulang.json'
```

Strict CI-style batch check:

```bash
find corpus -type f -name '*.txt' -print0 \
  | parallel -0 'cargo run --quiet -- check {} --rules rules --profile strict --format json > {.}.rulang.json'
```

Collect only error diagnostics:

```bash
find corpus -type f -name '*.txt' -print0 \
  | parallel -0 'cargo run --quiet -- check {} --rules rules --severity error --format json > {.}.errors.json'
```

The command starts a fresh process per file. For very large corpora, a future batch subcommand can reuse the loaded corpus, detector registry, capability registry, morphology lexicon, and compiled detector state across files.

## Engine architecture

See `docs/engine-architecture.md` for analysis context, detector registry, capability contracts, execution plans, suppressions, JSON contracts, performance, and future editor/LSP details.
