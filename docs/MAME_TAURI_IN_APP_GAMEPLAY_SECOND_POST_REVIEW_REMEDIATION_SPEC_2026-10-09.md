# MAME Tauri In-App Gameplay Second Post-Review Remediation Specification

**Date:** 2026-10-09  
**Status:** Active second post-review remediation specification  
**Reviewed baseline:** `461b05174df7b80351b74f589eb40e11b5647390` on `master`  
**Companion TODO:** [MAME Tauri In-App Gameplay Second Post-Review Remediation TODO](MAME_TAURI_IN_APP_GAMEPLAY_SECOND_POST_REVIEW_REMEDIATION_TODO_2026-10-09.md)  
**Previous remediation specification:** [MAME Tauri In-App Gameplay Post-Review Remediation Specification](MAME_TAURI_IN_APP_GAMEPLAY_POST_REVIEW_REMEDIATION_SPEC_2026-10-09.md)  
**Previous remediation TODO:** [MAME Tauri In-App Gameplay Post-Review Remediation TODO](MAME_TAURI_IN_APP_GAMEPLAY_POST_REVIEW_REMEDIATION_TODO_2026-10-09.md)  
**Parent gameplay specification:** [MAME Tauri In-App Gameplay Specification](MAME_TAURI_IN_APP_GAMEPLAY_SPEC_2026-10-08.md)  
**Package evidence:** [MAME Tauri In-App Gameplay Package Evidence](MAME_TAURI_IN_APP_GAMEPLAY_PACKAGE_EVIDENCE_2026-10-09.md)

## 1. Purpose

The first post-review remediation corrected the major defects identified after the original in-app gameplay implementation and qualified exact-source Linux, Windows and macOS builds plus a full real-MAME Linux Debian/AppImage artifact.

A second source-level review of the remediated implementation found that the architecture remains strong, but several independently actionable issues and evidence gaps remain. The most important confirmed production defect is a same-session frontend input-generation race that can allow a stale cleanup to release a control after a replacement input generation has already reasserted it. Additional work is required around durable history finalization, presentation-ack strictness, frame-reader producer disconnects, qualification output containment, package-integrity provenance verification, and lifecycle-level regression coverage.

This document defines that second remediation. It does not reopen user-deferred graphical-desktop or private-ROM acceptance. It does not erase or retroactively rewrite the previous remediation checklist. The previous checklist remains historical evidence; this specification and its companion TODO are the active source of truth for the newly discovered work.

## 2. Reviewed baseline and current qualification state

The second review examined `461b05174df7b80351b74f589eb40e11b5647390`.

At that baseline:

- the first post-review TODO records 78 checked entries and 14 unchecked deferred entries;
- the executable source qualified by the prior remediation is `0857595401d131080688baec2f3225b154787f2d`;
- Tauri project quality, Linux packaging, Windows packaging, macOS packaging, security, real-MAME Debian/AppImage construction, ROM-less frame qualification, installed-package checks and documentation CI passed for the prior remediation;
- package evidence records the qualified artifact, hashes, sizes, bundled MAME version and CI run IDs;
- the fourteen graphical-desktop/private-ROM/original-installed-MAME acceptance items remain deliberately deferred.

Any runtime-affecting change from this second remediation invalidates the prior executable artifact for final gameplay acceptance. A fresh exact-source full-runtime artifact is required after the last behavior-changing commit.

## 3. Second-review findings requiring remediation

### SPRR-001 — Same-session stale input-release race

The first remediation prevents stale asynchronous `setMameInputs` completions from repopulating accepted-input bookkeeping. It does not completely prevent stale **commands** from affecting MAME.

The current `GameSurface` input effect is recreated when `preferredGamepadId`, `session.sessionId`, or `session.state` changes. Cleanup invalidates the old generation, waits for an old request if necessary, then sends zero-valued releases to the captured session. If the effect is recreated because controller preference changes while the same MAME session remains active, the replacement generation may reassert an input before the old cleanup release reaches MAME. The stale zero can then override the new press while the replacement generation still believes the press is accepted.

**Required behavior:**

- session-level input teardown/release must occur only when ownership of that MAME session actually ends, not merely because controller preference or another non-session input parameter changes;
- preference changes within the same live session must not enqueue stale zero releases into that same session;
- asynchronous request completion and cleanup must both be generation/session safe;
- stale generations must be unable to mutate frontend accepted state **or** issue semantically stale releases that override the current generation;
- keyboard/gamepad desired state remains neutralized on blur, fullscreen ownership loss, stop, terminal session transition, return to library, component teardown and actual session replacement;
- batching remains bounded to 32 updates and coalescing behavior remains unchanged.

A preferred design is to make the long-lived input pump owned by `session.sessionId` and read mutable preferences/state through refs. An equivalent design is acceptable if it can prove stale commands are rejected or suppressed.

### SPRR-002 — Lifecycle-level input regression coverage

Current unit tests validate pure helpers such as `createInputAcceptanceGuard` and `diffInputState`, but they do not reproduce the actual React/effect/IPC ordering that exposed SPRR-001.

**Required coverage:**

- same session, controller preference changes, old request pending, new generation starts, new input is asserted, old cleanup finishes: old release must not override the new generation;
- actual session replacement releases the previous session's accepted controls and begins the new session neutral;
- stale request completion after cleanup cannot restore frontend accepted state;
- keyboard plus W3C-standard gamepad state is released correctly;
- batches larger than 32 are split or otherwise bounded without losing releases;
- terminal or closed control channels remain best-effort and do not deadlock teardown.

The test may use a component harness or an extracted session-owned input pump, but it must test the asynchronous command ordering rather than only a map helper.

### SPRR-003 — Retry-safe durable play-history settlement

The current history registry claims and removes a pending launch record before writing the final outcome to SQLite. This prevents competing success/failure outcomes from double-writing, but if the durable write fails, the in-memory decision is lost and the history row can remain unresolved.

**Required behavior:**

- the first legitimate outcome decides the session's history result exactly once;
- once decided, a competing later event cannot change success to failure or failure to success;
- a transient database failure must not discard the decided outcome;
- the same decided outcome may be retried safely until it is durably persisted or the application is shutting down;
- duplicate lifecycle callbacks do not create duplicate history rows;
- history persistence failure remains diagnostic and never destabilizes the emulator session;
- shutdown behavior is bounded and clearly defined if a retry cannot complete.

An acceptable model is `Pending -> Decided(success|failure) -> Persisted`, keyed by exact session ID.

### SPRR-004 — End-to-end history lifecycle tests

History tests currently exercise claim-map semantics more strongly than the real session/database lifecycle.

**Required regression coverage:**

- launch history is pending at runtime-ready;
- first accepted presentation changes the actual stored row to success;
- crash/exit/stop before first presentation changes the stored row to failure;
- no-first-frame reporting changes the stored row to failure only when no presentation has already been accepted;
- first-presentation and terminal callbacks racing settle exactly one outcome;
- retry/new-session history IDs remain independent;
- a simulated persistence failure retains the decided outcome for retry;
- stale/cross-session presentation acknowledgements cannot settle another row.

Use an in-memory SQLite catalog or equivalent real persistence surface rather than validating only a generic `HashMap`.

### SPRR-005 — Strict outstanding-frame acknowledgement semantics

The current mailbox rejects never-delivered and future sequences and does not double-count duplicate acknowledgements. However, it accepts an acknowledgement for `last_presented_sequence` before checking whether a newer frame is now the exact outstanding delivered frame.

**Required behavior:**

- with the current one-outstanding-frame polling contract, an acknowledgement is accepted only when it matches the exact delivered sequence eligible for acknowledgement;
- duplicate idempotence is allowed only when it cannot mask the fact that a newer frame is outstanding;
- an old duplicate after delivery of a newer frame is rejected or explicitly represented as a separate harmless duplicate path that cannot satisfy/clear the newer outstanding frame;
- presented metrics increment exactly once per accepted presentation;
- the first-success history path cannot be triggered by a stale duplicate;
- tests cover: duplicate current ACK, duplicate old ACK after newer delivery, future ACK, backward ACK, mailbox-dropped ACK and oversized duration.

If the design chooses to tolerate stale duplicate ACKs as no-op protocol noise, that behavior must be explicit and must not count as acknowledgement of the current outstanding frame.

### SPRR-006 — Frame-producer disconnect diagnosis

The nonblocking FIFO cancellation design fixed indefinite teardown. The reader currently treats `Ok(0)` as a retry condition indefinitely, which is required before the first writer connects but is ambiguous after a producer has already supplied frame bytes.

**Required behavior:**

- before a writer has ever produced frame data, FIFO EOF may be treated as “producer not connected yet” while respecting cancellation;
- after the stream has received frame bytes, producer EOF/disconnect must become a bounded frame-stream diagnostic rather than silent polling until some other timeout;
- an incomplete header/payload followed by disconnect must fail as a frame transport/protocol error;
- cancellation always wins over producer-disconnect reporting during intentional teardown;
- no busy spin; polling/wakeup remains bounded;
- supervisor diagnostics must preserve the actionable frame-stream cause.

### SPRR-007 — Stronger frame-reader lifecycle tests

Current transport tests demonstrate cancellation with no writer and with a writer held open, but do not deterministically exercise every lifecycle claim in the specification.

**Required coverage:**

- cancellation before or immediately after reader startup with no writer;
- cancellation while a writer is connected but idle;
- cancellation during a partial header or partial payload;
- producer disconnect after at least one successful frame;
- finalized supervisor session does not leave a logically active frame-reader;
- supervisor drop/app shutdown terminates child and frame transport without an unbounded join;
- cancellation cannot publish/authenticate a fabricated partial frame.

Tests should expose synchronization hooks or deterministic state rather than depending on an arbitrary sleep to infer that a particular race occurred.

### SPRR-008 — Qualification output-path containment

The frame-seam script validates machine identifiers but currently accepts an arbitrary relative explicit report path and canonicalizes it after directory creation.

**Required behavior:**

- default reports remain under `artifacts/in-app-gameplay/`;
- relative explicit report paths are resolved under the selected/default evidence root and cannot escape it with `..`, symlinks or separator tricks;
- deliberately supplied absolute output paths may be accepted if explicitly supported by policy;
- directory creation happens only after containment validation;
- invalid paths return a stable non-zero usage/policy status and actionable message;
- tests cover simple relative names, nested relative names, traversal attempts, absolute permitted paths and machine-name validation.

### SPRR-009 — Package provenance must match packaged executable bytes

The package staging script correctly computes `mame_sha256` from the staged executable and records it in `runtime-provenance.txt`. Runtime audit identity intentionally avoids rehashing the very large executable on every lookup.

The second review found that package qualification does not explicitly prove that the provenance digest embedded in the final installed/package runtime still matches the final `bin/mame` bytes.

**Required behavior:**

- real-runtime package qualification recomputes SHA-256 for the packaged/installed MAME executable and compares it to `runtime-provenance.txt`;
- Debian and AppImage qualification both verify this invariant;
- mismatch fails closed with a clear package-integrity diagnosis;
- runtime code may continue to consume the verified package provenance without hashing hundreds of megabytes for every audit operation;
- documentation must distinguish “package provenance verified by qualification” from “runtime continuously rehashes executable”.

This is primarily package-integrity hardening under the current package-owned trust model, not a reason to hash the executable on every application action.

### SPRR-010 — Correct provenance regression semantics

The current regression named as changed-binary coverage writes identical executable bytes and changes only the manifest digest.

**Required behavior:**

- one regression proves identical executable/digest across different AppImage-like mount paths remains current;
- one regression writes different executable bytes and proves the corresponding different digest invalidates the stored audit;
- one regression proves legacy digest-less bundled audits require requalification;
- optional negative qualification test proves a manifest/executable hash mismatch is rejected by package validation;
- test names accurately describe what they prove.

### SPRR-011 — First visible frame and durable-success timing

The frontend currently queues a presentation acknowledgement after drawing a frame and sends that acknowledgement on the next frame-poll request. Under the written contract, durable success means Rust accepted the presentation acknowledgement, so this is not inherently incorrect. However, there is a narrow interval where a frame was visibly drawn but the process exits before the acknowledgement reaches Rust, allowing terminal failure to win.

**Required behavior:**

Choose and document one precise contract:

1. **Accepted-ack contract:** durable success occurs only when Rust accepts the presentation acknowledgement. Keep the current semantics, explicitly document the small draw-to-ack interval, and test it; or
2. **Immediate presentation contract:** send/commit first-frame acknowledgement immediately after a successful draw, reducing the interval while maintaining backend validation.

Whichever contract is chosen must remain backend-authoritative and must not allow frontend-only state to write durable history directly.

**Decision:** Use the **accepted-ack contract**. The existing next-poll acknowledgement remains the durable-success boundary: a successfully drawn frame is queued for acknowledgement, and only Rust accepting that exact sequence can settle history success. A process exit between draw and accepted ACK therefore remains a failure by definition. This avoids an additional per-frame IPC command and keeps durable truth entirely in the backend.

### SPRR-012 — Final source review and completion truth

The previous remediation TODO currently says no known Moderate-or-higher defect remains and all autonomous remediation is complete. That statement is historical evidence of the state at the time; it must not be silently rewritten as though the second review never happened.

**Required behavior:**

- preserve the previous TODO's historical record;
- add a clear pointer from prior documentation to this second remediation during final reconciliation;
- this second TODO becomes the active completion source for second-review work;
- do not claim autonomous remediation complete until every non-deferred SPRR task has source/test/CI evidence;
- all fourteen desktop/private-ROM acceptance items remain deferred and unchecked unless the user explicitly resumes them.

## 4. Architecture invariants to preserve

The second remediation must not regress the strengths established by the original and first post-review implementations:

1. Work directly on `master`; no per-task branches or PRs.
2. MAME remains the emulator, machine-state, input-semantics and native-audio authority.
3. Rust remains the process/session/filesystem/transport trust boundary.
4. React/TypeScript remains the in-app presentation and browser/gameplay focus layer.
5. No shell-string MAME launch construction.
6. No generic WebView filesystem/process/socket capability.
7. Binary frame traffic remains separate from stdout/runtime-control traffic.
8. Frame storage remains bounded to latest-frame/backpressure semantics.
9. Session ID and authentication token remain session-scoped.
10. Unknown/stale/malformed frame data fails closed.
11. Native audio remains outside the frame transport.
12. Audit and launch share the same effective content-path calculation.
13. Unknown/stale/missing content remains gated before normal launch.
14. Unsupported platforms fail explicitly.
15. Diagnostic stdout/stderr remain bounded.
16. AppImage randomized mount paths do not make identical packaged MAME audits stale.
17. Automated `___empty` evidence is never represented as real-ROM desktop gameplay acceptance.
18. No desktop/private-ROM item is completed from CI evidence alone.

## 5. Implementation guidance

### 5.1 Session-owned input pump

Prefer a single input pump whose destructive lifecycle follows `session.sessionId`. Store changing values such as preferred controller ID and session running state in refs, or update them without triggering session cleanup.

A stale generation may discard its own asynchronous result, but must not send a release to a session now owned by a successor generation unless that release is still semantically correct for the successor's desired state.

### 5.2 Durable history state machine

Use backend-owned state representing both decision and persistence status. Claiming the outcome and persisting it are separate operations.

A suggested state model:

- `Pending(history_id)`
- `Decided { history_id, succeeded }`
- removed only after durable persistence succeeds.

A competing callback may observe `Decided` but cannot change the outcome. Retrying the same persistence is safe.

### 5.3 Presentation acknowledgements

Track whether the latest delivered sequence is outstanding. The model should make it impossible for an old duplicate to be mistaken for acknowledgement of a newer outstanding frame.

### 5.4 FIFO connection state

Track whether the reader has observed any producer bytes or at least one complete frame. EOF before producer connection and EOF after an established producer are different states and should produce different behavior.

### 5.5 Package provenance verification

Keep the efficient runtime audit model: use stable package provenance during normal operation. Verify the manifest-to-binary hash invariant at package qualification/install-test time and record that evidence.

## 6. Required automated regression matrix

At minimum, exact-source automation must cover:

- same-session input effect recreation cannot release successor-generation controls;
- actual session replacement releases the old session and starts the new session neutral;
- stale input completion cannot mutate successor bookkeeping;
- history persistence failure retains its decided result and can retry;
- real SQLite history lifecycle from launch to presentation/failure;
- duplicate current frame ACK remains idempotent;
- old duplicate ACK after newer delivery follows the documented strict behavior;
- dropped/future/backward ACKs fail;
- producer EOF after established frame traffic becomes a diagnostic;
- cancellation works with no writer, idle writer, partial header and partial payload;
- finalized supervisor/drop paths terminate frame transport;
- relative qualification output cannot escape evidence root;
- absolute output policy is explicitly tested;
- staged/installed Debian MAME hash matches runtime provenance;
- AppImage-extracted MAME hash matches runtime provenance;
- changed binary bytes plus changed digest invalidate audit while mount-path changes do not;
- legacy digest-less audit invalidation remains correct;
- all existing frame parsing, BGRX conversion, sizing, metrics, audit gating, content paths, packaging and documentation tests remain green.

## 7. Exact-source qualification requirements

After the final behavior-changing commit, run and record:

- Rust formatting;
- Clippy/lint;
- complete Rust tests;
- TypeScript formatting/lint/typecheck;
- Vitest/component tests;
- production Tauri build;
- Linux packaging;
- Windows packaging;
- macOS packaging;
- security workflow;
- real-MAME Linux Debian/AppImage build;
- ROM-less `___empty` headless snapshot/frame-seam checks;
- installed Debian package qualification;
- AppImage extraction/package qualification;
- runtime-provenance-to-executable hash verification;
- package artifact upload;
- documentation CI after final reconciliation.

Record exact implementation SHA, run IDs, artifact IDs, expiration, AppImage/deb hashes, sizes and bundled MAME version. Do not reuse the prior `08575954…` runtime artifact as final evidence after behavior changes.

## 8. Deferred acceptance scope

The user's 2026-10-09 deferral remains in force. The following fourteen original items stay unchecked and are outside autonomous remediation until explicitly resumed:

- reproduce the user's private-ROM machine case;
- determine the user's real-host causes of prior `CONTROL_CHANNEL_CLOSED` / `MAME_EARLY_EXIT`;
- record real available/best-available/missing-content baselines;
- characterize `screen:pixels()` on representative raster/vector machines;
- measure diagnostic capture overhead at native/high resolutions;
- verify the selected frame seam with real locally available machines beyond `___empty`;
- verify native audio/mute during actual gameplay;
- test representative raster/vector/rotated/resolution machines;
- capture the original locally installed MAME effective `rompath`;
- launch the latest exact-source AppImage on the user's graphical desktop and prove gameplay stays inside Tauri;
- measure real desktop frame/input/audio/CPU/memory/fullscreen/stop-relaunch behavior;
- record dated private desktop qualification evidence;
- repeat desktop qualification after later runtime-affecting changes;
- perform normal first-run user-facing acceptance with approved local media.

Do not invent evidence and do not repeatedly ask for access while the deferral remains active.

## 9. Completion criteria

This second remediation is complete only when:

1. every non-deferred SPRR item in the companion TODO is checked with appropriate source/test/CI evidence;
2. the same-session stale-release race is eliminated and reproduced by a regression that would fail on the reviewed baseline;
3. no decided history outcome can be lost solely because its first SQLite finalization attempt fails;
4. frame acknowledgement behavior exactly matches its documented outstanding-frame contract;
5. established FIFO producer disconnects are diagnosed instead of silently polling forever;
6. qualification report paths cannot escape policy unintentionally;
7. final package qualification proves the packaged executable matches its recorded provenance digest;
8. the final exact-source full-runtime artifact is green and recorded;
9. prior remediation documentation points to this second remediation without erasing historical evidence;
10. all fourteen user-deferred desktop/ROM items remain unchecked unless explicitly resumed.

A valid pre-desktop final state is: **all second post-review autonomous remediation complete; only the fourteen explicitly deferred desktop/ROM acceptance items remain**.
