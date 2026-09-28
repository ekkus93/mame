#!/usr/bin/env python3
"""Regression guard for the BMR bundled-runtime TODO/workflow wiring."""

from __future__ import annotations

from pathlib import Path

TODO = Path("docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md")
WORKFLOW = Path(".github/workflows/tauri-project.yml")
TODO_TOKEN = "docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md"
STEP_NAME = "BMR bundled runtime TODO wiring regression tests"
SCRIPT_TOKEN = "scripts/tauri/test-bmr-bundled-runtime-todo-wiring.py"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    require(TODO.is_file(), f"missing canonical BMR TODO: {TODO}")
    require(WORKFLOW.is_file(), f"missing Tauri workflow: {WORKFLOW}")

    todo = TODO.read_text(encoding="utf-8")
    workflow = WORKFLOW.read_text(encoding="utf-8")

    require(
        "Canonical specification: `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_SPEC_2026-09-27.md`" in todo,
        "BMR TODO must keep its canonical specification link",
    )
    require("## BMR-000" in todo, "BMR TODO must retain baseline/invariant section")
    require("## BMR-015" in todo, "BMR TODO must retain final qualification section")
    require("Synthetic package fixtures alone are never sufficient" in todo, "BMR completion rule must reject synthetic-only closure")

    require(TODO_TOKEN in workflow, "Tauri workflow must trigger/check out the canonical BMR TODO")
    require(STEP_NAME in workflow, "Tauri workflow must run the BMR TODO wiring guard")
    require(SCRIPT_TOKEN in workflow, "Tauri workflow must invoke this guard script")

    print("BMR bundled runtime TODO wiring regression passed")


if __name__ == "__main__":
    main()
