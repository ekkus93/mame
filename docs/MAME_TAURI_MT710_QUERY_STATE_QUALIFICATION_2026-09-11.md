# MT-710 Runtime-State Query Qualification

**Date:** 2026-09-11  
**Task:** MT-710 — Implement query-state/status  
**Branch:** `ralph/mt-710-query-state`  
**Qualified implementation SHA:** `d49ffd698de985f4eb5d7f04062dc887dc2535b8`  
**Authoritative Tauri CI run:** `34643034916` — PASS

## Scope

MT-710 adds a bounded, read-only runtime-state query over the authenticated session-scoped protocol established by MT-702 through MT-709.

The query reports only state required by the frontend:

- `running` — the runtime-control peer is attached to a running MAME machine;
- `paused` — native `manager.machine.paused`;
- `machine` — native `manager.machine.system.name`;
- `software` — the supervised session's active software context, if any;
- `uiMuted` — native `manager.machine.sound.ui_mute`;
- `effectiveMuted` — native aggregate `manager.machine.sound.muted`.

It does not expose arbitrary MAME options, device trees, memory, debugger state, files, Lua evaluation, or other unbounded internals.

## Request and completion semantics

Frontend request:

```text
{ sessionId }
```

Protocol request:

```text
command = "query_state"
params = {}
```

The operation is synchronous. MAME returns one `completed` response containing exactly the five native runtime fields (`running`, `paused`, `machine`, `uiMuted`, `effectiveMuted`). An `accepted` response or asynchronous completion event is a protocol violation.

The Rust Tauri layer separately resolves the authoritative supervised session and contributes the bounded software context. It rejects missing/non-running sessions before returning state.

## Context verification

Rust compares the machine short name returned by the MAME runtime with the machine recorded by `SessionSupervisor`. A mismatch returns `CONTROL_STATE_CONTEXT_MISMATCH` rather than presenting contradictory state to the UI.

A ready runtime-control peer reporting `running = false`, an empty/oversized machine identity, malformed fields, unexpected result fields, timeout, disconnect, or channel failure is rejected explicitly.

## Timeout and channel behavior

The query uses the protocol response deadline. If the authenticated peer fails to answer before the deadline, the logical control channel is failed because request/response ordering is no longer trustworthy. No query is retried blindly.

## Typed frontend surface

The Tauri command is:

```text
query_mame_runtime_state
```

The TypeScript wrapper is:

```text
queryMameRuntimeState({ sessionId })
```

The result is schema-versioned and contains only the bounded fields listed above.

## Tests and qualification

Exact SHA `d49ffd698de985f4eb5d7f04062dc887dc2535b8` passed Tauri CI run `34643034916`, including:

- frontend formatting;
- frontend lint;
- TypeScript typecheck;
- frontend tests;
- frontend production build;
- Rust formatting;
- Rust tests;
- full-catalog library performance qualification;
- Clippy with warnings denied;
- lockfile-integrity verification.

Regression coverage verifies that the generated runtime-control shim advertises `query_state`, uses only bounded native status properties, and does not introduce device-tree or memory exposure.

## Acceptance mapping

- **running:** implemented and runtime-validated.
- **paused:** native MAME pause state returned.
- **machine identity:** native identity returned and cross-checked against the supervised session.
- **other safe bounded status required by UI:** software context plus user/effective mute state are included; no broader introspection surface is exposed.
