from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from scripts.morph.validate_verb_government_fixtures import validate


class VerbGovernmentFixtureValidatorTest(unittest.TestCase):
    def test_builtin_fixtures_cover_builtin_seed(self):
        self.assertEqual(
            validate(
                Path("data/grammar/verb_government.seed.tsv"),
                Path("data/grammar/verb_government.fixtures.tsv"),
            ),
            [],
        )

    def test_rejects_missing_seed_coverage(self):
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            seed = root / "seed.tsv"
            fixtures = root / "fixtures.tsv"
            seed.write_text(
                "ждать\tdirect_object\t\tgen\tproject.test\tnote\n"
                "говорить\tprepositional_object\tо\tprep\tproject.test\tnote\n",
                encoding="utf-8",
            )
            fixtures.write_text(
                "ждать\tdirect_object\t\tждать ответа\tждать ответу\tждать ответу\n",
                encoding="utf-8",
            )

            errors = validate(seed, fixtures)

        self.assertTrue(any("missing fixture" in error for error in errors))

    def test_rejects_fixture_without_seed_row(self):
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            seed = root / "seed.tsv"
            fixtures = root / "fixtures.tsv"
            seed.write_text(
                "ждать\tdirect_object\t\tgen\tproject.test\tnote\n",
                encoding="utf-8",
            )
            fixtures.write_text(
                "читать\tdirect_object\t\tчитать проект\tчитать проекту\tчитать проекту\n",
                encoding="utf-8",
            )

            errors = validate(seed, fixtures)

        self.assertTrue(any("has no seed row" in error for error in errors))

    def test_rejects_invalid_excerpt_outside_invalid_text(self):
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            seed = root / "seed.tsv"
            fixtures = root / "fixtures.tsv"
            seed.write_text(
                "ждать\tdirect_object\t\tgen\tproject.test\tnote\n",
                encoding="utf-8",
            )
            fixtures.write_text(
                "ждать\tdirect_object\t\tждать ответа\tждать ответу\tдругая строка\n",
                encoding="utf-8",
            )

            errors = validate(seed, fixtures)

        self.assertTrue(any("invalid_excerpt" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
