#!/usr/bin/env python3
"""Regression checks for real bundled MAME runtime build automation."""

from __future__ import annotations

from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
BUILD_SCRIPT = REPO / "scripts" / "tauri" / "build-linux-bundled-mame-runtime.sh"
PREPARE_SCRIPT = REPO / "scripts" / "tauri" / "prepare-bundled-mame-runtime.sh"
VALIDATOR_SCRIPT = REPO / "scripts" / "tauri" / "validate-real-mame-runtime.sh"
DEB_DEPS_SCRIPT = REPO / "scripts" / "tauri" / "augment-deb-runtime-deps.sh"
REAL_WORKFLOW = REPO / ".github" / "workflows" / "tauri-linux-real-runtime-qualification.yml"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> None:
    build = BUILD_SCRIPT.read_text(encoding="utf-8")
    prepare = PREPARE_SCRIPT.read_text(encoding="utf-8")
    validator = VALIDATOR_SCRIPT.read_text(encoding="utf-8")
    deb_deps = DEB_DEPS_SCRIPT.read_text(encoding="utf-8")
    workflow = REAL_WORKFLOW.read_text(encoding="utf-8")

    require(BUILD_SCRIPT.stat().st_mode & 0o111 != 0, "build script must be executable")
    require(VALIDATOR_SCRIPT.stat().st_mode & 0o111 != 0, "real runtime validator must be executable")
    require(DEB_DEPS_SCRIPT.stat().st_mode & 0o111 != 0, "Debian dependency augmenter must be executable")
    require("prepare-bundled-mame-runtime.sh" in build, "build script must stage through the common preparer")
    require("MAME_TAURI_REAL_RUNTIME_SOURCE_ROOT" in build, "build script must provide a real source root")
    require("MAME_TAURI_REAL_RUNTIME_EXECUTABLE" in build, "build script must provide a real executable")
    require("MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME=1" not in build, "real build path must not opt into synthetic runtime")
    require("MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME" in prepare, "synthetic opt-in must remain isolated in the preparer")
    require("validate-real-mame-runtime.sh" in prepare, "real preparer must run the bounded runtime validator")

    require("-listxml" in validator and "MAME_TAURI_REAL_RUNTIME_SMOKE_MACHINE" in validator, "real runtime validator must prove configurable bounded metadata generation")
    require("synthetic|fixture payload|MT-1305" in validator, "real runtime validator must reject synthetic payloads")
    require("ldd" in deb_deps, "Debian dependency augmenter must inspect bundled MAME shared libraries")
    require("dpkg-query -S" in deb_deps, "Debian dependency augmenter must map runtime libraries to packages")
    require("dpkg-deb --build" in deb_deps, "Debian dependency augmenter must rebuild the package after dependency updates")

    require("workflow_dispatch:" in workflow, "real runtime qualification must be manually dispatchable")
    require("build-linux-bundled-mame-runtime.sh" in workflow, "workflow must use the real runtime build script")
    require("validate-real-mame-runtime.sh" in workflow, "workflow must validate the staged and installed real runtime")
    require("augment-deb-runtime-deps.sh" in workflow, "workflow must augment Debian dependencies from bundled MAME")
    require("release_qualified=true" in workflow, "workflow must assert release-qualified provenance")
    require("runtime-provenance.txt" in workflow, "workflow must inspect runtime provenance")
    require("MAME_TAURI_RUNTIME_MAKE_TARGET" in workflow, "workflow must expose the make target")
    require("MAME_TAURI_ALLOW_SYNTHETIC_RUNTIME" not in workflow, "real qualification workflow must not allow synthetic fallback")
    require("distro mame package" in workflow, "workflow must assert the distro mame package is not required")
    require("sudo apt-get remove" in workflow and "/mame-runtime/" in workflow, "workflow must verify package uninstall removes runtime files")

    print("BMR real-runtime build automation regression checks passed")


if __name__ == "__main__":
    main()
