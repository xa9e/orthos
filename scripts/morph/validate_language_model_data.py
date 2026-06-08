#!/usr/bin/env python3
"""Validate all checked-in language-model seed data that has a Python contract."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.morph.validate_verb_government_false_positives import validate as validate_verb_false_positives
from scripts.morph.validate_verb_government_fixtures import validate as validate_verb_fixtures
from scripts.morph.validate_verb_government_seed import validate as validate_verb_seed


def validate_all(root: Path) -> list[str]:
    seed = root / "data/grammar/verb_government.seed.tsv"
    fixtures = root / "data/grammar/verb_government.fixtures.tsv"
    false_positives = root / "data/grammar/verb_government.false_positive.tsv"
    errors: list[str] = []
    errors.extend(validate_verb_seed(seed))
    errors.extend(validate_verb_fixtures(seed, fixtures))
    errors.extend(validate_verb_false_positives(seed, false_positives))
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", nargs="?", default=Path("."), type=Path)
    args = parser.parse_args(argv)
    errors = validate_all(args.root)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    print(f"ok: language model data under {args.root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
