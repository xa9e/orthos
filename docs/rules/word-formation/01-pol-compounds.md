# Пол- compounds: seed implementation

This document describes the first implemented `пол-` word-formation check.

## Scope

The implemented rule `ru.orthography.pol_hyphen_missing_basic` detects only the
low-risk spacing shape:

- token `пол`;
- horizontal whitespace, not a newline;
- a following word that starts with a vowel, `л`, or an uppercase letter.

The detector suggests replacing the whitespace with a hyphen: `пол лимона` ->
`пол-лимона`.

## Out of scope

The seed detector intentionally does not try to solve the full rule. It does
not correct joined misspellings such as `поляблока`, does not decide between
`пол-`, `полу-`, joined spellings like `полчаса`, and free phrases such as
`пол чайной ложки`.

Those cases require a real word-formation model, lexicon support, and stronger
false-positive tests. The broader planned rule is tracked as
`ru.orthography.pol_compounds`.
