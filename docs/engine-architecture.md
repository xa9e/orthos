# Engine architecture: analysis context, registry, plans, suppressions, scale

This document describes the deterministic execution layer used by `orthos`. The engine deliberately favors explicit contracts over plugin magic: predictable inputs, stable rule ordering, auditable skipped-rule reasons, and no hidden runtime state.

## Core execution flow

The primary entry point is `Checker` in `src/engine.rs`.

Execution is staged:

1. Load and validate YAML corpus into `Corpus`.
2. Build a `Checker` with:
   - a `DetectorRegistry`;
   - a `CapabilityRegistry`;
   - a `MorphAnalyzer` implementation.
3. Convert CLI/API options into `CheckOptions`.
4. Build an `ExecutionPlan`:
   - apply profile/status/domain/severity/rule-id filters;
   - reject unknown detector kinds;
   - reject rules requiring unavailable capabilities;
   - sort executable rules by stable rule id.
5. Build a reusable `AnalysisContext` for the input document.
6. Run each selected detector using the requested `ExecutionStrategy`.
7. Apply suppressions after each detector output.
8. Sort final issues by `(span.start, span.end, rule_id)`.
9. Return `CheckResult` with issues, a serializable execution-plan summary, and optional per-rule timings.

`check --format json` still prints the issue array for backward compatibility. API users that serialize `CheckResult` also get `execution_plan`.

## Reusable analysis context

`src/detector.rs` now exposes `AnalysisContext` as the shared per-document analysis object. It owns or references:

- input text;
- cached `LineIndex`;
- lazily initialized token cache;
- lazily initialized word-token cache;
- morphology analyzer access.

`DetectorContext` remains the detector-facing façade and preserves compatibility-oriented fields (`text`, `line_index`, `morph`) while also exposing `tokens()` and `word_tokens()` methods backed by the shared context.

Important constraints:

- detectors must treat `AnalysisContext` as read-only;
- caches are deterministic and keyed only by the current input text;
- syntax algorithms remain outside this layer; future syntax document caches can hang off the same context without forcing every detector to retokenize;
- the context uses standard-library synchronization, so opt-in parallel execution can share cached tokens safely.

This gives editor/LSP and batch integrations one obvious place to add future sentence, syntax-document, and morphology memoization without turning detectors into stateful objects.

## Detector registry design

`src/detector.rs` exposes:

- `DetectorRunner` — trait implemented by concrete detector runners;
- `DetectorMetadata` — static metadata for one detector kind;
- `DetectorRegistry` — `BTreeMap` from detector kind to runner;
- `default_detector_registry()` — global registry used by compatibility helpers.

Each runner handles exactly one YAML detector kind. `DetectorRegistry::validate()` checks registry-key/metadata agreement, non-empty descriptions, duplicate capabilities, and sorted capability lists. `BTreeMap` provides stable detector-kind ordering.

Unknown detector handling is explicit in execution planning and runtime execution:

```text
unknown detector kind `<kind>` for rule `<rule_id>`
```

Serde will still reject unknown YAML enum variants during normal corpus loading. The runtime check exists for programmatic callers and future extension points.

## Capability model

Rules declare `requires` in YAML. `CapabilityRegistry` declares what the current runtime can execute.

Default runtime capabilities:

- `tokenization`
- `sentence_boundaries`
- `regex`
- `lexicon`
- `morphology`
- `syntax`

This does **not** claim a production syntactic parser. It means the current engine can run the lightweight syntax-aware heuristics already present in `src/syntax.rs`.

`ExecutionPlan` records skipped rules with `SkippedRuleReason::MissingCapabilities`. The serializable `ExecutionPlanSummary` includes:

- selected rule count;
- skipped rule count;
- selected rule ids, detector kinds, domain, severity, status, and required capabilities;
- skipped rules with machine-readable reasons.

## Execution strategies and deterministic parallelism

`CheckOptions.execution_strategy` supports:

- `Serial` — default behavior;
- `DeterministicParallel` — opt-in standard-library scoped-thread execution.

Parallel mode is intentionally conservative:

- no new runtime dependency;
- no dynamic loading;
- detectors receive shared read-only context;
- each rule output is collected independently;
- final issues are sorted by the same `(span.start, span.end, rule_id)` contract;
- per-rule timings are sorted by rule id before returning.

The point is not to pretend this is a high-throughput distributed scheduler. It is a stable execution-strategy seam that can be benchmarked and improved without changing detector semantics.

## Profile semantics

Profiles are first-stage gates. Other filters are applied on top.

`default`:
: implemented conservative checks only. Excludes planned/research rules, style-only rules, advanced/expert/heuristic levels, and tags such as `demo`, `experimental`, `research`, `roadmap`, and `strict-only`.

`strict`:
: all implemented rules, including style, heuristic, typography, and demo morphology checks.

`typography-only`:
: implemented rules in the typography domain only.

`grammar-research`:
: research rules in the grammar domain. Most are documentation/roadmap rules and may be skipped by capability checks or execute as manual no-op detectors.

Exact `--rule-id` includes are explicit selections and can select a rule outside the profile. Exact `--exclude-rule` always wins.

Status filters override the profile gate:

- `default-safe`
- `implemented`
- `planned`
- `research`

Domain and severity filters are intersections, not unions with profile output.

## Suppression semantics

Suppressions are disabled by default. Random text content should not silently mutate diagnostics unless a caller opts in.

CLI/file-level suppression:

```bash
cargo run -- check input.txt --rules rules --suppress-rule ru.punctuation.no_space_before_mark
cargo run -- check input.txt --rules rules --suppress-rule all
```

Inline suppression, enabled only with `--allow-inline-suppressions`:

```text
# orthos-disable-line ru.punctuation.no_space_before_mark
# orthos-disable-next-line ru.typography.multiple_spaces
# orthos-disable-file ru.style.pleonasm_phrase_seed
```

A directive without rule ids suppresses all rules for that directive scope. Rule-specific suppressions are preferred because they are auditable.

Suppressions are applied after detector execution and before final issue sorting. Detectors stay pure: they do not need to know about comments, CLI policy, editor policy, or CI policy.

## JSON output contract

`check --format json` writes an array of issue objects to stdout. Timings go to stderr.

Stable issue fields:

- `rule_id`
- `severity`
- `message`
- `span.start`
- `span.end`
- `start.line`
- `start.column`
- `end.line`
- `end.column`
- `replacement` when available
- `suggestion` when available
- `explanation` when available
- `source_refs` when available
- `excerpt`

Sorting is stable:

```text
(span.start, span.end, rule_id)
```

Do not add nondeterministic fields such as wall-clock timestamps, thread ids, randomized confidence values, or hash-map iteration order to stdout JSON. Put diagnostics/debug/performance noise on stderr or behind a separate explicit schema such as `plan --format json`.

## CLI/reporting contracts

Compatibility commands remain valid:

```bash
cargo run -- check examples/bad.txt --rules rules
cargo run -- validate --rules rules
cargo run -- test-examples --rules rules
```

Additional platform-oriented commands/options:

```bash
cargo run -- plan --rules rules --profile strict
cargo run -- plan --rules rules --profile strict --format json
cargo run -- check examples/bad.txt --rules rules --profile strict --execution-strategy deterministic-parallel
cargo run -- list-rules --rules rules --all
```

`list-rules` now exposes detector kind, required capabilities, and profile visibility so CI/editor integrations can inspect what they are about to enable.

## Performance roadmap

Near-term, low-risk work:

- precompile regex detectors at registry/corpus-load time instead of per rule execution;
- cache normalized token streams or phrase maps where profiling proves value;
- expose batch checking that reuses loaded corpus, registry, morphology, and compiled detector state;
- add deterministic benchmark fixtures around representative text sizes;
- add syntax document and sentence caches behind `AnalysisContext` where profiling proves repeated analysis cost.

Parallel execution should remain opt-in until benchmark data proves it helps. Thread overhead can dominate small files; using it blindly would be classic cargo-cult performance theater.

## Future LSP/editor integration notes

The engine returns byte offsets and 1-based line/column positions. LSP integration should add a thin adapter that converts to UTF-16 positions where required by the client protocol.

Recommended integration shape:

- keep `Checker` long-lived per workspace/config;
- rebuild corpus/registry only when rules or profiles change;
- reuse `AnalysisContext` per document check;
- run `default` profile for live typing;
- run `strict` profile on save or CI;
- expose code actions only when `replacement` is present;
- surface `explanation`, `source_refs`, and skipped-rule plan data in details panes;
- store suppressions as explicit text edits, not hidden editor state.

Do not make the LSP server own rule semantics. It should be transport glue around the engine API.
