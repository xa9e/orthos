#!/usr/bin/env python3
"""Compatibility wrapper for the orthos CLI check contract.

The benchmark accepts a checker binary invoked as:

    checker-bin check <file> --rules rules --format json [extra check args...]

This wrapper exposes that shape while delegating to `cargo run --quiet -- check`
from the project root. It is intentionally thin: argument validation belongs to
Rust CLI code, not this Python shim.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    if not args or args[0] != "check":
        print("usage: run_cargo_checker.py check <file> --rules <rules> --format json [check args...]", file=sys.stderr)
        return 2
    if shutil.which("cargo") is None:
        print("cargo is not available; use --checker-bin with a compiled binary or fake checker for tests", file=sys.stderr)
        return 127
    command = ["cargo", "run", "--quiet", "--"] + args
    return subprocess.run(command, cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
