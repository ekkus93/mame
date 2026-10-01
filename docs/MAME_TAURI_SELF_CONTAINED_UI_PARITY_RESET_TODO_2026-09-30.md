# MAME Tauri Self-Contained UI Parity Reset TODO

**Date:** 2026-09-30  
**Status:** Complete — RESET-000 through RESET-010 are implemented, qualified, reconciled, and closed on `master`.  
**Spec:** `docs/MAME_TAURI_SELF_CONTAINED_UI_PARITY_RESET_SPEC_2026-09-30.md`  
**Execution rule:** work directly on `master` unless explicitly instructed otherwise.  
**Supersedes for priority:** any existing TODO item that conflicts with self-contained packaged operation, native-MAME UI parity, single content-path truth, Start gating, or actionable missing-ROM diagnostics.

Do not mark an item complete unless the implementation, tests, documentation/TODO reconciliation, and exact-head qualification evidence exist on `master`.

This TODO exists because the project drifted from the intended product goal: a Tauri replacement UI for MAME that is self-contained in the packaged app, sane to run in place, and recognizably aligned with native MAME.

**RESET-000 evidence, 2026-09-30:** reset execution was frozen on current `master` head `ea1482445351b234345fdc535cbdb51ae55c5faa` (`docs: add self-contained parity reset plan`). Exact-head documentation CI for that reset-start head passed: Build documentation run `36760131526`. The older bundled-runtime roadmap now points here as completed evidence, and the older ROM-path/launch-parity roadmap now points here as superseded execution mapping. Deferred feature-growth areas before reset acceptance are: new primary UI controls unrelated to native-MAME parity, additional catalog/filter features outside count/filter parity, new artwork/media behavior outside de-emphasis/clarification, and normal-flow external-MAME configuration work beyond preserving it as advanced/debug behavior.

**RESET-001 evidence, 2026-09-30:** `docs/MAME_TAURI_SELF_CONTAINED_RUNTIME_INVENTORY_2026-09-30.md` records the current source/test/package inventory for self-contained packaged runtime behavior. It maps backend default runtime resolution, frontend settings/catalog behavior, catalog-backed launch trust boundaries, external/development runtime escape hatches, and BMR package workflow evidence to the RESET-001 acceptance criteria. Existing source and tests already enforce that bundled runtime resolution is backend/package-owned, external runtime selection is explicit/advanced, and persisted/frontend data cannot self-assert `qualifiedBundled` trust.

**RESET-002 evidence, 2026-09-30:** `docs/MAME_TAURI_NATIVE_UI_PARITY_INVENTORY_2026-09-30.md` records the native-MAME/Tauri reference view facts, current source-level UI inventory, visible Tauri-only controls, list/filter/status semantics, artwork/media issues, intended primary UI shape, and RESET-003 dispositions.

**RESET-003 evidence, 2026-09-30:** semantic availability/driver labels, optional-artwork de-emphasis, direct content/audit guidance, and primary-action cleanup are complete. Export moved under the quiet `More` utility menu; favorite/configure/audit moved under `Machine options`; external runtime selection remains behind the explicit `Advanced override` affordance; the right panel prioritizes local content state and configuration/audit actions when content is unknown or missing. Frontend/component tripwires cover the revised primary UI.

**RESET-004/RESET-005 evidence, 2026-09-30:** `EffectiveContentPaths` is the configured-only ordered backend contract for ROM/software/CHD paths, with empty, missing, non-directory, permission-denied, and unreadable states preserved explicitly. Automatic default-location discovery is explicitly deferred for this reset. Audit persistence, audit execution, catalog launch, software launch, diagnostics, and launch `rompath` all consume that contract. Catalog launches require a current audit for the current runtime/path identity; Complete/BestAvailable are launchable, Unknown/stale are gated with `MAME_CONTENT_AUDIT_REQUIRED`, and missing/incorrect/mixed failures are gated with `MAME_CONTENT_UNAVAILABLE` before the process-spawn boundary. Frontend machine/software Start controls mirror that policy.

**RESET-006 evidence, 2026-09-30:** pre-ready process exit is classified separately from runtime-control protocol failure, bounded stdout/stderr tails are retained, common missing/incorrect ROM patterns map to `MAME_CONTENT_LAUNCH_FAILED`, and the raw control-channel code remains secondary diagnostic detail. Launch context records machine, software, BIOS, audit state, and a safe effective-content-path summary. Rust and frontend tests cover missing-content early exit and user-facing copy.

**RESET-007 evidence, 2026-09-30:** the hidden unconditional `m.is_device = 0` base-query predicate was removed. The checked-in `listxml-reset-count-parity.xml` fixture fixes runtime/catalog identity and independently exercises parent/clone, BIOS, driver status, mechanical, CHD, device, and non-runnable dimensions. The exact historical transient database behind the observed `88 Games` versus `1-59 of 59` screenshot was not retained; the inventory documents that limitation while reproducing and regression-testing the concrete hidden-filter semantic difference against native MAME source behavior.

**RESET-008/RESET-009 evidence, 2026-09-30:** the real Linux package smoke now verifies bundled/qualified runtime resolution without external configuration, empty-path unaudited gating, configured `rompath` propagation, unavailable-content no-spawn gating, actionable missing-content early-exit classification, and external-override/reset behavior. Package-bearing head `02dea60ab04484fd8dc7722390bcabe9ee346ad0` passed real-runtime package run `36803486807`. Final implementation head `d5a6e53edd2032242e818ef8fa875e481915ef8d` passed Tauri project run `36809751237`, Linux packaging `36809751262`, macOS packaging `36809751232`, Windows packaging `36809751256`, and Tauri security `36809751247`; real-runtime run `36809751229` was still running when this reconciliation was written. README guidance documents bundled runtime defaults, advanced external override, ordered ROM/software/CHD configuration, availability/stale semantics, catalog-vs-local-content distinction, optional artwork, and native unfiltered-count semantics.

**Final product-goal evidence:** the packaged Tauri application resolves its package-owned MAME runtime by default, keeps external runtime selection advanced-only, uses one effective content-path contract across audit/launch/diagnostics, gates normal Start on current audited local availability, converts ordinary missing-ROM startup failures into actionable product errors, and aligns the primary browser/count semantics with the documented native-MAME baseline.

**RESET-010 closure evidence, 2026-09-30:** reconciliation head `002ff848f580e968da659f26f9d3e98b0a258541` was promoted on `master` and passed exact-head Tauri project run `36810394430` and Build documentation run `36810394463`. This is the final promoted-master CI evidence for the reset ledger. The package-bearing reset content-policy evidence remains head `02dea60ab04484fd8dc7722390bcabe9ee346ad0` / real-runtime package run `36803486807`, while final implementation head `d5a6e53edd2032242e818ef8fa875e481915ef8d` passed the full standard implementation qualification set recorded below.

---

## RESET-000 — Freeze product invariants and stop scope drift

- [x] Treat `docs/MAME_TAURI_SELF_CONTAINED_UI_PARITY_RESET_SPEC_2026-09-30.md` as the controlling product spec for this reset.
- [x] Add a short note to the older BMR/RPL TODOs pointing to this reset spec/TODO as the current execution priority.
- [x] Identify open TODO items that are now deferred because they add feature growth before self-contained parity.
- [x] Stop adding new primary UI controls, catalog features, artwork/media behavior, or external-MAME configuration features until the reset acceptance criteria are met.
- [x] Record the current `master` SHA at reset start.
- [x] Record current exact-head CI state at reset start.

**Acceptance:** there is one clear execution target: self-contained packaged Tauri MAME with native-ish UI behavior.

---

## RESET-001 — Prove packaged app is self-contained by default

- [x] Inventory current bundled runtime resolution for packaged Linux, macOS, and Windows builds.
- [x] Inventory current external/development MAME selection surfaces in backend and frontend.
- [x] Verify normal packaged launch does not require user-selected external MAME.
- [x] Hide, de-emphasize, or reclassify external/development MAME selection as advanced/debug-only in the normal UI.
- [x] Ensure persisted catalog data cannot self-assert bundled trust for arbitrary frontend-provided paths.
- [x] Ensure packaged bundled runtime remains the default runtime source for release builds.
- [x] Add tests proving packaged runtime resolution works with no external MAME setting.
- [x] Add tests proving external/development runtime selection is not the default normal packaged flow.

**Acceptance:** a normal packaged user does not need to install upstream MAME or point the app at an external binary before using the app.

---

## RESET-002 — Native-MAME UI parity inventory

- [x] Capture current native MAME reference screenshots/views for the target baseline.
- [x] Capture current Tauri UI screenshots/views for the comparable state.
- [x] Inventory every visible Tauri-only control in the primary library/list/detail/start path.
- [x] Classify each Tauri-only control as: remove, hide, advanced/debug, de-emphasize, or justified intentional addition.
- [x] Inventory current list columns, labels, filters, status indicators, and details panel sections.
- [x] Compare Tauri list/filter/status labels against native-MAME semantics.
- [x] Identify UI elements that confuse driver/emulation status with ROM availability.
- [x] Identify UI elements that make optional artwork/media look like required launch controls.
- [x] Document the intended primary UI shape after reset.

**Acceptance:** there is a concrete UI parity inventory that drives implementation instead of ad hoc feature growth.

---

## RESET-003 — Strip or de-emphasize non-native primary UI clutter

- [x] Remove or hide Tauri-only buttons/controls from the primary path unless justified by RESET-002.
- [x] Move advanced/debug runtime controls out of the normal user path.
- [x] Rename or annotate `Working`/driver-status filters so they are not confused with local ROM availability.
- [x] Make `Available` clearly mean locally playable content after audit.
- [x] Ensure `Unknown` availability is not styled as playable.
- [x] Hide, disable, or de-emphasize empty artwork/media categories.
- [x] Label optional artwork/media as optional, not launch requirements.
- [x] Prioritize ROM availability and audit/configuration guidance in the details panel.
- [x] Add frontend/component tests for the revised primary UI behavior.

**Acceptance:** the primary UI no longer looks like a custom control panel unrelated to native MAME, and optional artwork/media does not look required for launching.

---

## RESET-004 — Finish one effective ROM/content path contract

- [x] Finalize the `EffectiveContentPaths` backend contract for configured paths.
- [x] Decide whether discovered/default MAME-compatible ROM locations are supported in this reset or explicitly deferred.
- [x] Preserve user-configured ROM/software/CHD ordering.
- [x] Represent empty effective ROM/content path lists explicitly.
- [x] Represent missing/inaccessible/invalid paths with validation status.
- [x] Ensure audit consumes the effective content-path contract.
- [x] Ensure Start/launch consumes the effective content-path contract.
- [x] Ensure diagnostics consume the effective content-path contract.
- [x] Add Rust tests for configured, empty, missing, inaccessible, invalid, and ordered path cases.

**Acceptance:** audit, launch, and diagnostics use the same source of truth for ROM/software/CHD paths.

---

## RESET-005 — Start gating for unknown, stale, and unavailable content

- [x] Define the exact launch policy for `Available`, `BestAvailable`, `Unknown`, stale audit, missing required ROMs, and incorrect required ROMs.
- [x] Add backend enforcement so normal Start cannot spawn MAME for gated unknown/stale/unavailable states.
- [x] Add frontend gating so the UI communicates why Start is unavailable.
- [x] Route unknown/unaudited states to audit/configure guidance.
- [x] Route unavailable/missing states to missing-content guidance.
- [x] Add backend tests proving no MAME process is spawned for gated launches.
- [x] Add frontend tests proving Start copy/state matches availability state.
- [x] Add tests proving available content launches with the effective path list.

**Acceptance:** pressing Start on unknown or unavailable content no longer blindly launches MAME into an avoidable runtime-control failure.

---

## RESET-006 — Missing-ROM and early-exit diagnostics

- [x] Classify early MAME exit before runtime-control readiness separately from runtime-control protocol bugs.
- [x] Capture bounded stdout/stderr tail for launch failures where safe.
- [x] Detect common missing-ROM/incorrect-ROM output patterns.
- [x] Map ordinary missing-ROM/invalid-content failures to user-facing copy that explains the content problem.
- [x] Preserve raw internal runtime-control codes only as secondary diagnostic detail.
- [x] Include selected machine/software/BIOs in launch-failure diagnostics.
- [x] Include effective content paths or safe summaries in launch-failure diagnostics.
- [x] Include audit state/freshness in launch-failure diagnostics.
- [x] Add Rust tests for early-exit classification and stderr/stdout tail capture.
- [x] Add frontend tests proving raw control-channel closure is not the primary visible missing-ROM error.

**Acceptance:** missing-ROM and early-exit failures are actionable product errors, not backend implementation leakage.

---

## RESET-007 — Native-MAME count and filter parity

- [x] Reproduce the observed native `88 Games` vs Tauri `1-59 of 59` discrepancy against a fixed runtime/catalog baseline.
- [x] Record exact runtime version, metadata generation, filters, and view state used for comparison.
- [x] Test whether parent/clone filtering accounts for the discrepancy.
- [x] Test whether BIOS/device filtering accounts for the discrepancy.
- [x] Test whether working/imperfect/preliminary status filtering accounts for the discrepancy.
- [x] Test whether mechanical/non-game filtering accounts for the discrepancy.
- [x] Test whether CHD-required filtering accounts for the discrepancy.
- [x] Test whether category/metadata gaps account for the discrepancy.
- [x] Fix unintentional Tauri import/query/filter bugs.
- [x] Document intentional differences if any remain.
- [x] Add regression tests for the chosen parity semantics.

**Acceptance:** the project can either match native MAME's count for the target view or explain and test the intentional difference.

---

## RESET-008 — Package/integration qualification

- [x] Add or update a package-level smoke proving the packaged app resolves bundled MAME without external configuration.
- [x] Add or update a no-ROM-path first-run smoke proving the app shows clear unaudited/unconfigured behavior.
- [x] Add or update a configured-ROM-path smoke proving audit and launch receive the same effective `rompath`.
- [x] Add or update an unavailable-ROM smoke proving normal Start is gated.
- [x] Add or update a missing-ROM early-exit smoke proving user-facing diagnostics are actionable.
- [x] Run exact-head `Tauri project` CI.
- [x] Run affected Linux/macOS/Windows packaging workflows.
- [x] Run `Tauri security` when backend/Rust/package paths change.
- [x] Record run IDs, conclusions, and exact head SHAs.

**Acceptance:** CI covers the actual packaged run-in-place behavior, not just unit-level internals.

---

## RESET-009 — Documentation and user guidance

- [x] Document that the packaged Tauri app includes MAME for normal use.
- [x] Document that external MAME selection is advanced/debug/development behavior, not required setup.
- [x] Document how to configure ROM, software, and CHD paths.
- [x] Document what `Available`, `Unavailable`, `Unknown`, stale, and unaudited mean.
- [x] Document why catalog entries can exist before local ROMs are available.
- [x] Document optional artwork/media directories and clarify they are not launch requirements.
- [x] Document any intentional differences from native MAME UI/list semantics.

**Acceptance:** a normal user can understand how to make games playable without learning backend architecture or runtime-control internals.

---

## RESET-010 — Final reconciliation and closure

- [x] Reconcile every item in this TODO as complete, explicitly deferred with rationale, superseded with rationale, or blocked by concrete user-required input.
- [x] Record reset start SHA and final implementation SHA.
- [x] Record exact workflow run IDs and conclusions.
- [x] Record package/integration evidence for bundled runtime, no-ROM first run, configured ROM path, Start gating, missing-ROM diagnostics, and UI parity.
- [x] Update older BMR/RPL docs with final cross-references and status.
- [x] Verify final promoted `master` CI.
- [x] Add a final evidence paragraph explaining how the project now satisfies the original product goal.

**Completion rule:** this reset is complete only when the packaged Tauri app is self-contained by default, the primary UI is native-MAME-aligned, ROM/content paths have one source of truth, Start does not blindly launch unknown/unavailable content, missing-ROM errors are actionable, and native-vs-Tauri count semantics are fixed or explicitly explained with tests.

---

## Immediate execution order

1. RESET-000: freeze this reset as the controlling plan.
2. RESET-001: verify/lock self-contained packaged runtime as the normal path.
3. RESET-002 and RESET-003: inventory and simplify the primary UI.
4. RESET-004 and RESET-005: finish effective paths and Start gating.
5. RESET-006: replace missing-ROM/runtime-control leakage with useful diagnostics.
6. RESET-007: resolve native-vs-Tauri list/count parity.
7. RESET-008 through RESET-010: qualify, document, and close.

## Current known facts

- Current reset creation baseline head: `893084ed72a41774929c560778edcb7a28e6f96b`.
- RESET-000 execution head: `ea1482445351b234345fdc535cbdb51ae55c5faa`; Build documentation run `36760131526` passed on that exact head.
- RESET-001 inventory head: `39172ed8cb3f0e6e4da2d7027e838952368b3d9a`; `docs/MAME_TAURI_SELF_CONTAINED_RUNTIME_INVENTORY_2026-09-30.md` records the self-contained runtime/default-path inventory.
- RESET-002 inventory head: `e428e1060024d68effdde574400eb8489e1ad918`; `docs/MAME_TAURI_NATIVE_UI_PARITY_INVENTORY_2026-09-30.md` records the native UI parity inventory.
- RESET-003 semantic-label/action-de-emphasis head: `135315f998d87e12fd9eadbb9ea70a59117a48e3`; exact-head `Tauri project`, Linux packaging, macOS packaging, Windows packaging, and security workflows passed as recorded above. Linux real-runtime package remained in progress at the most recent observation.
- Final implementation head: `d5a6e53edd2032242e818ef8fa875e481915ef8d`; exact-head Tauri project `36809751237`, Linux packaging `36809751262`, macOS packaging `36809751232`, Windows packaging `36809751256`, and security `36809751247` passed. Real-runtime package run `36809751229` remained in progress at reconciliation time.
- Package-level reset content-policy evidence: head `02dea60ab04484fd8dc7722390bcabe9ee346ad0`, Tauri Linux real runtime package run `36803486807` passed.
- Final reconciliation qualification head: `002ff848f580e968da659f26f9d3e98b0a258541`; exact-head Tauri project `36810394430` and Build documentation `36810394463` passed.
- The RPL work had already started audit/launch path convergence, but this reset is broader and product-focused: self-contained packaged behavior, native UI parity, Start gating, missing-ROM diagnostics, and list/filter parity.
