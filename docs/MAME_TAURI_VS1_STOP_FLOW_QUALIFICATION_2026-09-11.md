# MAME Tauri VS-1 Frontend Stop Flow Qualification

**Date:** 2026-09-11  
**Branch:** `ralph/vs1-stop-flow`  
**Qualified implementation SHA:** `7e5a5b5353a48afb49f50f16a78c46c20acc3eef`  
**Tauri CI:** run `34662478512` — PASS

## Scope

This closes the frontend integration gap identified by the backlog reconciliation for VS-1 and MT-2002. Rust already exposed the qualified MT-706 `stop_mame` command, but the React application did not have a typed stop wrapper or a user-facing flow that could request shutdown and return the session UI to idle.

## Implemented behavior

- Added typed `StopMameRequest` and `StopMameResult` frontend contracts matching the Rust command.
- Added `stopMame()` to the typed Tauri command layer with an invocation regression test.
- Added a top-level `SessionControlPanel` so stop behavior is independent of which library row or software-list item is currently selected.
- The panel subscribes to existing `session.started`, `session.exited`, `session.crashed`, and `session.failed` lifecycle events.
- Lifecycle listeners are installed before the authoritative `get_mame_session` read, avoiding a mount-time race where a newly started session could otherwise be missed.
- Active machine/software context and PID are displayed when available.
- The user can request stop only for a running session; duplicate stop requests are suppressed while shutdown is in progress.
- The panel returns to idle only after Rust returns a terminal `StopSessionResult` or a terminal lifecycle event arrives.
- If Rust reports `forcedTermination: true`, the UI retains a visible warning rather than presenting forced kill as an ordinary graceful stop.
- Stop failures remain visible and retryable; the active session is not silently discarded on command failure.

## Architectural invariants

The frontend sends only the session ID through the typed command. Shutdown policy remains wholly owned by Rust/MT-706: authenticated native protocol exit first, then bounded OS-level graceful termination and forced-kill escalation if required. No video, PCM, gameplay input, process signal construction, or generic shell access is introduced in the WebView.

## Acceptance impact

This satisfies the previously open product-layer requirements:

- MT-2002 clean exit is available through the frontend product surface.
- VS-1 user requests stop.
- VS-1 child exits as the result of that request using the qualified Rust shutdown path.
- VS-1 frontend returns to an idle/no-active-session state after confirmed termination.

## Qualification evidence

Run `34662478512` passed on exact SHA `7e5a5b5353a48afb49f50f16a78c46c20acc3eef`, including:

- frontend format check;
- frontend lint;
- TypeScript typecheck;
- frontend unit tests;
- frontend production build;
- Rust format;
- Rust tests;
- library UX performance qualification;
- Rust Clippy with warnings denied;
- lockfile drift verification.
