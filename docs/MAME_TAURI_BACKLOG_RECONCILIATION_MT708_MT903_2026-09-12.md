# MAME Tauri Backlog Reconciliation — MT-708 through MT-903

**Date:** 2026-09-12  
**Baseline SHA:** `78fdaf15347478165183e5667b1698620ac6b562`

This audit reconciles stale unchecked planning items in the runtime-control, artwork, and save-state blocks against the executable implementation and existing qualification records. It does not treat the optional native-window research track as a prerequisite for product completion.

## MT-708 — Implement load state

Complete.

- `sessions/load_state.rs` implements the typed load command and explicit `LoadMameStateResult`.
- Missing, invalid, wrong-context, wrong-format, probe-timeout, protocol, and structural-incompatibility paths return stable structured errors.
- The load path performs a fresh compatibility probe against the running session before restoration.
- `docs/MAME_TAURI_MT708_LOAD_STATE_QUALIFICATION_2026-09-11.md` records the prior qualification evidence.
- MT-903 subsequently added regression evidence that incompatible targets are preserved.

## MT-709 — Implement mute/volume controls where native semantics allow

Complete under the selected native-semantics policy.

- The qualified MAME revision exposes UI mute to the Lua runtime-control shim, so `set_mute` is supported.
- A native live master-volume setter is not exposed through the selected control seam; `set_volume` remains deliberately unsupported instead of adding a PCM transport or generic-eval escape hatch.
- PCM never traverses Tauri IPC.
- `docs/MAME_TAURI_MT709_MUTE_VOLUME_QUALIFICATION_2026-09-11.md` records the decision and qualification.

## MT-710 — Implement query-state/status

Complete.

`query_mame_runtime_state` returns bounded typed status for running, paused, machine identity, software context, UI mute, and effective mute. It rejects a runtime machine identity that contradicts the supervised session. The qualification is recorded in `docs/MAME_TAURI_MT710_QUERY_STATE_QUALIFICATION_2026-09-11.md`.

## MT-711 — Protocol adversarial tests

Complete.

`control_adversarial.rs` covers malformed authenticated payloads, decoded and encoded oversize cases, unknown commands/completions, wrong protocol versions, stale-session frames, connection drop during a request, and response timeout. The generated Lua shim is also asserted to contain fail-closed handling for unknown commands, wrong versions, stale sessions, and oversized messages. Qualification is recorded in `docs/MAME_TAURI_MT711_PROTOCOL_ADVERSARIAL_QUALIFICATION_2026-09-11.md`.

## MT-801 — Define artwork data model

Complete.

The typed artwork model represents screenshot, cabinet, marquee, flyer, icon, and system-image slots plus source/provenance metadata. Public artwork descriptors use opaque asset IDs rather than granting frontend filesystem authority.

## MT-802 — Implement local artwork discovery

Complete.

The backend supports ordered configurable artwork roots, native directory selection, safe machine/kind/extension mapping, path validation, symlink/containment checks at the asset-read boundary, and explicit missing-art slots. The local discovery implementation remains independent of network availability.

## MT-803 — Add artwork to machine detail/library views

Complete.

Artwork discovery is scoped to the selected machine/detail view rather than the full catalog. Preview bytes are loaded only for the selected artwork category and use a hard-bounded LRU preview cache, preventing catalog-wide preloading and unbounded WebView memory growth.

## MT-804 — Define external artwork-provider policy

Complete as a decision-gate/policy task.

`docs/MAME_TAURI_DG7_EXTERNAL_ARTWORK_PROVIDER_POLICY_2026-09-12.md` defines all required policy dimensions:

- licensing;
- attribution/provenance;
- privacy and bounded disclosure;
- bounded provider-aware caching;
- offline behavior/failure isolation;
- rate limiting, retry, concurrency, and cancellation bounds.

DG-7 remains **NO-GO for provider enablement by default** until a concrete provider is separately nominated and qualified against that policy. Local artwork remains the default path.

## MT-901 — Define save-state record model

Complete.

The versioned record model contains machine, optional software item, MAME version/build identity, timestamp, logical slot plus application-owned path, byte count, and an optional reserved screenshot metadata field. Records are constructed from an authoritative supervised session plus a completed save result.

## MT-902 — Build save-state browser

Complete.

The application-owned SQLite index and frontend browser provide list, save, load, and confirmed delete operations. Frontend mutation uses backend-issued record IDs; the backend re-authorizes application-owned paths. Compatibility copy never claims a state is proven compatible before the live structural probe.

Exact executable qualification: SHA `40effb0e5485e1f288faa9bf1fff136aaa20ef86`, GitHub Actions run `34710800579` — PASS through frontend formatting/lint/typecheck/tests/build, Rust format/tests, library performance qualification, Clippy, and lockfile integrity.

## MT-903 — Handle incompatible state explicitly

Complete.

The UI states that compatibility is validated at load time and warns when stored/current MAME identity differs. The backend structural-signature failure reports `stateFilePreserved: true`, and regression tests assert byte-for-byte target preservation for structural mismatch, wrong machine, and wrong state format. Only temporary probe files are cleanup targets.

Exact executable qualification: SHA `78fdaf15347478165183e5667b1698620ac6b562`, GitHub Actions run `34710946504` — PASS through all required Tauri quality gates.

## Reconciliation result

MT-708 through MT-711, MT-801 through MT-804, and MT-901 through MT-903 are complete. Their unchecked boxes in the original planning TODO are stale bookkeeping. This ledger supplies the evidence basis for final MT-2201 TODO reconciliation without reopening already-qualified implementation work.
