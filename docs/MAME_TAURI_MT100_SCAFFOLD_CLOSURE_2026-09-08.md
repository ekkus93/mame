# MT-100 Tauri Scaffold Closure

**Date:** 2026-09-08  
**Repository:** `ekkus93/mame`  
**Implementation branch:** `ralph/mt-100-scaffold`  
**Closure branch:** `ralph/mt-100-closure`  
**Implementation PR:** #2  
**Closure PR:** #3  
**Status:** Closed

## Scope

MT-100 established the project-owned Tauri 2 application scaffold without modifying the MAME emulation core. It includes a React/TypeScript frontend, Rust/Tauri backend boundaries, versioned configuration, typed command/event contracts, stable error envelopes, explicit frontend state/error handling, automated frontend/Rust quality gates, and both production-build and real development-window smoke validation.

## Implementation merge evidence

The executable scaffold was developed on `ralph/mt-100-scaffold` and merged through PR #2.

- Original fully green scaffold head: `32567a322e9b5fe4f67d32ae61c9cdb63082e01e`
- Original Tauri exact-head run: `34259237211` — success
- Final implementation PR head after inherited-CI scope hardening: `2d51c87a4a3c7475c3a4ea74aadbd47b41632550`
- Exact-head PR-context qualification runs:
  - Tauri project: `34272146771` — success
  - CI (Linux): `34272146785` — success
  - CI (Windows): `34272146746` — success
  - CI (macOS): `34272146733` — success
  - Rebuild BGFX shaders: `34272146838` — success
  - Build documentation: `34272146788` — success
  - XML/JSON validation: `34272146798` — success
  - Check #include guards: `34272146751` — success
  - Compile UI translations: `34272146735` — success
- Squash-merge commit on `master`: `208bcda6ce6db21f379d453ab2be0f74075cf793`

## Dependency-lock closure

The first MT-100 implementation CI generated dependency lockfiles but did not commit them. Closure freezes the exact lockfile bytes from the qualified run artifact rather than regenerating dependency resolution later.

Qualified artifact:

- Artifact ID: `10069493506`
- `tauri/package-lock.json` SHA-256: `cc9d1593a0c44116579147b4bf432032fe7c2fb196253ebc4e051558c8bac04f`
- `tauri/src-tauri/Cargo.lock` SHA-256: `ed473c5b2d2b9a90ff0634cdf78fa9ade635a22a14838c079ba39f958df1ea93`
- Lockfile freeze commit: `4affc73060a6b1730dce48157533fe02d00a617e`

A regeneration probe intentionally failed the recorded `package-lock.json` digest, demonstrating that later regeneration was not byte-for-byte equivalent. The closure therefore restored the original artifact and verified both recorded hashes before committing the files.

## Final MT-100 executable qualification

Closure hardening replaced diagnostic/mutating CI behavior with deterministic check-only behavior:

- `npm ci` uses committed `package-lock.json`.
- Prettier is check-only via `npm run format:check`.
- Rust Clippy/tests/release build use `--locked` where applicable.
- CI verifies the committed lockfiles are unchanged after the job.
- Tauri production build runs with `--no-bundle` as a smoke build.
- Development launch runs under Xvfb and succeeds only after `xdotool` observes a window titled `MAME Tauri Frontend`.
- The smoke helper lives under `tauri/scripts/` so future Tauri helper edits do not match upstream MAME's legitimate `scripts/**` CI triggers.

Exact executable closure qualification:

- Head: `224a6c8cfb78b10f5f8d704b41f1d76515f7fe89`
- Tauri project run: `34289516634`
- Job: `102272695797`
- Result: success
- Passed gates: committed-lockfile install, frontend format/lint/typecheck/tests/build, Rust format/Clippy/tests/release build, Tauri production build, real development-window smoke, and lockfile no-drift verification.

## MT-101 through MT-109 closure mapping

- **MT-101:** Tauri 2 scaffold exists; production build passes; development window is observed under Xvfb; no MAME emulation-core source is modified.
- **MT-102:** React + strict TypeScript shell, baseline layout, and error boundary are implemented.
- **MT-103:** Rust backend responsibilities are separated into `app`, `config`, `errors`, `mame`, `metadata`, `library`, `storage`, `sessions`, and `platform` boundaries.
- **MT-104:** Versioned typed request/response command boundary and stable error envelope are implemented and tested.
- **MT-105:** Versioned backend-to-frontend lifecycle event boundary is implemented with bounded payloads.
- **MT-106:** Platform config root, schema versioning, defaults, migration boundary, and corrupt/unsupported configuration handling are implemented and tested.
- **MT-107:** Backend-authoritative application state and explicit loading/ready/error UI states are established.
- **MT-108:** Frontend format, lint, strict typecheck, tests, and production build are enforced in CI.
- **MT-109:** Rust format, Clippy with warnings denied, tests, locked release build, and Tauri smoke builds are enforced in CI.

## CI scope hardening

The inherited MAME workflows originally matched `.github/workflows/**`, which caused Tauri workflow edits to launch the full upstream MAME CI matrix. PR #2 narrowed each inherited workflow's workflow-file trigger to its own definition while preserving its existing source-path triggers.

During closure, the smoke helper was moved from `scripts/tauri/` to `tauri/scripts/` because upstream MAME correctly treats all `scripts/**` changes as MAME-relevant. This preserves upstream validation semantics and prevents future Tauri-only helper changes from reintroducing that fan-out.

## Closure PR and merge evidence

Formal backlog reconciliation and closure hardening were merged through PR #3.

- Closure PR head: `04ec433559c743657f19a02a259a8aaa62adbf3f`
- PR-context Tauri project run: `34290392374` — success
- PR-context Build documentation run: `34290392471` — success
- Squash-merge commit on `master`: `474a17580087b06bc3b7bf5be2f5463d1649db80`
- Merged `master` tree: `098b67a39024f4b827dfd27efd61fccebf1c5780`
- Qualified closure PR tree: `098b67a39024f4b827dfd27efd61fccebf1c5780`

The matching tree SHA confirms the squash merge preserved the exact qualified PR content.

## Result

MT-000 and MT-100 acceptance criteria are satisfied, their TODO checkboxes are reconciled on `master`, the qualified dependency lockfiles are committed, deterministic Tauri CI is in place, and the formal closure PR is merged.

The next implementation phase is **MT-200 — MAME executable and sidecar integration**.
