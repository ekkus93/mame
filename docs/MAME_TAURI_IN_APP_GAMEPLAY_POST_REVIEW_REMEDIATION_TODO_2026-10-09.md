# MAME Tauri In-App Gameplay Post-Review Remediation TODO

**Date:** 2026-10-09  
**Status:** Active implementation checklist  
**Specification:** [MAME Tauri In-App Gameplay Post-Review Remediation Specification](MAME_TAURI_IN_APP_GAMEPLAY_POST_REVIEW_REMEDIATION_SPEC_2026-10-09.md)  
**Parent TODO:** [MAME Tauri In-App Gameplay TODO](MAME_TAURI_IN_APP_GAMEPLAY_TODO_2026-10-08.md)  
**Reviewed baseline:** `b412c3840da4ed665f3e249722e302abb3c7f2f8`

This is the authoritative implementation checklist for findings discovered by the 2026-10-09 source-level review of the in-app gameplay work. The prior TODO remains the historical feature checklist and evidence ledger. Do not erase or rewrite its completed history.

## Execution policy

- Work directly on `master` with exact-head compare-and-swap commits.
- Use Ralph Bridge for GitHub and CI operations.
- Do not create per-task branches or pull requests.
- Re-read this TODO and current `master` at the start of each run and after every successful write.
- Prefer fixing and qualifying one coherent vertical slice over status-only commits.
- Do not mark an item complete without code/test/CI evidence appropriate to the requirement.
- The user's desktop/ROM deferral remains active. Deferred items below stay unchecked until the user explicitly resumes them.
- A transient tool error, CI still running, ordinary test failure, lint failure or implementation bug is not a user blocker.
- After any launch/frame/input/provenance/transport behavior change, rebuild and qualify the exact-source full runtime before treating package evidence as current.

## Phase 0 — Establish the post-review remediation baseline

- [x] **PRR-BASE-001:** Record the exact `master` SHA used when implementation begins and verify the two post-review documents are present.
- [x] **PRR-BASE-002:** Re-read the parent TODO and preserve its 14 deferred desktop/ROM items unchanged in completion meaning.
- [x] **PRR-BASE-003:** Inspect current exact-head CI and ensure no unrelated failing automated gate is being mistaken for a review finding.
- [x] **PRR-BASE-004:** Map each code change made by this remediation to one or more PRR requirement IDs in commit messages or reconciliation notes.

**Exit gate:** The implementation run is anchored to a known exact head and the deferred/user-dependent scope is not accidentally reopened.

> **2026-10-09 implementation baseline:** Started from exact `master` `f3bea95dedc03613909e32e895cc218255545e2b`. The parent checklist's 14 deferred desktop/ROM items remain unchanged. The new documents' exact-head documentation CI was inspected. Changes to frame validation and frontend input generations are mapped to PRR-FRAME-* and PRR-IN-*; full exact-head CI still determines final qualification. Checkboxes in this remediation TODO represent implemented code and focused added regressions, not user-desktop acceptance.

## Phase 1 — Correct startup success and durable play history

- [ ] **PRR-HIST-001:** Change launch history so runtime-control readiness alone does not finalize `succeeded=true`.
- [ ] **PRR-HIST-002:** Associate the pending history record with the exact supervised session using backend-owned state.
- [ ] **PRR-HIST-003:** Finalize success only when Rust accepts the first valid presentation acknowledgement for that session.
- [ ] **PRR-HIST-004:** Finalize failure when the session reaches failed/crashed/exited or user stop before first presentation.
- [ ] **PRR-HIST-005:** Make history settlement idempotent across presentation/exit races and repeated terminal callbacks.
- [ ] **PRR-HIST-006:** Ensure stale/cross-session/duplicate presentation acknowledgements cannot settle another session's history.
- [ ] **PRR-HIST-007:** Preserve diagnostic reporting when history finalization itself fails without corrupting emulator/session state.
- [ ] **PRR-HIST-008:** Add regressions for ready-without-frame, first-presented success, pre-frame crash/exit, stop-before-frame, retry/new-session, and duplicate acknowledgement behavior.

**Exit gate:** Durable history's success bit now means the in-app gameplay startup contract reached first valid presentation, not merely process/control readiness.

## Phase 2 — Eliminate asynchronous gameplay-input teardown races

- [x] **PRR-IN-001:** Introduce an input generation/session ownership token for each active `GameSurface` input pump.
- [x] **PRR-IN-002:** Prevent an old/in-flight `setMameInputs` completion from mutating `acceptedInputsRef` after cleanup or session replacement.
- [x] **PRR-IN-003:** Invalidate the active generation before teardown clears desired/accepted state.
- [x] **PRR-IN-004:** Best-effort release all known accepted non-zero controls while the old session channel is still usable.
- [x] **PRR-IN-005:** Ensure blur, fullscreen transitions, stop, return-to-library, terminal process events and component teardown neutralize desired input.
- [x] **PRR-IN-006:** Ensure a new session starts with no desired or accepted state inherited from the previous session.
- [x] **PRR-IN-007:** Preserve coalescing and the 32-update message cap.
- [x] **PRR-IN-008:** Add an asynchronous regression that resolves a previously in-flight input promise after teardown and proves stale state is not restored.
- [ ] **PRR-IN-009:** Add regressions for session replacement and balanced release behavior across keyboard plus gamepad state.

**Exit gate:** No asynchronous frontend completion can create stuck/reasserted gameplay input after ownership has ended.

## Phase 3 — Harden frame protocol and presentation accounting

- [x] **PRR-FRAME-001:** Define the supported native frame flag mask and reject unknown flag bits in Rust.
- [x] **PRR-FRAME-002:** Keep TypeScript unknown-flag rejection as defense in depth.
- [x] **PRR-FRAME-003:** Track the exact outstanding delivered frame eligible for acknowledgement.
- [x] **PRR-FRAME-004:** Require presentation acknowledgement to match the exact outstanding delivered sequence under the current polling model.
- [x] **PRR-FRAME-005:** Reject acknowledgements for mailbox-dropped/never-delivered sequences.
- [x] **PRR-FRAME-006:** Preserve idempotence only for a legitimately already-acknowledged sequence; do not increment presented metrics twice.
- [x] **PRR-FRAME-007:** Keep duration, session-state, sequence and protocol bounds fail-closed.
- [x] **PRR-FRAME-008:** Add Rust tests for unknown flags, skipped/dropped sequence acknowledgement, duplicate acknowledgement and future/backward sequences.
- [ ] **PRR-FRAME-009:** Run existing Rust and TypeScript frame protocol suites and confirm no regression in BGRX conversion, dimensions, orientation, stale-session or payload validation.

**Exit gate:** Native validation and presentation metrics cannot claim a frame was presented unless that frame was actually delivered and acknowledged correctly.

## Phase 4 — Make frame metrics semantically precise

- [ ] **PRR-METRIC-001:** Audit every field in `FrameMetricsSnapshot` and document its clock domain and exact meaning.
- [ ] **PRR-METRIC-002:** Rename or supersede `latestAgeMs` so it is explicitly backend latest-receive age rather than implied end-to-end latency.
- [ ] **PRR-METRIC-003:** Preserve separate last-received, last-presented, presentation-duration, dimensions, sequence and drop metrics.
- [ ] **PRR-METRIC-004:** Keep MAME `capture_timestamp_us` explicitly identified as a MAME/emulated-time-domain value unless a valid cross-clock conversion is implemented.
- [ ] **PRR-METRIC-005:** Update frontend status text and Diagnostics UI/types so labels match the actual semantics.
- [ ] **PRR-METRIC-006:** Add serialization/UI regression coverage for the revised metric names and compatibility behavior.

**Exit gate:** Diagnostics never represent backend receive-age as capture-to-screen latency.

## Phase 5 — Strengthen bundled-runtime audit provenance

- [ ] **PRR-PROV-001:** Choose a stable bundled runtime/package digest source, preferably build-generated SHA-256/provenance already produced by packaging.
- [ ] **PRR-PROV-002:** Extend audit provenance so randomized AppImage mount-path changes remain stable while changed bundled executable bytes invalidate prior audits.
- [ ] **PRR-PROV-003:** Define backward-compatible handling or explicit invalidation for existing audit rows without the digest.
- [ ] **PRR-PROV-004:** Avoid unnecessary repeated hashing of the large bundled executable if immutable packaged provenance can be verified and reused safely.
- [ ] **PRR-PROV-005:** Preserve external/development runtime provenance semantics.
- [ ] **PRR-PROV-006:** Add regression: identical packaged runtime, different AppImage mount path → audit remains current.
- [ ] **PRR-PROV-007:** Add regression: same MAME version/build text, different bundled runtime digest → audit becomes stale/invalidated.
- [ ] **PRR-PROV-008:** Re-run launch gating tests for complete, best-available, unknown/stale and missing-content classifications after provenance changes.

**Exit gate:** An audit is current only for the actual bundled runtime identity that produced it, without false invalidation from AppImage mount randomization.

## Phase 6 — Make frame-reader shutdown deterministic

- [ ] **PRR-LIFE-001:** Reproduce/test cancellation before the FIFO reader opens.
- [ ] **PRR-LIFE-002:** Add a deterministic wake/cancel path for a reader already blocked on an open FIFO.
- [ ] **PRR-LIFE-003:** Ensure stop, launch failure, child crash, app shutdown and supervisor drop all cancel the frame transport.
- [ ] **PRR-LIFE-004:** Ensure cancellation cannot authenticate or accept a fake gameplay frame.
- [ ] **PRR-LIFE-005:** Keep teardown bounded and avoid joining indefinitely on application/UI paths.
- [ ] **PRR-LIFE-006:** Add tests proving no frame-reader task remains logically active after finalized session teardown in both pre-open and post-open blocked states.

**Exit gate:** Fully finalized sessions cannot leave a frame-reader blocked indefinitely waiting for a producer.

## Phase 7 — Harden qualification tooling and evidence language

- [ ] **PRR-QUAL-001:** Validate or safely encode the machine identifier used by `scripts/tauri/qualify-mame-frame-seams.sh`, including its default report filename.
- [ ] **PRR-QUAL-002:** Add shell regressions for invalid machine identifiers and output-path behavior.
- [ ] **PRR-QUAL-003:** Preserve pre-build `bash -n` checks for gameplay qualification scripts in the real-runtime workflow.
- [ ] **PRR-QUAL-004:** Preserve `screen_status=not_measured` when zero diagnostic `screen:pixels()` samples are collected.
- [ ] **PRR-QUAL-005:** Audit comments/docs/output so `___empty` and ROM-less headless results are never described as real-game acceptance.
- [ ] **PRR-QUAL-006:** Ensure evidence notes distinguish automated package qualification, frame-seam smoke, and deferred desktop acceptance.

**Exit gate:** Qualification tooling cannot accidentally overstate what its evidence proves or create unsafe/ambiguous default evidence paths.

## Phase 8 — Regression and exact-source automated qualification

- [ ] **PRR-CI-001:** Run/qualify Rust formatting and Clippy/lint on the exact implementation head.
- [ ] **PRR-CI-002:** Run/qualify the complete Rust unit/integration suite, including new history/input/frame/provenance/lifecycle regressions.
- [ ] **PRR-CI-003:** Run/qualify TypeScript lint/typecheck and Vitest, including the asynchronous input lifecycle regression.
- [ ] **PRR-CI-004:** Qualify ordinary Linux packaging on the exact head.
- [ ] **PRR-CI-005:** Qualify Windows and macOS packaging workflows required by the existing project policy.
- [ ] **PRR-CI-006:** Build the full real-MAME Linux Debian package and AppImage from the exact remediation head.
- [ ] **PRR-CI-007:** Pass real-MAME `___empty` headless snapshot smoke and corrected frame-seam qualification.
- [ ] **PRR-CI-008:** Pass installed-package/dependency/provenance checks and preserve the package artifact.
- [ ] **PRR-CI-009:** Record exact source SHA, workflow/run IDs, artifact ID/name, expiration, AppImage/deb hashes, sizes and bundled MAME version.
- [ ] **PRR-CI-010:** Treat every earlier AppImage as stale for final acceptance after the last launch/frame/input/provenance/transport behavior change.

**Exit gate:** The final autonomous remediation source is green across required exact-head tests and produces a preserved, source-pinned full runtime artifact.

## Phase 9 — Reconcile documentation and the original checklist

- [ ] **PRR-DOC-001:** Update the parent gameplay specification where metric/history/provenance semantics were clarified by this remediation.
- [ ] **PRR-DOC-002:** Add a post-review note to the original 2026-10-08 TODO pointing to this TODO as the active remediation checklist.
- [ ] **PRR-DOC-003:** Do not erase historical checkmarks/evidence in the original TODO; explicitly document any original checked item whose correctness was strengthened by remediation.
- [ ] **PRR-DOC-004:** Update package evidence with the final exact-source remediation artifact.
- [ ] **PRR-DOC-005:** Run documentation CI and fix real documentation failures.
- [ ] **PRR-DOC-006:** Perform a final source-level review of every PRR item and verify no actionable non-deferred defect found by this review remains open.

**Exit gate:** Repository documentation accurately describes the corrected implementation and does not claim deferred desktop acceptance.

## Phase 10 — Deferred desktop/ROM acceptance — do not execute until user resumes

The following items are deliberately carried forward from the parent TODO. They are **not implementation failures and are not complete**. Do not repeatedly ask the user for access while the deferral remains active.

- [ ] **DEFER-ORIG-003:** Reproduce the user's real machine case using private ROMs in `/home/phil/mame/roms` and capture complete launch/runtime/UI evidence.
- [ ] **DEFER-ORIG-005:** Determine the user's real-host cause(s) of earlier `CONTROL_CHANNEL_CLOSED` / `MAME_EARLY_EXIT` reports.
- [ ] **DEFER-ORIG-006:** Record real available, best-available/no-ROM and known-missing-content baselines.
- [ ] **DEFER-ORIG-007:** Characterize `screen:pixels()` dimensions/format/orientation/callback behavior on representative raster/vector machines.
- [ ] **DEFER-ORIG-008:** Measure diagnostic Lua `screen:pixels()` and callback overhead at native/high resolutions.
- [ ] **DEFER-ORIG-013:** Verify the selected headless/snapshot seam with real locally available machines, not only `___empty`.
- [ ] **DEFER-ORIG-028:** Verify native audio plus real mute state during actual gameplay.
- [ ] **DEFER-ORIG-030:** Test representative raster, vector, rotated and different-resolution/aspect machines.
- [ ] **DEFER-ORIG-031:** Capture original installed-MAME effective `rompath` and compare with application behavior.
- [ ] **DEFER-ORIG-051:** Launch the latest exact-source AppImage on the user's real graphical desktop and prove gameplay is inside Tauri with no separate visible MAME game window.
- [ ] **DEFER-ORIG-052:** Measure presented frame rate, receive/presentation ages, drops, emulation speed, CPU/memory, input, native sound, resize, fullscreen and stop/relaunch on the real desktop.
- [ ] **DEFER-ORIG-053:** Record exact AppImage path/hash/build/MAME/machine/ROM configuration/results/limitations/screenshots/logs in dated private qualification evidence.
- [ ] **DEFER-ORIG-054:** Repeat real-desktop qualification after any later launch/frame/pixel/package/runtime change; never validate a stale artifact.
- [ ] **DEFER-ORIG-060:** Perform normal first-run user-facing review with an isolated fresh profile and the user's approved local media.

**Resumption gate:** Only the user explicitly resumes this phase.

## Final completion gate

The post-review remediation is complete when:

- [ ] Every **PRR-*** item above is checked with code/test/CI evidence.
- [ ] No known Moderate-or-higher post-review defect remains.
- [ ] The final full real-runtime artifact is exact-source and green.
- [ ] The original TODO points to this remediation and retains its historical evidence.
- [ ] All 14 **DEFER-ORIG-*** items remain unchecked unless the user explicitly resumes and supplies/permits the required host evidence.
- [ ] The repository can accurately report: **all autonomous post-review remediation complete; only explicitly deferred desktop/ROM acceptance remains**.
