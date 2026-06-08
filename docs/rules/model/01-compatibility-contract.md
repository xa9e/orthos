# Formal rule model

This document defines the normalized, long-term model for the `orthos` rule corpus. The model is intentionally more expressive than the current detector set: some fields are metadata until richer morphology, syntax, and execution primitives are implemented.

The design goal is boring but crucial: every rule should say **what linguistic phenomenon it models**, **which evidence supports it**, **which runtime capabilities it needs**, and **why false positives are expected or bounded**. A YAML rule is not just a detector config.

## Compatibility contract

Existing rules remain valid. New fields are additive and optional unless a rule chooses to use the extended model.

The current schema validates deterministic, low-risk properties:

- known enum values for `rule_family`, `confidence`, `false_positive_risk`, `pattern.kind`, `constraints[].kind`, `exceptions[].kind`, and `evidence[].kind`;
- duplicate values in list-like fields such as `related_rules`, `supersedes`, `tags`, `pattern.captures`, and examples;
- invalid rule-id shape in `related_rules` and `supersedes`;
- unknown `related_rules` and `supersedes` ids after all YAML files are loaded;
- unknown `evidence[].source_ref` ids after all source refs are loaded;
- empty structured metadata values;
- implemented rules without at least one executable valid and invalid example;
- implemented rules missing `confidence` or `false_positive_risk`;
- implemented rules using `manual` detectors;
- missing declared capabilities implied by detector type, selected extended metadata, or conservative text markers;
- duplicate examples within `examples.valid`, within `examples.invalid`, or across the valid/invalid split;
- source metadata fields that are present but empty.

The schema deliberately does **not** validate linguistic truth. That belongs to future analyzers, benchmark suites, and curated corpora, not to YAML parsing.
