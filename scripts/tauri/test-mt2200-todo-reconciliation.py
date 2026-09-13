#!/usr/bin/env python3
"""Regression guard for the MT-2200 TODO reconciliation contract."""

from __future__ import annotations

from pathlib import Path

TODO = Path("docs/MAME_TAURI_TODO_2026-09-08.md")
CLOSURE = Path("docs/MAME_TAURI_MT2200_ENGINEERING_CLOSURE_2026-09-13.md")
DEFERRED_PREFIX = "- [ ] **Deferred — optional research:**"
DEFERRED_MILESTONES = {"MT-1000", "MT-1100", "MT-1200"}
DEFERRED_SUBTASK = ("MT-1700", "MT-1705")


def fail(message: str) -> None:
    raise SystemExit(f"MT-2200 reconciliation regression: {message}")


def main() -> int:
    if not TODO.is_file():
        fail(f"missing {TODO}")
    if not CLOSURE.is_file():
        fail(f"missing {CLOSURE}")

    lines = TODO.read_text(encoding="utf-8").splitlines()
    major = ""
    minor = ""
    deferred = 0
    checked = 0

    for number, line in enumerate(lines, start=1):
        if line.startswith("# MT-"):
            major = line.split(" — ", 1)[0][2:].strip()
            minor = ""
        elif line.startswith("## MT-"):
            minor = line.split(" — ", 1)[0][3:].strip()

        if line.startswith("- [x]"):
            checked += 1
            if major in DEFERRED_MILESTONES or (major, minor) == DEFERRED_SUBTASK:
                fail(f"optional research was falsely checked at line {number}: {line}")
        elif line.startswith("- [ ]"):
            deferred += 1
            if not line.startswith(DEFERRED_PREFIX):
                fail(f"bare unchecked checkbox at line {number}: {line}")
            if major not in DEFERRED_MILESTONES and (major, minor) != DEFERRED_SUBTASK:
                fail(f"deferred marker outside approved research scope at line {number}: {line}")

    if checked == 0:
        fail("no completed checklist entries found")
    if deferred == 0:
        fail("optional research deferments disappeared")

    closure = CLOSURE.read_text(encoding="utf-8")
    required = (
        "MT-1000",
        "MT-1100",
        "MT-1200",
        "MT-1705",
        "external-window",
        "upstream-divergence audit",
        "Upstream synchronization procedure",
    )
    for token in required:
        if token not in closure:
            fail(f"closure handoff is missing required token {token!r}")

    print(
        f"MT-2200 TODO reconciliation passed: {checked} closed items, "
        f"{deferred} explicitly deferred research items, 0 ambiguous unchecked items"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
