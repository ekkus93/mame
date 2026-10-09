# MAME Tauri In-App Gameplay Second Post-Review Remediation TODO

**Date:** 2026-10-09  
**Status:** Active second post-review implementation checklist  
**Specification:** [MAME Tauri In-App Gameplay Second Post-Review Remediation Specification](MAME_TAURI_IN_APP_GAMEPLAY_SECOND_POST_REVIEW_REMEDIATION_SPEC_2026-10-09.md)  
**Previous remediation TODO:** [MAME Tauri In-App Gameplay Post-Review Remediation TODO](MAME_TAURI_IN_APP_GAMEPLAY_POST_REVIEW_REMEDIATION_TODO_2026-10-09.md)  
**Parent gameplay TODO:** [MAME Tauri In-App Gameplay TODO](MAME_TAURI_IN_APP_GAMEPLAY_TODO_2026-10-08.md)  
**Reviewed baseline:** `461b05174df7b80351b74f589eb40e11b5647390`

This checklist is the authoritative source of completion truth for issues discovered by the second source-level review. The previous remediation TODO remains a historical evidence ledger and must not be rewritten to hide that these issues were discovered later.

## Execution policy

- Use Ralph Bridge only for every GitHub, CI, branch, tag, artifact and repository operation.
- Work directly on `master` with exact-head compare-and-swap writes.
- Never create per-task branches or pull requests.
- Re-read current `master` and this TODO at the beginning of every run and after every successful write.
- Prefer coherent vertical slices with their regressions over tiny status-only commits.
- Do not check an item merely because related CI is green; verify that the code/test evidence actually proves the requirement.
- Any input/history/frame/FIFO/provenance behavior change invalidates the previously qualified runtime artifact for final acceptance.
- The user's 2026-10-09 graphical-desktop/private-ROM/original-installed-MAME deferral remains active. Keep all fourteen deferred items unchecked and do not repeatedly request access.
- Ordinary implementation bugs, test failures, formatting failures, transient API errors and CI still running are not user blockers.
- Do not move or overwrite the existing `v0.1.1` tag as part of this remediation unless the user explicitly requests release-tag work.

## Phase 0 — Establish the second-review baseline

- [ ] **SPRR-BASE-001:** Record the exact `master` SHA at implementation start and verify both second-remediation documents exist.
- [ ] **SPRR-BASE-002:** Re-read the first remediation TODO and confirm its 14 deferred items remain deferred in the new checklist.
- [ ] **SPRR-BASE-003:** Inspect current exact-head CI and separate gameplay-remediation failures from unrelated release/tag workflow behavior.
- [ ] **SPRR-BASE-004:** Map every second-remediation implementation commit to one or more SPRR IDs.
- [ ] **SPRR-BASE-005:** Treat prior runtime artifact source `0857595401d131080688baec2f3225b154787f2d` as historical evidence only once this remediation changes runtime behavior.

**Exit gate:** The second remediation is anchored to a known exact head without changing the user's deferred scope.

## Phase 1 — Eliminate same-session stale input releases

- [ ] **SPRR-IN-001:** Refactor the gameplay input pump so destructive release/teardown ownership follows the actual MAME session, not controller-preference changes.
- [ ] **SPRR-IN-002:** Ensure changing `preferredGamepadId` during a live session cannot enqueue stale zero releases that override the successor input generation.
- [ ] **SPRR-IN-003:** Ensure stale asynchronous `setMameInputs` completions cannot mutate current accepted-input bookkeeping.
- [ ] **SPRR-IN-004:** Ensure stale generations cannot send semantically stale releases into a currently owned session.
- [ ] **SPRR-IN-005:** Preserve correct releases on blur, fullscreen ownership loss, stop, terminal session transition, return-to-library, component teardown and real session replacement.
- [ ] **SPRR-IN-006:** Preserve keyboard/gamepad coalescing, W3C-standard controller filtering and the 32-update message bound.
- [ ] **SPRR-IN-007:** Ensure a genuinely new session begins with neutral desired/accepted input state.
- [ ] **SPRR-IN-008:** Add a regression that reproduces the reviewed same-session race and fails against baseline `461b05174df7b80351b74f589eb40e11b5647390`.
- [ ] **SPRR-IN-009:** Add asynchronous lifecycle coverage for actual session replacement, stale request completion and mixed keyboard/gamepad releases.

**Exit gate:** No old frontend generation can release or reassert controls in a live session now owned by a successor generation.

## Phase 2 — Make play-history finalization retry-safe

- [ ] **SPRR-HIST-001:** Replace remove-before-write settlement with explicit backend-owned pending/decided/persisted state or an equivalent retry-safe model.
- [ ] **SPRR-HIST-002:** Ensure first legitimate success/failure decides the outcome exactly once.
- [ ] **SPRR-HIST-003:** Prevent a later competing lifecycle event from changing an already decided outcome.
- [ ] **SPRR-HIST-004:** Retain a decided outcome after SQLite finalization failure so the same outcome can be retried safely.
- [ ] **SPRR-HIST-005:** Keep emulator/session lifecycle independent from history-persistence failure and record actionable diagnostics.
- [ ] **SPRR-HIST-006:** Define bounded app-shutdown behavior for any decided-but-not-yet-persisted history outcome.
- [ ] **SPRR-HIST-007:** Add real SQLite regression coverage for runtime-ready pending state, first-presentation success, pre-frame exit/crash/stop failure and no-first-frame failure.
- [ ] **SPRR-HIST-008:** Add race/retry regressions proving one decided outcome, independent retry sessions, persistence-failure retry and cross-session isolation.

**Exit gate:** A transient durable-history write failure cannot silently strand or discard a session's decided outcome.

## Phase 3 — Tighten presentation acknowledgement semantics

- [ ] **SPRR-ACK-001:** Explicitly represent the exact delivered frame currently eligible for acknowledgement.
- [ ] **SPRR-ACK-002:** Define whether an old duplicate ACK after a newer frame is delivered is rejected or accepted solely as protocol-noise no-op.
- [ ] **SPRR-ACK-003:** Ensure an old duplicate can never satisfy, clear or count as acknowledgement of a newer outstanding frame.
- [ ] **SPRR-ACK-004:** Preserve one-and-only-one presented metric increment per valid frame.
- [ ] **SPRR-ACK-005:** Ensure stale/duplicate ACK handling cannot incorrectly trigger first-frame durable history success.
- [ ] **SPRR-ACK-006:** Add regressions for duplicate current ACK, duplicate old ACK after newer delivery, future/backward ACKs, mailbox-dropped ACKs and duration bounds.

**Exit gate:** Presentation accounting has one documented, test-proven interpretation for every duplicate/stale acknowledgement case.

## Phase 4 — Diagnose producer disconnects and strengthen frame-reader lifecycle

- [ ] **SPRR-LIFE-001:** Track whether the FIFO has ever observed producer bytes or an established frame stream.
- [ ] **SPRR-LIFE-002:** Continue tolerating pre-connection FIFO EOF while respecting cancellation.
- [ ] **SPRR-LIFE-003:** Convert EOF after an established producer/stream into a bounded actionable frame-stream diagnostic.
- [ ] **SPRR-LIFE-004:** Convert partial-header or partial-payload producer disconnect into an actionable transport/protocol failure.
- [ ] **SPRR-LIFE-005:** Ensure intentional cancellation wins over disconnect reporting during stop/shutdown.
- [ ] **SPRR-LIFE-006:** Preserve bounded polling/wakeup with no busy spin or unbounded join.
- [ ] **SPRR-LIFE-007:** Add deterministic cancellation coverage with no writer, idle connected writer, partial header and partial payload.
- [ ] **SPRR-LIFE-008:** Add producer-disconnect-after-valid-frame regression coverage.
- [ ] **SPRR-LIFE-009:** Add supervisor-level coverage proving finalized/drop/shutdown sessions leave no logically active frame reader.
- [ ] **SPRR-LIFE-010:** Confirm cancellation cannot publish/authenticate a fabricated partial frame.

**Exit gate:** The frame transport distinguishes “producer not connected yet,” “producer disappeared unexpectedly,” and “session intentionally cancelled.”

## Phase 5 — Contain qualification evidence output paths

- [ ] **SPRR-QUAL-001:** Define the canonical evidence root for frame-seam reports.
- [ ] **SPRR-QUAL-002:** Resolve relative explicit report names beneath that evidence root.
- [ ] **SPRR-QUAL-003:** Reject relative traversal/symlink/path escape before directory creation.
- [ ] **SPRR-QUAL-004:** Explicitly define whether absolute report paths are allowed; if allowed, treat them as deliberate caller-selected destinations.
- [ ] **SPRR-QUAL-005:** Preserve existing safe MAME short-name validation.
- [ ] **SPRR-QUAL-006:** Add shell tests for simple relative output, nested relative output, traversal attempts, absolute-output policy and invalid machine identifiers.
- [ ] **SPRR-QUAL-007:** Keep zero diagnostic `screen.pixels()` samples as `not_measured` and preserve ROM-less `___empty` scope wording.

**Exit gate:** Qualification tooling cannot unintentionally write evidence outside its policy root or overstate ROM-less evidence.

## Phase 6 — Verify packaged runtime provenance against executable bytes

- [ ] **SPRR-PROV-001:** Keep build-time `mame_sha256` generation from the staged executable.
- [ ] **SPRR-PROV-002:** Add package qualification that recomputes SHA-256 of the installed Debian bundled MAME and compares it with `runtime-provenance.txt`.
- [ ] **SPRR-PROV-003:** Add equivalent AppImage-extracted runtime digest verification.
- [ ] **SPRR-PROV-004:** Fail package qualification with an actionable integrity error on manifest/executable mismatch.
- [ ] **SPRR-PROV-005:** Preserve efficient normal runtime audit behavior without hashing the full executable on every query.
- [ ] **SPRR-PROV-006:** Correct the audit regression so the changed-binary case actually writes different executable bytes and uses a corresponding different digest.
- [ ] **SPRR-PROV-007:** Preserve regression coverage showing identical digest/runtime across randomized AppImage-like mount paths remains current.
- [ ] **SPRR-PROV-008:** Preserve legacy digest-less audit invalidation.
- [ ] **SPRR-PROV-009:** Add a negative package-integrity regression with intentionally mismatched provenance and binary bytes.

**Exit gate:** The final package proves that the executable bytes shipped to the user match the runtime digest used for audit provenance.

## Phase 7 — Resolve first-frame success timing contract

- [ ] **SPRR-FIRST-001:** Choose and document either the accepted-ack contract or immediate-post-draw acknowledgement contract for first-frame durable success.
- [ ] **SPRR-FIRST-002:** Keep durable history backend-authoritative; frontend draw state alone must not update SQLite.
- [ ] **SPRR-FIRST-003:** If keeping delayed polling ACK, add a regression/documented behavior for process exit between draw and ACK.
- [ ] **SPRR-FIRST-004:** If switching to immediate ACK, bound the command and preserve exact session/sequence validation.
- [ ] **SPRR-FIRST-005:** Ensure no-first-frame reporting cannot overwrite a success already accepted by Rust.

**Exit gate:** “Successful launch” has one precise, test-proven definition at the draw/ack boundary.

## Phase 8 — Comprehensive automated regressions

- [ ] **SPRR-TEST-001:** Run and pass the new same-session input lifecycle regression.
- [ ] **SPRR-TEST-002:** Run and pass real SQLite history lifecycle/retry regressions.
- [ ] **SPRR-TEST-003:** Run and pass strengthened frame acknowledgement regressions.
- [ ] **SPRR-TEST-004:** Run and pass deterministic FIFO cancellation/disconnect regressions.
- [ ] **SPRR-TEST-005:** Run and pass qualification output-path shell regressions.
- [ ] **SPRR-TEST-006:** Run and pass packaged-runtime digest verification regressions.
- [ ] **SPRR-TEST-007:** Re-run all existing frame parsing/BGRX/sizing/metrics/input/content-path/audit/session regressions.
- [ ] **SPRR-TEST-008:** Confirm no test depends on user ROMs, graphical desktop, original installed MAME or personal runtime configuration.

**Exit gate:** Every second-review defect has a regression that would fail on the reviewed baseline or a deliberately constructed equivalent failure fixture.

## Phase 9 — Exact-source CI and full runtime requalification

- [ ] **SPRR-CI-001:** Pass Rust formatting and Clippy/lint on the final implementation SHA.
- [ ] **SPRR-CI-002:** Pass the complete Rust unit/integration suite.
- [ ] **SPRR-CI-003:** Pass frontend formatting/lint/typecheck/Vitest/component tests.
- [ ] **SPRR-CI-004:** Pass the production Tauri project build.
- [ ] **SPRR-CI-005:** Pass ordinary Linux packaging.
- [ ] **SPRR-CI-006:** Pass required Windows packaging.
- [ ] **SPRR-CI-007:** Pass required macOS packaging.
- [ ] **SPRR-CI-008:** Pass security workflow(s).
- [ ] **SPRR-CI-009:** Build the full real-MAME Linux Debian package and AppImage from the final exact implementation head.
- [ ] **SPRR-CI-010:** Pass ROM-less `___empty` headless snapshot and frame-seam qualification.
- [ ] **SPRR-CI-011:** Pass installed Debian package qualification including executable/provenance digest comparison.
- [ ] **SPRR-CI-012:** Pass AppImage extraction/package qualification including executable/provenance digest comparison.
- [ ] **SPRR-CI-013:** Preserve the final package artifact and record artifact ID/name/expiration.
- [ ] **SPRR-CI-014:** Record exact source SHA, CI run IDs, AppImage/deb hashes, sizes and bundled MAME version.
- [ ] **SPRR-CI-015:** Treat all earlier runtime artifacts, including the `08575954…` artifact, as stale for final acceptance after the last behavior change.

**Exit gate:** A new exact-source full-runtime package is green and cryptographically reconciled to its recorded provenance.

## Phase 10 — Documentation and completion reconciliation

- [ ] **SPRR-DOC-001:** Add a second-review note to the previous remediation specification pointing to this active specification/TODO.
- [ ] **SPRR-DOC-002:** Add a second-review note to the previous remediation TODO without erasing its historical checked state.
- [ ] **SPRR-DOC-003:** Document the fixed same-session input lifecycle and history retry semantics in the appropriate gameplay architecture/runtime docs.
- [ ] **SPRR-DOC-004:** Document the chosen first-frame success/ACK contract.
- [ ] **SPRR-DOC-005:** Document qualification output-path policy and package provenance verification.
- [ ] **SPRR-DOC-006:** Update package evidence with the final second-remediation exact-source artifact.
- [ ] **SPRR-DOC-007:** Run and pass documentation CI.
- [ ] **SPRR-DOC-008:** Perform a final source-level review of every SPRR item and verify no actionable non-deferred second-review defect remains.

**Exit gate:** Repository documentation accurately distinguishes original work, first remediation, second remediation, automated qualification and deferred desktop acceptance.

## Phase 11 — Deferred desktop/ROM acceptance — do not execute until user resumes

These fourteen items are carried forward unchanged in meaning. They are not second-remediation implementation failures and must remain unchecked until the user explicitly resumes this scope.

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

**Resumption gate:** Only an explicit user instruction resumes these tasks.

## Final completion gate

The second post-review remediation is complete only when:

- [ ] Every non-deferred **SPRR-*** item above is checked with appropriate code/test/CI evidence.
- [ ] The same-session stale input-release race has a fix and a regression that fails on the reviewed baseline behavior.
- [ ] Durable history decisions survive retryable persistence failure.
- [ ] Presentation ACK semantics, FIFO disconnect semantics and first-frame success timing are explicitly documented and tested.
- [ ] Qualification output containment and executable/provenance hash verification are enforced.
- [ ] The final exact-source full real-MAME Debian/AppImage artifact is green and recorded.
- [ ] Previous remediation documentation points to this second remediation without erasing historical evidence.
- [ ] All 14 **DEFER-ORIG-*** items remain unchecked unless the user explicitly resumes them.
- [ ] The repository can accurately report: **all second post-review autonomous remediation complete; only explicitly deferred desktop/ROM acceptance remains**.
