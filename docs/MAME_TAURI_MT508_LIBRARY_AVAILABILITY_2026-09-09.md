# MT-508 — Library availability integration

**Date:** 2026-09-09  
**Status:** Closed  
**Branch:** `ralph/mt-508-library-availability`  
**Implementation commit:** `9724056677e315990ba644fef5efa070446d9770`  
**Implementation parent:** `a9ecefb951129e0a507830b8109a25683f509e73`  
**Authoritative Tauri qualification:** GitHub Actions run `34446362812`

## Scope

MT-508 integrates the MT-500 audit subsystem into the primary machine library without introducing filesystem-presence heuristics or a second availability source of truth.

The implementation adds:

- an `Available`, `Missing`, and `Unknown` availability model for library rows;
- server-side availability filtering before pagination;
- an availability badge on each machine row;
- current-provenance gating against the persisted MT-505 audit results;
- immediate library refresh after per-machine audit, bulk audit, or content-path changes;
- fail-closed behavior when current executable provenance cannot be trusted;
- explicit propagation of storage, settings, catalog, and malformed-state failures instead of silently presenting them as `Unknown`.

## Authoritative availability mapping

Availability is derived only from persisted MAME audit classifications whose stored provenance matches both the current MAME executable identity and the current configured content paths.

The library mapping is:

| Persisted audit classification | Library availability |
| --- | --- |
| `complete` | `available` |
| `bestAvailable` | `available` |
| `missingRequired` | `missing` |
| `incorrect` | `missing` |
| `mixedFailure` | `missing` |
| `unknown` | `unknown` |
| no current-provenance audit row | `unknown` |

`bestAvailable` is deliberately considered available for library filtering while remaining distinct in the underlying structured audit result. This preserves the MT-503/MT-504 distinction instead of falsely rewriting MAME's best-available result as fully correct.

## Provenance and fail-closed behavior

The catalog query joins `machine_audit_results` using:

1. the machine short name;
2. serialized current `MameExecutableIdentity`;
3. serialized current `ContentPathsV1`.

A stale result therefore does not match the join and is exposed as `Unknown`. A path or executable change cannot cause an old audit to be displayed as valid.

When current executable provenance cannot be established safely — for example, a missing/unusable executable, an unrecognized or failed version probe, stale metadata identity, or unresolved bundled-executable ownership — the library remains browseable but uses an unverified availability query. All machines are therefore `Unknown` rather than optimistically valid.

Operational failures are not hidden by that fallback. Catalog/database errors, settings-read failures, invalid persisted source/trust pairings, and other non-provenance failures continue to propagate as errors.

## Query and performance design

Availability filtering is performed in the catalog SQL query before pagination. The frontend does not perform one audit lookup per visible machine and does not infer availability from local files.

The page response keeps existing `MachineListItem` consumers stable and carries availability separately in `availabilityByShortName`, keyed by machine short name. This avoids forcing unrelated favorites, collections, and history surfaces to claim an availability state they did not query with current provenance.

The existing 100,000-row library UX performance qualification passes with the provenance-gated audit join enabled, demonstrating that MT-508 preserves the bounded primary-library query path.

## UI behavior

The library filter includes:

- Any availability
- Available
- Missing
- Unknown / not verified

Each machine row displays a separate availability badge in addition to its MAME driver-status badge.

The library availability revision is bumped after:

- a successful per-machine audit;
- bulk-audit result changes;
- a successful content-path add/remove/reorder operation.

This prevents a previously rendered `Available` badge from surviving a content-path change that invalidates its audit provenance.

## Verification

Exact implementation commit `9724056677e315990ba644fef5efa070446d9770` was qualified by GitHub Actions run `34446362812`.

The `linux-quality` job passed all substantive gates:

- frontend format check;
- frontend lint;
- TypeScript typecheck;
- frontend tests;
- frontend production build;
- Rust format;
- Rust tests, including availability/provenance fixtures;
- 100,000-row Library UX performance qualification;
- Rust Clippy;
- lockfile integrity.

The release-qualification job was skipped according to the existing workflow policy for this push and is not an MT-508 acceptance requirement.

The earlier hardening materializer run `34445687742` also passed, but it is not the authoritative product qualification. Its generated product changes were normalized into the clean single implementation commit above, and all temporary materializer files were removed before final qualification.

## Acceptance closure

MT-508 satisfies all TODO acceptance items:

- [x] filter available/missing/unknown;
- [x] status badge;
- [x] do not label unverified content as valid.

With MT-501 through MT-508 complete, **MT-500 — ROM/software paths and auditing is closed**.

The next active task is **MT-601 — Inventory relevant MAME configuration sources**.
