#!/usr/bin/env python3
"""Shared helpers for deterministic dataset importers.

The normalized record contract is intentionally small and stable. Dataset-specific
fields belong in `metadata`, not as ad-hoc top-level keys.
"""

from __future__ import annotations

import json
import re
import zipfile
from pathlib import Path
from typing import Any, Iterable, Iterator, Mapping, Sequence

REQUIRED_KEYS = (
    "id",
    "dataset",
    "source_file",
    "input",
    "correction",
    "targets",
    "edits",
    "labels",
    "metadata",
)


def clean_text(value: Any) -> str:
    """Return a stripped one-line string for ids, labels, and metadata fields."""
    if value is None:
        return ""
    text = str(value).replace("\ufeff", "")
    text = re.sub(r"[\r\n\t]+", " ", text)
    return text.strip()


def clean_sentence(value: Any) -> str:
    """Return a one-line sentence while preserving leading/trailing surface spaces."""
    if value is None:
        return ""
    text = str(value).replace("\ufeff", "")
    return re.sub(r"[\r\n\t]+", " ", text)


def unique_preserve_order(values: Iterable[Any]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for value in values:
        text = clean_text(value)
        if not text or text in seen:
            continue
        seen.add(text)
        out.append(text)
    return out


def unique_sentences(values: Iterable[Any]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for value in values:
        text = clean_sentence(value)
        if not text.strip() or text in seen:
            continue
        seen.add(text)
        out.append(text)
    return out


def make_record(
    *,
    record_id: str,
    dataset: str,
    source_file: str,
    input_text: str,
    correction: str = "",
    targets: Sequence[str] | None = None,
    edits: Sequence[Mapping[str, Any]] | None = None,
    labels: Sequence[str] | None = None,
    metadata: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    normalized_correction = clean_sentence(correction)
    normalized_targets = unique_sentences(targets if targets is not None else ([normalized_correction] if normalized_correction.strip() else []))
    return {
        "id": clean_text(record_id),
        "dataset": clean_text(dataset),
        "source_file": clean_text(source_file),
        "input": clean_sentence(input_text),
        "correction": normalized_correction,
        "targets": normalized_targets,
        "edits": list(edits or []),
        "labels": unique_preserve_order(labels or []),
        "metadata": dict(metadata or {}),
    }


def validate_record(record: Mapping[str, Any]) -> None:
    missing = [key for key in REQUIRED_KEYS if key not in record]
    if missing:
        raise ValueError(f"record {record.get('id', '<unknown>')} missing keys: {', '.join(missing)}")
    if not isinstance(record["input"], str) or not record["input"].strip():
        raise ValueError(f"record {record.get('id', '<unknown>')} has empty input")
    if not isinstance(record["targets"], list):
        raise ValueError(f"record {record.get('id', '<unknown>')} targets must be a list")
    if not isinstance(record["edits"], list):
        raise ValueError(f"record {record.get('id', '<unknown>')} edits must be a list")
    if not isinstance(record["labels"], list):
        raise ValueError(f"record {record.get('id', '<unknown>')} labels must be a list")
    if not isinstance(record["metadata"], dict):
        raise ValueError(f"record {record.get('id', '<unknown>')} metadata must be an object")


def write_jsonl(records: Iterable[Mapping[str, Any]], output: Path) -> int:
    output.parent.mkdir(parents=True, exist_ok=True)
    count = 0
    with output.open("w", encoding="utf-8") as dst:
        for record in records:
            validate_record(record)
            dst.write(json.dumps(record, ensure_ascii=False, sort_keys=True) + "\n")
            count += 1
    return count


def read_jsonl(path: Path) -> Iterator[dict[str, Any]]:
    with path.open("r", encoding="utf-8") as src:
        for line_no, line in enumerate(src, start=1):
            if not line.strip():
                continue
            try:
                record = json.loads(line)
            except json.JSONDecodeError as exc:
                raise ValueError(f"{path}:{line_no}: invalid JSONL: {exc}") from exc
            validate_record(record)
            yield record


def iter_archive_members(input_path: Path) -> Iterator[tuple[str, bytes]]:
    """Yield `(relative_name, bytes)` for a directory tree or zip archive."""
    if input_path.is_dir():
        for path in sorted(p for p in input_path.rglob("*") if p.is_file()):
            yield path.relative_to(input_path).as_posix(), path.read_bytes()
        return
    if zipfile.is_zipfile(input_path):
        with zipfile.ZipFile(input_path) as archive:
            for name in sorted(n for n in archive.namelist() if not n.endswith("/")):
                yield name, archive.read(name)
        return
    raise ValueError(f"unsupported input path, expected directory or zip archive: {input_path}")


def find_member(input_path: Path, suffix: str) -> tuple[str, bytes]:
    matches = [(name, data) for name, data in iter_archive_members(input_path) if name.endswith(suffix)]
    if not matches:
        raise FileNotFoundError(f"could not find member ending with {suffix!r} in {input_path}")
    if len(matches) > 1:
        exact = [item for item in matches if Path(item[0]).name == Path(suffix).name]
        matches = exact or matches
    return matches[0]


def find_members(input_path: Path, suffixes: Sequence[str]) -> dict[str, tuple[str, bytes]]:
    return {suffix: find_member(input_path, suffix) for suffix in suffixes}
