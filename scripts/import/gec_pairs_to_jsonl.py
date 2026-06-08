#!/usr/bin/env python3
"""Normalize simple sentence-pair files to the common evaluation JSONL schema.

Input formats supported without dependencies:
- TSV: source<TAB>target
- CSV-ish files when delimiter and column indexes are configured

This generic importer is intentionally tiny. Dataset-specific importers should be
used whenever source metadata, labels, or edits exist.
"""

from __future__ import annotations

import argparse
import csv
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import clean_sentence, make_record, write_jsonl  # noqa: E402


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--dataset", required=True)
    parser.add_argument("--delimiter", default="\t")
    parser.add_argument("--source-col", type=int, default=0)
    parser.add_argument("--target-col", type=int, default=1)
    parser.add_argument("--skip-header", action="store_true")
    parser.add_argument("--label", action="append", default=[], help="Optional label to attach to every row; repeatable.")
    return parser.parse_args()


def validate_args(args: argparse.Namespace) -> None:
    if len(args.delimiter) != 1:
        raise ValueError(f"--delimiter must be exactly one character, got {args.delimiter!r}")
    if args.source_col < 0 or args.target_col < 0:
        raise ValueError("--source-col and --target-col must be non-negative")


def iter_records(args: argparse.Namespace):
    validate_args(args)
    required_columns = max(args.source_col, args.target_col) + 1
    with args.input.open("r", encoding="utf-8-sig", newline="") as src:
        reader = csv.reader(src, delimiter=args.delimiter)
        start_line = 1
        if args.skip_header:
            next(reader, None)
            start_line = 2
        for index, row in enumerate(reader, start=start_line):
            if not any(cell.strip() for cell in row):
                continue
            if len(row) < required_columns:
                raise ValueError(
                    f"{args.input}:{index}: expected at least {required_columns} columns, got {len(row)}"
                )
            input_text = clean_sentence(row[args.source_col])
            target = clean_sentence(row[args.target_col])
            if not input_text.strip() or not target.strip():
                continue
            yield make_record(
                record_id=f"{args.dataset}/{index}",
                dataset=args.dataset,
                source_file=args.input.as_posix(),
                input_text=input_text,
                correction=target,
                targets=[target],
                edits=[],
                labels=args.label,
                metadata={"source_row": index},
            )


def main() -> int:
    args = parse_args()
    count = write_jsonl(iter_records(args), args.output)
    print(f"wrote {count} {args.dataset} records to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
