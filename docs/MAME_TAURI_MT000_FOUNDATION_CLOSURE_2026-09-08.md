# MAME Tauri MT-000 Foundation Closure

**Date:** 2026-09-08  
**Branch:** `ralph/mt-000-foundations`  
**Planning baseline:** `7cc3033a50b00801240f020b7098e22ffffdcd44`  
**Master at audit start:** `c45b774487a97423a97e84a9a84f388752d1da8e`

## Result

MT-000 foundation work is complete subject to merge of this branch.

## MT-001 — Freeze project architecture baseline

- [x] Reviewed `MAME_TAURI_ARCHITECTURE_SPEC_2026-09-08.md` against the current fork state.
- [x] Compared the planning baseline with audit-start `master`.
- [x] Confirmed the only fork changes were the architecture specification and TODO documents; no MAME executable source/build implementation had drifted.
- [x] Confirmed sidecar-first architecture.
- [x] Confirmed native video/audio/gameplay-input hot paths.
- [x] Confirmed embedded rendering, dedicated OSD, and in-process hosting remain separately gated optional tracks.

**Acceptance:** satisfied. No implementation-level architecture correction was required.

## MT-002 — Define branch and merge policy

Completed in `docs/MAME_TAURI_DEVELOPMENT_POLICY_2026-09-08.md`:

- [x] feature/Ralph branch naming convention;
- [x] exact-head CI requirement for executable changes;
- [x] upstream synchronization procedure;
- [x] deliberate documentation-only direct-commit exception;
- [x] merged/stale branch cleanup policy.

**Acceptance:** satisfied.

## MT-003 — Protect project-owned namespaces / DG-1

Decision recorded in `docs/MAME_TAURI_DG1_REPOSITORY_LAYOUT_2026-09-08.md`.

Final layout:

```text
tauri/                     self-contained React/Tauri workspace
scripts/tauri/             project integration tooling
tests/tauri/               repository-level project fixtures when needed
docs/MAME_TAURI_*          project architecture/operations/evidence
src/osd/tauri/             reserved only for an approved future OSD phase
```

Within `tauri/`:

```text
tauri/src/                 React + TypeScript frontend
tauri/src-tauri/           Rust + Tauri backend
```

- [x] frontend directory selected;
- [x] backend directory selected;
- [x] tooling/test namespaces reserved;
- [x] collision with upstream `src/frontend` avoided.

**DG-1:** accepted.

## MT-004 — Establish project coding standards

Completed in the development policy:

- [x] Rust format/lint/test policy;
- [x] TypeScript strict/lint/format policy;
- [x] React conventions;
- [x] stable error-envelope convention;
- [x] serialization naming convention;
- [x] test naming/placement convention;
- [x] MAME C/C++ style/minimal-churn rule.

## MT-005 — Establish licensing/trademark constraints

Completed in the development policy using repository `COPYING`/README as the engineering baseline:

- [x] MAME whole-project GPLv2 distribution constraint recorded;
- [x] binary/resource redistribution notice/source-obligation review requirement recorded;
- [x] Rust/npm/native dependency inventory requirement recorded;
- [x] MAME registered-trademark/name/logo constraint recorded;
- [x] copyrighted ROM/software bundling explicitly prohibited by project policy.

A concrete shipped-artifact license inventory remains an MT-1307 release task; MT-005 establishes the policy, not the future artifact inventory.

## Validation

This branch changes documentation/policy only. Under the project development policy, executable CI is not required for this closure. Validation consists of repository-state comparison, source-document review, path-collision review, and cross-reference review.

No MAME source, Tauri executable code, workflow, or build configuration is changed by MT-000.