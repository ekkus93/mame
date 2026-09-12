# MAME Tauri MT-1301 — Developer executable modes

**Date:** 2026-09-12  
**Task:** MT-1301 — Define developer executable modes  
**Status:** Complete

MT-1301 is a source/trust and authority-definition task. The executable boundary now has explicit semantics for all three required modes, consistent with DG-2.

## External MAME path

An explicitly user-selected executable is represented by `MameExecutableSourceKind::External` and receives `MameExecutableTrust::UserConfigured`. The frontend may supply this path through the typed `MameExecutableRequest`, and the Rust backend canonicalizes, validates, and version-probes the executable before use. No shell command string is constructed from frontend input.

## In-tree development build

Contributor/test workflows may select an in-tree executable through `MameExecutableSelectionKind::DevelopmentTree`. The backend maps it to `MameExecutableSource::development_tree` and `MameExecutableTrust::Development`, keeping development identity distinct from both external user configuration and release-qualified binaries.

## Bundled sidecar build where supported

The executable model reserves `MameExecutableSourceKind::Bundled` and `MameExecutableTrust::QualifiedBundled`. A bundled executable is deliberately **not** representable in the untrusted path-based frontend request. Its path must be resolved by the Rust/package layer from release-owned packaging metadata. MT-1302 and platform packaging tasks own the actual staged runtime resources and package-format integration.

This separation prevents an arbitrary renderer-supplied path from self-asserting `qualifiedBundled` trust and also prevents silent fallback from a missing bundled binary to a system/PATH executable.

## Executable identity contract

Every selected executable is validated as a usable regular executable and probed with the discrete argument vector:

```text
-noreadconfig
-version
```

The resulting identity records source class, trust classification, canonical path, MAME version, optional build descriptor, and the bounded raw version line. Metadata/session provenance therefore remains tied to the actual executable in use.

## Verification

The Rust executable-source implementation defines all three source/trust classes. `sessions.rs` exposes only `external` and `developmentTree` to path-bearing frontend IPC, and its unit test `frontend_selectable_sources_cannot_self_assert_bundled_trust` proves those requests cannot acquire bundled trust.

MT-1301 is complete. Actual redistribution, runtime-resource staging, license preservation, and installed-path validation remain MT-1302+ concerns rather than ambiguities in the executable-mode model.
