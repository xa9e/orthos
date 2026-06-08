# Nominal-group candidate model

The model promotes the the current model `governor + one modifier + head` bridge into a
small typed nominal-group layer. The goal is still not full Russian parsing; the
point is to stop making grammar detectors rediscover the same local group shape
with private token-window hacks.

## Why this layer exists

Earlier morphology-backed grammar rules worked mostly on adjacent pairs:

```text
preposition -> nominal
numeral -> noun
adjective -> noun
```

That misses ordinary Russian groups where one or more modifiers intervene:

```text
согласно новому важному приказу
два новых больших дома
новый важный приказ
```

The head of the group is the word that satisfies government or quantity
constraints. Modifiers are separate dependents that need agreement constraints.
Treating all of this as pairwise adjacency is the road to regex-spaghetti with a
morphology sticker slapped on top.

## New syntax primitives

```text
NominalGroupCandidate {
    start_token,
    end_token,
    modifiers,
    head,
    span,
    confidence,
    blockers,
}

GovernedNominalGroupCandidate {
    kind,
    governor,
    group,
    span,
    confidence,
    blockers,
}
```

Public extractors:

```text
short_nominal_group_candidates(tokens, max_modifiers)
preposition_governed_nominal_group_candidates(tokens, prepositions, max_modifiers)
numeral_governed_nominal_group_candidates(tokens, max_modifiers)
```

The legacy relation API remains available:

```text
preposition_nominal_group_candidates(...)
numeral_nominal_group_candidates(...)
```

Those functions now consume the typed group candidates and expose the old
`left/right` relation shape for detectors that have not migrated yet.

## Implemented seed rules

```text
ru.grammar.preposition_nominal_group_government_basic
ru.grammar.numeral_nominal_group_agreement_basic
ru.grammar.nominal_group_modifier_agreement_basic
```

Covered examples:

```text
valid:   Согласно новому важному приказу, встреча перенесена.
invalid: Согласно новому важному приказа, встреча перенесена.

valid:   Два новых больших дома стояли рядом.
invalid: Два новых больших дом стояли рядом.

valid:   Новому важному приказу присвоили номер.
invalid: Новый важному приказу присвоили номер.
```

## Conservative limits

The model is intentionally small:

- only contiguous word groups are accepted;
- newline-separated groups are blocked;
- punctuation-separated groups are blocked by construction;
- group length is capped by the detector call site, currently up to three modifiers;
- morphology still decides whether a candidate is actually usable;
- ambiguous analyses suppress diagnostics;
- coordination, apposition, detached modifiers and participial clauses are not parsed.

The important design shift is the API. New rules can now ask for a *group with
modifiers and head*, not just a nearest token.

## Next step

Move from structural group candidates to typed slots:

```text
NominalGroupCandidate {
    modifiers: Vec<ModifierSlot>,
    head: HeadSlot,
    agreement_requirements,
    governor_requirements,
    blockers,
}
```

That lets the engine distinguish adjective, participle, pronoun-determiner,
ordinal numeral and substantivized-adjective groups without each detector
reimplementing the same morphology filters.

## Bridge to quantity phrases

The nominal-group model now also serves `QuantifiedNominalGroupCandidate`: a compound numeral phrase may govern either a head-only group (`двадцать пять домов`) or a short group with modifiers (`двадцать два новых дома`). This required allowing the internal group builder to construct zero-modifier head groups for syntax consumers that need a counted noun without an adjective slot.

The crucial design point remains unchanged: nominal grouping is structural, while morphology decides whether a candidate is safe enough for diagnostics.
