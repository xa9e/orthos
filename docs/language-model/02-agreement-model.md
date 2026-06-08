# Agreement Model

The project now has the first explicit layer for grammatical agreement diagnostics. It is intentionally small, but it moves the engine away from opaque boolean helpers and toward inspectable linguistic constraints.

## Why this layer exists

Russian agreement is not a string pattern. A useful checker needs typed facts:

- case;
- number;
- gender;
- person;
- tense;
- syntactic relation type;
- ambiguity and provenance.

The current implementation exposes agreement checks as structured values rather than only `true` or `false`.

## Current public concepts

- `AgreementRelationKind`
  - `AdjectiveNoun`
  - `SubjectPredicate`
- `AgreementFeatureKind`
  - `Case`
  - `Number`
  - `Gender`
  - `Person`
- `AgreementConflict`
  - feature name;
  - left value;
  - right value.
- `AgreementCheck`
  - relation kind;
  - compatibility;
  - conflicts;
  - unknown features.

This is the seed of a future diagnostic proof object.

## Implemented seed detector

`ru.grammar.subject_predicate_agreement_basic` now uses the new agreement layer through conservative `syntax/relations` candidates.

It can catch a tiny safe subset:

- `Девочка пришёл.`
- `Дети пришёл.`

It accepts:

- `Девочка пришла.`
- `Дети пришли.`

## Deliberate limitations

The detector currently ignores:

- inverted word order;
- coordinated subjects;
- long-distance dependencies;
- punctuation-separated clauses;
- quantitative groups;
- ambiguous analyses;
- unknown tokens.

That is not weakness; that is damage control. A grammar checker that confidently hallucinates syntax is worse than no grammar checker.

## Next step

The first `syntax/relations/` seed now exists. The next architectural move is to make it less toy-sized:

- produce relation candidates inside sentence and clause boundaries;
- annotate blockers such as punctuation, coordination, and ambiguity;
- feed agreement checks from candidate relations instead of raw neighboring words.

Once relation candidates exist, adjective-noun, subject-predicate, participle agreement, and preposition government can share the same infrastructure.
