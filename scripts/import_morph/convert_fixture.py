#!/usr/bin/env python3
"""Convert tiny morphology fixture snapshots into the project TSV format.

This tool is deliberately offline and deterministic. It is for fixture/smoke
conversion and local experiments, not for vendoring large third-party
dictionaries into the repository. Before committing generated data, verify the
license of the dictionary data itself and record attribution/provenance.
"""

from __future__ import annotations

import argparse
import csv
import sys
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

PROJECT_COLUMNS = ("form", "lemma", "pos", "features", "lemma_id", "paradigm_id", "source_id", "stress")


@dataclass(frozen=True, order=True)
class MorphRecord:
    form: str
    lemma: str
    pos: str
    features: str
    lemma_id: str = ""
    paradigm_id: str = ""
    source_id: str = ""
    stress: str = ""

    def as_tsv_row(self) -> list[str]:
        return [
            self.form,
            self.lemma,
            self.pos,
            self.features,
            self.lemma_id,
            self.paradigm_id,
            self.source_id,
            self.stress,
        ]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--source-format",
        required=True,
        choices=("project-tsv", "opencorpora-xml", "opencorpora-csv", "pymorphy-tsv"),
    )
    parser.add_argument("--source-id", required=True)
    parser.add_argument("--encoding", default="utf-8-sig")
    parser.add_argument("--no-header", action="store_true")
    return parser.parse_args()


def normalize_tags(raw: str) -> str:
    parts: list[str] = []
    for chunk in raw.replace(",", "|").split("|"):
        parts.extend(part.strip() for part in chunk.split() if part.strip())
    return "|".join(parts)


def first_pos(tags: Iterable[str], fallback: str = "Other") -> str:
    pos_tags = {
        "NOUN",
        "ADJF",
        "ADJS",
        "COMP",
        "VERB",
        "INFN",
        "PRTF",
        "PRTS",
        "GRND",
        "NUMR",
        "ADVB",
        "NPRO",
        "PRED",
        "PREP",
        "CONJ",
        "PRCL",
        "INTJ",
    }
    for tag in tags:
        clean = tag.strip()
        if clean.upper() in pos_tags:
            return clean.upper()
    return fallback


def read_project_tsv(path: Path, encoding: str) -> list[MorphRecord]:
    records: list[MorphRecord] = []
    with path.open("r", encoding=encoding, newline="") as fh:
        reader = csv.reader(fh, delimiter="\t")
        for line_no, row in enumerate(reader, start=1):
            if not row or not any(cell.strip() for cell in row) or row[0].lstrip().startswith("#"):
                continue
            if row[0] == "form":
                continue
            if len(row) < 4:
                raise ValueError(f"{path}:{line_no}: project TSV requires at least 4 columns")
            padded = row + [""] * (8 - len(row))
            records.append(MorphRecord(*[cell.strip() for cell in padded[:8]]))
    return records


def read_opencorpora_xml(path: Path, source_id: str, encoding: str) -> list[MorphRecord]:
    root = ET.fromstring(path.read_text(encoding=encoding))
    records: list[MorphRecord] = []
    for lemma_node in root.findall(".//lemma"):
        lemma_id = lemma_node.attrib.get("id", "")
        lemma_tag = lemma_node.find("l")
        if lemma_tag is None:
            continue
        lemma = lemma_tag.attrib.get("t", "").strip()
        lemma_grammemes = [g.attrib.get("v", "").strip() for g in lemma_tag.findall("g") if g.attrib.get("v")]
        pos = first_pos(lemma_grammemes)
        for form_node in lemma_node.findall("f"):
            form = form_node.attrib.get("t", "").strip()
            if not form or not lemma:
                continue
            form_grammemes = [g.attrib.get("v", "").strip() for g in form_node.findall("g") if g.attrib.get("v")]
            tags = [tag for tag in [*lemma_grammemes, *form_grammemes] if tag]
            records.append(
                MorphRecord(
                    form=form,
                    lemma=lemma,
                    pos=pos,
                    features="|".join(tags),
                    lemma_id=f"opencorpora:{lemma_id}" if lemma_id else "",
                    paradigm_id=f"opencorpora:paradigm:{lemma_id}" if lemma_id else "",
                    source_id=source_id,
                )
            )
    return records


def read_opencorpora_csv(path: Path, source_id: str, encoding: str) -> list[MorphRecord]:
    records: list[MorphRecord] = []
    with path.open("r", encoding=encoding, newline="") as fh:
        reader = csv.DictReader(fh)
        for line_no, row in enumerate(reader, start=2):
            form = (row.get("form") or "").strip()
            lemma = (row.get("lemma") or "").strip()
            pos = (row.get("pos") or "").strip()
            grammemes = normalize_tags(row.get("grammemes") or row.get("features") or "")
            if not form or not lemma or not pos:
                raise ValueError(f"{path}:{line_no}: form, lemma and pos are required")
            records.append(
                MorphRecord(
                    form=form,
                    lemma=lemma,
                    pos=pos,
                    features="|".join(tag for tag in (pos, grammemes) if tag),
                    lemma_id=(row.get("lemma_id") or "").strip(),
                    paradigm_id=(row.get("paradigm_id") or "").strip(),
                    source_id=(row.get("source_id") or source_id).strip(),
                    stress=(row.get("stress") or "").strip(),
                )
            )
    return records


def read_pymorphy_tsv(path: Path, source_id: str, encoding: str) -> list[MorphRecord]:
    records: list[MorphRecord] = []
    with path.open("r", encoding=encoding, newline="") as fh:
        reader = csv.DictReader(fh, delimiter="\t")
        for line_no, row in enumerate(reader, start=2):
            form = (row.get("word") or row.get("form") or "").strip()
            lemma = (row.get("normal_form") or row.get("lemma") or "").strip()
            tags = normalize_tags(row.get("tag") or row.get("features") or "")
            if not form or not lemma or not tags:
                raise ValueError(f"{path}:{line_no}: word, normal_form and tag are required")
            split = tags.split("|")
            records.append(
                MorphRecord(
                    form=form,
                    lemma=lemma,
                    pos=first_pos(split),
                    features=tags,
                    lemma_id=(row.get("lemma_id") or "").strip(),
                    paradigm_id=(row.get("paradigm_id") or "").strip(),
                    source_id=(row.get("source_id") or source_id).strip(),
                    stress=(row.get("stress") or "").strip(),
                )
            )
    return records


def write_project_tsv(records: Iterable[MorphRecord], output: Path, *, header: bool) -> int:
    ordered = sorted(records)
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.writer(fh, delimiter="\t", lineterminator="\n")
        if header:
            writer.writerow(PROJECT_COLUMNS)
        for record in ordered:
            writer.writerow(record.as_tsv_row())
    return len(ordered)


def main() -> int:
    args = parse_args()
    if args.source_format == "project-tsv":
        records = read_project_tsv(args.input, args.encoding)
    elif args.source_format == "opencorpora-xml":
        records = read_opencorpora_xml(args.input, args.source_id, args.encoding)
    elif args.source_format == "opencorpora-csv":
        records = read_opencorpora_csv(args.input, args.source_id, args.encoding)
    elif args.source_format == "pymorphy-tsv":
        records = read_pymorphy_tsv(args.input, args.source_id, args.encoding)
    else:  # pragma: no cover - argparse guards this
        raise AssertionError(args.source_format)
    count = write_project_tsv(records, args.output, header=not args.no_header)
    print(f"wrote {count} morphology records to {args.output}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
