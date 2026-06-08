## `source_refs`

Every rule must declare at least one `source_ref`. Sources are declared at the YAML-file level:

```yaml
source_refs:
  - id: gramota_lopatin_full
    title: В. В. Лопатин (ред.). Правила русской орфографии и пунктуации. Полный академический справочник
    source_type: normative_reference
    year: current online edition
    authority: normative
    url: https://gramota.ru/biblioteka/spravochniki/pravila-russkoy-orfografii-i-punktuatsii
    license_note: Reference pointer only; rule prose and examples are project-authored paraphrases.
```

Optional source metadata:

- `source_type`: `normative_reference`, `corpus_annotation_standard`, `dictionary`, `donor_project`, `dataset_taxonomy`, `online_reference`, `editorial_practice`, `project_internal`;
- `url`: stable public URL when available;
- `bibliographic_pointer`: book/edition/page pointer when URL is insufficient;
- `license_note`: redistribution and usage discipline;
- `year`: year or edition marker;
- `authority`: `normative`, `descriptive`, `corpus_derived`, or `project_internal`;
- `note`: project-specific explanation.

Source metadata is additive and backward-compatible. It exists to prevent cargo-cult citations and accidental dataset laundering.

## `confidence`

`confidence` estimates correctness of the rule model and current strategy:

- `high` — deterministic or tightly bounded;
- `medium` — stable norm, but non-trivial ambiguity or exceptions;
- `low` — immature analyzer/evidence/strategy.

Implemented rules must declare it. Planned/research rules should declare it when the field improves review quality.

## `false_positive_risk`

`false_positive_risk` estimates how likely a rule is to fire on acceptable text:

- `low` — false alarms should be rare;
- `medium` — context can change the judgment;
- `high` — detector needs morphology, syntax, semantics, benchmark gating, or exception lists.

Risk is independent of confidence. A norm can be correct while the detector is risky. That distinction is the whole point.

## Minimal templates

### Implemented surface rule

```yaml
- id: ru.typography.example_surface_rule
  title: Краткое название явления
  domain: typography
  rule_family: typography
  level: basic
  status: implemented
  severity: warning
  source_refs: [editorial_style_general]
  requires: [tokenization]
  confidence: high
  false_positive_risk: low
  pattern:
    kind: surface
    description: Surface token-adjacency signature.
  evidence:
    - kind: expert_review
      note: Project-authored deterministic typography rule.
  detector:
    kind: manual
    note: Replace with an executable detector before status becomes implemented.
  examples:
    valid: ["Корректный пример."]
    invalid: ["Некорректный  пример."]
```

The template intentionally shows the metadata first. For actual `implemented` status, `detector.kind` must be executable, not `manual`.

### Research grammar rule

```yaml
- id: ru.grammar.example_government_research
  title: Управление зависимого слова
  domain: grammar
  rule_family: syntax_dependent
  level: advanced
  status: research
  severity: warning
  source_refs: [gramota_lopatin_full, ruscorpora_syntax]
  requires: [morphology, syntax, lexicon]
  confidence: low
  false_positive_risk: high
  pattern:
    kind: dependency
    description: Head lemma with dependent case/preposition frame.
    captures: [head, dependent]
  constraints:
    - kind: government
      description: Head licenses only selected case/preposition frames.
  evidence:
    - kind: normative_source
      source_ref: gramota_lopatin_full
    - kind: syntax
      source_ref: ruscorpora_syntax
  detector:
    kind: manual
    note: Requires dependency parser and valency/government dictionary.
  examples:
    valid: ["Согласно приказу, встреча перенесена."]
    invalid: ["Согласно приказа, встреча перенесена."]
```

## Validation commands

Run these before handing off:

```bash
cargo fmt
cargo test
cargo run -- validate --rules rules
cargo run -- test-examples --rules rules
```

If Rust tooling is unavailable, run static YAML validation and document the limitation in the change notes. Do not claim compiled checks passed unless they actually ran.
