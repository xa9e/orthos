from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tests.importer_tests.support import PYTHON, run_cmd

class BenchmarkOrchestrationTests(unittest.TestCase):
    def test_run_benchmark_dry_run_uses_registry_and_local_archive(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            output_dir = tmp / "out"
            report_dir = tmp / "reports"
            proc = run_cmd(
                PYTHON,
                "scripts/eval/run_benchmark.py",
                "--dataset",
                "rlc-annotated",
                "--archive",
                "testdata/fixtures/import/rlc-annotated",
                "--output-dir",
                str(output_dir),
                "--report-dir",
                str(report_dir),
                "--checker-bin",
                "fake-checker",
                "--dry-run",
            )
        self.assertIn("scripts/import/rlc_annotated.py", proc.stdout)
        self.assertIn("scripts/eval/benchmark_jsonl.py", proc.stdout)
        self.assertIn("data/eval/gec/rlc-annotated.jsonl", proc.stdout)
