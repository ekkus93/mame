#!/usr/bin/env python3
"""Generate deterministic MAME Tauri license inventory artifacts."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


REQUIRED_MAME_NOTICES = (
    "COPYING",
    "3rdparty/README.md",
    "docs/legal/GPL-2.0",
    "docs/legal/LGPL-2.1",
    "docs/legal/BSD-2-Clause",
    "docs/legal/BSD-3-Clause",
    "docs/legal/BSL-1.0",
    "docs/legal/CC0",
    "docs/legal/MIT",
    "docs/legal/Zlib",
)


def _load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _cargo_metadata(manifest_path: Path) -> dict[str, Any]:
    completed = subprocess.run(
        [
            "cargo",
            "metadata",
            "--locked",
            "--format-version",
            "1",
            "--manifest-path",
            str(manifest_path),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(completed.stdout)


def _resolved_rust_packages(metadata: dict[str, Any]) -> list[dict[str, Any]]:
    resolve = metadata.get("resolve") or {}
    resolved_ids = {node["id"] for node in resolve.get("nodes", [])}
    packages: list[dict[str, Any]] = []

    for package in metadata.get("packages", []):
        if resolved_ids and package.get("id") not in resolved_ids:
            continue

        license_expression = package.get("license")
        license_file = package.get("license_file")
        if not license_expression and not license_file:
            raise RuntimeError(
                f"Rust package {package.get('name')} {package.get('version')} "
                "does not declare license metadata"
            )

        packages.append(
            {
                "name": package["name"],
                "version": package["version"],
                "license": license_expression,
                "licenseFile": Path(license_file).name if license_file else None,
                "source": package.get("source"),
                "repository": package.get("repository"),
                "links": package.get("links"),
            }
        )

    return sorted(packages, key=lambda package: (package["name"], package["version"]))


def _javascript_packages(package_lock: dict[str, Any]) -> list[dict[str, Any]]:
    packages: list[dict[str, Any]] = []
    for path, package in package_lock.get("packages", {}).items():
        if not path or "version" not in package:
            continue

        license_expression = package.get("license")
        if not license_expression:
            raise RuntimeError(
                f"JavaScript package {path} {package.get('version')} "
                "does not declare license metadata in package-lock.json"
            )

        packages.append(
            {
                "name": path.removeprefix("node_modules/"),
                "version": package["version"],
                "license": license_expression,
                "resolved": package.get("resolved"),
                "dev": bool(package.get("dev", False)),
                "optional": bool(package.get("optional", False)),
            }
        )

    return sorted(packages, key=lambda package: (package["name"], package["version"]))


def _mame_notices(repo_root: Path) -> list[dict[str, str]]:
    notices: list[dict[str, str]] = []
    for relative in REQUIRED_MAME_NOTICES:
        path = repo_root / relative
        if not path.is_file():
            raise RuntimeError(f"Required MAME legal notice is missing: {relative}")
        notices.append({"path": relative})
    return notices


def build_inventory(
    repo_root: Path,
    cargo_metadata: dict[str, Any],
    package_lock: dict[str, Any],
) -> dict[str, Any]:
    rust_packages = _resolved_rust_packages(cargo_metadata)
    javascript_packages = _javascript_packages(package_lock)
    native_linked = [package for package in rust_packages if package["links"]]

    return {
        "schemaVersion": 1,
        "mame": {
            "primaryLicense": "GPL-2.0",
            "notices": _mame_notices(repo_root),
            "redistributedRuntime": {
                "component": "MAME executable and runtime resources",
                "license": "GPL-2.0 plus component-specific licenses recorded by MAME",
                "notice": "COPYING",
            },
        },
        "rust": {
            "manifest": "tauri/src-tauri/Cargo.toml",
            "lockfile": "tauri/src-tauri/Cargo.lock",
            "packages": rust_packages,
        },
        "javascript": {
            "manifest": "tauri/package.json",
            "lockfile": "tauri/package-lock.json",
            "packages": javascript_packages,
        },
        "nativeLibraries": {
            "cargoLinkedPackages": native_linked,
            "platformPolicy": [
                {
                    "platform": "linux",
                    "policy": "System WebKit/AppIndicator dependencies are not project-owned; AppImage contents must be reviewed from the built package artifact.",
                },
                {
                    "platform": "windows",
                    "policy": "WebView2 is a platform runtime and is not treated as project-owned source; bundled MAME remains covered by the MAME notices.",
                },
                {
                    "platform": "macos",
                    "policy": "System WebKit frameworks are platform-provided; bundled MAME remains covered by the MAME notices.",
                },
            ],
        },
    }


def _render_markdown(inventory: dict[str, Any]) -> str:
    lines = [
        "# MAME Tauri generated license inventory",
        "",
        "> Generated by `scripts/tauri/generate-license-inventory.py`. Do not edit by hand.",
        "",
        "## MAME redistribution notices",
        "",
        f"Primary project license: `{inventory['mame']['primaryLicense']}`.",
        "",
    ]
    lines.extend(f"- `{notice['path']}`" for notice in inventory["mame"]["notices"])

    lines.extend(
        [
            "",
            "## Rust dependency closure",
            "",
            "| Package | Version | License | Native link target |",
            "| --- | --- | --- | --- |",
        ]
    )
    for package in inventory["rust"]["packages"]:
        license_name = package["license"] or f"license-file:{package['licenseFile']}"
        lines.append(
            f"| `{package['name']}` | `{package['version']}` | "
            f"`{license_name}` | `{package['links'] or ''}` |"
        )

    lines.extend(
        [
            "",
            "## JavaScript dependency closure",
            "",
            "| Package | Version | License | Dev | Optional |",
            "| --- | --- | --- | --- | --- |",
        ]
    )
    for package in inventory["javascript"]["packages"]:
        lines.append(
            f"| `{package['name']}` | `{package['version']}` | "
            f"`{package['license']}` | `{str(package['dev']).lower()}` | "
            f"`{str(package['optional']).lower()}` |"
        )

    lines.extend(
        [
            "",
            "## Redistributed/native-library policy",
            "",
            "The bundled MAME executable/runtime is governed by MAME's `COPYING` notice and component-specific legal notices. Native Cargo packages that declare a `links` target are listed in the Rust table above and in the JSON artifact under `nativeLibraries.cargoLinkedPackages`.",
            "",
        ]
    )
    for policy in inventory["nativeLibraries"]["platformPolicy"]:
        lines.append(f"- **{policy['platform']}**: {policy['policy']}")

    lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[2],
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=None,
        help="Defaults to <repo>/artifacts/license-inventory",
    )
    parser.add_argument(
        "--cargo-metadata-json",
        type=Path,
        default=None,
        help="Use pre-captured cargo metadata instead of invoking cargo (test/debug helper).",
    )
    args = parser.parse_args()

    repo_root = args.repo_root.resolve()
    output_dir = (args.output_dir or repo_root / "artifacts/license-inventory").resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    cargo_metadata = (
        _load_json(args.cargo_metadata_json)
        if args.cargo_metadata_json
        else _cargo_metadata(repo_root / "tauri/src-tauri/Cargo.toml")
    )
    package_lock = _load_json(repo_root / "tauri/package-lock.json")
    inventory = build_inventory(repo_root, cargo_metadata, package_lock)

    json_path = output_dir / "mame-tauri-license-inventory.json"
    markdown_path = output_dir / "mame-tauri-license-inventory.md"
    json_path.write_text(
        json.dumps(inventory, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    markdown_path.write_text(_render_markdown(inventory), encoding="utf-8")

    print(json_path)
    print(markdown_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
