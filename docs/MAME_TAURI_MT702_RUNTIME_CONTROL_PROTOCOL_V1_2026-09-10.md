# MT-702 — Runtime Control Protocol v1

**Date:** 2026-09-10  
**Repository:** `ekkus93/mame`  
**Task:** MT-702 — Define protocol v1  
**Parent closure:** MT-701 `0941150895fb84a8199a3f9ff64b237d1c04f0e5`  
**Transport decision:** DG-3 — parent-owned anonymous stdin/stdout pipes with a project-owned MAME Lua shim

## 1. Status and scope

This document is the normative common-envelope specification for **MAME Tauri runtime-control protocol version 1**.

Protocol v1 is a narrow, low-rate management plane between the trusted Rust session supervisor and the project-owned Lua shim running inside the supervised MAME process. It is not a frontend protocol and it is never used to carry video frames, PCM audio, or high-rate gameplay input.

The Rust backend is the protocol client. The MAME Lua shim is the protocol server/adapter. The WebView invokes typed Tauri commands only; it never receives the child stdin handle, framing token, raw protocol writer, or a generic Lua-evaluation primitive.

MT-702 freezes:

- the version field and common message envelopes;
- request-ID rules;
- the version-1 command vocabulary;
- structured success/error representation;
- framing and message-size bounds;
- request acceptance/completion semantics;
- connection lifecycle;
- timeout/disconnect behavior;
- malformed-message behavior.

Command-specific `params`, result payloads, and state-event payloads that are intentionally owned by MT-704 through MT-710 may be narrowed further by those tasks without changing the common envelope. A command MUST NOT be advertised as supported until its command-specific schema and semantics are implemented and tested. Any incompatible change to the common envelope requires a new protocol version.

Normative terms **MUST**, **MUST NOT**, **SHOULD**, and **MAY** are used in their ordinary protocol-specification sense.

---

## 2. Protocol constants

| Constant | v1 value | Rationale |
| --- | ---: | --- |
| `version` | `1` | explicit compatibility boundary |
| maximum decoded JSON message | `16,384` bytes | management messages are small; hard cap before parsing/dispatch |
| maximum encoded transport line | `24,576` bytes including line ending | accommodates base64url expansion plus fixed framing/wrapper overhead |
| maximum request ID | `64` ASCII bytes | bounded opaque correlation key |
| maximum session ID | `128` UTF-8 bytes | bounded exact match to the supervisor session |
| maximum in-flight requests | `32` | bounds correlation state and prevents accidental flooding |
| control-ready timeout | `15,000 ms` | bounds initialization without assuming instant MAME startup |
| request-response timeout | `5,000 ms` | bounds command acceptance/rejection/completion acknowledgement |

All timeouts are measured with a monotonic clock. Wall-clock changes MUST NOT extend or shorten a protocol deadline.

The decoded-message limit is applied to the UTF-8 JSON bytes **before JSON parsing**. A sender MUST also enforce the encoded-line limit before writing. A receiver MUST stop accumulating a candidate protocol frame once the encoded-line limit is exceeded.

---

## 3. Transport framing

DG-3 selected anonymous child-process pipes and a Lua shim. Protocol v1 therefore uses one logical request per child-stdin line and one authenticated/framed protocol record per child-stdout line.

### 3.1 Session frame token

Rust MUST generate a fresh unpredictable frame token for every MAME session. The v1 token is the unpadded base64url encoding of 32 random bytes, yielding 43 ASCII characters from `[A-Za-z0-9_-]`.

The token:

- is owned by the Rust supervisor and the project Lua shim only;
- MUST NOT be exposed to the WebView;
- MUST NOT be written to normal application diagnostics;
- is used to distinguish protocol output from ordinary MAME stdout;
- is defense in depth and framing discrimination, not a substitute for the anonymous-pipe isolation selected by DG-3.

MT-703 owns the exact secure bootstrap/provisioning mechanism by which the shim receives the token and session ID before emitting `ready`.

### 3.2 JSON encoding

Every protocol message is a UTF-8 JSON object. On the wire, the JSON bytes are encoded as **unpadded base64url**. This keeps the console transport wrapper and stdout framing single-line and prevents user-controlled JSON strings from becoming Lua syntax or framing delimiters.

Writers SHOULD serialize compact JSON. JSON whitespace is not semantically significant after decoding.

### 3.3 Host-to-MAME line

Once the shim is ready to accept dispatcher calls, Rust delivers a request only through the fixed project dispatcher form:

```text
mame_tauri_control_v1("<frame-token>","<base64url-json>")\n
```

Both interpolated values are restricted to the base64url alphabet. Rust MUST construct this line itself from a typed request; it MUST NOT interpolate frontend-provided Lua/code fragments.

The exact shim-loading/bootstrap sequence is MT-703 implementation work. The common v1 message inside `<base64url-json>` is defined by this document.

### 3.4 MAME-to-host line

The Lua shim emits protocol output using this exact line prefix:

```text
@@MAME_TAURI_CONTROL_V1@@<frame-token>@@<base64url-json>\n
```

The host parser accepts `LF` and also accepts one `CR` immediately before `LF` for Windows text-output compatibility.

A stdout line that does not begin with the exact v1 prefix **and the current session token** is ordinary diagnostic output and MUST remain on the existing bounded diagnostic path.

A line that does begin with the exact prefix and token is an authenticated candidate protocol frame. If that candidate exceeds the wire bound, has invalid base64url, decodes to invalid UTF-8/JSON, or violates the v1 output envelope, the logical control channel MUST transition to `failed`; it MUST NOT silently reinterpret the line as diagnostics.

Pipe read chunks are not message boundaries. The host parser MUST be incremental and correct when one frame is split across reads or several lines arrive in one read.

---

## 4. Common JSON fields

### 4.1 Version

Every request, response, and event MUST contain:

```json
"version": 1
```

A request with another integer version is rejected with `PROTOCOL_UNSUPPORTED_VERSION` when a valid request ID can be recovered. The response itself uses the server's supported envelope version (`1`).

A peer MUST NOT guess at compatibility or silently reinterpret an unknown version.

### 4.2 Session ID

Every protocol message MUST contain `sessionId` equal to the authoritative Rust `SessionSnapshot.session_id` for the child process owning the pipes.

A request for another session is a stale/cross-session request and is rejected with `PROTOCOL_SESSION_MISMATCH`. Because anonymous pipes cannot legitimately move between sessions, a session mismatch is also a fatal logical-channel condition after the rejection is emitted.

### 4.3 Request ID

Every request MUST contain `requestId` matching:

```text
^[A-Za-z0-9_-]{1,64}$
```

Rust generates request IDs. The WebView does not choose them.

A request ID:

- is opaque; no semantic meaning may be inferred from its text;
- MUST be unique for the entire lifetime of one control connection;
- MUST NOT be reused after success, rejection, timeout, or completion;
- is echoed exactly by the corresponding response;
- is carried on any asynchronous command-completion event associated with that request.

At most 32 requests may be outstanding. A request beyond that bound is rejected locally by Rust before write, or by the shim as `PROTOCOL_TOO_MANY_IN_FLIGHT` if the peer-side bound is reached unexpectedly.

---

## 5. Request envelope

A request has this exact common shape:

```json
{
  "version": 1,
  "type": "request",
  "sessionId": "session-opaque-id",
  "requestId": "request-opaque-id",
  "command": "pause",
  "params": {}
}
```

Rules:

- `type` MUST be `"request"`.
- `params` MUST be a JSON object, including `{}` for commands with no arguments.
- Common-envelope fields are mandatory.
- Unknown command names are rejected; they are never treated as aliases or Lua function names.
- Once a command-specific parameter schema is frozen, unknown or incorrectly typed parameters are rejected with `PROTOCOL_INVALID_PARAMS` before side effects.
- Validation precedes dispatch. A rejected request MUST NOT partially execute.

### 5.1 Command enum

Protocol v1 defines exactly these runtime command names:

```text
pause
resume
reset
exit
save_state
load_state
set_mute
set_volume
query_state
```

There is deliberately no `eval`, `lua`, `shell`, `exec`, generic file command, arbitrary option setter, or raw input-injection command.

The command name is protocol vocabulary, not proof of runtime support. The `ready` event advertises the subset actually implemented by the running shim/MAME build. A valid v1 command absent from the advertised set is rejected with `CONTROL_UNSUPPORTED`.

The immediately stable no-argument parameter objects are:

```text
pause       {}
resume      {}
exit        {}
query_state {}
```

`reset`, `save_state`, `load_state`, `set_mute`, and `set_volume` retain this same envelope but their exact v1 parameter/result semantics are finalized by MT-705, MT-707, MT-708, and MT-709 respectively before those commands may be advertised.

---

## 6. Response envelope

Every valid request receives **exactly one** response unless the connection closes/fails before the peer can emit it.

There are three response states.

### 6.1 Completed success

Use when the command is terminal at response time:

```json
{
  "version": 1,
  "type": "response",
  "sessionId": "session-opaque-id",
  "requestId": "request-opaque-id",
  "status": "completed",
  "ok": true,
  "result": {}
}
```

`result` is always an object. A command with no result fields returns `{}`.

A `completed` response is terminal for that request. The shim MUST NOT later emit a `command_completed` event for the same request, although independently meaningful state events such as `paused` may still be emitted.

### 6.2 Accepted asynchronous success

Use when validation/dispatch succeeded but operation completion cannot yet be proven:

```json
{
  "version": 1,
  "type": "response",
  "sessionId": "session-opaque-id",
  "requestId": "request-opaque-id",
  "status": "accepted",
  "ok": true,
  "result": {}
}
```

`accepted` means **scheduled/accepted, not completed**. The frontend/backend MUST NOT convert acceptance into a user-visible claim of successful completion when the command's task requires completion evidence.

An accepted request is completed later by exactly one `command_completed` event carrying the same request ID, unless the connection terminates or the local completion deadline expires first.

This status exists specifically to avoid falsely claiming completion for operations such as save/load state or other MAME actions whose native API is asynchronous.

### 6.3 Rejected/error response

A request that is valid enough to recover a request ID but cannot be accepted uses:

```json
{
  "version": 1,
  "type": "response",
  "sessionId": "session-opaque-id",
  "requestId": "request-opaque-id",
  "status": "rejected",
  "ok": false,
  "error": {
    "code": "PROTOCOL_INVALID_PARAMS",
    "message": "The command parameters are invalid.",
    "details": {},
    "retryable": false
  }
}
```

`completed` and `accepted` responses MUST have `ok: true` and `result`, and MUST NOT have `error`. A `rejected` response MUST have `ok: false` and `error`, and MUST NOT have `result`. Inconsistent combinations are malformed protocol output.

---

## 7. Error object

The wire error shape intentionally mirrors the Rust `AppError` contract:

```json
{
  "code": "UPPER_SNAKE_CASE_CODE",
  "message": "Bounded human-readable explanation.",
  "details": {},
  "retryable": false
}
```

Rules:

- `code` MUST match `^[A-Z][A-Z0-9_]{0,63}$`.
- `message` MUST be non-empty and bounded by the enclosing 16 KiB message limit.
- `details` MUST be a JSON object.
- `retryable` MUST be explicit.
- `retryable: true` is advisory only. Neither Rust nor the frontend may automatically replay a timed-out or state-changing command merely because this flag is true.
- frame tokens/secrets MUST NOT appear in an error object.

Baseline v1 protocol/control error codes are:

```text
PROTOCOL_UNSUPPORTED_VERSION
PROTOCOL_INVALID_REQUEST
PROTOCOL_INVALID_REQUEST_ID
PROTOCOL_DUPLICATE_REQUEST_ID
PROTOCOL_TOO_MANY_IN_FLIGHT
PROTOCOL_UNKNOWN_COMMAND
PROTOCOL_INVALID_PARAMS
PROTOCOL_MESSAGE_TOO_LARGE
PROTOCOL_SESSION_MISMATCH
PROTOCOL_NOT_READY
CONTROL_UNSUPPORTED
CONTROL_OPERATION_FAILED
CONTROL_CHANNEL_CLOSED
CONTROL_CHANNEL_FAILED
CONTROL_READY_TIMEOUT
CONTROL_RESPONSE_TIMEOUT
CONTROL_COMPLETION_TIMEOUT
```

Command-specific tasks may add stable command-domain error codes, but they retain the same error object.

If a response/error that the shim is attempting to serialize would exceed the message limit, it MUST replace it with a bounded `CONTROL_OPERATION_FAILED`/protocol-safe error rather than truncate JSON into an invalid frame.

---

## 8. Event envelope

Runtime-originated notifications use:

```json
{
  "version": 1,
  "type": "event",
  "sessionId": "session-opaque-id",
  "event": "paused",
  "payload": {}
}
```

Events caused by a specific request MAY include its `requestId`. Events caused independently by MAME/user input omit it.

The common v1 event vocabulary initially reserves:

```text
ready
paused
resumed
reset
exit_requested
command_completed
protocol_error
```

Command-specific tasks define the payload fields for their state events before advertising the corresponding command capability.

### 8.1 `ready`

`ready` is the first authenticated protocol message for a connection. It has no request ID and includes at minimum:

```json
{
  "commands": ["pause", "resume", "query_state"],
  "maxMessageBytes": 16384
}
```

`commands` is the subset of the v1 command enum this exact running shim/MAME build can accept with qualified semantics. It MUST contain no duplicates or unknown command names.

Rust MUST validate the frame token, protocol version, session ID, command names, and message-size constant before transitioning the control channel to `ready`.

### 8.2 `command_completed`

An asynchronously accepted command terminates with:

```json
{
  "version": 1,
  "type": "event",
  "sessionId": "session-opaque-id",
  "event": "command_completed",
  "requestId": "request-opaque-id",
  "payload": {
    "command": "save_state",
    "ok": true,
    "result": {}
  }
}
```

Failure uses `ok: false` and the normal `error` object instead of `result`.

The `command` MUST equal the command recorded for that outstanding request. Mismatch is a fatal protocol violation.

### 8.3 `protocol_error`

When the shim can identify a framed protocol problem but cannot safely associate it with a valid request ID, it emits a bounded `protocol_error` event:

```json
{
  "version": 1,
  "type": "event",
  "sessionId": "session-opaque-id",
  "event": "protocol_error",
  "payload": {
    "fatal": true,
    "error": {
      "code": "PROTOCOL_INVALID_REQUEST",
      "message": "The request envelope is malformed.",
      "details": {},
      "retryable": false
    }
  }
}
```

A `fatal: true` protocol error transitions the logical channel to `failed` after the event is emitted.

---

## 9. Request acceptance and completion semantics

Protocol v1 distinguishes four concepts that MUST NOT be conflated:

1. **written** — Rust successfully wrote/flushed the request line to child stdin;
2. **accepted** — the shim validated and scheduled an operation (`status: "accepted"`);
3. **completed** — the requested operation has terminal success evidence;
4. **state observed** — a MAME state event or query reflects current runtime state.

A successful pipe write is never command success.

For commands whose side effect can be proven synchronously, the shim may return `completed` directly. For asynchronous MAME operations, it returns `accepted` and later `command_completed` only when the command-specific implementation has real completion evidence.

A state event may be emitted in addition to command completion. For example, a pause request can complete and separately produce `paused`; an operator pressing MAME's own pause control can also produce `paused` with no request ID.

No code may infer save/load success from elapsed time, popup text, a file appearing, or another heuristic when the command requires native completion evidence.

---

## 10. Connection lifecycle

The Rust control channel has these logical states:

```text
initializing -> ready -> closing -> closed
       |          |         |
       +----------+---------+-> failed
```

### 10.1 Initializing

After the child process is spawned and the project shim/bootstrap is installed, the channel is `initializing`.

- Rust MUST NOT send runtime requests yet.
- The shim MUST emit `ready` as its first authenticated output frame.
- If valid `ready` is not received within 15 seconds, Rust reports `CONTROL_READY_TIMEOUT` and the channel becomes `failed`.
- A control-channel initialization failure does not silently become a working channel. If MAME remains running, the application must explicitly represent controls as unavailable or apply an explicitly documented higher-level launch policy.

### 10.2 Ready

In `ready`:

- requests may be issued only for commands listed in `ready.payload.commands`;
- Rust owns one serialized child-stdin writer;
- request IDs/correlation state are scoped to this connection;
- the 32-request in-flight cap applies.

### 10.3 Closing

The channel enters `closing` when teardown begins or when a qualified exit command has been accepted and the supervisor is waiting for process termination.

New runtime requests are rejected locally as `CONTROL_CHANNEL_CLOSED`/closing. Existing accepted exit/completion handling may continue until child exit or the supervisor's bounded shutdown/escalation policy takes over.

### 10.4 Closed

Child-process exit or orderly pipe EOF closes the connection. All unresolved requests immediately fail with `CONTROL_CHANNEL_CLOSED`; they do not wait for their original timers.

The process supervisor remains authoritative for clean/crash/forced-exit classification.

### 10.5 Failed

The logical channel becomes `failed` after a fatal framing/protocol error or a request-response timeout that leaves command ordering/correlation uncertain.

Protocol v1 has **no reconnect**. A replacement MAME process/session receives a new session ID, pipes, frame token, and request-ID namespace.

Failure of the runtime-control channel MUST NOT fall back to synthesized hotkeys, arbitrary Lua, or an unauthenticated network endpoint.

---

## 11. Timeout semantics

### 11.1 Ready timeout

`CONTROL_READY_TIMEOUT` fires 15 seconds after the supervisor has completed control-channel bootstrap/startup work if no valid `ready` event has been accepted.

### 11.2 Request-response timeout

The 5-second request-response timer begins only after the complete request line has been successfully written/flushed to child stdin.

It ends on the first valid matching response (`completed`, `accepted`, or `rejected`).

If no valid response arrives by the deadline:

- that request fails as `CONTROL_RESPONSE_TIMEOUT`;
- Rust MUST NOT automatically retry it;
- the logical channel becomes `failed`, because the peer's command queue/correlation state is now uncertain;
- any later matching response is diagnostic evidence only, not retroactive command success.

### 11.3 Asynchronous completion timeout

A response with `status: "accepted"` starts a separate completion deadline owned by that command's specification. MT-705/MT-707/MT-708/MT-706 may use different finite bounds according to the native operation.

Every asynchronously completed command MUST have a documented finite completion timeout before its capability is advertised.

If the completion deadline expires:

- the request fails locally as `CONTROL_COMPLETION_TIMEOUT`;
- it is not automatically retried;
- the channel may remain `ready` if framing/request-response health is intact;
- a later completion event may be logged and may inform subsequently observed runtime state, but MUST NOT silently rewrite the already-returned timeout into success.

Where state can be queried safely, the host may issue a **new** `query_state` request with a new request ID to resolve uncertainty.

### 11.4 Disconnect precedence

Pipe EOF, broken pipe, or child exit immediately resolves all outstanding requests as `CONTROL_CHANNEL_CLOSED`/`CONTROL_CHANNEL_FAILED`; the host does not wait for request or completion deadlines after the channel is known unusable.

---

## 12. Malformed-message behavior

Malformed behavior is deliberately fail-closed at the protocol boundary while keeping ordinary MAME diagnostics separate.

### 12.1 Before host write

Rust validates and bounds typed requests before writing. It rejects locally without touching child stdin when:

- decoded JSON would exceed 16 KiB;
- encoded wrapper line would exceed 24 KiB;
- request/session IDs are invalid;
- the command is not in the v1 enum or is not advertised by the current `ready` event;
- parameters violate the frozen command schema;
- the in-flight bound is exceeded.

### 12.2 Request received by shim

For a correctly framed request:

- wrong `version` with recoverable request ID -> `rejected` / `PROTOCOL_UNSUPPORTED_VERSION`;
- duplicate request ID -> `rejected` / `PROTOCOL_DUPLICATE_REQUEST_ID`;
- unknown command -> `rejected` / `PROTOCOL_UNKNOWN_COMMAND`;
- known but unavailable command -> `rejected` / `CONTROL_UNSUPPORTED`;
- invalid command parameters -> `rejected` / `PROTOCOL_INVALID_PARAMS`;
- stale/wrong session -> `rejected` / `PROTOCOL_SESSION_MISMATCH`, then logical channel failure;
- invalid JSON/envelope with no safe request ID -> bounded `protocol_error`; structurally unrecoverable input is fatal.

A rejected request performs no partial side effects.

### 12.3 Output received by Rust

- wrong prefix/token -> ordinary bounded diagnostic output, not a protocol frame;
- correct prefix/token plus oversized line, invalid base64url, invalid UTF-8/JSON, impossible response-field combination, unknown message type/event, wrong session, duplicate/unknown terminal request correlation, or mismatched completion command -> `CONTROL_CHANNEL_FAILED` and logical-channel failure.

Rust MUST NOT discard an authenticated malformed protocol frame and continue as though nothing happened.

---

## 13. Serialization and compatibility rules

- Field names are exactly as shown and are case-sensitive.
- Common v1 request/response/event fields MUST be present with the documented JSON types.
- `params`, `result`, `payload`, and error `details` are JSON objects, not `null`, unless a later command-specific v1 specification explicitly says otherwise.
- Protocol wire command/event names use lowercase `snake_case`, independently of Tauri's existing camelCase serde convention.
- Unknown common-envelope fields are rejected in v1. This avoids ambiguous same-version interpretation.
- An incompatible common-envelope change, changed meaning of an existing field, or changed framing contract requires `version: 2` or later.
- The v1 `ready.commands` list provides capability discovery for implementations that intentionally support only a subset of the frozen command vocabulary.

The Rust backend should map a received protocol error into the existing `AppError` shape without losing `code`, `message`, `details`, or `retryable`. Raw frame tokens and raw Lua transport lines must never be forwarded to the frontend error envelope.

---

## 14. Security invariants carried into MT-703

MT-702 does not implement the endpoint, but v1 requires MT-703 to preserve these protocol assumptions:

- one control channel belongs to exactly one supervised MAME session;
- the anonymous stdin writer is retained only by trusted Rust state;
- a fresh unpredictable frame token is generated per session;
- the token/bootstrap data are not frontend-controlled;
- no ambient LAN listener is created;
- no generic Lua evaluation API is exposed to the frontend;
- only typed command enum values can reach dispatch;
- protocol and diagnostic stdout are separated by exact tokenized framing;
- teardown invalidates the channel and all correlation state.

---

## 15. Required MT-702 / MT-711 test vectors

Implementation work should preserve these protocol-level vectors for later automated tests:

1. valid `query_state` request -> one correlated terminal response;
2. valid async command -> `accepted`, then one correlated `command_completed` event;
3. wrong protocol version -> structured rejection;
4. unknown command -> structured rejection with no side effect;
5. invalid request ID -> protocol error/rejection according to recoverability;
6. duplicate request ID -> structured rejection;
7. wrong session ID -> structured rejection then channel failure;
8. decoded payload exactly at size limit -> accepted for parsing;
9. payload one byte over limit -> rejected before parsing/dispatch;
10. bad base64url in an authenticated output frame -> channel failure;
11. ordinary stdout containing JSON but not the exact tokenized prefix -> diagnostics only;
12. protocol-looking stdout with wrong token -> diagnostics only;
13. authenticated malformed JSON -> channel failure;
14. response timeout -> request failure, no automatic retry, channel failure;
15. accepted-operation completion timeout -> request failure without automatic replay;
16. child exit with outstanding request -> immediate channel-closed failure;
17. split/coalesced pipe reads -> identical parsed messages;
18. no frontend path can submit arbitrary Lua/code.

---

## 16. MT-702 acceptance mapping

- **Version field:** complete; every message carries exact integer `version: 1`, with explicit unsupported-version behavior.
- **Request ID:** complete; Rust-owned, bounded, connection-unique IDs correlate responses and asynchronous completion.
- **Command enum:** complete; nine narrow runtime commands are frozen and generic code execution is explicitly excluded.
- **Structured result/error:** complete; `completed` / `accepted` / `rejected` responses and an `AppError`-compatible error object are defined.
- **Bounded message size:** complete; 16 KiB decoded JSON, 24 KiB encoded line, bounded in-flight state, and pre-parse enforcement are defined.
- **Connection lifecycle:** complete; `initializing -> ready -> closing -> closed/failed`, one channel per session, no v1 reconnect.
- **Timeout semantics:** complete; 15 s ready timeout, 5 s response timeout, finite command-specific async completion deadlines, disconnect precedence, and no blind retry.
- **Malformed-message behavior:** complete; local pre-write rejection, structured recoverable rejections, fatal authenticated framing/output corruption, and diagnostic separation are defined.

MT-702 is complete when this exact specification and the TODO closure pass repository documentation validation. MT-703 should implement the local/session-scoped transport and bootstrap without changing this common v1 envelope.
