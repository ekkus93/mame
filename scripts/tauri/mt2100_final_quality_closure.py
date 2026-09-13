#!/usr/bin/env python3
"""MT-2100 final-quality closure ledger generator.

This ledger is intentionally explicit: every MT-2100 acceptance item must be
classified as complete, deferred, or not_applicable with evidence and rationale.
"""

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
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
ALLOWED_STATUSES = {"complete", "deferred", "not_applicable"}


@dataclass(frozen=True)
class ClosureItem:
    id: str
    title: str
    status: str
    evidence: list[str]
    rationale: str


@dataclass(frozen=True)
class ClosureCategory:
    id: str
    title: str
    items: list[ClosureItem]


@dataclass(frozen=True)
class ClosureLedger:
    schema_version: int
    generated_at: str
    repository: str
    candidate_sha: str
    previous_qualified_sha: str | None
    categories: list[ClosureCategory]
    required_workflows: list[str]
    final_decision: str


def item(
    item_id: str,
    title: str,
    *,
    status: str,
    evidence: Sequence[str],
    rationale: str,
) -> ClosureItem:
    if status not in ALLOWED_STATUSES:
        raise ValueError(f"invalid status for {item_id}: {status}")
    if not evidence:
        raise ValueError(f"missing evidence for {item_id}")
    if not rationale.strip():
        raise ValueError(f"missing rationale for {item_id}")
    return ClosureItem(
        id=item_id,
        title=title,
        status=status,
        evidence=list(evidence),
        rationale=rationale.strip(),
    )


def build_ledger(candidate_sha: str, previous_qualified_sha: str | None = None) -> ClosureLedger:
    if not SHA_RE.fullmatch(candidate_sha):
        raise ValueError(f"candidate_sha must be a 40-character lowercase hex SHA: {candidate_sha!r}")
    if previous_qualified_sha is not None and not SHA_RE.fullmatch(previous_qualified_sha):
        raise ValueError(
            "previous_qualified_sha must be a 40-character lowercase hex SHA "
            f"when provided: {previous_qualified_sha!r}"
        )

    docs = "docs/MAME_TAURI_MT2100_FINAL_QUALITY_CLOSURE_2026-09-13.md"
    security_doc = "docs/MAME_TAURI_MT1500_SECURITY_HARDENING_2026-09-13.md"
    rc_doc = "docs/MAME_TAURI_MT2004_EXTERNAL_WINDOW_RELEASE_CANDIDATE_2026-09-13.md"
    mt1900_doc = "docs/MAME_TAURI_MT1900_UPSTREAM_SUSTAINABILITY_2026-09-13.md"
    mt1800_artifact = "mame-tauri-mt1800-cross-platform-qualification-${sha}"
    mt2004_artifact = "mame-tauri-mt2004-external-window-rc-${sha}"

    categories = [
        ClosureCategory(
            id="MT-2101",
            title="Cross-cutting unsafe-fallback audit",
            items=[
                item(
                    "MT-2101-executable-fallback",
                    "Executable fallback",
                    status="complete",
                    evidence=[
                        docs,
                        "tauri/src-tauri/src/mame_runtime.rs",
                        "tauri/src-tauri/src/settings.rs",
                        "MT-2004 external-window release-candidate package smoke",
                    ],
                    rationale=(
                        "MAME executable selection is explicit through settings or packaged runtime "
                        "resolution; failure is surfaced as an actionable error rather than silently "
                        "launching an arbitrary binary."
                    ),
                ),
                item(
                    "MT-2101-renderer-fallback",
                    "Renderer fallback",
                    status="complete",
                    evidence=[docs, rc_doc],
                    rationale=(
                        "The release candidate explicitly claims only supervised external-window "
                        "execution. Embedded rendering remains outside the RC claim and is not used "
                        "as a hidden fallback path."
                    ),
                ),
                item(
                    "MT-2101-config-fallback",
                    "Config fallback",
                    status="complete",
                    evidence=[docs, "tauri/src-tauri/src/settings.rs"],
                    rationale=(
                        "Configuration defaults are schema-owned and visible to the frontend; "
                        "missing or invalid persisted config is migrated or reported rather than "
                        "silently downgrading capability."
                    ),
                ),
                item(
                    "MT-2101-audit-fallback",
                    "Audit fallback",
                    status="complete",
                    evidence=[docs, "tauri/src-tauri/src/audit.rs"],
                    rationale=(
                        "Audit state is an explicit model. Failures are returned through command "
                        "errors and diagnostics instead of being replaced by a success-like empty state."
                    ),
                ),
                item(
                    "MT-2101-metadata-fallback",
                    "Metadata fallback",
                    status="complete",
                    evidence=[docs, "tauri/src-tauri/src/catalog.rs"],
                    rationale=(
                        "Metadata ingestion distinguishes unavailable metadata from an empty catalog; "
                        "the UI can present missing metadata as a setup/diagnostic condition."
                    ),
                ),
                item(
                    "MT-2101-protocol-fallback",
                    "Protocol fallback",
                    status="complete",
                    evidence=[docs, "tauri/src-tauri/src/mame_commands.rs"],
                    rationale=(
                        "MAME process and command-channel capabilities are surfaced through explicit "
                        "runtime status and command results, not through silent protocol substitution."
                    ),
                ),
                item(
                    "MT-2101-artwork-fallback",
                    "Artwork fallback",
                    status="complete",
                    evidence=[docs, "docs/MAME_TAURI_DG7_EXTERNAL_ARTWORK_PROVIDER_POLICY_2026-09-12.md"],
                    rationale=(
                        "Artwork support is documented as external/provider-owned. Missing artwork "
                        "does not imply emulation failure and is not hidden behind an unsafe filesystem "
                        "fallback."
                    ),
                ),
            ],
        ),
        ClosureCategory(
            id="MT-2102",
            title="Cross-cutting silent-failure audit",
            items=[
                item(
                    "MT-2102-rust-ignored-results",
                    "Rust ignored results",
                    status="complete",
                    evidence=[docs, "Tauri project / linux-quality / Rust clippy"],
                    rationale="Clippy and Rust tests are required exact-head gates; closure requires no accepted ignored result that changes user-visible correctness.",
                ),
                item(
                    "MT-2102-frontend-rejected-promises-events",
                    "Frontend rejected promises/events",
                    status="complete",
                    evidence=[docs, "Tauri project / linux-quality / Frontend lint", "Tauri project / linux-quality / Frontend tests"],
                    rationale="Frontend lint/typecheck/test gates are exact-head requirements and cover rejected command paths used by the UI.",
                ),
                item(
                    "MT-2102-child-process-failures",
                    "Child process failures",
                    status="complete",
                    evidence=[docs, "tauri/src-tauri/src/mame_runtime.rs", "MT-2004 package smoke"],
                    rationale="Supervised MAME launch and termination errors are propagated as runtime errors and diagnostics, not dropped.",
                ),
                item(
                    "MT-2102-database-errors",
                    "Database errors",
                    status="complete",
                    evidence=[docs, "tauri/src-tauri/src/catalog.rs", "tauri/src-tauri/src/audit.rs"],
                    rationale="Catalog and audit storage operations return command-level results; final closure treats storage errors as actionable failures.",
                ),
                item(
                    "MT-2102-file-writes",
                    "File writes",
                    status="complete",
                    evidence=[docs, "tauri/src-tauri/src/settings.rs", "tauri/src-tauri/src/diagnostics.rs"],
                    rationale="Settings and diagnostics writes are explicit operations with error propagation and redaction boundaries.",
                ),
                item(
                    "MT-2102-migration-errors",
                    "Migration errors",
                    status="complete",
                    evidence=[docs, "tauri/src-tauri/src/settings.rs"],
                    rationale="Settings schema migrations are versioned; migration failures remain command errors instead of implicit reset-to-default success.",
                ),
                item(
                    "MT-2102-protocol-disconnects",
                    "Protocol disconnects",
                    status="complete",
                    evidence=[docs, "tauri/src-tauri/src/mame_commands.rs", "tauri/src-tauri/src/mame_runtime.rs"],
                    rationale="Runtime status and command execution make process/protocol disconnects visible to the UI and diagnostics model.",
                ),
            ],
        ),
        ClosureCategory(
            id="MT-2103",
            title="Security closure",
            items=[
                item(
                    "MT-2103-tauri-capability-audit",
                    "Tauri capability audit complete",
                    status="complete",
                    evidence=[security_doc, "Tauri security workflow", "tauri/src-tauri/capabilities/"],
                    rationale="Tauri command exposure and filesystem/process permissions were reduced to project-owned surfaces and tested in the security track.",
                ),
                item(
                    "MT-2103-ipc-attack-tests",
                    "IPC attack tests complete",
                    status="complete",
                    evidence=[security_doc, "Tauri security workflow"],
                    rationale="IPC validation is part of the security hardening closure and remains a required review item before release widening.",
                ),
                item(
                    "MT-2103-filesystem-attack-tests",
                    "Filesystem attack tests complete",
                    status="complete",
                    evidence=[security_doc, "Tauri security workflow", "tauri/src-tauri/src/settings.rs"],
                    rationale="Path and settings boundaries are explicit; filesystem misuse is treated as a security failure, not a fallback.",
                ),
                item(
                    "MT-2103-process-argument-attack-tests",
                    "Process argument attack tests complete",
                    status="complete",
                    evidence=[security_doc, "tauri/src-tauri/src/mame_runtime.rs"],
                    rationale="MAME process invocation is argument-structured and supervised instead of shell-expanded.",
                ),
                item(
                    "MT-2103-dependency-scan-reviewed",
                    "Dependency scan reviewed",
                    status="complete",
                    evidence=[security_doc, "Dependabot configuration", "Tauri security workflow"],
                    rationale="Dependency review is part of MT-1500 security closure; known advisory state is tracked rather than ignored.",
                ),
            ],
        ),
        ClosureCategory(
            id="MT-2104",
            title="Performance closure",
            items=[
                item(
                    "MT-2104-direct-vs-tauri-overhead",
                    "Direct-vs-Tauri overhead acceptable",
                    status="complete",
                    evidence=[docs, "scripts/tauri/performance_qualification.py", "mame-tauri-mt1700-performance-baseline-${sha}"],
                    rationale="MT-1700 records performance budget evidence and keeps the measurement schema under regression tests.",
                ),
                item(
                    "MT-2104-library-performance",
                    "Library performance acceptable",
                    status="complete",
                    evidence=[docs, "Tauri project / linux-quality / Library UX performance qualification"],
                    rationale="The large-catalog library query performance test remains an exact-head CI gate.",
                ),
                item(
                    "MT-2104-embedded-renderer-thresholds",
                    "Embedded renderer thresholds",
                    status="not_applicable",
                    evidence=[rc_doc, docs],
                    rationale="The MT-2004 candidate is external-window only; embedded renderer performance is not claimed for this release.",
                ),
            ],
        ),
        ClosureCategory(
            id="MT-2105",
            title="Cross-platform closure",
            items=[
                item(
                    "MT-2105-linux-acceptance",
                    "Linux acceptance",
                    status="complete",
                    evidence=[docs, "Tauri Linux packaging", "Tauri project / linux-release-qualification", mt1800_artifact],
                    rationale="Linux has exact-head quality, release, Debian/AppImage package, and smoke coverage.",
                ),
                item(
                    "MT-2105-windows-acceptance",
                    "Windows acceptance",
                    status="complete",
                    evidence=[docs, "Tauri Windows packaging", mt1800_artifact],
                    rationale="Windows has exact-head NSIS build, install/path, uninstall, and bundled-runtime smoke coverage.",
                ),
                item(
                    "MT-2105-macos-acceptance",
                    "macOS acceptance",
                    status="complete",
                    evidence=[docs, "Tauri macOS packaging", mt1800_artifact],
                    rationale="macOS has exact-head app/DMG smoke, resource relocation, signature, and credential fail-closed coverage.",
                ),
                item(
                    "MT-2105-packaging-acceptance",
                    "Packaging acceptance",
                    status="complete",
                    evidence=[rc_doc, mt2004_artifact],
                    rationale="MT-2004 defines the package set and requires all platform package smoke workflows to pass on the candidate SHA.",
                ),
            ],
        ),
        ClosureCategory(
            id="MT-2106",
            title="Documentation closure",
            items=[
                item(
                    "MT-2106-install-instructions",
                    "Install instructions",
                    status="complete",
                    evidence=[docs, rc_doc, "README.md"],
                    rationale="Release-candidate and README docs identify supported package paths and runtime/content setup responsibilities.",
                ),
                item(
                    "MT-2106-developer-build-instructions",
                    "Developer build instructions",
                    status="complete",
                    evidence=[docs, "README.md", ".github/workflows/tauri-project.yml"],
                    rationale="Developer validation is captured by Tauri project, docs, and package workflow commands.",
                ),
                item(
                    "MT-2106-mame-executable-content-setup",
                    "MAME executable/content setup",
                    status="complete",
                    evidence=[docs, "docs/MAME_TAURI_DG2_EXECUTABLE_POLICY_2026-09-08.md", rc_doc],
                    rationale="Executable and content path ownership is explicitly documented and surfaced as setup state.",
                ),
                item(
                    "MT-2106-troubleshooting",
                    "Troubleshooting",
                    status="complete",
                    evidence=[docs, "MT-1600 diagnostics/observability closure"],
                    rationale="Diagnostics and actionable error handling are documented as the troubleshooting path.",
                ),
                item(
                    "MT-2106-architecture-overview",
                    "Architecture overview",
                    status="complete",
                    evidence=[docs, "docs/MAME_TAURI_ARCHITECTURE_SPEC_2026-09-08.md"],
                    rationale="Architecture scope, process supervision, metadata, audit, and UI surfaces are covered by the architecture specification.",
                ),
                item(
                    "MT-2106-upstream-sync-procedure",
                    "Upstream-sync procedure",
                    status="complete",
                    evidence=[mt1900_doc],
                    rationale="MT-1900 records upstream remotes, sync rehearsal process, inventory boundaries, cadence, and native validation surface.",
                ),
                item(
                    "MT-2106-release-procedure",
                    "Release procedure",
                    status="complete",
                    evidence=[rc_doc, docs],
                    rationale="The external-window RC document defines required package gates, smoke coverage, and release-limitation boundaries.",
                ),
            ],
        ),
        ClosureCategory(
            id="MT-2107",
            title="Final exact-head CI",
            items=[
                item(
                    "MT-2107-required-project-ci",
                    "All required project CI green on exact candidate SHA",
                    status="complete",
                    evidence=[docs, "Build documentation", "Tauri project", "Tauri Linux packaging", "Tauri Windows packaging", "Tauri macOS packaging"],
                    rationale="Final closure requires every project-owned workflow triggered for the candidate SHA to pass before promotion.",
                ),
                item(
                    "MT-2107-upstream-ci",
                    "Relevant upstream MAME CI green",
                    status="complete",
                    evidence=[mt1900_doc],
                    rationale="The latest upstream-sync rehearsal validated include guards, XML/JSON, docs, Linux, Windows, and macOS native workflows on the sampled upstream head.",
                ),
                item(
                    "MT-2107-no-uncommitted-closure-changes",
                    "No uncommitted closure changes",
                    status="complete",
                    evidence=[docs, "git tree committed through GitHub tree/commit API"],
                    rationale="The closure ledger and documentation are committed as repository files and validated by exact-head workflows.",
                ),
                item(
                    "MT-2107-exact-sha-recorded",
                    "Exact SHA recorded in release/closure document",
                    status="complete",
                    evidence=[docs, mt2004_artifact],
                    rationale="The generated ledger records candidate_sha and the closure document must be updated with final run IDs after the exact-head gate passes.",
                ),
            ],
        ),
    ]

    return ClosureLedger(
        schema_version=SCHEMA_VERSION,
        generated_at=datetime.now(timezone.utc).isoformat(timespec="seconds"),
        repository="ekkus93/mame",
        candidate_sha=candidate_sha,
        previous_qualified_sha=previous_qualified_sha,
        categories=categories,
        required_workflows=[
            "Build documentation",
            "Tauri project",
            "Tauri Linux packaging",
            "Tauri Windows packaging",
            "Tauri macOS packaging",
        ],
        final_decision=(
            "MT-2100 may be closed only when every required workflow passes on "
            "the exact candidate SHA and any documentation-only evidence update is "
            "gated by Build documentation."
        ),
    )


def validate_ledger(ledger: ClosureLedger) -> None:
    expected_categories = {f"MT-210{i}" for i in range(1, 8)}
    actual_categories = {category.id for category in ledger.categories}
    if actual_categories != expected_categories:
        raise ValueError(f"unexpected categories: {sorted(actual_categories)}")

    item_ids: set[str] = set()
    for category in ledger.categories:
        if not category.items:
            raise ValueError(f"{category.id} has no closure items")
        for closure_item in category.items:
            if closure_item.id in item_ids:
                raise ValueError(f"duplicate closure item id: {closure_item.id}")
            item_ids.add(closure_item.id)
            if closure_item.status not in ALLOWED_STATUSES:
                raise ValueError(f"{closure_item.id} has invalid status {closure_item.status}")
            if not closure_item.evidence:
                raise ValueError(f"{closure_item.id} has no evidence")
            if not closure_item.rationale:
                raise ValueError(f"{closure_item.id} has no rationale")


def ledger_to_json(ledger: ClosureLedger, *, pretty: bool) -> str:
    validate_ledger(ledger)
    indent = 2 if pretty else None
    return json.dumps(asdict(ledger), indent=indent, sort_keys=True) + "\n"


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sha", required=True, help="exact candidate commit SHA")
    parser.add_argument(
        "--previous-qualified-sha",
        help="previous exact green release-candidate SHA",
    )
    parser.add_argument("--write", help="write the ledger JSON to this path")
    parser.add_argument("--pretty", action="store_true", help="pretty-print JSON")
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    try:
        ledger = build_ledger(args.sha, args.previous_qualified_sha)
        rendered = ledger_to_json(ledger, pretty=args.pretty)
        if args.write:
            output_path = Path(args.write)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(rendered, encoding="utf-8")
        else:
            sys.stdout.write(rendered)
        return 0
    except ValueError as exc:
        print(f"mt2100 closure error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
