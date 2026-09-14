#!/usr/bin/env python3
"""Regression coverage for the post-MT-2200 hardening batch."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SESSION_PANEL = ROOT / "tauri/src/session/SessionControlPanel.tsx"
APP = ROOT / "tauri/src/App.tsx"
TYPES = ROOT / "tauri/src/backend/types.ts"
CONTROL_RS = ROOT / "tauri/src-tauri/src/sessions/control.rs"
CONTROL_SPLIT_FILES = (
    "control_registry.rs",
    "control_bootstrap.rs",
    "control_channel.rs",
    "control_requests.rs",
    "control_wait.rs",
    "control_correlation.rs",
    "control_parser.rs",
    "control_events.rs",
    "control_protocol.rs",
    "control_tests.rs",
)
BUILD_RS = ROOT / "tauri/src-tauri/build.rs"
WORKFLOW = ROOT / ".github/workflows/tauri-project.yml"
SPEC = ROOT / "docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_SPEC_2026-09-13.md"
TODO = ROOT / "docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_TODO_2026-09-13.md"


def read(path: Path) -> str:
    if not path.is_file():
        raise SystemExit(f"post-closeout hardening regression: missing {path}")
    return path.read_text(encoding="utf-8")


def require(text: str, token: str, label: str) -> None:
    if token not in text:
        raise SystemExit(f"post-closeout hardening regression: {label} missing {token!r}")


def main() -> int:
    session_panel = read(SESSION_PANEL)
    app = read(APP)
    types = read(TYPES)
    control_rs = read(CONTROL_RS)
    control_line_count = len(control_rs.splitlines())
    if control_line_count >= 800:
        raise SystemExit(
            "post-closeout hardening regression: control.rs must remain under "
            f"800 lines after split refactor; found {control_line_count}"
        )
    control_split_texts = []
    for split_name in CONTROL_SPLIT_FILES:
        split_path = ROOT / "tauri/src-tauri/src/sessions" / split_name
        control_split_texts.append(read(split_path))
        require(control_rs, split_name, "runtime-control split include")
    control_combined = "\n".join([control_rs, *control_split_texts])
    build_rs = read(BUILD_RS)
    workflow = read(WORKFLOW)
    spec = read(SPEC)
    todo = read(TODO)

    for command in (
        "pauseMame",
        "resumeMame",
        "resetMame",
        "setMameMute",
        "queryMameRuntimeState",
    ):
        require(session_panel, command, "SessionControlPanel runtime command exposure")

    for label in (
        "Pause",
        "Resume",
        "Soft reset",
        "Mute",
        "Unmute",
        "Refresh runtime state",
    ):
        require(session_panel, label, "SessionControlPanel visible runtime control")

    require(session_panel, "const canCommand = isRunningSession(session)", "running-session gate")
    require(session_panel, 'role="group"', "control grouping role")
    require(session_panel, 'aria-label="MAME runtime controls"', "control grouping label")
    require(session_panel, "Runtime:", "runtime-state display")

    for event_name in ("session.started", "session.exited", "session.crashed", "session.failed"):
        require(app, event_name, "application lifecycle shortcut refresh bridge")
    require(app, 'window.dispatchEvent(new Event("focus"))', "shortcut ownership refresh event")
    require(app, "Library shortcuts already fail closed", "fail-closed listener setup documentation")

    for field in (
        "executable: MameExecutableIdentity;",
        "effectiveArgv: string[];",
        "effectiveConfig: EffectiveLaunchConfig;",
        "createdAtEpochMs: number;",
        "startedAtEpochMs: number | null;",
        "endedAtEpochMs: number | null;",
        "exitCode: number | null;",
        "terminationSignal: number | null;",
        "forcedTermination: boolean;",
        "stdoutTail: string;",
        "stderrTail: string;",
        "stdoutTruncated: boolean;",
        "stderrTruncated: boolean;",
        "diagnosticError: string | null;",
    ):
        require(types, field, "frontend SessionSnapshot parity")

    compatibility = read(ROOT / "tauri/src/session/saveStateCompatibility.ts")
    require(compatibility, "const executable = session.executable;", "typed session executable usage")
    if "as SessionSnapshot" in compatibility or "SessionWithExecutable" in compatibility:
        raise SystemExit(
            "post-closeout hardening regression: save-state compatibility must not widen SessionSnapshot ad hoc"
        )

    require(control_combined, "std::fs::Permissions::from_mode(0o600)", "Unix bootstrap chmod")
    for token in ("std::fs::Permissions::from_mode(0o600)", "#[cfg(not(unix))]"):
        require(control_combined, token, "runtime-control bootstrap platform boundary")
    for token in ("TEMP directory ACLs", "per-session frame-token entropy", "RAII cleanup"):
        require(spec, token, "non-Unix bootstrap security documentation")

    for token in (
        "control_pause_resume.lua",
        "control_mute.lua",
        "control_query_state.lua",
        "control_mute.rs",
        "control_query_state.rs",
        "replace_once",
    ):
        require(build_rs, token, "runtime-control source-generation guardrail")
    for split_name in CONTROL_SPLIT_FILES:
        require(build_rs, split_name, "runtime-control split rerun guard")

    require(workflow, "Post-closeout hardening regression tests", "CI wiring")
    require(workflow, "python3 scripts/tauri/test-post-closeout-hardening.py", "CI wiring")
    require(workflow, "/docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_SPEC_2026-09-13.md", "CI sparse checkout")
    require(workflow, "/docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_TODO_2026-09-13.md", "CI sparse checkout")

    for task in ("PCH-001", "PCH-002", "PCH-003", "PCH-004", "PCH-005", "PCH-006", "PCH-007"):
        require(spec, task, "hardening spec task coverage")
        require(todo, task, "hardening TODO task coverage")

    print("post-closeout hardening regression passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
