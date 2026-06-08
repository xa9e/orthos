## Evidence model

`evidence` records why a rule exists or why its planned implementation is justified.

Allowed `evidence.kind` values:

- `normative_source` — spelling/punctuation/grammar reference;
- `corpus_attestation` — real examples or distributional evidence;
- `donor_taxonomy` — external label taxonomy used only as a design signal;
- `benchmark` — measured behavior in an evaluation set;
- `lexicon` — dictionary/lexical source;
- `morphology` — analyzer or morphological resource dependency;
- `syntax` — syntax treebank/parser resource dependency;
- `stress_dictionary` — stress source;
- `expert_review` — manually reviewed linguistic decision.

Each evidence item should include `source_ref` when it points to a corpus source. Freeform `note` is allowed for non-source evidence.

## Link model

`related_rules` and `supersedes` are rule ids.

- `related_rules` means the rules overlap, share a concept, or should be reviewed together.
- `supersedes` means the current rule replaces an older or narrower model.

Validation rejects duplicates, invalid id shapes, unknown ids, and self-links.
