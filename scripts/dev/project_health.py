#!/usr/bin/env python3
"""Small repository health checks for local development."""

from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence

CHECKED_SUFFIXES = {
    ".rs",
    ".py",
    ".md",
    ".yaml",
    ".yml",
    ".toml",
    ".json",
    ".txt",
    ".tsv",
}
IGNORED_PARTS = {".git", "target", "__pycache__", ".pytest_cache", ".venv", "venv"}
OVERSIZED_FILE_ALLOWLIST = {
    Path("data/lexicon/demo_morph.tsv"),
}
BAD_LOCK_CHECKSUM = "Could not get crate checksum"
RUST_INCLUDE_STR_RE = re.compile(r'include_str!\(\s*"([^"]+)"\s*\)')
RUST_ITEM_STARTS = (
    "fn ",
    "pub fn ",
    "mod ",
    "pub mod ",
    "struct ",
    "pub struct ",
    "enum ",
    "pub enum ",
    "impl ",
    "trait ",
    "pub trait ",
    "type ",
    "pub type ",
    "const ",
    "pub const ",
    "static ",
    "pub static ",
    "use ",
    "pub use ",
    "macro_rules!",
)


@dataclass(frozen=True)
class OversizedFile:
    path: Path
    line_count: int


def is_checked_file(path: Path) -> bool:
    return path.is_file() and path.suffix in CHECKED_SUFFIXES


def is_ignored(path: Path) -> bool:
    return any(part in IGNORED_PARTS for part in path.parts)


def count_lines(path: Path) -> int:
    with path.open("r", encoding="utf-8", errors="ignore") as fh:
        return sum(1 for _ in fh)


def find_oversized_files(root: Path, limit: int) -> list[OversizedFile]:
    oversized = []
    for path in sorted(root.rglob("*")):
        relative = path.relative_to(root)
        if is_ignored(relative) or not is_checked_file(path):
            continue
        if relative in OVERSIZED_FILE_ALLOWLIST:
            continue
        line_count = count_lines(path)
        if line_count > limit:
            oversized.append(OversizedFile(relative, line_count))
    return oversized


def cargo_lock_errors(root: Path, require_lock: bool) -> list[str]:
    lock = root / "Cargo.lock"
    if not lock.exists():
        if require_lock:
            return ["Cargo.lock is missing; run `cargo generate-lockfile`."]
        return []
    content = lock.read_text(encoding="utf-8", errors="ignore")
    if BAD_LOCK_CHECKSUM in content:
        return [
            "Cargo.lock contains placeholder crate checksums; remove it and run "
            "`cargo generate-lockfile`."
        ]
    return []


def rust_include_str_errors(root: Path) -> list[str]:
    errors = []
    src = root / "src"
    if not src.exists():
        return errors
    for path in sorted(src.rglob("*.rs")):
        content = path.read_text(encoding="utf-8", errors="ignore")
        for match in RUST_INCLUDE_STR_RE.finditer(content):
            raw_target = match.group(1)
            if raw_target.startswith(("/", "env!", "$")):
                continue
            target = (path.parent / raw_target).resolve()
            try:
                target.relative_to(root.resolve())
            except ValueError:
                errors.append(f"{path.relative_to(root)} includes outside repo: {raw_target}")
                continue
            if not target.exists():
                errors.append(f"{path.relative_to(root)} has missing include_str target: {raw_target}")
    return errors


def dangling_rust_attribute_errors(root: Path) -> list[str]:
    errors = []
    src = root / "src"
    if not src.exists():
        return errors
    for path in sorted(src.rglob("*.rs")):
        lines = path.read_text(encoding="utf-8", errors="ignore").splitlines()
        cursor = len(lines) - 1
        while cursor >= 0 and not lines[cursor].strip():
            cursor -= 1
        if cursor >= 0 and lines[cursor].strip().startswith("#["):
            errors.append(f"{path.relative_to(root)}:{cursor + 1} has dangling Rust attribute")
    return errors


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--line-limit", type=int, default=300)
    parser.add_argument("--require-cargo-lock", action="store_true")
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    root = args.root.resolve()
    errors: list[str] = []

    oversized = find_oversized_files(root, args.line_limit)
    for item in oversized:
        errors.append(f"{item.path}: {item.line_count} lines > {args.line_limit}")

    errors.extend(cargo_lock_errors(root, args.require_cargo_lock))
    errors.extend(rust_include_str_errors(root))
    errors.extend(dangling_rust_attribute_errors(root))

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    print(f"OK: project health checks passed for {root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
