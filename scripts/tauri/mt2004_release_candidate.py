#!/usr/bin/env python3
"""MT-2004 external-window release-candidate manifest generator."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Sequence


SCHEMA_VERSION = 1
SHA_PATTERN = re.compile(r"^[0-9a-f]{40}$")


@dataclass(frozen=True)
class ReleaseArtifact:
    platform: str
    package_formats: list[str]
    workflow: str
    job: str
    required_smoke: list[str]


@dataclass(frozen=True)
class RequiredGate:
    gate_id: str
    workflow: str
    job: str
    purpose: str


@dataclass(frozen=True)
class KnownLimitation:
    limitation_id: str
    status: str
    user_impact: str
    mitigation: str


@dataclass(frozen=True)
class ReleaseCandidateManifest:
    schema_version: int
    generated_at: str
    repository: str
    candidate_sha: str
    release_candidate_type: str
    milestone: str
    acceptance: dict[str, str]
    supported_artifacts: list[ReleaseArtifact]
    required_gates: list[RequiredGate]
    known_limitations: list[KnownLimitation]


class ManifestError(RuntimeError):
    """Raised for user-correctable manifest input problems."""


def validate_sha(candidate_sha: str) -> str:
    if not SHA_PATTERN.fullmatch(candidate_sha):
        raise ManifestError(
            "candidate SHA must be a 40-character lowercase hexadecimal commit SHA"
        )
    return candidate_sha


def supported_artifacts() -> list[ReleaseArtifact]:
    return [
        ReleaseArtifact(
            platform="linux",
            package_formats=["Debian package", "AppImage"],
            workflow="Tauri Linux packaging",
            job="linux-deb-appimage-smoke",
            required_smoke=[
                "stage synthetic packaged MAME runtime",
                "validate bundled runtime paths",
                "build Debian package and AppImage",
                "install/uninstall Debian package",
                "execute AppImage package smoke",
                "verify lockfiles remain unchanged",
            ],
        ),
        ReleaseArtifact(
            platform="windows",
            package_formats=["NSIS installer"],
            workflow="Tauri Windows packaging",
            job="windows-nsis-smoke",
            required_smoke=[
                "stage synthetic packaged MAME runtime",
                "validate bundled runtime paths",
                "build NSIS package",
                "clean install and packaged-path smoke",
                "clean uninstall smoke",
                "verify lockfiles remain unchanged",
            ],
        ),
        ReleaseArtifact(
            platform="macos",
            package_formats=["app bundle", "DMG"],
            workflow="Tauri macOS packaging",
            job="macos-app-dmg-smoke",
            required_smoke=[
                "stage synthetic packaged MAME runtime",
                "validate bundled runtime paths",
                "reject missing Apple release credentials",
                "build ad-hoc signed app bundle and DMG",
                "run clean-runner app, DMG, resource, relocation, and signature smoke",
                "verify lockfiles remain unchanged",
            ],
        ),
    ]


def required_gates() -> list[RequiredGate]:
    return [
        RequiredGate(
            gate_id="mt2004-docs",
            workflow="Build documentation",
            job="build-docs",
            purpose="render release-candidate documentation to HTML and PDF",
        ),
        RequiredGate(
            gate_id="mt2004-linux-quality",
            workflow="Tauri project",
            job="linux-quality",
            purpose="run frontend, Rust, performance, license, MT-1800, and MT-2004 regressions",
        ),
        RequiredGate(
            gate_id="mt2004-linux-release-window",
            workflow="Tauri project",
            job="linux-release-qualification",
            purpose="build the production Tauri binary and smoke the development window under Xvfb",
        ),
        RequiredGate(
            gate_id="mt2004-linux-packaging",
            workflow="Tauri Linux packaging",
            job="linux-deb-appimage-smoke",
            purpose="produce and smoke Linux external-window release packages",
        ),
        RequiredGate(
            gate_id="mt2004-windows-packaging",
            workflow="Tauri Windows packaging",
            job="windows-nsis-smoke",
            purpose="produce and smoke the Windows external-window installer",
        ),
        RequiredGate(
            gate_id="mt2004-macos-packaging",
            workflow="Tauri macOS packaging",
            job="macos-app-dmg-smoke",
            purpose="produce and smoke macOS external-window app and DMG packages",
        ),
    ]


def known_limitations() -> list[KnownLimitation]:
    return [
        KnownLimitation(
            limitation_id="external-window-only",
            status="intentional MT-2004 boundary",
            user_impact="MAME runs as a supervised external process/window; embedded rendering is not claimed for this release candidate.",
            mitigation="Use the external-window frontend while embedded-rendering research remains optional/future work.",
        ),
        KnownLimitation(
            limitation_id="runtime-and-content-ownership",
            status="release-operator responsibility",
            user_impact="CI uses synthetic packaged-runtime fixtures; publishing with a real MAME executable requires staging reviewed runtime files and respecting MAME/content licenses.",
            mitigation="Use scripts/tauri/stage-mame-runtime.sh with a reviewed MAME source/runtime before publication; users supply their own ROM/software content.",
        ),
        KnownLimitation(
            limitation_id="macos-notarization-credentials",
            status="credential-gated",
            user_impact="CI proves the release script rejects missing Apple credentials and uses ad-hoc signing only for smoke tests.",
            mitigation="Provide real Apple signing/notarization credentials for a public macOS release.",
        ),
        KnownLimitation(
            limitation_id="physical-device-coverage",
            status="delegated/manual",
            user_impact="Controller hardware, host audio devices, and display/fullscreen behavior are not exhaustively proven by hosted CI.",
            mitigation="Run manual device smoke before broad user-facing distribution on target hardware.",
        ),
    ]


def build_manifest(candidate_sha: str) -> ReleaseCandidateManifest:
    return ReleaseCandidateManifest(
        schema_version=SCHEMA_VERSION,
        generated_at=datetime.now(timezone.utc).isoformat(timespec="seconds"),
        repository="ekkus93/mame",
        candidate_sha=validate_sha(candidate_sha),
        release_candidate_type="external-window",
        milestone="production-useful Tauri MAME frontend exists without embedded rendering",
        acceptance={
            "package_on_supported_platforms": "covered by Linux .deb/AppImage, Windows NSIS, and macOS app/DMG packaging workflows",
            "complete_smoke_test": "covered by package install/uninstall, runtime-path, app/DMG/resource/signature, production-build, and dev-window smoke gates",
            "document_known_limitations": "covered by known_limitations entries in this manifest and the MT-2004 release-candidate note",
            "record_exact_qualified_commit": "candidate_sha is the exact commit that must have all required_gates green",
        },
        supported_artifacts=supported_artifacts(),
        required_gates=required_gates(),
        known_limitations=known_limitations(),
    )


def manifest_to_json(manifest: ReleaseCandidateManifest, *, pretty: bool) -> str:
    return json.dumps(asdict(manifest), indent=2 if pretty else None, sort_keys=True) + "\n"


def manifest_to_markdown(manifest: ReleaseCandidateManifest) -> str:
    lines = [
        "# MT-2004 External-Window Release Candidate Manifest",
        "",
        f"- Repository: `{manifest.repository}`",
        f"- Candidate SHA: `{manifest.candidate_sha}`",
        f"- Type: `{manifest.release_candidate_type}`",
        f"- Milestone: {manifest.milestone}",
        "",
        "## Supported artifacts",
        "",
    ]
    for artifact in manifest.supported_artifacts:
        lines.append(
            f"- **{artifact.platform}:** {', '.join(artifact.package_formats)} "
            f"via `{artifact.workflow}` / `{artifact.job}`."
        )
    lines.extend(["", "## Required gates", ""])
    for gate in manifest.required_gates:
        lines.append(f"- `{gate.gate_id}` — `{gate.workflow}` / `{gate.job}`: {gate.purpose}")
    lines.extend(["", "## Known limitations", ""])
    for limitation in manifest.known_limitations:
        lines.append(
            f"- `{limitation.limitation_id}` — {limitation.status}: "
            f"{limitation.user_impact} Mitigation: {limitation.mitigation}"
        )
    lines.append("")
    return "\n".join(lines)


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sha", required=True, help="exact candidate commit SHA")
    parser.add_argument(
        "--format",
        choices=("json", "markdown"),
        default="json",
        help="manifest output format",
    )
    parser.add_argument("--write", help="write output to this path instead of stdout")
    parser.add_argument("--pretty", action="store_true", help="pretty-print JSON output")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        manifest = build_manifest(args.sha)
        if args.format == "json":
            rendered = manifest_to_json(manifest, pretty=args.pretty)
        else:
            rendered = manifest_to_markdown(manifest)

        if args.write:
            output_path = Path(args.write)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(rendered, encoding="utf-8")
        else:
            sys.stdout.write(rendered)
        return 0
    except ManifestError as exc:
        print(f"mt2004 release-candidate manifest error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
