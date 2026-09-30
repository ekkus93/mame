# MAME Tauri Self-Contained UI Parity Reset TODO

**Date:** 2026-09-30  
**Status:** In progress — RESET-000 through RESET-002 are complete; RESET-003 has landed the first semantic UI cleanup slices for availability/driver-status labels, selected-machine secondary-action de-emphasis, and optional-artwork de-emphasis.  
**Spec:** `docs/MAME_TAURI_SELF_CONTAINED_UI_PARITY_RESET_SPEC_2026-09-30.md`  
**Execution rule:** work directly on `master` unless explicitly instructed otherwise.  
**Supersedes for priority:** any existing TODO item that conflicts with self-contained packaged operation, native-MAME UI parity, single content-path truth, Start gating, or actionable missing-ROM diagnostics.

Do not mark an item complete unless the implementation, tests, documentation/TODO reconciliation, and exact-head qualification evidence exist on `master`.

This TODO exists because the project drifted from the intended product goal: a Tauri replacement UI for MAME that is self-contained in the packaged app, sane to run in place, and recognizably aligned with native MAME.

**RESET-000 evidence, 2026-09-30:** reset execution was frozen on current `master` head `ea1482445351b234345fdc535cbdb51ae55c5faa` (`docs: add self-contained parity reset plan`). Exact-head documentation CI for that reset-start head passed: Build documentation run `36760131526`. The older bundled-runtime roadmap now points here as completed evidence, and the older ROM-path/launch-parity roadmap now points here as superseded execution mapping. Deferred feature-growth areas before reset acceptance are: new primary UI controls unrelated to native-MAME parity, additional catalog/filter features outside count/filter parity, new artwork/media behavior outside de-emphasis/clarification, and normal-flow external-MAME configuration work beyond preserving it as advanced/debug behavior.

**RESET-001 evidence, 2026-09-30:** `docs/MAME_TAURI_SELF_CONTAINED_RUNTIME_INVENTORY_2026-09-30.md` records the current source/test/package inventory for self-contained packaged runtime behavior. It maps backend default runtime resolution, frontend settings/catalog behavior, catalog-backed launch trust boundaries, external/development runtime escape hatches, and BMR package workflow evidence to the RESET-001 acceptance criteria. Existing source and tests already enforce that bundled runtime resolution is backend/package-owned, external runtime selection is explicit/advanced, and persisted/frontend data cannot self-assert `qualifiedBundled` trust.

**RESET-002 evidence, 2026-09-30:** `docs/MAME_TAURI_NATIVE_UI_PARITY_INVENTORY_2026-09-30.md` records the native-MAME/Tauri reference view facts, current source-level UI inventory, visible Tauri-only controls, list/filter/status semantics, artwork/media issues, intended primary UI shape, and RESET-003 dispositions.

**RESET-003 partial evidence, 2026-09-30:** head `e757d757368a0acd301cdf59d7bde2b7553780ec` renamed local availability and driver-status labels so `Available locally`, `Missing content`, `Not audited`, `Driver Working`, `Driver Not Working`, and `Driver status` no longer confuse ROM availability with emulation-driver state; exact-head `Tauri project` run `36764890153` passed its core quality jobs, and Linux/macOS/Windows packaging plus security also passed for that head. Head `135315f998d87e12fd9eadbb9ea70a59117a48e3` de-emphasized secondary selected-machine actions while keeping `Start` primary and passed exact-head `Tauri project` run `36769119025`, Linux packaging run `36769118944`, macOS packaging run `36769118948`, Windows packaging run `36769119026`, and security run `36769118974`; the long real-runtime package workflow for that head was still running at the most recent observation. The optional-artwork/media slice is pending exact-head CI for this TODO update.

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

- [ ] Remove or hide Tauri-only buttons/controls from the primary path unless justified by RESET-002.
- [ ] Move advanced/debug runtime controls out of the normal user path.
- [x] Rename or annotate `Working`/driver-status filters so they are not confused with local ROM availability.
- [x] Make `Available` clearly mean locally playable content after audit.
- [x] Ensure `Unknown` availability is not styled as playable.
- [x] Hide, disable, or de-emphasize empty artwork/media categories.
- [x] Label optional artwork/media as optional, not launch requirements.
- [ ] Prioritize ROM availability and audit/configuration guidance in the details panel.
- [x] Add frontend/component tests for the revised primary UI behavior.

**Acceptance:** the primary UI no longer looks like a custom control panel unrelated to native MAME, and optional artwork/media does not look required for launching.

---

## RESET-004 — Finish one effective ROM/content path contract

- [ ] Finalize the `EffectiveContentPaths` backend contract for configured paths.
- [ ] Decide whether discovered/default MAME-compatible ROM locations are supported in this reset or explicitly deferred.
- [ ] Preserve user-configured ROM/software/CHD ordering.
- [ ] Represent empty effective ROM/content path lists explicitly.
- [ ] Represent missing/inaccessible/invalid paths with validation status.
- [ ] Ensure audit consumes the effective content-path contract.
- [ ] Ensure Start/launch consumes the effective content-path contract.
- [ ] Ensure diagnostics consume the effective content-path contract.
- [ ] Add Rust tests for configured, empty, missing, inaccessible, invalid, and ordered path cases.

**Acceptance:** audit, launch, and diagnostics use the same source of truth for ROM/software/CHD paths.

---

## RESET-005 — Start gating for unknown, stale, and unavailable content

- [ ] Define the exact launch policy for `Available`, `BestAvailable`, `Unknown`, stale audit, missing required ROMs, and incorrect required ROMs.
- [ ] Add backend enforcement so normal Start cannot spawn MAME for gated unknown/stale/unavailable states.
- [ ] Add frontend gating so the UI communicates why Start is unavailable.
- [ ] Route unknown/unaudited states to audit/configure guidance.
- [ ] Route unavailable/missing states to missing-content guidance.
- [ ] Add backend tests proving no MAME process is spawned for gated launches.
- [ ] Add frontend tests proving Start copy/state matches availability state.
- [ ] Add tests proving available content launches with the effective path list.

**Acceptance:** pressing Start on unknown or unavailable content no longer blindly launches MAME into an avoidable runtime-control failure.

---

## RESET-006 — Missing-ROM and early-exit diagnostics

- [ ] Classify early MAME exit before runtime-control readiness separately from runtime-control protocol bugs.
- [ ] Capture bounded stdout/stderr tail for launch failures where safe.
- [ ] Detect common missing-ROM/incorrect-ROM output patterns.
- [ ] Map ordinary missing-ROM/invalid-content failures to user-facing copy that explains the content problem.
- [ ] Preserve raw internal runtime-control codes only as secondary diagnostic detail.
- [ ] Include selected machine/software/BIOs in launch-failure diagnostics.
- [ ] Include effective content paths or safe summaries in launch-failure diagnostics.
- [ ] Include audit state/freshness in launch-failure diagnostics.
- [ ] Add Rust tests for early-exit classification and stderr/stdout tail capture.
- [ ] Add frontend tests proving raw control-channel closure is not the primary visible missing-ROM error.

**Acceptance:** missing-ROM and early-exit failures are actionable product errors, not backend implementation leakage.

---

## RESET-007 — Native-MAME count and filter parity

- [ ] Reproduce the observed native `88 Games` vs Tauri `1-59 of 59` discrepancy against a fixed runtime/catalog baseline.
- [ ] Record exact runtime version, metadata generation, filters, and view state used for comparison.
- [ ] Test whether parent/clone filtering accounts for the discrepancy.
- [ ] Test whether BIOS/device filtering accounts for the discrepancy.
- [ ] Test whether working/imperfect/preliminary status filtering accounts for the discrepancy.
- [ ] Test whether mechanical/non-game filtering accounts for the discrepancy.
- [ ] Test whether CHD-required filtering accounts for the discrepancy.
- [ ] Test whether category/metadata gaps account for the discrepancy.
- [ ] Fix unintentional Tauri import/query/filter bugs.
- [ ] Document intentional differences if any remain.
- [ ] Add regression tests for the chosen parity semantics.

**Acceptance:** the project can either match native MAME's count for the target view or explain and test the intentional difference.

---

## RESET-008 — Package/integration qualification

- [ ] Add or update a package-level smoke proving the packaged app resolves bundled MAME without external configuration.
- [ ] Add or update a no-ROM-path first-run smoke proving the app shows clear unaudited/unconfigured behavior.
- [ ] Add or update a configured-ROM-path smoke proving audit and launch receive the same effective `rompath`.
- [ ] Add or update an unavailable-ROM smoke proving normal Start is gated.
- [ ] Add or update a missing-ROM early-exit smoke proving user-facing diagnostics are actionable.
- [ ] Run exact-head `Tauri project` CI.
- [ ] Run affected Linux/macOS/Windows packaging workflows.
- [ ] Run `Tauri security` when backend/Rust/package paths change.
- [ ] Record run IDs, conclusions, and exact head SHAs.

**Acceptance:** CI covers the actual packaged run-in-place behavior, not just unit-level internals.

---

## RESET-009 — Documentation and user guidance

- [ ] Document that the packaged Tauri app includes MAME for normal use.
- [ ] Document that external MAME selection is advanced/debug/development behavior, not required setup.
- [ ] Document how to configure ROM, software, and CHD paths.
- [ ] Document what `Available`, `Unavailable`, `Unknown`, stale, and unaudited mean.
- [ ] Document why catalog entries can exist before local ROMs are available.
- [ ] Document optional artwork/media directories and clarify they are not launch requirements.
- [ ] Document any intentional differences from native MAME UI/list semantics.

**Acceptance:** a normal user can understand how to make games playable without learning backend architecture or runtime-control internals.

---

## RESET-010 — Final reconciliation and closure

- [ ] Reconcile every item in this TODO as complete, explicitly deferred with rationale, superseded with rationale, or blocked by concrete user-required input.
- [ ] Record reset start SHA and final implementation SHA.
- [ ] Record exact workflow run IDs and conclusions.
- [ ] Record package/integration evidence for bundled runtime, no-ROM first run, configured ROM path, Start gating, missing-ROM diagnostics, and UI parity.
- [ ] Update older BMR/RPL docs with final cross-references and status.
- [ ] Verify final promoted `master` CI.
- [ ] Add a final evidence paragraph explaining how the project now satisfies the original product goal.

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
- The RPL work had already started audit/launch path convergence, but this reset is broader and product-focused: self-contained packaged behavior, native UI parity, Start gating, missing-ROM diagnostics, and list/filter parity.
