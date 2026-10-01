# MAME Tauri ROM Path and Launch Parity TODO

**Date:** 2026-09-30  
**Status:** Superseded and implemented through the self-contained UI parity reset; use this roadmap as historical evidence/mapping only.  
**Specification:** `docs/MAME_TAURI_ROM_PATH_AND_LAUNCH_PARITY_SPEC_2026-09-30.md`  
**Depends on completed roadmap:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`

**Reset closure cross-reference, 2026-09-30:** The RPL work mapped into RESET-004 through RESET-009 is implemented on final implementation head `d5a6e53edd2032242e818ef8fa875e481915ef8d`. Effective paths, Start gating, missing-ROM diagnostics, native-count/filter parity, package smokes, and user documentation are reconciled in `docs/MAME_TAURI_SELF_CONTAINED_UI_PARITY_RESET_TODO_2026-09-30.md`. Legacy unchecked RPL boxes remain as historical pre-reset state and are not a competing completion ledger.

This is the canonical checklist for the post-BMR ROM path, launch diagnostics, native-MAME parity, and artwork/details-panel UX work.

Do not mark a checkbox complete unless the implementation, tests, and required qualification evidence exist on `master`. The bundled-runtime package roadmap is already complete; this TODO must not reopen packaging closure except by cross-reference.

**Baseline evidence, 2026-09-30:** roadmap creation head `2e524aef61f17905fb406b3d96ea519d3662a0bb` added this TODO and `docs/MAME_TAURI_ROM_PATH_AND_LAUNCH_PARITY_SPEC_2026-09-30.md`. Baseline inventory is recorded in `docs/MAME_TAURI_ROM_PATH_AND_LAUNCH_PARITY_BASELINE_2026-09-30.md`. Source inspection confirmed that audit composes `-rompath` from configured `romPaths`/`softwarePaths`/`chdPaths`, while bundled Start launch appends package `hashpath`/`bgfx_path` and user-state directories but no `rompath`; the existing launch argv validator rejects semicolon path lists, so a safe fix requires an explicit effective content-path/path-list contract rather than stuffing a MAME path list into a single ordinary path field.

---

## RPL-000 — Baseline and reproduction evidence

- [x] Record the installed package/build used for reproduction, including exact commit SHA and package artifact identity when applicable.
- [x] Capture the current native MAME vs Tauri MAME visual differences: native `88 Games` count, Tauri `1-59 of 59` count, Tauri `Unknown` availability column, and the runtime-control launch error.
- [x] Record the current code paths for settings content paths, audit inputs, launch argv construction, bundled resource project paths, and runtime-control startup failure handling.
- [x] Confirm whether current bundled launch omits an explicit/default `rompath` when no Tauri content paths are configured.
- [x] Add a regression fixture or source-level inventory note tying this TODO to the observed failure class.

**Required evidence:** current-source inventory plus a reproducible description of the installed-package behavior.

---

## RPL-001 — Effective ROM path model

- [ ] Define an `EffectiveContentPaths` or equivalent backend contract that reports configured paths, discovered/default paths, validation status, and source/reason for each path.
- [ ] Investigate bundled MAME's effective/default `rompath` behavior in the installed package environment.
- [ ] Decide and document which MAME-compatible default ROM locations the Tauri frontend should honor automatically.
- [ ] Preserve explicit user-configured ROM paths and their ordering.
- [ ] Ensure an empty effective ROM path list is represented explicitly and not confused with `Unknown` catalog availability.
- [ ] Add Rust tests for configured paths, empty paths, missing paths, inaccessible paths, and discovered/default path ordering.

**Acceptance:** the app can report the exact ROM path set it will use before audit or launch.

---

## RPL-002 — Audit and launch path parity

- [x] Audit the current ROM availability implementation and identify which paths it uses.
- [x] Audit the current Start/launch implementation and identify which `rompath` or project-path arguments it passes to MAME.
- [ ] Refactor audit and launch to share the same effective ROM path contract.
- [ ] Add or update launch argv construction so bundled MAME receives the intended ROM path set explicitly when required.
- [ ] Add tests proving audit and launch use the same effective path list.
- [ ] Add diagnostics showing effective ROM paths for each launch attempt.

**Acceptance:** a ROM path added for audit is also used for Start, and a launch cannot silently use a different ROM search path.

---

## RPL-003 — Availability-aware Start behavior

- [ ] Define launch behavior for `Available`, `Unknown`, `Unavailable`, and stale-audit states.
- [ ] Prevent normal Start from blindly launching machines with `Unknown` availability unless an explicit, tested override is intentionally provided.
- [ ] Prevent normal Start from blindly launching machines with `Unavailable` availability.
- [ ] Show the user a path to audit/configure ROM directories when availability is unknown or unavailable.
- [ ] Add frontend/component tests covering Start gating and user-facing copy.
- [ ] Add backend command tests or integration tests proving gated launches do not spawn MAME for known unavailable content.

**Acceptance:** pressing Start on a row with unknown/missing ROMs leads to audit/configuration guidance instead of a runtime-control startup failure.

---

## RPL-004 — Startup failure diagnostics

- [ ] Classify control-channel closed-before-ready failures separately from ordinary runtime-control protocol bugs.
- [ ] Preserve the raw internal code for diagnostics while mapping user-facing copy to `MAME exited before startup completed` or a more specific diagnosis.
- [ ] Include MAME stderr/stdout tail in the user-visible launch failure details when available.
- [ ] Include selected machine, software, BIOS, effective ROM paths, and audit state in launch-failure details.
- [ ] Detect common missing-ROM output patterns and classify them as missing or invalid ROM content when safe.
- [ ] Add Rust tests for early MAME exit before control readiness with stderr tail capture.
- [ ] Add frontend tests proving the raw internal control-channel phrase is not the primary visible error for ordinary startup failure.

**Acceptance:** missing-ROM or early-exit launches show actionable diagnostics rather than `The MAME runtime-control channel closed before it became ready.` as the main message.

---

## RPL-005 — Native-MAME list/filter parity investigation

- [ ] Build a reproducible metadata fixture or installed-runtime query for comparing native MAME UI counts to Tauri catalog counts.
- [ ] Compare native `88 Games` view against the Tauri `59` result for the observed environment.
- [ ] Identify whether parent/clone filtering accounts for the count difference.
- [ ] Identify whether BIOS/non-BIOS filtering accounts for the count difference.
- [ ] Identify whether Working/Imperfect/Preliminary status mapping accounts for the count difference.
- [ ] Identify whether mechanical, CHD-required, save-supported, or category data gaps account for the count difference.
- [ ] Fix unintentional parity bugs.
- [ ] Document intentional differences in the UI and docs.
- [ ] Add regression tests for the chosen parity semantics.

**Acceptance:** a developer can explain and test why native MAME reports `88 Games` while Tauri reports its count, or the Tauri count is corrected to match the intended native view.

---

## RPL-006 — Catalog vs playable-library UX

- [ ] Rename or annotate filters so `Working` is clearly an emulation-driver status, not ROM availability.
- [ ] Make `Available` the clear playable filter once ROM audit data exists.
- [ ] Add first-run/unaudited guidance explaining catalog rows vs local ROM availability.
- [ ] Ensure `Unknown` availability is not styled as playable.
- [ ] Add an obvious way to run Audit from unaudited views.
- [ ] Add frontend tests for first-run/unaudited copy and filter semantics.

**Acceptance:** a normal user can tell whether they are browsing the full MAME catalog or games they can actually run locally.

---

## RPL-007 — Artwork/details panel UX

- [ ] Inventory the current `Snapshots`, `Cabinet`, `Control Panel`, `PCB`, `Flyer`, and `Title Screen` tabs/buttons and the data source expected for each.
- [ ] Label these categories as optional artwork/media, not launch requirements.
- [ ] Hide, disable, or de-emphasize empty artwork categories when no image exists.
- [ ] Ensure details panel layout prioritizes ROM availability, effective paths, and audit state in unaudited/missing-ROM states.
- [ ] Add frontend tests for empty artwork handling.
- [ ] Update docs or help text explaining optional artwork directories if applicable.

**Acceptance:** optional artwork tabs no longer look like required game controls or proof that a game is runnable.

---

## RPL-008 — Diagnostics bundle and support evidence

- [ ] Add effective content paths and validation status to diagnostics/support output.
- [ ] Add launch argv, selected machine/software/BIOS, audit state, and MAME stderr/stdout tail to launch failure diagnostics, subject to existing path/data redaction policy.
- [ ] Ensure diagnostics distinguish bundled-runtime identity from ROM/content availability.
- [ ] Add tests that diagnostics contain enough information to debug missing ROMs without exposing unrelated filesystem data.

**Acceptance:** a user can provide a diagnostics bundle that explains a failed launch without needing to understand runtime-control internals.

---

## RPL-009 — Documentation

- [ ] Document bundled MAME vs user-supplied ROM/content separation.
- [ ] Document default/discovered ROM path behavior.
- [ ] Document how to configure ROM, software, and CHD paths.
- [ ] Document what `Available`, `Unavailable`, and `Unknown` mean.
- [ ] Document what the optional artwork/media tabs mean.
- [ ] Document known intentional differences from native MAME UI, if any remain.

**Acceptance:** README/product docs explain why catalog entries can appear before ROMs are available and how to make games playable.

---

## RPL-010 — Package/integration qualification

- [ ] Add a package-level or integration smoke that installs/runs the frontend with no ROM paths configured and verifies clear unaudited/unconfigured behavior.
- [ ] Add a smoke or fixture proving a configured ROM path is passed consistently to audit and launch.
- [ ] Add an early-exit fake or bounded real-runtime test proving user-facing launch diagnostics include stderr and effective ROM paths.
- [ ] Run exact-head Tauri project CI for the implementation head.
- [ ] Run affected packaging/security/documentation workflows when their paths are changed.
- [ ] Reconcile this TODO only after the exact implementation head is green.

**Acceptance:** CI would fail if the UI regressed to blind Start on unknown ROMs or exposed raw control-channel closure as the main missing-ROM error.

---

## RPL-011 — Final closure

- [ ] Reconcile every task/subtask in this TODO as complete, explicitly deferred with rationale, superseded with rationale, or blocked by concrete user-required input.
- [ ] Record exact implementation/master SHAs.
- [ ] Record exact workflow run IDs and conclusions.
- [ ] Record package/integration evidence for ROM path, audit, launch, diagnostics, and artwork/details UX behavior.
- [ ] Verify final promoted `master` CI.
- [ ] Add a short final evidence paragraph linking this roadmap back to the completed BMR package baseline.

**Completion rule:** This roadmap is complete only when Tauri MAME can explain and use ROM paths coherently, prevent or clearly diagnose launches without available ROM content, reconcile native-MAME vs Tauri list/filter semantics, and make optional artwork/details UI unambiguous.

---

## Current suspected bugs to verify first

1. Verified: bundled-runtime launch currently does not pass configured ROM/software/CHD paths as an explicit `rompath`, while audit does use configured paths.
2. Start is available for rows with `Unknown` ROM availability.
3. Early MAME exit before runtime-control readiness exposes an internal control-channel message as the primary user-facing error.
4. Native MAME reports `88 Games` while Tauri reports `59` in the observed comparable view.
5. Optional artwork tabs are visually prominent even when no artwork is present and no ROM availability has been established.
