# MAME Tauri In-App Gameplay Post-Review Remediation Specification

**Date:** 2026-10-09  
**Status:** Active post-review remediation specification  
**Baseline reviewed:** `b412c3840da4ed665f3e249722e302abb3c7f2f8` on `master`  
**Companion TODO:** [MAME Tauri In-App Gameplay Post-Review Remediation TODO](MAME_TAURI_IN_APP_GAMEPLAY_POST_REVIEW_REMEDIATION_TODO_2026-10-09.md)  
**Parent specification:** [MAME Tauri In-App Gameplay Specification](MAME_TAURI_IN_APP_GAMEPLAY_SPEC_2026-10-08.md)  
**Prior checklist/evidence:** [MAME Tauri In-App Gameplay TODO](MAME_TAURI_IN_APP_GAMEPLAY_TODO_2026-10-08.md) and [package evidence](MAME_TAURI_IN_APP_GAMEPLAY_PACKAGE_EVIDENCE_2026-10-09.md)  
**Second-review follow-up:** [Second post-review remediation specification](MAME_TAURI_IN_APP_GAMEPLAY_SECOND_POST_REVIEW_REMEDIATION_SPEC_2026-10-09.md) and [active second-remediation TODO](MAME_TAURI_IN_APP_GAMEPLAY_SECOND_POST_REVIEW_REMEDIATION_TODO_2026-10-09.md)

## 1. Purpose

The 2026-10-08 in-app gameplay implementation established the intended architecture: MAME remains the emulator and native audio owner; Rust owns process supervision, filesystem and transport boundaries; a private session-scoped binary frame path carries video; and React/TypeScript owns the visible game surface and browser/gameplay input focus.

A source-level post-implementation review of current `master` found that the architecture is sound but identified several correctness, provenance, metrics, teardown, and qualification gaps that must be remediated before the non-deferred implementation can be considered fully reconciled.

This document defines those remediations. It does **not** replace the product contract in the parent specification. It tightens implementation semantics and evidence requirements where the review found ambiguity or defects.

## 2. Review baseline and evidence boundary

The reviewed repository state is `b412c3840da4ed665f3e249722e302abb3c7f2f8`.

At that baseline:

- the original gameplay TODO records 46 of 60 items checked;
- 14 original items remain explicitly deferred by user decision because they require the user's graphical desktop, private ROMs, original installed MAME, or user-specific configuration;
- exact-source Tauri tests and Linux/Windows/macOS packaging passed for the latest implementation revision;
- full real-MAME Linux Debian/AppImage packaging and ROM-less headless frame qualification passed;
- automated evidence does **not** prove playable real-ROM desktop gameplay, native audio behavior, controller behavior on actual hardware, raster/vector/rotation/high-resolution performance, original installed-MAME path parity, or normal first-run user acceptance.

The post-review remediation must preserve this distinction. No deferred acceptance item may be marked complete from synthetic, `___empty`, headless, unit-test, or packaging evidence alone.

## 3. Findings that require remediation

### PRR-001 — Startup success and play-history semantics

Current launch history is finalized as successful when runtime-control reaches the running state, before the frontend has presented a first valid gameplay frame.

This conflicts with the product contract: a successful in-app gameplay start requires both runtime readiness and a valid first frame presented in the Tauri surface.

**Required behavior:**

- a launch-history entry begins before spawn as it does today;
- reaching runtime-control readiness does not finalize history success;
- the first valid frame-presentation acknowledgement for the active session finalizes the pending history entry as successful exactly once;
- terminal failure, crash, startup failure, no-first-frame failure, or user stop before first presentation finalizes it as unsuccessful;
- duplicate, stale, or cross-session acknowledgements cannot finalize the wrong history record;
- history finalization failure is diagnostic and does not corrupt session lifecycle;
- retries create independent history records.

### PRR-002 — Race-free gameplay input teardown

The current frontend input pump can have an asynchronous `setMameInputs` request complete after effect cleanup. That stale completion can repopulate accepted-input bookkeeping after teardown has already cleared it.

**Required behavior:**

- every gameplay input pump instance has a generation/session identity;
- asynchronous input completions may mutate accepted state only while their generation is current;
- blur, fullscreen transitions, component teardown, session replacement, stop, process termination, and return-to-library neutralize desired state;
- releases for all known accepted non-zero inputs are sent best-effort before or during teardown where the channel remains usable;
- stale completions after cleanup are ignored;
- a new session starts with empty desired/accepted input state;
- no old session can release or reassert inputs into a new session;
- the 32-update cap and coalescing behavior remain intact.

### PRR-003 — Native frame protocol validation

The Rust frame parser is the native trust boundary and must reject every unsupported protocol bit itself.

**Required behavior:**

- frame flags are explicitly masked and unknown bits are rejected in Rust;
- TypeScript retains equivalent validation as defense in depth;
- version, dimensions, stride, payload length, pixel format, orientation, session identity, authentication token, header lengths, reserved fields, and monotonic sequence checks remain bounded and fail closed;
- malformed frames never become presentation candidates.

### PRR-004 — Exact presentation acknowledgement accounting

Presentation metrics must represent frames that were actually delivered to the WebView.

**Required behavior:**

- a presentation acknowledgement is accepted only for the exact most recently delivered, not-yet-acknowledged frame for that polling contract;
- acknowledgements cannot move backward, jump forward, fabricate an undelivered sequence, or double-count a previously acknowledged frame;
- session state must still be running;
- presentation duration remains bounded;
- tests cover dropped-mailbox frames and attempts to acknowledge skipped sequences.

If the transport evolves to allow multiple outstanding delivered frames, the acknowledgement model must be explicitly redesigned rather than weakening validation implicitly.

### PRR-005 — Frame age and latency metric semantics

Current `latestAgeMs` is backend receive age, not end-to-end capture-to-presentation latency.

**Required behavior:**

- existing metrics are renamed or documented so their clock domain and meaning are unambiguous;
- backend receive age, time since last presented frame, frontend draw duration, frame sequence/drop counts, and source frame dimensions remain separately visible;
- do not subtract MAME emulated-time timestamps from host wall/monotonic clocks;
- true capture-to-presentation latency may be added only if a shared/convertible clock domain is established;
- diagnostics and UI must not label receive age as end-to-end latency.

### PRR-006 — Stable bundled-runtime audit provenance

Replacing randomized AppImage mount paths with `<bundled-mame>` correctly avoids false audit invalidation, but version/build strings alone are insufficient to prove that the bundled executable bytes are unchanged.

**Required behavior:**

- bundled-runtime audit provenance includes a stable package/runtime digest or equally strong immutable build identifier;
- AppImage mount-path changes do not invalidate an otherwise identical bundled runtime;
- changed bundled executable bytes invalidate previously stored audit results even if the reported MAME version/build text is identical;
- external/development runtime provenance remains path/identity aware;
- migrations or compatibility handling for existing stored audit rows are explicit and tested.

The digest should preferably come from package/runtime provenance generated during build rather than hashing a very large executable on every launch if that can be avoided safely.

### PRR-007 — Deterministic frame-reader shutdown

Frame transport cancellation should not depend solely on the producer eventually closing an already-open FIFO.

**Required behavior:**

- session stop, launch failure, app shutdown, supervisor drop, and child crash reliably unblock/terminate the frame-reader path;
- cancellation remains session-scoped and cannot connect an unrelated writer;
- reader threads do not survive a fully finalized session indefinitely;
- teardown does not introduce an unbounded wait on the UI/backend path;
- tests cover cancellation before FIFO open and cancellation after the reader is already blocked.

An implementation may use a retained cancellation writer, poll/select-style wake mechanism, nonblocking FIFO strategy, or another bounded local primitive, but the shutdown contract must be demonstrable.

### PRR-008 — Qualification-tool input hardening

Host-side qualification tools are evidence tooling and must not accidentally create ambiguous output paths or misleading success reports.

**Required behavior:**

- machine identifiers accepted by frame-seam tooling are validated to the same safe short-name policy used by the application, or safely encoded for filenames;
- output paths are always under the explicitly selected evidence directory unless an absolute output is deliberately supplied;
- shell syntax checks run before expensive native builds in CI;
- zero diagnostic `screen:pixels()` samples remain `not_measured`, never `ok`;
- ROM-less `___empty` evidence remains clearly labeled as smoke/package evidence rather than real-game acceptance.

## 4. Architecture invariants to preserve

The remediation must not regress the design strengths already present:

1. **No shell command construction.** MAME launches use typed/discrete argv.
2. **No generic WebView process/filesystem/socket capability.**
3. **Video data plane remains separate from stdout/runtime-control traffic.**
4. **Frame memory remains bounded.** Prefer latest-frame replacement over queues that accumulate latency.
5. **Session identity and authentication remain scoped and unpredictable.**
6. **MAME remains authoritative for machine execution, content validation, native audio and machine input semantics.**
7. **TypeScript owns presentation only.** It does not emulate MAME output.
8. **Native audio does not travel through Tauri/WebView PCM paths.**
9. **Audit and launch use the same effective content-path computation.**
10. **Unknown/stale/missing content fails closed before normal catalog launch.**
11. **Unsupported platforms fail explicitly rather than silently claiming gameplay support.**
12. **Terminal session diagnostics remain bounded and available after failure.**
13. **No separate visible MAME gameplay window is accepted as in-app gameplay success.**
14. **No desktop/ROM acceptance claim is inferred from CI-only evidence.**

## 5. Implementation requirements

### 5.1 History ownership

The backend must own durable launch-history truth. Frontend state alone must not be trusted to mark a launch successful.

Associate the pending history record with the supervised session or a session-scoped success-finalizer. The preferred success trigger is the first valid presentation acknowledgement accepted by the Rust frame mailbox. Terminal session finalization must settle any still-pending history record as unsuccessful.

History settlement must be idempotent. A race between first presentation and terminal exit must result in one durable outcome and no duplicate database writes.

### 5.2 Input generation model

The frontend should maintain a monotonically increasing input-generation identifier keyed to the gameplay session/effect instance.

Each asynchronous `setMameInputs` completion captures its generation and session ID. It may update `acceptedInputsRef` only if both still match the active input owner.

Cleanup must invalidate the generation before clearing local state. Release attempts from cleanup must never be followed by stale promise handlers restoring old accepted values.

### 5.3 Frame acknowledgement state

The mailbox must distinguish at least:

- latest received sequence;
- latest delivered sequence;
- whether that delivered sequence has been acknowledged;
- latest acknowledged/presented sequence.

With the current one-frame-at-a-time frontend polling model, an accepted acknowledgement should equal the exact outstanding delivered sequence.

### 5.4 Metrics naming

Use names that describe observable facts. Suitable examples include:

- `latestReceivedAgeMs`;
- `lastPresentedAtEpochMs`;
- `lastPresentationDurationUs`;
- `received`, `dropped`, `delivered`, `presented`;
- `lastCaptureTimestampUs` with explicit documentation that it is MAME/emulated-time-domain data.

If compatibility requires retaining `latestAgeMs`, document it as backend receive age and add a replacement field before later removing it.

### 5.5 Runtime provenance

Persist an immutable bundled-runtime identifier in audit provenance. A build-generated SHA-256 is preferred where already available.

The application must avoid rehashing hundreds of megabytes on every query if package provenance can be verified once and cached safely. Any cache must be bound to immutable packaged resources rather than writable user state.

## 6. Regression requirements

At minimum, add automated tests for:

- launch history remains pending at runtime-ready;
- first valid presentation settles history success once;
- terminal failure before first presentation settles history failure;
- duplicate/stale/cross-session presentation acknowledgements cannot alter history;
- in-flight input completion after teardown cannot repopulate accepted input state;
- session replacement cannot inherit old desired/accepted input;
- teardown release generation is balanced and bounded;
- Rust rejects unknown frame flag bits;
- presentation ack must match the exact outstanding delivered frame;
- dropped mailbox frames cannot be acknowledged as presented;
- metrics naming/serialization preserves intended clock semantics;
- identical AppImage runtime with different mount path preserves audit;
- same reported MAME version with different runtime digest invalidates audit;
- frame reader cancellation before open and while blocked after open terminates cleanly;
- qualification script rejects unsafe/invalid machine identifiers;
- existing frame parsing, pixel conversion, sizing, sequence, startup/stall, content-path and package regressions continue to pass.

## 7. Qualification requirements

Every implementation-changing remediation commit must be qualified from its exact source head.

Required automated gates:

- Rust formatting;
- Clippy/lint;
- Rust unit/integration tests;
- TypeScript lint/typecheck;
- Vitest;
- Linux packaging;
- real-runtime Linux Debian/AppImage qualification;
- headless real-MAME frame smoke;
- frame-seam shell syntax and ROM-less smoke;
- Windows/macOS packaging jobs where they are part of the existing required workflow;
- documentation build for documentation changes.

A package produced before a launch/frame/input/provenance/transport behavior change is stale for final gameplay acceptance. Record the exact source SHA, artifact ID/path, hash, bundled MAME version and relevant CI run IDs.

## 8. Deferred real-desktop acceptance

The user explicitly deferred every original TODO item requiring the graphical desktop, private ROMs in `/home/phil/mame/roms`, original installed MAME, or personal runtime configuration.

Carry forward these original checklist items as deferred, unchecked acceptance work:

- original items 3, 5 and 6: reproduce/diagnose the user case and record real content baselines;
- items 7 and 8: real raster/vector `screen:pixels()` characterization and performance measurement;
- item 13: real-machine headless/frame-seam validation beyond `___empty`;
- item 28: native audio and mute behavior with actual gameplay;
- item 30: raster/vector/rotation/resolution representative machines;
- item 31: original installed-MAME effective ROM path;
- items 51–54: real desktop AppImage gameplay, performance/evidence capture and repeat qualification after relevant fixes;
- item 60: normal first-run user-facing review.

These remain **deferred, not complete**. Do not repeatedly request access while the deferral is active. Resume only when the user explicitly says to resume desktop/ROM qualification.

## 9. Performance risk to preserve for later acceptance

The current initial Linux path can require multiple full-frame memory operations:

`MAME snapshot render → Lua string → FIFO → Rust buffer → Tauri IPC → JS typed view → BGRX→RGBA conversion → ImageData → staging canvas → visible canvas`.

The canvas-backing reuse optimization reduces avoidable clears/resizes, but it does not eliminate the principal full-frame copies/conversion.

Do not prematurely rewrite the transport solely from theoretical bandwidth. Preserve the current bounded architecture, fix correctness first, and use the deferred real-machine measurements to decide whether snapshot/Lua/WebView copy overhead requires a native capture adapter, WebGL upload path, or another targeted optimization.

## 10. Completion criteria

This post-review remediation is complete only when:

1. every non-deferred item in the companion TODO is checked with source/test/CI evidence;
2. no known Moderate-or-higher review finding remains;
3. the exact-head full real-runtime package is green after the final behavior change;
4. the original TODO is annotated to point to this post-review remediation rather than claiming the reviewed implementation is final;
5. all desktop/ROM-dependent items remain visibly deferred unless the user explicitly resumes them;
6. documentation accurately distinguishes automated qualification from real user acceptance.

The final state before desktop acceptance may therefore be: **all post-review autonomous remediation complete; original 14 desktop/ROM acceptance items still deferred**.


## 11. Second-review follow-up (2026-10-09)

A later source review found additional independently actionable issues after this remediation had been qualified: same-session stale input releases, retry-safe history persistence, stricter outstanding-frame acknowledgement semantics, established frame-producer disconnect diagnosis, qualification output containment, and final package provenance-to-executable verification. Historical evidence and checked boxes in this remediation remain unchanged; new work is governed by the [second post-review remediation specification](MAME_TAURI_IN_APP_GAMEPLAY_SECOND_POST_REVIEW_REMEDIATION_SPEC_2026-10-09.md) and its [TODO](MAME_TAURI_IN_APP_GAMEPLAY_SECOND_POST_REVIEW_REMEDIATION_TODO_2026-10-09.md).
