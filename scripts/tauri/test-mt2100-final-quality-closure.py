#!/usr/bin/env python3
"""Regression tests for the MT-2100 final-quality closure ledger."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("mt2100_final_quality_closure.py")
MODULE_NAME = "mt2100_final_quality_closure_under_test"
VALID_SHA = "0123456789abcdef0123456789abcdef01234567"
PREVIOUS_SHA = "fedcba9876543210fedcba9876543210fedcba98"


def load_module():
    spec = importlib.util.spec_from_file_location(MODULE_NAME, SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("failed to load MT-2100 module")
    module = importlib.util.module_from_spec(spec)
    sys.modules[MODULE_NAME] = module
    spec.loader.exec_module(module)
    return module


class Mt2100FinalQualityClosureTests(unittest.TestCase):
    def test_ledger_covers_every_mt2100_category(self) -> None:
        module = load_module()
        ledger = module.build_ledger(VALID_SHA, PREVIOUS_SHA)

        self.assertEqual(ledger.schema_version, 1)
        self.assertEqual(ledger.candidate_sha, VALID_SHA)
        self.assertEqual(ledger.previous_qualified_sha, PREVIOUS_SHA)
        self.assertEqual(
            {category.id for category in ledger.categories},
            {"MT-2101", "MT-2102", "MT-2103", "MT-2104", "MT-2105", "MT-2106", "MT-2107"},
        )

        required_counts = {
            "MT-2101": 7,
            "MT-2102": 7,
            "MT-2103": 5,
            "MT-2104": 3,
            "MT-2105": 4,
            "MT-2106": 7,
            "MT-2107": 4,
        }
        self.assertEqual(
            {category.id: len(category.items) for category in ledger.categories},
            required_counts,
        )

    def test_every_item_has_explicit_safe_status_evidence_and_rationale(self) -> None:
        module = load_module()
        ledger = module.build_ledger(VALID_SHA, PREVIOUS_SHA)
        module.validate_ledger(ledger)

        allowed_statuses = {"complete", "deferred", "not_applicable"}
        seen: set[str] = set()
        for category in ledger.categories:
            for item in category.items:
                self.assertNotIn(item.id, seen)
                seen.add(item.id)
                self.assertIn(item.status, allowed_statuses)
                self.assertNotIn(item.status, {"todo", "unknown", "partial"})
                self.assertGreaterEqual(len(item.evidence), 1)
                self.assertTrue(item.rationale.strip())

    def test_cli_writes_valid_json(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "closure.json"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "--sha",
                    VALID_SHA,
                    "--previous-qualified-sha",
                    PREVIOUS_SHA,
                    "--pretty",
                    "--write",
                    str(output),
                ],
                check=False,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            payload = json.loads(output.read_text(encoding="utf-8"))
            self.assertEqual(payload["candidate_sha"], VALID_SHA)
            self.assertEqual(payload["previous_qualified_sha"], PREVIOUS_SHA)
            self.assertIn("Tauri project", payload["required_workflows"])

    def test_cli_rejects_invalid_sha(self) -> None:
        completed = subprocess.run(
            [sys.executable, str(SCRIPT), "--sha", "not-a-sha"],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

        self.assertEqual(completed.returncode, 2)
        self.assertIn("candidate_sha", completed.stderr)


if __name__ == "__main__":
    unittest.main()
