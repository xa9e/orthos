from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

from scripts.morph.validate_verb_government_seed import validate


class VerbGovernmentSeedValidatorTest(unittest.TestCase):
    def test_builtin_seed_is_valid(self):
        self.assertEqual(validate(Path("data/grammar/verb_government.seed.tsv")), [])

    def test_rejects_missing_preposition_for_prepositional_object(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "seed.tsv"
            path.write_text(
                "говорить\tprepositional_object\t\tprep\tproject.test\tnote\n",
                encoding="utf-8",
            )

            errors = validate(path)

        self.assertTrue(any("must specify preposition" in error for error in errors))

    def test_rejects_duplicate_complement_key(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "seed.tsv"
            path.write_text(
                "говорить\tprepositional_object\tо\tprep\tproject.test\tnote\n"
                "говорить\tprepositional_object\tо\tprep\tproject.test\tnote\n",
                encoding="utf-8",
            )

            errors = validate(path)

        self.assertTrue(any("duplicate" in error for error in errors))


if __name__ == "__main__":
    unittest.main()
