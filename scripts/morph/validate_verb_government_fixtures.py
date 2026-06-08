#!/usr/bin/env python3
"""Validate verb-government regression fixtures against seed rows."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

VALID_COMPLEMENT_KINDS = {"direct_object", "prepositional_object"}

Key = tuple[str, str, str]


def _data_rows(path: Path) -> list[tuple[int, list[str]]]:
    rows: list[tuple[int, list[str]]] = []
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        rows.append((line_number, raw_line.split("\t")))
    return rows


def seed_keys(path: Path) -> tuple[set[Key], list[str]]:
    keys: set[Key] = set()
    errors: list[str] = []
    for line_number, columns in _data_rows(path):
        if len(columns) != 6:
            errors.append(f"{path}: line {line_number}: expected 6 columns, got {len(columns)}")
            continue
        lemma, complement_kind, preposition = [columns[index].strip() for index in range(3)]
        if complement_kind not in VALID_COMPLEMENT_KINDS:
            errors.append(f"{path}: line {line_number}: unknown complement_kind {complement_kind!r}")
            continue
        keys.add((lemma.lower(), complement_kind, preposition.lower()))
    return keys, errors


def validate(seed_path: Path, fixtures_path: Path) -> list[str]:
    errors: list[str] = []
    expected_keys, seed_errors = seed_keys(seed_path)
    errors.extend(seed_errors)
    seen: set[Key] = set()

    for line_number, columns in _data_rows(fixtures_path):
        if len(columns) != 6:
            errors.append(
                f"{fixtures_path}: line {line_number}: expected 6 columns, got {len(columns)}"
            )
            continue
        lemma, complement_kind, preposition, valid_text, invalid_text, invalid_excerpt = [
            column.strip() for column in columns
        ]
        key = (lemma.lower(), complement_kind, preposition.lower())
        if complement_kind not in VALID_COMPLEMENT_KINDS:
            errors.append(f"{fixtures_path}: line {line_number}: unknown complement_kind {complement_kind!r}")
        if complement_kind == "direct_object" and preposition:
            errors.append(f"{fixtures_path}: line {line_number}: direct_object must not specify preposition")
        if complement_kind == "prepositional_object" and not preposition:
            errors.append(f"{fixtures_path}: line {line_number}: prepositional_object must specify preposition")
        if key in seen:
            errors.append(f"{fixtures_path}: line {line_number}: duplicate fixture key {key!r}")
        seen.add(key)
        if key not in expected_keys:
            errors.append(f"{fixtures_path}: line {line_number}: fixture key has no seed row {key!r}")
        if not valid_text:
            errors.append(f"{fixtures_path}: line {line_number}: empty valid_text")
        if not invalid_text:
            errors.append(f"{fixtures_path}: line {line_number}: empty invalid_text")
        if not invalid_excerpt:
            errors.append(f"{fixtures_path}: line {line_number}: empty invalid_excerpt")
        if invalid_excerpt and invalid_excerpt not in invalid_text:
            errors.append(f"{fixtures_path}: line {line_number}: invalid_excerpt is not inside invalid_text")

    missing = sorted(expected_keys - seen)
    for key in missing:
        errors.append(f"{fixtures_path}: missing fixture for seed row {key!r}")
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("seed", nargs="?", default="data/grammar/verb_government.seed.tsv", type=Path)
    parser.add_argument(
        "fixtures",
        nargs="?",
        default="data/grammar/verb_government.fixtures.tsv",
        type=Path,
    )
    args = parser.parse_args(argv)
    errors = validate(args.seed, args.fixtures)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(f"ok: {args.fixtures}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
