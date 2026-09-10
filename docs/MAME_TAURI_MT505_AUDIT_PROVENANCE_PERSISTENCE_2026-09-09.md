# MT-505 — Audit-result persistence with provenance

Date: 2026-09-09

## Status

**Qualified for closure.**

Implementation commit:

`9a2c131ab2c84aca09085bdaca26c472b2a4235a`

Implementation tree:

`0040df41cad5cbfb99d954f3cdf641ab4052939b`

Authoritative Tauri qualification:

- workflow run: `34437998222`
- result: **PASS**
- exact tested head: `9a2c131ab2c84aca09085bdaca26c472b2a4235a`

The run passed frontend format/lint/typecheck/tests/build, Rust formatting, Rust tests, the inherited MT-410 library performance qualification, Clippy with warnings denied, and lockfile-integrity verification.

## Scope

MT-505 persists structured per-machine audit results from MT-504 and makes their validity explicitly dependent on the environment that produced them. A previously successful audit is not treated as current merely because a row exists in SQLite.

The persisted provenance consists of:

1. the complete `MameExecutableIdentity` used for the audit; and
2. the complete ordered `ContentPathsV1` configuration used for the audit.

This intentionally includes path ordering and the lossless platform-path representation established by MT-501.

## Storage model

Catalog schema version advances from v1 to v2 through an incremental transactional migration. Schema v2 adds `machine_audit_results` with:

- `machine_short_name` as the per-machine key;
- an indexed normalized classification token;
- the full structured MT-504 parser result as JSON;
- audit timestamp;
- exact MAME-identity JSON;
- exact ordered content-path JSON.

The table deliberately has no foreign key into generated metadata. Metadata regeneration therefore cannot cascade-delete audit rows before the application has a chance to evaluate their provenance.

The v1→v2 migration preserves existing generated metadata and user-owned state. A regression test seeds v1 user state and verifies it survives migration.

## Persistence contract

`save_machine_audit_result` upserts a machine result together with the exact MAME identity and content-path configuration that produced it.

`load_current_machine_audit_result` first compares persisted provenance against the current identity and path configuration. Stale rows are deleted before the requested machine result is returned.

`invalidate_stale_machine_audit_results` provides explicit bulk invalidation and returns the number of rows removed.

Persisted classification is stored both as an indexed token and inside the structured parser result. Reads verify the two representations agree; inconsistent stored data is rejected rather than silently trusted.

## Staleness rules

An audit result is stale when either of these changes:

- MAME executable identity, including version/build/path/source/trust/raw version identity; or
- configured ROM/software/CHD paths, including ordering and non-UTF-8 path bytes.

Consequences:

- a MAME upgrade cannot inherit old audit truth;
- changing or reordering content paths invalidates old results;
- non-UTF-8 paths are compared losslessly rather than through lossy display strings;
- current provenance is preserved while mismatching rows are removed.

## Qualification coverage

Rust tests verify at minimum:

- exact-provenance save/load round trip;
- changed MAME identity invalidates and deletes persisted results;
- changed or reordered paths invalidate results;
- explicit invalidation deletes stale rows while preserving current rows;
- non-UTF-8 platform paths participate losslessly in provenance;
- v1→v2 migration preserves existing user state;
- future unsupported schema versions are rejected;
- failed migrations roll back rather than leaving partial schema state.

## MT-505 acceptance

- Result tied to MAME identity: **satisfied**.
- Result tied to configured content paths: **satisfied**.
- Stale audit results invalidated: **satisfied**.

## Boundary to MT-506

MT-505 does not execute MAME audit commands and does not add UI actions. MT-506 is responsible for the per-machine audit operation: executing the authoritative MT-503 command, feeding captured output into the MT-504 parser, persisting the result through this MT-505 store, and exposing progress/structured diagnostics to the frontend.
