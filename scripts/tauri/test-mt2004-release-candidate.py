#!/usr/bin/env python3
"""Regression tests for the MT-2004 release-candidate manifest."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("mt2004_release_candidate.py")
VALID_SHA = "a" * 40


class Mt2004ReleaseCandidateTests(unittest.TestCase):
    def run_manifest(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(SCRIPT), *args],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

    def manifest(self) -> dict[str, object]:
        completed = self.run_manifest("--sha", VALID_SHA, "--pretty")
        self.assertEqual(completed.returncode, 0, completed.stderr)
        return json.loads(completed.stdout)

    def test_manifest_records_all_mt2004_acceptance_items(self) -> None:
        payload = self.manifest()
        self.assertEqual(payload["schema_version"], 1)
        self.assertEqual(payload["repository"], "ekkus93/mame")
        self.assertEqual(payload["candidate_sha"], VALID_SHA)
        self.assertEqual(payload["release_candidate_type"], "external-window")
        self.assertEqual(
            set(payload["acceptance"]),
            {
                "package_on_supported_platforms",
                "complete_smoke_test",
                "document_known_limitations",
                "record_exact_qualified_commit",
            },
        )

    def test_supported_platform_packages_are_explicit(self) -> None:
        payload = self.manifest()
        artifacts = {entry["platform"]: entry for entry in payload["supported_artifacts"]}
        self.assertEqual(set(artifacts), {"linux", "windows", "macos"})
        self.assertEqual(artifacts["linux"]["package_formats"], ["Debian package", "AppImage"])
        self.assertEqual(artifacts["windows"]["package_formats"], ["NSIS installer"])
        self.assertEqual(artifacts["macos"]["package_formats"], ["app bundle", "DMG"])

        for platform, artifact in artifacts.items():
            with self.subTest(platform=platform):
                smoke_text = " ".join(artifact["required_smoke"])
                self.assertIn("runtime", smoke_text)
                self.assertIn("lockfiles", smoke_text)

    def test_required_gates_include_docs_quality_and_all_packaging_workflows(self) -> None:
        payload = self.manifest()
        gate_ids = {entry["gate_id"] for entry in payload["required_gates"]}
        self.assertEqual(
            gate_ids,
            {
                "mt2004-docs",
                "mt2004-linux-quality",
                "mt2004-linux-release-window",
                "mt2004-linux-packaging",
                "mt2004-windows-packaging",
                "mt2004-macos-packaging",
            },
        )

    def test_known_limitations_preserve_external_window_boundary(self) -> None:
        payload = self.manifest()
        limitation_ids = {entry["limitation_id"] for entry in payload["known_limitations"]}
        self.assertEqual(
            limitation_ids,
            {
                "external-window-only",
                "runtime-and-content-ownership",
                "macos-notarization-credentials",
                "physical-device-coverage",
            },
        )
        limitations_text = json.dumps(payload["known_limitations"])
        self.assertIn("embedded rendering is not claimed", limitations_text)
        self.assertIn("users supply their own ROM/software content", limitations_text)

    def test_bad_sha_is_rejected(self) -> None:
        completed = self.run_manifest("--sha", "ABC123")
        self.assertEqual(completed.returncode, 1)
        self.assertIn("40-character lowercase hexadecimal", completed.stderr)

    def test_markdown_and_write_modes_work(self) -> None:
        completed = self.run_manifest("--sha", VALID_SHA, "--format", "markdown")
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("MT-2004 External-Window Release Candidate Manifest", completed.stdout)
        self.assertIn(VALID_SHA, completed.stdout)

        with tempfile.TemporaryDirectory() as tmpdir:
            output_path = Path(tmpdir) / "nested" / "manifest.json"
            completed = self.run_manifest(
                "--sha",
                VALID_SHA,
                "--write",
                str(output_path),
            )
            self.assertEqual(completed.returncode, 0, completed.stderr)
            payload = json.loads(output_path.read_text(encoding="utf-8"))
            self.assertEqual(payload["candidate_sha"], VALID_SHA)


if __name__ == "__main__":
    unittest.main()
