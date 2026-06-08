#!/usr/bin/env python3
"""Import a registered local dataset archive and run the diagnostic benchmark.

This script is a small orchestration layer over:
- data/dataset-registry.json
- scripts/import/*.py
- scripts/eval/benchmark_jsonl.py

It does not download datasets. Point it at local archives or a directory that
contains the expected archive names from the registry.
"""

from __future__ import annotations

import argparse
import json
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any, Mapping, Sequence

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_REGISTRY = ROOT / "data" / "dataset-registry.json"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dataset", required=True, help="Dataset registry name, e.g. lorugec")
    parser.add_argument("--archive", type=Path, help="Path to the local archive or extracted dataset directory")
    parser.add_argument(
        "--dataset-root",
        type=Path,
        default=Path("."),
        help="Directory containing expected local archive names when --archive is omitted",
    )
    parser.add_argument("--registry", type=Path, default=DEFAULT_REGISTRY)
    parser.add_argument("--output-dir", type=Path, default=ROOT, help="Base directory for normalized_output_path")
    parser.add_argument("--report-dir", type=Path, default=ROOT / "reports" / "eval")
    parser.add_argument("--rules", type=Path, default=Path("rules"))
    parser.add_argument("--checker-bin", type=Path, help="Compiled checker or compatible fake checker")
    parser.add_argument("--morph-lexicon", type=Path)
    parser.add_argument("--profile", default="default", choices=("default", "strict", "typography-only", "grammar-research"))
    parser.add_argument("--mode", choices=("per-record", "batch"), help="Override registry benchmark mode")
    parser.add_argument("--limit", type=int)
    parser.add_argument("--skip-import", action="store_true", help="Reuse existing normalized JSONL")
    parser.add_argument("--dry-run", action="store_true", help="Print commands without executing")
    return parser.parse_args()


def shell_join(command: Sequence[str]) -> str:
    return " ".join(shlex.quote(part) for part in command)


def load_registry(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as src:
        registry = json.load(src)
    if not isinstance(registry, dict) or not isinstance(registry.get("datasets"), list):
        raise ValueError(f"{path} does not look like a dataset registry")
    return registry


def find_dataset(registry: Mapping[str, Any], name: str) -> dict[str, Any]:
    for item in registry.get("datasets", []):
        if item.get("name") == name:
            return dict(item)
    known = ", ".join(sorted(str(item.get("name")) for item in registry.get("datasets", [])))
    raise KeyError(f"unknown dataset {name!r}; known datasets: {known}")


def resolved_archive(args: argparse.Namespace, dataset: Mapping[str, Any]) -> Path:
    if args.archive:
        return args.archive
    expected = str(dataset.get("expected_local_archive_name") or "")
    if not expected:
        raise ValueError(f"dataset {dataset.get('name')} has no expected_local_archive_name")
    return args.dataset_root / expected


def normalized_output(args: argparse.Namespace, dataset: Mapping[str, Any]) -> Path:
    rel = Path(str(dataset.get("normalized_output_path") or f"data/eval/{dataset['name']}.jsonl"))
    return rel if rel.is_absolute() else args.output_dir / rel


def render_command(parts: Sequence[str], *, input_path: Path, output_path: Path) -> list[str]:
    return [part.format(input=str(input_path), output=str(output_path)) for part in parts]


def run(command: Sequence[str], *, dry_run: bool) -> None:
    print(shell_join(command))
    if dry_run:
        return
    subprocess.run(command, cwd=ROOT, check=True)


def benchmark_command(args: argparse.Namespace, dataset: Mapping[str, Any], normalized: Path, report: Path) -> list[str]:
    mode = args.mode or str(dataset.get("recommended_benchmark_mode") or "per-record")
    command = [
        sys.executable,
        "scripts/eval/benchmark_jsonl.py",
        "--input",
        str(normalized),
        "--rules",
        str(args.rules),
        "--mode",
        mode,
        "--profile",
        args.profile,
        "--output",
        str(report),
    ]
    if args.checker_bin:
        command += ["--checker-bin", str(args.checker_bin)]
    if args.morph_lexicon:
        command += ["--morph-lexicon", str(args.morph_lexicon)]
    if args.limit is not None:
        command += ["--limit", str(args.limit)]
    return command


def main() -> int:
    args = parse_args()
    registry = load_registry(args.registry)
    dataset = find_dataset(registry, args.dataset)
    archive = resolved_archive(args, dataset)
    output = normalized_output(args, dataset)
    report = args.report_dir / f"{args.dataset}.json"

    if not args.skip_import and not archive.exists():
        raise FileNotFoundError(
            f"local dataset input not found: {archive}; pass --archive or put {dataset.get('expected_local_archive_name')} under --dataset-root"
        )

    if not args.skip_import:
        importer = render_command(dataset["importer_command"], input_path=archive, output_path=output)
        run(importer, dry_run=args.dry_run)

    run(benchmark_command(args, dataset, output, report), dry_run=args.dry_run)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
