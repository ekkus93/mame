# MAME Tauri Architecture Specification

**Document date:** 2026-09-08  
**Repository:** `ekkus93/mame`  
**Upstream:** `mamedev/mame`  
**Planning baseline:** `7cc3033a50b00801240f020b7098e22ffffdcd44`  
**Status:** Initial architecture/specification baseline

---

## 1. Purpose

This document defines the architecture, scope, constraints, quality requirements, migration strategy, and acceptance model for a modern Tauri-based application shell and operating-system integration layer for MAME.

The project is **not** a rewrite of MAME in Rust, JavaScript, TypeScript, WebAssembly, or another language. The existing MAME C++ emulation core remains authoritative for emulation, timing, device models, rendering primitives, audio generation, input semantics, software lists, machine definitions, validation, and compatibility.

The project introduces a modern desktop application around MAME using:

- Tauri 2 as the desktop application framework;
- Rust for the trusted application/backend layer;
- React and TypeScript for the user interface;
- existing MAME C++ binaries and subsystems as the emulation engine;
- native rendering/audio/input paths for latency-sensitive work;
- a staged path from sidecar integration to optional deeper OSD or in-process integration.

The central architectural rule is:

> **The WebView/JavaScript layer must never become part of the real-time emulation, video, or audio hot path.**

The Tauri frontend may command and observe MAME, but frame transport, audio sample transport, emulation timing, and other latency-critical work remain native.

---

## 2. Project goals

### 2.1 Primary goals

The project shall:

1. Provide a modern, responsive desktop user experience for browsing, configuring, launching, and managing MAME systems and software.
2. Preserve MAME's existing emulation accuracy and upstream architecture.
3. Minimize invasive changes to upstream MAME source.
4. Preserve the ability to synchronize the fork with `mamedev/mame` regularly.
5. Provide a useful product early using a sidecar architecture before attempting deep native integration.
6. Support Linux, macOS, and Windows.
7. Keep emulation, rendering, audio, and input latency native and deterministic.
8. Provide typed, versioned, testable boundaries between the Tauri application and MAME.
9. Use MAME's own metadata and validation mechanisms rather than constructing an incompatible parallel model.
10. Establish measurable performance, security, failure-recovery, and cross-platform acceptance criteria.

### 2.2 Secondary goals

The project should eventually support:

- modern library browsing and full-text search;
- favorites, collections, tags, and recent machines/software;
- machine and software metadata views;
- artwork integration;
- ROM/software path management and auditing;
- per-machine configuration;
- controller profiles;
- save-state management;
- runtime pause/resume/reset/save/load controls;
- fullscreen and multi-monitor handling;
- crash recovery and useful diagnostics;
- optional embedding of the native MAME render surface in the Tauri application window;
- optional development of a dedicated Tauri-oriented MAME OSD implementation;
- optional investigation of an in-process MAME library interface if justified by demonstrated product requirements.

---

## 3. Explicit non-goals

The initial project shall **not**:

1. Rewrite MAME emulation cores in Rust.
2. Reimplement MAME CPU/device drivers.
3. Send video frames through Tauri IPC for normal gameplay.
4. Send real-time PCM audio through JavaScript or WebView IPC for normal gameplay.
5. Require embedded native rendering before the frontend becomes useful.
6. Require an in-process `libmame` architecture for the first production-capable application.
7. Fork large portions of MAME's core architecture solely to accommodate the UI.
8. Replace MAME's machine/software definitions with a separately maintained database.
9. Bundle copyrighted ROM images or software content.
10. Hide MAME validation failures or silently fall back to behavior that may misrepresent emulator state.

---

## 4. Baseline and upstream policy

At the planning baseline, `ekkus93/mame:master` and `mamedev/mame:master` both point to:

```text
7cc3033a50b00801240f020b7098e22ffffdcd44
```

This exact synchronization is valuable and should be preserved as an operating discipline.

### 4.1 Branch policy

Recommended repository policy:

- `master` remains the clean integration branch.
- Feature work occurs on dedicated branches.
- Each change reaches `master` through a reviewable pull request when practical.
- Exact-head CI must pass before merge for changes that affect executable behavior.
- Documentation-only exceptions may be allowed deliberately, but should not become the normal engineering path.

### 4.2 Upstream synchronization principle

The project should prefer additions that are physically and logically separated from upstream MAME source.

Preferred ownership:

```text
src-tauri/                 project-owned Rust/Tauri backend
ui/ or frontend/           project-owned React/TypeScript frontend
scripts/tauri/             project-owned generation/integration tooling
src/osd/tauri/             only if/when a dedicated MAME OSD is introduced
```

Existing upstream source should be modified only where a clean extension point does not exist.

### 4.3 Patch budget

A standing design objective is that the overwhelming majority of Tauri-specific code lives outside existing MAME core files. Any proposal that requires broad edits to `src/emu`, `src/devices`, or machine drivers must justify why an OSD/module boundary cannot solve the problem.

---

## 5. Current MAME architectural seam

MAME already provides the most important boundary needed by this project: an OS-dependent abstraction layer.

At the planning baseline, `osd_common_t` derives from `osd_interface` and manages platform-facing facilities including:

- video/rendering;
- windows;
- sound;
- keyboard/mouse/joystick/lightgun input;
- monitors;
- fonts;
- debugger integration;
- MIDI;
- networking;
- output modules;
- event processing;
- focus state.

Platform-specific implementations such as SDL, SDL3, Windows, and macOS derive from or compose this common OSD layer.

Rendering is also modular, with implementations including software, SDL-family, GDI, BGFX, and other platform-specific renderers.

This means the project can pursue a dedicated Tauri/native host without changing machine emulation semantics.

---

## 6. Target architecture

### 6.1 Logical architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                     Tauri desktop app                       │
│                                                             │
│  React + TypeScript                                         │
│  ├─ library browser                                         │
│  ├─ search/filter                                           │
│  ├─ machine/software details                                │
│  ├─ favorites/collections/recents                           │
│  ├─ configuration                                           │
│  ├─ save-state UI                                           │
│  └─ runtime controls/overlays                               │
│                       │                                     │
│                typed Tauri commands/events                  │
│                       │                                     │
│  Rust application layer                                    │
│  ├─ process supervision                                     │
│  ├─ metadata/indexing                                       │
│  ├─ configuration                                           │
│  ├─ ROM/software auditing                                   │
│  ├─ runtime control protocol                                │
│  ├─ filesystem/security                                     │
│  └─ native window/render coordination                       │
│                       │                                     │
├───────────────────────┼─────────────────────────────────────┤
│                       ▼                                     │
│                 MAME C++ runtime                            │
│  ├─ emulation core                                          │
│  ├─ devices/machine drivers                                 │
│  ├─ rendering primitives/native renderer                    │
│  ├─ audio                                                   │
│  ├─ input                                                   │
│  ├─ software lists                                          │
│  └─ validation                                              │
└─────────────────────────────────────────────────────────────┘
```

### 6.2 Architectural stages

The implementation shall progress through increasing levels of integration.

#### Stage A — Sidecar launcher

Tauri launches and supervises the existing MAME executable.

```text
Tauri process
   └─ MAME child process
```

This stage provides the lowest-risk route to a useful application.

#### Stage B — Full modern frontend with external MAME window

Tauri owns library discovery, metadata, configuration, launch UX, process state, errors, favorites, collections, and related application features while MAME still owns its native gameplay window.

This stage is expected to be production-useful.

#### Stage C — Runtime control channel

A small, explicit control protocol enables commands and events such as:

- pause/resume;
- reset;
- exit;
- save/load state;
- mute/volume;
- machine state;
- error/status reporting.

The control path must not carry frame or PCM payloads.

#### Stage D — Embedded native render surface

The application hosts a native surface associated with the Tauri window while MAME renders directly to native/GPU resources.

The WebView remains a control/overlay UI and does not relay the frame stream.

#### Stage E — Dedicated Tauri MAME OSD

If required, add a Tauri-oriented OSD implementation under a narrow source boundary such as `src/osd/tauri/`.

#### Stage F — Optional in-process integration

Only after sidecar and OSD approaches are characterized, investigate a stable native API/library boundary that allows Rust to host MAME in-process.

This stage is optional and must be justified by measured benefits that cannot reasonably be achieved with a supervised child process or dedicated OSD.

---

## 7. Tauri application architecture

### 7.1 Frontend

The frontend shall use React and TypeScript.

Frontend responsibilities include:

- visual navigation and layout;
- user-entered search/filter state;
- library presentation;
- artwork presentation;
- configuration forms;
- runtime command invocation;
- non-real-time status visualization;
- accessibility and keyboard navigation;
- localization hooks where practical.

The frontend shall not directly read arbitrary filesystem paths or execute arbitrary shell commands.

### 7.2 Rust backend

The Rust layer is the trusted application boundary.

It owns:

- MAME process launch/termination;
- executable selection;
- command-line construction;
- configuration persistence;
- metadata ingestion;
- SQLite access;
- filesystem path authorization;
- audit invocation and parsing;
- child-process stdout/stderr handling;
- runtime-control transport;
- platform integration;
- native window-handle coordination;
- security policy enforcement.

### 7.3 Typed command model

All frontend/backend commands should use typed request/response structures rather than loosely shaped JSON.

Examples:

```text
launch_machine(request) -> LaunchResult
stop_session(session_id) -> StopResult
query_library(query) -> MachinePage
get_machine(machine_id) -> MachineDetails
audit_machine(machine_id) -> AuditResult
save_state(session_id, slot) -> OperationResult
```

Every command that mutates runtime state should return a structured result and stable error code.

### 7.4 Event model

Backend-to-frontend events may include:

```text
session.started
session.ready
session.paused
session.resumed
session.exited
session.crashed
session.stderr
library.scan_started
library.scan_progress
library.scan_completed
audit.completed
configuration.changed
```

Events must be versioned or structured so incompatible payload changes are detectable.

---

## 8. MAME process supervision

### 8.1 Sidecar lifecycle

The backend must treat MAME as a supervised process, not as an opaque fire-and-forget executable.

Each launch creates a session record containing at minimum:

- session identifier;
- requested machine/system;
- optional software item;
- executable path/version;
- effective argument vector;
- effective configuration profile;
- start timestamp;
- child process identifier;
- exit status;
- captured diagnostics location.

### 8.2 Process rules

The application must:

- prevent accidental duplicate launches when incompatible with the selected mode;
- distinguish normal exit from crash/forced termination;
- capture stdout/stderr without unbounded memory growth;
- provide deterministic stop behavior;
- avoid shell interpolation when constructing child commands;
- surface launch errors directly to the user;
- clean up session state after abnormal termination.

### 8.3 No silent fallback

If the requested executable, ROM path, software item, renderer, configuration, or control channel is unavailable, the backend must not silently substitute materially different behavior unless the fallback is explicit, documented, and surfaced to the user.

---

## 9. Metadata subsystem

### 9.1 Source of truth

MAME remains the source of truth for machine metadata.

The application should ingest machine metadata from MAME-generated outputs, beginning with `-listxml` and supplementing only where necessary with other stable MAME commands or files.

### 9.2 Local index

A local SQLite database shall provide fast query performance without invoking MAME for every UI interaction.

The index should track:

- MAME version/build identity;
- source metadata generation timestamp;
- machine short name;
- display description;
- year;
- manufacturer;
- clone/parent relationships;
- source file;
- device/CPU/sound/display metadata where useful;
- driver status;
- software-list relationships;
- ROM/disk requirements where practical;
- user-owned annotations such as favorite/collection state in separate tables.

### 9.3 Separation of generated and user data

Generated MAME metadata and user-managed application state must be stored separately enough that metadata regeneration cannot erase favorites, collections, custom artwork choices, tags, play history, or per-user settings.

### 9.4 Versioning

The database schema must have explicit migrations.

The index must be invalidated or refreshed when the MAME executable identity changes in a way that may alter `-listxml` output.

---

## 10. Library and search UX

The frontend should support:

- fast search by machine description and short name;
- filtering by manufacturer, year, source, status, type/category where available;
- clone/parent grouping;
- favorites;
- collections;
- recent systems/software;
- machine details;
- software-list browsing;
- audit/availability state;
- configurable artwork views;
- keyboard/controller-friendly navigation.

Search must remain responsive for the full MAME machine catalog.

Large lists must use pagination, virtualization, or equivalent bounded rendering.

---

## 11. ROM and software management

### 11.1 Scope

The application may manage paths and invoke MAME validation/audit facilities, but it shall not distribute copyrighted game/software content.

### 11.2 Features

The application should provide:

- ROM path configuration;
- software path configuration;
- CHD path awareness where appropriate;
- per-machine audit status;
- clear missing-file/dependency messages;
- distinction between missing, incorrect, optional, and unavailable content where MAME exposes those states;
- explicit rescan/reaudit actions.

### 11.3 Integrity

Audit results must come from MAME or a deliberately equivalent validated mechanism. The application must not report a machine as runnable based only on file presence when MAME itself would reject the content.

---

## 12. Configuration model

### 12.1 Layers

The application should model configuration as explicit layers, for example:

1. application defaults;
2. MAME global defaults;
3. platform/profile defaults;
4. machine-specific overrides;
5. transient launch overrides.

The exact precedence must be documented and deterministic.

### 12.2 Preservation

The project must not destructively rewrite unrelated user MAME configuration.

Where possible, project-owned configuration should be separated from upstream/user-managed files or written through a controlled merge process.

### 12.3 Explainability

For any effective option, the UI should eventually be able to show where the value came from.

---

## 13. Runtime control protocol

### 13.1 Purpose

The runtime-control protocol provides a narrow, non-real-time management plane between the Tauri host and a running MAME session.

### 13.2 Initial commands

Target commands include:

- `pause`;
- `resume`;
- `reset`;
- `exit`;
- `save_state`;
- `load_state`;
- `set_mute`;
- `set_volume` where technically appropriate;
- `query_state`.

### 13.3 Initial events

Target events include:

- startup/ready;
- paused/resumed;
- reset acknowledged;
- save/load success/failure;
- exit requested;
- clean exit;
- crash;
- fatal emulator error.

### 13.4 Protocol requirements

The protocol must:

- have an explicit version;
- use bounded message sizes;
- reject malformed requests;
- use local-only transport by default;
- avoid ambient network exposure;
- distinguish command acceptance from command completion when needed;
- define timeout and disconnect behavior;
- never be used for continuous raw video or PCM transport.

---

## 14. Video architecture

### 14.1 Hard rule

Normal gameplay frames must not be transported as full framebuffers through Tauri event/command IPC into JavaScript.

### 14.2 Stage A/B video

MAME owns its native gameplay window using existing render modules.

### 14.3 Embedded video target

For embedded rendering, the intended shape is:

```text
MAME render primitives/native renderer
              ↓
        native GPU surface
              ↓
      Tauri/TAO-owned window
```

The WebView may coexist with or overlay the native surface but is not responsible for presenting every emulated frame.

### 14.4 Renderer investigation

BGFX is a strong candidate for early investigation because MAME already supports a BGFX render module and BGFX provides cross-platform native graphics backends. No renderer shall be selected solely on architectural preference; prototypes must measure compatibility, latency, resize behavior, fullscreen behavior, and platform complexity.

### 14.5 Required embedded-render tests

Any embedded renderer must be tested for:

- frame pacing;
- vsync behavior;
- resizing;
- HiDPI scaling;
- fullscreen transitions;
- multi-monitor movement;
- minimized/occluded behavior;
- device loss/recreation where applicable;
- shader/effect compatibility;
- correct aspect ratio;
- input focus interaction.

---

## 15. Audio architecture

Audio remains native to MAME unless a future requirement demonstrates a compelling need otherwise.

The application may expose controls such as mute or volume, but the PCM stream remains inside native code.

Any future audio bridge must demonstrate that it does not regress:

- latency;
- underrun behavior;
- synchronization;
- channel semantics;
- platform device handling.

---

## 16. Input architecture

### 16.1 Initial model

MAME retains gameplay input ownership in the sidecar stages.

The Tauri frontend handles application-navigation input only while it has focus.

### 16.2 Focus policy

The project must define deterministic focus ownership for:

- keyboard;
- mouse;
- gamepads;
- lightguns;
- global UI shortcuts;
- fullscreen escape/menu behavior.

### 16.3 Deep integration

If a dedicated Tauri OSD is introduced, input may be integrated at the OSD/module level rather than relayed through JavaScript.

No design may require high-frequency gameplay input to make a JavaScript round trip before reaching MAME.

---

## 17. Save states and session persistence

The application should expose save-state management without inventing incompatible state formats.

Requirements include:

- MAME remains responsible for state serialization;
- UI exposes state slots and metadata;
- failure to save/load is surfaced;
- state files are associated with the correct machine/software/configuration context;
- incompatible state handling is explicit;
- the application must not imply portability across MAME versions unless validated.

---

## 18. Artwork subsystem

Artwork is intentionally decoupled from the emulation core.

The architecture should support configurable providers/sources for:

- screenshots;
- cabinet images;
- marquees;
- flyers;
- icons;
- system images;
- user-selected artwork.

The initial implementation may operate entirely on local user-provided artwork.

Any network-backed provider added later must have explicit licensing, caching, privacy, and failure behavior.

---

## 19. Native window integration

Embedded rendering is a research/engineering phase, not an initial prerequisite.

The design must investigate native window/display handles supplied by the Tauri/TAO/wry stack and determine whether MAME/BGFX or another native renderer can target a child/sibling/native surface safely on each supported OS.

Key concerns:

- ownership and lifetime;
- thread affinity;
- event-loop ownership;
- resize synchronization;
- z-order with the WebView;
- focus;
- fullscreen;
- macOS view/layer requirements;
- Windows HWND/D3D integration;
- Linux X11/Wayland differences;
- GPU backend compatibility.

The project must not commit to a single cross-platform strategy until platform prototypes exist.

---

## 20. Dedicated Tauri OSD

If external-window limitations justify deeper integration, the preferred next step is a dedicated OSD implementation rather than broad core modifications.

Conceptual structure:

```text
src/osd/tauri/
├── osdtauri.h
├── osdtauri.cpp
├── window.h
├── window.cpp
└── platform/render/input glue as needed
```

Conceptual interface:

```cpp
class tauri_osd_interface : public osd_common_t
{
public:
    void process_events() override;
    bool has_focus() const override;
    bool window_init() override;
    void window_exit() override;
};
```

Exact implementation details must follow current MAME OSD conventions at implementation time.

### 20.1 OSD acceptance principle

A new OSD is accepted only if it can run a representative machine subset without modifying emulation drivers and without routing timing-sensitive data through the WebView.

---

## 21. Optional in-process MAME integration

### 21.1 Decision gate

In-process hosting is not assumed.

Before beginning it, the project must document concrete shortcomings of the sidecar/OSD design that justify the additional coupling.

### 21.2 Preferred boundary

If pursued, Rust should not bind directly to large unstable portions of the C++ object graph.

A narrow C-compatible façade is preferred, conceptually exposing operations such as:

```text
create
configure
start
pause
resume
reset
save state
load state
request shutdown
query status
destroy
```

### 21.3 Requirements

The bridge must define:

- ownership;
- thread model;
- callback model;
- exception containment;
- panic containment;
- ABI/versioning;
- shutdown semantics;
- crash behavior;
- build/linking policy.

No in-process design may make upstream synchronization unreasonably costly without explicit approval.

---

## 22. Filesystem and data layout

The application should distinguish:

- bundled application resources;
- MAME executable/resources;
- generated machine index;
- user settings;
- ROM/software paths;
- artwork/cache;
- save states;
- logs/crash diagnostics.

Paths must use platform-appropriate application directories.

The UI must not be allowed to request arbitrary sensitive filesystem reads through generic backend commands.

---

## 23. Security model

### 23.1 Tauri boundary

The Rust backend is a privileged boundary. The frontend is not implicitly trusted with arbitrary OS capabilities.

### 23.2 Required controls

The project must:

- expose narrow commands rather than a generic shell endpoint;
- validate machine names and paths;
- construct child argument vectors without shell concatenation;
- scope filesystem access;
- keep runtime-control endpoints local by default;
- bound log and IPC payload sizes;
- prevent path traversal in project-managed stores;
- avoid loading arbitrary remote web content into privileged application views;
- review Tauri capabilities/permissions before release;
- document any intentional external network access.

### 23.3 Sidecar trust

Bundled MAME binaries must be tied to the release/build process. If user-selected MAME executables are supported, the UI must clearly distinguish them from the bundled/tested version.

---

## 24. Error handling and observability

Errors must be actionable and categorized.

Suggested classes:

- configuration error;
- missing executable;
- metadata generation failure;
- malformed metadata;
- ROM/software audit failure;
- launch failure;
- runtime protocol failure;
- MAME fatal error;
- child crash;
- renderer initialization failure;
- filesystem permission failure;
- database migration failure.

Logs should include session IDs and component context while avoiding unnecessary private user data.

The UI should provide a diagnostics view/export mechanism in a later phase.

---

## 25. Performance requirements

### 25.1 UI

The frontend must remain responsive across the full MAME catalog.

Targets should include:

- bounded initial render;
- indexed search;
- virtualized large lists;
- background metadata generation;
- no blocking synchronous MAME invocation on the UI thread.

### 25.2 Emulation

The Tauri application must not materially regress emulation performance compared with launching the same MAME build directly with equivalent options.

### 25.3 Embedded rendering

An embedded-render implementation must be benchmarked against external-window MAME for:

- average frame time;
- frame-time variance;
- input latency where measurable;
- CPU overhead;
- GPU overhead;
- dropped/stalled frames;
- audio synchronization.

A visually integrated result is not acceptable if it introduces material latency or timing instability.

---

## 26. Cross-platform requirements

Supported desktop targets:

- Linux;
- Windows;
- macOS.

The architecture must account for platform-specific differences rather than assuming one native-window strategy works identically everywhere.

Linux qualification should explicitly distinguish X11 and Wayland where relevant.

Windows qualification should cover at least the renderer/input configuration chosen for release.

macOS qualification must cover current supported macOS behavior, signing/notarization requirements, and native view/layer constraints where embedded rendering is used.

---

## 27. Packaging and release model

The project should initially support two development modes:

1. use an explicitly configured external MAME executable;
2. optionally bundle a known MAME executable as a Tauri sidecar for controlled releases.

Release packaging must preserve required MAME runtime data and licenses.

The application must not use the MAME trademark/logo in a way that violates MAME's trademark terms.

Third-party license inventory must include Tauri/Rust/JS dependencies and any redistributed MAME components.

---

## 28. CI strategy

### 28.1 Preserve upstream CI

Existing MAME validation remains important and must not be weakened.

### 28.2 Project-specific CI

Add targeted workflows for:

- Rust formatting/lint/tests;
- TypeScript lint/typecheck/tests;
- frontend build;
- Tauri build/smoke checks;
- metadata parser fixtures;
- database migrations;
- sidecar command construction;
- security/path validation;
- selected MAME tiny-target integration where practical.

### 28.3 Cost control

Full MAME builds are expensive. CI should use the smallest adequate MAME target for rapid project validation and reserve full builds for appropriate integration/release gates.

The existing `SUBTARGET=tiny` path is a useful fast-loop candidate.

### 28.4 Exact-head evidence

Release and major merge gates should record the exact commit SHA that passed validation.

---

## 29. Testing strategy

Testing should be layered.

### 29.1 Unit tests

Cover:

- command construction;
- config precedence;
- XML parsing;
- database operations;
- schema migrations;
- path validation;
- control protocol codecs;
- error mapping.

### 29.2 Integration tests

Cover:

- launching a known small MAME target;
- process startup and clean exit;
- intentional launch failure;
- metadata generation/import;
- ROM audit parsing with fixtures;
- child crash detection;
- runtime protocol round trips when implemented.

### 29.3 UI tests

Cover critical workflows:

- search;
- select machine;
- inspect details;
- configure paths;
- launch;
- stop;
- favorites/collections;
- error display.

### 29.4 Manual qualification

Native rendering, gamepad behavior, fullscreen, multi-monitor, audio latency, and platform packaging require explicit manual qualification even when automation exists.

---

## 30. Development phases

### Phase 0 — Foundations

- project directories;
- branch/CI conventions;
- Tauri/React/Rust scaffold;
- basic docs and architecture tests.

### Phase 1 — Sidecar vertical slice

Minimum proof:

> Open Tauri app → query/index machines → select a known machine → launch MAME → observe state → terminate cleanly.

### Phase 2 — Modern frontend

- complete library UX;
- metadata/index;
- configuration;
- favorites/collections/recents;
- audit status;
- artwork foundation.

At the end of this phase, the project should already be useful with MAME's external gameplay window.

### Phase 3 — Runtime control

- protocol;
- pause/resume/reset;
- save/load;
- status/events;
- clean shutdown integration.

### Phase 4 — Embedded rendering research

- native handle prototypes;
- renderer prototypes;
- platform characterization;
- latency/performance measurements;
- explicit go/no-go decision.

### Phase 5 — Dedicated Tauri OSD

Only if warranted by Phase 4.

### Phase 6 — Optional in-process integration

Only if a documented decision gate approves it.

### Phase 7 — Release hardening

- cross-platform qualification;
- security audit;
- failure-mode audit;
- packaging/signing;
- performance qualification;
- upstream-sync rehearsal;
- documentation.

---

## 31. Decision gates

The following decisions must be made using evidence rather than assumption.

### DG-1 — Frontend directory/layout

Choose final repository layout after scaffold validation.

### DG-2 — Bundled vs external MAME default

Decide release policy after packaging and licensing evaluation.

### DG-3 — Runtime control transport

Choose local IPC mechanism after prototype comparison.

### DG-4 — Embedded renderer

Choose BGFX/other native strategy only after cross-platform prototypes.

### DG-5 — Dedicated Tauri OSD

Proceed only if embedded/native integration benefits justify maintenance cost.

### DG-6 — In-process library

Proceed only if sidecar/OSD architecture cannot satisfy concrete requirements.

### DG-7 — Artwork/network providers

Add only after licensing/privacy/cache behavior is specified.

---

## 32. Quality invariants

The following are project-wide invariants:

1. MAME remains authoritative for emulation semantics.
2. No real-time framebuffer path through JavaScript.
3. No real-time PCM path through JavaScript.
4. No gameplay-input round trip through JavaScript when a native path exists.
5. No generic privileged shell command exposed to the frontend.
6. No silent material fallback that can mislead the user.
7. Generated metadata refresh must not destroy user state.
8. Runtime failures must leave recoverable application state.
9. Deep integration must remain replaceable/isolated enough to permit upstream MAME synchronization.
10. Performance regressions must be measured, not inferred.

---

## 33. Initial acceptance milestone

The first architectural milestone is complete when all of the following are demonstrated from one reproducible development build:

1. Tauri 2 application launches on the primary development platform.
2. React/TypeScript frontend communicates with Rust using typed commands.
3. Rust locates or uses a configured MAME executable.
4. The application obtains machine metadata from MAME.
5. Metadata is indexed locally.
6. The user can search/select a machine.
7. The user can launch that machine through a supervised MAME child process.
8. The UI receives lifecycle state from the backend.
9. The user can request clean termination.
10. Failed launch produces a structured visible error.
11. No video/audio payload crosses the WebView IPC boundary.
12. Automated tests cover command construction, metadata parsing, and process-state handling.

This milestone intentionally does **not** require embedded rendering.

---

## 34. Product milestone before embedded rendering

Before beginning a major embedded-render implementation, the external-window application should be capable of standing on its own as a useful modern MAME frontend.

Expected capabilities:

- indexed machine browser;
- machine details;
- fast search/filtering;
- software-list browsing where supported;
- ROM/software path configuration;
- audit status;
- launch history;
- favorites/collections;
- artwork foundation;
- per-machine configuration;
- process lifecycle and diagnostics;
- runtime controls supported by the chosen control channel.

This sequencing protects the project from becoming blocked by native-window research.

---

## 35. Definition of project success

The project succeeds if it provides a modern cross-platform MAME desktop experience while preserving MAME's native emulation quality and maintaining a sustainable upstream relationship.

A successful final architecture may still use a supervised MAME process. In-process hosting is not itself a success criterion.

The strongest outcome is the least invasive architecture that provides:

- excellent UX;
- low latency;
- robust process/session control;
- native video/audio/input performance;
- maintainable upstream synchronization;
- reliable cross-platform packaging.

---

## 36. Related execution document

Implementation tasks, dependencies, acceptance criteria, and phase gates are tracked in:

```text
docs/MAME_TAURI_TODO_2026-09-08.md
```
