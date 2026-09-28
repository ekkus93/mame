#!/usr/bin/env python3
"""Regression checks for real bundled MAME runtime build automation."""

from __future__ import annotations

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BUILD_SCRIPT = REPO / "scripts" / "tauri" / "build-linux-bundled-mame-runtime.sh"
PREPARE_SCRIPT = REPO / "scripts" / "tauri" / "prepare-bundled-mame-runtime.sh"
REAL_WORKFLOW = REPO / ".github" / "workflows" / "tauri-linux-real-runtime-qualification.yml"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    build = BUILD_SCRIPT.read_text(encoding="utf-8")
    prepare = PREPARE_SCRIPT.read_text(encoding="utf-8")
    workflow = REAL_WORKFLOW.read_text(encoding="utf-8")

    require(BUILD_SCRIPT.stat().st_mode & 0o111 != 0, "build script must be executable")
    require("prepare-bundled-mame-runtime.sh" in build, "build script must stage through the common preparer")
    require("MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT" in build, "build script must provide a real source root")
    require("MAME_TAURI_REAL_RUNTIME_EXECUTABLE" in build, "build script must provide a real executable")
    require("MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME=1" not in build, "real build path must not opt into synthetic runtime")
    require("MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME" in prepare, "synthetic opt-in must remain isolated in the preparer")

    require("workflow_dispatch:" in workflow, "real runtime qualification must be manually dispatchable")
    require("build-linux-bundled-mame-runtime.sh" in workflow, "workflow must use the real runtime build script")
    require("release_qualified=true" in workflow, "workflow must assert release-qualified provenance")
    require("runtime-provenance.txt" in workflow, "workflow must inspect runtime provenance")
    require("MAME_TAURI_RUNTIME_MAKE_TARGET" in workflow, "workflow must expose the make target")
    require("MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME" not in workflow, "real qualification workflow must not allow synthetic fallback")

    print("BMR real-runtime build automation regression checks passed")


if __name__ == "__main__":
    main()
