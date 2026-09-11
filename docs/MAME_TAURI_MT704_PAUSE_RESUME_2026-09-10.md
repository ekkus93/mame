# MT-704 — Pause/Resume Runtime Control

**Date:** 2026-09-10  
**Repository:** `ekkus93/mame`  
**Task:** MT-704 — Implement pause/resume  
**Parent closure:** MT-703 `8994d9188c8e233e87ae6a376433eb5c85888273`  
**Qualified implementation:** `4a7f4b92e6cbdc106e42dc04805b0feea09f956e`  
**Authoritative Tauri CI:** GitHub Actions run `34547346375` — PASS

## Result

MT-704 adds typed pause and resume operations over the MT-702 protocol and MT-703 session-scoped anonymous-pipe transport. The WebView receives only dedicated Tauri commands; it does not receive the child stdin handle, framing token, raw protocol writer, or Lua evaluation capability.

The running shim advertises exactly `pause` and `resume` as implemented runtime-control capabilities. Pause maps to MAME `emu.pause()` and resume maps to `emu.unpause()`.

## Command surface

The Rust/Tauri command layer exposes:

- `pause_mame`
- `resume_mame`

Both accept the authoritative session ID and return a versioned result containing the same session ID plus the observed `paused` boolean. Frontend wrappers preserve the typed request/result contract rather than constructing raw protocol messages.

Requests are session-scoped and serialized through the control channel. Teardown uses the same request gate, so a stop/endpoint-close transition cannot race a live pause/resume write.

## Acknowledgement and completion semantics

MT-704 preserves the distinction frozen by MT-702:

1. writing a request to stdin is not command success;
2. the Lua shim validates the request and returns `accepted` for a real state transition;
3. MAME's pause/resume notifier provides state-transition evidence;
4. the shim emits the state event and then correlated `command_completed`;
5. Rust accepts successful asynchronous completion only after the matching notifier-backed state event has been observed.

If MAME is already in the requested state, the operation is idempotent and returns a terminal `completed` response with the current pause state. It does not fabricate a pause/resume transition event when no transition occurred.

Rejected requests use the MT-702 structured error envelope. Duplicate request IDs are rejected, request parameters are fixed to an empty object, and unsupported protocol commands remain unavailable.

## State events

The Lua shim retains MAME pause/resume notifier subscription objects for the session lifetime:

- `emu.add_machine_pause_notifier`
- `emu.add_machine_resume_notifier`

Notifier output is translated to the application event layer as:

- `session.paused`
- `session.resumed`

The payload is bounded and versioned with:

- `schemaVersion: 1`
- `sessionId`
- `paused`

These events are emitted for genuine MAME state transitions, including transitions initiated from MAME itself rather than only transitions caused by the Tauri commands. The application event sink is detached when the control channel closes or fails, preventing stale late frames from leaking session-state events after teardown.

## Disconnect and error behavior

Pause/resume use a five-second request-response deadline. If MAME does not acknowledge the written request by that deadline:

- the operation returns `CONTROL_RESPONSE_TIMEOUT`;
- the request is not automatically retried;
- the logical control channel is failed because command ordering/correlation is uncertain.

After an `accepted` response, pause/resume use a separate five-second completion deadline. If notifier-backed completion is not observed:

- the operation returns `CONTROL_COMPLETION_TIMEOUT`;
- there is no blind retry;
- an otherwise healthy channel remains usable;
- bounded abandoned-request correlation allows a late matching state/completion record to be consumed safely without retroactively changing the already-returned timeout into success.

EOF, broken/closed channel state, teardown, or authenticated protocol failure resolves an outstanding operation immediately rather than waiting for its original timeout. A failed channel never falls back to synthesized keyboard input, arbitrary Lua, or an unauthenticated network endpoint.

Duplicate-request history in the Lua shim and abandoned-request history in Rust are bounded to avoid lifetime-unbounded correlation state.

## Regression coverage

MT-704 adds coverage for:

- bootstrap capability advertisement for pause/resume;
- presence of MAME pause/resume notifier hooks and native pause/unpause calls;
- authenticated `ready -> accepted -> state event -> command_completed` parsing/correlation;
- notifier-backed state-sink propagation;
- immediate outstanding-command resolution on channel close;
- late completion after a local completion timeout;
- token redaction and ordinary-diagnostic separation;
- authenticated malformed-frame fail-closed behavior;
- typed frontend `pause_mame` / `resume_mame` command envelopes.

## Qualification

Exact implementation SHA:

`4a7f4b92e6cbdc106e42dc04805b0feea09f956e`

GitHub Actions run:

`34547346375`

The `Tauri project` `linux-quality` job passed the frontend formatting/lint/typecheck/tests/build gates, Rust formatting and tests, the inherited library-performance qualification, Clippy with warnings denied, and lockfile-integrity verification.

An earlier candidate `c0b7db342ec569b859a827e75e9352cc95b93818` reached and passed the frontend tests/build but failed `cargo fmt --check` on one mechanical formatting difference in `control.rs`; it is not qualification evidence. The formatting-only correction produced the exact qualified SHA above.

## MT-704 acceptance mapping

- **command:** satisfied by the typed `pause_mame` and `resume_mame` Tauri commands and the bounded protocol dispatcher.
- **acknowledgement/completion semantics:** satisfied by idempotent terminal completion or `accepted` followed by notifier-backed correlated completion.
- **state event:** satisfied by MAME notifier-driven `session.paused` and `session.resumed` events.
- **disconnect/error behavior:** satisfied by explicit response/completion deadlines, immediate EOF/channel-failure handling, bounded late-completion correlation, and no unsafe fallback.

MT-704 is qualified on the exact implementation SHA above. MT-705 can build reset semantics on the same authenticated protocol/runtime-control infrastructure.
