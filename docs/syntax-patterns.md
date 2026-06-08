# Syntax-aware punctuation pattern layer

`orthos` still avoids a full parser in the MVP. The punctuation layer therefore uses **conservative token patterns**: it can flag clear local omissions, but it must suppress diagnostics when the local context is ambiguous.

## Goals

- keep punctuation detectors deterministic and explainable;
- avoid one-off regex implementations for clause-level punctuation;
- expose reusable primitives for future morphology/syntax integration;
- prefer silence over noisy advice in quotes, parentheses, headings, ellipsis, direct speech, and unclear clause boundaries.

## Core primitives

Implemented in `src/syntax.rs`:

- `sentence_spans` and `sentence_span_at` — lightweight sentence segmentation over terminal punctuation and newlines;
- `token_window` — previous/current/next non-space view for local rules;
- `previous_non_space_token`, `next_non_space_token`, `previous_word_token`, `next_word_token` — shared token navigation;
- `punctuation_context_before` and `has_comma_before_marker` — local punctuation state before a candidate marker;
- `is_sentence_initial` — detects sentence-initial markers, including markers after an opening quote/bracket;
- `is_inside_quotes_or_parentheses` — suppresses diagnostics inside `«...»`, curly quotes, `(...)`, `[...]`, `{...}`;
- `find_clause_marker` — recognizes configured subordinator markers and treats `потому что` as a single marker;
- `is_safe_boundary_before_clause_marker` — common suppression gate for comma-before-subordinator checks.

## Current implemented use

`ru.punctuation.comma_before_subordinator_basic` now runs through the pattern layer instead of directly scanning raw token neighbors.

It flags clearer cases:

- `Я знаю что он придёт.` → likely missing comma before `что`;
- `Я уйду если начнётся дождь.` → likely missing comma before `если`;
- `Он ушёл потому что устал.` → likely missing comma before the multiword marker `потому что`.

It suppresses common traps:

- sentence-initial markers: `Если он придёт, мы уйдём.`;
- already punctuated markers: `Я знаю, что он придёт.`;
- multiword marker already punctuated: `Он ушёл, потому что устал.`;
- quoted fragments: `Он спросил: «что делать дальше?»`;
- parenthesized fragments: `Пометка (если возможно) останется.`;
- discourse/coordinating context before `что`: `А что он сказал?`.

## Why the rule remains heuristic

The detector still does not know whether a word is a predicate, pronoun, complementizer, particle, or part of a frozen expression. Examples that need future syntax/morphology hooks:

- distinguishing complementizer `что` from interrogative/relative pronoun uses;
- deciding whether `потому, что` is a split correlative construction or a punctuation error;
- detecting the right edge of a sentence-initial subordinate clause;
- recognizing homogeneous predicates and shared dependent elements;
- handling direct speech with author words and nested quotes.

## Design rule for future detectors

New punctuation detectors should follow this shape:

1. tokenize once;
2. find a narrow candidate marker or punctuation position;
3. query shared pattern primitives for sentence, punctuation, quote/parenthesis, and safe-boundary context;
4. require a positive local signal;
5. suppress if the signal is not strong enough.

The detector should not copy raw scanning loops if an equivalent primitive already exists in `src/syntax.rs`.

## Planned extensions

- delimiter-span iterator for direct speech and nested quote styles;
- predicate-light hooks backed by morphology tags;
- clause-boundary candidates with confidence levels;
- profile-based severity controls for editorial, school, and informal text;
- benchmark-driven false-positive tracking before enabling advanced punctuation rules by default.
