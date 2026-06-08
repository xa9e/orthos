from __future__ import annotations

import unittest

from tests.importer_tests.benchmark_orchestration import BenchmarkOrchestrationTests
from tests.importer_tests.importer_smoke import ImporterSmokeTests
from tests.importer_tests.morph_importer_smoke import MorphImporterSmokeTests

__all__ = [
    "BenchmarkOrchestrationTests",
    "ImporterSmokeTests",
    "MorphImporterSmokeTests",
]


if __name__ == "__main__":
    unittest.main()
