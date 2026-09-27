# MAME Tauri Bundled Runtime Distribution TODO

**Date:** 2026-09-27  
**Status:** Open  
**Canonical specification:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_SPEC_2026-09-27.md`

This is the canonical implementation and completion ledger for making the Debian/Ubuntu MAME Tauri distribution a single complete installation containing both the frontend and a functional MAME runtime.

Treat each checkbox as incomplete until implementation, tests, and required qualification evidence exist. Do not mark an item complete based only on a synthetic runtime fixture when the item requires production-package behavior.

---

## BMR-000 — Baseline and invariants

- [ ] Record the current bundled-runtime, settings, startup, metadata, launch, and Linux packaging paths relevant to this work.
- [ ] Confirm the current failure mode: no `settings.mameExecutable` yields `MameVersionReport::NotConfigured` even when a package-owned runtime is present.
- [ ] Preserve the invariant that Rust owns runtime resolution, validation, and process launch.
- [ ] Preserve the invariant that ROM/CHD/software content is not bundled as part of this work.
- [ ] Add a regression guard that the canonical bundled-runtime TODO remains wired into applicable CI/documentation checks.

**Required evidence:** source inventory plus regression/static test for the core product invariant.

---

## BMR-001 — Make bundled MAME the default runtime source

- [ ] Change effective-runtime resolution so a valid package-owned bundled runtime is used whenever no explicit external override is active.
- [ ] Stop interpreting “no external executable path configured” as “MAME not configured” in a production package with a valid bundled runtime.
- [ ] Reuse `BundledRuntimeLayout` and existing `MameExecutableSource` source/trust types rather than creating a parallel untyped path.
- [ ] Resolve bundled resources from Tauri’s package resource directory, independent of current working directory and `PATH`.
- [ ] Canonicalize and containment-check the bundled runtime root/executable before use.
- [ ] Define a clear backend error/report when a production package is missing or has a corrupt bundled runtime.
- [ ] Add Rust tests for bundled-default resolution, missing/corrupt bundled runtime, and package-root containment.

**Acceptance:** fresh installed app + no user settings resolves a `bundled` MAME identity, not `notConfigured`.

---

## BMR-002 — Redefine persisted runtime preference and migrate settings

- [ ] Replace “optional MAME executable means runtime existence” semantics with “bundled/default vs explicit external override” semantics.
- [ ] Update settings schema/types as required while retaining deterministic migration.
- [ ] Migrate legacy `mameExecutable: null` to bundled/default.
- [ ] Preserve an existing valid legacy external MAME path as an explicit external override.
- [ ] Define behavior for an invalid legacy external path, including a one-action recovery to bundled runtime.
- [ ] Ensure clearing/resetting an external override restores bundled/default rather than disabling MAME.
- [ ] Add migration/round-trip tests covering old and new schemas.

**Acceptance:** settings persistence cannot represent “normal package install but MAME absent” merely because no override path is stored.

---

## BMR-003 — Make application/runtime identity report the effective source

- [ ] Update `get_app_info`/runtime identity resolution to inspect the effective bundled/external source.
- [ ] Ensure the frontend receives source kind, trust, path/version/build information needed to explain which runtime is active.
- [ ] Preserve fail-closed reporting when the selected effective source cannot be validated.
- [ ] Ensure diagnostics distinguish broken bundled package/runtime from broken external override.
- [ ] Add tests proving bundled identity on default install and external identity only after explicit override.

**Acceptance:** normal installed startup reports bundled MAME version/source without reading a user-provided path.

---

## BMR-004 — Remove required executable selection from normal Settings UX

- [ ] Remove the prominent editable “MAME executable” path/Browse/Save/Clear group from the normal General Settings path.
- [ ] Replace it with a read-only runtime-status presentation for bundled MAME, including version/source information.
- [ ] Put external executable selection behind an explicit advanced/developer override affordance.
- [ ] Rename copy so the external path is consistently described as an override.
- [ ] Provide a visible “Use bundled MAME”/equivalent action when an external override is active or invalid.
- [ ] Ensure clearing the override immediately returns the application to bundled runtime semantics.
- [ ] Add component/source tests proving normal users are not told to locate MAME.

**Acceptance:** the screen shown in the 2026-09-27 report cannot appear as the required first-run/normal configuration experience.

---

## BMR-005 — Automatic first-run and upgrade metadata bootstrap

- [ ] On a clean install with valid bundled MAME and no metadata generation, automatically start metadata generation/import.
- [ ] On application upgrade where bundled MAME identity changes, automatically detect stale metadata and refresh it.
- [ ] Do not require a normal user to click “Import Metadata” after fresh install or routine bundled-runtime upgrade.
- [ ] Preserve clear in-shell progress while metadata generation is running.
- [ ] Preserve actionable Retry/diagnostic behavior on metadata generation failure.
- [ ] Ensure a successful bootstrap transitions directly into the normal machine query path.
- [ ] Add tests covering first-run empty metadata, stale metadata after bundled-version change, success, and failure/retry.

**Acceptance:** clean first launch progresses from initialization to machine browser without executable selection or manual metadata import.

---

## BMR-006 — Keep metadata/audit/launch on one effective runtime identity

- [ ] Audit all metadata, BIOS, software-list, machine audit, Start, Start Empty, and software launch paths for runtime-source consistency.
- [ ] Ensure every operation resolves/reuses the same effective runtime contract.
- [ ] Ensure switching bundled ↔ external marks incompatible catalog generations stale before catalog-backed launch.
- [ ] Preserve Rust-side revalidation of machine/software/BIOS/launch values.
- [ ] Add regression tests proving catalog/runtime mismatch cannot silently launch against the wrong MAME executable.

**Acceptance:** metadata generation and launch cannot accidentally use different runtimes after the bundled-default change.

---

## BMR-007 — Build and stage a real production MAME runtime

- [ ] Define the production MAME build input/revision used for the Tauri package.
- [ ] Prefer building MAME from the same qualified repository source/revision unless a different pinned/reproducible source is explicitly justified.
- [ ] Add or update build automation that produces a real MAME executable for the Linux release package.
- [ ] Stage the real binary with `hash`, `bgfx`, COPYING, and required legal/runtime resources via the package staging path.
- [ ] Record bundled MAME version/source provenance in build/release evidence.
- [ ] Ensure the staged executable has correct permissions and is runnable on the target Linux environment.
- [ ] Identify and package/declare any system shared-library dependencies needed by the real MAME binary.
- [ ] Keep synthetic fixtures available only for fast structural tests where useful.

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

- [ ] Keep the existing synthetic MT-1305-style package test clearly labeled as structural/fixture validation if it remains useful.
- [ ] Add a release-grade workflow/job that stages a real MAME runtime.
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

- [ ] Identify MAME runtime assumptions for `hash`, `bgfx`, INI/CFG, plugins, and other resources in the packaged layout.
- [ ] Ensure package-owned `hash`/`bgfx` assets are discovered without relying on a developer checkout or mutable current working directory.
- [ ] Ensure MAME-generated/user configuration is directed to appropriate user-writable directories.
- [ ] Ensure package operation does not require writes under `/usr` or the bundled runtime resource directory.
- [ ] Add tests/smoke evidence for launch from an arbitrary working directory.
- [ ] Add package smoke proving a read-only package runtime can still launch and initialize user state.

**Acceptance:** the installed app remains functional when its package resource tree is non-writable.

---

## BMR-011 — Upgrade and override lifecycle

- [ ] Verify upgrading the `.deb` replaces the bundled runtime atomically with the new package version.
- [ ] Verify bundled-runtime version changes stale/refresh metadata automatically.
- [ ] Preserve user content paths, favorites, collections, controller profiles, launch preferences, and other user data across upgrades.
- [ ] Preserve an intentional external override across upgrades.
- [ ] If an external override becomes invalid, present recovery and “Use bundled MAME” rather than leaving the user permanently blocked.
- [ ] Add migration/upgrade qualification covering at least one prior-settings scenario.

**Acceptance:** normal package upgrade requires no executable reconfiguration.

---

## BMR-012 — Security, legal, and provenance closure

- [ ] Re-run bundled-runtime containment and executable-validation threat review after default-source changes.
- [ ] Verify no generic shell/process path was added to the WebView API.
- [ ] Verify external override remains validated and explicit.
- [ ] Verify required MAME COPYING/license/legal resources are included in production packages.
- [ ] Record source/build provenance for the bundled runtime in release evidence.
- [ ] Explicitly document that ROMs/CHDs/game software/undistributable firmware are not included.
- [ ] Ensure diagnostics expose useful runtime provenance without leaking unrelated filesystem data.

**Acceptance:** making bundled MAME the default does not weaken the existing process/filesystem security boundary or redistribution compliance.

---

## BMR-013 — Documentation and user-facing installation contract

- [ ] Update README/product documentation to say the Debian/Ubuntu `.deb` includes MAME.
- [ ] Remove documentation that says normal end users need to supply a MAME executable.
- [ ] Document the one-package install/launch flow.
- [ ] Document that ROM/content paths remain user supplied.
- [ ] Document first-run metadata initialization and expected progress/failure behavior.
- [ ] Document the external MAME executable as an advanced override only.
- [ ] Document how developers build/stage the real bundled runtime.
- [ ] Document which CI jobs use synthetic fixtures and which qualify a real release runtime.

**Acceptance:** documentation describes the product users will actually receive, not the obsolete external-runtime-first workflow.

---

## BMR-014 — Regression tripwires

- [ ] Add a static/source/component tripwire preventing the normal settings UI from reverting to a required executable-path picker.
- [ ] Add a Rust/backend tripwire proving default resolution prefers bundled runtime.
- [ ] Add a package tripwire requiring `mame-runtime/bin/mame` and required resources.
- [ ] Add a real-runtime qualification assertion that rejects a synthetic/non-MAME executable.
- [ ] Add a first-run tripwire proving bundled install does not report `MameVersionReport::NotConfigured`.
- [ ] Add an override-reset tripwire proving “Use bundled MAME” restores bundled source.
- [ ] Wire these tests into the applicable project/package/security workflows.

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
