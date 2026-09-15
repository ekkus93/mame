#!/usr/bin/env python3
"""Static regression coverage for the MAME-style Tauri UI reproduction milestone."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(relative: str) -> str:
    path = ROOT / relative
    if not path.is_file():
        raise SystemExit(f"MAME UI reproduction regression: missing {relative}")
    return path.read_text(encoding="utf-8")


def require(text: str, token: str, label: str) -> None:
    if token not in text:
        raise SystemExit(f"MAME UI reproduction regression: {label} missing {token!r}")


def forbid(text: str, token: str, label: str) -> None:
    if token in text:
        raise SystemExit(f"MAME UI reproduction regression: {label} still contains {token!r}")


def main() -> int:
    app = read("tauri/src/App.tsx")
    shell = read("tauri/src/shell/MameShell.tsx")
    shell_base_css = read("tauri/src/shell/MameShell.css")
    shell_css = read("tauri/src/shell/ContextualSurfaces.css")
    browser = read("tauri/src/browser/MameBrowser.tsx")
    filters = read("tauri/src/browser/MachineFilterPanel.tsx")
    machine_list = read("tauri/src/browser/MachineList.tsx")
    right_panel = read("tauri/src/browser/MachineRightPanel.tsx")
    software = read("tauri/src/browser/SoftwareBrowser.tsx")
    controller = read("tauri/src/settings/ControllerConfigurationPanel.tsx")
    session = read("tauri/src/session/SessionControlPanel.tsx")
    save_states = read("tauri/src/session/SaveStateBrowser.tsx")
    frontend_mame_ui = read("tauri/src/backend/mameUi.ts")
    export_backend = read("tauri/src-tauri/src/mame_ui_export.rs")
    workflow = read(".github/workflows/tauri-project.yml")
    spec = read("docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md")
    todo = read("docs/MAME_TAURI_MAME_UI_REPRODUCTION_TODO_2026-09-14.md")
    parity = read("docs/MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md")
    accessibility = read("docs/MAME_TAURI_MAME_UI_ACCESSIBILITY_QUALIFICATION_2026-09-15.md")
    closure = read("docs/MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md")

    require(app, "<MameShell", "thin application composition")
    for legacy_panel in (
        "GeneralSettingsPanel",
        "DiagnosticsPanel",
        "SessionControlPanel",
        "BulkAuditPanel",
        "LibraryBrowser",
        "RecentHistoryPanel",
        "CollectionManager",
    ):
        forbid(app, legacy_panel, "App.tsx legacy permanent panel stack")

    require(shell, "<MameBrowser", "primary machine browser")
    require(shell, "<GeneralSettingsPanel", "secondary Settings surface")
    require(shell, "<DiagnosticsPanel", "secondary Diagnostics surface")
    require(shell, "<SessionControlPanel", "secondary Session surface")
    require(shell, "<BulkAuditPanel", "secondary Audit surface")
    require(shell, "<RecentHistoryPanel", "secondary History surface")
    require(shell, "<CollectionManager", "secondary Collections surface")
    require(shell, "mame-session-status", "active-session shell status")
    forbid(shell, "LibraryBrowser", "legacy library composition")
    forbid(shell, '"legacy"', "legacy shell route")
    forbid(shell_base_css, "mame-legacy", "legacy dashboard-only CSS")

    for token in ("MachineFilterPanel", "MachineList", "MachineRightPanel", "SoftwareBrowser"):
        require(browser, token, "MAME browser composition")
    require(browser, 'aria-keyshortcuts="/"', "search shortcut semantics")
    require(browser, 'event.key === "ArrowLeft"', "left region navigation")
    require(browser, 'event.key === "ArrowRight"', "right region navigation")
    require(browser, "mame-narrow-details-toggle", "narrow-window details control")
    require(browser, "gameplayInputOwned", "fail-closed gameplay-input ownership")
    require(browser, "exportMameUiDisplayedList", "displayed-list export action")
    require(browser, ">Export<", "visible displayed-list export control")

    require(filters, 'role="listbox"', "filter list semantics")
    require(filters, 'role="option"', "filter option semantics")
    require(filters, 'event.key === "ArrowRight"', "filter-to-machine focus transition")
    require(machine_list, 'role="listbox"', "machine list semantics")
    require(machine_list, 'aria-selected={isSelected}', "machine selection semantics")

    require(right_panel, "Images", "Images mode")
    require(right_panel, "Info", "Info mode")
    require(right_panel, "MachineSettingsPanel", "contextual machine configuration")
    require(right_panel, "← Machine details", "configuration back path")
    require(controller, "Inherit global selection", "machine/global controller distinction")

    for token in (
        "queryMameSoftware",
        "queryMameBiosChoices",
        "Start Empty",
        "Launch part",
        "onBack",
        'event.key === "Escape"',
    ):
        require(software, token, "contextual software browser parity")

    for token in (
        "pauseMame",
        "resumeMame",
        "resetMame",
        "setMameMute",
        "stopMame",
        "SaveStateBrowser",
    ):
        require(session, token, "contextual session capability")
    for token in (
        "saveKnownState",
        "loadKnownSaveState",
        "deleteSaveStateRecord",
        "saveStateCompatibility",
    ):
        require(save_states, token, "contextual save-state capability")

    require(shell_css, ":focus-visible", "visible keyboard focus")
    require(shell_css, ".mame-browser-grid.show-details .mame-right-panel", "narrow details strategy")
    require(shell_css, "prefers-reduced-motion", "reduced-motion qualification")

    require(frontend_mame_ui, 'invoke<ExportMameUiDisplayedListResult>("export_mame_ui_displayed_list"', "typed export invocation")
    require(export_backend, "MAX_EXPORT_ROWS: u64 = 100_000", "bounded export row limit")
    require(export_backend, "blocking_save_file", "native export destination picker")
    require(export_backend, "tempfile_in(parent)", "atomic export staging")
    forbid(export_backend, "std::process::Command", "export generic process execution")

    for task_number in range(1, 21):
        task = f"MUI-{task_number:03d}"
        require(spec, task, "spec task inventory")
        require(todo, task, "TODO task inventory")
    forbid(todo, "- [ ]", "reconciled milestone TODO")
    require(todo, "MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md", "TODO parity-matrix link")
    require(todo, "MAME_TAURI_MAME_UI_ACCESSIBILITY_QUALIFICATION_2026-09-15.md", "TODO accessibility link")
    require(todo, "MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md", "TODO closure link")

    for token in ("Category", "Custom Filter", "DAT", "Software Favorites"):
        require(parity + closure, token, "explicit parity disposition")
    for token in (
        "Keyboard-only path",
        "Focus visibility and restoration",
        "Controller/gamepad navigation disposition",
    ):
        require(accessibility, token, "accessibility qualification")
    for token in (
        "Displayed-list export",
        "Deliberate machine-filter defers",
        "Deliberate software defers",
        "Security invariants preserved",
    ):
        require(closure, token, "closure record")

    for required_path in (
        "/docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md",
        "/docs/MAME_TAURI_MAME_UI_REPRODUCTION_TODO_2026-09-14.md",
        "/docs/MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md",
        "/docs/MAME_TAURI_MAME_UI_ACCESSIBILITY_QUALIFICATION_2026-09-15.md",
        "/docs/MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md",
    ):
        require(workflow, required_path, "MAME UI CI sparse checkout")

    print("MAME UI reproduction regression passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
