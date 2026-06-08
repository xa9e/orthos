from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tests.importer_tests.support import PYTHON, run_cmd

class MorphImporterSmokeTests(unittest.TestCase):
    def test_morph_fixture_converter_normalizes_opencorpora_xml(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            output = Path(tmpdir) / "morph.tsv"
            run_cmd(
                PYTHON,
                "scripts/import_morph/convert_fixture.py",
                "--input",
                "testdata/fixtures/morph/opencorpora.xml",
                "--output",
                str(output),
                "--source-format",
                "opencorpora-xml",
                "--source-id",
                "fixture.opencorpora.xml",
            )
            rows = output.read_text(encoding="utf-8").splitlines()

        self.assertEqual(rows[0].split("\t"), ["form", "lemma", "pos", "features", "lemma_id", "paradigm_id", "source_id", "stress"])
        self.assertIn(
            "кота\tкот\tNOUN\tNOUN|anim|masc|gent|sing\topencorpora:1\topencorpora:paradigm:1\tfixture.opencorpora.xml\t",
            rows,
        )

    def test_morph_fixture_converter_normalizes_pymorphy_export(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            output = Path(tmpdir) / "pymorphy.tsv"
            run_cmd(
                PYTHON,
                "scripts/import_morph/convert_fixture.py",
                "--input",
                "testdata/fixtures/morph/pymorphy.tsv",
                "--output",
                str(output),
                "--source-format",
                "pymorphy-tsv",
                "--source-id",
                "fixture.pymorphy",
                "--no-header",
            )
            rows = output.read_text(encoding="utf-8").splitlines()

        self.assertEqual(len(rows), 2)
        self.assertIn("стали\tсталь\tNOUN\tNOUN|inan|femn|sing|gent", rows[0])
