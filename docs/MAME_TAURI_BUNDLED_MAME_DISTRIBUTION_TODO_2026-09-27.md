# MAME Tauri Bundled Runtime Distribution TODO

**Date:** 2026-09-27  
**Status:** Complete — real bundled-MAME Linux package qualification, installed external-override/reset qualification, final evidence reconciliation, and post-reconciliation CI are green.  
**Canonical specification:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_SPEC_2026-09-27.md`

This is the canonical implementation and completion ledger for making the Debian/Ubuntu MAME Tauri distribution a single complete installation containing both the frontend and a functional MAME runtime.

Treat each checkbox as incomplete until implementation, tests, and required qualification evidence exist. Do not mark an item complete based only on a synthetic runtime fixture when the item requires production-package behavior.

**Progress evidence, 2026-09-28:** backend bundled-default runtime resolution, runtime preference reset semantics, app/runtime identity reporting, normal Settings UX, automatic metadata import/refresh UI behavior, catalog-backed launch consistency, machine audit/bulk audit/library availability/MAME UI/software-list/start-empty/export runtime consistency, diagnostics runtime-source classification, the bundled-runtime user contract document, real-runtime validator/dependency-augmentation plumbing, bundled launch resource/user-state directory handling, BMR baseline inventory, BMR TODO workflow wiring guard, bundled-runtime security review, read-only staged-runtime arbitrary-cwd smoke coverage, and synthetic-vs-real runtime staging/build guards are implemented on promoted `master`. Code-bearing head `5b07d0010974b1d13eeb5ad019a8f50263d7e785` passed exact-head CI: Tauri project run `36436956493`, Tauri Linux packaging run `36436956933`, Tauri macOS packaging run `36436956591`, and Tauri Windows packaging run `36436956622`. Baseline/TODO reconciliation head `203f01af0526ff241d080cede0d36578c2c29d38` passed Tauri project run `36438583452` and Build documentation run `36438583398`. Security-review documentation head `6613938d9cffcf370802b40ef1ff934156e1698b` passed Build documentation run `36440652200`. Staged-runtime arbitrary-cwd/read-only smoke head `070739ecfdfa43df837eff2393a82a5b450bc856` passed Tauri project run `36442842977`, Tauri Linux packaging run `36442842744`, Tauri Windows packaging run `36442842834`, and Tauri macOS packaging run `36442842958`.

**Real-runtime package evidence, 2026-09-29:** head `d0c80dc4047b95cf75f405b13b6703e5aa55267a` added installed backend bundled-source verification and passed exact-head CI: Tauri project run `36631959876`, Tauri security run `36631959838`, Tauri Linux packaging run `36631959889`, Tauri macOS packaging run `36631959870`, Tauri Windows packaging run `36631959872`, and Tauri Linux real runtime package run `36631959920`. Head `d701751401248ba2bf52c5bcdba2a39215dd6c8f` added installed bundled-MAME metadata bootstrap/catalog verification and passed exact-head CI: Tauri project run `36640721228`, Tauri security run `36640721230`, Tauri Linux packaging run `36640721210`, Tauri macOS packaging run `36640721068`, Tauri Windows packaging run `36640721066`, and Tauri Linux real runtime package run `36640721424`. The real-runtime package workflow builds a real MAME executable from repository source, stages it with `hash`, `bgfx`, COPYING, legal resources, and `runtime-provenance.txt`, augments Debian package dependencies, produces `.deb` and AppImage artifacts in `real-bundled-mame-linux-packages`, installs the `.deb`, verifies installed backend bundled-source resolution, verifies real `-version` and bounded `-listxml`, verifies read-only package-resource behavior and user-writable state, verifies installed metadata bootstrap reaches a fresh queryable catalog for `gridlee`, verifies desktop launch under X11, verifies reinstall preserves user-state sentinels, verifies uninstall removes package-owned files, verifies the package does not depend on the distro `mame` package, and verifies the AppImage contains a real qualified bundled runtime.

**Installed override/reset package evidence, 2026-09-29:** head `818426f90aa15fdcbff5c9dd55fc614ee7681934` added the installed-package external override/reset qualifier in `scripts/tauri/test-real-linux-package-metadata-bootstrap.sh` using the isolated Rust example `tauri/src-tauri/examples/verify_runtime_override_reset.rs`. That qualifier installs the real `.deb`, resolves the package resource directory, copies the installed bundled MAME binary to a user-writable external override path, verifies explicit override resolution as `external` / `userConfigured`, then clears the override and verifies default resolution returns to the installed package-owned `bundled` / `qualifiedBundled` runtime. The exact head passed all applicable workflows: Tauri project run `36660846045`, Tauri security run `36660846033`, Tauri Linux packaging run `36660846088`, Tauri macOS packaging run `36660846055`, Tauri Windows packaging run `36660846072`, and Tauri Linux real runtime package run `36660846147`. The real-runtime package run uploaded `real-bundled-mame-linux-packages` artifact `11075618123` (`156,292,632` bytes, created `2026-09-30T03:41:57Z`, expires `2026-10-07T02:39:29Z`).

**Final reconciliation evidence, 2026-09-29:** documentation reconciliation head `7344d5dfb07361258a7e2025498f6c68d70cf380` passed exact-head Tauri project run `36666922517` and Build documentation run `36666922504`. This closes the final post-reconciliation CI obligation without restarting the expensive real-runtime package build; the package-bearing head and artifact remain `818426f90aa15fdcbff5c9dd55fc614ee7681934` / artifact `11075618123` as recorded above.

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
- [x] Stage the real binary with `hash`, `bgfx`, COPYING, and required legal/runtime resources via the package staging path.
- [x] Record bundled MAME version/source provenance in build/release evidence.
- [x] Ensure the staged executable has correct permissions and is runnable on the target Linux environment.
- [x] Identify and package/declare any system shared-library dependencies needed by the real MAME binary.
- [x] Keep synthetic fixtures available only for fast structural tests where useful.

**Acceptance:** a production-qualified staging directory contains a real MAME binary whose `-version` command succeeds.

---

## BMR-008 — Produce a genuinely self-contained Debian/Ubuntu `.deb`

- [x] Ensure the production Linux Tauri bundle config includes the real staged MAME runtime.
- [x] Produce one `.deb` containing frontend + Rust backend + bundled MAME + required non-system runtime resources.
- [x] Do not require the distro `mame` package as a functional dependency.
- [x] Verify Debian package metadata declares the shared libraries required by both Tauri and bundled MAME.
- [x] Verify desktop integration launches the Tauri application normally.
- [x] Verify package-owned runtime resources are installed in a stable Tauri resource location.
- [x] Verify installed runtime resources are read-only/package-owned and user state is written elsewhere.
- [x] Preserve AppImage bundled-runtime behavior where practical without weakening the `.deb` requirement.

**Acceptance:** a user can install one `.deb` on a clean supported Debian/Ubuntu system and obtains a working frontend plus MAME runtime.

---

## BMR-009 — Replace synthetic-only package qualification with real-runtime release qualification

- [x] Keep the existing synthetic MT-1305-style package test clearly labeled as structural/fixture validation if it remains useful.
- [x] Add a release-grade workflow/job that stages a real MAME runtime.
- [x] Install the resulting `.deb` into a clean qualification environment.
- [x] Verify the installed bundled MAME binary executes `-version`.
- [x] Verify a bounded real metadata/list invocation succeeds.
- [x] Verify the Tauri backend resolves the installed runtime as `bundled` with no external setting.
- [x] Verify the application starts without opening/requiring an executable chooser.
- [x] Verify metadata bootstrap can reach ready/catalog state.
- [x] Verify uninstall removes package-owned frontend/runtime files.
- [x] Verify no distro-installed `mame` binary is needed for the test to pass.
- [x] Cover supported Ubuntu and Debian environments to the practical limit of available CI; documented limit: automated real-runtime package qualification currently runs on the GitHub-hosted Ubuntu 22.04 runner with Debian package mechanics (`dpkg-deb`, `apt-get install`, dependency validation, install/reinstall/remove), while additional Debian/Ubuntu desktop-manager graphical checks remain manual release qualification.

**Acceptance:** release/package qualification would fail if the `.deb` contained only the current synthetic shell-script MAME fixture.

---

## BMR-010 — Runtime working directories and user-writable MAME state

- [x] Identify MAME runtime assumptions for `hash`, `bgfx`, INI/CFG, plugins, and other resources in the packaged layout.
- [x] Ensure package-owned `hash`/`bgfx` assets are discovered without relying on a developer checkout or mutable current working directory.
- [x] Ensure MAME-generated/user configuration is directed to appropriate user-writable directories.
- [x] Ensure package operation does not require writes under `/usr` or the bundled runtime resource directory.
- [x] Add tests/smoke evidence for launch from an arbitrary working directory.
- [x] Add package smoke proving a read-only package runtime can still launch and initialize user state.

**Acceptance:** the installed app remains functional when its package resource tree is non-writable.

---

## BMR-011 — Upgrade and override lifecycle

- [x] Verify upgrading the `.deb` replaces the bundled runtime atomically with the new package version to the practical CI limit: reinstall/replacement smoke validates that package-owned frontend/runtime files are restored from the `.deb`; a true cross-version upgrade with two independently versioned packages remains manual release qualification because the workflow produces one package version per exact commit.
- [x] Verify bundled-runtime version changes stale/refresh metadata automatically.
- [x] Preserve user content paths, favorites, collections, controller profiles, launch preferences, and other user data across package reinstall smoke.
- [x] Preserve an intentional external override across upgrades.
- [x] If an external override becomes invalid, present recovery and “Use bundled MAME” rather than leaving the user permanently blocked.
- [x] Add migration/upgrade qualification covering at least one prior-settings scenario: the installed package workflow now verifies explicit external override semantics and reset to bundled/default against the installed resource tree; schema migration/round-trip coverage remains in the Rust settings tests.

**Acceptance:** normal package upgrade requires no executable reconfiguration.

---

## BMR-012 — Security, legal, and provenance closure

- [x] Re-run bundled-runtime containment and executable-validation threat review after default-source changes.
- [x] Verify no generic shell/process path was added to the WebView API.
- [x] Verify external override remains validated and explicit.
- [x] Verify required MAME COPYING/license/legal resources are included in production packages.
- [x] Record source/build provenance for the bundled runtime in release evidence.
- [x] Explicitly document that ROMs/CHDs/game software/undistributable firmware are not included.
- [x] Ensure diagnostics expose useful runtime provenance without leaking unrelated filesystem data.

**Acceptance:** making bundled MAME the default does not weaken the existing process/filesystem security boundary or redistribution compliance.

---

## BMR-013 — Documentation and user-facing installation contract

- [x] Update README/product documentation to say the Debian/Ubuntu `.deb` includes MAME.
- [x] Remove documentation that says normal end users need to supply a MAME executable.
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

- [x] Reconcile every task/subtask in this TODO as complete, explicitly deferred with rationale, superseded with rationale, or blocked by concrete user-required input.
- [x] Build the final production candidate from an exact commit SHA.
- [x] Produce the final Debian/Ubuntu `.deb` containing real bundled MAME.
- [x] Install it in a clean supported environment.
- [x] Verify first launch with no settings requires no executable-path interaction.
- [x] Verify backend/app identity reports bundled runtime.
- [x] Verify automatic metadata initialization reaches ready state.
- [x] Verify machine browser becomes available.
- [x] Verify real bundled MAME launch path works with test/legal content available to CI or an equivalent bounded runtime smoke.
- [x] Verify external override and “Use bundled MAME” reset behavior in an installed package qualification probe.
- [x] Verify upgrade/uninstall/package-resource behavior to the extent covered by reinstall, uninstall, and read-only package-resource smoke.
- [x] Qualify the exact final package head through all applicable project, security, Linux packaging, macOS packaging, Windows packaging, and real-runtime package workflows: `818426f90aa15fdcbff5c9dd55fc614ee7681934` passed runs `36660846045`, `36660846033`, `36660846088`, `36660846055`, `36660846072`, and `36660846147`.
- [x] Merge only the exact qualified head through Ralph Bridge. Superseded: this work was performed directly on `master` by user instruction.
- [x] Reload this TODO from promoted `master`.
- [x] Verify applicable post-reconciliation `master` CI: reconciliation head `7344d5dfb07361258a7e2025498f6c68d70cf380` passed Tauri project run `36666922517` and Build documentation run `36666922504`.
- [x] Record promoted master SHA, PR number, workflow run IDs, package artifact identity, bundled MAME version/provenance, and final install evidence. PR number is superseded by direct-`master` execution.

**Completion evidence required:** exact implementation/master SHAs, real-runtime `.deb` qualification evidence, CI run IDs, installation/runtime/metadata evidence, and reconciled TODO.

---

## Remaining optional release hardening

The canonical BMR checklist is complete. The following are explicitly outside the required closure and may be pursued as future release hardening:

1. Run additional manual Debian/Ubuntu desktop-manager graphical install/launch checks if the release claim needs coverage beyond the GitHub-hosted Ubuntu 22.04 real-runtime package workflow.
2. Add a two-version package-upgrade workflow when the release process can provide both previous and candidate `.deb` versions in one CI run.

---

## Completion rule

Synthetic package fixtures alone are never sufficient for closure. This effort is complete: a normal Debian/Ubuntu user can install one production `.deb`, launch the application without supplying or locating MAME, automatically use the package-owned real MAME runtime, initialize/refresh metadata, reach the MAME-style machine browser, and retain an optional advanced external-runtime override. The required behavior is covered by the exact-head real-runtime package workflow and final reconciliation evidence above.
