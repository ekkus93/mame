# MT-607 — Controller Profile Data Model Qualification

**Date:** 2026-09-10  
**Repository:** `ekkus93/mame`  
**Task:** MT-607 — Add controller profile data model  
**Parent closure:** MT-606 `e4998a6c17048f0c635d98722968f9f478c0f238`  
**Implementation:** `a5d54f1c0551bb3c75458df3fbcc83acd79a9d24`  
**Implementation tree:** `fbdfb4d70de4730b58ca7f594eceff24bc6f9509`  
**Exact-head CI:** GitHub Actions run `34479984748` — PASS

## Scope

MT-607 adds the durable backend data model needed for controller profiles. It does not add controller-profile UI and does not route gameplay input through Tauri or the WebView. MAME remains the gameplay-input authority.

The implementation is intentionally source-aware because MT-601 established that MAME core `-ctrlr` controller configurations and the generic OSD `controller_map` option are separate mechanisms. MT-408 also established that the browser Gamepad API `id` is useful frontend identity evidence but must not be presented as a guaranteed physical-device serial number.

## Model

`tauri/src-tauri/src/controller_profiles.rs` defines versioned typed records for:

- controller profile identity;
- target controller/device identity;
- mapping provenance;
- global or exact-machine assignment scope.

A persisted profile has a stable SQLite integer ID, human-readable name, target-device identity, mapping provenance, and create/update timestamps.

### Target-device identity

`ControllerDeviceIdentityKind` distinguishes:

- `browserGamepadId` — the browser/OS-reported Gamepad API ID string;
- `mameInputDeviceId` — reserved for a future native/MAME input-device identifier;
- `userDefined` — an explicit user label when no stronger platform identity exists.

The model separately preserves an optional reported mapping label such as `standard`. This is descriptive evidence only; it is not treated as proof that a mapping is active at runtime.

### Mapping provenance

`ControllerMappingProvenanceKind` keeps distinct:

- W3C/browser standard-gamepad mapping;
- MAME core `-ctrlr` controller configuration;
- generic OSD `controller_map` mapping;
- project-owned mapping data.

A bounded optional source reference can preserve a `-ctrlr` profile name, OSD map path, or project mapping identifier without collapsing these mechanisms into one ambiguous string.

### Association scope

`ControllerProfileScope` represents either:

- `Global`; or
- `Machine { short_name }`.

Machine short names use the existing MAME identifier validator.

Profile identity and assignment are persisted separately. This prevents the meaning of a profile from being coupled to where it currently applies and lets MT-608 inspect/select profiles without rewriting profile identity.

## SQLite schema v4

Catalog schema version advances from 3 to 4.

New tables:

- `controller_profiles` — profile identity, typed device/provenance JSON, and timestamps;
- `controller_profile_assignments` — global/machine scope records referencing profiles.

Assignments intentionally have no foreign key to generated `machines` metadata. Metadata regeneration or a MAME upgrade therefore cannot cascade-delete user controller choices. Assignments do reference their owning profile with `ON DELETE CASCADE`, so deleting a profile removes only state owned by that profile.

The scope table stores an explicit `scope_kind` plus normalized `scope_key`; SQL checks reject structurally inconsistent global/machine rows.

## Persistence API

The Rust module exposes typed operations to:

- create and update profiles;
- get or list profiles;
- assign a profile globally or to one machine;
- list a profile's assignments;
- delete a profile.

Profile names, device IDs, reported mapping labels, provenance references, timestamps, and machine identifiers are validated/bounded before persistence. Stored JSON is decoded back into typed structures and corrupt stored data becomes an explicit structured error rather than a silent fallback.

## Tests

The MT-607 regression coverage verifies:

- stable profile identity plus target-device/provenance round-trip;
- distinct global and machine associations;
- separation of MAME `-ctrlr` provenance from OSD `controller_map` provenance;
- machine-scope persistence without a generated metadata row;
- profile-owned assignment cascade on profile deletion;
- rejection of invalid/nonpositive profile IDs before database access;
- schema migration to v4 and presence of both controller-profile tables through the existing migration suite.

## Qualification

Exact-head Tauri quality run `34479984748` passed on implementation SHA `a5d54f1c0551bb3c75458df3fbcc83acd79a9d24`.

The run passed:

- frontend formatting, lint, TypeScript typecheck, tests, and production build;
- Rust formatting;
- Rust tests, including MT-607 model/schema tests;
- the inherited 100k-row library performance qualification;
- Rust Clippy;
- lockfile-integrity verification.

## Acceptance mapping

MT-607 acceptance is satisfied as follows:

- **profile identity** — stable SQLite profile ID plus bounded profile name and schema version;
- **target device/controller identity** — typed identity source/value plus optional reported mapping label;
- **machine/global association** — separate validated assignment model with global and exact-machine scopes;
- **mapping provenance** — typed distinction between browser-standard, MAME `-ctrlr`, OSD `controller_map`, and project-owned mappings, with bounded source reference.
