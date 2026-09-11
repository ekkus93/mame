# MT-706 — Clean Exit Qualification

Date: 2026-09-10

## Qualified implementation

- Branch: `ralph/mt-706-clean-exit`
- Implementation SHA: `6f7983f4073eaaa00b5878d5487289a5167d84c7`
- Tauri CI run: `34560524540` — PASS

The exact implementation SHA passed the complete `Tauri project` Linux quality workflow, including frontend formatting/lint/typecheck/tests/build, Rust formatting/tests, library UX performance qualification, Clippy, and lockfile verification.

## MT-706 contract

`stop_mame` now prefers the authenticated runtime-control protocol before process-level termination.

The runtime shim advertises protocol-v1 `exit` in its authenticated `ready` capability set and accepts only an empty parameter object for the command. Execution uses MAME's native `manager.machine:exit()` API; no keyboard synthesis, raw Lua evaluation, shell command, network listener, or generic input fallback is used.

A successful request write is not treated as shutdown completion. Rust requires an authenticated `accepted` response, moves the control state to closing while the serialized request gate is still held, and then observes the supervised child/session for a bounded 1500 ms protocol-exit grace period. Process termination remains the authoritative evidence that clean exit completed.

If protocol exit is unavailable, rejected, fails, or MAME remains alive after the protocol grace period, shutdown delegates to the pre-existing MT-207 `SessionSupervisor::stop` escalation path. That path remains authoritative for OS soft termination, bounded waiting, forced kill, and `forced_termination` accounting.

## Regression coverage

The MT-706 tests cover:

- clean protocol exit completing without OS soft-stop or forced termination;
- an acknowledged protocol exit that stalls, followed by the existing soft-stop then forced-kill escalation;
- bootstrap/shim presence of native `manager.machine:exit()` handling;
- production capability advertisement of `pause`, `resume`, `reset`, and `exit` in the authenticated `ready` event.

The last point closes a defect found during qualification review: an earlier green candidate handled `exit` in Lua but still advertised only the MT-705 capability set, which would have caused production Rust to reject clean exit as unsupported. That candidate is not considered qualified; `6f7983f4073eaaa00b5878d5487289a5167d84c7` is the corrected qualified implementation.

## Acceptance mapping

- `prefer protocol exit before process kill`: satisfied by `stop_with_protocol_exit`, authenticated `exit`, native `manager.machine:exit()`, and supervised process-exit completion evidence.
- `integrate with MT-207 escalation path`: satisfied by delegation to the existing `SessionSupervisor::stop` fallback when protocol shutdown is unavailable or does not terminate MAME within the grace deadline.
