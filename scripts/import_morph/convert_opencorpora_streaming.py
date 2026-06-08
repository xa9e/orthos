#!/usr/bin/env python3
"""Streaming converter: OpenCorpora XML -> project TSV.

Uses xml.etree.ElementTree.iterparse for constant-memory processing
of the full dict.opcorpora.xml (~400MB, ~392K lemmas, ~5M wordforms).

Optionally filters by POS and deduplicates records.
"""

from __future__ import annotations

import argparse
import csv
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

POS_TAGS = {
    "NOUN", "ADJF", "ADJS", "COMP",
    "VERB", "INFN", "PRTF", "PRTS", "GRND",
    "NUMR", "ADVB", "NPRO", "PRED",
    "PREP", "CONJ", "PRCL", "INTJ",
}

GRAMMATICALLY_RELEVANT_POS = {
    "NOUN", "ADJF", "ADJS", "COMP",
    "VERB", "INFN", "PRTF", "PRTS", "GRND",
    "NUMR", "NPRO", "PREP",
}

PROJECT_COLUMNS = ("form", "lemma", "pos", "features", "lemma_id", "paradigm_id", "source_id", "stress")


def first_pos(tags: list[str]) -> str:
    for tag in tags:
        if tag in POS_TAGS:
            return tag
    return "Other"


def stream_opencorpora_xml(
    path: Path,
    source_id: str,
    pos_filter: set[str] | None,
    output: Path,
) -> int:
    count = 0
    seen: set[str] = set()

    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.writer(fh, delimiter="\t", lineterminator="\n")
        writer.writerow(PROJECT_COLUMNS)

        lemma_id = ""
        lemma = ""
        lemma_tags: list[str] = []

        context = ET.iterparse(str(path), events=("start", "end"))

        for event, elem in context:
            if event == "start":
                if elem.tag == "lemma":
                    lemma_id = elem.attrib.get("id", "")
                elif elem.tag == "l":
                    lemma = elem.attrib.get("t", "").strip()
                    lemma_tags = [g.attrib.get("v", "").strip() for g in elem.findall("g") if g.attrib.get("v")]
                elif elem.tag == "f":
                    form = elem.attrib.get("t", "").strip()
                    if not form or not lemma:
                        continue

                    form_tags = [g.attrib.get("v", "").strip() for g in elem.findall("g") if g.attrib.get("v")]
                    all_tags = lemma_tags + form_tags
                    pos = first_pos(all_tags)

                    if pos_filter and pos not in pos_filter:
                        continue

                    features = "|".join(t for t in all_tags if t)
                    dedup_key = f"{form}\t{lemma}\t{pos}\t{features}"

                    if dedup_key in seen:
                        continue
                    seen.add(dedup_key)

                    writer.writerow([
                        form,
                        lemma,
                        pos,
                        features,
                        f"opencorpora:{lemma_id}" if lemma_id else "",
                        f"opencorpora:paradigm:{lemma_id}" if lemma_id else "",
                        source_id,
                        "",
                    ])
                    count += 1

            elif event == "end":
                if elem.tag == "lemma":
                    lemma_id = ""
                    lemma = ""
                    lemma_tags = []
                elem.clear()

        return count


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--source-id", default="opencorpora")
    parser.add_argument("--pos-filter", action="store_true",
                        help="Only include grammatically relevant POS (NOUN, ADJF, ADJS, VERB, etc.)")
    args = parser.parse_args()

    pos_filter = GRAMMATICALLY_RELEVANT_POS if args.pos_filter else None
    count = stream_opencorpora_xml(args.input, args.source_id, pos_filter, args.output)
    print(f"wrote {count} morphology records to {args.output}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1)
