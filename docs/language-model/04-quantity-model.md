# Quantity model: numeral → noun

The model adds the first explicit quantity relation layer. The goal is not to
hard-code `два дома` as a phrase. The goal is to represent the relation:

```text
quantifier/numeral -> counted noun -> expected case + number
```

## Why this belongs in the language model

Russian cardinal numerals are not ordinary adjective modifiers. In nominative
contexts the classes behave differently:

- `один` behaves like an agreeing modifier;
- `два`, `три`, `четыре` govern genitive singular;
- `пять` and higher cardinal numerals govern genitive plural;
- collectives, fractions, ranges and oblique cases require separate rules.

The existing `numeral_government_class` helper already encoded part of this. The
new `QuantityAgreementCheck` makes it available as a structured diagnostic
primitive instead of a bare compatibility value.

## New primitives

```text
QuantityRelationKind::NumeralNoun
QuantityConflict
QuantityAgreementCheck
numeral_noun_agreement_check(...)
```

The check returns one of three outcomes:

- compatible: at least one confident numeral/noun analysis fits;
- incompatible: all known analyses conflict and the conflict can be explained;
- unknown: morphology is missing, unsupported or ambiguous.

## Syntax relation candidate

`syntax/relations` now has:

```text
SyntaxRelationKind::NumeralNoun
adjacent_numeral_noun_candidates(...)
```

This is deliberately shallow. It creates only adjacent word candidates and lets
the morphology layer decide whether they are actually numeral/noun pairs. This
keeps the detector conservative while moving the project away from raw token
window hacks.

## Implemented seed rule

```text
ru.grammar.numeral_noun_agreement
```

Covered examples:

```text
valid:   два дома, пять домов
invalid: два дом, пять дома
```

## Known limitations

The original seed detector intentionally ignored intervening modifiers. handles one-modifier groups such as `два новых дома`, but still intentionally ignores:

- составные числительные: `двадцать два дома`;
- дробные числительные;
- ranges: `2–4 дома`;
- longer groups with multiple intervening modifiers;
- composed numerals with intervening modifiers;
- oblique-case quantity groups;
- ambiguous forms where one analysis may still be valid.

Those are not bugs in this layer. They are future relation-graph and nominal-group
work.
