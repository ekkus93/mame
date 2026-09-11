# MAME Tauri Backlog Reconciliation — MT-1400, MT-2000, VS-1

**Date:** 2026-09-11  
**Baseline SHA:** `1448fc0f44c0830b2d8a41f88cdb84aaeb162e7f`

This audit separates stale unchecked planning items from genuine remaining product work. Completion is credited only where current executable code and regression/CI evidence satisfy the stated requirement.

## MT-1400 — CI and automated validation

### MT-1401 — project-specific CI workflow

`.github/workflows/tauri-project.yml` provides the full project quality gate:

- `npm run format:check`;
- frontend lint;
- TypeScript typecheck;
- frontend tests;
- production frontend build;
- `cargo fmt --check`;
- `cargo test --locked`;
- Clippy with `-D warnings`;
- lockfile drift protection.

It also has a release-qualification job with a production Tauri build smoke check and an Xvfb development-window smoke check.

**Result:** all MT-1401 checklist items are complete.

### MT-1402 — metadata fixture tests in CI

The Rust test suite executed by `cargo test --locked` includes listxml parser tests, catalog import/generation tests, migration tests and indexed library-search tests. The qualified MT-711 run `34644544775` passed this suite on exact SHA `d63bfb7662ae7dbbfe69ca53119700e050d38b75`.

**Result:** all MT-1402 checklist items are complete.

### MT-1403 — sidecar/process tests in CI

`SessionSupervisor` tests use a controlled fake MAME executable and cover launch/exit, invalid/missing executable paths and identifiers, abnormal exit, bounded stdout/stderr diagnostic capture, duplicate-session rejection and forced-stop escalation. These tests are part of the same required Rust CI suite.

**Result:** all MT-1403 checklist items are complete.

### MT-1404 — `SUBTARGET=tiny` validation

The Tauri workflow deliberately sparse-checks the project tree and does not build native MAME on ordinary Tauri-only changes. This avoids redundant full native builds, but no project-specific `SUBTARGET=tiny` qualification has yet been integrated.

**Result:** MT-1404 remains open pending an explicit representative native-MAME validation profile and trigger policy.

### MT-1405 — path-based CI optimization

The Tauri workflow itself is path-scoped to `.github/workflows/tauri-project.yml`, `tauri/**`, `scripts/tauri/**` and `tests/tauri/**`, so project-only changes avoid a full native MAME compile. However the complete project policy for MAME-core/OSD changes versus project-only changes has not yet been formally qualified as one path-aware matrix.

**Result:** partial evidence only; keep MT-1405 open until the native-MAME trigger matrix is explicitly documented/tested.

### MT-1406 — cross-platform CI matrix

The current project-specific workflow is Linux-only. Windows and macOS project jobs are not present.

**Result:** MT-1406 remains open.

### MT-1407 — exact-head CI evidence helper

The development process records exact SHAs and GitHub Actions runs, and Ralph Bridge can retrieve them, but the repository does not yet contain a project-owned evidence helper/release checklist that makes this independently reproducible.

**Result:** MT-1407 remains open.

## MT-2000 — Product completion before embedded rendering

### MT-2001 — modern frontend completeness gate

Current implementation satisfies:

- machine browser;
- machine detail;
- indexed search/filter;
- favorites;
- collections;
- recents/history;
- machine and software-list launch;
- executable/content-path configuration;
- authoritative MAME audit state;
- per-machine settings foundation;
- supervised MAME lifecycle;
- structured/actionable error presentation.

The remaining requirement is **local artwork support**, which belongs to still-open MT-800.

**Result:** all MT-2001 items except local artwork support are complete.

### MT-2002 — runtime management completeness gate

MT-700 now provides qualified pause/resume, reset, protocol-first clean exit, save/load state and bounded runtime-state query behavior. Typed frontend wrappers exist for pause/resume/reset/save/load/query-state and mute. However the frontend command layer still lacks a typed `stopMame()` wrapper/UI flow even though Rust exposes `stop_mame`.

Because MT-2000 is a product gate rather than a backend-only gate, clean exit should remain open until the user-facing frontend can request stop and consume the result.

**Result:** pause/resume, reset, save/load state and runtime status are complete; clean exit remains open at the product layer.

### MT-2003 — UX acceptance pass

The library is keyboard-operable (MT-407), controller-navigation ownership was researched/tested (MT-408), backend-heavy metadata/audit/process work is asynchronous or off the WebView hot path, structured errors are rendered in the relevant UI states, and general/per-machine settings are visible in the application shell/library detail flow.

**Result:** MT-2003 acceptance is satisfied by current implementation and prior MT-400/600 qualification.

### MT-2004 — external-window release candidate

Cross-platform packaging, release smoke testing and a final external-window release candidate have not been completed.

**Result:** MT-2004 remains open.

## VS-1 — Tauri → catalog → launch → stop

The current implementation proves these vertical-slice requirements:

- Tauri/React application shell launches and calls typed Rust commands;
- configured MAME executable selection/validation exists;
- bounded metadata generation/import and indexed machine search exist;
- the frontend displays and selects catalog machines;
- Rust launches MAME as a supervised child and returns session-started state;
- MAME remains in its normal external native window;
- failed launches produce structured errors;
- video frames and PCM are not transported through Tauri IPC.

Three linked acceptance items remain genuinely incomplete in the frontend vertical slice:

1. user requests stop;
2. child exits cleanly as a result of that frontend request;
3. frontend returns to idle state.

Rust already has the qualified protocol-first stop/escalation implementation from MT-706, so this is an integration/UI gap rather than process-supervision work.

**Next implementation target:** add a typed frontend `stopMame()` command/result and a bounded launched-session stop control that returns the library session UI to idle only after Rust confirms the stop result.

## CI basis

- MT-711 executable qualification: `d63bfb7662ae7dbbfe69ca53119700e050d38b75`, Tauri run `34644544775` — PASS.
- MT-711 documentation qualification: `bc417f32e56a633f7a52150ac1b05eed93216f7a`, docs run `34645194151` — PASS.
- Earlier reconciliation documentation: `1448fc0f44c0830b2d8a41f88cdb84aaeb162e7f`, docs run `34646280727` — PASS.
