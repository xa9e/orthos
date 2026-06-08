## Normalized YAML shape

A rule has the historical core fields plus optional model metadata:

```yaml
- id: ru.grammar.subject_predicate_agreement_basic
  title: Согласование подлежащего и сказуемого в числе и роде
  domain: grammar
  level: advanced
  status: research
  severity: warning
  source_refs: [ruscorpora_morph, ruscorpora_syntax]
  requires: [morphology, syntax]

  rule_family: syntax_dependent
  confidence: low
  false_positive_risk: high

  pattern:
    kind: dependency
    description: Typed subject-predicate dependency with finite predicate morphology.
    captures: [subject, predicate]

  constraints:
    - kind: agreement
      features: [number, gender]
      description: Predicate form must be compatible with subject number and, where applicable, gender.
    - kind: coordination
      description: Coordinated subjects change agreement target and default number.

  exceptions:
    - kind: idiom_fixed_expression_exception
      value: fixed expression or curated exception id

  evidence:
    - kind: corpus_attestation
      source_ref: ruscorpora_syntax

  related_rules: [ru.grammar.participle_agreement]
  supersedes: []
  implementation_notes: Keep as research until parser confidence and coordination handling are measurable.

  detector:
    kind: manual
    note: Research model only.
```

## Rule id convention

Rule ids are stable public identifiers. The validated shape is:

```text
ru.<namespace>.<snake_case_slug>
```

Each component after `ru` must start with an ASCII lowercase letter and then use only ASCII lowercase letters, digits, and single underscores. Empty components, uppercase letters, hyphens, trailing underscores, and repeated underscores are rejected. The namespace is usually the visible `domain`, but architecture-oriented namespaces such as `syntax` are allowed when they are intentional and documented by the rule title/model.

## Source metadata

`source_refs` remain backward-compatible: `id`, `title`, `url`, and `note` still work. New optional fields make source discipline explicit:

- `source_type`: `normative_reference`, `corpus_annotation_standard`, `dictionary`, `donor_project`, `dataset_taxonomy`, `online_reference`, `editorial_practice`, or `project_internal`;
- `bibliographic_pointer`: edition/page/book pointer when a URL is insufficient;
- `license_note`: redistribution and usage constraints;
- `year`: year or edition marker;
- `authority`: `normative`, `descriptive`, `corpus_derived`, or `project_internal`.

These fields do not license external material into the repository. They document how a source may be used and make it harder to accidentally convert a dataset label into a copied rule.
