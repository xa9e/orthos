#!/usr/bin/env python3
"""Validate data/grammar/verb_government.seed.tsv.

The Rust model intentionally loads this file as a small built-in seed. Keep this
validator cheap and dependency-free so environments without a Rust toolchain can still
check the data contract.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

VALID_COMPLEMENT_KINDS = {"direct_object", "prepositional_object"}
VALID_CASES = {
    "nom",
    "nomn",
    "nominative",
    "gen",
    "gent",
    "genitive",
    "dat",
    "datv",
    "dative",
    "acc",
    "accs",
    "accusative",
    "ins",
    "ablt",
    "inst",
    "instrumental",
    "prep",
    "loct",
    "prepositional",
    "loc",
    "loc2",
    "locative",
    "part",
    "gen2",
    "partitive",
    "voct",
    "voc",
    "vocative",
}


def validate(path: Path) -> list[str]:
    errors: list[str] = []
    seen: set[tuple[str, str, str]] = set()
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        columns = raw_line.split("\t")
        if len(columns) != 6:
            errors.append(f"line {line_number}: expected 6 columns, got {len(columns)}")
            continue
        lemma, complement_kind, preposition, cases, source_id, note = [c.strip() for c in columns]
        if not lemma:
            errors.append(f"line {line_number}: empty lemma")
        if complement_kind not in VALID_COMPLEMENT_KINDS:
            errors.append(f"line {line_number}: unknown complement_kind {complement_kind!r}")
        if complement_kind == "direct_object" and preposition:
            errors.append(f"line {line_number}: direct_object must not specify preposition")
        if complement_kind == "prepositional_object" and not preposition:
            errors.append(f"line {line_number}: prepositional_object must specify preposition")
        case_values = [value.strip() for value in cases.split("|") if value.strip()]
        if not case_values:
            errors.append(f"line {line_number}: empty cases")
        for case in case_values:
            if case not in VALID_CASES:
                errors.append(f"line {line_number}: unknown case {case!r}")
        if not source_id:
            errors.append(f"line {line_number}: empty source_id")
        if not note:
            errors.append(f"line {line_number}: empty note")
        key = (lemma.lower(), complement_kind, preposition.lower())
        if key in seen:
            errors.append(f"line {line_number}: duplicate lemma/complement/preposition {key!r}")
        seen.add(key)
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "path",
        nargs="?",
        default="data/grammar/verb_government.seed.tsv",
        type=Path,
    )
    args = parser.parse_args(argv)
    errors = validate(args.path)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(f"ok: {args.path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
