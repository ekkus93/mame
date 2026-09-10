# MT-701 — MAME Runtime-Control Options Characterization

**Date:** 2026-09-10  
**Repository:** `ekkus93/mame`  
**Task:** MT-701 — Characterize MAME control options  
**Evidence baseline:** MT-608 closure `80880d785907dff9213a57265e7c2443e5091570`  
**Upstream MAME version in tree:** 0.289

## Executive decision

For the sidecar architecture, use a **parent-owned anonymous stdin/stdout pipe transport through MAME's existing Lua console** as the DG-3 baseline for protocol v1.

The Tauri/Rust supervisor should launch MAME with `-console`, pipe the child's stdin, retain the existing piped stdout/stderr capture, and load a project-owned Lua control shim. Rust is the only process that receives the writable stdin handle. The Lua shim adapts the project's narrow framed protocol to MAME's Lua API and emits bounded, uniquely framed responses/events on stdout. Ordinary MAME stdout remains diagnostic data.

This gives the project a session-scoped local IPC channel without opening a TCP port, creating a globally named endpoint, or patching MAME merely to establish transport. It also matches the existing `std::process::Command` supervision model.

No upstream MAME C++ change is required for transport, pause/resume, reset, clean exit, mute, or basic status. Two later capability gaps must remain explicit:

1. `machine:save()` and `machine:load()` schedule operations but do not expose a reliable success/failure completion contract to Lua. MT-707/MT-708 therefore require a narrow MAME-side completion hook unless upstream gains an equivalent API first.
2. MAME 0.289 documentation describes a writable `sound.volume`, but this source tree's Lua binding exposes mute properties without the documented volume property. Native `sound_manager::master_gain()`/`set_master_gain()` exists, so MT-709 must re-qualify the API and add only a narrow Lua binding if it is still absent.

DG-3 should be revisited only if redirected `-console` input fails a supported-platform end-to-end smoke test or upstream MAME gains a purpose-built local control endpoint with equal or better isolation, lifecycle, and maintenance properties.

---

## 1. Required product operations

| Product operation | Existing MAME capability | Completion/event capability | MAME-side change needed? |
| --- | --- | --- | --- |
| pause | `emu.pause()` | pause notifier | no |
| resume | `emu.unpause()` | resume notifier | no |
| soft reset | `manager.machine:soft_reset()` | reset notifier | no |
| hard reset | `manager.machine:hard_reset()` | new-session/reset lifecycle observable | no for command; semantics must be specified |
| clean exit | `manager.machine:exit()` | stop notifier plus supervised process exit | no |
| save state | `manager.machine:save(filename)` | pre-save notification, but no reliable completion result | **yes for explicit success/failure completion** |
| load state | `manager.machine:load(filename)` | post-load notification exists, but no failure/result contract | **yes for explicit success/failure completion** |
| mute | `manager.machine.sound.ui_mute` read/write | state can be queried | no |
| volume | native master-gain setter exists; documented Lua `sound.volume` is absent in the inspected 0.289 binding | no qualified Lua contract | **likely narrow binding if still absent at MT-709** |
| query state | machine identity and properties including paused/exit/reset state | direct query | no |

The native API surface is therefore sufficient to keep runtime management out of the real-time video/audio/input path. The principal missing pieces are a project-owned transport/protocol contract and explicit save/load completion semantics.

---

## 2. Existing MAME mechanisms

### 2.1 Lua console over stdin/stdout — suitable and preferred

MAME has an interactive Lua console enabled by `-console`. It executes in the Lua environment that exposes `manager.machine`, session-control methods, sound state, and event notifiers.

Source evidence:

- `src/emu/emuopts.cpp` defines `-console`, `-autoboot_script`, plugin options, HTTP options, and `comm_*` options.
- `plugins/console/init.lua` runs the console reader through `emu.thread()` and evaluates complete commands read from the input stream.
- `3rdparty/linenoise/linenoise.c` falls back to line-oriented `stdin` reads when raw terminal mode cannot be enabled, including non-TTY redirected input.
- `3rdparty/linenoise/linenoise-win32.c` similarly cannot establish console mode for redirected/non-console input, allowing the common line-input fallback.
- `src/frontend/mame/luaengine.cpp` exposes the session-control calls and notifiers required by MT-704 onward.

The current Tauri supervisor already owns the child and already pipes stdout/stderr. Establishing the control channel requires changing child stdin from `Stdio::null()` to `Stdio::piped()` and retaining the `ChildStdin` handle in the managed session.

The product protocol must **not** be arbitrary Lua. MT-702 defines a typed, bounded command protocol. Trusted Rust should generate only the narrow Lua-shim invocation required to deliver a validated protocol request. The WebView must never receive the raw stdin handle and must never submit Lua/code fragments.

Because stdout is shared with console/diagnostic output, protocol records must use an exact, unpredictable per-session frame marker and bounded single-line serialization. Rust consumes only exact frames and leaves all other stdout in the diagnostic path.

### 2.2 Autoboot script / Lua plugin — suitable in-MAME adapter

MAME can run a script with `-autoboot_script` and supports Lua plugins. A project-owned shim can install the protocol adapter and event subscriptions without adding a broad C++ control server.

The shim can:

- register pause/resume/reset/stop notifiers;
- inspect bounded machine/session state;
- invoke pause/resume/reset/exit/save/load/mute operations;
- serialize bounded result/event records using the shipped Lua JSON support;
- emit uniquely framed protocol records to stdout.

This is project code executed by MAME, but it does not require a fork patch merely to establish v1 transport.

### 2.3 Built-in HTTP/WebSocket server — technically capable, unsuitable as-is

MAME has an HTTP server enabled by `-http`, with `-http_port` and `-http_root`. The underlying server can host request and WebSocket handlers, and the current machine exports informational HTTP state.

It is not acceptable for MT-703 without additional MAME work:

- the CLI exposes a port but no bind-address option;
- `src/emu/http.cpp` sets the port but not the server address;
- the HTTP implementation's empty address binds an IPv4 any-address endpoint;
- there is no project session-scoped authentication token in the built-in API;
- the existing API is not the required runtime-control protocol;
- port selection introduces allocation, race, firewall, and endpoint-reuse concerns avoided by anonymous pipes.

A loopback-only authenticated HTTP/WebSocket API could be implemented, but that adds MAME patch surface solely to recreate properties the inherited pipe already gives us.

### 2.4 `comm_*` options — not a host management API

`comm_localhost`, `comm_localport`, `comm_remotehost`, `comm_remoteport`, and `comm_framesync` support emulated linked-machine communication. They are guest/emulated-system communication facilities, not a host API for pause, reset, save/load, exit, or status. They must not be repurposed for MT-700.

### 2.5 Debugger/GDB interfaces — wrong privilege and semantic boundary

MAME's debugger can perform some overlapping operations, and remote debugger facilities exist. They expose substantially more power than this frontend requires, have debugger-specific execution semantics, and are not a stable product-management protocol. The GDB stub is also target-limited. Enabling debugger infrastructure just for runtime controls would expand privilege and attack surface without product value.

### 2.6 OS process controls — shutdown fallback only

The existing supervisor already has platform termination/escalation behavior. It remains necessary when the control channel is unavailable, but it cannot implement the MT-700 command set and provides no structured command acknowledgement/completion model.

After MT-706, protocol clean exit should be attempted first and the current OS termination path should remain a bounded fallback.

### 2.7 Synthesized keyboard/UI input — reject

Sending pause/reset/save/load hotkeys is focus-dependent, user-mapping-dependent, and conflicts with the project's gameplay-input ownership invariant. It cannot produce reliable structured acknowledgements. It is not a control protocol.

---

## 3. IPC transport comparison

| Transport | Cross-platform | Isolation/security | MAME patch surface | Lifecycle | Assessment |
| --- | --- | --- | --- | --- | --- |
| **anonymous stdin/stdout pipes + Lua console** | yes | excellent: inherited parent/child handles, no named listener | none for transport | naturally session-scoped; EOF/exit tears down | **preferred v1** |
| Unix-domain socket | Linux/macOS strong; Windows semantics differ from native named pipes | strong with filesystem/peer permissions | socket implementation required | explicit path cleanup | good native option, unnecessary for v1 |
| Windows named pipe + Unix socket abstraction | yes with two backends | strong/local | platform-specific endpoint implementation | explicit creation/cleanup | strong fallback but more maintenance |
| loopback TCP | yes | requires strict bind plus authentication | endpoint implementation needed | port allocation/reuse management | workable but weaker default isolation |
| built-in HTTP/WebSocket | yes | unacceptable as-is due listener/auth model | patch required | explicit server lifecycle | reject for v1 |
| POSIX FIFO | not symmetric across supported platforms | filesystem permissions/name exposure | adapter required | stale-path cleanup | inferior to anonymous pipes |
| OS signals | platform-specific command subset | process-scoped | none | tied to process | shutdown escalation only |
| debugger/GDB | not uniformly useful | excessive privilege | none/varies | debugger-specific | reject |
| synthesized input | nominally cross-platform | focus/input-map dependent | none | no protocol lifecycle | reject |

### Why anonymous pipes win DG-3

1. **Natural session scoping.** The endpoint exists only as handles attached to the launched child/parent pair.
2. **No ambient network listener.** No LAN or loopback service is exposed and no firewall surface is created.
3. **No endpoint-name race.** There is no socket pathname or TCP port to predict, squat, reuse, or clean up.
4. **Cross-platform host support.** Rust's child-process stdin/stdout piping exists on Linux, macOS, and Windows.
5. **Minimal MAME coupling.** Existing MAME console/Lua mechanisms supply the transport adapter.
6. **Fits existing architecture.** Rust already supervises MAME and consumes bounded stdout/stderr.
7. **Sufficient bandwidth.** MT-700 is a low-rate management plane; it explicitly excludes video, PCM, and raw gameplay input.

The primary authorization boundary is possession of the inherited anonymous pipe handle by the trusted parent. A per-session frame token is still required for robust stdout discrimination and defense in depth; it is not a substitute for transport isolation.

---

## 4. Minimal implementation changes implied by the characterization

### 4.1 Tauri/Rust side

MT-702/MT-703 should later add a control-channel object owned by the current managed session. At minimum it will:

- spawn MAME with piped stdin instead of null stdin;
- retain the child stdin writer;
- add `-console` and load the project Lua shim through the chosen script/plugin mechanism;
- generate an unpredictable per-session framing token;
- serialize only typed MT-702 requests;
- cap request size before writing;
- parse stdout incrementally because pipe reads may split/coalesce logical lines;
- recognize protocol records only when the exact session frame marker is present;
- route non-protocol stdout to existing diagnostics unchanged;
- correlate requests/results with request IDs;
- surface EOF/broken pipe/timeouts as explicit control-channel failures;
- drop the writer during teardown and preserve existing kill escalation.

The frontend remains behind typed Tauri commands. It does not get a generic "send Lua" or "send protocol bytes" primitive.

### 4.2 Project-owned Lua shim

The shim should be deliberately small. It should:

- verify protocol version and bounded command envelope;
- dispatch only the allowed command enum;
- never evaluate user-provided code;
- produce one bounded response for each accepted/rejected request;
- emit relevant lifecycle events from MAME notifiers;
- ensure callback/subscription teardown follows session teardown;
- avoid exposing arbitrary file access; save/load path semantics belong to MT-707/MT-708 and must be scoped by Rust.

### 4.3 Minimal MAME-side changes if later required

#### Save/load completion

`machine:save(filename)` and `machine:load(filename)` are asynchronous scheduling calls. The inspected Lua binding itself notes the absence of a completion notification. MAME's internal save/load path does obtain a result, but that result is currently surfaced through human-facing behavior rather than a stable Lua result event.

MT-707/MT-708 explicitly require success/failure completion. Do not infer that from a file appearing, a popup string, or elapsed time. The minimal acceptable change is a generic Lua-visible save/load completion notification carrying enough information to distinguish operation, success/failure, and the requested state target/correlation context.

#### Runtime volume

The 0.289 documentation describes a writable Lua volume property, but the inspected binding does not expose it while the native sound manager does expose master gain. MT-709 should first re-check the then-current upstream source/API. If still absent, add only a narrow master-gain/dB Lua binding with bounded semantics. PCM remains entirely in MAME's native audio path.

No broader MAME runtime-control server is justified by these two API gaps.

---

## 5. Platform implications

### Linux

Anonymous child stdin/stdout pipes are directly supported by Rust. Redirected input is non-TTY, and MAME's linenoise path falls back to line input. No Unix-socket path permissions, cleanup, or port allocation is required.

If a dedicated local endpoint ever becomes necessary, a Unix-domain socket is preferable to loopback TCP, but pathname limits, ownership/mode, and stale-path cleanup become product responsibilities.

### macOS

The same anonymous-pipe model applies. It avoids a network listener and filesystem socket lifecycle. Future app-sandbox/hardened-runtime packaging must already allow launching the selected MAME sidecar; inherited standard handles do not introduce a separate network endpoint.

### Windows

Rust child-process pipes provide the required anonymous endpoint without a globally named object. MAME's Windows console input path cannot establish normal console input against redirected pipe input and falls back to line-oriented stdin handling.

If a future dedicated endpoint is required, Windows named pipes are preferable to forcing a TCP listener. Named-pipe ACLs can provide strong local authorization, but they introduce endpoint naming, ACL, and lifecycle code not needed for v1.

### Framing, encoding, and shutdown

- Protocol payloads should be UTF-8 JSON with a strict size cap and LF-delimited transport framing.
- Parsing must be incremental; pipe byte chunks are not message boundaries.
- Child exit closes its output pipes and invalidates the parent writer.
- A broken stdin pipe is a concrete channel failure, not a reason to silently synthesize keyboard input.
- Parent/app teardown drops the control writer and then uses existing bounded process shutdown behavior as needed.
- Once MT-706 exists, protocol exit is the preferred first shutdown step.

---

## 6. Security/failure requirements inherited by MT-702/MT-703

The transport removes network exposure but does not eliminate protocol hardening. MT-702/MT-703 must require:

- explicit protocol version;
- request ID;
- fixed command enum;
- bounded line/message size before JSON parsing;
- bounded strings/paths after parsing;
- exact session ownership/matching in Rust;
- one authoritative writer owned by the supervisor;
- no frontend-provided Lua/code fragments;
- unpredictable per-session output-frame marker/token;
- deterministic timeout semantics;
- explicit EOF/broken-pipe handling;
- malformed/unknown-command rejection;
- no fallback to synthesized input or an unauthenticated listener.

---

## 7. Rejected alternatives

**Built-in HTTP/WebSocket as-is:** rejected because the listener is not scoped/authenticated to this session and the built-in API is not the control protocol.

**New TCP control server:** rejected because it duplicates existing Lua control capability while adding bind/auth/port lifecycle work.

**Debugger/GDB:** rejected because it grants excessive control and has debugger-specific semantics/coverage.

**`comm_*`:** rejected because it is an emulated communications subsystem, not a host management API.

**UI hotkeys:** rejected because focus and user mappings make them nondeterministic and they cannot provide structured acknowledgements.

**Parsing popup/error text for save/load completion:** rejected because human-facing messages are not a stable API and would create silent false-success/false-failure paths.

---

## 8. DG-3 decision register

**Decision ID:** DG-3  
**Date:** 2026-09-10  
**Evidence repository SHA:** `80880d785907dff9213a57265e7c2443e5091570`  
**Question:** What transport should the Tauri host use for the sidecar runtime-control protocol?  
**Options evaluated:** anonymous stdio pipes/Lua console; Unix-domain sockets; Windows named pipes; loopback TCP; built-in HTTP/WebSocket; MAME `comm_*`; debugger/GDB; OS signals; synthesized input.  
**Evidence/prototypes:** source-level characterization of MAME 0.289 console/linenoise behavior, Lua bindings/notifiers, HTTP bind behavior, current Tauri supervisor stdio/process lifecycle, and official MAME 0.289 documentation.  
**Performance data:** no high-rate stream is transported; expected command/event volume is negligible relative to emulation. MT-700 excludes video/PCM/raw gameplay-input transport.  
**Cross-platform data:** redirected console input has fallback handling on POSIX and Windows; Rust anonymous child pipes are available on Linux, macOS, and Windows.  
**Upstream-maintenance impact:** no MAME C++ patch for transport; narrowly scoped API work remains for save/load completion and potentially master volume.  
**Security impact:** no named/network listener; endpoint access derives from inherited process handles.  
**Decision:** use parent-owned anonymous stdin/stdout pipes with `-console` plus a project-owned Lua protocol shim for protocol v1.  
**Rejected alternatives:** built-in HTTP/WebSocket as-is, new TCP server, debugger/GDB, `comm_*`, signals as primary transport, synthesized input.  
**Revisit conditions:** redirected MAME console input fails a supported-platform end-to-end smoke test; upstream changes the console incompatibly; or upstream adds a purpose-built local endpoint with equivalent isolation, lifecycle, and maintenance properties.

This decision applies to the external-sidecar architecture. It does not constrain the optional dedicated-OSD or in-process phases.

---

## 9. Evidence references

Repository source at the evidence baseline:

- `makefile` — MAME version 0.289.
- `src/emu/emuopts.cpp` / `src/emu/emuopts.h` — scripting, HTTP, and `comm_*` options.
- `plugins/console/init.lua` — console input/evaluation loop.
- `3rdparty/linenoise/linenoise.c` — non-TTY input fallback.
- `3rdparty/linenoise/linenoise-win32.c` — redirected/non-console Windows behavior.
- `src/frontend/mame/luaengine.cpp` — session-control bindings/notifiers, sound bindings, and save/load completion TODOs.
- `src/emu/machine.cpp` / `src/emu/machine.h` — session lifecycle and save/load execution.
- `src/emu/http.cpp` — HTTP server setup and machine API export.
- `src/lib/util/server_http_impl.hpp` — empty bind address resolves to an IPv4 any-address endpoint.
- `src/emu/sound.h` and MAME UI slider code — native master-gain API.
- `tauri/src-tauri/src/sessions/supervisor.rs` — current null stdin, piped stdout/stderr, and shutdown escalation.

Official MAME 0.289 documentation checked 2026-09-10:

- <https://docs.mamedev.org/commandline/commandline-all.html>
- <https://docs.mamedev.org/luascript/index.html>
- <https://docs.mamedev.org/luascript/ref-core.html>
- <https://docs.mamedev.org/plugins/console.html>
- <https://docs.mamedev.org/plugins/gdbstub.html>
- <https://docs.mamedev.org/debugger/general.html>

---

## 10. MT-701 acceptance mapping

- **Inventory existing mechanisms suitable for external control:** complete; Lua console/script/plugin, HTTP/WebSocket, debugger/GDB, `comm_*`, process controls, and input synthesis were characterized.
- **Identify minimal MAME-side changes if required:** complete; transport requires none, while save/load completion requires a narrow reliable completion notification and volume may require a narrow Lua binding.
- **Compare local socket/pipe/other IPC options:** complete; anonymous pipes, local sockets/named pipes, loopback TCP/HTTP, FIFOs, signals, and non-IPC input injection were compared.
- **Document platform implications:** complete for Linux, macOS, and Windows.

MT-701 is complete. MT-702 should define protocol v1 against the selected anonymous-pipe/Lua-shim transport without implementing runtime commands prematurely.
