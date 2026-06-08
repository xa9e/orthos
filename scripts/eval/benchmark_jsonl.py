#!/usr/bin/env python3
"""Run orthos over normalized JSONL examples and report detection metrics.

This benchmark deliberately evaluates diagnostic behavior, not generative
correction quality. It can run in two modes:
- per-record: accurate per-example invocation, best for regression/smoke tests;
- batch: one checker call for all inputs and one for targets, faster for large
  exploratory runs, but line/global detectors can differ from per-record runs.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import tempfile
import time
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable, Mapping, Sequence

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "import"))
from common import clean_text, read_jsonl  # noqa: E402

MappingLike = Mapping[str, Any]
MAX_EXAMPLE_TEXT = 240
MAX_REPORTED_ISSUES_PER_EXAMPLE = 8
MAX_UNEXPECTED_TARGET_EXAMPLES = 20

def _exec_script_shard(relative_path: str) -> None:
    shard = Path(__file__).with_name("benchmark_jsonl") / relative_path
    exec(shard.read_text(), globals())

_exec_script_shard("types_and_args.py")
_exec_script_shard("execution.py")
_exec_script_shard("metrics.py")
_exec_script_shard("reporting.py")
_exec_script_shard("main.py")
