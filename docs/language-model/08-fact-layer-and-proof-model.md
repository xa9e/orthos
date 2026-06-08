# Fact layer and proof model

This document describes the current architectural direction for the Russian grammar engine. The goal is to make rules consume typed linguistic facts instead of rescanning raw tokens with ad-hoc windows.

## Pipeline

The intended analysis stack is:

1. tokenize text;
2. collect morphology and ambiguity facts;
3. build safe syntactic islands;
4. build local candidates: nominal groups, clauses, coordination groups, punctuation slots, government frames, agreement edges;
5. expose all facts through `LinguisticFactStore`;
6. let detectors consume facts and emit `Issue` values with `DiagnosticProof`.

The important boundary is between fact extraction and diagnostics. A detector should ask the fact store for a relation or slot and then decide whether the corresponding rule should emit an issue. It should not reimplement morphology, punctuation safety, or quote/parenthesis suppression.

## FeatureUnification

`FeatureUnification` is the shared agreement math layer. It treats morphology as sets of possible values and checks compatibility through intersections.

Example:

- modifier case set: `{Dative, Locative}`;
- head case set: `{Dative}`;
- intersection: `{Dative}`;
- status: `Compatible`.

Hard conflict:

- modifier number set: `{Plural}`;
- head number set: `{Singular}`;
- intersection: `∅`;
- status: `Conflict`.

Unknown is different from conflict. If morphology is missing or role ambiguity makes the relation unsafe, the unifier returns an unknown step, and a conservative detector should stay silent.

## PunctuationSlot

`PunctuationSlot` models a potential punctuation position between two word tokens. It stores:

- left and right token context;
- boundary strength;
- quote/parenthesis/direct-speech blockers;
- clause and introductory evidence;
- existing marks;
- expected marks;
- forbidden marks;
- structured `PunctuationSlotEvidence`.

This turns punctuation rules into checks over a slot object instead of “if left token X and right token Y”. The introductory-comma detector now consumes punctuation slots for single-token introductory candidates and emits a proof.

## CoordinationGroup

`CoordinationGroup` models coordinated and homogeneous groups such as:

- `умный, быстрый и надёжный анализатор`;
- `мама и папа пришли`;
- `ни рыба ни мясо`.

The seed model stores members, connectors, inferred kind, shared case candidates, and group-level agreement number. Compound nominal subjects currently expose plural agreement number, which is the hook for future subject-predicate agreement over a group instead of over one adjacent noun.

## DocumentContext

`DocumentContext` is the document-level seed. It stores:

- headings;
- list items;
- repeated terms;
- first/later mentions;
- abbreviations;
- parenthetical abbreviation expansions;
- glossary entries under glossary headings;
- style profile;
- cross-sentence facts.

This is intentionally conservative. It is not a full discourse parser. It is the first stable boundary that lets the engine reason about consistency, glossary/list style, repeated entities, and first mention vs later mention.

## DiagnosticProof

A proof should explain both emitted diagnostics and, later, suppressed alternatives. Current proof values carry:

- facts;
- assumptions;
- conflict;
- confidence;
- blockers;
- suppressed alternatives.

Rule code should prefer reusable proof builders such as `agreement_edge_proof`, `government_frame_proof`, and `punctuation_slot_proof`.

## Test doctrine

Every fact layer must have multi-fixture tests. A single hand-picked sentence is not enough. Tests should document:

- compatible extraction;
- conflict extraction;
- ambiguity or unknown morphology;
- boundary suppression;
- the public fields that rules are expected to consume.
