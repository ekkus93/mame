# Bundled MAME Distribution Baseline

**Date:** 2026-09-27  
**Canonical TODO:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`

## Baseline failure

At the start of BMR implementation, the repository already contained package-resource plumbing for a bundled MAME runtime, but product runtime selection still depended on the optional persisted `mameExecutable` setting. With no external path configured, `app.rs` returned `MameVersionReport::NotConfigured`, so General Settings presented an executable path field, Browse, Save, and Clear as normal setup.

Linux package smoke staged a synthetic shell script under `mame-runtime/bin/mame`. This proved resource placement and package mechanics, but did not prove that a production `.deb` contained or could execute real MAME.

## Authoritative implementation paths

| Concern | Canonical path(s) |
| --- | --- |
| Package-owned runtime layout/containment | `tauri/src-tauri/src/bundled_runtime.rs` |
| Effective bundled/external runtime selection | `tauri/src-tauri/src/effective_runtime.rs` |
| Runtime identity/version probing | `tauri/src-tauri/src/mame/executable.rs`, `tauri/src-tauri/src/app.rs` |
| Settings persistence/migration | `tauri/src-tauri/src/config.rs`, `tauri/src-tauri/src/general_settings.rs` |
| Metadata generation/freshness | `tauri/src-tauri/src/metadata.rs`, `tauri/src-tauri/src/metadata/generator.rs` |
| Catalog-backed launch/audit/software/BIOS | `library.rs`, `library/audit.rs`, `mame_ui_launch.rs`, `software.rs`, `bulk_audit.rs` |
| Supervised native MAME launch | `tauri/src-tauri/src/sessions.rs`, `sessions/supervisor.rs` |
| User-writable MAME state | `tauri/src-tauri/src/storage.rs` |
| Normal/advanced settings UX | `tauri/src/settings/GeneralSettingsPanel.tsx` |
| Startup metadata state machine | `tauri/src/browser/MameBrowser.tsx`, `mameCatalogState.ts` |
| Linux runtime staging | `scripts/tauri/stage-mame-runtime.sh` |
| Structural package smoke | `scripts/tauri/test-linux-packages.sh`, `.github/workflows/tauri-linux-packaging.yml` |
| Real-runtime rejection/qualification | `scripts/tauri/validate-real-mame-runtime.sh` |
| Linux bundle resource mapping | `tauri/src-tauri/tauri.linux-bundle.conf.json` |

## Invariants

Rust remains authoritative for executable/resource resolution and process launch. The WebView cannot self-assert `qualifiedBundled` trust or turn arbitrary paths into generic shell commands.

This work bundles the MAME emulator/runtime and redistributable MAME resources only. It does **not** bundle ROMs, CHDs, copyrighted game software, or undistributable firmware/BIOS content.

Package-owned runtime files are treated as immutable. Mutable CFG/NVRAM/state/snapshot/diff data belongs under the application user-data directory.

The effective runtime identity is one of:

- package-owned bundled MAME, used by default; or
- an explicit validated external override.

A missing external override is therefore a normal bundled-runtime preference, not an unconfigured application.
