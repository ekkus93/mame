# MT-707 Save State Qualification

**Task:** MT-707 — Implement save state  
**Date:** 2026-09-11  
**Qualified implementation SHA:** `0a4d26e94259002494e6931b7ab2b4485ed35aef`  
**Tauri CI:** `34616631612` — PASS

## Qualified contract

MT-707 adds a typed `save_mame_state` command and matching TypeScript `saveMameState()` wrapper. The frontend supplies a running `sessionId` and a logical slot name; it never supplies a filesystem path.

Logical slots are 1–32 ASCII bytes, must start with an alphanumeric character, and may contain only alphanumerics, `_`, or `-`. Path syntax, traversal components, whitespace, and oversized slot names are rejected before a runtime-control request is sent.

Rust owns save-state placement beneath the application data directory under `save-states/v1`. State paths are partitioned by hex-encoded machine identity and, when present, software identity. The logical slot maps to `<slot>.sta`; the in-progress write uses a session-specific hidden pending filename so an incomplete write cannot replace a previously qualified slot.

## Runtime-control behavior

The authenticated protocol v1 capability list now advertises `save_state`. Rust serializes the operation through the existing per-session request gate and sends only the application-generated pending path plus the logical slot. The Lua shim validates the bounded parameter object and invokes MAME's native `manager.machine:save(path)` API.

An authenticated `accepted` response proves only that MAME accepted the request. It is not treated as completion. MAME's Lua surface does not provide a post-save completion notifier suitable for this contract, so Rust verifies completion at the filesystem boundary instead of fabricating a protocol completion event.

## Completion and integrity

After acceptance, Rust waits up to five seconds for the pending state file while requiring the session control channel to remain ready. A candidate file must:

- exceed the minimum MAME save-state header size;
- begin with the `MAMESAVE` signature;
- carry the expected machine identity in the MAME save-state header; and
- retain a stable file size over three consecutive polls.

Only then is it promoted into the logical slot. When replacing an existing slot, the old state is staged as a backup and restored if promotion fails. Cleanup and rollback failures are retained in structured error details rather than silently discarded.

## Result and events

A successful result includes schema version, session ID, machine, optional software context, slot, final application-owned path, byte size, and save timestamp.

Successful saves emit `session.state_saved`. Failed saves emit `session.state_save_failed` with the same session/machine/software/slot context and the structured application error. Event-emission failures are not swallowed: a failed success-event returns `SAVE_STATE_EVENT_EMIT_FAILED` with the completed save result in details, while failure-event emission errors are attached to the original save error.

## Session and machine association

The command requires the requested session to be the current running session. The storage namespace is derived from that authoritative session snapshot, not from frontend-supplied machine/software values. File validation additionally checks the MAME state header against the running machine identity.

## Regression coverage

Automated coverage includes:

- bounded logical slot validation and path-traversal rejection;
- filesystem-safe/unambiguous machine/software context encoding;
- MAME save-state header and machine-identity validation;
- production bootstrap advertisement and native save dispatch;
- typed frontend command invocation;
- preservation of the MT-706 clean-exit capability regression as protocol capabilities expand.

Exact-head CI `34616631612` passed frontend formatting/lint/typecheck/tests/build, Rust formatting/tests, library performance qualification, Clippy, and lockfile verification.
