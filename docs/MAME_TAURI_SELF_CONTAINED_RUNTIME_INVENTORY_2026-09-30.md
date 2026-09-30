# MAME Tauri Self-Contained Runtime Inventory

**Date:** 2026-09-30  
**Reset TODO:** `docs/MAME_TAURI_SELF_CONTAINED_UI_PARITY_RESET_TODO_2026-09-30.md`  
**Spec:** `docs/MAME_TAURI_SELF_CONTAINED_UI_PARITY_RESET_SPEC_2026-09-30.md`  
**Scope:** RESET-001 evidence for packaged self-contained runtime behavior and non-default external/development runtime selection.

## Product invariant

The normal packaged Tauri app must own MAME runtime resolution. A normal packaged user must not need to install upstream MAME separately or point the app at an external `mame` binary before using the app.

External and development-tree MAME sources may remain available as advanced/developer escape hatches, but they must not be the default product path and must not be presented as required setup.

## Backend runtime resolution inventory

### Effective runtime resolution

Source path: `tauri/src-tauri/src/effective_runtime.rs`

The backend resolves the effective MAME source from settings plus the package resource directory:

1. If a configured external override exists, it is validated as an explicit external source.
2. Otherwise, the backend resolves the package-owned bundled runtime from the Tauri resource directory via `BundledRuntimeLayout`.
3. The default settings path resolves to a `bundled` / `qualifiedBundled` runtime rather than `notConfigured`.

Existing tests in this area cover:

- default settings resolving to the bundled runtime;
- explicit external override remaining explicit;
- empty external override being rejected before bundled fallback.

### Bundled runtime layout and package resources

Source paths:

- `tauri/src-tauri/src/bundled_runtime.rs`
- `tauri/src-tauri/src/lib.rs`
- `tauri/src-tauri/src/effective_runtime.rs`

The package-owned runtime layout is backend-owned. The backend validates package resource layout and source/trust identity before treating a runtime as bundled. The frontend does not get to self-assert an arbitrary path as `qualifiedBundled`.

### App/runtime identity reporting

Source paths:

- `tauri/src-tauri/src/app.rs`
- `tauri/src-tauri/src/diagnostics.rs`

App/runtime identity reports distinguish bundled package runtime from external override failures. Existing tests include default settings resolving the package-owned bundled runtime and diagnostics classification for bundled versus external runtime failures.

## Frontend runtime-selection inventory

### Normal settings UI

Source paths:

- `tauri/src/settings/GeneralSettingsPanel.tsx`
- `tauri/src/settings/GeneralSettingsPanel.test.ts`

The normal settings UI presents bundled runtime status and labels external runtime selection as an advanced override. Existing source/component tests verify that the normal settings source does not contain the obsolete required setup copy such as `Choose a MAME executable` and presents `Use bundled MAME` recovery/reset behavior.

### Metadata/catalog frontend mapping

Source paths:

- `tauri/src/browser/mameCatalogState.ts`
- `tauri/src/browser/mameCatalogState.test.ts`
- `tauri/src/browser/MameCatalogStatePanel.tsx`
- `tauri/src/browser/MameCatalogStatePanel.test.ts`

The frontend maps bundled runtime metadata requests to backend-owned bundled resolution rather than passing an arbitrary path. Existing tests cover backend-owned bundled resolution for catalog metadata and continued explicit handling of external overrides.

## Backend launch and catalog trust inventory

### Frontend-selectable launch sources

Source path: `tauri/src-tauri/src/sessions.rs`

The `MameExecutableSelectionKind` exposed to untrusted frontend launch requests intentionally represents only `External` and `DevelopmentTree`. A release-qualified bundled executable is not representable as a frontend-supplied arbitrary path. Existing tests assert that frontend-selectable sources cannot self-assert bundled trust.

### Catalog-backed launch

Source paths:

- `tauri/src-tauri/src/library.rs`
- `tauri/src-tauri/src/software.rs`
- `tauri/src-tauri/src/mame_ui_launch.rs`

Persisted catalog data may contain source/trust identity for the generation, but bundled catalog launch requires backend/package-owned source resolution. Existing tests cover:

- bundled catalog launch using a backend-resolved effective runtime;
- bundled catalog launch failing if an effective bundled source is not supplied;
- mismatched source/trust pairs being rejected.

This prevents persisted catalog data or frontend-originated data from self-asserting bundled trust for arbitrary filesystem paths.

## Packaging and workflow inventory

Existing BMR evidence records the self-contained package direction and package qualification history:

- `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`
- `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_SPEC_2026-09-27.md`

Relevant package workflow evidence already recorded there includes:

- Linux real-runtime package qualification with a real MAME executable staged into the package resource tree.
- Installed backend bundled-source verification.
- Installed bundled-MAME metadata bootstrap/catalog verification.
- Installed external override/reset qualification proving explicit override can be cleared back to bundled/default runtime.
- Linux, macOS, and Windows packaging smoke workflows validating bundled runtime path wiring for their package layouts.

The reset still keeps package/integration behavior as a final acceptance requirement in RESET-008 and RESET-010, but RESET-001's runtime-source/default-path contract is already implemented and covered by existing source, frontend, backend, and package evidence.

## External/development runtime surfaces retained as advanced/debug behavior

External/development surfaces still exist intentionally:

- backend source kind `external` for explicit user-configured overrides;
- backend source kind `developmentTree` for development use;
- frontend advanced external runtime override UI;
- backend command for picking an executable path.

These are retained as escape hatches. They are not the default packaged app path and must not be expanded until the reset acceptance criteria are met.

## RESET-001 conclusion

RESET-001 is considered complete for the reset baseline:

- the backend default runtime source is bundled/package-owned;
- external runtime configuration is explicit and advanced;
- untrusted frontend requests cannot self-assert `qualifiedBundled` trust;
- catalog-backed launch requires backend-resolved bundled source for bundled generations;
- existing tests and package evidence cover the normal packaged flow without a user-selected external MAME binary.

Future work should not reopen external-runtime-first behavior. Remaining package-level user-flow smokes belong to RESET-008 and final closure evidence belongs to RESET-010.
