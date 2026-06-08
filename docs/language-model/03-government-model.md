# Government model

The model adds the first executable **government** layer. It is intentionally small, but it changes the architectural direction: case-government rules should not remain a pile of phrase substitutions.

## Linguistic basis

Russian syntactic descriptions traditionally separate subordinate links into agreement, government, and adjunction. Government is the relation where the head licenses the dependent form, often by requiring a particular case. The project models this as a typed constraint, not as a string pair.

For prepositions, this maps cleanly to a relation:

```text
preposition -> nominal dependent -> required case set
```

The original seed covered only simple one-word preposition + one-word nominal windows. adds a narrow `preposition + modifier + nominal head` candidate; variant government and longer multiword nominal groups are still deliberately excluded until the parser can represent them.

## Runtime layers

### `PrepositionGovernmentRegistry`

Stores a normalized preposition and its allowed cases. adds a curated seed registry for frequent single-case governors:

- genitive: `без`, `для`, `до`, `из`, `от`, `у`, `около`, `вокруг`, `после`, `против`;
- dative: `к`, `согласно`, `благодаря`, `вопреки`;
- prepositional: `о`, `об`, `при`;
- instrumental: `между`, `над`, `под`.

Ambiguous prepositions such as `в`, `на`, `с`, `за`, and broad `по` are intentionally not in the first executable seed. They need semantic/syntactic disambiguation, so adding them now would create noisy diagnostics.

### `FeatureConstraintSet`

A reusable constraint container for grammatical features. It starts with case constraints and reserves room for number and gender constraints. The point is to let future rules share the same typed representation instead of hand-rolling one more `Option<Case>` comparison.

### `GovernmentCheck`

The government checker returns a structured result:

- relation kind;
- compatibility;
- expected case set;
- observed case set;
- unknown reason if the diagnostic must be suppressed.

This mirrors the agreement layer and prepares the future `DiagnosticProof` API.

### `preposition_government_candidates`

The syntax layer now emits a `SyntaxRelationKind::PrepositionGovernment` candidate for adjacent preposition + word pairs. The detector consumes this relation candidate instead of scanning raw tokens directly.

## Executable rule

Implemented rule:

```text
ru.grammar.preposition_government_basic
```

Examples:

```text
Согласно приказа, встреча перенесена. -> diagnostic
Согласно приказу, встреча перенесена. -> no diagnostic
К дом пришёл курьер. -> diagnostic
К дому пришёл курьер. -> no diagnostic
```

The detector is conservative:

- unknown preposition -> suppress;
- no nominal analysis -> suppress;
- ambiguous nominal with at least one compatible case -> suppress;
- missing case feature -> suppress;
- all known cases incompatible -> diagnostic.

## Why this matters

The old phrase seed is still useful as a baseline, but it cannot scale. `согласно приказа` and `к дом` are not the same surface phrase problem; they are instances of a more general government mechanism. The new layer gives future rules a place to grow without becoming regex soup.

## Next steps

1. Promote the the current model short nominal-group seed into a typed `NominalGroupCandidate` with modifier nodes.
2. Represent variant government as context-conditioned alternatives.
3. Add lexical governors beyond prepositions: verbs and deverbal nouns.
4. Add explanations: expected dative, observed genitive, source rule.
5. Feed UD/SynTagRus-shaped fixtures into relation extraction tests.
