# MAME Tauri Implementation TODO

**Document date:** 2026-09-08  
**Repository:** `ekkus93/mame`  
**Architecture spec:** `docs/MAME_TAURI_ARCHITECTURE_SPEC_2026-09-08.md`  
**Original planning baseline:** `7cc3033a50b00801240f020b7098e22ffffdcd44`  
**Status:** Engineering-phase closure reconciled; optional MT-1000/1100/1200 and MT-1705 research deferred

---

## 1. Purpose

This document converts the MAME Tauri architecture specification into an ordered, auditable engineering backlog.

The backlog is designed for incremental/Ralph-loop execution. Each task should be completed with explicit evidence, tests, documentation, and exact-head validation appropriate to its scope.

The central sequencing rule is:

> **Deliver a useful Tauri frontend around a supervised native MAME process before attempting embedded rendering, a dedicated Tauri OSD, or in-process MAME hosting.**

## MT-2200 reconciliation status convention

As of the MT-2200 engineering-phase closure, this file is an authoritative status ledger rather than a historical unchecked planning list:

- `[x]` means the requirement is **closed** by implementation, qualification evidence, an explicit policy decision, or a documented runtime/platform boundary.
- `[ ] **Deferred — optional research:**` means the requirement is intentionally **not complete** and is outside the external-window product completion claim.
- No bare unchecked checkbox is permitted after MT-2201 reconciliation.

Cross-platform qualification items may be closed by automated CI, delegated native-runtime ownership, or a documented unsupported boundary as defined in `MAME_TAURI_MT1800_CROSS_PLATFORM_QUALIFICATION_2026-09-13.md`. A checked item therefore does not imply synthetic CI coverage where the qualification record explicitly says coverage is delegated or documented.

The optional native-window, dedicated-OSD, and in-process tracks remain open research even though the useful external-window frontend is engineering-complete. See `MAME_TAURI_MT2200_ENGINEERING_CLOSURE_2026-09-13.md`.

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

- [x] Model executable identity.
- [x] Record version/build information.
- [x] Distinguish trusted bundled executable from arbitrary external executable.

**Decision gate DG-2:** default release executable policy.

## MT-202 — Implement MAME executable discovery/configuration

- [x] Configure explicit MAME executable path.
- [x] Validate that the path is executable/usable.
- [x] Obtain version/build identity.
- [x] Return structured errors for missing/invalid executable.

## MT-203 — Implement safe argument construction

- [x] Represent arguments as an argv vector.
- [x] No shell-string concatenation.
- [x] Validate machine/software identifiers.
- [x] Validate project-controlled paths.
- [x] Unit-test spaces, Unicode, metacharacters, and malformed values.

## MT-204 — Implement supervised process launch

- [x] Spawn MAME.
- [x] Create unique session ID.
- [x] Record PID/process handle.
- [x] Capture launch timestamp.
- [x] Record effective argv/config context.
- [x] Return launch result.

## MT-205 — Capture stdout/stderr safely

- [x] Stream/capture stdout.
- [x] Stream/capture stderr.
- [x] Prevent unbounded in-memory accumulation.
- [x] Preserve recent diagnostic context.
- [x] Associate output with session ID.

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

- [x] Define legal transitions.
- [x] Reject impossible transitions.
- [x] Test normal and abnormal paths.

## MT-207 — Implement clean stop

- [x] Define preferred graceful exit mechanism.
- [x] Add bounded shutdown timeout.
- [x] Add explicit escalation policy if graceful shutdown fails.
- [x] Surface forced termination.

## MT-208 — Detect child crash/abnormal exit

- [x] Distinguish normal exit from abnormal exit.
- [x] Emit structured crash/exit event.
- [x] Preserve diagnostics.
- [x] Reset application session state.

## MT-209 — Handle duplicate/concurrent sessions

- [x] Decide whether multiple MAME sessions are supported initially.
- [x] If no, reject second launch clearly.
- [x] If yes later, isolate session state and resources.

## MT-210 — Sidecar process integration tests

- [x] Successful launch fixture/known machine.
- [x] Missing executable.
- [x] Invalid machine.
- [x] Clean exit.
- [x] forced/abnormal exit.
- [x] stdout/stderr capture.
- [x] argument quoting/path cases.

**Milestone:** supervised MAME process exists behind typed Rust commands.

---

# MT-300 — MAME metadata subsystem

## MT-301 — Capture representative `-listxml` fixture

- [x] Generate fixture from pinned/known MAME build.
- [x] Record producing MAME identity.
- [x] Keep fixture size appropriate for tests.
- [x] Include representative clones/devices/displays/software-list relations.

## MT-302 — Implement streaming `-listxml` parser

- [x] Avoid requiring the entire XML document in memory if unnecessary.
- [x] Parse machine short name and description.
- [x] Parse year/manufacturer.
- [x] Parse clone/parent relationships.
- [x] Parse source file.
- [x] Parse status metadata.
- [x] Parse display/device metadata required by UI.
- [x] Parse software-list associations.
- [x] Preserve forward compatibility with unknown elements.

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

- [x] Normalize only where query/maintenance value justifies it.
- [x] Add indexes for common search/filter paths.

## MT-304 — Add schema migration framework

- [x] Schema version table.
- [x] Forward migrations.
- [x] Migration tests.
- [x] Failed migration recovery behavior.

## MT-305 — Implement metadata generation/import transaction

- [x] Run MAME metadata command outside UI thread.
- [x] Import into staging/transaction.
- [x] Do not expose partially imported catalog.
- [x] Atomically activate successful generation.
- [x] Preserve prior usable generation on failure.

## MT-306 — Tie metadata generation to MAME identity

- [x] Record executable identity/version.
- [x] Detect changed MAME build.
- [x] Mark metadata stale.
- [x] Refresh explicitly or according to documented policy.

## MT-307 — Separate generated metadata from user state

- [x] Regeneration does not delete favorites.
- [x] Regeneration does not delete collections.
- [x] Regeneration does not delete play history.
- [x] Handle removed/renamed machines gracefully.

## MT-308 — Implement indexed machine search

- [x] Search short name.
- [x] Search description.
- [x] Search manufacturer.
- [x] Filter by year.
- [x] Filter by status.
- [x] Parent/clone handling.
- [x] Bounded page size.

## MT-309 — Metadata parser/database regression suite

- [x] malformed XML.
- [x] unexpected element.
- [x] Unicode.
- [x] duplicate/edge relationship cases.
- [x] transaction rollback.
- [x] MAME identity change.

**Milestone:** frontend can query a durable indexed MAME catalog.

---

# MT-400 — Library UX

## MT-401 — Build primary library browser

- [x] Search box.
- [x] machine list/grid.
- [x] loading state.
- [x] empty state.
- [x] error state.
- [x] bounded rendering/virtualization.

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

- [x] version field.
- [x] request ID.
- [x] command enum.
- [x] structured result/error.
- [x] bounded message size.
- [x] connection lifecycle.
- [x] timeout semantics.
- [x] malformed-message behavior.

## MT-703 — Implement local authenticated/scoped endpoint model

- [x] local-only by default.
- [x] unpredictable/session-scoped endpoint or equivalent protection.
- [x] no ambient LAN listener.
- [x] teardown with session.

## MT-704 — Implement pause/resume

- [x] command.
- [x] acknowledgement/completion semantics.
- [x] state event.
- [x] disconnect/error behavior.

## MT-705 — Implement reset

- [x] supported reset semantics documented.
- [x] command/result.

## MT-706 — Implement clean exit command

- [x] prefer protocol exit before process kill.
- [x] integrate with MT-207 escalation path.

## MT-707 — Implement save state

- [x] command.
- [x] slot/path semantics.
- [x] success/failure event.
- [x] associate state with session/machine context.

## MT-708 — Implement load state

- [x] command.
- [x] incompatible/missing state error handling.
- [x] explicit completion result.

## MT-709 — Implement mute/volume controls where native semantics allow

- [x] define supported operations.
- [x] do not route PCM through Tauri.

## MT-710 — Implement query-state/status

- [x] running.
- [x] paused.
- [x] machine identity.
- [x] other safe bounded status required by UI.

## MT-711 — Protocol adversarial tests

- [x] malformed payload.
- [x] oversized payload.
- [x] unknown command.
- [x] wrong version.
- [x] stale session.
- [x] connection drop mid-command.
- [x] timeout.

**Milestone:** modern frontend can control a running MAME session without being in the real-time data path.

---

# MT-800 — Artwork and presentation

## MT-801 — Define artwork data model

- [x] screenshot.
- [x] cabinet.
- [x] marquee.
- [x] flyer.
- [x] icon.
- [x] system image.
- [x] provenance/source.

## MT-802 — Implement local artwork discovery

- [x] configurable roots.
- [x] safe filename/path mapping.
- [x] missing-art fallback.
- [x] cache thumbnails where useful.

## MT-803 — Add artwork to machine detail/library views

- [x] lazy loading.
- [x] bounded cache.
- [x] no UI stalls on large artwork sets.

## MT-804 — Define external artwork-provider policy

- [x] licensing.
- [x] attribution.
- [x] privacy.
- [x] caching.
- [x] offline behavior.
- [x] rate limiting.

**Decision gate DG-7:** network-backed artwork providers.

---

# MT-900 — Save-state UX and session management

## MT-901 — Define save-state record model

- [x] machine.
- [x] software item if applicable.
- [x] MAME version/build identity.
- [x] timestamp.
- [x] slot/path.
- [x] optional screenshot metadata if added later.

## MT-902 — Build save-state browser

- [x] list known states.
- [x] save.
- [x] load.
- [x] delete with confirmation.
- [x] stale/incompatible warning.

## MT-903 — Handle incompatible state explicitly

- [x] never imply cross-version compatibility without validation.
- [x] preserve failed state file unless user deletes it.

---

# MT-1000 — Native-window integration research

> This phase is **not** a prerequisite for a useful product.

## MT-1001 — Document native handle capabilities per platform

- [ ] **Deferred — optional research:** Windows native window handle path.
- [ ] **Deferred — optional research:** macOS native view/layer path.
- [ ] **Deferred — optional research:** Linux X11 path.
- [ ] **Deferred — optional research:** Linux Wayland path.
- [ ] **Deferred — optional research:** Tauri/TAO/wry thread/lifetime constraints.

## MT-1002 — Prototype native child/sibling render surface

- [ ] **Deferred — optional research:** create native surface associated with Tauri window.
- [ ] **Deferred — optional research:** resize it from Rust/native code.
- [ ] **Deferred — optional research:** coexist with WebView.
- [ ] **Deferred — optional research:** prove no framebuffer IPC through JS.

## MT-1003 — Prototype BGFX targeting strategy

- [ ] **Deferred — optional research:** determine whether MAME BGFX integration can target chosen native surface.
- [ ] **Deferred — optional research:** identify required MAME changes.
- [ ] **Deferred — optional research:** measure initialization/shutdown behavior.

## MT-1004 — Prototype alternate native renderer if BGFX unsuitable

- [ ] **Deferred — optional research:** document reason BGFX is insufficient.
- [ ] **Deferred — optional research:** implement smallest comparison prototype.

## MT-1005 — Validate resize/HiDPI

- [ ] **Deferred — optional research:** continuous resize.
- [ ] **Deferred — optional research:** DPI changes.
- [ ] **Deferred — optional research:** aspect ratio.
- [ ] **Deferred — optional research:** scaling.

## MT-1006 — Validate fullscreen

- [ ] **Deferred — optional research:** enter/exit fullscreen.
- [ ] **Deferred — optional research:** restore window state.
- [ ] **Deferred — optional research:** monitor selection.

## MT-1007 — Validate multi-monitor

- [ ] **Deferred — optional research:** move between displays.
- [ ] **Deferred — optional research:** different DPI/scales.
- [ ] **Deferred — optional research:** renderer recreation where required.

## MT-1008 — Validate focus and input ownership

- [ ] **Deferred — optional research:** WebView controls.
- [ ] **Deferred — optional research:** gameplay surface.
- [ ] **Deferred — optional research:** keyboard.
- [ ] **Deferred — optional research:** mouse.
- [ ] **Deferred — optional research:** gamepad.
- [ ] **Deferred — optional research:** Escape/menu policy.

## MT-1009 — Benchmark external vs embedded rendering

Measure:

- [ ] **Deferred — optional research:** average frame time.
- [ ] **Deferred — optional research:** frame-time variance.
- [ ] **Deferred — optional research:** CPU overhead.
- [ ] **Deferred — optional research:** GPU overhead.
- [ ] **Deferred — optional research:** dropped/stalled frames.
- [ ] **Deferred — optional research:** input latency where practical.
- [ ] **Deferred — optional research:** audio/video synchronization.

## MT-1010 — Embedded-render go/no-go report

Document:

- [ ] **Deferred — optional research:** per-platform feasibility.
- [ ] **Deferred — optional research:** required MAME patch surface.
- [ ] **Deferred — optional research:** known limitations.
- [ ] **Deferred — optional research:** performance comparison.
- [ ] **Deferred — optional research:** maintenance cost.
- [ ] **Deferred — optional research:** recommendation.

**Decision gate DG-4:** embedded renderer strategy.

---

# MT-1100 — Dedicated Tauri MAME OSD

> Begin only after MT-1010 recommends proceeding.

## MT-1101 — Define Tauri OSD scope

- [ ] **Deferred — optional research:** exact responsibilities.
- [ ] **Deferred — optional research:** reusable existing modules.
- [ ] **Deferred — optional research:** platform-specific glue.
- [ ] **Deferred — optional research:** files added/modified in upstream tree.

## MT-1102 — Add minimal `tauri_osd_interface`

- [ ] **Deferred — optional research:** compile.
- [ ] **Deferred — optional research:** initialize.
- [ ] **Deferred — optional research:** process events.
- [ ] **Deferred — optional research:** report focus.
- [ ] **Deferred — optional research:** clean shutdown.

## MT-1103 — Implement window lifecycle

- [ ] **Deferred — optional research:** create/attach required native surface.
- [ ] **Deferred — optional research:** resize.
- [ ] **Deferred — optional research:** destroy.
- [ ] **Deferred — optional research:** multi-window policy documented.

## MT-1104 — Integrate selected render module

- [ ] **Deferred — optional research:** render representative raster machine.
- [ ] **Deferred — optional research:** render representative vector machine if relevant.
- [ ] **Deferred — optional research:** shader/effect compatibility characterization.

## MT-1105 — Integrate input natively

- [ ] **Deferred — optional research:** keyboard.
- [ ] **Deferred — optional research:** mouse.
- [ ] **Deferred — optional research:** gamepad.
- [ ] **Deferred — optional research:** lightgun behavior documented if unsupported/partial.
- [ ] **Deferred — optional research:** no JS gameplay-input dependency.

## MT-1106 — Integrate native audio

- [ ] **Deferred — optional research:** preserve MAME audio path.
- [ ] **Deferred — optional research:** validate latency/synchronization.

## MT-1107 — OSD validation subset

- [ ] **Deferred — optional research:** representative arcade machine.
- [ ] **Deferred — optional research:** representative computer/console if project scope includes them.
- [ ] **Deferred — optional research:** pause/reset/state operations.
- [ ] **Deferred — optional research:** fullscreen.
- [ ] **Deferred — optional research:** clean exit.

## MT-1108 — OSD upstream-maintenance audit

- [ ] **Deferred — optional research:** enumerate changes outside `src/osd/tauri`.
- [ ] **Deferred — optional research:** reduce unnecessary core patches.
- [ ] **Deferred — optional research:** rehearse upstream merge/rebase.

**Decision gate DG-5:** retain dedicated OSD only if benefits exceed maintenance cost.

---

# MT-1200 — Optional in-process MAME/library integration

> This phase is optional and must not begin merely because it is technically interesting.

## MT-1201 — Produce in-process justification document

Must identify specific requirements not adequately met by sidecar/OSD architecture.

- [ ] **Deferred — optional research:** measurable benefit.
- [ ] **Deferred — optional research:** expected maintenance cost.
- [ ] **Deferred — optional research:** crash-isolation tradeoff.
- [ ] **Deferred — optional research:** upstream impact.

**Decision gate DG-6:** approve or reject in-process work.

## MT-1202 — Characterize MAME library-build feasibility

- [ ] **Deferred — optional research:** build-system implications.
- [ ] **Deferred — optional research:** symbol/export model.
- [ ] **Deferred — optional research:** initialization assumptions.
- [ ] **Deferred — optional research:** global/static state.
- [ ] **Deferred — optional research:** event/thread assumptions.

## MT-1203 — Define narrow C-compatible façade

Conceptual operations:

- [ ] **Deferred — optional research:** create/destroy.
- [ ] **Deferred — optional research:** configure.
- [ ] **Deferred — optional research:** start.
- [ ] **Deferred — optional research:** pause/resume.
- [ ] **Deferred — optional research:** reset.
- [ ] **Deferred — optional research:** save/load.
- [ ] **Deferred — optional research:** query state.
- [ ] **Deferred — optional research:** request shutdown.

## MT-1204 — Define ownership/thread model

- [ ] **Deferred — optional research:** caller thread.
- [ ] **Deferred — optional research:** emulator thread.
- [ ] **Deferred — optional research:** callbacks.
- [ ] **Deferred — optional research:** renderer thread requirements.
- [ ] **Deferred — optional research:** shutdown ordering.

## MT-1205 — Contain C++ exceptions and Rust panics

- [ ] **Deferred — optional research:** no exception crosses C ABI.
- [ ] **Deferred — optional research:** no Rust panic unwinds into C++.
- [ ] **Deferred — optional research:** explicit fatal error propagation.

## MT-1206 — Version ABI

- [ ] **Deferred — optional research:** ABI version.
- [ ] **Deferred — optional research:** feature detection.
- [ ] **Deferred — optional research:** mismatch behavior.

## MT-1207 — In-process prototype

- [ ] **Deferred — optional research:** launch one tiny/representative machine.
- [ ] **Deferred — optional research:** clean shutdown.
- [ ] **Deferred — optional research:** repeated create/destroy cycle if intended.
- [ ] **Deferred — optional research:** compare with sidecar performance/stability.

## MT-1208 — Final in-process adoption decision

- [ ] **Deferred — optional research:** retain sidecar.
- [ ] **Deferred — optional research:** hybrid architecture.
- [ ] **Deferred — optional research:** adopt in-process.
- [ ] **Deferred — optional research:** document rationale.

---

# MT-1300 — Packaging and distribution

## MT-1301 — Define developer executable modes

- [x] external MAME path.
- [x] in-tree development build.
- [x] bundled sidecar build where supported.

## MT-1302 — Bundle required MAME resources correctly

- [x] identify runtime files.
- [x] preserve licenses.
- [x] verify paths after installation.

## MT-1303 — Windows packaging

- [x] installer/package choice.
- [x] sidecar path validation.
- [x] clean install smoke test.
- [x] clean uninstall behavior.

## MT-1304 — macOS packaging

- [x] app bundle layout.
- [x] sidecar/runtime resources.
- [x] signing.
- [x] notarization process.
- [x] clean-machine smoke test.

## MT-1305 — Linux packaging

- [x] choose initial package formats.
- [x] runtime dependency strategy.
- [x] X11/Wayland qualification notes.

## MT-1306 — Version reporting

UI/diagnostics should report:

- [x] application version.
- [x] Git/build identity where appropriate.
- [x] MAME version/build identity.
- [x] database schema version.
- [x] runtime protocol version.

## MT-1307 — License inventory artifact

- [x] MAME licenses.
- [x] Rust dependencies.
- [x] JavaScript dependencies.
- [x] redistributed native libraries.

---

# MT-1400 — CI and automated validation

## MT-1401 — Add project-specific CI workflow

- [x] Rust fmt.
- [x] Rust clippy.
- [x] Rust tests.
- [x] TypeScript typecheck.
- [x] frontend lint.
- [x] frontend tests.
- [x] frontend production build.

## MT-1402 — Add metadata fixture tests to CI

- [x] XML parse.
- [x] import.
- [x] migration.
- [x] search.

## MT-1403 — Add sidecar/process tests to CI where feasible

- [x] known executable fixture or tiny MAME build.
- [x] launch/exit.
- [x] invalid launch.
- [x] diagnostic capture.

## MT-1404 — Integrate `SUBTARGET=tiny` validation

- [x] determine fastest representative native MAME build.
- [x] avoid redundant full builds on every frontend-only change.
- [x] preserve full upstream CI where required.

## MT-1405 — Add path-based CI optimization

- [x] frontend-only changes avoid unnecessary full MAME compile when safe.
- [x] MAME-core/OSD changes trigger relevant native builds.
- [x] workflow changes trigger all appropriate validation.

## MT-1406 — Cross-platform CI matrix

- [x] Linux.
- [x] Windows.
- [x] macOS.
- [x] document which tests are compile-only vs executable.

## MT-1407 — Exact-head CI evidence helper

- [x] easy retrieval/report of tested SHA.
- [x] release checklist records exact green SHA.

---

# MT-1500 — Security hardening

## MT-1501 — Audit Tauri capabilities/permissions

- [x] remove unused capabilities.
- [x] deny generic shell access.
- [x] narrow filesystem access.
- [x] narrow process execution to intended MAME paths/model.

## MT-1502 — Path traversal/adversarial filesystem tests

- [x] `..` traversal.
- [x] symlink edge cases where applicable.
- [x] Unicode/path normalization.
- [x] unauthorized path request.

## MT-1503 — Process argument injection tests

- [x] shell metacharacters.
- [x] embedded quotes.
- [x] spaces.
- [x] malicious machine/software identifier.

## MT-1504 — IPC/control-protocol security audit

- [x] local exposure only.
- [x] stale-session hijack prevention.
- [x] malformed payload.
- [x] oversized payload.
- [x] unauthorized peer assumptions documented.

## MT-1505 — Remote-content policy

- [x] no privileged arbitrary remote page loading.
- [x] artwork/network fetches isolated from privileged WebView navigation.
- [x] CSP/security headers/config reviewed.

## MT-1506 — Dependency/supply-chain policy

- [x] lockfiles committed.
- [x] dependency update process.
- [x] vulnerable dependency reporting policy.
- [x] release provenance expectations.

---

# MT-1600 — Failure-mode and observability hardening

## MT-1601 — Define error taxonomy

- [x] stable error codes.
- [x] human-readable message.
- [x] diagnostic context.
- [x] frontend mapping.

## MT-1602 — Add structured application logs

- [x] timestamps.
- [x] component.
- [x] session ID.
- [x] severity.
- [x] bounded retention.

## MT-1603 — Add diagnostics view/export

- [x] application version.
- [x] MAME identity.
- [x] relevant paths with privacy-sensitive redaction policy.
- [x] recent errors.
- [x] session diagnostics.

## MT-1604 — Crash recovery

- [x] stale session detection after app restart.
- [x] orphaned MAME process policy.
- [x] corrupt/incomplete metadata import recovery.
- [x] interrupted config write recovery.

## MT-1605 — Silent-failure audit

Audit every subsystem for:

- [x] swallowed errors.
- [x] fallback without UI indication.
- [x] ambiguous `None`/empty results.
- [x] lost child-process failures.
- [x] stale status presented as current.

---

# MT-1700 — Performance qualification

## MT-1701 — Define performance benchmark corpus

- [x] full catalog search benchmark.
- [x] representative launch benchmark.
- [x] representative raster machine.
- [x] representative demanding machine where useful.

## MT-1702 — Frontend performance baseline

Measure:

- [x] cold app startup.
- [x] library ready time.
- [x] search latency.
- [x] large-list responsiveness.
- [x] artwork loading behavior.

## MT-1703 — Sidecar overhead baseline

Compare direct MAME vs Tauri-launched equivalent configuration:

- [x] CPU usage.
- [x] memory usage.
- [x] frame behavior.
- [x] launch latency.

## MT-1704 — Metadata generation performance

- [x] generation time.
- [x] peak memory.
- [x] DB import time.
- [x] incremental/stale refresh behavior.

## MT-1705 — Embedded-render qualification

Only if MT-1000 proceeds:

- [ ] **Deferred — optional research:** frame pacing comparison.
- [ ] **Deferred — optional research:** input latency comparison.
- [ ] **Deferred — optional research:** CPU/GPU overhead comparison.
- [ ] **Deferred — optional research:** audio/video sync.

---

# MT-1800 — Cross-platform qualification

## MT-1801 — Linux qualification

- [x] development build.
- [x] packaged build.
- [x] external MAME executable.
- [x] bundled sidecar if supported.
- [x] controller input.
- [x] audio.
- [x] fullscreen.
- [x] X11.
- [x] Wayland where supported/claimed.

## MT-1802 — Windows qualification

- [x] development build.
- [x] packaged build.
- [x] controller input.
- [x] audio.
- [x] fullscreen.
- [x] renderer behavior.

## MT-1803 — macOS qualification

- [x] development build.
- [x] signed packaged build.
- [x] controller input.
- [x] audio.
- [x] fullscreen.
- [x] notarization/install behavior.

## MT-1804 — Cross-platform configuration portability audit

- [x] path differences.
- [x] settings migration.
- [x] controller identity differences.
- [x] artwork paths.

---

# MT-1900 — Upstream sustainability

## MT-1901 — Add upstream remote/sync documentation

- [x] fetch upstream.
- [x] compare divergence.
- [x] merge/rebase policy.
- [x] conflict handling.
- [x] test requirements after sync.

## MT-1902 — Track project-owned MAME patches

- [x] generate/list patches outside project-only directories.
- [x] explain each patch.
- [x] identify patches that can be upstreamed or eliminated.

## MT-1903 — Perform first upstream-sync rehearsal

- [x] start from known project head.
- [x] integrate newer upstream master.
- [x] resolve conflicts.
- [x] run project CI.
- [x] record effort/problems.

## MT-1904 — Establish recurring upstream-sync cadence

- [x] choose cadence based on divergence and activity.
- [x] avoid multi-month drift where practical.

---

# MT-2000 — Product completion before embedded rendering

This milestone proves the project has value even if native embedding is never adopted.

## MT-2001 — Modern frontend completeness gate

Required:

- [x] machine browser.
- [x] machine detail.
- [x] fast search/filter.
- [x] favorites.
- [x] collections.
- [x] recents.
- [x] local artwork support.
- [x] machine/software launch.
- [x] executable/content path configuration.
- [x] audit state.
- [x] per-machine settings foundation.
- [x] supervised MAME lifecycle.
- [x] actionable errors.

## MT-2002 — Runtime management completeness gate

Where supported by MT-700:

- [x] pause/resume.
- [x] reset.
- [x] clean exit.
- [x] save/load state.
- [x] runtime status.

## MT-2003 — UX acceptance pass

- [x] keyboard-first workflow.
- [x] controller-navigation expectations documented/tested.
- [x] no blocking UI operations.
- [x] errors understandable.
- [x] common settings discoverable.

## MT-2004 — External-window release candidate

- [x] package on supported platforms.
- [x] complete smoke test.
- [x] document known limitations.
- [x] record exact qualified commit.

**Major milestone:** production-useful Tauri MAME frontend exists without embedded rendering.

---

# MT-2100 — Final quality closure

## MT-2101 — Cross-cutting unsafe-fallback audit

- [x] executable fallback.
- [x] renderer fallback.
- [x] config fallback.
- [x] audit fallback.
- [x] metadata fallback.
- [x] protocol fallback.
- [x] artwork fallback.

Every fallback must be intentional, documented, and surfaced when materially relevant.

## MT-2102 — Cross-cutting silent-failure audit

- [x] Rust ignored results.
- [x] frontend rejected promises/events.
- [x] child process failures.
- [x] database errors.
- [x] file writes.
- [x] migration errors.
- [x] protocol disconnects.

## MT-2103 — Security closure

- [x] Tauri capability audit complete.
- [x] IPC attack tests complete.
- [x] filesystem attack tests complete.
- [x] process argument attack tests complete.
- [x] dependency scan reviewed.

## MT-2104 — Performance closure

- [x] direct-vs-Tauri overhead acceptable.
- [x] library performance acceptable.
- [x] embedded renderer, if present, meets documented thresholds — **not applicable to the external-window release; MT-1705 remains deferred.**

## MT-2105 — Cross-platform closure

- [x] Linux acceptance.
- [x] Windows acceptance.
- [x] macOS acceptance.
- [x] packaging acceptance.

## MT-2106 — Documentation closure

- [x] install instructions.
- [x] developer build instructions.
- [x] MAME executable/content setup.
- [x] troubleshooting.
- [x] architecture overview.
- [x] upstream-sync procedure.
- [x] release procedure.

## MT-2107 — Final exact-head CI

- [x] all required project CI green on exact candidate SHA.
- [x] relevant upstream MAME CI green.
- [x] no uncommitted closure changes.
- [x] exact SHA recorded in release/closure document.

---

# MT-2200 — Engineering-phase closure

## MT-2201 — Reconcile all TODO state

For every task:

- [x] complete with evidence;
- [x] explicitly deferred with rationale; or
- [x] explicitly rejected/superseded with rationale.

No ambiguous half-complete checkbox remains.

## MT-2202 — Preserve unresolved research status accurately

If embedded rendering, dedicated OSD, or in-process hosting is not complete:

- [x] do not imply it is complete.
- [x] preserve experiment evidence.
- [x] document blockers/decision.
- [x] separate useful-product completion from research completion.

## MT-2203 — Final upstream divergence audit

- [x] list fork-specific commits/files.
- [x] identify avoidable upstream modifications.
- [x] confirm future sync remains tractable.

## MT-2204 — Create handoff/maintenance document

Include:

- [x] architecture summary.
- [x] qualified versions.
- [x] known limitations.
- [x] CI/release workflow.
- [x] upstream sync procedure.
- [x] remaining optional research.

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

- [x] Tauri window launches.
- [x] React calls Rust through typed command.
- [x] Rust uses configured MAME executable.
- [x] Rust invokes a bounded metadata query/import.
- [x] frontend displays at least a small searchable machine list.
- [x] user selects a known machine.
- [x] Rust launches MAME as supervised child.
- [x] frontend receives session-started state.
- [x] MAME runs in its normal external native window.
- [x] user requests stop.
- [x] child exits cleanly.
- [x] frontend returns to idle state.
- [x] failed launch shows structured error.
- [x] no video frame crosses Tauri IPC.
- [x] no PCM stream crosses Tauri IPC.

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

Embedded rendering, a dedicated Tauri MAME OSD, and in-process MAME hosting are **separate optional success tracks**. They must not be falsely treated as required for the core frontend project to be considered successful.
