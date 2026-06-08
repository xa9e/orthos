# Project-authored morpheme seed data

The word-formation layer has a tiny redistributable seed inventory at:

```text
data/morphemes/ru_derivational_morphemes.seed.tsv
```

This is **not** a comprehensive morphemic dictionary. It is a reviewed project
seed used to shape the data model and catch regressions before larger licensed
resources are introduced.

Validate it with:

```bash
python scripts/morph/validate_morpheme_seed.py
```

The executable Rust seed currently lives in:

```text
src/morph/derivation/seed_inventory.rs
```

It uses a macro DSL. The TSV is the intended import/codegen shape for future
work, so the repository can grow toward generated static inventories without
requiring runtime data loading for every checker run.
