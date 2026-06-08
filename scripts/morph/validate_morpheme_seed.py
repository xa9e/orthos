#!/usr/bin/env python3
"""Validate the project-authored Russian morpheme seed TSV."""

from __future__ import annotations

import argparse
import csv
from pathlib import Path

ALLOWED_KINDS = {
    "prefix",
    "root",
    "derivational_suffix",
    "inflectional_suffix",
    "ending",
    "interfix",
    "postfix",
}

ALLOWED_PRODUCTIVITY = {"closed", "limited", "productive", "highly_productive"}
REQUIRED_COLUMNS = ["kind", "form", "tags", "productivity", "note"]


def validate(path: Path) -> list[str]:
    errors: list[str] = []
    seen: set[tuple[str, str]] = set()

    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if reader.fieldnames != REQUIRED_COLUMNS:
            return [f"expected columns {REQUIRED_COLUMNS}, got {reader.fieldnames}"]

        for line_no, row in enumerate(reader, start=2):
            kind = (row.get("kind") or "").strip()
            form = (row.get("form") or "").strip()
            tags = (row.get("tags") or "").strip()
            productivity = (row.get("productivity") or "").strip()
            note = (row.get("note") or "").strip()

            if kind not in ALLOWED_KINDS:
                errors.append(f"line {line_no}: unknown kind {kind!r}")
            if kind != "ending" and not form:
                errors.append(f"line {line_no}: non-ending morpheme must have a form")
            if not tags:
                errors.append(f"line {line_no}: tags must not be empty")
            if productivity not in ALLOWED_PRODUCTIVITY:
                errors.append(f"line {line_no}: unknown productivity {productivity!r}")
            if not note:
                errors.append(f"line {line_no}: note must not be empty")

            key = (kind, form)
            if key in seen:
                errors.append(f"line {line_no}: duplicate morpheme {kind}:{form}")
            seen.add(key)

    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "path",
        nargs="?",
        type=Path,
        default=Path("data/morphemes/ru_derivational_morphemes.seed.tsv"),
    )
    args = parser.parse_args()

    errors = validate(args.path)
    for error in errors:
        print(error)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
