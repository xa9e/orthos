from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from scripts.morph.validate_language_model_data import validate_all


class LanguageModelDataValidatorTest(unittest.TestCase):
    def test_builtin_language_model_data_is_valid(self):
        self.assertEqual(validate_all(Path(".")), [])

    def test_reports_seed_and_fixture_errors_together(self):
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            grammar = root / "data" / "grammar"
            grammar.mkdir(parents=True)
            (grammar / "verb_government.seed.tsv").write_text(
                "ждать\tdirect_object\t\tweird\tproject.test\tnote\n",
                encoding="utf-8",
            )
            (grammar / "verb_government.fixtures.tsv").write_text(
                "читать\tdirect_object\t\tчитать проект\tчитать проекту\tчитать проекту\n",
                encoding="utf-8",
            )
            (grammar / "verb_government.false_positive.tsv").write_text(
                "fp1\tчитать\tdirect_object\t\tчитать проекту\tчитать проекту\tMagicBoundary\treason\n",
                encoding="utf-8",
            )

            errors = validate_all(root)

        self.assertTrue(any("unknown case" in error for error in errors))
        self.assertTrue(any("has no seed row" in error for error in errors))
        self.assertTrue(any("unknown expected_blocker" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
