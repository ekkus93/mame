# MAME Tauri In-App Gameplay TODO

**Date:** 2026-10-08  
**Status:** In progress; automated implementation and documentation are substantially reconciled, but real-ROM performance and full-AppImage desktop gameplay qualification remain open  
**Specification:** [MAME Tauri In-App Gameplay Specification](MAME_TAURI_IN_APP_GAMEPLAY_SPEC_2026-10-08.md)

This TODO replaces the former assumption that a supervised MAME process plus a separate SDL window constitutes playable Tauri gameplay. It tracks the complete path from discovered local ROMs through live video, controls, diagnostics, and validation in the full AppImage.

## Deferred acceptance scope (2026-10-09 — user decision)

**On hold, not complete:** All 14 currently unchecked tasks and subtasks below are deferred **to the extent they require the user's graphical desktop, locally installed ROMs or the original installed MAME and its user-specific configuration**. This includes Phase 0 user-case reproduction and startup-cause triage; Phase 1 representative raster/vector/high-resolution capture and Lua timing measurements plus real-machine headless behavior; Phase 3 native audio and representative game testing; Phase 4 original installed-MAME `rompath` parity; Phase 6 end-to-end AppImage gameplay, measured desktop performance, qualification evidence and repeat qualification after a relevant fix; and Phase 7 fresh-profile user review. Do **not** mark any of these checkboxes `[x]` based only on CI's ROM-less `___empty` or synthetic-frame tests. Do **not** request or invent the user's private ROMs, screenshots, performance measurements or machine configuration.

**Not deferred:** Automated source fixes, regression/shell validation, build/package correctness, and exact-source CI investigation that require **neither** the user's desktop **nor** installed ROMs. In particular, the frame-seam script quoting regression in [run 37973227910](https://github.com/ekkus93/mame/actions/runs/37973227910) has been fixed on `master` in commit `7d1ed8426e5b3c8f436002edfd79fdf5cf74f3c1`; its new [real-runtime CI run 37983935896](https://github.com/ekkus93/mame/actions/runs/37983935896) is the outstanding automated gate as of this note. Continue diagnosing and fixing any genuine automated failure; do not conflate this with the deferred desktop acceptance.

**Resumption condition:** When the user explicitly resumes desktop/ROM qualification, use the current passing exact-source AppImage (not a stale binary), the host-local evidence scripts documented below, and the user's own approved locally available MAME media. Reconcile completion only from observed evidence. Until then, preserve the outstanding checkboxes as deferred rather than finished.


## Phase 0 — Establish a reproducible baseline

- [x] Record current branch/head and preserve all pre-existing user changes. The implementation baseline is preserved in Git history beginning at `bea14e376d9491cfb7a38b51d11371bb57a431fe`; Ralph writes use exact-head compare-and-swap on `master`.
- [x] Record the qualified CI full AppImage identity, path within its artifact, SHA-256, bundled MAME version, and exact build command/configuration. The host-local installation path remains to be captured separately during desktop acceptance; see `docs/MAME_TAURI_IN_APP_GAMEPLAY_PACKAGE_EVIDENCE_2026-10-09.md` and Phase 6.
- [ ] Reproduce the current user's case with a machine from `/home/phil/mame/roms` and save the complete launch request, child exit code, bounded stdout/stderr, effective paths/argv, runtime-control transition, and UI state.
- [x] Add a repeatable local launch/diagnostic recipe that does not depend on a screenshot or manually searching session logs. See `scripts/tauri/qualify-in-app-gameplay.sh` and `docs/MAME_TAURI_IN_APP_GAMEPLAY_LOCAL_QUALIFICATION_2026-10-08.md`.
- [ ] Determine whether recent `CONTROL_CHANNEL_CLOSED`/`MAME_EARLY_EXIT` reports are an independent startup defect, a consequence of launching with an incompatible display mode, or both. Keep the causes separate until evidence identifies them.
- [ ] Record baseline behavior for an available ROM machine, a best-available/no-ROM machine, and a known missing-content machine.

**Exit gate:** A developer can reproduce and identify each current failure from saved diagnostics, and knows exactly which full AppImage/runtime was tested.

> **2026-10-08 CI startup diagnosis (not a desktop acceptance result):** real-runtime run [37856024647](https://github.com/ekkus93/mame/actions/runs/37856024647) compiled MAME 0.289 successfully, then its `-video none` smoke exited 255 with `Could not initialize SDL No available video device` before Lua frame capture. This demonstrates a distinct SDL video-subsystem initialization failure despite `-video none`; it does **not** establish the cause of the user's earlier desktop `CONTROL_CHANNEL_CLOSED` or `MAME_EARLY_EXIT` reports. Commits `24d48f1` and `746f77c` select the SDL dummy video backend for headless qualification and the supervised Linux in-app MAME child, without overriding native audio; `56cfd8e` corrected its regression test. At `8ff6c54510aba7647956eadd0c2a9d74ec277e34`, Tauri project [37871818728](https://github.com/ekkus93/mame/actions/runs/37871818728) and ordinary Linux package smoke [37871818721](https://github.com/ekkus93/mame/actions/runs/37871818721) passed. The *real-runtime* package [37871818774](https://github.com/ekkus93/mame/actions/runs/37871818774) is still in progress as of this evidence record; do not infer that the capture spike, full AppImage, or desktop game passed. Preserve the user-host Phase 0 investigation and Phase 1/6 acceptance items unchecked until their own evidence exists.

> **Original-MAME baseline capture helper:** `scripts/tauri/capture-original-mame-baseline.sh` records the independently installed MAME binary/version/configuration and available/missing ROM audit results under the user's normal profile. It is a reproducibility aid, **not** a substitute for host evidence or full-AppImage gameplay acceptance. The unchecked Phase 0/4 observations still require a desktop run.


## Phase 1 — Prove the frame capture and transport seam

- [ ] Build a small capture spike against the bundled MAME version. Verify the `screen:pixels()` return value, dimensions, byte order, alpha behavior, visible-area semantics, screen orientation, and callback timing on raster and vector machines.
- [ ] Measure Lua `screen:pixels()` plus callback overhead at representative native and high resolutions. Reject it for production if it misses the performance/stability budget.
- [x] Compare candidate production seams: `screen:pixels()` diagnostic/prototype capture, MAME native `snapshot_pixels()` plus a dedicated binary FIFO, and a project-specific MAME capture adapter/OSD fallback. The initial Linux release selects `snapshot_pixels()` + private authenticated FIFO; JSON/base64 events, console framing, per-frame files, unbounded pipes, and a visible SDL gameplay window are rejected. Performance qualification remains open.
- [x] Define a versioned frame header: session ID, sequence, dimensions, pixel format, stride, payload size, capture timestamp, orientation/rotation, and flags.
- [x] Choose and document a cross-platform transport strategy or explicitly constrain the first release to Linux AppImage. The initial transport is a private authenticated mode-0600 FIFO on Linux little-endian; unsupported platforms fail closed.
- [x] Prototype one-slot/latest-frame or bounded-ring semantics and prove stale frames are replaced instead of queued. `FrameMailbox` and its tests implement one-slot replacement/drop accounting.
- [ ] Verify `-video none`/headless operation still makes valid machine frames available through the selected seam and does not block normal MAME execution.
- [x] Verify current stdout runtime-control parsing is unaffected; frame bytes use the dedicated binary FIFO and never share the stdout control/diagnostic stream.

**Exit gate:** A real MAME process delivers validated live pixels to a minimal canvas prototype at target performance without a visible SDL game window, with measured overhead and a documented transport decision.

## Phase 2 — Implement Rust frame host and TypeScript game surface

- [x] Add a session-owned frame endpoint/mailbox to the Rust supervisor and tie its lifetime to the exact MAME child/session.
- [x] Add strict dimension/payload limits, protocol-version checks, sequence checks, stale-session rejection, and explicit errors for malformed or unsupported frames.
- [x] Add backpressure/latest-frame behavior and metrics for received, dropped, delivered, and presented frames, presentation duration, age, dimensions, and stream errors/stalls.
- [x] Expose frames to the WebView via the selected typed binary command without a generic socket/filesystem capability.
- [x] Implement a React `GameSurface` with a TypeScript-owned canvas. Draw raw frames, resize backing resolution, preserve aspect ratio, and support nearest/smooth scaling.
- [x] Support dynamic screen dimensions and rotation, with the selected initial multi-screen scope documented as MAME's single composed snapshot target. The canvas resizes per frame and handles 0/90/180/270-degree orientation; independently addressable per-screen WebView surfaces remain an explicit non-goal/gap.
- [x] Add loading, first-frame, active, stalled, unsupported, stopping, and ended gameplay presentation states. The surface remains in waiting/first-frame state until a valid frame is presented even after runtime readiness.
- [x] Add fullscreen, return/stop, pause/resume, reset, mute controls as available, and accessible focus indicators. Unsupported live master-volume control is not falsely claimed.
- [x] Ensure exit/stop/app shutdown/crash/component teardown release child and transport resources. Supervisor drop tests plus closed-mailbox/late-frame tests cover cleanup and stale-frame prevention.

**Exit gate:** A real game frame is drawn by the frontend canvas in the main Tauri window and frame stream lifecycle survives stop/relaunch/failure without leaking or displaying stale frames.


> **2026-10-08 presentation-state/topology evidence:** `GameSurface` resizes its backing canvas from each validated frame, `frameDisplaySize` covers all four supported quarter-turn orientations, unsupported transport/protocol outcomes have a dedicated presentation state, terminal sessions map to an ended state, and active presentation begins only after a valid first frame. The initial multi-screen contract is MAME's composed snapshot target; separate independently controlled screen surfaces are not claimed.

## Phase 3 — Input, audio, and normal gameplay behavior

- [x] Design and implement keyboard input forwarding into MAME's normal input model while the WebView owns focus; input uses the authenticated runtime-control bridge rather than the video queue.
- [x] Implement controller/gamepad forwarding for W3C-standard gamepads, including axes/buttons and browser-standard profile selection.
- [x] Balance desired/accepted input state on blur, fullscreen changes, stop/component teardown, and process termination; tests cover release-to-zero generation.
- [x] Coalesce high-rate axis/button updates by diffing desired versus accepted state and cap each runtime-control batch at 32 updates.
- [ ] Keep MAME's native audio output functioning with video hidden. Verify mute/volume UI reflects actual runtime state and does not imply WebView PCM support.
- [x] Test/implement keyboard-navigation ownership transitions: terminal session events return the shell to the library and gameplay shortcuts are focus-owned while the game surface is active.
- [ ] Test a representative raster game, a vector game, a game with rotation, and a machine with a different resolution/aspect ratio.

**Exit gate:** A user can play a representative game with keyboard and gamepad from the Tauri game surface, hear normal sound, and reliably return to the library without stuck input.

## Phase 4 — Fix ROM discovery, audit, and launch parity

- [ ] Verify how the original installed MAME resolves the user's actual ROM path and effective `rompath`; capture evidence from the same host/configuration.
- [x] Make `~/mame/roms` discoverable by default for the normal Linux user profile while respecting configured paths and documented precedence. Resolution uses the runtime home directory and never hardcodes the developer's home.
- [x] Ensure a configured `<home>/mame` parent resolves its `roms` subdirectory when present; Rust tests cover the convention without hardcoding a username.
- [x] Use one effective-path computation for audit availability and actual game launch. Diagnostics now show effective paths, source, and validation/readability status.
- [x] Separate catalog metadata refresh, content audit, and ROM installation in UI copy. Metadata import now explicitly states that it imports machine metadata only, while path configuration says the app does not download/install ROM, CHD, BIOS, or software content and points users to media audit for verification.
- [x] Keep bundled MAME audit identity stable across random AppImage mount locations. `bundled_appimage_mount_changes_preserve_existing_audits` verifies rebase of persisted bundled-runtime audit identity across randomized mount paths; full desktop relaunch remains part of Phase 6 qualification.
- [x] Verify complete and best-available audit classifications are preserved accurately. Regression coverage includes `best_available_romset_summary_is_not_misreported_as_missing_content`, and availability/audit state remains distinct from driver support and gameplay readiness.
- [x] Gate launch and provide explicit audit paths for Unknown/stale/unavailable content. Tests verify unaudited and missing-content launches fail closed before spawn.
- [x] Test paths with spaces, non-ASCII names, multiple directories, missing directories, permissions errors, and AppImage execution. Rust content-path regressions cover the path/validation matrix, and Linux AppImage packaging smoke passed on binary head `ad605d64145d9bcca547bc0b689fe244c17b2744` in run `37842790394`.

**Exit gate:** The user's default ROM folder appears after a normal launch, audit and Start use identical paths, and a valid locally available machine can reach the first-frame-ready state without manual catalog import.


> **2026-10-08 content/audit evidence:** commit `0e4c1708bae0053926fd3a143ea1ffd31bf50a69` clarifies in-product metadata/content/audit ownership and adds a Unicode/space-containing ROM-path regression. Existing audit-store coverage rebases bundled audit provenance across randomized AppImage mount paths, and supervisor regression coverage preserves best-available classification semantics. AppImage execution of unusual paths is still reserved for full desktop qualification.

## Phase 5 — Diagnose startup and frame failures separately

- [x] Preserve terminal session snapshots long enough for UI/diagnostics to read exit status, bounded stdout/stderr, argv, runtime identity, effective content paths, and machine identity.
- [x] Map missing/incorrect content, invalid paths/permissions, unsupported runtime, child crash, control-channel failure, and generic early exit to distinct stable diagnoses.
- [x] Replace the primary `CONTROL_CHANNEL_CLOSED` message with the underlying MAME startup cause where output supports classification; retain the raw runtime-control code in developer details.
- [x] Add separate no-first-frame and post-start stream-stall timeouts; frame metrics retain last sequence/dimensions/error state for diagnostics.
- [x] Ensure diagnostics export includes bounded/sanitized startup output and separate runtime, video, input, session, and effective content-path states.
- [x] Test app shutdown and retry after pre-ready failure and post-ready crash so a stale failed session cannot hide a new one. Regression coverage launches successfully after both classes of terminal failure, while supervisor-drop coverage reaps active children and closes gameplay resources.

**Exit gate:** Every failure in the spec produces a specific user-visible state, useful details, and a retry/return path; runtime ready is never misreported as video ready.

> **2026-10-09 Linux executable-busy remediation:** Tauri project run [37900948141](https://github.com/ekkus93/mame/actions/runs/37900948141) showed two intermittent fake-MAME test failures when the executable version probe returned Linux `ETXTBSY` (`Text file busy`, OS error 26), with 295 other Rust tests passing. Commits `71b3c07` and `d82dc7` add bounded retries only for that transient Linux spawn error during version probing and supervised launch, preserving other errors and failing after a bounded deadline if an executable stays writable. Regression tests cover both the released-writer success path and the persistent-busy failure path. Exact-head Tauri project [37906371297](https://github.com/ekkus93/mame/actions/runs/37906371297) has **passed its `linux-quality` job**, including Rust format and Rust tests. This is automated launch resilience; it is not proof of working real-ROM video/audio or full-AppImage gameplay.

> **2026-10-08 retry evidence:** commit `7ff890680d060ea0f1ac9cde51f91850f4b468e7` added pre-ready-failure and post-ready-crash relaunch regressions. The subsequent corrected binary head `ad605d64145d9bcca547bc0b689fe244c17b2744` completed the `linux-quality` job successfully in Tauri project run `37842790331`, including Rust tests.

## Phase 6 — Automated verification and full AppImage qualification

- [x] Add Rust unit/integration tests for frame protocol validation, caps, session binding, ordering, backpressure, disconnect, timeout, and cleanup.
- [x] Add TypeScript tests for canvas conversion/pixel format, sizing/aspect/rotation, sequence/stale-frame handling, loading/stalled/error states, and input ownership.
- [x] Add a supervisor integration test using a deterministic fake frame producer and child process, plus a bundled-MAME smoke test where the environment permits.
- [x] Run frontend lint, typecheck, build, and all Vitest tests; run Rust formatting, clippy where project CI requires it, and all Rust tests.
- [x] Rebuild and qualify the full Linux AppImage after the canvas-rendering performance changes. Exact-source [run 37969476682](https://github.com/ekkus93/mame/actions/runs/37969476682) **passed both jobs** at binary commit `b7c12a82e4d412305dc323168b221970dc99acaa`. The real MAME 0.289 Debian/AppImage packages, headless frame capture, metadata bootstrap, provenance manifest, and SHA-pinned [artifact 11636880295](https://github.com/ekkus93/mame/actions/runs/37969476682/artifacts/11636880295) were produced successfully. The earlier passing artifact at `3da835f8` is historical; desktop real-ROM gameplay remains separately unchecked.

> **2026-10-09 CI hardening (qualification pending):** The real-MAME compile and full AppImage package/qualification phases now use separate dependent jobs, passing the exact built executable through a SHA-named Actions artifact with checksum verification. This prevents a nearly six-hour native compile from consuming the entire package job's GitHub-hosted time budget and allows the compiler cache to save independently of package failures. Headless smoke and frame-seam timeouts now include a bounded SIGKILL grace period to avoid indefinitely stuck MAME descendants. Neither the job split nor a successful headless smoke proves interactive gameplay; keep this item unchecked until the complete full-runtime package artifact is built and inspected.

> **2026-10-09 exact-source CI reconciliation:** Binary head `d82dc7f28c79f458a39742a08bfbdd1538f50261` passed Tauri project [37906371297](https://github.com/ekkus93/mame/actions/runs/37906371297), ordinary Linux packaging [37906371306](https://github.com/ekkus93/mame/actions/runs/37906371306), Windows packaging [37906371325](https://github.com/ekkus93/mame/actions/runs/37906371325), macOS packaging [37906371277](https://github.com/ekkus93/mame/actions/runs/37906371277), and security [37906371337](https://github.com/ekkus93/mame/actions/runs/37906371337). Full real-MAME build/package run [37906371265](https://github.com/ekkus93/mame/actions/runs/37906371265) remains a **separate in-progress acceptance gate**; do not mark the full-AppImage row done until the actual package and its provenance manifest exist and pass qualification. A documentation-only TODO change does not modify the binary-under-test SHA.

> **Release-tag scope warning:** Existing `v0.1.1` resolves to historical source `4cdd0372632a4c4132a79c12c028773e1b2d8e68`, not this qualified branch head; the historical tagged full-runtime run [37072502489](https://github.com/ekkus93/mame/actions/runs/37072502489) failed. A successful [37907074417](https://github.com/ekkus93/mame/actions/runs/37907074417) “start-release” workflow must **not** be treated as publication or qualification of the corrected binary. Do not move or reuse that tag automatically; release promotion requires a distinct, source-pinned tag/decision after full-runtime and desktop acceptance.


- [ ] Launch that AppImage from a real desktop session with the user's default ROM path and verify the game renders in the Tauri window, with no separate visible MAME window.
- [ ] Capture evidence for frame rate, frame age, dropped frames, emulation speed, CPU/memory, input, sound, resize, fullscreen, stop, and relaunch.
- [ ] Record exact AppImage path/hash, build configuration, MAME version, machine short name, ROM path configuration, test result, known limitations, and screenshots/logs in a dated qualification note.
- [ ] Rebuild and repeat qualification after fixes to launch, frame transport, pixel conversion, packaging, or runtime options; do not validate a stale artifact.


> **2026-10-09 package-gate closure (not desktop gameplay acceptance):** full-runtime [run 37923443872](https://github.com/ekkus93/mame/actions/runs/37923443872) succeeded on binary source `3da835f8cb7d15910f363448d4d0bc95d1456834`. It validated/reused the exact real MAME executable, passed `___empty` headless capture and repeated frame-seam tests, packaged and qualified an actual Debian package and **full AppImage**, and uploaded the package/provenance artifact [11614039521](https://github.com/ekkus93/mame/actions/runs/37923443872/artifacts/11614039521). AppImage filename: `MAME Tauri Frontend_0.1.1_amd64.AppImage`; CI build path: `tauri/src-tauri/target/release/bundle/appimage/MAME Tauri Frontend_0.1.1_amd64.AppImage`; SHA-256: `0dee88f12cd09c8468ebb70f0c1bcd533a24fb1cf186b243a5fc47f19fcfa835`; MAME version: `0.289 (unknown)`. Build: `npm run tauri -- build --config src-tauri/tauri.linux-bundle.conf.json --bundles deb,appimage`. The documented CI artifact expires **2026-10-12 11:23:41 UTC** and must be saved for host qualification. No installed machine from `/home/phil/mame/roms`, real WebView video/audio, keyboard/gamepad, performance, or first-run desktop acceptance has been demonstrated by this CI. The old `v0.1.1` tag remains a different historical source. See `docs/MAME_TAURI_IN_APP_GAMEPLAY_PACKAGE_EVIDENCE_2026-10-09.md`.

**Exit gate:** All automated checks pass and the exact full AppImage passes the desktop gameplay acceptance criteria in the spec.


> **2026-10-08 reconciliation evidence:** exact-head `e5dda4ece55c8491a702ac20cf7cd449fc1be5e6` completed the `linux-quality` job successfully in Tauri project run `37838923882`. That job ran frontend formatting, lint, typecheck, Vitest, production build, Rust formatting, Rust tests, ignored performance qualification, and clippy. Source coverage includes authenticated frame-protocol/mailbox tests, teardown/backpressure tests, the deterministic fake-frame-producer supervisor integration test, TypeScript frame conversion/parser tests, gameplay input tests, and game-surface sizing/sequence/stall-state tests. README and the 2026-09-08 architecture baseline now describe the Linux little-endian in-app canvas/frame-data-plane architecture, ROM defaults/path behavior, and known renderer/platform gaps. Full real-ROM desktop AppImage qualification remains open and is not implied by these checks.

> **2026-10-09 canvas performance follow-up (automated qualification passed):** The staging/WebView canvas backing stores now remain stable on unchanged-size frames, and nearest/smooth scaling changes no longer restart frame polling. Unit tests verify stable dimensions and dynamic/rotated resizing. Exact-source Tauri project [37969476609](https://github.com/ekkus93/mame/actions/runs/37969476609) passed frontend checks, tests, and Linux release qualification; standard Linux [37969476754](https://github.com/ekkus93/mame/actions/runs/37969476754), Windows [37969476591](https://github.com/ekkus93/mame/actions/runs/37969476591), macOS [37969476753](https://github.com/ekkus93/mame/actions/runs/37969476753), security [37969476741](https://github.com/ekkus93/mame/actions/runs/37969476741), and full real-runtime package [37969476682](https://github.com/ekkus93/mame/actions/runs/37969476682) all passed at binary commit `b7c12a82e4d412305dc323168b221970dc99acaa`. This does **not** establish real-ROM, input/audio, or sustained 60 Hz desktop acceptance; keep those items unchecked.

## Phase 7 — Documentation and release readiness

- [x] Update `README.md`, the 2026-09-08 architecture baseline, and any relevant runtime-control/performance docs to describe the in-app TypeScript canvas and the separate binary frame data plane accurately.
- [x] Remove or revise statements that gameplay video must not pass through Tauri or that the external MAME window is the intended product experience.
- [x] Document initial renderer scope and known gaps (BGFX effects, artwork/bezel compositing, vector effects, multi-screen, external runtime compatibility).
- [x] Document ROM defaults, path precedence, audit freshness, and the distinction between catalog metadata, ROM auditing, and obtaining ROM files.
- [x] Link this spec/TODO from `docs/README.md` and update checkboxes only when implementation and evidence exist.
- [ ] Perform a user-facing review using a normal first-run profile, not a developer database with warm audit/catalog state.

**Exit gate:** User-facing and engineering docs agree with actual behavior, and all remaining limitations are explicit and acceptable for the first supported release.

> **2026-10-09 qualification-harness integrity improvement (not yet requalified):** The Lua diagnostic frame-seam report will mark zero collected `screen:pixels()` samples as `not_measured`, not `ok`; valid Lua success requires at least one sample. The fresh-profile wrapper isolates `XDG_STATE_HOME` alongside HOME, config, data, and cache. These are evidence-integrity improvements, not proof of real-ROM video, audio, rotation, or performance. Re-run exact-source CI for this script revision before relying on its results.

> **2026-10-08 fresh-profile harness:** `scripts/tauri/qualify-in-app-gameplay-fresh-profile.sh` isolates HOME/XDG state and exposes the selected ROM directory through the normal `~/mame/roms` convention. The Phase 7 first-run review remains unchecked until a real desktop operator records the UI observations and diagnostics from the full AppImage.

## Definition of done

Every phase exit gate has evidence. Passing unit tests or seeing a MAME child process alive is not sufficient: the required proof is a playable machine rendered by the TypeScript canvas in the full AppImage, using the user's normal ROM directory and controls, with useful diagnostics for negative cases.

