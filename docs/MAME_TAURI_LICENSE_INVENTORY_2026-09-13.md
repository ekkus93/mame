# MAME Tauri license inventory

**Document date:** 2026-09-13  
**Task:** MT-1307 — License inventory artifact

## Purpose

Release qualification must have a reproducible inventory of the licenses that apply to the MAME Tauri frontend and the runtime material it redistributes. The inventory is generated from the same locked dependency inputs used by CI rather than maintained as a hand-edited package list.

## Generated artifacts

Run from the repository root:

```sh
python3 scripts/tauri/generate-license-inventory.py
```

The generator writes:

```text
artifacts/license-inventory/mame-tauri-license-inventory.json
artifacts/license-inventory/mame-tauri-license-inventory.md
```

The `Tauri project` workflow generates the same files and uploads them as a per-SHA workflow artifact named `mame-tauri-license-inventory-<sha>`.

## Inventory domains

### MAME runtime and legal notices

The packaged frontend redistributes a MAME executable and the runtime resources selected by the MT-1302 staging contract. MAME's umbrella notice is `COPYING`; full common license texts are under `docs/legal`; `3rdparty/README.md` records the third-party components used by MAME.

The generator refuses to produce an inventory if the required MAME legal-notice set is missing from the checkout.

### Rust dependencies

The generator executes:

```sh
cargo metadata --locked --format-version 1 \
  --manifest-path tauri/src-tauri/Cargo.toml
```

It records the resolved package name, exact version, SPDX-style license expression or declared license file, source/repository metadata, and any Cargo `links` target. A resolved Rust package with neither a license expression nor a license file causes inventory generation to fail.

Packages with a Cargo `links` value are also surfaced under `nativeLibraries.cargoLinkedPackages`, making native/static library participation visible. This is important for components such as the bundled SQLite path used by `rusqlite`.

### JavaScript dependencies

The generator reads the committed npm v3 lockfile, `tauri/package-lock.json`, and records every locked package with its exact version, license expression, resolved source, development-only status, and optional status.

A locked package without license metadata causes inventory generation to fail.

### Redistributed native/platform libraries

The generated artifact distinguishes project-redistributed components from platform-provided runtime facilities:

- the MAME sidecar/runtime is redistributed and remains covered by MAME's `COPYING` and component notices;
- native Cargo packages declaring a `links` target are explicitly listed;
- Linux WebKit/AppIndicator packages used as system dependencies are not treated as project-owned source; built AppImage contents still require release-artifact review because AppImage bundling can copy host libraries;
- Windows WebView2 and macOS WebKit are platform runtimes and are not represented as project-owned source dependencies.

This distinction prevents the source dependency inventory from falsely claiming that every platform runtime library is redistributed by the project.

## Release use

For a release candidate:

1. Generate the inventory from the exact candidate SHA.
2. Preserve the CI artifact alongside package artifacts.
3. Review `nativeLibraries.cargoLinkedPackages` and package-format-specific bundled native files.
4. Ensure the staged MAME `COPYING`/legal resources remain present in the packaged runtime.
5. Treat newly missing/unknown dependency license metadata as a release blocker until reviewed.

The inventory is an engineering/compliance aid. It does not replace legal review when redistribution behavior or dependencies materially change.
