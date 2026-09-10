# MT-703 — Local authenticated/scoped endpoint model

**Date:** 2026-09-10  
**Task:** MT-703  
**Protocol:** MT-702 runtime-control protocol v1  
**Implementation SHA:** `5a02fdddeb6f5dd6405257494d361205b2f26a4c`  
**Authoritative CI:** GitHub Actions run `34520910780` (`Tauri project`) — PASS

## Result

MT-703 establishes the runtime-control endpoint selected by DG-3 without opening a TCP, HTTP, WebSocket, Unix-domain, or named-pipe listener. The Tauri/Rust parent owns an anonymous stdin pipe to the supervised MAME child and parses authenticated protocol frames from the child's stdout.

The endpoint is deliberately not exposed to the WebView. Runtime commands are not implemented by MT-703; later MT-704 through MT-710 tasks add the typed command behaviors over this transport.

## Acceptance mapping

### Local-only by default

Satisfied. The endpoint consists of anonymous child-process stdin/stdout pipes inherited only by the supervised MAME process. No network or filesystem socket listener is created.

### Unpredictable/session-scoped endpoint or equivalent protection

Satisfied. Each launch creates a fresh 32-byte token from the operating system randomness source and encodes it as unpadded base64url for authenticated stdout framing. The token is associated only with that session's private bootstrap and retained Rust control state.

The bootstrap file is created as a temporary file and restricted to owner-only permissions on Unix. The token is not included in frontend-visible effective argv. MAME console echo is filtered so the current token is redacted before diagnostic stdout is stored or surfaced.

The authenticated `ready` event is validated against protocol version, session ID, message-size contract, and known command vocabulary before the session can be considered ready. A monotonic ready latch preserves proof that authentication completed even if a very short-lived child closes stdout immediately afterward; a protocol failure still takes precedence over the latch.

### No ambient LAN listener

Satisfied. MT-703 creates no listening socket of any kind. In particular, it does not enable MAME's HTTP/WebSocket server and does not bind loopback or wildcard TCP ports.

### Teardown with session

Satisfied. The parent-owned control writer is dropped when shutdown begins and the control state is finalized when the supervised child exits. Launch/bootstrap failures invalidate the channel and terminate the unusable child. Child stdout EOF or read failure is reflected in the channel state rather than silently reopening or reconnecting.

## Security and failure invariants

- No raw pipe handle, framing token, or Lua dispatcher is exposed to the WebView.
- No generic Lua-eval, shell, arbitrary file, or raw input-injection API is introduced.
- Authenticated malformed frames fail closed.
- Protocol frames are bounded by the MT-702 16 KiB decoded-message and 24 KiB encoded-line limits.
- Ordinary MAME stdout remains bounded diagnostic output and is separated from authenticated protocol frames.
- The session remains in `starting` until a valid authenticated `ready` event has been observed.
- A channel that closes before authentication is rejected.
- There is no reconnect or endpoint reuse within a v1 session.

## Validation evidence

Exact implementation SHA `5a02fdddeb6f5dd6405257494d361205b2f26a4c` passed GitHub Actions run `34520910780`.

The `linux-quality` job passed frontend formatting/lint/typecheck/tests/build, Rust formatting, Rust tests, the library performance qualification, Clippy with warnings denied, and lockfile verification. The MT-703 test coverage includes token generation/encoding, bootstrap privacy, split-frame parsing, current-token redaction, fail-closed malformed authenticated frames, session-scoped endpoint readiness, and endpoint teardown.

Earlier failing iterations are not qualification evidence. They exposed and drove fixes for formatting drift, two compiler errors, two Clippy findings, and a fast-exit ready/EOF race. The exact SHA above is the qualified implementation.

## Scope boundary

MT-703 only establishes and authenticates the local session-scoped transport. Pause/resume begins in MT-704. Reset, clean protocol exit, save/load state, mute/volume, query-state, and the broader adversarial protocol suite remain explicitly open under MT-705 through MT-711.
