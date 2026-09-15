#!/usr/bin/env python3
"""Regression coverage for the post-MT-2200 hardening batch."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SESSION_PANEL = ROOT / "tauri/src/session/SessionControlPanel.tsx"
APP = ROOT / "tauri/src/App.tsx"
TYPES = ROOT / "tauri/src/backend/types.ts"
EVENTS_TS = ROOT / "tauri/src/backend/events.ts"
EVENT_NAMES_RS = ROOT / "tauri/src-tauri/src/event_names.rs"
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
MAME_UI_REGRESSION = ROOT / "scripts/tauri/test-mame-ui-reproduction.py"


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
    events_ts = read(EVENTS_TS)
    event_names_rs = read(EVENT_NAMES_RS)
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
    require(control_rs, 'include_str!("control_pause_resume.lua")', "runtime-control Lua composition anchor")
    if any('include_str!("control_pause_resume.lua")' in text for text in control_split_texts):
        raise SystemExit(
            "post-closeout hardening regression: runtime-control Lua composition anchor "
            "must remain in control.rs for build.rs rewriting"
        )
    build_rs = read(BUILD_RS)
    workflow = read(WORKFLOW)
    spec = read(SPEC)
    todo = read(TODO)

    for command in ("pauseMame", "resumeMame", "resetMame", "setMameMute", "queryMameRuntimeState"):
        require(session_panel, command, "SessionControlPanel runtime command exposure")

    for label in ("Pause", "Resume", "Soft reset", "Mute", "Unmute", "Refresh runtime state"):
        require(session_panel, label, "SessionControlPanel visible runtime control")

    require(session_panel, "const canCommand = isRunningSession(session)", "running-session gate")
    require(session_panel, 'role="group"', "control grouping role")
    require(session_panel, 'aria-label="MAME runtime controls"', "control grouping label")
    require(session_panel, "Runtime:", "runtime-state display")

    # Gameplay-input ownership is now authoritative in MameShell. App must remain thin
    # composition and must not reintroduce the retired lifecycle-to-focus bridge.
    require(app, "<MameShell", "thin application composition")
    for obsolete in (
        "SESSION_STARTED_EVENT",
        "SESSION_EXITED_EVENT",
        "SESSION_CRASHED_EVENT",
        "SESSION_FAILED_EVENT",
        'window.dispatchEvent(new Event("focus"))',
    ):
        if obsolete in app:
            raise SystemExit(
                "post-closeout hardening regression: obsolete application lifecycle focus bridge "
                f"still contains {obsolete!r}"
            )

    allowed_event_name = re.compile(r"^[A-Za-z0-9/:_-]+$")
    frontend_event_names = re.findall(r'export const [A-Z0-9_]+_EVENT = "([^"]+)"', events_ts)
    backend_event_names = re.findall(r'pub const [A-Z0-9_]+_EVENT: &str = "([^"]+)"', event_names_rs)
    if not frontend_event_names or not backend_event_names:
        raise SystemExit("post-closeout hardening regression: centralized Tauri event contract is missing")
    for event_name in [*frontend_event_names, *backend_event_names]:
        if not allowed_event_name.fullmatch(event_name):
            raise SystemExit(f"post-closeout hardening regression: invalid Tauri event name {event_name!r}")
    if set(frontend_event_names) != set(backend_event_names):
        raise SystemExit("post-closeout hardening regression: Rust and TypeScript Tauri event contracts diverge")

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

    subprocess.run([sys.executable, str(MAME_UI_REGRESSION)], check=True, cwd=ROOT)

    print("post-closeout hardening regression passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
