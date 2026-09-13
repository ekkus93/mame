#!/usr/bin/env python3
"""Offline regression tests for report-ci-evidence.py."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path


SCRIPT = Path(__file__).with_name("report-ci-evidence.py")
SHA = "0123456789abcdef0123456789abcdef01234567"
WORKFLOWS = (
    "Tauri project",
    "Tauri Linux packaging",
    "Tauri Windows packaging",
    "Tauri macOS packaging",
)


def run_helper(payload: dict, *extra: str) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory() as temporary:
        fixture = Path(temporary) / "runs.json"
        fixture.write_text(json.dumps(payload), encoding="utf-8")
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--sha", SHA, "--input-json", str(fixture), *extra],
            capture_output=True,
            text=True,
            check=False,
        )


def green_payload() -> dict:
    return {
        "workflow_runs": [
            {
                "id": 1000 + index,
                "name": workflow,
                "head_sha": SHA,
                "status": "completed",
                "conclusion": "success",
                "html_url": f"https://github.example/runs/{1000 + index}",
                "updated_at": f"2026-09-13T00:0{index}:00Z",
            }
            for index, workflow in enumerate(WORKFLOWS)
        ]
        + [
            {
                "id": 9999,
                "name": "Tauri project",
                "head_sha": "ffffffffffffffffffffffffffffffffffffffff",
                "status": "completed",
                "conclusion": "failure",
                "updated_at": "2026-09-13T23:59:59Z",
            }
        ]
    }


def main() -> int:
    green = run_helper(green_payload())
    assert green.returncode == 0, green.stderr
    assert "Result: **PASS**" in green.stdout
    assert SHA in green.stdout

    green_json = run_helper(green_payload(), "--json")
    assert green_json.returncode == 0, green_json.stderr
    decoded = json.loads(green_json.stdout)
    assert decoded["qualified"] is True
    assert decoded["sha"] == SHA
    assert len(decoded["evidence"]) == 4

    failed_payload = green_payload()
    failed_payload["workflow_runs"][2]["conclusion"] = "failure"
    failed = run_helper(failed_payload)
    assert failed.returncode == 1
    assert "Result: **FAIL**" in failed.stdout

    missing_payload = green_payload()
    missing_payload["workflow_runs"] = [
        run
        for run in missing_payload["workflow_runs"]
        if run["name"] != "Tauri Linux packaging"
    ]
    missing = run_helper(missing_payload)
    assert missing.returncode == 1
    assert "Tauri Linux packaging | missing" in missing.stdout

    duplicate_payload = green_payload()
    duplicate_payload["workflow_runs"].append(
        {
            "id": 1,
            "name": "Tauri project",
            "head_sha": SHA,
            "status": "completed",
            "conclusion": "failure",
            "updated_at": "2026-09-12T00:00:00Z",
        }
    )
    duplicate = run_helper(duplicate_payload, "--json")
    decoded_duplicate = json.loads(duplicate.stdout)
    assert duplicate.returncode == 0
    project = next(
        item for item in decoded_duplicate["evidence"] if item["workflow"] == "Tauri project"
    )
    assert project["runId"] == 1000

    print("MT-1407 exact-head CI evidence helper tests passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
