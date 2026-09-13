#!/usr/bin/env python3
"""Regression tests for MT-1900 upstream inventory classification."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("mt1900_upstream_inventory.py")


class Mt1900InventoryTests(unittest.TestCase):
    def run_inventory(self, paths: list[str], *extra: str) -> subprocess.CompletedProcess[str]:
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as handle:
            handle.write("\n".join(paths))
            handle.write("\n")
            path_file = handle.name
        try:
            return subprocess.run(
                [sys.executable, str(SCRIPT), "--paths-file", path_file, "--pretty", *extra],
                check=False,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
            )
        finally:
            Path(path_file).unlink(missing_ok=True)

    def test_project_owned_paths_are_not_reported_as_upstream_patches(self) -> None:
        completed = self.run_inventory(
            [
                "tauri/src/App.tsx",
                "scripts/tauri/mt1900_upstream_inventory.py",
                "tests/tauri/fixture.txt",
                "docs/MAME_TAURI_MT1900_UPSTREAM_SUSTAINABILITY_2026-09-13.md",
                ".github/workflows/tauri-project.yml",
            ]
        )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        payload = json.loads(completed.stdout)
        self.assertEqual(payload["schema_version"], 1)
        self.assertEqual(payload["project_owned_count"], 5)
        self.assertEqual(payload["upstream_tree_touch_count"], 0)
        self.assertEqual(payload["upstream_tree_touches"], [])

    def test_upstream_tree_touches_are_reported_and_can_fail_the_gate(self) -> None:
        completed = self.run_inventory(
            [
                "tauri/src/App.tsx",
                "src/mame/video/example.cpp",
                "makefile",
            ],
            "--expect-no-upstream-touches",
        )

        self.assertEqual(completed.returncode, 2, completed.stdout)
        payload = json.loads(completed.stdout)
        self.assertEqual(payload["project_owned_count"], 1)
        self.assertEqual(payload["upstream_tree_touch_count"], 2)
        touched_paths = {entry["path"] for entry in payload["upstream_tree_touches"]}
        self.assertEqual(touched_paths, {"makefile", "src/mame/video/example.cpp"})

    def test_ambiguous_paths_are_rejected(self) -> None:
        completed = self.run_inventory(["tauri/../src/mame.cpp"])

        self.assertEqual(completed.returncode, 1)
        self.assertIn("ambiguous path segment", completed.stderr)


if __name__ == "__main__":
    unittest.main()
