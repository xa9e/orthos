#!/usr/bin/env python3
"""Import RuSpellGold into the common evaluation JSONL schema."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import clean_text, find_member, iter_archive_members, make_record, write_jsonl  # noqa: E402

DATASET = "ruspellgold"
DOMAIN_SUFFIXES = (
    "data/aranea/split.json",
    "data/literature/split.json",
    "data/news/split.json",
    "data/social_media/split.json",
    "data/strategic_documents/split.json",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path, help="Path to RuSpellGold.zip or unpacked directory")
    parser.add_argument("--output", required=True, type=Path, help="Output normalized JSONL path")
    parser.add_argument(
        "--split",
        choices=("complete", "domains"),
        default="complete",
        help="Use complete_test/test.json or concatenate domain shards without the complete duplicate.",
    )
    parser.add_argument(
        "--keep-identical",
        action="store_true",
        help="Keep rows where source and correction are identical.",
    )
    return parser.parse_args()


def decode_jsonl(data: bytes, source_file: str) -> Iterable[dict[str, Any]]:
    for line_no, line in enumerate(data.decode("utf-8-sig").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            yield json.loads(line)
        except json.JSONDecodeError as exc:
            raise ValueError(f"{source_file}:{line_no}: invalid JSON: {exc}") from exc


def selected_members(input_path: Path, split: str) -> Iterable[tuple[str, bytes]]:
    if split == "complete":
        yield find_member(input_path, "data/complete_test/test.json")
        return
    wanted = set(DOMAIN_SUFFIXES)
    for suffix in DOMAIN_SUFFIXES:
        yield find_member(input_path, suffix)
    # Defensive duplicate check if archive layout changes.
    seen = set()
    for name, _ in iter_archive_members(input_path):
        for suffix in wanted:
            if name.endswith(suffix):
                seen.add(suffix)
    missing = wanted - seen
    if missing:
        raise FileNotFoundError(f"missing RuSpellGold shards: {sorted(missing)}")


def iter_records(input_path: Path, *, split: str, keep_identical: bool) -> Iterable[dict[str, Any]]:
    global_index = 0
    for source_file, data in selected_members(input_path, split):
        for local_index, row in enumerate(decode_jsonl(data, source_file), start=1):
            input_text = clean_text(row.get("source"))
            correction = clean_text(row.get("correction"))
            if not input_text:
                continue
            if not keep_identical and correction and correction == input_text:
                continue
            global_index += 1
            domain = clean_text(row.get("domain"))
            yield make_record(
                record_id=f"{DATASET}/{global_index}",
                dataset=DATASET,
                source_file=source_file,
                input_text=input_text,
                correction=correction,
                targets=[correction] if correction else [],
                edits=[],
                labels=["spelling"],
                metadata={"domain": domain, "source_row": local_index, "split": split},
            )


def main() -> int:
    args = parse_args()
    count = write_jsonl(iter_records(args.input, split=args.split, keep_identical=args.keep_identical), args.output)
    print(f"wrote {count} {DATASET} records to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
