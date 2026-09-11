# MAME Tauri Backlog Reconciliation — MT-200, MT-300, MT-401

**Date:** 2026-09-11  
**Baseline SHA:** `bc417f32e56a633f7a52150ac1b05eed93216f7a`

This audit reconciles early unchecked planning items against the current implementation. It does not mark an item complete merely because similarly named code exists; each conclusion below is tied to executable implementation and regression coverage.

## MT-200 — MAME executable and sidecar integration

MT-200 is implemented in the current codebase.

### MT-201 / MT-202 — executable source, configuration and identity

`mame/executable.rs` defines explicit bundled, external and development-tree sources, maps them to distinct trust classes, validates regular/executable paths, probes MAME with a bounded `-noreadconfig -version` process, records path/version/build/raw identity, and returns structured failures for missing, unusable, non-executable, timeout, output and exit errors.

Regression coverage includes explicit external configuration, empty configuration rejection, source/trust identity, Unicode/space-containing executable paths, missing executables and non-executable files.

### MT-203 — safe argv construction

`mame/argv.rs` constructs `Vec<OsString>` arguments directly and never creates a shell command string. Machine/software identifiers, option names and project-controlled paths are validated before launch. Tests cover spaces, Unicode, metacharacters, malformed machine/software identifiers, traversal, path-list injection and environment-expression injection.

### MT-204 / MT-205 — supervised launch and diagnostics

`sessions/supervisor.rs` creates a unique session ID, records creation/start/end timestamps, PID, executable identity, effective argv/config, spawns MAME directly, captures stdout/stderr on bounded background readers, retains only the recent diagnostic tail, records truncation and associates all diagnostics with the session snapshot.

### MT-206 — lifecycle state machine

The supervisor implements `created`, `starting`, `running`, `stopping`, `exited`, `failed` and `crashed`, validates legal transitions and rejects impossible transitions. Tests cover normal exit, crash and invalid terminal transitions.

### MT-207 — clean stop

MT-706 subsequently strengthened the original MT-207 path: authenticated native protocol exit is preferred, followed by the existing bounded OS soft-stop and forced-kill escalation. `StopSessionResult` and the session snapshot report forced termination truthfully.

### MT-208 — abnormal exit

The exit watcher differentiates successful exit, requested stopping, abnormal crash and supervision failure. It emits `session.exited`, `session.crashed` or `session.failed` with the bounded diagnostic snapshot. Terminal states are inactive, allowing a later launch instead of leaving an active-session latch set.

### MT-209 — concurrent sessions

The initial product policy is single active MAME session. `launch_gate` serializes launch decisions and an active session produces `MAME_SESSION_ALREADY_ACTIVE`; regression coverage verifies a second launch is rejected. Multi-session resource isolation is therefore not an active requirement under the selected policy.

### MT-210 — integration regression coverage

Existing tests cover successful supervised launch, executable validation failures, malformed launch identifiers/paths, clean exit, abnormal exit, forced termination, bounded stdout/stderr capture, path quoting/Unicode/metacharacter behavior and duplicate active launches.

**Reconciliation result:** MT-201 through MT-210 acceptance is satisfied by current implementation and tests. The TODO checkboxes are stale bookkeeping.

## MT-300 — metadata subsystem

MT-302 through MT-309 are implemented. MT-301 has one genuine provenance gap described below.

### MT-301 — representative fixture

`tauri/tests/fixtures/listxml-representative.xml` is small and deliberately covers parents/clones, a device entry, display/chip metadata and software-list associations. However its root build is explicitly `0.288 test-fixture`; it is a synthetic representative fixture, not auditable captured output from a pinned MAME executable.

Therefore these MT-301 requirements remain open:

- generate/capture the fixture from a pinned or otherwise known MAME build;
- record the producing executable identity as provenance.

The bounded-size and representative-content requirements are satisfied.

### MT-302 — streaming parser

`metadata/parser.rs` uses `quick_xml::Reader` over `BufRead` and emits each completed machine through a callback rather than retaining the whole XML document. It parses machine identity/description/year/manufacturer, clone/ROM relationships, source, driver state, displays, devices and software-list associations. Unknown elements are ignored while known fields continue parsing.

Tests cover representative metadata, future/unknown elements, Unicode, malformed XML and empty catalogs.

### MT-303 / MT-304 — SQLite schema and migrations

`storage/migrations.rs` maintains a singleton schema version and forward transactional migrations through schema v4. The schema contains generated metadata domains, indexed machine search fields, software relationships and user-owned favorites/collections/history/tags. Tests verify current schema creation, forward migration without user-state loss, future-version rejection and rollback of failed migrations.

### MT-305 — transactional generation/import

`metadata/generator.rs` runs `-listxml` outside the UI thread through `spawn_blocking`; stdout is streamed directly into a transaction. A generation remains inactive until parsing, process exit, identity validation and import all succeed. Failed replacements roll back and preserve the previous active generation.

### MT-306 — MAME identity provenance

Metadata generations record executable source/trust/path/version/build/raw version and listxml build identity. A changed executable identity returns `MetadataFreshness::Stale`; refresh is an explicit typed operation.

### MT-307 — generated vs user state

User favorites, collections, history, tags, machine launch preferences and controller state are deliberately not foreign-keyed to generated machine rows. Regeneration/pruning therefore cannot cascade-delete user state. Regression coverage replaces the catalog with a generation that removes the previous machine and verifies favorites, collection membership and recent history survive.

### MT-308 — indexed bounded machine search

Catalog queries search short name, description and manufacturer, filter manufacturer/year/driver status/parent-clone state, and enforce a maximum page size of 200. Corresponding indexes exist in schema v1 and regression tests exercise filtering and page bounds.

### MT-309 — regression suite

Tests cover malformed XML, unknown elements, Unicode, duplicate machine/import failure, clone/device relationships, transaction rollback and MAME identity changes.

**Reconciliation result:** MT-302 through MT-309 are complete. MT-301 is partially complete with the two provenance requirements still open.

## MT-401 — primary library browser

`LibraryBrowser.tsx` implements search, a bounded paginated machine list, loading state, empty state and retryable error state. Server-side catalog queries cap each page at 200 rows, providing a bounded DOM workload without requiring virtualization. MT-410 separately qualified full-catalog latency and bounded rendering behavior.

**Reconciliation result:** all MT-401 checklist items are satisfied.

## CI basis

The current executable baseline inherited by this audit includes the MT-711 qualified Tauri run `34644544775`, which passed frontend formatting/lint/typecheck/tests/build, Rust formatting/tests, performance qualification, Clippy and lockfile verification on exact SHA `d63bfb7662ae7dbbfe69ca53119700e050d38b75`. MT-711 qualification documentation subsequently passed on `bc417f32e56a633f7a52150ac1b05eed93216f7a`.
