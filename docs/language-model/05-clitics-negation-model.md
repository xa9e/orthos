# Clitic and negation model

This layer models two orthographic zones that look deceptively simple but become noisy if implemented as raw regular expressions.

## Clitic hyphenation

Russian has particles and pronominal components that attach orthographically with a hyphen:

- indefinite pronominal suffixes: `кто-то`, `что-либо`, `где-нибудь`;
- emphatic enclitic `-то` in a narrow seed set: `я-то`, `он-то`;
- imperative/colloquial `-ка` in a narrow seed set: `ну-ка`, `скажи-ка`.

The project represents this as `RussianCliticModel`, not as per-rule YAML lists. A detector asks the model whether `base + particle` belongs to a known hyphen group, then emits a replacement. New groups should be added only when their ambiguity is understood.

## Negation spacing

The rule `не + verb` is usually a free particle plus a verbal form, but Russian also has lexicalized words that do not exist without `не`: `ненавидеть`, `негодовать`, `несдобровать`.

The project therefore uses `split_negated_verb_candidate`:

1. Require a token beginning with `не`.
2. Do not flag lexicalized exceptions.
3. Do not flag if the whole token is already a known verbal lexeme.
4. Strip `не` and ask the morphological analyzer whether the remainder is a verb, participle, or gerund.
5. Emit `не <verb>` only when the remainder is known.

This is conservative by design. Unknown words are not guessed. The next milestone is to add lexicon-backed exception provenance and a larger OpenCorpora-derived test fixture.
