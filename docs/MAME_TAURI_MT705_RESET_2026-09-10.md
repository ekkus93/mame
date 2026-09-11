# MT-705 — Reset Runtime Control

**Date:** 2026-09-10  
**Repository:** `ekkus93/mame`  
**Task:** MT-705 — Implement reset  
**Parent closure:** MT-704 `4d452b4a0bc4aa670f71e33bdc8d1c860711d223`  
**Qualified implementation:** `0949db54258c2846b94ad0597aff0224dc9f7b7f`  
**Authoritative Tauri CI:** GitHub Actions run `34550205672` — PASS

## Result

MT-705 adds a typed reset operation to runtime-control protocol v1. The supported reset operation is deliberately narrow: **protocol v1 supports MAME soft reset only**.

The frontend invokes the dedicated `reset_mame` Tauri command with the authoritative session ID. Rust constructs the protocol request itself; the WebView does not choose a raw command, reset kind, request ID, Lua expression, framing token, or child-process transport payload.

A successful result has this typed application shape:

```text
schemaVersion: 1
sessionId: <current session>
kind: soft
```

## Supported reset semantics

MAME exposes distinct soft-reset and hard-reset semantics through its Lua API.

MT-705 selects **soft reset** because it resets the running machine/device tree while preserving the current supervised MAME process and runtime-control session. The Lua adapter invokes:

```text
manager.machine:soft_reset()
```

The wire request is exactly:

```json
{
  "command": "reset",
  "params": {
    "kind": "soft"
  }
}
```

No additional reset parameters are accepted.

Hard reset is intentionally not supported by MT-705. MAME hard reset tears down the current emulation session and starts a replacement session for the same system. That lifecycle is materially different from the session-scoped MT-703 control-channel model and would require explicit re-bootstrap/re-correlation semantics. Protocol v1 therefore does not silently map hard reset to soft reset and does not expose a frontend hard-reset selector.

A peer request containing any reset kind other than `soft` is rejected with `CONTROL_UNSUPPORTED_RESET_KIND`. Extra or malformed reset parameters are rejected with `PROTOCOL_INVALID_PARAMS` before reset dispatch.

## Acknowledgement and completion

MAME's reset API schedules the operation, so a successful stdin write or Lua function return is not considered reset completion.

The MT-705 sequence is:

1. Rust sends the bounded authenticated `reset` request with `kind: "soft"`.
2. The Lua shim validates the request and emits an `accepted` response.
3. The shim calls `manager.machine:soft_reset()`.
4. MAME invokes the retained `emu.add_machine_reset_notifier` subscription.
5. The shim emits a correlated `reset` event with `{ "kind": "soft" }`.
6. The shim emits correlated `command_completed` with `{ "kind": "soft" }`.
7. Rust returns success only after it has observed the matching notifier-backed reset event and completion.

A synchronous `completed` response for reset is treated as protocol failure because it would bypass the completion evidence required by MT-705.

Likewise, a nominally successful `command_completed` message that arrives without the preceding matching reset-notifier event is rejected as invalid protocol output rather than converted into false success.

## Timeout and disconnect behavior

Reset inherits the MT-702/MT-704 request-response deadline of five seconds. If the peer does not acknowledge the request before that deadline:

- Rust returns `CONTROL_RESPONSE_TIMEOUT`;
- the request is not retried automatically;
- the logical control channel is failed because request ordering/correlation can no longer be trusted.

After an `accepted` response, MT-705 applies a separate five-second reset-completion deadline. If notifier-backed reset completion does not arrive:

- Rust returns `CONTROL_COMPLETION_TIMEOUT`;
- the request is not retried blindly;
- an otherwise healthy control channel remains available;
- the request ID enters the same bounded abandoned-request correlation path used by MT-704 so a late matching reset event/completion can be consumed safely without retroactively turning the returned timeout into success.

Channel close, channel failure, broken stdin, authenticated malformed protocol output, or supervised-session teardown resolves an outstanding reset operation explicitly. There is no fallback to synthesized F3 input, generic Lua evaluation, signals, or a network endpoint.

## Capability advertisement

The authenticated `ready` event now advertises exactly the implemented runtime commands at this stage:

```text
pause
resume
reset
```

`reset` is not advertised until the MT-705 parser, correlation logic, Lua adapter, typed Tauri surface, and regression tests are present together.

## Regression coverage

MT-705 adds or extends coverage for:

- `reset` capability advertisement;
- retained MAME reset notifier subscription;
- use of `manager.machine:soft_reset()`;
- absence of a hard-reset call in the project Lua shim;
- explicit non-soft reset rejection policy;
- exact typed frontend `reset_mame` invocation;
- authenticated `ready -> accepted -> reset event -> command_completed` parsing and correlation;
- rejection of reset completion without notifier-backed reset evidence;
- reset result validation requiring exactly `kind: "soft"`;
- inherited response/completion timeout and disconnect behavior;
- preservation of MT-704 pause/resume behavior.

## Qualification

Exact implementation SHA:

`0949db54258c2846b94ad0597aff0224dc9f7b7f`

GitHub Actions run:

`34550205672`

The `Tauri project` workflow passed frontend formatting, lint, typecheck, tests, and production build; Rust formatting and tests; library UX performance qualification; Clippy with warnings denied; and lockfile-integrity verification.

The implementation diff from the qualified MT-704 closure is limited to the runtime-control Rust/Lua implementation and the typed Rust/TypeScript reset command surface. It makes no MAME-core, workflow, dependency, video/audio, or gameplay-input-path changes.

## MT-705 acceptance mapping

- **supported reset semantics documented:** soft reset is the sole supported v1 reset kind; hard reset is explicitly unsupported because it replaces the emulation session and therefore needs lifecycle semantics beyond MT-705.
- **command/result:** the typed `reset_mame` command emits a bounded authenticated `reset` request with fixed `kind: "soft"` semantics and returns a versioned soft-reset result only after notifier-backed completion.

MT-705 is qualified on the exact implementation SHA above. MT-706 can add clean protocol exit on the same authenticated command/correlation infrastructure.
