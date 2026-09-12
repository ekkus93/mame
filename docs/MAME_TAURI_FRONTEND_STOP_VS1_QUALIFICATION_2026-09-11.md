# MAME Tauri Frontend Stop / VS-1 Qualification

**Date:** 2026-09-11  
**Qualified implementation SHA:** `1824282583310c0994a85cb81429a5dbee560377`  
**Tauri CI run:** `34675895277` — PASS

## Scope

This qualification closes the frontend integration gap identified by the backlog reconciliation for MT-2000 and VS-1: the Rust backend already provided MT-706 protocol-first clean shutdown, but the React command layer and user interface did not expose a stop flow.

## Implemented behavior

- Added typed `StopMameRequest` and `StopMameResult` frontend contracts matching the existing Rust `stop_mame` command.
- Added `stopMame()` to the typed Tauri command layer with an invocation regression test.
- Added an app-level `SessionControlPanel` that hydrates authoritative state from `get_mame_session` and subscribes to `session.started`, `session.exited`, `session.crashed`, and `session.failed` lifecycle events.
- The stop control is enabled only while the authoritative session state is `running`.
- The UI enters an explicit stopping state while the command is outstanding.
- A failed stop request keeps the session visible and presents the structured backend failure rather than falsely returning to idle.
- The panel returns to `No active MAME session` only after a clean `session.exited` event or a successful `stop_mame` result with a terminal session state.
- Abnormal `crashed`/`failed` events are surfaced as errors instead of being silently presented as a clean stop.

## VS-1 evidence

The previously open vertical-slice requirements are now satisfied:

1. **User requests stop:** `SessionControlPanel` exposes `Stop MAME` for the active running session.
2. **Child exits cleanly:** the command invokes the qualified Rust MT-706 `stop_mame` path, which prefers authenticated protocol exit and retains bounded soft/forced escalation.
3. **Frontend returns to idle state:** the panel clears the active session only on backend-confirmed terminal state.

The existing architecture invariants remain unchanged: MAME video and PCM do not cross Tauri IPC and MAME continues to own the native gameplay window/input hot path.

## Exact-head qualification

GitHub Actions run `34675895277` passed on exact SHA `1824282583310c0994a85cb81429a5dbee560377`:

- frontend formatting;
- frontend lint;
- TypeScript typecheck;
- frontend tests;
- frontend production build;
- Rust formatting;
- Rust tests;
- library UX performance qualification;
- Clippy with warnings denied;
- lockfile drift verification.
