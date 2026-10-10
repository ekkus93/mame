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

- [x] **SPRR-BASE-001:** Record the exact `master` SHA at implementation start and verify both second-remediation documents exist.
- [x] **SPRR-BASE-002:** Re-read the first remediation TODO and confirm its 14 deferred items remain deferred in the new checklist.
- [x] **SPRR-BASE-003:** Inspect current exact-head CI and separate gameplay-remediation failures from unrelated release/tag workflow behavior.
- [x] **SPRR-BASE-004:** Map every second-remediation implementation commit to one or more SPRR IDs.
- [x] **SPRR-BASE-005:** Treat prior runtime artifact source `0857595401d131080688baec2f3225b154787f2d` as historical evidence only once this remediation changes runtime behavior.

**Exit gate:** The second remediation is anchored to a known exact head without changing the user's deferred scope.

> **2026-10-09 implementation baseline:** Started from exact `master` `0176498e59d0250c11a10bfbf961ebaeba588144`. The first-remediation TODO still contained exactly 14 unchecked deferred host tasks and current exact-head documentation CI was green. Runtime behavior changes in this remediation make the prior `08575954…` package historical rather than final qualification evidence.

## Phase 1 — Eliminate same-session stale input releases

- [x] **SPRR-IN-001:** Refactor the gameplay input pump so destructive release/teardown ownership follows the actual MAME session, not controller-preference changes.
- [x] **SPRR-IN-002:** Ensure changing `preferredGamepadId` during a live session cannot enqueue stale zero releases that override the successor input generation.
- [x] **SPRR-IN-003:** Ensure stale asynchronous `setMameInputs` completions cannot mutate current accepted-input bookkeeping.
- [x] **SPRR-IN-004:** Ensure stale generations cannot send semantically stale releases into a currently owned session.
- [x] **SPRR-IN-005:** Preserve correct releases on blur, fullscreen ownership loss, stop, terminal session transition, return-to-library, component teardown and real session replacement.
- [x] **SPRR-IN-006:** Preserve keyboard/gamepad coalescing, W3C-standard controller filtering and the 32-update message bound.
- [x] **SPRR-IN-007:** Ensure a genuinely new session begins with neutral desired/accepted input state.
- [x] **SPRR-IN-008:** Add a regression that reproduces the reviewed same-session race and fails against baseline `461b05174df7b80351b74f589eb40e11b5647390`.
- [x] **SPRR-IN-009:** Add asynchronous lifecycle coverage for actual session replacement, stale request completion and mixed keyboard/gamepad releases.

**Exit gate:** No old frontend generation can release or reassert controls in a live session now owned by a successor generation.

> **SPRR-IN implementation:** `GameSurface` now owns destructive input lifecycle by `session.sessionId` only and reads controller preference/session state/stopping through refs. `createSessionInputPump` owns accepted state, serializes same-session successor sends behind previous cleanup barriers, suppresses stale completion mutation, and releases pending/accepted controls in 32-update batches. Async and source-level regressions cover the original preference-change race and same-session replacement ordering.

## Phase 2 — Make play-history finalization retry-safe

- [x] **SPRR-HIST-001:** Replace remove-before-write settlement with explicit backend-owned pending/decided/persisted state or an equivalent retry-safe model.
- [x] **SPRR-HIST-002:** Ensure first legitimate success/failure decides the outcome exactly once.
- [x] **SPRR-HIST-003:** Prevent a later competing lifecycle event from changing an already decided outcome.
- [x] **SPRR-HIST-004:** Retain a decided outcome after SQLite finalization failure so the same outcome can be retried safely.
- [x] **SPRR-HIST-005:** Keep emulator/session lifecycle independent from history-persistence failure and record actionable diagnostics.
- [x] **SPRR-HIST-006:** Define bounded app-shutdown behavior for any decided-but-not-yet-persisted history outcome.
- [x] **SPRR-HIST-007:** Add real SQLite regression coverage for runtime-ready pending state, first-presentation success, pre-frame exit/crash/stop failure and no-first-frame failure.
- [x] **SPRR-HIST-008:** Add race/retry regressions proving one decided outcome, independent retry sessions, persistence-failure retry and cross-session isolation.

**Exit gate:** A transient durable-history write failure cannot silently strand or discard a session's decided outcome.

> **SPRR-HIST implementation:** History state now separates the winning decision from persistence-in-flight state. A failed SQLite finalization keeps the original decision retryable; competing callbacks retry but cannot change it. App shutdown makes one bounded retry pass. Real SQLite regression coverage deletes/restores the actual history row to prove persistence failure retains and later commits the original outcome.

## Phase 3 — Tighten presentation acknowledgement semantics

- [x] **SPRR-ACK-001:** Explicitly represent the exact delivered frame currently eligible for acknowledgement.
- [x] **SPRR-ACK-002:** Define whether an old duplicate ACK after a newer frame is delivered is rejected or accepted solely as protocol-noise no-op.
- [x] **SPRR-ACK-003:** Ensure an old duplicate can never satisfy, clear or count as acknowledgement of a newer outstanding frame.
- [x] **SPRR-ACK-004:** Preserve one-and-only-one presented metric increment per valid frame.
- [x] **SPRR-ACK-005:** Ensure stale/duplicate ACK handling cannot incorrectly trigger first-frame durable history success.
- [x] **SPRR-ACK-006:** Add regressions for duplicate current ACK, duplicate old ACK after newer delivery, future/backward ACKs, mailbox-dropped ACKs and duration bounds.

**Exit gate:** Presentation accounting has one documented, test-proven interpretation for every duplicate/stale acknowledgement case.

> **SPRR-ACK implementation:** The mailbox now explicitly tracks one outstanding presentation sequence. A duplicate ACK is idempotent only after the current delivered frame was already acknowledged and no newer frame is outstanding; an older duplicate after a newer delivery is rejected. Existing dropped/future/duration checks remain fail-closed, so stale ACK rejection occurs before durable first-frame success settlement.

## Phase 4 — Diagnose producer disconnects and strengthen frame-reader lifecycle

- [x] **SPRR-LIFE-001:** Track whether the FIFO has ever observed producer bytes or an established frame stream.
- [x] **SPRR-LIFE-002:** Continue tolerating pre-connection FIFO EOF while respecting cancellation.
- [x] **SPRR-LIFE-003:** Convert EOF after an established producer/stream into a bounded actionable frame-stream diagnostic.
- [x] **SPRR-LIFE-004:** Convert partial-header or partial-payload producer disconnect into an actionable transport/protocol failure.
- [x] **SPRR-LIFE-005:** Ensure intentional cancellation wins over disconnect reporting during stop/shutdown.
- [x] **SPRR-LIFE-006:** Preserve bounded polling/wakeup with no busy spin or unbounded join.
- [x] **SPRR-LIFE-007:** Add deterministic cancellation coverage with no writer, idle connected writer, partial header and partial payload.
- [x] **SPRR-LIFE-008:** Add producer-disconnect-after-valid-frame regression coverage.
- [x] **SPRR-LIFE-009:** Add supervisor-level coverage proving finalized/drop/shutdown sessions leave no logically active frame reader.
- [x] **SPRR-LIFE-010:** Confirm cancellation cannot publish/authenticate a fabricated partial frame.

**Exit gate:** The frame transport distinguishes “producer not connected yet,” “producer disappeared unexpectedly,” and “session intentionally cancelled.”

> **SPRR-LIFE implementation:** Linux FIFO transport tracks reader-start and producer-byte observation. Pre-producer EOF remains a bounded wait; EOF after observed producer bytes returns stream EOF so header/identity/payload `read_exact` reports an actionable error. Cancellation is checked first and suppresses intentional-shutdown errors. Tests cover no writer, idle writer, partial header/payload, disconnect after a valid frame, partial-frame cancellation, normal stop and supervisor drop.

## Phase 5 — Contain qualification evidence output paths

- [x] **SPRR-QUAL-001:** Define the canonical evidence root for frame-seam reports.
- [x] **SPRR-QUAL-002:** Resolve relative explicit report names beneath that evidence root.
- [x] **SPRR-QUAL-003:** Reject relative traversal/symlink/path escape before directory creation.
- [x] **SPRR-QUAL-004:** Explicitly define whether absolute report paths are allowed; if allowed, treat them as deliberate caller-selected destinations.
- [x] **SPRR-QUAL-005:** Preserve existing safe MAME short-name validation.
- [x] **SPRR-QUAL-006:** Add shell tests for simple relative output, nested relative output, traversal attempts, absolute-output policy and invalid machine identifiers.
- [x] **SPRR-QUAL-007:** Keep zero diagnostic `screen.pixels()` samples as `not_measured` and preserve ROM-less `___empty` scope wording.

**Exit gate:** Qualification tooling cannot unintentionally write evidence outside its policy root or overstate ROM-less evidence.

> **SPRR-QUAL implementation:** Relative output reports resolve under `MAME_TAURI_FRAME_SEAM_EVIDENCE_ROOT` (default `artifacts/in-app-gameplay`) and are containment-checked before directory creation, including existing symlink components. Explicit absolute paths remain deliberate caller-selected destinations. Shell regressions cover simple/nested relative paths, traversal, symlink escape, absolute output and invalid machine names; existing `not_measured`/`___empty` semantics are preserved.

## Phase 6 — Verify packaged runtime provenance against executable bytes

- [x] **SPRR-PROV-001:** Keep build-time `mame_sha256` generation from the staged executable.
- [x] **SPRR-PROV-002:** Add package qualification that recomputes SHA-256 of the installed Debian bundled MAME and compares it with `runtime-provenance.txt`.
- [x] **SPRR-PROV-003:** Add equivalent AppImage-extracted runtime digest verification.
- [x] **SPRR-PROV-004:** Fail package qualification with an actionable integrity error on manifest/executable mismatch.
- [x] **SPRR-PROV-005:** Preserve efficient normal runtime audit behavior without hashing the full executable on every query.
- [x] **SPRR-PROV-006:** Correct the audit regression so the changed-binary case actually writes different executable bytes and uses a corresponding different digest.
- [x] **SPRR-PROV-007:** Preserve regression coverage showing identical digest/runtime across randomized AppImage-like mount paths remains current.
- [x] **SPRR-PROV-008:** Preserve legacy digest-less audit invalidation.
- [x] **SPRR-PROV-009:** Add a negative package-integrity regression with intentionally mismatched provenance and binary bytes.

**Exit gate:** The final package proves that the executable bytes shipped to the user match the runtime digest used for audit provenance.

> **SPRR-PROV implementation:** Added `verify-runtime-provenance.sh`, called by real Linux package qualification for both installed Debian and extracted AppImage runtime trees. It recomputes `bin/mame` SHA-256 and requires exactly one matching `mame_sha256` manifest entry. A standalone negative integrity regression mutates executable bytes and requires an actionable mismatch failure. The audit-store changed-binary fixture now also writes different executable bytes, while normal runtime audit lookups continue consuming package provenance without per-query rehashing.

## Phase 7 — Resolve first-frame success timing contract

- [x] **SPRR-FIRST-001:** Choose and document either the accepted-ack contract or immediate-post-draw acknowledgement contract for first-frame durable success.
- [x] **SPRR-FIRST-002:** Keep durable history backend-authoritative; frontend draw state alone must not update SQLite.
- [x] **SPRR-FIRST-003:** If keeping delayed polling ACK, add a regression/documented behavior for process exit between draw and ACK.
- [x] **SPRR-FIRST-004:** If switching to immediate ACK, bound the command and preserve exact session/sequence validation.
- [x] **SPRR-FIRST-005:** Ensure no-first-frame reporting cannot overwrite a success already accepted by Rust.

**Exit gate:** “Successful launch” has one precise, test-proven definition at the draw/ack boundary.

> **SPRR-FIRST decision:** Keep the accepted-ACK contract. Browser draw alone is not durable success; only Rust accepting the queued presentation sequence settles success. A terminal event between draw and accepted ACK therefore decides failure. Backend history regressions cover draw-without-ACK failure and prove an accepted success decision cannot be overwritten. `SPRR-FIRST-004` is satisfied as not applicable because no immediate-ACK command was introduced.

> **2026-10-10 package-qualification diagnosis:** Exact-head workflow [38008178789](https://github.com/ekkus93/mame/actions/runs/38008178789) on `b950ab5eb03874b0a2e9f7102409548d4d9c7e4b` passed real-MAME build, headless capture, frame seam, Debian/AppImage construction, installed Debian runtime checks, Debian reinstall and X11 smoke. The package job failed when the **AppImage-extracted** MAME executable hash (`e852607f…`) differed from the staged provenance (`dfaf9258…`). This is consistent with linuxdeploy modifying ELF metadata while packaging. The remediation requires verification of unchanged GNU build ID and machine code before recording the final AppImage executable digest and rebuilding the SquashFS payload. Final qualification still fails closed on a mismatched manifest/executable; this build-time reconciliation is not marked complete until exact-head CI passes.

## Phase 8 — Comprehensive automated regressions

- [x] **SPRR-TEST-001:** Run and pass the new same-session input lifecycle regression.
- [x] **SPRR-TEST-002:** Run and pass real SQLite history lifecycle/retry regressions.
- [x] **SPRR-TEST-003:** Run and pass strengthened frame acknowledgement regressions.
- [x] **SPRR-TEST-004:** Run and pass deterministic FIFO cancellation/disconnect regressions.
- [x] **SPRR-TEST-005:** Run and pass qualification output-path shell regressions.
- [x] **SPRR-TEST-006:** Run and pass packaged-runtime digest verification regressions.
- [x] **SPRR-TEST-007:** Re-run all existing frame parsing/BGRX/sizing/metrics/input/content-path/audit/session regressions.
- [x] **SPRR-TEST-008:** Confirm no test depends on user ROMs, graphical desktop, original installed MAME or personal runtime configuration.

**Exit gate:** Every second-review defect has a regression that would fail on the reviewed baseline or a deliberately constructed equivalent failure fixture.

> **2026-10-10 qualification repair:** Exact-head real-runtime run `38006770425` on `8383bf44ad1b758ede8e41f6091fbb351cc32fea` failed in the frame-seam step because the workflow passed `artifacts/in-app-gameplay/ci-frame-seam-<SHA>.txt` as a relative output, while the hardened script already resolves relative outputs under `artifacts/in-app-gameplay/`. The snapshot itself succeeded (`___empty`, `snapshot_status=ok`), but the workflow tried to read the wrong path. The workflow now passes a basename to the script and reads the evidence-root path. Re-run exact-source full-runtime qualification before checking CI gates; this repair is not itself qualification evidence. The separate `v0.1.1` release-starter failure is an existing-tag mismatch and must not be “fixed” by moving that tag.

## Phase 9 — Exact-source CI and full runtime requalification

- [x] **SPRR-CI-001:** Pass Rust formatting and Clippy/lint on the final implementation SHA.
- [x] **SPRR-CI-002:** Pass the complete Rust unit/integration suite.
- [x] **SPRR-CI-003:** Pass frontend formatting/lint/typecheck/Vitest/component tests.
- [x] **SPRR-CI-004:** Pass the production Tauri project build.
- [x] **SPRR-CI-005:** Pass ordinary Linux packaging.
- [x] **SPRR-CI-006:** Pass required Windows packaging.
- [x] **SPRR-CI-007:** Pass required macOS packaging.
- [ ] **SPRR-CI-008:** Pass security workflow(s).
- [x] **SPRR-CI-009:** Build the full real-MAME Linux Debian package and AppImage from the final exact implementation head.
- [x] **SPRR-CI-010:** Pass ROM-less `___empty` headless snapshot and frame-seam qualification.
- [x] **SPRR-CI-011:** Pass installed Debian package qualification including executable/provenance digest comparison.
- [x] **SPRR-CI-012:** Pass AppImage extraction/package qualification including executable/provenance digest comparison.
- [x] **SPRR-CI-013:** Preserve the final package artifact and record artifact ID/name/expiration.
- [x] **SPRR-CI-014:** Record exact source SHA, CI run IDs, AppImage/deb hashes, sizes and bundled MAME version.
- [x] **SPRR-CI-015:** Treat all earlier runtime artifacts, including the `08575954…` artifact, as stale for final acceptance after the last behavior change.

**Exit gate:** A new exact-source full-runtime package is green and cryptographically reconciled to its recorded provenance.

> **2026-10-10 exact-head automated qualification:** Final qualified source `add08e6ae7baae90c5e9e77e8ce26fce588a9a83` (the SquashFS repack fix) passed [Tauri project run 38014688032](https://github.com/ekkus93/mame/actions/runs/38014688032): frontend formatting, lint, typecheck, 51 Vitest files/195 tests, production build; Rust formatting, Clippy, 315 library tests (one ignored), four integration tests and release smoke. [Linux packaging 38014688006](https://github.com/ekkus93/mame/actions/runs/38014688006), [Windows packaging 38014688107](https://github.com/ekkus93/mame/actions/runs/38014688107) and [macOS packaging 38014688066](https://github.com/ekkus93/mame/actions/runs/38014688066) passed on the same SHA. [Real-runtime package run 38014688027](https://github.com/ekkus93/mame/actions/runs/38014688027) passed shell syntax and path/provenance negative tests, reused a SHA-validated real MAME binary, captured ROM-less `___empty` 320×240 headless frames (`snapshot_status=ok`), reconciled linuxdeploy relocation, validated Debian and extracted AppImage executable digests, installed/reinstalled Debian, and passed runtime, dependency and X11 smoke checks. Package evidence: bundled MAME `0.289 (unknown)`; AppImage SHA-256 `347a5dd15ada23aa8cec45ad963303b6aed88002adfb5e3db6d1fccb6029ece4`, 232,995,320 bytes; Debian SHA-256 `8e59c49030ba39ade6c86303636678eb3d2ba1d327765ba09cdd799558f4611b`, 124,954,508 bytes. Provenance executable SHA-256 is `dfaf9258dba976e868b2e40ba072a6cf5f8e901e1e8ee40453bb0f3bdfe705a6` in the Debian staging tree and `e852607f0a40dff3a15fdea6a82ba76f5ac5a7683a00b384edbd97b3445ed1c0` after the verified AppImage relocation. Preserved package artifact ID `11656011923`, name `real-bundled-mame-linux-packages-add08e6ae7baae90c5e9e77e8ce26fce588a9a83`, expires 2026-10-13 01:50 UTC (GitHub Actions retention); executable artifact ID `11654509258`, same expiration. All 14 user-deferred desktop/private-ROM acceptance tasks remain unchecked. The separate one-time `v0.1.1` release-starter [run 38015085562](https://github.com/ekkus93/mame/actions/runs/38015085562) fails only because the existing tag targets another SHA; the tag is deliberately not moved. Security and documentation CI and final source-level review remain separately unverified at this source head.

## Phase 10 — Documentation and completion reconciliation

- [x] **SPRR-DOC-001:** Add a second-review note to the previous remediation specification pointing to this active specification/TODO.
- [x] **SPRR-DOC-002:** Add a second-review note to the previous remediation TODO without erasing its historical checked state.
- [x] **SPRR-DOC-003:** Document the fixed same-session input lifecycle and history retry semantics in the appropriate gameplay architecture/runtime docs.
- [x] **SPRR-DOC-004:** Document the chosen first-frame success/ACK contract.
- [x] **SPRR-DOC-005:** Document qualification output-path policy and package provenance verification.
- [x] **SPRR-DOC-006:** Update package evidence with the final second-remediation exact-source artifact.
- [ ] **SPRR-DOC-007:** Run and pass documentation CI.
- [ ] **SPRR-DOC-008:** Perform a final source-level review of every SPRR item and verify no actionable non-deferred second-review defect remains.

**Exit gate:** Repository documentation accurately distinguishes original work, first remediation, second remediation, automated qualification and deferred desktop acceptance.

> **Documentation reconciliation in progress:** The parent gameplay specification now records session-owned input, retry-safe history, strict outstanding ACKs, stateful producer EOF, the accepted-ACK first-frame contract, contained evidence paths and package provenance verification. The first-remediation spec/TODO point forward without changing historical checkmarks. Final artifact evidence, documentation CI and the final source review remain open until exact-source qualification completes.

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
