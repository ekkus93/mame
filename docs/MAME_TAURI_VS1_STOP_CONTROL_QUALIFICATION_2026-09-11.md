# MAME Tauri VS-1 Frontend Stop Control Qualification

**Date:** 2026-09-11  
**Qualified implementation SHA:** `1e3c2a7b95efc54fdb738e518826c655369a4ad0`  
**Tauri CI run:** `34665902655` — PASS

## Scope

This qualification closes the frontend integration gap identified by the backlog reconciliation for VS-1 and the MT-2002 product-level clean-exit gate. The Rust supervisor/protocol exit path was already qualified under MT-706; this change makes that capability directly usable and truthfully observable from the Tauri frontend.

## Typed stop command

The frontend command layer now exposes:

- `StopMameRequest { sessionId }`;
- `StopMameResult { schemaVersion, softStopRequested, forcedTermination, session }`;
- `stopMame(request)` invoking the existing Rust `stop_mame` command.

A Vitest regression verifies the exact Tauri invoke envelope.

## Session control UI

`SessionControlPanel` owns the user-facing active-session stop flow. It:

1. reconciles current session state with `get_mame_session` at mount and when the Tauri window regains focus;
2. subscribes to the existing bounded `session.started`, `session.exited`, `session.crashed`, and `session.failed` lifecycle events so a newly launched session becomes controllable immediately without requiring a focus change;
3. displays the active session identity, machine/software target and PID when available;
4. sends `stop_mame` only for the exact active session ID;
5. remains in a stopping state until Rust returns a terminal session result;
6. treats a still-active result as an error rather than pretending shutdown succeeded;
7. preserves a failed stop as a retryable visible error;
8. distinguishes clean stop from forced termination and surfaces forced termination to the user;
9. resets session-local library UI only after Rust confirms terminal shutdown.

Lifecycle events observed while the panel owns an in-flight stop request cannot race the command result into a false early-idle state. The request result remains authoritative for the user-requested stop.

## VS-1 effect

The previously open vertical-slice sequence is now executable from the frontend:

- user requests stop;
- Rust performs the qualified protocol-first MT-706 shutdown/escalation path;
- the child reaches a terminal state;
- frontend session-local state returns to idle only after confirmation.

The session controller is library-global rather than tied to the currently selected machine, so changing catalog selection cannot hide the active-session stop affordance. Software-list launches are also discovered through the same backend lifecycle event path.

## No real-time-path regression

The implementation transports only bounded lifecycle/control metadata through Tauri IPC. It does not route video frames, PCM samples, or gameplay input through the WebView.

## CI evidence

GitHub Actions run `34665902655` passed on exact implementation SHA `1e3c2a7b95efc54fdb738e518826c655369a4ad0`, including:

- frontend formatting;
- frontend lint;
- TypeScript typecheck;
- frontend tests;
- frontend production build;
- Rust formatting;
- Rust tests;
- library UX performance qualification;
- Clippy with warnings denied;
- lockfile integrity.

The earlier candidate `2d1fdb0e3f8d5eaa42350d5dc050f949e1aeda68` also passed CI, but it is not the qualification point because the post-green audit found that focus-only session reconciliation could delay the stop affordance after an in-focus launch. The qualified SHA adds lifecycle-event-driven synchronization and closes that gap.
