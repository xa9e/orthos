from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from scripts.morph.validate_verb_government_false_positives import validate


class VerbGovernmentFalsePositiveValidatorTest(unittest.TestCase):
    def test_builtin_false_positive_fixtures_are_valid(self):
        self.assertEqual(
            validate(
                Path("data/grammar/verb_government.seed.tsv"),
                Path("data/grammar/verb_government.false_positive.tsv"),
            ),
            [],
        )

    def test_rejects_false_positive_without_seed_row(self):
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            seed = root / "seed.tsv"
            fixtures = root / "false_positive.tsv"
            seed.write_text(
                "ждать\tdirect_object\t\tgen\tproject.test\tnote\n",
                encoding="utf-8",
            )
            fixtures.write_text(
                "fp1\tчитать\tdirect_object\t\tчитать проекту\tчитать проекту\t\treason\n",
                encoding="utf-8",
            )

            errors = validate(seed, fixtures)

        self.assertTrue(any("has no seed row" in error for error in errors))

    def test_rejects_unknown_blocker(self):
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            seed = root / "seed.tsv"
            fixtures = root / "false_positive.tsv"
            seed.write_text(
                "ждать\tdirect_object\t\tgen\tproject.test\tnote\n",
                encoding="utf-8",
            )
            fixtures.write_text(
                "fp1\tждать\tdirect_object\t\tждать ответу\tждать ответу\tMagicBoundary\treason\n",
                encoding="utf-8",
            )

            errors = validate(seed, fixtures)

        self.assertTrue(any("unknown expected_blocker" in error for error in errors))

    def test_rejects_excerpt_outside_text(self):
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            seed = root / "seed.tsv"
            fixtures = root / "false_positive.tsv"
            seed.write_text(
                "говорить\tprepositional_object\tо\tprep\tproject.test\tnote\n",
                encoding="utf-8",
            )
            fixtures.write_text(
                "fp1\tговорить\tprepositional_object\tо\tговорить о задачу\tдругая строка\t\treason\n",
                encoding="utf-8",
            )

            errors = validate(seed, fixtures)

        self.assertTrue(any("forbidden_excerpt" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
