#!/usr/bin/env python3
"""Validate verb-government false-positive regression fixtures."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from scripts.morph.validate_verb_government_fixtures import VALID_COMPLEMENT_KINDS, Key, seed_keys

KNOWN_BLOCKERS = {"", "DirectSpeechBoundary", "ParenthesisBoundary", "UnsafeBoundary", "ClauseBoundary"}


def _data_rows(path: Path) -> list[tuple[int, list[str]]]:
    rows: list[tuple[int, list[str]]] = []
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        rows.append((line_number, raw_line.split("\t")))
    return rows


def validate(seed_path: Path, fixtures_path: Path) -> list[str]:
    errors: list[str] = []
    expected_keys, seed_errors = seed_keys(seed_path)
    errors.extend(seed_errors)
    seen_ids: set[str] = set()

    for line_number, columns in _data_rows(fixtures_path):
        if len(columns) != 8:
            errors.append(
                f"{fixtures_path}: line {line_number}: expected 8 columns, got {len(columns)}"
            )
            continue
        fixture_id, lemma, complement_kind, preposition, text, forbidden_excerpt, blocker, reason = [
            column.strip() for column in columns
        ]
        key: Key = (lemma.lower(), complement_kind, preposition.lower())
        if not fixture_id:
            errors.append(f"{fixtures_path}: line {line_number}: empty id")
        elif fixture_id in seen_ids:
            errors.append(f"{fixtures_path}: line {line_number}: duplicate id {fixture_id!r}")
        seen_ids.add(fixture_id)
        if complement_kind not in VALID_COMPLEMENT_KINDS:
            errors.append(f"{fixtures_path}: line {line_number}: unknown complement_kind {complement_kind!r}")
        if complement_kind == "direct_object" and preposition:
            errors.append(f"{fixtures_path}: line {line_number}: direct_object must not specify preposition")
        if complement_kind == "prepositional_object" and not preposition:
            errors.append(f"{fixtures_path}: line {line_number}: prepositional_object must specify preposition")
        if key not in expected_keys:
            errors.append(f"{fixtures_path}: line {line_number}: false-positive key has no seed row {key!r}")
        if not text:
            errors.append(f"{fixtures_path}: line {line_number}: empty text")
        if not forbidden_excerpt:
            errors.append(f"{fixtures_path}: line {line_number}: empty forbidden_excerpt")
        if forbidden_excerpt and forbidden_excerpt not in text:
            errors.append(f"{fixtures_path}: line {line_number}: forbidden_excerpt is not inside text")
        if blocker not in KNOWN_BLOCKERS:
            errors.append(f"{fixtures_path}: line {line_number}: unknown expected_blocker {blocker!r}")
        if not reason:
            errors.append(f"{fixtures_path}: line {line_number}: empty reason")
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("seed", nargs="?", default="data/grammar/verb_government.seed.tsv", type=Path)
    parser.add_argument(
        "fixtures",
        nargs="?",
        default="data/grammar/verb_government.false_positive.tsv",
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
