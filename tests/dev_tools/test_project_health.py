import tempfile
import unittest
from pathlib import Path

from scripts.dev.project_health import (
    cargo_lock_errors,
    dangling_rust_attribute_errors,
    find_oversized_files,
    rust_include_str_errors,
)


class ProjectHealthTests(unittest.TestCase):
    def test_finds_oversized_checked_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "src").mkdir()
            target = root / "src" / "large.rs"
            target.write_text("x\n" * 4, encoding="utf-8")

            oversized = find_oversized_files(root, limit=3)

        self.assertEqual(len(oversized), 1)
        self.assertEqual(str(oversized[0].path), "src/large.rs")
        self.assertEqual(oversized[0].line_count, 4)

    def test_allows_large_demo_morph_fixture(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            target = root / "data" / "lexicon" / "demo_morph.tsv"
            target.parent.mkdir(parents=True)
            target.write_text("x\n" * 4, encoding="utf-8")

            oversized = find_oversized_files(root, limit=3)

        self.assertEqual(oversized, [])

    def test_rejects_placeholder_cargo_lock_checksums(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "Cargo.lock").write_text(
                'checksum = "Could not get crate checksum"\n', encoding="utf-8"
            )

            errors = cargo_lock_errors(root, require_lock=False)

        self.assertEqual(len(errors), 1)
        self.assertIn("placeholder crate checksums", errors[0])

    def test_allows_missing_lock_when_not_required(self):
        with tempfile.TemporaryDirectory() as tmp:
            errors = cargo_lock_errors(Path(tmp), require_lock=False)

        self.assertEqual(errors, [])

    def test_rejects_missing_include_str_targets(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            src = root / "src"
            src.mkdir()
            (src / "lib.rs").write_text(
                'const DATA: &str = include_str!("../data/missing.tsv");\n',
                encoding="utf-8",
            )

            errors = rust_include_str_errors(root)

        self.assertEqual(len(errors), 1)
        self.assertIn("missing include_str target", errors[0])

    def test_rejects_dangling_rust_attributes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            src = root / "src"
            src.mkdir()
            (src / "lib.rs").write_text("#[cfg(test)]\n", encoding="utf-8")

            errors = dangling_rust_attribute_errors(root)

        self.assertEqual(len(errors), 1)
        self.assertIn("dangling Rust attribute", errors[0])


if __name__ == "__main__":
    unittest.main()
