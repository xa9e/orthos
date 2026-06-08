#!/usr/bin/env python3
"""Import LORuGEC into the common evaluation JSONL schema.

The preferred source is LORuGEC.xlsx because it preserves rule names,
definitions, sections, source URLs, and validation/test split markers. The M2
files can be used as a fallback when the workbook is unavailable.
"""

from __future__ import annotations

import argparse
import io
import re
import sys
import zipfile
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Any, Iterable

sys.path.insert(0, str(Path(__file__).resolve().parent))
from common import clean_text, find_member, make_record, unique_preserve_order, write_jsonl  # noqa: E402

DATASET = "lorugec"
SPREADSHEET_COLUMNS = {
    "The rule": "rule",
    "The definition of the rule": "rule_definition",
    "Source": "rule_source",
    "Grammar section": "grammar_section",
    "Did the base model have difficulties with the rule?": "base_model_difficulty",
    "Initial sentence": "input",
    "Correct sentence": "correction",
    "Are both sentences the same?": "sentences_same",
    "Prompt/Query": "subset_marker",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", required=True, type=Path, help="Path to LORuGEC-main.zip or unpacked directory")
    parser.add_argument("--output", required=True, type=Path, help="Output normalized JSONL path")
    parser.add_argument(
        "--source",
        choices=("xlsx", "m2"),
        default="xlsx",
        help="Normalize from LORuGEC.xlsx metadata or from M2 files.",
    )
    return parser.parse_args()


def column_index(cell_ref: str) -> int:
    match = re.match(r"([A-Z]+)", cell_ref)
    if not match:
        raise ValueError(f"invalid cell reference: {cell_ref}")
    result = 0
    for char in match.group(1):
        result = result * 26 + (ord(char) - ord("A") + 1)
    return result - 1


def xlsx_rows(xlsx_bytes: bytes) -> list[list[str]]:
    """Read the first worksheet from a minimal XLSX using only stdlib XML."""
    ns = {"a": "http://schemas.openxmlformats.org/spreadsheetml/2006/main"}
    rel_ns = {
        "a": "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
        "r": "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    }
    with zipfile.ZipFile(io.BytesIO(xlsx_bytes)) as archive:
        shared_strings: list[str] = []
        if "xl/sharedStrings.xml" in archive.namelist():
            root = ET.fromstring(archive.read("xl/sharedStrings.xml"))
            for item in root.findall("a:si", ns):
                shared_strings.append("".join(text.text or "" for text in item.findall(".//a:t", ns)))

        workbook = ET.fromstring(archive.read("xl/workbook.xml"))
        first_sheet = workbook.find("a:sheets/a:sheet", rel_ns)
        if first_sheet is None:
            raise ValueError("workbook has no sheets")
        rel_id = first_sheet.attrib["{http://schemas.openxmlformats.org/officeDocument/2006/relationships}id"]

        rels_root = ET.fromstring(archive.read("xl/_rels/workbook.xml.rels"))
        rels_ns = {"rel": "http://schemas.openxmlformats.org/package/2006/relationships"}
        target = None
        for rel in rels_root.findall("rel:Relationship", rels_ns):
            if rel.attrib.get("Id") == rel_id:
                target = rel.attrib["Target"]
                break
        if target is None:
            raise ValueError(f"could not resolve worksheet relationship {rel_id}")
        sheet_path = f"xl/{target.lstrip('/')}" if not target.startswith("xl/") else target
        worksheet = ET.fromstring(archive.read(sheet_path))

        output: list[list[str]] = []
        for row in worksheet.findall(".//a:sheetData/a:row", ns):
            cells: dict[int, str] = {}
            for cell in row.findall("a:c", ns):
                ref = cell.attrib.get("r", "A1")
                idx = column_index(ref)
                text = ""
                if cell.attrib.get("t") == "inlineStr":
                    inline = cell.find("a:is", ns)
                    if inline is not None:
                        text = "".join(item.text or "" for item in inline.findall(".//a:t", ns))
                else:
                    value = cell.find("a:v", ns)
                    if value is not None and value.text is not None:
                        text = value.text
                        if cell.attrib.get("t") == "s":
                            text = shared_strings[int(text)]
                cells[idx] = clean_text(text)
            if not cells:
                output.append([])
                continue
            max_idx = max(cells)
            output.append([cells.get(i, "") for i in range(max_idx + 1)])
        return output


def subset_from_marker(marker: str) -> str:
    normalized = marker.casefold()
    if normalized == "prompt":
        return "validation"
    if normalized == "query":
        return "test"
    return marker or "unknown"


def spreadsheet_records(input_path: Path) -> Iterable[dict[str, Any]]:
    source_file, data = find_member(input_path, "LORuGEC.xlsx")
    rows = xlsx_rows(data)
    if not rows:
        return
    header = rows[0]
    field_by_index = {idx: SPREADSHEET_COLUMNS.get(name, name or f"unnamed_{idx}") for idx, name in enumerate(header)}

    for index, row in enumerate(rows[1:], start=1):
        normalized = {field_by_index[idx]: clean_text(value) for idx, value in enumerate(row) if idx in field_by_index}
        input_text = detokenize_m2(normalized.get("input", ""))
        correction = detokenize_m2(normalized.get("correction", ""))
        if not input_text:
            continue
        labels = unique_preserve_order([normalized.get("grammar_section"), normalized.get("rule")])
        metadata = {
            "rule": normalized.get("rule", ""),
            "rule_definition": normalized.get("rule_definition", ""),
            "rule_source": normalized.get("rule_source", ""),
            "grammar_section": normalized.get("grammar_section", ""),
            "base_model_difficulty": normalized.get("base_model_difficulty", ""),
            "sentences_same": normalized.get("sentences_same", ""),
            "subset_marker": normalized.get("subset_marker", ""),
            "subset": subset_from_marker(normalized.get("subset_marker", "")),
            "source_row": index + 1,
        }
        edits = []
        if input_text != correction:
            edits.append(
                {
                    "tag": normalized.get("grammar_section", ""),
                    "rule": normalized.get("rule", ""),
                    "source_file": source_file,
                    "note": "spreadsheet-level pair; no source span provided",
                }
            )
        yield make_record(
            record_id=f"{DATASET}/xlsx-{index}",
            dataset=DATASET,
            source_file=source_file,
            input_text=input_text,
            correction=correction,
            targets=[correction] if correction else [],
            edits=edits,
            labels=labels,
            metadata=metadata,
        )


def detokenize_m2(text: str) -> str:
    """Convert conservative M2 token spacing back to readable Russian text."""
    text = clean_text(text)
    replacements = [
        (r"\s+([,.:;!?%])", r"\1"),
        (r"([«(\[])\s+", r"\1"),
        (r"\s+([»)\]])", r"\1"),
        (r"\s+([—–-])\s+", r" \1 "),
        (r"\s+", " "),
    ]
    for pattern, replacement in replacements:
        text = re.sub(pattern, replacement, text)
    return text.strip()


def apply_m2_edits(tokens: list[str], edits: list[dict[str, Any]]) -> str:
    current = list(tokens)
    # Apply from right to left so earlier spans remain stable.
    for edit in sorted(edits, key=lambda item: int(item["start"]), reverse=True):
        start = int(edit["start"])
        end = int(edit["end"])
        correction = clean_text(edit.get("correction"))
        replacement = [] if correction in {"", "-NONE-", "None"} else correction.split()
        current[start:end] = replacement
    return detokenize_m2(" ".join(current))


def parse_m2(data: bytes, source_file: str, offset: int) -> Iterable[dict[str, Any]]:
    text = data.decode("utf-8-sig")
    blocks = [block for block in text.split("\n\n") if block.strip()]
    for local_index, block in enumerate(blocks, start=1):
        lines = [line for line in block.splitlines() if line.strip()]
        if not lines or not lines[0].startswith("S "):
            continue
        tokenized_source = lines[0][2:].strip()
        tokens = tokenized_source.split()
        edits = []
        labels = []
        for edit_index, line in enumerate(lines[1:], start=1):
            if not line.startswith("A "):
                continue
            parts = line[2:].split("|||")
            if len(parts) < 3:
                continue
            start_end = parts[0].split()
            if len(start_end) != 2:
                continue
            tag = clean_text(parts[1])
            correction = clean_text(parts[2])
            labels.append(tag)
            edits.append(
                {
                    "id": f"{Path(source_file).name}-{local_index}-edit-{edit_index}",
                    "start": start_end[0],
                    "end": start_end[1],
                    "tag": tag,
                    "correction": correction,
                    "required": clean_text(parts[5]) if len(parts) > 5 else "",
                    "annotator": clean_text(parts[-1]) if parts else "",
                    "source_file": source_file,
                }
            )
        input_text = detokenize_m2(tokenized_source)
        correction = apply_m2_edits(tokens, edits) if edits else input_text
        yield make_record(
            record_id=f"{DATASET}/m2-{offset + local_index}",
            dataset=DATASET,
            source_file=source_file,
            input_text=input_text,
            correction=correction,
            targets=[correction] if correction else [],
            edits=edits,
            labels=unique_preserve_order(labels),
            metadata={"m2_block_index": local_index},
        )


def m2_records(input_path: Path) -> Iterable[dict[str, Any]]:
    offset = 0
    for suffix in ("LORuGEC.val.m2", "LORuGEC.test.m2"):
        source_file, data = find_member(input_path, suffix)
        records = list(parse_m2(data, source_file, offset))
        for record in records:
            record["metadata"]["subset"] = "validation" if suffix.endswith("val.m2") else "test"
            yield record
        offset += len(records)


def main() -> int:
    args = parse_args()
    records = spreadsheet_records(args.input) if args.source == "xlsx" else m2_records(args.input)
    count = write_jsonl(records, args.output)
    print(f"wrote {count} {DATASET} records to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
