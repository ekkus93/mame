# MAME Tauri Bundled Runtime Distribution TODO

**Date:** 2026-09-27  
**Status:** Open  
**Canonical specification:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_SPEC_2026-09-27.md`

This is the canonical implementation and completion ledger for making the Debian/Ubuntu MAME Tauri distribution a single complete installation containing both the frontend and a functional MAME runtime.

Treat each checkbox as incomplete until implementation, tests, and required qualification evidence exist. Do not mark an item complete based only on a synthetic runtime fixture when the item requires production-package behavior.

**Progress evidence, 2026-09-28:** backend bundled-default runtime resolution, runtime preference reset semantics, app/runtime identity reporting, normal Settings UX, automatic metadata import/refresh UI behavior, catalog-backed launch consistency, machine audit/bulk audit/library availability/MAME UI/software-list/start-empty/export runtime consistency, diagnostics runtime-source classification, the bundled-runtime user contract document, real-runtime validator/dependency-augmentation plumbing, bundled launch resource/user-state directory handling, BMR baseline inventory, BMR TODO workflow wiring guard, bundled-runtime security review, and synthetic-vs-real runtime staging/build guards are implemented on promoted `master`. Code-bearing head `5b07d0010974b1d13eeb5ad019a8f50263d7e785` passed exact-head CI: Tauri project run `36436956493`, Tauri Linux packaging run `36436956933`, Tauri macOS packaging run `36436956591`, and Tauri Windows packaging run `36436956622`. Baseline/TODO reconciliation head `203f01af0526ff241d080cede0d36578c2c29d38` passed Tauri project run `36438583452` and Build documentation run `36438583398`. Security-review documentation head `6613938d9cffcf370802b40ef1ff934156e1698b` passed Build documentation run `36440652200`. Real production MAME build/staging dispatch and real-runtime `.deb` qualification evidence remain open and are intentionally not claimed complete by synthetic fixture evidence.

---

## BMR-000 — Baseline and invariants

- [x] Record the current bundled-runtime, settings, startup, metadata, launch, and Linux packaging paths relevant to this work.
- [x] Confirm the current failure mode: no `settings.mameExecutable` yields `MameVersionReport::NotConfigured` even when a package-owned runtime is present.
- [x] Preserve the invariant that Rust owns runtime resolution, validation, and process launch.
- [x] Preserve the invariant that ROM/CHD/software content is not bundled as part of this work.
- [x] Add a regression guard that the canonical bundled-runtime TODO remains wired into applicable CI/documentation checks.

**Required evidence:** source inventory plus regression/static test for the core product invariant.

---

## BMR-001 — Make bundled MAME the default runtime source

- [x] Change effective-runtime resolution so a valid package-owned bundled runtime is used whenever no explicit external override is active.
- [x] Stop interpreting “no external executable path configured” as “MAME not configured” in a production package with a valid bundled runtime.
- [x] Reuse `BundledRuntimeLayout` and existing `MameExecutableSource` source/trust types rather than creating a parallel untyped path.
- [x] Resolve bundled resources from Tauri’s package resource directory, independent of current working directory and `PATH`.
- [x] Canonicalize and containment-check the bundled runtime root/executable before use.
- [x] Define a clear backend error/report when a production package is missing or has a corrupt bundled runtime.
- [x] Add Rust tests for bundled-default resolution, missing/corrupt bundled runtime, and package-root containment.

**Acceptance:** fresh installed app + no user settings resolves a `bundled` MAME identity, not `notConfigured`.

---

## BMR-002 — Redefine persisted runtime preference and migrate settings

- [x] Replace “optional MAME executable means runtime existence” semantics with “bundled/default vs explicit external override” semantics.
- [x] Update settings schema/types as required while retaining deterministic migration.
- [x] Migrate legacy `mameExecutable: null` to bundled/default.
- [x] Preserve an existing valid legacy external MAME path as an explicit external override.
- [x] Define behavior for an invalid legacy external path, including a one-action recovery to bundled runtime.
- [x] Ensure clearing/resetting an external override restores bundled/default rather than disabling MAME.
- [x] Add migration/round-trip tests covering old and new schemas.

**Acceptance:** settings persistence cannot represent “normal package install but MAME absent” merely because no override path is stored.

---

## BMR-003 — Make application/runtime identity report the effective source

- [x] Update `get_app_info`/runtime identity resolution to inspect the effective bundled/external source.
- [x] Ensure the frontend receives source kind, trust, path/version/build information needed to explain which runtime is active.
- [x] Preserve fail-closed reporting when the selected effective source cannot be validated.
- [x] Ensure diagnostics distinguish broken bundled package/runtime from broken external override.
- [x] Add tests proving bundled identity on default install and external identity only after explicit override.

**Acceptance:** normal installed startup reports bundled MAME version/source without reading a user-provided path.

---

## BMR-004 — Remove required executable selection from normal Settings UX

- [x] Remove the prominent editable “MAME executable” path/Browse/Save/Clear group from the normal General Settings path.
- [x] Replace it with a read-only runtime-status presentation for bundled MAME, including version/source information.
- [x] Put external executable selection behind an explicit advanced/developer override affordance.
- [x] Rename copy so the external path is consistently described as an override.
- [x] Provide a visible “Use bundled MAME”/equivalent action when an external override is active or invalid.
- [x] Ensure clearing the override immediately returns the application to bundled runtime semantics.
- [x] Add component/source tests proving normal users are not told to locate MAME.

**Acceptance:** the screen shown in the 2026-09-27 report cannot appear as the required first-run/normal configuration experience.

---

## BMR-005 — Automatic first-run and upgrade metadata bootstrap

- [x] On a clean install with valid bundled MAME and no metadata generation, automatically start metadata generation/import.
- [x] On application upgrade where bundled MAME identity changes, automatically detect stale metadata and refresh it.
- [x] Do not require a normal user to click “Import Metadata” after fresh install or routine bundled-runtime upgrade.
- [x] Preserve clear in-shell progress while metadata generation is running.
- [x] Preserve actionable Retry/diagnostic behavior on metadata generation failure.
- [x] Ensure a successful bootstrap transitions directly into the normal machine query path.
- [x] Add tests covering first-run empty metadata, stale metadata after bundled-version change, success, and failure/retry.

**Acceptance:** clean first launch progresses from initialization to machine browser without executable selection or manual metadata import.

---

## BMR-006 — Keep metadata/audit/launch on one effective runtime identity

- [x] Audit all metadata, BIOS, software-list, machine audit, Start, Start Empty, and software launch paths for runtime-source consistency.
- [x] Ensure every operation resolves/reuses the same effective runtime contract.
- [x] Ensure switching bundled ↔ external marks incompatible catalog generations stale before catalog-backed launch.
- [x] Preserve Rust-side revalidation of machine/software/BIOS/launch values.
- [x] Add regression tests proving catalog/runtime mismatch cannot silently launch against the wrong MAME executable.

**Acceptance:** metadata generation and launch cannot accidentally use different runtimes after the bundled-default change.

---

## BMR-007 — Build and stage a real production MAME runtime

- [x] Define the production MAME build input/revision used for the Tauri package.
- [x] Prefer building MAME from the same qualified repository source/revision unless a different pinned/reproducible source is explicitly justified.
- [x] Add or update build automation that produces a real MAME executable for the Linux release package.
- [ ] Stage the real binary with `hash`, `bgfx`, COPYING, and required legal/runtime resources via the package staging path.
- [x] Record bundled MAME version/source provenance in build/release evidence.
- [ ] Ensure the staged executable has correct permissions and is runnable on the target Linux environment.
- [ ] Identify and package/declare any system shared-library dependencies needed by the real MAME binary.
- [x] Keep synthetic fixtures available only for fast structural tests where useful.

**Acceptance:** a production-qualified staging directory contains a real MAME binary whose `-version` command succeeds.

---

## BMR-008 — Produce a genuinely self-contained Debian/Ubuntu `.deb`

- [ ] Ensure the production Linux Tauri bundle config includes the real staged MAME runtime.
- [ ] Produce one `.deb` containing frontend + Rust backend + bundled MAME + required non-system runtime resources.
- [ ] Do not require the distro `mame` package as a functional dependency.
- [ ] Verify Debian package metadata declares the shared libraries required by both Tauri and bundled MAME.
- [ ] Verify desktop integration launches the Tauri application normally.
- [ ] Verify package-owned runtime resources are installed in a stable Tauri resource location.
- [ ] Verify installed runtime resources are read-only/package-owned and user state is written elsewhere.
- [ ] Preserve AppImage bundled-runtime behavior where practical without weakening the `.deb` requirement.

**Acceptance:** a user can install one `.deb` on a clean supported Debian/Ubuntu system and obtains a working frontend plus MAME runtime.

---

## BMR-009 — Replace synthetic-only package qualification with real-runtime release qualification

- [x] Keep the existing synthetic MT-1305-style package test clearly labeled as structural/fixture validation if it remains useful.
- [x] Add a release-grade workflow/job that stages a real MAME runtime.
- [ ] Install the resulting `.deb` into a clean qualification environment.
- [ ] Verify the installed bundled MAME binary executes `-version`.
- [ ] Verify a bounded real metadata/list invocation succeeds.
- [ ] Verify the Tauri backend resolves the installed runtime as `bundled` with no external setting.
- [ ] Verify the application starts without opening/requiring an executable chooser.
- [ ] Verify metadata bootstrap can reach ready/catalog state.
- [ ] Verify uninstall removes package-owned frontend/runtime files.
- [ ] Verify no distro-installed `mame` binary is needed for the test to pass.
- [ ] Cover supported Ubuntu and Debian environments to the practical limit of available CI; explicitly document any remaining manual graphical qualification.

**Acceptance:** release/package qualification would fail if the `.deb` contained only the current synthetic shell-script MAME fixture.

---

## BMR-010 — Runtime working directories and user-writable MAME state

- [x] Identify MAME runtime assumptions for `hash`, `bgfx`, INI/CFG, plugins, and other resources in the packaged layout.
- [x] Ensure package-owned `hash`/`bgfx` assets are discovered without relying on a developer checkout or mutable current working directory.
- [x] Ensure MAME-generated/user configuration is directed to appropriate user-writable directories.
- [x] Ensure package operation does not require writes under `/usr` or the bundled runtime resource directory.
- [ ] Add tests/smoke evidence for launch from an arbitrary working directory.
- [ ] Add package smoke proving a read-only package runtime can still launch and initialize user state.

**Acceptance:** the installed app remains functional when its package resource tree is non-writable.

---

## BMR-011 — Upgrade and override lifecycle

- [ ] Verify upgrading the `.deb` replaces the bundled runtime atomically with the new package version.
- [x] Verify bundled-runtime version changes stale/refresh metadata automatically.
- [ ] Preserve user content paths, favorites, collections, controller profiles, launch preferences, and other user data across upgrades.
- [x] Preserve an intentional external override across upgrades.
- [x] If an external override becomes invalid, present recovery and “Use bundled MAME” rather than leaving the user permanently blocked.
- [ ] Add migration/upgrade qualification covering at least one prior-settings scenario.

**Acceptance:** normal package upgrade requires no executable reconfiguration.

---

## BMR-012 — Security, legal, and provenance closure

- [x] Re-run bundled-runtime containment and executable-validation threat review after default-source changes.
- [x] Verify no generic shell/process path was added to the WebView API.
- [x] Verify external override remains validated and explicit.
- [ ] Verify required MAME COPYING/license/legal resources are included in production packages.
- [x] Record source/build provenance for the bundled runtime in release evidence.
- [x] Explicitly document that ROMs/CHDs/game software/undistributable firmware are not included.
- [x] Ensure diagnostics expose useful runtime provenance without leaking unrelated filesystem data.

**Acceptance:** making bundled MAME the default does not weaken the existing process/filesystem security boundary or redistribution compliance.

---

## BMR-013 — Documentation and user-facing installation contract

- [x] Update README/product documentation to say the Debian/Ubuntu `.deb` includes MAME.
- [ ] Remove documentation that says normal end users need to supply a MAME executable.
- [x] Document the one-package install/launch flow.
- [x] Document that ROM/content paths remain user supplied.
- [x] Document first-run metadata initialization and expected progress/failure behavior.
- [x] Document the external MAME executable as an advanced override only.
- [x] Document how developers build/stage the real bundled runtime.
- [x] Document which CI jobs use synthetic fixtures and which qualify a real release runtime.

**Acceptance:** documentation describes the product users will actually receive, not the obsolete external-runtime-first workflow.

---

## BMR-014 — Regression tripwires

- [x] Add a static/source/component tripwire preventing the normal settings UI from reverting to a required executable-path picker.
- [x] Add a Rust/backend tripwire proving default resolution prefers bundled runtime.
- [x] Add a package tripwire requiring `mame-runtime/bin/mame` and required resources.
- [x] Add a real-runtime qualification assertion that rejects a synthetic/non-MAME executable.
- [x] Add a first-run tripwire proving bundled install does not report `MameVersionReport::NotConfigured`.
- [x] Add an override-reset tripwire proving “Use bundled MAME” restores bundled source.
- [x] Wire these tests into the applicable project/package/security workflows.

**Acceptance:** the original broken UX cannot silently return without CI failing.

---

## BMR-015 — Final end-to-end qualification and closure

- [ ] Reconcile every task/subtask in this TODO as complete, explicitly deferred with rationale, superseded with rationale, or blocked by concrete user-required input.
- [ ] Build the final production candidate from an exact commit SHA.
- [ ] Produce the final Debian/Ubuntu `.deb` containing real bundled MAME.
- [ ] Install it in a clean supported environment.
- [ ] Verify first launch with no settings requires no executable-path interaction.
- [ ] Verify backend/app identity reports bundled runtime.
- [ ] Verify automatic metadata initialization reaches ready state.
- [ ] Verify machine browser becomes available.
- [ ] Verify real bundled MAME launch path works with test/legal content available to CI or an equivalent bounded runtime smoke.
- [ ] Verify external override and “Use bundled MAME” reset behavior.
- [ ] Verify upgrade/uninstall/package-resource behavior.
- [ ] Qualify the exact final PR head through all applicable project, security, Linux packaging, documentation, and other affected workflows.
- [ ] Merge only the exact qualified head through Ralph Bridge.
- [ ] Reload this TODO from promoted `master`.
- [ ] Verify applicable post-merge `master` CI.
- [ ] Record promoted master SHA, PR number, workflow run IDs, package artifact identity, bundled MAME version/provenance, and final install evidence.

**Completion evidence required:** exact implementation/PR/master SHAs, real-runtime `.deb` qualification evidence, CI run IDs, installation/runtime/metadata evidence, and reconciled TODO.

---

## Completion rule

This effort is complete only when a normal Debian/Ubuntu user can install one production `.deb`, launch the application without supplying or locating MAME, automatically use the package-owned real MAME runtime, initialize/refresh metadata, reach the MAME-style machine browser, and retain an optional advanced external-runtime override. Synthetic package fixtures alone are never sufficient for closure.
