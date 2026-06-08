# Quantity phrase model

## Why this layer exists

The the current model quantity model introduced a morphology-backed check for adjacent
`numeral + noun` pairs. the current model and moved one-word numerals across short nominal
groups, and added the first phrase-level abstraction for constructions such
as:

```text
двадцать два дома
двадцать два новых дома
```

adds typed numeral components so the detector does not have to hard-code
`last token wins` privately. Russian cardinal compounds are not arbitrary word
chains: hundreds, decades, teens, terminal `one`, terminal `two/three/four`, and
`five+` components impose different constraints on the counted noun.

## Phrase and component candidates

```text
NumeralComponentSlot {
    token_index,
    token,
    component_class,
    span,
    confidence,
    blockers,
}

NumeralPhraseCandidate {
    tokens,
    components,
    span,
    confidence,
    blockers,
}

QuantifiedNominalGroupCandidate {
    numeral_phrase,
    group,
    span,
    confidence,
    blockers,
}
```

Current component classes:

```text
UnitOne
UnitPaucal
UnitMany
Teen
Decade
Hundred
Thousand
Collective
Ordinal
Unknown
```

The extractor is still shallow. It builds continuous two- or three-word numeral
phrases followed by a head-only or short nominal group:

```text
NUMR NUMR NOUN
NUMR NUMR ADJ NOUN
NUMR NUMR ADJ ADJ NOUN
NUMR NUMR NUMR NOUN
NUMR NUMR NUMR ADJ NOUN
```

The syntax layer gives each component a conservative surface class. It does not
pretend that surface classification is enough for diagnostics. Detectors still
require morphology to prove that every phrase token has an unambiguous `NUMR`
analysis.

## Governing component

`NumeralPhraseCandidate::governing_component()` returns the final component that
can select the nominal form:

```text
двадцать два дома
           ^ governs

сто двадцать два дома
               ^ governs

сто двадцать пять домов
               ^ governs
```

This matters because partial phrase candidates are dangerous. In
`сто двадцать два дома`, the candidate `[сто двадцать] + [два дома]` is
structurally possible if the engine only sees token windows. suppresses such
partial candidates when the next plain word is also an unambiguous numeral.

## Current executable rules

```text
ru.grammar.compound_numeral_nominal_group_agreement_basic
ru.grammar.typed_compound_numeral_nominal_group_agreement_basic
```

The first rule covers two-word compounds:

```text
valid:   Двадцать два новых дома стояли рядом.
invalid: Двадцать два новых дом стояли рядом.

valid:   Двадцать пять домов снесли.
invalid: Двадцать пять дома снесли.
```

The second rule covers the current three-component seed:

```text
valid:   Сто двадцать два новых дома стояли рядом.
invalid: Сто двадцать два новых дом стояли рядом.

valid:   Сто двадцать пять домов снесли.
invalid: Сто двадцать пять дома снесли.
```

## Limits

The seed intentionally does not handle:

- digit numerals: `22 дома`;
- long compounds beyond the configured window;
- fractional numerals: `две третьих`;
- ranges: `два-три дома`, `2–3 дома`;
- oblique-case quantity phrases;
- coordination and apposition inside the counted group;
- full numeric value normalization.

These are not bugs in the current model. They are blocked until the project has
numeric-token normalization and a richer quantity constraint model.

## Next step

Move from surface component slots to morphology-backed component typing:

```text
NumeralComponentSlot {
    token,
    surface_class,
    morph_class,
    numeric_contribution,
    terminal_selector,
    confidence,
}
```

That will let the engine handle digit tokens, mixed digit-word compounds,
collectives, fractions, ranges and ordinals without each detector reinventing
the same brittle logic.
