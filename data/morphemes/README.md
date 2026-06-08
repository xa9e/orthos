# Russian morpheme seed inventory

This directory is for project-authored, redistributable morpheme inventories.
The current Rust prototype keeps the executable seed inventory in
`src/morph/derivation/seed_inventory.rs` so the checker has zero runtime data
loading path for this layer. The TSV file here mirrors the intended data shape
for future generation/import tooling.

Columns:

- `kind`: prefix, root, derivational_suffix, ending, interfix, postfix.
- `form`: morpheme spelling in lowercase Russian.
- `tags`: `|`-separated semantic, grammatical or derivational hints.
- `productivity`: closed, limited, productive, highly_productive.
- `note`: short project-authored note, not copied from textbooks.

Licensing discipline: do not bulk-copy proprietary morphemic dictionaries here.
Use this as a curated seed and add external dictionary importers separately.
