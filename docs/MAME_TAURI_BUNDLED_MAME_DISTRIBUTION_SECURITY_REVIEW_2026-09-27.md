# Bundled MAME Distribution Security and Redistribution Review

**Date:** 2026-09-27  
**Canonical TODO:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`

## Runtime authority

The package-owned runtime is resolved in Rust from Tauri's resource directory. `BundledRuntimeLayout` canonicalizes the package resource root, runtime root, executable, hash directory, BGFX directory, COPYING file, and legal directory, and rejects any canonical path escaping the package-owned root.

A missing persisted executable path means bundled/default. An external executable remains an explicit advanced override and is validated before persistence. The WebView cannot self-assert `qualifiedBundled` trust for an arbitrary path.

Metadata refresh/status, catalog-backed machine/software launch, Start Empty, BIOS/software queries, audit, export availability, and the retained typed generic launch path use the effective Rust runtime authority. Catalog-backed launch additionally compares the active metadata generation identity with the currently inspected runtime and fails with `MAME_METADATA_STALE` on mismatch.

## Process boundary

No generic shell command bridge is introduced. MAME is spawned as a supervised native child with discrete typed argv entries. Machine/software/BIOS identifiers and project-controlled paths remain validated in Rust. The advanced external picker validates the executable before storing it.

Package-owned resources are treated as immutable. Bundled launches pass explicit package-owned `hashpath` and `bgfx_path` arguments and direct mutable CFG, NVRAM, state, snapshot, and diff output into the application user-data tree. The real package smoke also verifies the installed runtime root is not user-writable.

## Diagnostics and privacy

Runtime failures report bounded source/path/version/error provenance needed to diagnose a corrupt package or invalid override. Diagnostics do not add arbitrary filesystem enumeration or expose unrelated user files to the WebView.

## Redistribution

The staged runtime includes the repository `COPYING` file and the `docs/legal` tree. Real-runtime staging writes `provenance.json` containing source revision, reported MAME version, build target/profile/host, executable SHA-256, and qualification kind.

The bundled product includes the emulator and redistributable runtime resources only. It does **not** bundle ROMs, CHDs, copyrighted game software, or non-redistributable firmware/BIOS dumps. User content remains external and user-supplied.

## Qualification

Security regression CI remains applicable to the Tauri/Rust changes. Linux release qualification additionally proves that a synthetic fixture is rejected by the real-runtime validator, that installed package resources are package-owned/read-only, that no distro `mame` dependency is declared, and that the real binary executes from an arbitrary working directory.
