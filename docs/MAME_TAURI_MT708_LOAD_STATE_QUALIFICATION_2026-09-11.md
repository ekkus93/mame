# MT-708 Load-State Qualification

**Date:** 2026-09-11  
**Task:** MT-708 — Implement load state  
**Branch:** `ralph/mt-708-load-state`  
**Qualified implementation SHA:** `e7fed2149977709086bbaad30947cb2a3ac8715a`  
**Authoritative Tauri CI run:** `34638686449` — PASS

## Scope

MT-708 adds a typed load-state command on top of the authenticated, session-scoped runtime-control protocol established by MT-702 through MT-707. The WebView supplies only the active session ID and a bounded logical slot. It never supplies a filesystem path or raw Lua/control payload.

The implementation adds:

- Rust/Tauri `load_mame_state`;
- frontend `loadMameState` typed request/result wrappers;
- native MAME `manager.machine:load(path)` dispatch;
- explicit logical-slot and application-owned path semantics;
- missing/invalid/wrong-machine/incompatible state rejection before native load;
- a current-runtime compatibility probe using the established native save path;
- post-load-notifier-backed completion proof;
- bounded, request-authenticated completion markers;
- explicit success/failure frontend events;
- fail-closed timeout/disconnect behavior.

## Native MAME semantics

The implementation follows the current MAME Lua API rather than synthesizing keyboard input or adding a generic Lua-eval surface:

- `manager.machine:load(filename)` requests an asynchronous state restore;
- `emu.add_machine_post_load_notifier(callback)` runs after a successful load and its subscription is retained for the control shim lifetime;
- MAME save files use the `MAMESAVE` header, format version, machine-name field, and structural signature defined by MAME's save-state implementation.

A protocol write or successful call to `machine:load` is therefore not considered completion. Successful MT-708 completion requires the post-load notifier to fire.

## Request and path model

Frontend request:

```text
{ sessionId, slot }
```

The logical slot must be 1–32 ASCII bytes, begin with an alphanumeric character, and contain only alphanumerics, `_`, or `-`.

Rust resolves the target under the project-owned save-state root introduced by MT-707:

```text
<app-data>/save-states/v1/
  machine-<hex-machine>/
    machine-only|software-<hex-software>/
      <slot>.sta
```

Temporary compatibility-probe and completion-marker files are also application-owned and session-specific. No arbitrary frontend filesystem path is accepted.

## Missing and compatibility handling

Before dispatching native load, Rust inspects the requested state file and returns structured errors for distinct conditions:

- `LOAD_STATE_NOT_FOUND` — logical slot does not exist;
- `LOAD_STATE_INVALID` — truncated/invalid MAME state-file structure;
- `LOAD_STATE_CONTEXT_MISMATCH` — state belongs to another MAME machine;
- `LOAD_STATE_INCOMPATIBLE` — unsupported state format or structural signature mismatch;
- `LOAD_STATE_READ_FAILED` — state exists but cannot be inspected.

For structural compatibility, MT-708 asks the currently running MAME session to create a temporary state through the already-qualified MT-707 save transport. Rust waits for that probe to become complete and stable, reads its current structural signature, deletes the temporary probe, and compares it with the requested state's signature. A mismatch is rejected before `machine:load` is invoked.

This does not claim arbitrary cross-version compatibility. It establishes that the requested state matches the current runtime's MAME save format, machine identity, and registered-state structure before native load is attempted.

## Acknowledgement and completion

The authenticated `load_state` request contains only Rust-generated values:

- application-owned state path;
- logical slot;
- application-owned one-shot completion path;
- 32-byte OS-CSPRNG completion token encoded as 43-character unpadded base64url.

MAME first emits the normal protocol `accepted` response. That response means only that the native load request was scheduled.

After successful restoration, the retained post-load notifier writes a bounded JSON marker containing the completion token and request ID. Rust verifies:

- bounded marker size;
- exact known fields;
- matching unpredictable completion token;
- matching request ID;
- explicit `ok` or structured failure status.

Only a verified post-load marker produces `LoadMameStateResult`.

The same runtime request gate remains held until completion, so pause/reset/save/load/exit cannot interleave with an in-progress load operation.

## Timeout and disconnect semantics

Response acknowledgement remains bounded by the protocol response deadline. Post-load completion has a separate five-second deadline.

If MAME accepts the load but no post-load confirmation appears by the completion deadline, MT-708 returns `LOAD_STATE_COMPLETION_TIMEOUT` and marks the control channel failed. This is intentionally fail-closed: a late post-load callback cannot be mistaken for a later request.

If the control channel closes or fails during the operation, load returns the corresponding channel error. Completion-marker cleanup failures are preserved as structured diagnostic detail rather than replacing or hiding the primary operation error. If restoration succeeds but marker cleanup fails, the caller receives an explicit partial-success cleanup error.

## Result and events

Successful result includes:

- schema version;
- session ID;
- machine;
- software context, if present;
- logical slot;
- project-owned state path;
- state-file byte count;
- load-completion timestamp.

Frontend events:

- `session.state_loaded` on confirmed successful restore;
- `session.state_load_failed` on failure.

Event-emission errors are not silently discarded. A success-event failure explicitly reports that the load itself completed; failure-event emission problems are attached to the original operation error.

## Tests and exact-head qualification

Exact SHA `e7fed2149977709086bbaad30947cb2a3ac8715a` passed Tauri CI run `34638686449`, including:

- frontend formatting;
- frontend lint;
- TypeScript typecheck;
- frontend tests;
- frontend production build;
- Rust formatting;
- Rust unit tests;
- full-catalog library performance qualification;
- Clippy with warnings denied;
- lockfile-integrity verification.

The MT-708 tests cover logical-slot validation, missing-state handling, wrong-machine rejection, unsupported state-format rejection, native-load bootstrap wiring, retained post-load completion semantics, and completion-token/request authentication.

## Scope boundary

MT-708 implements the runtime-control load primitive only. Save-state browsing, durable state metadata/provenance, deletion UX, and broader compatibility warnings remain MT-900 work. Mute/volume and runtime query-state remain MT-709 and MT-710 respectively; cross-command adversarial protocol qualification remains MT-711.
