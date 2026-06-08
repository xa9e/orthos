#!/usr/bin/env python3
"""Import RLC-GEC / rlc-annotated into the common evaluation JSONL schema."""

from __future__ import annotations

import argparse
import csv
import io
import sys
from collections import OrderedDict, defaultdict
from pathlib import Path
from typing import Any, Iterable

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import clean_text, find_member, make_record, unique_preserve_order, write_jsonl  # noqa: E402

DATASET = "rlc-annotated"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path, help="Path to rlc-annotated-main.zip or unpacked directory")
    parser.add_argument("--output", required=True, type=Path, help="Output normalized JSONL path")
    parser.add_argument(
        "--part",
        choices=("sentences", "rlc-test", "all"),
        default="sentences",
        help="Dataset part to normalize. `all` appends grouped RLC-Test pairs after main sentences.",
    )
    parser.add_argument(
        "--keep-uncorrected",
        action="store_true",
        help="Keep rows whose original text equals the corrected text. Useful for clean-target FP checks.",
    )
    return parser.parse_args()


def read_csv_member(input_path: Path, suffix: str) -> tuple[str, list[dict[str, str]]]:
    source_file, data = find_member(input_path, suffix)
    text = data.decode("utf-8-sig")
    reader = csv.DictReader(io.StringIO(text))
    return source_file, list(reader)


def sentence_records(input_path: Path, *, keep_uncorrected: bool) -> Iterable[dict[str, Any]]:
    sentences_file, sentences = read_csv_member(input_path, "sentences.csv")
    _, documents = read_csv_member(input_path, "documents.csv")
    annotations_file, annotations = read_csv_member(input_path, "annotations.csv")

    document_by_id = {row["id"]: row for row in documents}
    edits_by_sentence: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in annotations:
        sentence_id = clean_text(row.get("sentence_id"))
        tag = clean_text(row.get("tag"))
        edit = {
            "id": clean_text(row.get("id")),
            "tag": tag,
            "quote": clean_text(row.get("quote")),
            "correction": clean_text(row.get("correction")),
            "start": clean_text(row.get("start")),
            "end": clean_text(row.get("end")),
            "annotation_source": clean_text(row.get("annotation_source")),
            "source_file": annotations_file,
        }
        edits_by_sentence[sentence_id].append(edit)

    for row in sentences:
        sentence_id = clean_text(row.get("id"))
        input_text = clean_text(row.get("text"))
        correction = clean_text(row.get("corrected"))
        if not input_text:
            continue
        if not keep_uncorrected and correction and correction == input_text:
            continue
        edits = edits_by_sentence.get(sentence_id, [])
        labels = unique_preserve_order(edit.get("tag") for edit in edits)
        document = document_by_id.get(clean_text(row.get("document_id")), {})
        metadata = {
            "document_id": clean_text(row.get("document_id")),
            "sentence_index": clean_text(row.get("sentence_index")),
            "status": clean_text(row.get("status")),
            "document": {
                "subcorpus": clean_text(document.get("subcorpus")),
                "native": clean_text(document.get("native")),
                "language_background": clean_text(document.get("language_background")),
                "level": clean_text(document.get("level")),
                "words": clean_text(document.get("words")),
                "sentences": clean_text(document.get("sentences")),
            },
        }
        yield make_record(
            record_id=f"{DATASET}/sentence-{sentence_id}",
            dataset=DATASET,
            source_file=sentences_file,
            input_text=input_text,
            correction=correction,
            targets=[correction] if correction else [],
            edits=edits,
            labels=labels,
            metadata=metadata,
        )


def rlc_test_records(input_path: Path) -> Iterable[dict[str, Any]]:
    source_file, rows = read_csv_member(input_path, "rlc_test.csv")
    grouped: OrderedDict[tuple[str, str], list[dict[str, str]]] = OrderedDict()
    for row in rows:
        key = (clean_text(row.get("text_orig")), clean_text(row.get("text_cor")))
        if not key[0]:
            continue
        grouped.setdefault(key, []).append(row)

    for index, ((input_text, correction), edits_raw) in enumerate(grouped.items(), start=1):
        edits = []
        for edit_index, row in enumerate(edits_raw, start=1):
            edits.append(
                {
                    "id": f"rlc-test-{index}-edit-{edit_index}",
                    "tag": clean_text(row.get("tag")),
                    "quote": clean_text(row.get("quote")),
                    "correction": clean_text(row.get("correction")),
                    "source_file": source_file,
                }
            )
        labels = unique_preserve_order(edit.get("tag") for edit in edits)
        yield make_record(
            record_id=f"{DATASET}/rlc-test-{index}",
            dataset=DATASET,
            source_file=source_file,
            input_text=input_text,
            correction=correction,
            targets=[correction] if correction else [],
            edits=edits,
            labels=labels,
            metadata={"part": "rlc-test", "edit_count": len(edits)},
        )


def iter_records(input_path: Path, *, part: str, keep_uncorrected: bool) -> Iterable[dict[str, Any]]:
    if part in {"sentences", "all"}:
        yield from sentence_records(input_path, keep_uncorrected=keep_uncorrected)
    if part in {"rlc-test", "all"}:
        yield from rlc_test_records(input_path)


def main() -> int:
    args = parse_args()
    count = write_jsonl(iter_records(args.input, part=args.part, keep_uncorrected=args.keep_uncorrected), args.output)
    print(f"wrote {count} {DATASET} records to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
