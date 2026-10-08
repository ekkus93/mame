# MAME Tauri In-App Gameplay TODO

**Date:** 2026-10-08  
**Status:** Open; none of the implementation phases below are complete by virtue of this plan  
**Specification:** [MAME Tauri In-App Gameplay Specification](MAME_TAURI_IN_APP_GAMEPLAY_SPEC_2026-10-08.md)

This TODO replaces the former assumption that a supervised MAME process plus a separate SDL window constitutes playable Tauri gameplay. It tracks the complete path from discovered local ROMs through live video, controls, diagnostics, and validation in the full AppImage.

## Phase 0 — Establish a reproducible baseline

- [ ] Record current branch/head and preserve all pre-existing user changes.
- [ ] Record current full AppImage path, hash, bundled MAME version, and build command/configuration.
- [ ] Reproduce the current user's case with a machine from `/home/phil/mame/roms` and save the complete launch request, child exit code, bounded stdout/stderr, effective paths/argv, runtime-control transition, and UI state.
- [ ] Add a repeatable local launch/diagnostic recipe that does not depend on a screenshot or manually searching session logs.
- [ ] Determine whether recent `CONTROL_CHANNEL_CLOSED`/`MAME_EARLY_EXIT` reports are an independent startup defect, a consequence of launching with an incompatible display mode, or both. Keep the causes separate until evidence identifies them.
- [ ] Record baseline behavior for an available ROM machine, a best-available/no-ROM machine, and a known missing-content machine.

**Exit gate:** A developer can reproduce and identify each current failure from saved diagnostics, and knows exactly which full AppImage/runtime was tested.

## Phase 1 — Prove the frame capture and transport seam

- [ ] Build a small capture spike against the bundled MAME version. Verify the `screen:pixels()` return value, dimensions, byte order, alpha behavior, visible-area semantics, screen orientation, and callback timing on raster and vector machines.
- [ ] Measure Lua `screen:pixels()` plus callback overhead at representative native and high resolutions. Reject it for production if it misses the performance/stability budget.
- [ ] Compare candidate production seams: Lua callback plus dedicated binary channel, a narrow MAME-side capture adapter/OSD module, and other supported MAME rendering hooks. Do not use JSON/base64 events, console text, or per-frame temporary files as the production data plane.
- [ ] Define a versioned frame header: session ID, sequence, dimensions, pixel format, stride, payload size, capture timestamp, orientation/rotation, and flags.
- [ ] Choose and document a cross-platform transport strategy or explicitly constrain the first release to Linux AppImage. Transport must be private, authenticated/session-scoped, bounded, and teardown-safe.
- [ ] Prototype one-slot/latest-frame or bounded-ring semantics and prove stale frames are replaced instead of queued.
- [ ] Verify `-video none`/headless operation still makes valid machine frames available through the selected seam and does not block normal MAME execution.
- [ ] Verify current stdout runtime-control parsing is unaffected; frame bytes must not share the control/diagnostic stream.

**Exit gate:** A real MAME process delivers validated live pixels to a minimal canvas prototype at target performance without a visible SDL game window, with measured overhead and a documented transport decision.

## Phase 2 — Implement Rust frame host and TypeScript game surface

- [ ] Add a session-owned frame endpoint/mailbox to the Rust supervisor and tie its lifetime to the exact MAME child/session.
- [ ] Add strict dimension/payload limits, protocol-version checks, sequence checks, stale-session rejection, and explicit errors for malformed or unsupported frames.
- [ ] Add backpressure/latest-frame behavior and metrics for received, dropped, and presented frames, capture/presentation time, age, and stream stalls.
- [ ] Expose frames to the WebView via the selected typed, low-overhead API without a generic socket/filesystem capability.
- [ ] Implement a React `GameSurface` with a TypeScript-owned canvas. Draw raw machine frames; resize backing resolution correctly; preserve aspect ratio; implement nearest/integer and smooth scaling modes where supported.
- [ ] Support dynamic screen dimensions, rotation, and multiple screens according to the selected initial scope. Reject or clearly explain unsupported topologies.
- [ ] Add loading, first-frame, active, stalled, unsupported, stopping, and ended states. Require both runtime-ready and first-frame-presented before reporting gameplay as ready.
- [ ] Add fullscreen, return/stop, pause/resume, reset, mute/volume controls as available, and accessible focus indicators.
- [ ] Ensure exiting a game, closing the app, MAME crash, timeout, component unmount, and failed startup release all resources and cannot leave orphaned children or transport endpoints.

**Exit gate:** A real game frame is drawn by the frontend canvas in the main Tauri window and frame stream lifecycle survives stop/relaunch/failure without leaking or displaying stale frames.

## Phase 3 — Input, audio, and normal gameplay behavior

- [ ] Design and implement keyboard input forwarding into MAME's normal input model while the WebView owns focus; do not route key events through the video event queue.
- [ ] Implement controller/gamepad forwarding for at least one supported controller path, including axes/buttons and configurable mapping compatibility.
- [ ] Balance all key/button down/up transitions on blur, fullscreen changes, stop, process exit, and app shutdown; test for stuck keys and duplicate browser shortcuts.
- [ ] Coalesce high-rate axis/button updates and ensure input traffic cannot block video or runtime control.
- [ ] Keep MAME's native audio output functioning with video hidden. Verify mute/volume UI reflects actual runtime state and does not imply WebView PCM support.
- [ ] Test keyboard navigation returns correctly to the library after gameplay ends and does not consume game input while gameplay owns focus.
- [ ] Test a representative raster game, a vector game, a game with rotation, and a machine with a different resolution/aspect ratio.

**Exit gate:** A user can play a representative game with keyboard and gamepad from the Tauri game surface, hear normal sound, and reliably return to the library without stuck input.

## Phase 4 — Fix ROM discovery, audit, and launch parity

- [ ] Verify how the original installed MAME resolves the user's actual ROM path and effective `rompath`; capture evidence from the same host/configuration.
- [ ] Make `~/mame/roms` discoverable by default for the normal Linux user profile while respecting configured paths, MAME config, and documented precedence. Never hardcode the developer's home directory.
- [ ] Ensure configured parent path `/home/<user>/mame` correctly resolves its `roms` subdirectory when that is the supported convention; avoid duplicating or hiding paths in the UI.
- [ ] Use one effective-path computation for metadata/audit availability and actual game launch. Show the effective paths and validation/readability status in settings/diagnostics.
- [ ] Separate catalog metadata refresh, content audit, and ROM installation in UI copy. Do not require metadata import to discover local ROMs, and do not imply the app can acquire ROM content.
- [ ] Keep bundled MAME audit identity stable across random AppImage mount locations; verify migration/rebase of existing audit data and relaunch in a different mount path.
- [ ] Verify complete and best-available sets are reported accurately; do not claim that a best-available result has local files when none are required/present. Distinguish driver support from content availability.
- [ ] Gate launch or provide an explicit audit path for Unknown, stale, and unavailable content. Preserve a deliberate override only if the UX explains the risk and its result is diagnosable.
- [ ] Test paths with spaces, non-ASCII names, multiple directories, missing directories, permissions errors, and AppImage execution.

**Exit gate:** The user's default ROM folder appears after a normal launch, audit and Start use identical paths, and a valid locally available machine can reach the first-frame-ready state without manual catalog import.

## Phase 5 — Diagnose startup and frame failures separately

- [ ] Preserve terminal session snapshots long enough for the UI and diagnostics panel to read exit code, stdout/stderr tails, argv, runtime identity, content paths, and selected machine even when there is no active session.
- [ ] Map missing ROM/CHD/BIOS, invalid paths, permissions, unsupported renderer/runtime, child crash, control-channel failure, and generic early exit to distinct user-facing diagnoses.
- [ ] Replace the primary `CONTROL_CHANNEL_CLOSED` message with the underlying MAME startup cause where output supports a classification; retain raw protocol codes in developer details.
- [ ] Add a separate no-first-frame timeout and post-start stream-stall timeout, with last valid frame metadata and actionable recovery.
- [ ] Ensure diagnostics export includes the bounded/sanitized startup output and separate runtime, video, input, and content states.
- [ ] Test app shutdown and retry after each pre-ready and post-ready failure so a stale failed session cannot hide a new one.

**Exit gate:** Every failure in the spec produces a specific user-visible state, useful details, and a retry/return path; runtime ready is never misreported as video ready.

## Phase 6 — Automated verification and full AppImage qualification

- [ ] Add Rust unit/integration tests for frame protocol validation, caps, session binding, ordering, backpressure, disconnect, timeout, and cleanup.
- [ ] Add TypeScript tests for canvas conversion/pixel format, sizing/aspect/rotation, sequence/stale-frame handling, loading/stalled/error states, and input ownership.
- [ ] Add a supervisor integration test using a deterministic fake frame producer and child process, plus a bundled-MAME smoke test where the environment permits.
- [ ] Run frontend lint, typecheck, build, and all Vitest tests; run Rust formatting, clippy where project CI requires it, and all Rust tests.
- [ ] Build the actual full Linux AppImage after source changes. Keep and inspect the full-runtime artifact; do not substitute a slim image or test only the unpacked AppDir.
- [ ] Launch that AppImage from a real desktop session with the user's default ROM path and verify the game renders in the Tauri window, with no separate visible MAME window.
- [ ] Capture evidence for frame rate, frame age, dropped frames, emulation speed, CPU/memory, input, sound, resize, fullscreen, stop, and relaunch.
- [ ] Record exact AppImage path/hash, build configuration, MAME version, machine short name, ROM path configuration, test result, known limitations, and screenshots/logs in a dated qualification note.
- [ ] Rebuild and repeat qualification after fixes to launch, frame transport, pixel conversion, packaging, or runtime options; do not validate a stale artifact.

**Exit gate:** All automated checks pass and the exact full AppImage passes the desktop gameplay acceptance criteria in the spec.

## Phase 7 — Documentation and release readiness

- [ ] Update `README.md`, the 2026-09-08 architecture baseline, and any relevant runtime-control/performance docs to describe the in-app TypeScript canvas and the separate binary frame data plane accurately.
- [ ] Remove or revise statements that gameplay video must not pass through Tauri or that the external MAME window is the intended product experience.
- [ ] Document initial renderer scope and known gaps (BGFX effects, artwork/bezel compositing, vector effects, multi-screen, external runtime compatibility).
- [ ] Document ROM defaults, path precedence, audit freshness, and the distinction between catalog metadata, ROM auditing, and obtaining ROM files.
- [ ] Link this spec/TODO from `docs/README.md` and update checkboxes only when implementation and evidence exist.
- [ ] Perform a user-facing review using a normal first-run profile, not a developer database with warm audit/catalog state.

**Exit gate:** User-facing and engineering docs agree with actual behavior, and all remaining limitations are explicit and acceptable for the first supported release.

## Definition of done

Every phase exit gate has evidence. Passing unit tests or seeing a MAME child process alive is not sufficient: the required proof is a playable machine rendered by the TypeScript canvas in the full AppImage, using the user's normal ROM directory and controls, with useful diagnostics for negative cases.

