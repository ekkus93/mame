# MAME Tauri Implementation TODO

**Document date:** 2026-09-08  
**Repository:** `ekkus93/mame`  
**Architecture spec:** `docs/MAME_TAURI_ARCHITECTURE_SPEC_2026-09-08.md`  
**Original planning baseline:** `7cc3033a50b00801240f020b7098e22ffffdcd44`  
**Status:** Active implementation backlog — MT-000 and MT-100 closed

---

## 1. Purpose

This document converts the MAME Tauri architecture specification into an ordered, auditable engineering backlog.

The backlog is designed for incremental/Ralph-loop execution. Each task should be completed with explicit evidence, tests, documentation, and exact-head validation appropriate to its scope.

The central sequencing rule is:

> **Deliver a useful Tauri frontend around a supervised native MAME process before attempting embedded rendering, a dedicated Tauri OSD, or in-process MAME hosting.**

The project must not block useful frontend functionality on speculative deep-integration work.

---

## 2. Global completion rules

Unless a task explicitly states otherwise, completion requires:

- implementation committed to a feature branch or approved integration branch;
- relevant automated tests;
- formatting/lint/type checks passing;
- no known silent-failure path introduced;
- user-visible failures mapped to actionable errors where applicable;
- documentation updated when behavior/configuration changes;
- exact commit SHA recorded for significant qualification gates;
- no weakening of existing MAME validation without explicit justification.

A checkbox is not complete merely because code exists. Its acceptance criteria must be satisfied.

---

# MT-000 — Project foundations and upstream policy

## MT-001 — Freeze project architecture baseline

- [x] Review `MAME_TAURI_ARCHITECTURE_SPEC_2026-09-08.md` against current MAME master.
- [x] Record any architectural drift since planning baseline `7cc3033a50b00801240f020b7098e22ffffdcd44`.
- [x] Confirm sidecar-first approach.
- [x] Confirm native video/audio/input hot-path invariant.
- [x] Confirm embedded rendering and in-process hosting remain optional gated phases.

**Acceptance:** architecture document matches implementation intent and current repository reality.

## MT-002 — Define branch and merge policy

- [x] Document feature-branch naming convention.
- [x] Define exact-head CI requirement for executable changes.
- [x] Define upstream synchronization procedure.
- [x] Define when documentation-only direct commits are permissible, if at all.
- [x] Define branch cleanup policy.

**Acceptance:** one documented workflow exists for normal development and upstream sync.

## MT-003 — Protect project-owned namespaces

- [x] Select final Tauri frontend directory name.
- [x] Select final Rust/Tauri backend directory name.
- [x] Reserve any project-specific scripts/test fixtures namespaces.
- [x] Avoid naming collisions with upstream MAME directories.

**Decision gate DG-1:** final repository layout.

## MT-004 — Establish project coding standards

- [x] Rust formatting/lint policy.
- [x] TypeScript formatting/lint policy.
- [x] React conventions.
- [x] Error-type conventions.
- [x] Serialization conventions.
- [x] Test naming/placement conventions.
- [x] C/C++ rules for any future MAME-side additions: follow surrounding MAME style and minimize whitespace churn.

## MT-005 — Establish licensing/trademark constraints

- [x] Inventory MAME license requirements relevant to redistribution.
- [x] Document requirements for redistributed MAME binaries/resources.
- [x] Document Tauri/Rust/JS dependency license inventory requirements.
- [x] Document MAME trademark/name/logo constraints.
- [x] Confirm no ROM/software content will be bundled.

---

# MT-100 — Tauri application scaffold

## MT-101 — Create Tauri 2 scaffold

- [x] Add Tauri 2 application project.
- [x] Verify development launch.
- [x] Verify production build on primary development platform.
- [x] Keep scaffold changes isolated from upstream MAME source.

**Acceptance:** a minimal desktop window launches successfully.

## MT-102 — Add React + TypeScript frontend

- [x] Configure React.
- [x] Configure TypeScript strict mode.
- [x] Add routing/layout foundation if needed.
- [x] Add baseline application shell.
- [x] Add error boundary.

## MT-103 — Establish Rust backend module layout

Suggested initial modules:

```text
app
config
errors
mame
metadata
library
storage
sessions
platform
```

- [x] Define responsibilities.
- [x] Prevent circular/implicit cross-layer dependencies.

## MT-104 — Establish typed Tauri command layer

- [x] Add one typed request/response command.
- [x] Define shared serialization conventions.
- [x] Define stable error envelope.
- [x] Add command tests.

**Acceptance:** frontend invokes Rust and receives a typed structured response.

## MT-105 — Establish backend-to-frontend event layer

- [x] Define event naming convention.
- [x] Define event payload version policy.
- [x] Add test/demo lifecycle event.
- [x] Ensure event payloads are bounded.

## MT-106 — Add application configuration root

- [x] Resolve platform-appropriate config path.
- [x] Add versioned settings schema.
- [x] Add safe defaults.
- [x] Add migration framework.
- [x] Add corrupt-config handling.

## MT-107 — Add frontend state architecture

- [x] Separate backend source-of-truth state from transient UI state.
- [x] Avoid duplicating authoritative session state in multiple stores.
- [x] Define loading/error states.

## MT-108 — Add baseline frontend quality gates

- [x] Typecheck.
- [x] Lint.
- [x] Unit/component test command.
- [x] Production frontend build.

## MT-109 — Add baseline Rust quality gates

- [x] `cargo fmt --check`.
- [x] `cargo clippy` with agreed warning policy.
- [x] `cargo test`.
- [x] Release build/smoke compile.

---

# MT-200 — MAME executable and sidecar integration

## MT-201 — Define executable-source model

Support concepts:

- bundled/tested executable;
- explicitly configured external executable;
- development-tree executable.

- [ ] Model executable identity.
- [ ] Record version/build information.
- [ ] Distinguish trusted bundled executable from arbitrary external executable.

**Decision gate DG-2:** default release executable policy.

## MT-202 — Implement MAME executable discovery/configuration

- [ ] Configure explicit MAME executable path.
- [ ] Validate that the path is executable/usable.
- [ ] Obtain version/build identity.
- [ ] Return structured errors for missing/invalid executable.

## MT-203 — Implement safe argument construction

- [ ] Represent arguments as an argv vector.
- [ ] No shell-string concatenation.
- [ ] Validate machine/software identifiers.
- [ ] Validate project-controlled paths.
- [ ] Unit-test spaces, Unicode, metacharacters, and malformed values.

## MT-204 — Implement supervised process launch

- [ ] Spawn MAME.
- [ ] Create unique session ID.
- [ ] Record PID/process handle.
- [ ] Capture launch timestamp.
- [ ] Record effective argv/config context.
- [ ] Return launch result.

## MT-205 — Capture stdout/stderr safely

- [ ] Stream/capture stdout.
- [ ] Stream/capture stderr.
- [ ] Prevent unbounded in-memory accumulation.
- [ ] Preserve recent diagnostic context.
- [ ] Associate output with session ID.

## MT-206 — Implement session lifecycle state machine

Suggested states:

```text
created
starting
running
stopping
exited
failed
crashed
```

- [ ] Define legal transitions.
- [ ] Reject impossible transitions.
- [ ] Test normal and abnormal paths.

## MT-207 — Implement clean stop

- [ ] Define preferred graceful exit mechanism.
- [ ] Add bounded shutdown timeout.
- [ ] Add explicit escalation policy if graceful shutdown fails.
- [ ] Surface forced termination.

## MT-208 — Detect child crash/abnormal exit

- [ ] Distinguish normal exit from abnormal exit.
- [ ] Emit structured crash/exit event.
- [ ] Preserve diagnostics.
- [ ] Reset application session state.

## MT-209 — Handle duplicate/concurrent sessions

- [ ] Decide whether multiple MAME sessions are supported initially.
- [ ] If no, reject second launch clearly.
- [ ] If yes later, isolate session state and resources.

## MT-210 — Sidecar process integration tests

- [ ] Successful launch fixture/known machine.
- [ ] Missing executable.
- [ ] Invalid machine.
- [ ] Clean exit.
- [ ] forced/abnormal exit.
- [ ] stdout/stderr capture.
- [ ] argument quoting/path cases.

**Milestone:** supervised MAME process exists behind typed Rust commands.

---

# MT-300 — MAME metadata subsystem

## MT-301 — Capture representative `-listxml` fixture

- [ ] Generate fixture from pinned/known MAME build.
- [ ] Record producing MAME identity.
- [ ] Keep fixture size appropriate for tests.
- [ ] Include representative clones/devices/displays/software-list relations.

## MT-302 — Implement streaming `-listxml` parser

- [ ] Avoid requiring the entire XML document in memory if unnecessary.
- [ ] Parse machine short name and description.
- [ ] Parse year/manufacturer.
- [ ] Parse clone/parent relationships.
- [ ] Parse source file.
- [ ] Parse status metadata.
- [ ] Parse display/device metadata required by UI.
- [ ] Parse software-list associations.
- [ ] Preserve forward compatibility with unknown elements.

## MT-303 — Define SQLite schema

Minimum domains:

```text
metadata_generation
machines
machine_alias/relationships as needed
displays
devices
software_lists
machine_software_list_relations
user_favorites
user_collections
recent_history
user_tags
```

- [ ] Normalize only where query/maintenance value justifies it.
- [ ] Add indexes for common search/filter paths.

## MT-304 — Add schema migration framework

- [ ] Schema version table.
- [ ] Forward migrations.
- [ ] Migration tests.
- [ ] Failed migration recovery behavior.

## MT-305 — Implement metadata generation/import transaction

- [ ] Run MAME metadata command outside UI thread.
- [ ] Import into staging/transaction.
- [ ] Do not expose partially imported catalog.
- [ ] Atomically activate successful generation.
- [ ] Preserve prior usable generation on failure.

## MT-306 — Tie metadata generation to MAME identity

- [ ] Record executable identity/version.
- [ ] Detect changed MAME build.
- [ ] Mark metadata stale.
- [ ] Refresh explicitly or according to documented policy.

## MT-307 — Separate generated metadata from user state

- [ ] Regeneration does not delete favorites.
- [ ] Regeneration does not delete collections.
- [ ] Regeneration does not delete play history.
- [ ] Handle removed/renamed machines gracefully.

## MT-308 — Implement indexed machine search

- [ ] Search short name.
- [ ] Search description.
- [ ] Search manufacturer.
- [ ] Filter by year.
- [ ] Filter by status.
- [ ] Parent/clone handling.
- [ ] Bounded page size.

## MT-309 — Metadata parser/database regression suite

- [ ] malformed XML.
- [ ] unexpected element.
- [ ] Unicode.
- [ ] duplicate/edge relationship cases.
- [ ] transaction rollback.
- [ ] MAME identity change.

**Milestone:** frontend can query a durable indexed MAME catalog.

---

# MT-400 — Library UX

## MT-401 — Build primary library browser

- [ ] Search box.
- [ ] machine list/grid.
- [ ] loading state.
- [ ] empty state.
- [ ] error state.
- [ ] bounded rendering/virtualization.

## MT-402 — Build machine detail view

Display at minimum:

- [x] description.
- [x] short name.
- [x] year.
- [x] manufacturer.
- [x] parent/clone relation.
- [x] working/imperfect status.
- [x] display information.
- [x] source/driver information where useful.
- [x] launch action.

## MT-403 — Add filtering/sorting

- [x] manufacturer.
- [x] year.
- [x] status.
- [x] parent/clone policy.
- [x] available/audited state when MT-500 exists.

## MT-404 — Add favorites

- [x] favorite/unfavorite.
- [x] favorites view.
- [x] persistence across metadata refresh.

## MT-405 — Add collections

- [x] create collection.
- [x] rename collection.
- [x] add/remove machine.
- [x] delete collection.
- [x] safe behavior when machine disappears from newer metadata.

## MT-406 — Add recents/play history

- [x] launch timestamp.
- [x] machine/software reference.
- [x] success/failure distinction where useful.
- [x] recents view.
- [x] bounded history retention policy.

## MT-407 — Add keyboard navigation

- [x] predictable focus order.
- [x] launch/search shortcuts.
- [x] no conflict with gameplay window ownership.

## MT-408 — Add controller-friendly frontend navigation research

- [x] determine desired UI gamepad behavior.
- [x] prototype without stealing gameplay input from MAME.
- [x] document focus transitions.

## MT-409 — Add software-list browser foundation

- [x] expose machine-associated software lists.
- [x] browse known software metadata.
- [x] support selecting machine + software launch target.

## MT-410 — Library UX performance qualification

- [x] measure startup catalog load.
- [x] measure search latency.
- [x] test full catalog.
- [x] confirm bounded DOM/render workload.

**Milestone:** application is a useful machine browser independent of embedded rendering.

---

# MT-500 — ROM/software paths and auditing

## MT-501 — Define path configuration model

- [x] ROM paths.
- [x] software paths.
- [x] CHD-related paths where applicable.
- [x] platform-safe path representation.
- [x] validation and permissions errors.

## MT-502 — Expose path configuration UI

- [x] choose path through native dialog.
- [x] add/remove/reorder paths as supported.
- [x] display invalid/unavailable paths.

## MT-503 — Determine authoritative MAME audit commands

- [x] document selected MAME command(s).
- [x] capture representative outputs.
- [x] document exit-code semantics.
- [x] avoid inventing availability from file presence alone.

## MT-504 — Implement audit parser

Represent at least:

- [x] runnable/complete where MAME confirms it.
- [x] missing required content.
- [x] incorrect content.
- [x] optional content distinction where exposed.
- [x] unknown/error.

## MT-505 — Persist audit results with provenance

- [x] tie result to MAME identity.
- [x] tie result to configured content paths.
- [x] invalidate stale audit results.

## MT-506 — Add per-machine audit action

- [x] run audit.
- [x] show progress.
- [x] show structured result.
- [x] show raw diagnostic excerpt when useful.

## MT-507 — Add bounded bulk audit

- [x] cancellation.
- [x] progress.
- [x] bounded parallelism.
- [x] resumable or safely restartable behavior if expensive.

## MT-508 — Integrate availability into library

- [x] filter available/missing/unknown.
- [x] status badge.
- [x] do not label unverified content as valid.

---

# MT-600 — Configuration subsystem

## MT-601 — Inventory relevant MAME configuration sources

- [x] command-line options.
- [x] `mame.ini`/platform equivalents.
- [x] machine-specific INI behavior.
- [x] controller mappings.
- [x] renderer/audio options relevant to frontend.

## MT-602 — Define deterministic configuration precedence

Document:

```text
application defaults
MAME/global defaults
profile defaults
machine overrides
transient launch overrides
```

- [x] precise precedence.
- [x] conflict behavior.
- [x] effective-value calculation tests.

## MT-603 — Implement non-destructive config persistence

- [x] avoid overwriting unrelated user settings.
- [x] backup/recovery policy where files are edited.
- [x] atomic writes.
- [x] parse/write round-trip tests.

## MT-604 — Add general settings UI

- [x] executable.
- [x] content paths.
- [x] window/fullscreen preference.
- [x] renderer selection where supported.
- [x] audio preference where supported.

## MT-605 — Add per-machine settings

- [x] override storage.
- [x] reset to inherited/default.
- [x] effective value display.

## MT-606 — Add configuration explainability

- [x] show effective value.
- [x] show source layer.
- [x] show pending launch override.

## MT-607 — Add controller profile data model

- [x] profile identity.
- [x] target device/controller identity.
- [x] machine/global association.
- [x] mapping provenance.

## MT-608 — Controller configuration UI

- [x] inspect active profile.
- [x] assign/select profile.
- [x] avoid pretending unsupported mappings are active.

---

# MT-700 — Runtime control protocol

## MT-701 — Characterize MAME control options

- [x] inventory existing mechanisms suitable for external control.
- [x] identify minimal MAME-side changes if required.
- [x] compare local socket/pipe/other IPC options.
- [x] document platform implications.

**Decision gate DG-3:** runtime-control transport.

## MT-702 — Define protocol v1

- [ ] version field.
- [ ] request ID.
- [ ] command enum.
- [ ] structured result/error.
- [ ] bounded message size.
- [ ] connection lifecycle.
- [ ] timeout semantics.
- [ ] malformed-message behavior.

## MT-703 — Implement local authenticated/scoped endpoint model

- [ ] local-only by default.
- [ ] unpredictable/session-scoped endpoint or equivalent protection.
- [ ] no ambient LAN listener.
- [ ] teardown with session.

## MT-704 — Implement pause/resume

- [ ] command.
- [ ] acknowledgement/completion semantics.
- [ ] state event.
- [ ] disconnect/error behavior.

## MT-705 — Implement reset

- [ ] supported reset semantics documented.
- [ ] command/result.

## MT-706 — Implement clean exit command

- [ ] prefer protocol exit before process kill.
- [ ] integrate with MT-207 escalation path.

## MT-707 — Implement save state

- [ ] command.
- [ ] slot/path semantics.
- [ ] success/failure event.
- [ ] associate state with session/machine context.

## MT-708 — Implement load state

- [ ] command.
- [ ] incompatible/missing state error handling.
- [ ] explicit completion result.

## MT-709 — Implement mute/volume controls where native semantics allow

- [ ] define supported operations.
- [ ] do not route PCM through Tauri.

## MT-710 — Implement query-state/status

- [ ] running.
- [ ] paused.
- [ ] machine identity.
- [ ] other safe bounded status required by UI.

## MT-711 — Protocol adversarial tests

- [ ] malformed payload.
- [ ] oversized payload.
- [ ] unknown command.
- [ ] wrong version.
- [ ] stale session.
- [ ] connection drop mid-command.
- [ ] timeout.

**Milestone:** modern frontend can control a running MAME session without being in the real-time data path.

---

# MT-800 — Artwork and presentation

## MT-801 — Define artwork data model

- [ ] screenshot.
- [ ] cabinet.
- [ ] marquee.
- [ ] flyer.
- [ ] icon.
- [ ] system image.
- [ ] provenance/source.

## MT-802 — Implement local artwork discovery

- [ ] configurable roots.
- [ ] safe filename/path mapping.
- [ ] missing-art fallback.
- [ ] cache thumbnails where useful.

## MT-803 — Add artwork to machine detail/library views

- [ ] lazy loading.
- [ ] bounded cache.
- [ ] no UI stalls on large artwork sets.

## MT-804 — Define external artwork-provider policy

- [ ] licensing.
- [ ] attribution.
- [ ] privacy.
- [ ] caching.
- [ ] offline behavior.
- [ ] rate limiting.

**Decision gate DG-7:** network-backed artwork providers.

---

# MT-900 — Save-state UX and session management

## MT-901 — Define save-state record model

- [ ] machine.
- [ ] software item if applicable.
- [ ] MAME version/build identity.
- [ ] timestamp.
- [ ] slot/path.
- [ ] optional screenshot metadata if added later.

## MT-902 — Build save-state browser

- [ ] list known states.
- [ ] save.
- [ ] load.
- [ ] delete with confirmation.
- [ ] stale/incompatible warning.

## MT-903 — Handle incompatible state explicitly

- [ ] never imply cross-version compatibility without validation.
- [ ] preserve failed state file unless user deletes it.

---

# MT-1000 — Native-window integration research

> This phase is **not** a prerequisite for a useful product.

## MT-1001 — Document native handle capabilities per platform

- [ ] Windows native window handle path.
- [ ] macOS native view/layer path.
- [ ] Linux X11 path.
- [ ] Linux Wayland path.
- [ ] Tauri/TAO/wry thread/lifetime constraints.

## MT-1002 — Prototype native child/sibling render surface

- [ ] create native surface associated with Tauri window.
- [ ] resize it from Rust/native code.
- [ ] coexist with WebView.
- [ ] prove no framebuffer IPC through JS.

## MT-1003 — Prototype BGFX targeting strategy

- [ ] determine whether MAME BGFX integration can target chosen native surface.
- [ ] identify required MAME changes.
- [ ] measure initialization/shutdown behavior.

## MT-1004 — Prototype alternate native renderer if BGFX unsuitable

- [ ] document reason BGFX is insufficient.
- [ ] implement smallest comparison prototype.

## MT-1005 — Validate resize/HiDPI

- [ ] continuous resize.
- [ ] DPI changes.
- [ ] aspect ratio.
- [ ] scaling.

## MT-1006 — Validate fullscreen

- [ ] enter/exit fullscreen.
- [ ] restore window state.
- [ ] monitor selection.

## MT-1007 — Validate multi-monitor

- [ ] move between displays.
- [ ] different DPI/scales.
- [ ] renderer recreation where required.

## MT-1008 — Validate focus and input ownership

- [ ] WebView controls.
- [ ] gameplay surface.
- [ ] keyboard.
- [ ] mouse.
- [ ] gamepad.
- [ ] Escape/menu policy.

## MT-1009 — Benchmark external vs embedded rendering

Measure:

- [ ] average frame time.
- [ ] frame-time variance.
- [ ] CPU overhead.
- [ ] GPU overhead.
- [ ] dropped/stalled frames.
- [ ] input latency where practical.
- [ ] audio/video synchronization.

## MT-1010 — Embedded-render go/no-go report

Document:

- [ ] per-platform feasibility.
- [ ] required MAME patch surface.
- [ ] known limitations.
- [ ] performance comparison.
- [ ] maintenance cost.
- [ ] recommendation.

**Decision gate DG-4:** embedded renderer strategy.

---

# MT-1100 — Dedicated Tauri MAME OSD

> Begin only after MT-1010 recommends proceeding.

## MT-1101 — Define Tauri OSD scope

- [ ] exact responsibilities.
- [ ] reusable existing modules.
- [ ] platform-specific glue.
- [ ] files added/modified in upstream tree.

## MT-1102 — Add minimal `tauri_osd_interface`

- [ ] compile.
- [ ] initialize.
- [ ] process events.
- [ ] report focus.
- [ ] clean shutdown.

## MT-1103 — Implement window lifecycle

- [ ] create/attach required native surface.
- [ ] resize.
- [ ] destroy.
- [ ] multi-window policy documented.

## MT-1104 — Integrate selected render module

- [ ] render representative raster machine.
- [ ] render representative vector machine if relevant.
- [ ] shader/effect compatibility characterization.

## MT-1105 — Integrate input natively

- [ ] keyboard.
- [ ] mouse.
- [ ] gamepad.
- [ ] lightgun behavior documented if unsupported/partial.
- [ ] no JS gameplay-input dependency.

## MT-1106 — Integrate native audio

- [ ] preserve MAME audio path.
- [ ] validate latency/synchronization.

## MT-1107 — OSD validation subset

- [ ] representative arcade machine.
- [ ] representative computer/console if project scope includes them.
- [ ] pause/reset/state operations.
- [ ] fullscreen.
- [ ] clean exit.

## MT-1108 — OSD upstream-maintenance audit

- [ ] enumerate changes outside `src/osd/tauri`.
- [ ] reduce unnecessary core patches.
- [ ] rehearse upstream merge/rebase.

**Decision gate DG-5:** retain dedicated OSD only if benefits exceed maintenance cost.

---

# MT-1200 — Optional in-process MAME/library integration

> This phase is optional and must not begin merely because it is technically interesting.

## MT-1201 — Produce in-process justification document

Must identify specific requirements not adequately met by sidecar/OSD architecture.

- [ ] measurable benefit.
- [ ] expected maintenance cost.
- [ ] crash-isolation tradeoff.
- [ ] upstream impact.

**Decision gate DG-6:** approve or reject in-process work.

## MT-1202 — Characterize MAME library-build feasibility

- [ ] build-system implications.
- [ ] symbol/export model.
- [ ] initialization assumptions.
- [ ] global/static state.
- [ ] event/thread assumptions.

## MT-1203 — Define narrow C-compatible façade

Conceptual operations:

- [ ] create/destroy.
- [ ] configure.
- [ ] start.
- [ ] pause/resume.
- [ ] reset.
- [ ] save/load.
- [ ] query state.
- [ ] request shutdown.

## MT-1204 — Define ownership/thread model

- [ ] caller thread.
- [ ] emulator thread.
- [ ] callbacks.
- [ ] renderer thread requirements.
- [ ] shutdown ordering.

## MT-1205 — Contain C++ exceptions and Rust panics

- [ ] no exception crosses C ABI.
- [ ] no Rust panic unwinds into C++.
- [ ] explicit fatal error propagation.

## MT-1206 — Version ABI

- [ ] ABI version.
- [ ] feature detection.
- [ ] mismatch behavior.

## MT-1207 — In-process prototype

- [ ] launch one tiny/representative machine.
- [ ] clean shutdown.
- [ ] repeated create/destroy cycle if intended.
- [ ] compare with sidecar performance/stability.

## MT-1208 — Final in-process adoption decision

- [ ] retain sidecar.
- [ ] hybrid architecture.
- [ ] adopt in-process.
- [ ] document rationale.

---

# MT-1300 — Packaging and distribution

## MT-1301 — Define developer executable modes

- [ ] external MAME path.
- [ ] in-tree development build.
- [ ] bundled sidecar build where supported.

## MT-1302 — Bundle required MAME resources correctly

- [ ] identify runtime files.
- [ ] preserve licenses.
- [ ] verify paths after installation.

## MT-1303 — Windows packaging

- [ ] installer/package choice.
- [ ] sidecar path validation.
- [ ] clean install smoke test.
- [ ] clean uninstall behavior.

## MT-1304 — macOS packaging

- [ ] app bundle layout.
- [ ] sidecar/runtime resources.
- [ ] signing.
- [ ] notarization process.
- [ ] clean-machine smoke test.

## MT-1305 — Linux packaging

- [ ] choose initial package formats.
- [ ] runtime dependency strategy.
- [ ] X11/Wayland qualification notes.

## MT-1306 — Version reporting

UI/diagnostics should report:

- [ ] application version.
- [ ] Git/build identity where appropriate.
- [ ] MAME version/build identity.
- [ ] database schema version.
- [ ] runtime protocol version.

## MT-1307 — License inventory artifact

- [ ] MAME licenses.
- [ ] Rust dependencies.
- [ ] JavaScript dependencies.
- [ ] redistributed native libraries.

---

# MT-1400 — CI and automated validation

## MT-1401 — Add project-specific CI workflow

- [ ] Rust fmt.
- [ ] Rust clippy.
- [ ] Rust tests.
- [ ] TypeScript typecheck.
- [ ] frontend lint.
- [ ] frontend tests.
- [ ] frontend production build.

## MT-1402 — Add metadata fixture tests to CI

- [ ] XML parse.
- [ ] import.
- [ ] migration.
- [ ] search.

## MT-1403 — Add sidecar/process tests to CI where feasible

- [ ] known executable fixture or tiny MAME build.
- [ ] launch/exit.
- [ ] invalid launch.
- [ ] diagnostic capture.

## MT-1404 — Integrate `SUBTARGET=tiny` validation

- [ ] determine fastest representative native MAME build.
- [ ] avoid redundant full builds on every frontend-only change.
- [ ] preserve full upstream CI where required.

## MT-1405 — Add path-based CI optimization

- [ ] frontend-only changes avoid unnecessary full MAME compile when safe.
- [ ] MAME-core/OSD changes trigger relevant native builds.
- [ ] workflow changes trigger all appropriate validation.

## MT-1406 — Cross-platform CI matrix

- [ ] Linux.
- [ ] Windows.
- [ ] macOS.
- [ ] document which tests are compile-only vs executable.

## MT-1407 — Exact-head CI evidence helper

- [ ] easy retrieval/report of tested SHA.
- [ ] release checklist records exact green SHA.

---

# MT-1500 — Security hardening

## MT-1501 — Audit Tauri capabilities/permissions

- [ ] remove unused capabilities.
- [ ] deny generic shell access.
- [ ] narrow filesystem access.
- [ ] narrow process execution to intended MAME paths/model.

## MT-1502 — Path traversal/adversarial filesystem tests

- [ ] `..` traversal.
- [ ] symlink edge cases where applicable.
- [ ] Unicode/path normalization.
- [ ] unauthorized path request.

## MT-1503 — Process argument injection tests

- [ ] shell metacharacters.
- [ ] embedded quotes.
- [ ] spaces.
- [ ] malicious machine/software identifier.

## MT-1504 — IPC/control-protocol security audit

- [ ] local exposure only.
- [ ] stale-session hijack prevention.
- [ ] malformed payload.
- [ ] oversized payload.
- [ ] unauthorized peer assumptions documented.

## MT-1505 — Remote-content policy

- [ ] no privileged arbitrary remote page loading.
- [ ] artwork/network fetches isolated from privileged WebView navigation.
- [ ] CSP/security headers/config reviewed.

## MT-1506 — Dependency/supply-chain policy

- [ ] lockfiles committed.
- [ ] dependency update process.
- [ ] vulnerable dependency reporting policy.
- [ ] release provenance expectations.

---

# MT-1600 — Failure-mode and observability hardening

## MT-1601 — Define error taxonomy

- [ ] stable error codes.
- [ ] human-readable message.
- [ ] diagnostic context.
- [ ] frontend mapping.

## MT-1602 — Add structured application logs

- [ ] timestamps.
- [ ] component.
- [ ] session ID.
- [ ] severity.
- [ ] bounded retention.

## MT-1603 — Add diagnostics view/export

- [ ] application version.
- [ ] MAME identity.
- [ ] relevant paths with privacy-sensitive redaction policy.
- [ ] recent errors.
- [ ] session diagnostics.

## MT-1604 — Crash recovery

- [ ] stale session detection after app restart.
- [ ] orphaned MAME process policy.
- [ ] corrupt/incomplete metadata import recovery.
- [ ] interrupted config write recovery.

## MT-1605 — Silent-failure audit

Audit every subsystem for:

- [ ] swallowed errors.
- [ ] fallback without UI indication.
- [ ] ambiguous `None`/empty results.
- [ ] lost child-process failures.
- [ ] stale status presented as current.

---

# MT-1700 — Performance qualification

## MT-1701 — Define performance benchmark corpus

- [ ] full catalog search benchmark.
- [ ] representative launch benchmark.
- [ ] representative raster machine.
- [ ] representative demanding machine where useful.

## MT-1702 — Frontend performance baseline

Measure:

- [ ] cold app startup.
- [ ] library ready time.
- [ ] search latency.
- [ ] large-list responsiveness.
- [ ] artwork loading behavior.

## MT-1703 — Sidecar overhead baseline

Compare direct MAME vs Tauri-launched equivalent configuration:

- [ ] CPU usage.
- [ ] memory usage.
- [ ] frame behavior.
- [ ] launch latency.

## MT-1704 — Metadata generation performance

- [ ] generation time.
- [ ] peak memory.
- [ ] DB import time.
- [ ] incremental/stale refresh behavior.

## MT-1705 — Embedded-render qualification

Only if MT-1000 proceeds:

- [ ] frame pacing comparison.
- [ ] input latency comparison.
- [ ] CPU/GPU overhead comparison.
- [ ] audio/video sync.

---

# MT-1800 — Cross-platform qualification

## MT-1801 — Linux qualification

- [ ] development build.
- [ ] packaged build.
- [ ] external MAME executable.
- [ ] bundled sidecar if supported.
- [ ] controller input.
- [ ] audio.
- [ ] fullscreen.
- [ ] X11.
- [ ] Wayland where supported/claimed.

## MT-1802 — Windows qualification

- [ ] development build.
- [ ] packaged build.
- [ ] controller input.
- [ ] audio.
- [ ] fullscreen.
- [ ] renderer behavior.

## MT-1803 — macOS qualification

- [ ] development build.
- [ ] signed packaged build.
- [ ] controller input.
- [ ] audio.
- [ ] fullscreen.
- [ ] notarization/install behavior.

## MT-1804 — Cross-platform configuration portability audit

- [ ] path differences.
- [ ] settings migration.
- [ ] controller identity differences.
- [ ] artwork paths.

---

# MT-1900 — Upstream sustainability

## MT-1901 — Add upstream remote/sync documentation

- [ ] fetch upstream.
- [ ] compare divergence.
- [ ] merge/rebase policy.
- [ ] conflict handling.
- [ ] test requirements after sync.

## MT-1902 — Track project-owned MAME patches

- [ ] generate/list patches outside project-only directories.
- [ ] explain each patch.
- [ ] identify patches that can be upstreamed or eliminated.

## MT-1903 — Perform first upstream-sync rehearsal

- [ ] start from known project head.
- [ ] integrate newer upstream master.
- [ ] resolve conflicts.
- [ ] run project CI.
- [ ] record effort/problems.

## MT-1904 — Establish recurring upstream-sync cadence

- [ ] choose cadence based on divergence and activity.
- [ ] avoid multi-month drift where practical.

---

# MT-2000 — Product completion before embedded rendering

This milestone proves the project has value even if native embedding is never adopted.

## MT-2001 — Modern frontend completeness gate

Required:

- [ ] machine browser.
- [ ] machine detail.
- [ ] fast search/filter.
- [ ] favorites.
- [ ] collections.
- [ ] recents.
- [ ] local artwork support.
- [ ] machine/software launch.
- [ ] executable/content path configuration.
- [ ] audit state.
- [ ] per-machine settings foundation.
- [ ] supervised MAME lifecycle.
- [ ] actionable errors.

## MT-2002 — Runtime management completeness gate

Where supported by MT-700:

- [ ] pause/resume.
- [ ] reset.
- [ ] clean exit.
- [ ] save/load state.
- [ ] runtime status.

## MT-2003 — UX acceptance pass

- [ ] keyboard-first workflow.
- [ ] controller-navigation expectations documented/tested.
- [ ] no blocking UI operations.
- [ ] errors understandable.
- [ ] common settings discoverable.

## MT-2004 — External-window release candidate

- [ ] package on supported platforms.
- [ ] complete smoke test.
- [ ] document known limitations.
- [ ] record exact qualified commit.

**Major milestone:** production-useful Tauri MAME frontend exists without embedded rendering.

---

# MT-2100 — Final quality closure

## MT-2101 — Cross-cutting unsafe-fallback audit

- [ ] executable fallback.
- [ ] renderer fallback.
- [ ] config fallback.
- [ ] audit fallback.
- [ ] metadata fallback.
- [ ] protocol fallback.
- [ ] artwork fallback.

Every fallback must be intentional, documented, and surfaced when materially relevant.

## MT-2102 — Cross-cutting silent-failure audit

- [ ] Rust ignored results.
- [ ] frontend rejected promises/events.
- [ ] child process failures.
- [ ] database errors.
- [ ] file writes.
- [ ] migration errors.
- [ ] protocol disconnects.

## MT-2103 — Security closure

- [ ] Tauri capability audit complete.
- [ ] IPC attack tests complete.
- [ ] filesystem attack tests complete.
- [ ] process argument attack tests complete.
- [ ] dependency scan reviewed.

## MT-2104 — Performance closure

- [ ] direct-vs-Tauri overhead acceptable.
- [ ] library performance acceptable.
- [ ] embedded renderer, if present, meets documented thresholds.

## MT-2105 — Cross-platform closure

- [ ] Linux acceptance.
- [ ] Windows acceptance.
- [ ] macOS acceptance.
- [ ] packaging acceptance.

## MT-2106 — Documentation closure

- [ ] install instructions.
- [ ] developer build instructions.
- [ ] MAME executable/content setup.
- [ ] troubleshooting.
- [ ] architecture overview.
- [ ] upstream-sync procedure.
- [ ] release procedure.

## MT-2107 — Final exact-head CI

- [ ] all required project CI green on exact candidate SHA.
- [ ] relevant upstream MAME CI green.
- [ ] no uncommitted closure changes.
- [ ] exact SHA recorded in release/closure document.

---

# MT-2200 — Engineering-phase closure

## MT-2201 — Reconcile all TODO state

For every task:

- [ ] complete with evidence;
- [ ] explicitly deferred with rationale; or
- [ ] explicitly rejected/superseded with rationale.

No ambiguous half-complete checkbox remains.

## MT-2202 — Preserve unresolved research status accurately

If embedded rendering, dedicated OSD, or in-process hosting is not complete:

- [ ] do not imply it is complete.
- [ ] preserve experiment evidence.
- [ ] document blockers/decision.
- [ ] separate useful-product completion from research completion.

## MT-2203 — Final upstream divergence audit

- [ ] list fork-specific commits/files.
- [ ] identify avoidable upstream modifications.
- [ ] confirm future sync remains tractable.

## MT-2204 — Create handoff/maintenance document

Include:

- [ ] architecture summary.
- [ ] qualified versions.
- [ ] known limitations.
- [ ] CI/release workflow.
- [ ] upstream sync procedure.
- [ ] remaining optional research.

---

# 3. Recommended execution order

The default sequence is:

```text
MT-000   Foundations
   ↓
MT-100   Tauri scaffold
   ↓
MT-200   MAME sidecar/process supervision
   ↓
MT-300   Metadata/index
   ↓
MT-400   Library UX
   ↓
MT-500   ROM/software audit
   ↓
MT-600   Configuration
   ↓
MT-700   Runtime control
   ↓
MT-800/900 Artwork + save-state UX
   ↓
MT-1300/1400 Packaging + CI maturation
   ↓
MT-1500/1600 Security + failure hardening
   ↓
MT-1700/1800 Performance + cross-platform qualification
   ↓
MT-2000  Useful external-window release milestone
   ↓
         ┌──────────────── OPTIONAL ────────────────┐
         │ MT-1000 Native-window research          │
         │    ↓                                     │
         │ MT-1100 Dedicated Tauri OSD             │
         │    ↓                                     │
         │ MT-1200 Optional in-process integration │
         └──────────────────────────────────────────┘
   ↓
MT-1900  Upstream sustainability
   ↓
MT-2100  Final quality closure
   ↓
MT-2200  Engineering-phase closure
```

The optional block may begin experimentally earlier once MT-200/300 fundamentals exist, but it must not derail completion of the useful external-window frontend.

---

# 4. First vertical-slice milestone

The first implementation iteration should be intentionally narrow.

## VS-1 — Tauri → catalog → launch → stop

- [ ] Tauri window launches.
- [ ] React calls Rust through typed command.
- [ ] Rust uses configured MAME executable.
- [ ] Rust invokes a bounded metadata query/import.
- [ ] frontend displays at least a small searchable machine list.
- [ ] user selects a known machine.
- [ ] Rust launches MAME as supervised child.
- [ ] frontend receives session-started state.
- [ ] MAME runs in its normal external native window.
- [ ] user requests stop.
- [ ] child exits cleanly.
- [ ] frontend returns to idle state.
- [ ] failed launch shows structured error.
- [ ] no video frame crosses Tauri IPC.
- [ ] no PCM stream crosses Tauri IPC.

This vertical slice is the recommended first Ralph-loop target after the foundation/scaffold work.

---

# 5. Decision-register template

Each decision gate should eventually have a short durable record containing:

```text
Decision ID:
Date:
Exact repository SHA:
Question:
Options evaluated:
Evidence/prototypes:
Performance data:
Cross-platform data:
Upstream-maintenance impact:
Security impact:
Decision:
Rejected alternatives:
Revisit conditions:
```

This is particularly important for DG-3 through DG-6 because they control architectural coupling to MAME.

---

# 6. Definition of done for the overall project

The core project is considered complete when:

1. A modern Tauri/React application can browse and search the MAME catalog.
2. Metadata is derived from MAME and indexed reliably.
3. Users can configure legal local content paths and understand audit state.
4. Users can launch and supervise MAME sessions reliably.
5. Application settings and per-machine state are durable and non-destructive.
6. Runtime controls required by the selected product scope are reliable.
7. Video/audio/gameplay input remain on native low-latency paths.
8. Linux, Windows, and macOS release targets meet documented acceptance criteria.
9. Security, unsafe-fallback, and silent-failure audits are closed.
10. Project-specific MAME divergence is documented and maintainable.
11. Exact-head CI/release qualification is recorded.

Embedded rendering, a dedicated Tauri OSD, and in-process MAME hosting are **separate optional success tracks**. They must not be falsely treated as required for the core frontend project to be considered successful.