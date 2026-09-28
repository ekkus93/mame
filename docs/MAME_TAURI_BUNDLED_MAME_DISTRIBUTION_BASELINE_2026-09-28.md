# MAME Tauri Bundled Runtime Distribution Baseline

**Date:** 2026-09-28  
**Specification:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_SPEC_2026-09-27.md`  
**Checklist:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`  
**Baseline SHA:** `5b07d0010974b1d13eeb5ad019a8f50263d7e785`

This document records the source inventory and baseline failure mode for the bundled-runtime distribution work. It is intentionally focused on the paths that determine whether a normal Debian/Ubuntu package install can use package-owned MAME without an end-user executable picker.

## Source inventory

### Bundled runtime layout and staging

- `tauri/src-tauri/src/bundled_runtime.rs` defines the package-owned `mame-runtime` layout and validates the executable, `hash`, `bgfx`, and legal-resource paths under Tauri's resource directory.
- `scripts/tauri/stage-mame-runtime.sh` stages the executable, software-list `hash` XML, `bgfx` runtime assets, and legal/COPYING files.
- `scripts/tauri/prepare-bundled-mame-runtime.sh` distinguishes release-qualified real runtime staging from explicit synthetic structural fixtures and records runtime provenance.
- `scripts/tauri/build-linux-bundled-mame-runtime.sh` builds or selects a real Linux MAME executable from the repository source tree and feeds it through the common staging path.
- `scripts/tauri/validate-real-mame-runtime.sh` rejects synthetic payloads and performs bounded `-version` and `-listxml pacman` runtime smoke validation.
- `scripts/tauri/augment-deb-runtime-deps.sh` derives Debian shared-library package dependencies from the bundled MAME executable and rewrites the `.deb` control metadata without adding the distro `mame` package.
- `tauri/src-tauri/tauri.linux-bundle.conf.json` includes `bundle-resources/mame-runtime` in the Linux bundle resources.
- `.github/workflows/tauri-linux-real-runtime-qualification.yml` is the release-grade path that stages a real runtime, builds the `.deb`, augments dependencies, installs the package, verifies the installed runtime, and checks uninstall cleanup.

### Runtime source and settings semantics

- `tauri/src-tauri/src/effective_runtime.rs` owns bundled/default versus explicit external override resolution.
- `tauri/src-tauri/src/config.rs` persists runtime preference semantics so clearing an override returns to bundled/default rather than disabling MAME.
- `tauri/src-tauri/src/app.rs` reports the effective runtime identity to the frontend and no longer treats missing external settings as normal absence of MAME.
- `tauri/src-tauri/src/general_settings.rs` exposes bundled runtime status and keeps external executable selection as an advanced override.
- `tauri/src/settings/GeneralSettingsPanel.tsx` and related frontend tests enforce the normal Settings UX contract.

### Startup, metadata, audit, and launch consistency

- `tauri/src/browser/mameCatalogState.ts` and `tauri/src/browser/MameBrowser.tsx` handle first-run and stale-runtime metadata bootstrap without a normal-user executable selection step.
- `tauri/src-tauri/src/metadata.rs` resolves metadata refresh/status against the effective runtime source.
- `tauri/src-tauri/src/library/audit.rs`, `bulk_audit.rs`, `mame_ui.rs`, `mame_ui_export.rs`, `mame_ui_launch.rs`, and `software.rs` keep audit, availability, BIOS, software-list, Start, Start Empty, and software launch paths on the effective runtime identity.
- `tauri/src-tauri/src/sessions.rs` appends package-owned `hash`/`bgfx` paths for bundled launches and redirects MAME INI/CFG/NVRAM/input/state/snapshot/diff/comment outputs to user-writable application data directories.

### CI and regression tripwires

- `.github/workflows/tauri-project.yml` runs bundled runtime staging, preparation, real-runtime build automation, and BMR TODO wiring guards before frontend/Rust qualification.
- `scripts/tauri/test-bmr-bundled-runtime-todo-wiring.py` verifies the canonical BMR TODO remains in the Tauri workflow trigger/sparse-checkout/test path.
- The Tauri project workflow on `5b07d0010974b1d13eeb5ad019a8f50263d7e785` passed as run `36436956493`, including the BMR TODO wiring guard and all frontend/Rust checks.
- Linux, macOS, and Windows packaging workflows for the same SHA passed as runs `36436956933`, `36436956591`, and `36436956622` respectively.

## Baseline failure mode

Before the bundled-default work, a clean production install with no persisted `settings.mameExecutable` reached runtime identity through an optional external executable path. When that setting was absent, backend application info reported `MameVersionReport::NotConfigured`, even if a package-owned `mame-runtime` payload was present in Tauri resources. General Settings therefore presented a normal-user executable path picker as the first-run recovery path.

The corrected invariant is now:

```text
explicit valid external override -> external runtime
otherwise valid package-owned bundled runtime -> bundled runtime
otherwise -> package/runtime installation failure
```

A missing external override is not a valid representation of “MAME absent” for a correctly packaged production install.

## Preserved invariants

- Rust owns runtime resolution, validation, metadata identity checks, and process launch.
- The frontend cannot self-assert `qualifiedBundled` trust for an arbitrary path.
- ROMs, CHDs, game software, artwork packs, BIOS files, and redistribution-sensitive content remain user supplied and are not bundled by this work.
- Synthetic fixtures remain structural tests only and are not release evidence for a production real-runtime `.deb`.
- Package-owned runtime resources are treated as immutable application resources; writable MAME state is redirected to user-scoped application data.
