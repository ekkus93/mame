#!/usr/bin/env python3
"""Static regression checks for the privileged Tauri security boundary."""

from __future__ import annotations

import json
import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TAURI_ROOT = REPO_ROOT / "tauri"
EXPECTED_CSP = (
    "default-src 'self'; "
    "connect-src 'self' ipc: http://ipc.localhost ws://localhost:1420; "
    "img-src 'self' data:; "
    "style-src 'self' 'unsafe-inline'"
)
FORBIDDEN_JS_PLUGINS = (
    "@tauri-apps/plugin-shell",
    "@tauri-apps/plugin-fs",
    "@tauri-apps/plugin-http",
    "@tauri-apps/plugin-opener",
)
FORBIDDEN_RUST_PLUGINS = (
    "tauri-plugin-shell",
    "tauri-plugin-fs",
    "tauri-plugin-http",
    "tauri-plugin-opener",
)


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def main() -> int:
    capability = load_json(TAURI_ROOT / "src-tauri/capabilities/default.json")
    require(
        capability.get("windows") == ["main"],
        "default Tauri capability must remain scoped only to the main window",
    )
    require(
        capability.get("permissions") == ["core:default"],
        "default Tauri capability must expose only core:default",
    )

    tauri_config = load_json(TAURI_ROOT / "src-tauri/tauri.conf.json")
    build = tauri_config.get("build", {})
    require(
        build.get("frontendDist") == "../dist",
        "production WebView content must remain the local bundled frontend",
    )
    require(
        build.get("devUrl") == "http://localhost:1420",
        "development WebView content must remain the fixed local Vite origin",
    )
    for window in tauri_config.get("app", {}).get("windows", []):
        require(
            "url" not in window,
            "privileged application windows must not navigate to arbitrary remote content",
        )

    csp = tauri_config.get("app", {}).get("security", {}).get("csp")
    require(
        csp == EXPECTED_CSP,
        f"privileged WebView CSP changed and requires security review: {csp!r}",
    )

    package = load_json(TAURI_ROOT / "package.json")
    javascript_dependencies = {
        **package.get("dependencies", {}),
        **package.get("devDependencies", {}),
    }
    for dependency in FORBIDDEN_JS_PLUGINS:
        require(
            dependency not in javascript_dependencies,
            f"privileged Tauri JavaScript plugin requires explicit security review: {dependency}",
        )

    cargo_toml = (TAURI_ROOT / "src-tauri/Cargo.toml").read_text(encoding="utf-8")
    for dependency in FORBIDDEN_RUST_PLUGINS:
        require(
            re.search(rf"(?m)^\s*{re.escape(dependency)}\s*=", cargo_toml) is None,
            f"privileged Tauri Rust plugin requires explicit security review: {dependency}",
        )

    for lockfile in (
        TAURI_ROOT / "package-lock.json",
        TAURI_ROOT / "src-tauri/Cargo.lock",
    ):
        require(
            lockfile.is_file(),
            f"required dependency lockfile is missing: {lockfile.relative_to(REPO_ROOT)}",
        )

    require(
        re.search(r'(?m)^license\s*=\s*"GPL-2\.0-only"\s*$', cargo_toml) is not None,
        "project Rust crate must retain explicit GPL-2.0-only package metadata",
    )

    control_rs = "\n".join(
        (TAURI_ROOT / "src-tauri/src/sessions" / path).read_text(encoding="utf-8")
        for path in (
            "control.rs",
            "control_registry.rs",
            "control_bootstrap.rs",
            "control_channel.rs",
            "control_requests.rs",
            "control_correlation.rs",
            "control_parser.rs",
            "control_events.rs",
            "control_protocol.rs",
        )
    )
    require(
        "std::fs::Permissions::from_mode(0o600)" in control_rs,
        "runtime-control bootstrap files must remain private on Unix",
    )
    hardening_spec = (
        REPO_ROOT / "docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_SPEC_2026-09-13.md"
    ).read_text(encoding="utf-8")
    require(
        "TEMP directory ACLs" in hardening_spec
        and "per-session frame-token entropy" in hardening_spec
        and "RAII cleanup" in hardening_spec,
        "non-Unix runtime-control bootstrap security assumptions must stay documented",
    )

    print("MT-1500 static security policy regression passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
