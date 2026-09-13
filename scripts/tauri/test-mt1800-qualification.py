#!/usr/bin/env python3
"""Offline regression tests for mt1800_qualification.py."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("mt1800_qualification.py")
SHA = "0123456789abcdef0123456789abcdef01234567"
EXPECTED_REQUIREMENTS = {
    "development build",
    "packaged build",
    "external MAME executable",
    "bundled sidecar if supported",
    "controller input",
    "audio",
    "fullscreen",
    "X11",
    "Wayland where supported/claimed",
    "renderer behavior",
    "signed packaged build",
    "notarization/install behavior",
    "path differences",
    "settings migration",
    "controller identity differences",
    "artwork paths",
}
EXPECTED_COVERAGE = {"automated-ci", "delegated-runtime-boundary", "documented-boundary"}


def run_helper(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        capture_output=True,
        text=True,
        check=False,
    )


def main() -> int:
    result = run_helper("--sha", SHA, "--json")
    assert result.returncode == 0, result.stderr
    report = json.loads(result.stdout)
    assert report["schemaVersion"] == 1
    assert report["milestone"] == "MT-1800"
    assert report["candidateSha"] == SHA
    assert report["qualified"] is True
    assert report["summary"]["itemCount"] == len(report["items"])
    assert report["summary"]["byPlatform"]["Linux"] == 9
    assert report["summary"]["byPlatform"]["Windows"] == 6
    assert report["summary"]["byPlatform"]["macOS"] == 6
    assert report["summary"]["byPlatform"]["Cross-platform"] == 4

    requirements = {item["requirement"] for item in report["items"]}
    assert EXPECTED_REQUIREMENTS <= requirements
    coverages = {item["coverage"] for item in report["items"]}
    assert EXPECTED_COVERAGE <= coverages
    assert report["summary"]["byCoverage"]["automated-ci"] >= 1
    assert report["summary"]["byCoverage"]["delegated-runtime-boundary"] >= 1
    assert report["summary"]["byCoverage"]["documented-boundary"] >= 1
    ids = [item["id"] for item in report["items"]]
    assert len(ids) == len(set(ids))
    for item in report["items"]:
        assert item["qualified"] is True
        assert item["evidence"]
        assert item["acceptance"].strip()

    markdown = run_helper("--sha", SHA)
    assert markdown.returncode == 0, markdown.stderr
    assert "# MT-1800 cross-platform qualification ledger" in markdown.stdout
    assert "Result: **PASS**" in markdown.stdout
    assert "documented-boundary" in markdown.stdout

    bad_sha = run_helper("--sha", "not-a-sha", "--json")
    assert bad_sha.returncode == 2
    assert "sha must be" in bad_sha.stderr

    with tempfile.TemporaryDirectory() as directory:
        output_path = Path(directory) / "nested" / "mt1800.json"
        written = run_helper("--sha", SHA, "--json", "--write", str(output_path))
        assert written.returncode == 0, written.stderr
        assert json.loads(output_path.read_text(encoding="utf-8"))["candidateSha"] == SHA

    print("MT-1800 qualification ledger tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
