#!/usr/bin/env python3
"""MT-1900 upstream-sustainability inventory helper.

The Tauri frontend is intentionally isolated from upstream MAME source.  This
helper classifies changed paths so an upstream-sync rehearsal can distinguish
project-owned files from patches that touch upstream-maintained MAME files.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import PurePosixPath
from typing import Iterable, Sequence


SCHEMA_VERSION = 1

PROJECT_OWNED_PREFIXES: tuple[str, ...] = (
    "tauri/",
    "scripts/tauri/",
    "tests/tauri/",
    "docs/MAME_TAURI",
    "docs/legal/MAME_TAURI",
)

PROJECT_OWNED_EXACT: frozenset[str] = frozenset(
    {
        ".github/dependabot.yml",
        ".github/workflows/docs.yml",
        ".github/workflows/tauri-project.yml",
        ".github/workflows/tauri-security.yml",
        ".github/workflows/tauri-linux-packaging.yml",
        ".github/workflows/tauri-macos-packaging.yml",
        ".github/workflows/tauri-windows-packaging.yml",
    }
)


@dataclass(frozen=True)
class ClassifiedPath:
    path: str
    classification: str
    reason: str


@dataclass(frozen=True)
class Inventory:
    schema_version: int
    generated_at: str
    upstream_remote: str
    fork_remote: str
    compare_base: str | None
    project_owned_count: int
    upstream_tree_touch_count: int
    project_owned_paths: list[ClassifiedPath]
    upstream_tree_touches: list[ClassifiedPath]
    recommended_validation: list[str]


class InventoryError(RuntimeError):
    """Raised for user-correctable inventory input problems."""


def normalize_path(path: str) -> str:
    candidate = path.strip().replace("\\", "/")
    if not candidate:
        raise InventoryError("empty paths are not valid inventory entries")
    pure = PurePosixPath(candidate)
    if pure.is_absolute():
        raise InventoryError(f"absolute paths are not valid inventory entries: {path!r}")
    if any(part in {"", ".", ".."} for part in pure.parts):
        raise InventoryError(f"ambiguous path segment in inventory entry: {path!r}")
    return pure.as_posix()


def is_project_owned(path: str) -> tuple[bool, str]:
    if path in PROJECT_OWNED_EXACT:
        return True, "project-owned exact file"
    for prefix in PROJECT_OWNED_PREFIXES:
        if path.startswith(prefix):
            return True, f"project-owned prefix {prefix}"
    return False, "upstream-maintained tree path"


def classify_paths(paths: Iterable[str]) -> Inventory:
    normalized = sorted({normalize_path(path) for path in paths})
    project_owned: list[ClassifiedPath] = []
    upstream_touches: list[ClassifiedPath] = []

    for path in normalized:
        owned, reason = is_project_owned(path)
        classified = ClassifiedPath(
            path=path,
            classification="project_owned" if owned else "upstream_tree_touch",
            reason=reason,
        )
        if owned:
            project_owned.append(classified)
        else:
            upstream_touches.append(classified)

    validation = [
        "Run scripts/tauri/test-mt1900-upstream-inventory.py.",
        "For project-owned-only changes, run the Tauri project workflow and docs workflow.",
        "For any upstream-tree touch, additionally run the appropriate native MAME build/test workflow.",
        "After integrating upstream, record base SHA, upstream SHA, fork SHA, conflicts, and exact green CI runs.",
    ]

    return Inventory(
        schema_version=SCHEMA_VERSION,
        generated_at=datetime.now(timezone.utc).isoformat(timespec="seconds"),
        upstream_remote="https://github.com/mamedev/mame.git",
        fork_remote="https://github.com/ekkus93/mame.git",
        compare_base=None,
        project_owned_count=len(project_owned),
        upstream_tree_touch_count=len(upstream_touches),
        project_owned_paths=project_owned,
        upstream_tree_touches=upstream_touches,
        recommended_validation=validation,
    )


def paths_from_file(path_file: str) -> list[str]:
    with open(path_file, encoding="utf-8") as handle:
        return [
            line.strip()
            for line in handle
            if line.strip() and not line.lstrip().startswith("#")
        ]


def paths_from_git(compare_base: str) -> list[str]:
    command = ["git", "diff", "--name-only", f"{compare_base}...HEAD"]
    completed = subprocess.run(
        command,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if completed.returncode != 0:
        raise InventoryError(completed.stderr.strip() or "git diff failed")
    return [line for line in completed.stdout.splitlines() if line.strip()]


def inventory_to_json(inventory: Inventory, *, pretty: bool) -> str:
    payload = asdict(inventory)
    indent = 2 if pretty else None
    return json.dumps(payload, indent=indent, sort_keys=True) + "\n"


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument(
        "--paths-file",
        help="newline-delimited changed-path file to classify",
    )
    source.add_argument(
        "--from-git",
        metavar="BASE",
        help="classify git diff --name-only BASE...HEAD",
    )
    parser.add_argument(
        "--write",
        help="write JSON inventory to this path instead of stdout",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="emit pretty-printed JSON",
    )
    parser.add_argument(
        "--expect-no-upstream-touches",
        action="store_true",
        help="fail when changed paths include upstream-maintained MAME files",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)

    try:
        if args.paths_file:
            paths = paths_from_file(args.paths_file)
            compare_base = None
        else:
            paths = paths_from_git(args.from_git)
            compare_base = args.from_git

        inventory = classify_paths(paths)
        inventory = Inventory(
            **{
                **asdict(inventory),
                "compare_base": compare_base,
                "project_owned_paths": inventory.project_owned_paths,
                "upstream_tree_touches": inventory.upstream_tree_touches,
            }
        )
        rendered = inventory_to_json(inventory, pretty=args.pretty)

        if args.write:
            with open(args.write, "w", encoding="utf-8") as handle:
                handle.write(rendered)
        else:
            sys.stdout.write(rendered)

        if args.expect_no_upstream_touches and inventory.upstream_tree_touches:
            return 2
        return 0
    except InventoryError as exc:
        print(f"mt1900 inventory error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
