# MAME Tauri Bundled Runtime User Contract

**Date:** 2026-09-28  
**Related specification:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_SPEC_2026-09-27.md`  
**Related checklist:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`

This document describes the user-facing runtime contract for the Debian/Ubuntu MAME Tauri distribution.

## Product contract

The Debian/Ubuntu package is intended to install the Tauri frontend, Rust backend, and a package-owned MAME runtime as one application. A normal user should not need to install the distribution `mame` package, download a separate MAME binary, browse for an executable, or type an executable path before the application can identify the active runtime and initialize metadata.

The normal first-run flow is:

```text
install the .deb
launch the application
backend resolves the package-owned bundled MAME runtime
metadata status is checked for that runtime identity
missing or stale metadata starts importing automatically
machine browser becomes available after metadata import succeeds
```

The package does not include ROMs, CHDs, game software, artwork collections, BIOS files, or other user-supplied/redistribution-sensitive content. Users still configure content paths for their own files.

## Runtime source semantics

The runtime source is resolved in Rust. The frontend cannot self-assert package-owned bundled trust for an arbitrary path.

The effective source order is:

1. an explicit, validated external override, when configured; otherwise
2. the package-owned bundled runtime resolved from the Tauri resource directory.

A missing or corrupt bundled runtime is treated as a package/runtime installation failure, not as a prompt for a normal user to locate MAME manually.

## Settings and advanced override

General Settings presents bundled runtime status as the normal path. The external MAME executable field is an advanced/developer override only.

Clearing the override means “Use bundled MAME,” not “disable MAME.” Existing launch preferences, content paths, controller profiles, favorites, collections, and history are user data and should remain independent of which runtime source is active unless a runtime/catalog identity mismatch requires metadata refresh.

## Metadata bootstrap

When metadata is missing on first run, the application starts metadata import automatically. When the active runtime identity changes, the existing catalog becomes stale and metadata refresh starts automatically before normal catalog browsing or catalog-backed launch proceeds.

The shell must keep progress visible while metadata import is running and preserve retry/diagnostic behavior when metadata generation fails.

## Runtime consistency

Metadata generation, machine browsing, machine audit, and catalog-backed launch must use the same effective runtime identity. A catalog generated from one runtime must not silently authorize launch against another runtime. Rust revalidates runtime identity and launch identifiers before starting MAME.

## Packaging and qualification

Fast package-layout CI may use synthetic fixtures, but those fixtures are structural tests only. They are not release evidence for a production Debian/Ubuntu package.

Release-grade Linux qualification must build or otherwise provide a real MAME executable, stage it with `hash`, `bgfx`, `COPYING`, and legal/runtime resources, record provenance in `runtime-provenance.txt`, build the `.deb`, install it in a clean environment, verify the installed bundled runtime executes `-noreadconfig -version`, and verify the package does not depend on an externally installed `mame` binary.

The manual workflow `.github/workflows/tauri-linux-real-runtime-qualification.yml` is the release-grade path for that real-runtime package evidence. The synthetic Linux packaging workflow remains useful for fast bundle layout, install/uninstall, desktop-entry, and AppImage structural checks.
