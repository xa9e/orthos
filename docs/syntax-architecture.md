# Syntax architecture for deterministic punctuation

`orthos` uses a deliberately shallow syntax layer. It is not a parser and it must not pretend to recover the full grammar of a Russian sentence. Its job is narrower: expose reusable, deterministic facts that let punctuation and grammar heuristics avoid obvious false positives.

The implementation lives in `src/syntax.rs` and is designed around one rule: **miss hard punctuation cases rather than emit noisy local advice**.

## Scope

The syntax layer is optimized for conservative rule-based checks:

- document-level caching of tokenization, sentence spans and safe zones;
- local punctuation decisions;
- shallow clause-marker recognition;
- quote and parenthesis suppression;
- future dependency-like hooks for grammar rules;
- explainable confidence levels.

It explicitly does **not** introduce ML parsers, statistical taggers, external GEC datasets, or aggressive comma insertion.

## Document-level API

`SyntaxDocument` is the reusable entry point for syntax-aware detectors:

```rust
let marker_set = HashSet::from(["что".to_string(), "потому что".to_string()]);
let document = SyntaxDocument::with_clause_markers(text, &marker_set);
```

It caches:

- original `text`;
- `Token` stream from `text::tokenize`;
- `SentenceSyntax` values;
- balanced `ParentheticalSpan` values;
- balanced `DirectSpeechSpan` values;
- `PunctuationSafeZone` values derived from parentheticals and direct speech.

`SyntaxDocument::new(text)` builds the same document without clause-marker analysis. Detectors that need clause boundaries should use `SyntaxDocument::with_clause_markers` so tokenization, sentence segmentation and safe-zone scans stay centralized.

Useful accessors:

- `tokens()` — cached token stream;
- `sentences()` — sentence-level syntax records;
- `safe_zones()` — typed suppression zones;
- `sentence_at(byte_index)` — sentence lookup by byte offset;
- `clause_boundaries()` — flattened iterator over shallow clause-boundary candidates;
- `is_inside_punctuation_safe_zone(byte_index)` — document-local suppression predicate.

This avoids the previous bad shape where each detector could tokenize and rescan delimiters independently.

## Sentence model

`SentenceSyntax` stores:

- `SentenceSpan` byte range;
- optional first and last token indexes;
- `ClauseGraph`;
- `SyntaxConfidence`.

`ClauseGraph` is intentionally shallow. It contains:

- `boundaries: Vec<ClauseBoundary>`;
- `edges: Vec<DependencyEdge>`;
- `fragments: Vec<SyntaxSpan>`.

This is **not** an AST. It is a small deterministic graph of facts that a detector can use without pretending that the sentence has been fully parsed.

## Sentence segmentation policy

`sentence_spans` now handles common punctuation hazards more conservatively:

- leading whitespace is skipped;
- final fragments without terminal punctuation are retained;
- `...`, `?!`, `!!` and similar terminal clusters are treated as one sentence boundary;
- common abbreviations such as `г.`, `ул.`, `см.`, `т. е.`, `т. к.`, `т. п.`, `т. д.` do not split a sentence;
- decimal-like numeric periods do not split;
- single-letter initials before another uppercase token are treated as initials;
- balanced parenthetical and quote spans suppress internal sentence splitting unless the terminal punctuation closes the whole fragment and the following text looks like a new sentence.

Examples:

```text
В г. Алматы тихо... Потом начался дождь.
```

splits into two sentences, not four bogus fragments.

```text
Он оставил заметку (пример: «что делать?» внутри) и ушёл.
```

stays one sentence because the question mark belongs to a nested quote inside a parenthetical continuation.

```text
Он спросил: «Что делать?» Потом ушёл.
```

splits after the closing quote because the direct-speech fragment closes and the next token starts a new sentence.

Author-word continuations remain conservative:

```text
Он спросил: «Что делать?» — и замолчал.
```

The dash after the quote is treated as a continuation signal, so the sentence is not split at the question mark.

## Clause boundary model

`ClauseBoundary` represents a candidate boundary around a clause marker, usually immediately before a subordinator:

- `BeforeMarker` — actionable candidate for a missing comma before a strong marker such as `что`, `чтобы`, `если`, `поскольку`, `хотя`, or `потому что`;
- `SentenceStartMarker` — marker at the beginning of a sentence or after an opening delimiter; currently safe;
- `PunctuatedBeforeMarker` — marker already preceded by a comma, colon, semicolon, dash, or sentence terminal;
- `SuppressedSafeZone` — marker inside a parenthetical or direct-speech span;
- `Ambiguous` — local evidence is too weak.

The detector for `ru.punctuation.comma_before_subordinator_basic` reports only `BeforeMarker` cases that remain valid after `should_report_missing_comma_before_clause_marker` rechecks confidence and safe boundaries.

Weak relative markers such as `который` are recognized for future grammar work but are not actionable for comma insertion yet. That is intentional: local token-only matching around relative pronouns is a false-positive factory.

## Clause marker model

`ClauseMarker` stores:

- start and end token indexes;
- byte span;
- canonical marker text;
- `ClauseMarkerKind`;
- `SyntaxConfidence`.

Current marker kinds:

- `Subordinator`;
- `MultiwordSubordinator`;
- `RelativePronoun`;
- `Unknown`.

Multiword markers are matched as compact whitespace-only token sequences. `потому что` is treated as one marker; the trailing `что` is suppressed so the detector does not emit duplicate comma advice. The same generic machinery can handle future markers such as `так как` if a rule adds them, but punctuation between marker parts blocks the multiword match.

## Dependency edge model

`DependencyEdge` is a lightweight placeholder for deterministic grammar relations. It contains:

- `head_token`;
- `dependent_token`;
- `DependencyRelation`;
- `SyntaxConfidence`.

The current relation set is deliberately small:

- `MarkerIntroducesClause`;
- `QuoteContainsSpeech`;
- `ParentheticalContainsFragment`;
- `Unknown`.

This is not a dependency parse. It is a stable interface for future detectors that need to say “this token controls that span” without depending on a parser implementation.

## Constituency and fragment model

`SyntaxSpan` is the common shallow constituent representation. It can describe:

- a sentence;
- a clause fragment;
- a parenthetical fragment;
- a direct-speech fragment;
- a punctuation-safe zone.

Spans may optionally carry token indexes. Character-scanned zones such as quotes and parentheses often do not map cleanly to token indexes because punctuation can be grouped by the tokenizer; those spans keep `start_token` and `end_token` empty.

This split avoids forcing everything into a fake tree. The current model is a set of typed spans and candidate edges, not an AST.

## Parenthetical expression spans

`parenthetical_spans` extracts balanced spans for:

- `(...)`;
- `[...]`;
- `{...}`.

The delimiter scanner uses simple stack logic and refuses cross-nested matches. That means a malformed sequence is ignored rather than converted into a misleading “balanced” span.

Each `ParentheticalSpan` stores the full delimiter span and the inner span. Parentheticals are punctuation-safe zones because comma heuristics inside them frequently refer to fragmentary notes rather than the main sentence.

Example:

```text
Пометка (если возможно) останется.
```

A subordinator inside `(если возможно)` is suppressed.

## Direct speech spans

`direct_speech_spans` extracts balanced quote spans for common Russian typography:

- `«...»`;
- `„...“`;
- `“...”`.

The current model treats the whole quoted fragment as a safe zone. This is conservative. It avoids firing main-sentence comma heuristics inside direct speech or quoted questions:

```text
Он спросил: «что делать дальше?»
```

### Direct speech limitations

The layer does not yet decide whether punctuation belongs before or after author words, nor does it model all direct-speech layouts. In particular, these remain out of scope for now:

- author words before, after and inside direct speech;
- dialogue dash conventions;
- quote nesting with broken typography;
- comma replacement around quoted declaratives;
- punctuation transfer across quote boundaries.

The only direct-speech behavior this layer owns is safe-zone extraction plus conservative sentence-boundary suppression or splitting around closed quoted fragments.

## Punctuation-safe zones

`PunctuationSafeZone` is the typed suppression model. It records:

- full byte `span`;
- `inner_span` without delimiters;
- `PunctuationSafeZoneKind` (`Parenthetical` or `DirectSpeech`);
- confidence.

`punctuation_safe_zone_records` returns typed zones. `punctuation_safe_zones` remains a compatibility helper that converts those records into `SyntaxSpanKind::PunctuationSafeZone` spans.

`is_inside_punctuation_safe_zone` is the preferred suppression primitive. The older `is_inside_quotes_or_parentheses` remains as a compatibility wrapper.

A detector should suppress a diagnostic inside a safe zone unless it is explicitly designed to operate there. This avoids high-noise behavior in:

- quoted questions;
- fragmentary parentheticals;
- examples embedded in prose;
- nested editorial notes.

## Uncertainty and ambiguity policy

The syntax layer uses `SyntaxConfidence`:

- `Certain` — deterministic punctuation or balanced delimiter fact;
- `Strong` — safe local syntactic signal;
- `Weak` — plausible but too noisy for automatic advice;
- `Ambiguous` — recognized shape but insufficient evidence.

Current comma insertion requires `Certain` or `Strong`. `Weak` and `Ambiguous` are useful for future analysis and reporting, but they should not produce punctuation issues by default.

The policy is blunt by design:

> Missing a hard case is better than spraying bogus comma advice.

Russian punctuation has too many local traps for token-only rules to be aggressive.

## Detector integration pattern

New syntax-aware detectors should follow this sequence:

1. Build `SyntaxDocument` once.
2. Find a narrow candidate marker or punctuation location from cached syntax objects.
3. Check `SyntaxConfidence`.
4. Suppress inside `PunctuationSafeZone` unless the detector explicitly owns safe-zone diagnostics.
5. Emit only when the local signal is strong.

Avoid copying delimiter stacks or marker scans into individual detectors. Put reusable scans in `src/syntax.rs` and keep detectors thin.

`ru.punctuation.comma_before_subordinator_basic` now uses `SyntaxDocument::with_clause_markers`; `ru.punctuation.introductory_phrase_comma_basic` uses syntax-backed sentence starts and skips starts inside safe zones.

## Future non-ML parsing hook

A future deterministic dependency layer can plug in behind `ClauseGraph` without changing detector contracts:

- keep `SyntaxDocument` as the cache owner;
- add a new deterministic analyzer that consumes tokens, morphology and rule dictionaries;
- populate additional `DependencyEdge` relations and `SyntaxSpan` fragments;
- preserve `SyntaxConfidence` gates;
- keep ML/statistical parsers out unless the project explicitly changes scope.

A practical path is dictionary-driven shallow parsing: preposition government, finite-verb detection, conjunction inventories and punctuation-safe constituency. That gives useful structure without smuggling in opaque parser behavior.

## Current behavior changes

`ru.punctuation.comma_before_subordinator_basic` now delegates reporting policy to syntax helpers:

- detects strong missing comma candidates before `что`, `чтобы`, `если`, `поскольку`, `хотя`, and `потому что`;
- suppresses sentence-initial markers;
- suppresses markers inside parentheticals and direct speech;
- suppresses markers already preceded by punctuation;
- recognizes but does not report weak relative markers such as `который`;
- treats compact multiword markers as one marker, avoiding duplicate reports on the second word.

This is intentionally less ambitious than a human editor. It is safer, and safer is the right default for an early deterministic checker.
