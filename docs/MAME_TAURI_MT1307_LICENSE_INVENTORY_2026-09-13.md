# MT-1307 — License inventory qualification

**Qualified date:** 2026-09-13  
**Implementation head:** `5c765e13de73b6ca68365db8b3767cf7ce4f0a4c`

## Scope

MT-1307 requires a reproducible license inventory covering:

- MAME licenses and legal notices;
- Rust dependency licenses;
- JavaScript dependency licenses; and
- redistributed/native-library participation.

## Implementation

The repository now contains `scripts/tauri/generate-license-inventory.py`, which produces deterministic JSON and Markdown reports from the exact locked dependency closure used by CI.

The generator:

- verifies the required MAME legal-notice files are present;
- resolves Rust dependencies with `cargo metadata --locked` and records exact versions, license metadata, repository/source data, and Cargo `links` targets;
- parses the committed npm lockfile and records exact JavaScript package versions and license metadata;
- identifies Cargo packages that participate in native linking;
- records the bundled MAME runtime and platform-runtime policy; and
- fails closed if a resolved Rust or JavaScript package lacks license metadata.

The `Tauri project` workflow generates the inventory and uploads it as a per-SHA artifact named `mame-tauri-license-inventory-<sha>`.

The human review/release policy is documented in `docs/MAME_TAURI_LICENSE_INVENTORY_2026-09-13.md`.

## Failure found and corrected during qualification

The first inventory run correctly found that the project-owned `mame-tauri` Rust crate did not declare package license metadata. Rather than weaken the generator, `tauri/src-tauri/Cargo.toml` now declares `GPL-2.0-only`, matching the repository's MAME GPL v2 licensing notice.

## Exact-head qualification

All application/package workflows for `5c765e13de73b6ca68365db8b3767cf7ce4f0a4c` completed successfully:

- Tauri project: `34744016743` — **success**
- Tauri macOS packaging: `34744016739` — **success**
- Tauri Windows packaging: `34744016747` — **success**
- Tauri Linux packaging: `34744016757` — **success**

The Tauri project run additionally proved that inventory generation and artifact upload themselves complete successfully before the normal frontend and Rust quality gates.

## Result

MT-1307 is complete. License inventory generation is reproducible, fail-closed for missing package metadata, CI-published, and exact-head qualified across all supported package workflows.
