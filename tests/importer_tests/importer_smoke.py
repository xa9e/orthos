from __future__ import annotations

import json
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path

from tests.importer_tests.support import (
    PYTHON,
    ROOT,
    make_zip_from_dir,
    read_jsonl,
    run_cmd,
)

class ImporterSmokeTests(unittest.TestCase):
    def test_rlc_importer_uses_common_schema_and_groups_rlc_test(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            output = Path(tmpdir) / "rlc.jsonl"
            run_cmd(
                PYTHON,
                "scripts/import/rlc_annotated.py",
                "--input",
                "testdata/fixtures/import/rlc-annotated",
                "--output",
                str(output),
                "--part",
                "all",
            )
            records = read_jsonl(output)
        self.assertEqual(len(records), 2)
        self.assertEqual(records[0]["dataset"], "rlc-annotated")
        self.assertEqual(records[0]["input"], "Я незнаю ответ.")
        self.assertEqual(records[0]["correction"], "Я не знаю ответ.")
        self.assertEqual(records[0]["labels"], ["ortho"])
        self.assertEqual(len(records[1]["edits"]), 2)

    def test_lorugec_xlsx_importer_preserves_rule_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            output = Path(tmpdir) / "lorugec.jsonl"
            run_cmd(
                PYTHON,
                "scripts/import/lorugec.py",
                "--input",
                "testdata/fixtures/import/lorugec",
                "--output",
                str(output),
            )
            records = read_jsonl(output)
        self.assertEqual(len(records), 2)
        self.assertEqual(records[0]["dataset"], "lorugec")
        self.assertEqual(records[0]["input"], "Я знаю что он придёт.")
        self.assertEqual(records[0]["correction"], "Я знаю, что он придёт.")
        self.assertEqual(records[0]["metadata"]["subset"], "test")
        self.assertIn("Punctuation", records[0]["labels"])
        self.assertEqual(records[1]["metadata"]["subset"], "validation")

    def test_lorugec_m2_fallback_importer(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            output = Path(tmpdir) / "lorugec-m2.jsonl"
            run_cmd(
                PYTHON,
                "scripts/import/lorugec.py",
                "--input",
                "testdata/fixtures/import/lorugec",
                "--source",
                "m2",
                "--output",
                str(output),
            )
            records = read_jsonl(output)
        self.assertEqual(len(records), 2)
        self.assertEqual(records[0]["input"], "Я незнаю ответ.")
        self.assertEqual(records[0]["correction"], "Я не знаю ответ.")
        self.assertEqual(records[1]["correction"], "Я знаю, что он придёт.")

    def test_ruspellgold_importer_can_keep_or_skip_identical_pairs(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            output = Path(tmpdir) / "ruspellgold.jsonl"
            run_cmd(
                PYTHON,
                "scripts/import/ruspellgold.py",
                "--input",
                "testdata/fixtures/import/ruspellgold",
                "--output",
                str(output),
            )
            records = read_jsonl(output)
        self.assertEqual(len(records), 1)
        self.assertEqual(records[0]["labels"], ["spelling"])
        self.assertEqual(records[0]["metadata"]["domain"], "news")


    def test_dataset_specific_importers_accept_zip_archives(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)

            rlc_zip = make_zip_from_dir(ROOT / "testdata/fixtures/import/rlc-annotated", tmp / "rlc-annotated-main.zip", "rlc-annotated-main")
            rlc_output = tmp / "rlc.jsonl"
            run_cmd(
                PYTHON,
                "scripts/import/rlc_annotated.py",
                "--input",
                str(rlc_zip),
                "--output",
                str(rlc_output),
                "--part",
                "all",
            )
            self.assertEqual(len(read_jsonl(rlc_output)), 2)

            lorugec_zip = make_zip_from_dir(ROOT / "testdata/fixtures/import/lorugec", tmp / "LORuGEC-main.zip", "LORuGEC-main")
            lorugec_output = tmp / "lorugec.jsonl"
            run_cmd(
                PYTHON,
                "scripts/import/lorugec.py",
                "--input",
                str(lorugec_zip),
                "--output",
                str(lorugec_output),
            )
            self.assertEqual(len(read_jsonl(lorugec_output)), 2)

            ruspellgold_zip = make_zip_from_dir(ROOT / "testdata/fixtures/import/ruspellgold", tmp / "RuSpellGold.zip", "RuSpellGold")
            ruspellgold_output = tmp / "ruspellgold.jsonl"
            run_cmd(
                PYTHON,
                "scripts/import/ruspellgold.py",
                "--input",
                str(ruspellgold_zip),
                "--output",
                str(ruspellgold_output),
            )
            self.assertEqual(len(read_jsonl(ruspellgold_output)), 1)

    def test_generic_pair_importer_reports_common_schema(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            output = Path(tmpdir) / "custom.jsonl"
            run_cmd(
                PYTHON,
                "scripts/import/gec_pairs_to_jsonl.py",
                "testdata/custom/smoke.tsv",
                str(output),
                "--dataset",
                "custom-smoke",
                "--source-col",
                "2",
                "--target-col",
                "3",
                "--skip-header",
                "--label",
                "smoke",
            )
            records = read_jsonl(output)
        self.assertEqual(len(records), 3)
        self.assertEqual(records[0]["dataset"], "custom-smoke")
        self.assertEqual(records[0]["labels"], ["smoke"])
        self.assertEqual(records[0]["metadata"], {"source_row": 2})

    def test_generic_pair_importer_rejects_short_rows(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            source = tmp / "broken.tsv"
            output = tmp / "broken.jsonl"
            source.write_text("только один столбец\n", encoding="utf-8")
            proc = subprocess.run(
                [
                    PYTHON,
                    "scripts/import/gec_pairs_to_jsonl.py",
                    str(source),
                    str(output),
                    "--dataset",
                    "broken",
                ],
                cwd=ROOT,
                text=True,
                capture_output=True,
                check=False,
                timeout=60,
            )
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("expected at least 2 columns", proc.stderr)

    def test_benchmark_script_accepts_fake_checker_binary(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            jsonl = tmp / "dataset.jsonl"
            run_cmd(
                PYTHON,
                "scripts/import/rlc_annotated.py",
                "--input",
                "testdata/fixtures/import/rlc-annotated",
                "--output",
                str(jsonl),
            )
            fake_checker = tmp / "fake_checker.py"
            fake_checker.write_text(
                textwrap.dedent(
                    """
                    #!/usr/bin/env python3
                    import json, pathlib, sys
                    path = pathlib.Path(sys.argv[2])
                    issues = []
                    for line_no, line in enumerate(path.read_text(encoding='utf-8').splitlines(), start=1):
                        if 'незнаю' in line:
                            issues.append({
                                'rule_id': 'ru.orthography.ne_common_confusables',
                                'start': {'line': line_no, 'column': 1},
                                'end': {'line': line_no, 'column': 7},
                            })
                    print(json.dumps(issues, ensure_ascii=False))
                    """
                ).strip()
                + "\n",
                encoding="utf-8",
            )
            fake_checker.chmod(0o755)
            report = tmp / "report.json"
            run_cmd(
                PYTHON,
                "scripts/eval/benchmark_jsonl.py",
                "--input",
                str(jsonl),
                "--checker-bin",
                str(fake_checker),
                "--mode",
                "per-record",
                "--output",
                str(report),
            )
            payload = json.loads(report.read_text(encoding="utf-8"))
        self.assertEqual(payload["examples"], 1)
        self.assertEqual(payload["source_diagnostics"], 1)
        self.assertEqual(payload["target_diagnostics"], 0)
        self.assertEqual(payload["target_rule_hit_rate"], 1.0)
        self.assertEqual(payload["source_detection_precision_proxy"], 1.0)
        self.assertEqual(payload["source_detection_recall_proxy"], 1.0)
        self.assertEqual(payload["source_detection_f0_5_proxy"], 1.0)
        self.assertEqual(payload["false_positive_diagnostics_per_1000_target_tokens"], 0.0)
        self.assertEqual(payload["source_issues_by_dataset_label"], {"ortho": 1})
        self.assertEqual(payload["source_issues_by_rule_id"], {"ru.orthography.ne_common_confusables": 1})
        self.assertGreaterEqual(payload["runtime"]["checker_invocations"], 2)
        self.assertEqual(payload["configuration"]["profile"], "default")
        self.assertIn("--profile", payload["configuration"]["checker_sample_command"])
        self.assertEqual(payload["unexpected_target_diagnostic_examples"], [])

    def test_dataset_regression_fixture_uses_benchmark_schema(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp = Path(tmpdir)
            fake_checker = tmp / "fake_checker.py"
            fake_checker.write_text("#!/usr/bin/env python3\nprint('[]')\n", encoding="utf-8")
            fake_checker.chmod(0o755)
            report = tmp / "report.json"
            run_cmd(
                PYTHON,
                "scripts/eval/benchmark_jsonl.py",
                "--input",
                "testdata/fixtures/eval/gec_dataset_regressions.jsonl",
                "--checker-bin",
                str(fake_checker),
                "--mode",
                "per-record",
                "--output",
                str(report),
            )
            payload = json.loads(report.read_text(encoding="utf-8"))
        self.assertEqual(payload["examples"], 4)
        self.assertEqual(payload["source_diagnostics"], 0)
        self.assertEqual(payload["target_diagnostics"], 0)
