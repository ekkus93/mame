#!/usr/bin/env python3
"""Report whether required GitHub Actions workflows are green on one exact SHA."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

DEFAULT_REQUIRED = (
    "Tauri project",
    "Tauri Linux packaging",
    "Tauri Windows packaging",
    "Tauri macOS packaging",
)


def resolve_sha(explicit: str | None) -> str:
    if explicit:
        return explicit
    if os.environ.get("GITHUB_SHA"):
        return os.environ["GITHUB_SHA"]
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return completed.stdout.strip()


def fetch_runs(repo: str, sha: str) -> dict[str, Any]:
    encoded_sha = urllib.parse.quote(sha, safe="")
    url = f"https://api.github.com/repos/{repo}/actions/runs?head_sha={encoded_sha}&per_page=100"
    request = urllib.request.Request(
        url,
        headers={
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "mame-tauri-ci-evidence",
        },
    )
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        request.add_header("Authorization", f"Bearer {token}")
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def newest_exact_run(runs: list[dict[str, Any]], workflow: str, sha: str) -> dict[str, Any] | None:
    candidates = [
        run
        for run in runs
        if run.get("name") == workflow and run.get("head_sha") == sha
    ]
    if not candidates:
        return None
    return max(
        candidates,
        key=lambda run: (
            str(run.get("updated_at") or run.get("created_at") or ""),
            int(run.get("id") or 0),
        ),
    )


def build_report(payload: dict[str, Any], sha: str, required: list[str]) -> dict[str, Any]:
    runs = payload.get("workflow_runs")
    if not isinstance(runs, list):
        raise ValueError("workflow payload does not contain a workflow_runs list")

    evidence: list[dict[str, Any]] = []
    qualified = True
    for workflow in required:
        run = newest_exact_run(runs, workflow, sha)
        if run is None:
            evidence.append(
                {
                    "workflow": workflow,
                    "found": False,
                    "runId": None,
                    "status": "missing",
                    "conclusion": None,
                    "url": None,
                }
            )
            qualified = False
            continue

        status = run.get("status")
        conclusion = run.get("conclusion")
        passed = status == "completed" and conclusion == "success"
        qualified = qualified and passed
        evidence.append(
            {
                "workflow": workflow,
                "found": True,
                "runId": run.get("id"),
                "status": status,
                "conclusion": conclusion,
                "url": run.get("html_url"),
            }
        )

    return {
        "schemaVersion": 1,
        "sha": sha,
        "qualified": qualified,
        "requiredWorkflows": required,
        "evidence": evidence,
    }


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# Exact-head CI evidence",
        "",
        f"Candidate SHA: `{report['sha']}`",
        "",
        f"Result: **{'PASS' if report['qualified'] else 'FAIL'}**",
        "",
        "| Workflow | Run | Status | Conclusion |",
        "| --- | ---: | --- | --- |",
    ]
    for item in report["evidence"]:
        run_text = "missing"
        if item["runId"] is not None:
            if item["url"]:
                run_text = f"[{item['runId']}]({item['url']})"
            else:
                run_text = str(item["runId"])
        lines.append(
            f"| {item['workflow']} | {run_text} | {item['status']} | "
            f"{item['conclusion'] or '-'} |"
        )
    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--repo",
        default=os.environ.get("GITHUB_REPOSITORY", "ekkus93/mame"),
        help="GitHub repository in owner/name form",
    )
    parser.add_argument("--sha", help="Exact candidate SHA; defaults to GITHUB_SHA or HEAD")
    parser.add_argument(
        "--require",
        action="append",
        dest="required",
        help="Required workflow name; repeat to override the default application/package set",
    )
    parser.add_argument(
        "--input-json",
        type=Path,
        help="Read a captured Actions runs payload instead of calling GitHub (test/offline mode)",
    )
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of Markdown")
    args = parser.parse_args()

    try:
        sha = resolve_sha(args.sha)
        required = args.required or list(DEFAULT_REQUIRED)
        payload = (
            json.loads(args.input_json.read_text(encoding="utf-8"))
            if args.input_json
            else fetch_runs(args.repo, sha)
        )
        report = build_report(payload, sha, required)
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        print(f"CI evidence error: {exc}", file=sys.stderr)
        return 2

    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(render_markdown(report), end="")
    return 0 if report["qualified"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
