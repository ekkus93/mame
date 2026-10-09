# MAME Tauri In-App Gameplay Specification

**Date:** 2026-10-08  
**Status:** Required product specification; implementation is incomplete  
**Companion TODO:** [MAME Tauri In-App Gameplay TODO](MAME_TAURI_IN_APP_GAMEPLAY_TODO_2026-10-08.md)  
**Active post-review remediation:** [2026-10-09 remediation specification](MAME_TAURI_IN_APP_GAMEPLAY_POST_REVIEW_REMEDIATION_SPEC_2026-10-09.md) and [remediation TODO](MAME_TAURI_IN_APP_GAMEPLAY_POST_REVIEW_REMEDIATION_TODO_2026-10-09.md)  
**Supersedes:** The external-window-only gameplay assumption in the 2026-09-08 architecture baseline and the statement in `README.md` that gameplay video is not proxied to the frontend.

## 1. Problem statement

The current Tauri application can browse MAME metadata, audit local content, and supervise a MAME process, but it does not present live gameplay inside its TypeScript interface. MAME is currently launched with its own native SDL video path. A successful process launch therefore opens a separate MAME window; a failed startup leaves the user with no gameplay display. Neither behavior meets the intended product contract.

The application must present gameplay in its own Tauri window. The TypeScript frontend owns the visible game surface and draws the live machine image into a browser canvas. MAME remains the authoritative emulator: it executes the machine and produces the machine's video pixels. A bounded native frame bridge transports those pixels to the WebView. This is a host/rendering change, not a rewrite of MAME's emulation in TypeScript.

ROM discovery and launch failures are related user-visible failures but are distinct from video presentation. A correct frame renderer cannot compensate for missing content or a MAME process that exits during startup. The launch and content workflows must be qualified alongside the video path so a user can get from the default ROM directory to an actual playable machine.

## 2. Product contract

1. Starting a machine displays its live output in the Tauri application window. Normal gameplay does not require or open a separate visible MAME SDL window.
2. The TypeScript frontend owns presentation: canvas sizing, scaling, aspect ratio, rotation, fullscreen layout, and application overlays. It consumes MAME-produced machine pixels; it does not synthesize or emulate the game's image.
3. MAME remains authoritative for emulation timing, machine video generation, sound generation, machine/device state, and content validation.
4. The Rust backend owns process supervision, frame transport, input transport, filesystem access, and lifecycle/security boundaries. TypeScript does not get a generic process, filesystem, or socket capability.
5. Gameplay video is a dedicated high-throughput data plane. It must not be serialized as ordinary JSON events or compete with the authenticated runtime-control protocol.
6. Keyboard, controller, and other supported gameplay input must reach MAME while the Tauri game surface owns focus. The browser must not also consume gameplay keystrokes as library shortcuts.
7. MAME's normal sound output remains connected to the user's audio device unless the user mutes it. PCM audio does not travel through the WebView unless a later, separately approved design demonstrates a need.
8. The initial release target is the full Linux AppImage on a normal desktop session. Cross-platform behavior must be designed explicitly; a Linux-only transport must not silently be presented as cross-platform support.
9. ROMs, CHDs, BIOS files, and other user content are never bundled or copied by this work.

## 3. Terminology and ownership

| Concern | Owner |
| --- | --- |
| Emulated machine state, raster/vector output, emulation clock | MAME C++ runtime |
| Reading/encoding machine frames and moving them across the process boundary | Rust bridge and/or a narrowly scoped MAME-side adapter |
| Canvas creation, drawing, scaling, rotation/layout, fullscreen, overlays | TypeScript/React frontend |
| Child lifetime, launch arguments, paths, cleanup, diagnostics | Rust supervisor |
| Machine-specific control semantics | MAME; the host forwards supported physical input without redefining it |
| ROM discovery, authoritative audit, launch readiness | Existing Rust path/audit/catalog model, corrected where this spec requires |

## 4. Target architecture

```text
┌──────────────────────── Tauri application window ────────────────────────┐
│ React / TypeScript                                                       │
│   GameSurface                                                            │
│   ├─ Canvas 2D or WebGL presentation                                     │
│   ├─ aspect/rotation/fullscreen/overlays                                 │
│   └─ keyboard/controller focus and lifecycle UX                          │
│            │ typed low-rate control APIs + frame-surface access          │
│            ▼                                                              │
│ Rust host                                                                │
│   ├─ supervised MAME process                                             │
│   ├─ bounded frame reader / latest-frame buffer                         │
│   ├─ input bridge                                                        │
│   └─ typed status, errors, and cleanup                                   │
└────────────┬─────────────────────────────────────────────────────────────┘
             │ session-scoped binary frame transport + control/input
             ▼
┌──────────────────────── MAME runtime ───────────────────────────────────┐
│ MAME C++ machine emulation → screen pixels; native audio → system output │
│ No visible SDL gameplay window in the normal in-app mode                 │
└──────────────────────────────────────────────────────────────────────────┘
```

The frame transport must preserve frame boundaries and metadata (protocol version, session identity, sequence number, dimensions, pixel format, stride, and payload length). Prefer a latest-frame mailbox or bounded ring with backpressure that discards stale frames rather than accumulating latency. The UI must never render a queue of old frames after load or a slow frame.

The implementation must compare available capture seams before selecting one. MAME currently exposes Lua screen pixel APIs (`screen_device::pixels` is bound as `screen:pixels()` and returns pixels plus visible dimensions) and a frame-done callback. These are a practical prototype seam, not a pre-approved production transport. A Lua-per-frame file dump, large base64 JSON/Tauri events, unbounded pipes, and a second visible SDL window are not acceptable final implementations. If the Lua callback is too expensive or unstable, add a small, isolated MAME-side capture adapter or OSD output module and keep the bulk of project-specific code outside core emulation files.

### 4.1 Production capture-seam decision

The first Linux release selects MAME's existing native snapshot renderer, called from the per-frame Lua callback, plus a dedicated authenticated binary FIFO into Rust. The alternatives were evaluated against the product boundary rather than treated as equivalent transports:

| Candidate | Decision | Rationale |
| --- | --- | --- |
| `screen:pixels()` from Lua | Rejected for production | The binding may expose screen-device pixel representations rather than a guaranteed finished RGB32 presentation, including palette-index ambiguity. It remains useful only as a diagnostic/prototype comparison seam. |
| `manager.machine.video:snapshot_pixels()` + dedicated binary FIFO | Selected for the initial Linux release | It renders through MAME's native C++ software snapshot path to RGB32, keeps frame bytes off stdout/runtime-control, requires no core emulator fork, and fits the session-scoped bounded latest-frame transport. |
| Project-specific MAME capture adapter/OSD output module | Deferred fallback | It offers tighter control and potentially lower overhead, but adds invasive MAME-side maintenance. Promote it only if measured snapshot/Lua overhead fails the performance budget. |
| JSON/base64 Tauri events, per-frame files, or unbounded pipes | Rejected | These violate the bounded high-throughput data-plane requirement and/or create avoidable allocation, latency, or filesystem overhead. |
| Visible native SDL window | Rejected for supported in-app mode | It does not satisfy the product requirement that gameplay be presented in the Tauri-owned surface. |

This decision does **not** close the performance qualification: representative raster, high-resolution, and vector measurements remain required before release acceptance. The selected path is intentionally replaceable behind the Rust frame protocol if those measurements show unacceptable overhead.

The transport decision must be recorded with measured results for representative raster, high-resolution, and vector machines. The transport must be authenticated/session-scoped, private to the current user, bounded in memory, and cleaned up on normal exit, crash, timeout, and app shutdown. It must not expose arbitrary file paths or a general local network listener to the WebView.

## 5. User experience and lifecycle

### 5.1 Library to game

- The user selects a machine and sees its content readiness using the existing catalog/audit model.
- A known available machine starts without a manual metadata import. Metadata/catalog import remains separate from ROM scanning/auditing and must not be described as installing or importing ROMs.
- Unknown or stale availability prompts for the authoritative audit/path setup before normal launch. Missing content reports the machine and searched paths with a clear next action.
- While starting, the app presents an explicit loading state. It changes to gameplay only after both MAME runtime readiness and a valid first video frame have arrived.
- If MAME exits before readiness, show a mapped startup diagnosis and useful captured output. If MAME becomes ready but no video frame arrives before the frame deadline, show a distinct video-pipeline failure, preserve diagnostics, and offer retry/return to library.

### 5.2 In-game surface

- The game image is visible inside the Tauri application window, with no separate MAME gameplay window in normal mode.
- Preserve source aspect ratio by default. Scaling must not stretch pixels. Offer an integer/nearest-neighbor option for pixel art and a smooth option where appropriate.
- Support desktop resize and fullscreen without restarting the machine. Handle changing screen dimensions and rotation without stale frames or a stretched image.
- Provide accessible in-app controls for pause/resume, reset, mute/volume where supported, fullscreen, stop/return, and help. Existing session controls remain available from gameplay.
- Returning to the library cleanly stops or backgrounds the session according to explicit UX; first release may stop on return, but must not orphan the child.
- Window focus and input ownership are explicit. While gameplay owns input, library shortcuts and text search must not intercept game keys. On return, browser ownership is restored only after input state is neutralized to prevent stuck keys.

### 5.3 Failure states

Use separate actionable states for at least:

- MAME executable could not be started;
- MAME exited before runtime-control readiness;
- runtime-control protocol failure;
- MAME started but no first frame arrived;
- malformed/unsupported frame protocol or pixel format;
- frame stream stalled after gameplay began;
- unsupported machine/video topology;
- content missing or invalid;
- input bridge unavailable;
- user-requested stop and unexpected process exit.

Do not show `CONTROL_CHANNEL_CLOSED` as the primary user-facing explanation. Retain raw internal codes in developer diagnostics, alongside the child exit status, bounded stdout/stderr tails, selected machine/software, effective argv/options, effective content paths, runtime identity, and last received frame metadata.

## 6. Functional requirements

### Video and presentation

- **GAME-VID-001:** MAME's machine pixels are delivered to the active Tauri game surface without using the separate visible SDL gameplay window.
- **GAME-VID-002:** TypeScript draws the received frame into a canvas owned by the gameplay view.
- **GAME-VID-003:** Frame messages include version, session ID, monotonic sequence, width, height, pixel format, stride, payload size, capture timestamp, orientation, and flags. Reject malformed, stale-session, oversized, or unsupported messages.
- **GAME-VID-004:** Transport is bounded and drops/replaces old frames rather than allowing queue latency or memory to grow.
- **GAME-VID-005:** Frame arrival and frame presentation timestamps are observable for diagnostics and performance qualification.
- **GAME-VID-006:** Resize, aspect ratio, orientation, fullscreen, and renderer teardown/recreation do not restart the emulated machine.
- **GAME-VID-007:** The selected capture seam supports MAME raster and vector output. If effects such as BGFX shaders, artwork overlays, bezels, multi-screen layouts, or vector phosphor effects are not reproduced in the first implementation, the UI must state the initial scope and a follow-up item must track parity. Machine video pixels must never be silently mistaken for finished MAME presentation when those features are absent.
- **GAME-VID-008:** Session end, app shutdown, and failed startup release frame buffers, IPC handles, temp resources, and callbacks.

### Input and audio

- **GAME-IO-001:** Focused gameplay receives supported keyboard and controller input through a typed, bounded bridge into MAME's normal input model.
- **GAME-IO-002:** Key up/down state is balanced through blur, stop, crash, and focus changes; no held key remains stuck.
- **GAME-IO-003:** Input mappings remain configurable using the existing MAME configuration model where possible; the frontend does not hardcode machine-specific controls.
- **GAME-IO-004:** Input traffic is separate from video frames and runtime-control messages; high-rate axes/buttons are coalesced or sampled without delaying video.
- **GAME-IO-005:** Default audio continues through MAME's native audio backend. Mute/volume behavior and ownership are unambiguous; no audio work is implied complete by video completion.

### Launch and local content

- **GAME-LAUNCH-001:** Launch uses the same effective content path set as the authoritative audit.
- **GAME-LAUNCH-002:** The normal Linux setup discovers the user's conventional MAME ROM location, including `~/mame/roms` as the reported user environment requires, while retaining configured paths and MAME-compatible defaults. The path is constructed from the current user's home directory, never hardcoded to `/home/phil`.
- **GAME-LAUNCH-003:** First run does not require a user to perform a metadata import merely to see their already installed ROMs. Catalog metadata refresh and content audit are distinct operations with clear labels.
- **GAME-LAUNCH-004:** Audit freshness is stable across AppImage mount paths. Bundled MAME identity must not change merely because AppImage's temporary mount directory changes between launches.
- **GAME-LAUNCH-005:** Every launch failure retains and surfaces actionable MAME stdout/stderr and exit status. Diagnostics remain available after terminal sessions are cleared from the active-session view.
- **GAME-LAUNCH-006:** Available locally means audited as complete or best available for the effective current paths/runtime; it does not guarantee that video or input initialization has succeeded.
- **GAME-LAUNCH-007:** Startup success is a two-part barrier: authenticated MAME runtime ready and first valid video frame presented. A missing half has its own diagnosis.

## 7. Non-functional requirements and performance budgets

- Keep emulation scheduling, timing, audio, and machine state out of JavaScript.
- Keep the WebView work per frame to bounded decode/copy/draw operations. Do not run disk, XML, audit, or blocking IPC work on the UI thread.
- Use a bounded memory model. The frame path should retain at most the current frame plus a small documented number of in-flight frames; the UI should favor recency over completeness.
- Qualification target on the supported Linux desktop baseline: sustain 60 presented frames/s for a representative 60 Hz raster machine at native canvas resolution, without monotonically increasing latency, and with no visible gameplay window outside Tauri. Record actual capture, transport, and draw timing. High-resolution and vector-machine budgets must be separately measured and documented; do not claim unsupported rates.
- Frame transport must not materially destabilize emulation speed. Compare the same machine and settings with capture disabled and enabled; record frame rate, emulation speed, CPU, memory, dropped frames, and end-to-end frame age.
- Protect the UI from renderer crashes, malformed frames, and unexpectedly large screen dimensions. Apply explicit dimension and payload caps before allocation.
- Maintain the current typed/security boundary: no generic shell command, arbitrary filesystem access, or unrestricted socket capability is granted to TypeScript.

## 8. Compatibility and scope

The first deliverable is the normal full-runtime Linux AppImage on the user's desktop session, using the full bundled MAME executable. Do not produce or validate only a slimmed-down image. The tested artifact must be the artifact intended for local users.

The design must identify capability differences for development/external MAME overrides and for Windows/macOS. If the capture/input adapter only works with the bundled MAME build, unsupported external runtimes must fail with a clear capability message rather than opening a separate SDL window as an undocumented fallback. Preserve a deliberate developer-only external-window mode only if it is labeled as such and cannot be confused with the supported in-app gameplay mode.

## 9. Verification and acceptance

The work is complete only when all of these are true:

1. A real full-runtime AppImage launched from the desktop starts a machine with valid ROMs and displays the game inside the Tauri window.
2. The run does not open a separate visible MAME gameplay window.
3. Keyboard and at least one supported gamepad/controller path can play a representative game, and pause/reset/stop remain reliable.
4. A normal user's `~/mame/roms` content is found and audited using the same paths used for launch, without an unrelated metadata import step.
5. A missing-ROM machine and an invalid content path produce clear diagnostics rather than an opaque startup/control error.
6. A failure to receive a frame is distinguishable from a failure to start MAME.
7. Raster, vector, resize, fullscreen, stop, crash, and relaunch paths are qualified; any deliberately deferred rendering parity is documented.
8. Automated Rust and frontend tests, lint, typecheck, and build pass. The full AppImage is rebuilt and exercised after source changes; evidence records exact artifact path, hash, MAME version/runtime identity, machine, result, and observed window behavior.
9. No ROM/CHD/BIOS content is added to the artifact, and the AppImage contains the full bundled MAME runtime.
10. README and architecture documentation describe the implemented in-app frame path accurately and no longer promise that gameplay video is intentionally absent.


## 10. Post-review clarifications (2026-10-09)

The parent requirements remain authoritative for product behavior. Implement and qualify all corrections under the [post-review remediation specification](MAME_TAURI_IN_APP_GAMEPLAY_POST_REVIEW_REMEDIATION_SPEC_2026-10-09.md); the [post-review TODO](MAME_TAURI_IN_APP_GAMEPLAY_POST_REVIEW_REMEDIATION_TODO_2026-10-09.md) is the active code-fix checklist.

- **Launch-history success is stricter than runtime-control readiness.** Persist a pending attempt when launching. Mark it successful only on the first accepted frame-presentation acknowledgement for the same live session. Termination, user stop or a reported first-frame presentation failure before any acknowledgement settle the pending attempt as unsuccessful. Repeated callbacks/acknowledgements must be idempotent, and failed database writes remain diagnostic.
- **Input completion must be generation-owned.** Keyboard and W3C-standard gamepad state is released on blur, fullscreen ownership loss, teardown or session replacement. In-flight IPC responses from an obsolete effect generation must not mutate the next session's accepted-state bookkeeping.
- **Frame protocol validation is native-first.** Unknown flag bits and invalid/stale presentation acknowledgements are rejected by Rust before reaching the WebView; frontend parsing independently validates wire presentation fields.
- **Metrics use distinct clock domains.** `latestReceivedAgeMs` is time since Rust received its latest frame. `lastPresentedAgeMs` is time since the backend's accepted presentation acknowledgement. `latestAgeMs` is a deprecated alias for backend receive age, **not** end-to-end latency; MAME's capture timestamp is emulated-time-domain and cannot be subtracted from a host wall clock to claim latency.
- **Audit provenance includes bundled executable identity.** The randomized AppImage mount path is deliberately normalized, while the packaged `mame_sha256` digest invalidates saved audits when binary bytes change despite an identical MAME version string. Legacy audits without the digest must be requalified.
- **Frame-reader teardown must be deterministic.** Cancellation must interrupt a reader waiting for its first writer, blocked between frames or reading a partial frame; do not wait indefinitely on the UI/host shutdown path.
- **Qualification evidence remains scoped.** Full-runtime Debian/AppImage checks and ROM-less `___empty` headless capture are necessary automated gates but are not evidence of real user-ROM gameplay, native sound or desktop frame performance.

**User decision (2026-10-09):** The original 14 desktop, private-ROM and installed-MAME-configuration acceptance items are explicitly deferred, not satisfied. Preserve their checkboxes. Resume them only when the user explicitly authorizes that scope. Automated remediation and CI remain in scope.


## 11. Second post-review runtime clarifications (2026-10-09)

The [second post-review remediation](MAME_TAURI_IN_APP_GAMEPLAY_SECOND_POST_REVIEW_REMEDIATION_SPEC_2026-10-09.md) tightens several lifecycle contracts without changing the core architecture.

- **Input ownership is session-owned.** Controller-profile preference changes within one MAME session update mutable selection state but do not trigger destructive session teardown. Actual session teardown/replacement releases accepted and pending non-zero controls in bounded batches; any successor for the same session waits for an older cleanup barrier before sending new input.
- **History decision and persistence are distinct.** The first legitimate presentation-success or terminal-failure event decides the outcome. A SQLite write failure retains that decision for retry; later competing callbacks cannot change it. Application shutdown performs only a bounded retry pass and records any remaining persistence failure diagnostically.
- **Presentation ACKs use one outstanding-frame contract.** A newly delivered frame replaces the outstanding ACK target. The exact current ACK is counted once; an immediate transport retry for the already-acknowledged current frame is harmless, but an old duplicate after a newer delivery is invalid.
- **Frame-producer EOF is stateful.** FIFO EOF before any producer bytes means “not connected yet”; EOF after producer bytes means the frame producer disappeared or truncated a frame and becomes an actionable stream diagnostic unless intentional cancellation is already in progress.
- **First-frame durable success uses the accepted-ACK contract.** A browser draw alone is not durable success. The frontend queues the drawn sequence and Rust settles success only after accepting that exact presentation acknowledgement. If the process exits after draw but before ACK, the terminal event may settle failure under this contract. Frontend state never writes history directly.
- **Frame-seam evidence paths are contained.** Relative explicit report names resolve beneath the configured/default `artifacts/in-app-gameplay` evidence root and may not escape it. Absolute paths are allowed only as deliberate caller-selected destinations.
- **Package provenance is verified at qualification time.** Build staging records `mame_sha256`; Debian-installed and AppImage-extracted qualification recompute the bundled executable SHA-256 and require an exact match. Normal runtime audit lookups continue using the verified package provenance rather than rehashing the large executable for every query.
