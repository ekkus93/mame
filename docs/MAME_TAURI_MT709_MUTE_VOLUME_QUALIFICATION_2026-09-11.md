# MT-709 — Mute/volume control qualification

**Date:** 2026-09-11  
**Branch:** `ralph/mt-709-mute-volume`  
**Qualified implementation SHA:** `af45f1b9e0d764dde5f53ba2e9769931d34c22a6`  
**Authoritative Tauri CI:** `34641307034` — PASS

## Supported operations

MT-709 deliberately supports **user mute** and does not claim live volume support on the exact MAME revision in this repository.

The fork's Lua binding in `src/frontend/mame/luaengine.cpp` exposes `manager.machine.sound.ui_mute` as a read/write property and `manager.machine.sound.muted` as the effective read-only mute state. The C++ sound manager also has a live `set_master_gain` API, but this repository revision does not expose that setter through Lua. Therefore protocol v1 advertises `set_mute` only. `set_volume` remains a known protocol-v1 command name but is not advertised by the running shim and is rejected as unsupported rather than routed through a generic Lua evaluator or a new MAME-core binding.

The supported mute operation writes only `manager.machine.sound.ui_mute`. Its synchronous completion result returns both:

- `uiMuted` — the user-controlled UI mute bit;
- `effectiveMuted` — MAME's aggregate mute state, which can also be true for pause/debugger/system mute reasons.

Rust verifies that the observed `uiMuted` value matches the requested value before reporting success.

## No PCM through Tauri

MT-709 changes control state only. It does not capture, copy, resample, encode, forward, or otherwise route audio samples through Rust, Tauri IPC, or the WebView. The generated runtime-control shim regression test asserts that the MT-709 path contains the native mute properties and does not introduce sample/update hooks such as `get_samples`, `register_sound_update`, or equivalent audio-stream transport.

Gameplay audio therefore remains on MAME's native low-latency audio path.

## Typed surface

Rust/Tauri:

- `SetMameMuteRequest { session_id, muted }`
- `SetMameMuteResult { schema_version, session_id, ui_muted, effective_muted }`
- command `set_mame_mute`

TypeScript:

- `SetMameMuteRequest`
- `SetMameMuteResult`
- `setMameMute()`

The authenticated request uses the existing serialized request gate, bounded protocol envelope, response timeout, and session-scoped control channel. The operation is synchronous at the Lua property boundary; an asynchronous `accepted`/completion sequence is rejected rather than treated as success.

## Exact-head evidence

Tauri CI run `34641307034` passed on exact SHA `af45f1b9e0d764dde5f53ba2e9769931d34c22a6`.

Passed gates include frontend format/lint/typecheck/tests/build, Rust format/tests, library UX performance qualification, Clippy with warnings denied, and lockfile integrity.

An earlier backend-only SHA `3075359ca5a201e194ef577c3061b6286f03ab56` also passed Tauri CI run `34640741014`. The qualified SHA adds the typed frontend request/result wrapper and invocation regression.

## MT-709 acceptance mapping

- **define supported operations** — satisfied: native user mute is supported; live volume is explicitly unsupported on this exact MAME Lua binding rather than emulated or overclaimed.
- **do not route PCM through Tauri** — satisfied: MT-709 changes only MAME's native UI-mute property and leaves the audio stream entirely inside MAME/OSD.
