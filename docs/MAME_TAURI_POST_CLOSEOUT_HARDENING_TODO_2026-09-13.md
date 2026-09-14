# MAME Tauri Post-Closeout Hardening TODO — 2026-09-13

**Repository:** `ekkus93/mame`  
**Baseline:** post-MT-2200 `master` after `b12fef0e9f68ce62376d4c996ceb4647346bfa88`  
**Spec:** `docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_SPEC_2026-09-13.md`  
**Scope:** bounded hardening after code review; does not reopen optional embedded-render/OSD/in-process research tracks.

## PCH-001 — Expose supported runtime controls in the visible session UI

- [x] Import and use `pauseMame` from the typed frontend command wrapper.
- [x] Import and use `resumeMame` from the typed frontend command wrapper.
- [x] Import and use `resetMame` from the typed frontend command wrapper.
- [x] Import and use `setMameMute` from the typed frontend command wrapper.
- [x] Import and use `queryMameRuntimeState` from the typed frontend command wrapper.
- [x] Add visible Pause and Resume controls.
- [x] Add visible Soft reset control.
- [x] Add visible Mute/Unmute control.
- [x] Add visible Refresh runtime state control.
- [x] Disable controls unless a supervised MAME session is currently running.
- [x] Surface operation success through a notice and operation failure through the existing error-message path.
- [x] Preserve the selected product boundary: no video frames, PCM audio, or gameplay input are transported through Tauri IPC.

## PCH-002 — Refresh shortcut/gameplay-input ownership on session lifecycle events

- [x] Subscribe the application shell to `session.started`.
- [x] Subscribe the application shell to `session.exited`.
- [x] Subscribe the application shell to `session.crashed`.
- [x] Subscribe the application shell to `session.failed`.
- [x] Trigger the existing library shortcut ownership refresh path on lifecycle events.
- [x] Clear stale gameplay-input ownership without waiting for a user focus change.
- [x] Preserve fail-closed shortcut suppression if lifecycle event subscription fails.

## PCH-003 — Document non-Unix runtime-control bootstrap security semantics

- [x] Preserve Unix `0600` bootstrap-file permission hardening.
- [x] Document the non-Unix/Windows path as relying on user TEMP directory ACLs, unguessable per-session frame-token entropy, and RAII cleanup.
- [x] Add regression coverage that rejects an undocumented non-Unix platform boundary.

## PCH-004 — Add runtime-control source-generation guardrails

- [x] Verify build-script source generation still uses fail-closed exact anchors.
- [x] Verify all runtime-control Rust/Lua source fragments remain visible to the generator.
- [x] Verify generated output filenames remain tracked by regression tests.
- [x] Keep this as a bounded guardrail rather than a broad generator rewrite.

## PCH-005 — Align frontend session typing with the backend payload

- [x] Add executable identity to the frontend `SessionSnapshot` type.
- [x] Add effective argv and effective launch config to the frontend `SessionSnapshot` type.
- [x] Add created/started/ended timestamps to the frontend `SessionSnapshot` type.
- [x] Add exit code, termination signal, forced-termination flag, stdout/stderr tails, truncation flags, and diagnostic error to the frontend `SessionSnapshot` type.
- [x] Update frontend tests that construct session snapshots to use full backend-shaped fixtures.

## PCH-006 — Add and wire post-closeout regression tests

- [x] Add `scripts/tauri/test-post-closeout-hardening.py`.
- [x] Extend security regression coverage for runtime-control bootstrap semantics.
- [x] Wire post-closeout hardening regression into `Tauri project` CI.
- [x] Include the post-closeout spec/TODO files in the sparse checkout used by CI.

## PCH-007 — Qualification protocol

- [x] Run local static regression checks where the archive environment permits.
- [x] Require exact-head PR CI for branch qualification.
- [x] Require post-merge `master` CI verification before claiming closure.

## Remaining work

No required PCH task remains open in this TODO. Any later work on native-window embedding, dedicated OSD, in-process hosting, release signing/notarization, or public bundled MAME binaries belongs in a separately scoped research or release-qualification track.
