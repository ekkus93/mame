# MT-711 Protocol Adversarial Qualification

**Date:** 2026-09-11  
**Task:** MT-711 — Protocol adversarial tests  
**Branch:** `ralph/mt-711-protocol-adversarial-tests`  
**Qualified implementation SHA:** `d63bfb7662ae7dbbfe69ca53119700e050d38b75`  
**Authoritative Tauri CI run:** `34644544775` — PASS

## Scope

MT-711 closes the runtime-control protocol milestone with adversarial qualification of the authenticated, session-scoped protocol introduced in MT-702 and implemented through MT-710. The change is tests-only: it does not expand the production control surface or weaken existing validation.

The qualification covers the required adversarial cases:

- malformed authenticated payload;
- oversized payload;
- unknown command;
- wrong protocol version;
- stale/wrong session identity;
- connection drop while a command is outstanding;
- bounded response timeout.

## Malformed authenticated payload

The existing parser regression `authenticated_malformed_frame_fails_closed` verifies that a frame carrying the correct private session token but malformed base64url is not treated as ordinary console noise. It produces a fatal parser event and therefore fails the logical channel closed.

Wrong-token material remains diagnostic/non-authoritative and the current token continues to be redacted from console echo.

## Oversized payload

The MT-711 suite feeds a correctly authenticated frame whose decoded JSON body exceeds `MAX_DECODED_MESSAGE_BYTES`. The parser must reject it as fatal rather than partially parsing, truncating, or accepting it.

The runtime command implementations also retain encoded-line size checks before host-to-MAME transmission.

## Wrong version and stale session

Authenticated messages with a protocol version other than v1 are rejected. Authenticated messages carrying a different session ID from the private channel's bound session are also rejected.

These tests prove that knowledge of the framing token does not allow a message for another protocol generation or another session to be accepted on the current logical channel.

## Unknown command

The generated MAME bootstrap is inspected to verify the request dispatcher rejects commands that are not members of its fixed `supported_commands` set with `PROTOCOL_UNKNOWN_COMMAND` rather than evaluating or forwarding arbitrary command text.

This preserves the protocol-v1 fixed-command invariant and confirms there is no generic Lua/shell/raw-input escape hatch.

## Connection drop mid-command

The existing `channel_close_resolves_outstanding_command_immediately` regression verifies that closing the logical peer resolves an outstanding request as `CommandSignal::Closed`; callers are not left blocked until a normal command deadline and no success is synthesized after disconnect.

## Timeout

MT-711 directly exercises the bounded receive primitive with a short test-only deadline and no delivered signal. It must terminate as `ReceiveDeadlineError::Timeout` rather than wait indefinitely.

Production commands continue to use the frozen MT-702 response/completion deadlines and explicit command-specific timeout errors.

## Exact-head qualification

Exact SHA `d63bfb7662ae7dbbfe69ca53119700e050d38b75` passed Tauri CI run `34644544775`, including:

- frontend formatting;
- frontend lint;
- TypeScript typecheck;
- frontend tests;
- frontend production build;
- Rust formatting;
- Rust unit tests, including the MT-711 adversarial suite;
- full-catalog library performance qualification;
- Clippy with warnings denied;
- lockfile-integrity verification.

## Milestone result

With MT-711 qualified, MT-700 now has executable evidence for a local authenticated/scoped control endpoint, pause/resume, reset, clean exit, save/load state, supported native mute control, bounded runtime status, and adversarial protocol behavior while keeping video, PCM, and gameplay input outside Tauri IPC.
