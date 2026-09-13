# MT-2200 Engineering-Phase Closure and Maintenance Handoff

**Date:** 2026-09-13  
**Repository:** `ekkus93/mame`  
**Closure branch:** `ralph/mt-2200-engineering-closure`  
**Pre-closure master SHA:** `f4a453956e3af9ba9b5ee91124b8433e523f4605`  
**Planning/upstream merge-base baseline:** `7cc3033a50b00801240f020b7098e22ffffdcd44`

## 1. Closure statement

MT-2200 closes the engineering phase for the **production-useful external-window Tauri frontend**. It does not claim that the optional native-window embedding, dedicated Tauri OSD, or in-process MAME tracks are complete.

The completed product boundary is the MT-2004/MT-2100 architecture: React/TypeScript provides the user-facing library and configuration experience; Rust owns trusted filesystem/process/database/runtime-control authority; MAME remains a separately supervised native process and owns emulation, rendering, audio, and gameplay input.

`docs/MAME_TAURI_TODO_2026-09-08.md` is reconciled by MT-2201 with two explicit states:

- `[x]` — closed by implementation, qualification evidence, an explicit policy decision, or a documented runtime/platform boundary;
- `[ ] **Deferred — optional research:**` — deliberately incomplete and outside the external-window product completion claim.

There are no bare unchecked planning boxes after reconciliation.

## 2. MT-2201 — TODO reconciliation

The closure reconciliation rolls forward the existing implementation and qualification records rather than reopening already-qualified work.

| Area | Closure status | Primary evidence |
| --- | --- | --- |
| MT-000 / MT-100 | Complete | `MAME_TAURI_MT000_FOUNDATION_CLOSURE_2026-09-08.md`, `MAME_TAURI_MT100_SCAFFOLD_CLOSURE_2026-09-08.md` |
| MT-200 | Complete | `MAME_TAURI_BACKLOG_RECONCILIATION_MT200_MT300_MT401_2026-09-11.md` |
| MT-300 | Complete | prior reconciliation plus pinned MT-301 fixture capture run `34688122578` and checked provenance regression |
| MT-400 | Complete | checked milestone state, MT-401 reconciliation, MT-408 research note, MT-410 performance qualification |
| MT-500 / MT-600 | Complete | checked milestone state and MT-501 through MT-508 / MT-601 through MT-608 qualification notes |
| MT-700 | Complete | MT-701 through MT-711 qualification plus `MAME_TAURI_BACKLOG_RECONCILIATION_MT708_MT903_2026-09-12.md` |
| MT-800 / MT-900 | Complete | `MAME_TAURI_BACKLOG_RECONCILIATION_MT708_MT903_2026-09-12.md` and DG-7 policy |
| MT-1000 | **Deferred optional research** | no embedded-render adoption claim |
| MT-1100 | **Deferred optional research** | gated on a positive MT-1010 decision; no dedicated OSD claim |
| MT-1200 | **Deferred optional research** | no in-process justification/adoption decision |
| MT-1300 | Complete for packaging mechanics/product boundary | MT-1301 through MT-1307 records; public release limitations remain documented |
| MT-1400 | Complete | MT-1404–1407 CI/qualification records plus existing project CI |
| MT-1500 | Complete | `MAME_TAURI_MT1500_SECURITY_HARDENING_2026-09-13.md` |
| MT-1600 | Complete | `MAME_TAURI_MT1600_DIAGNOSTICS_OBSERVABILITY_2026-09-13.md` |
| MT-1700 | Complete **except MT-1705 deferred** | `MAME_TAURI_MT1700_PERFORMANCE_QUALIFICATION_2026-09-13.md` |
| MT-1800 | Complete under its explicit coverage model | `MAME_TAURI_MT1800_CROSS_PLATFORM_QUALIFICATION_2026-09-13.md` |
| MT-1900 | Complete | `MAME_TAURI_MT1900_UPSTREAM_SUSTAINABILITY_2026-09-13.md` |
| MT-2000 | Complete | MT-2004 release-candidate record plus prior product/VS-1 reconciliation and stop qualification |
| MT-2100 | Complete | `MAME_TAURI_MT2100_FINAL_QUALITY_CLOSURE_2026-09-13.md` |
| VS-1 | Complete | `MAME_TAURI_FRONTEND_STOP_VS1_QUALIFICATION_2026-09-11.md` plus prior reconciliation |

### MT-301 provenance gap closure

The earlier MT-300 reconciliation correctly left two MT-301 provenance items open. They were subsequently closed by the checked-in capture workflow and fixture provenance:

- pinned package: `mame=0.264+dfsg.1-1`;
- observed MAME version line: `0.264 (unknown)`;
- GitHub Actions capture run: `34688122578`;
- provenance is asserted in `tauri/src-tauri/src/metadata/provenance_fixture.rs`.

The pinned 0.264 package is a **fixture provenance source**, not a statement that 0.264 is the release MAME runtime for the Tauri application.

## 3. MT-2202 — unresolved research status

The following tracks remain deliberately incomplete:

### MT-1000 — native-window integration research

No embedded native render surface is part of the qualified external-window product. The platform handle, child/sibling surface, BGFX targeting, resize/HiDPI, fullscreen, multi-monitor, focus/input ownership, and external-vs-embedded benchmark tasks remain deferred.

### MT-1100 — dedicated Tauri MAME OSD

MT-1100 is gated on a positive MT-1010 embedded-render recommendation. That decision has not been made, so no `tauri_osd_interface`, render-module integration, native gameplay-input/audio integration, or OSD maintenance claim is made.

### MT-1200 — optional in-process MAME hosting

No demonstrated product requirement currently justifies giving up the sidecar process-isolation boundary. Library-build feasibility, C ABI design, ownership/threading, exception/panic containment, ABI versioning, and an in-process prototype remain deferred.

### MT-1705 — embedded-render performance qualification

MT-1705 remains `deferred_until_mt1000`. No embedded renderer exists to measure for frame pacing, input latency, CPU/GPU overhead, or A/V synchronization. This is not a failed product gate; it is a conditional research gate that was never activated.

These deferred items are intentionally left unchecked and explicitly labeled in the TODO. Useful-product completion and optional research completion are therefore separate and auditable.

## 4. MT-2203 — final upstream-divergence audit

### Recorded divergence point

Ralph Bridge compared the current pre-closure `master` (`f4a453956e3af9ba9b5ee91124b8433e523f4605`) with the recorded planning/upstream merge base `7cc3033a50b00801240f020b7098e22ffffdcd44` and reported:

- status: `ahead`;
- fork commits since that base: **302**;
- behind relative to that base: **0**.

This is a comparison with the recorded merge base, **not** a claim that the fork is zero commits behind current `mamedev/mame:master`. Current-upstream synchronization must continue to use the MT-1900 rehearsal procedure.

### Fork-owned file surface

At the pre-closure tree, the principal isolated project surface is:

- `tauri/**` — 165 files;
- `scripts/tauri/**` — 23 files;
- `tests/tauri/**` — currently no files;
- `docs/MAME_TAURI*` — 58 files;
- project workflows: `tauri-project.yml`, `tauri-security.yml`, `tauri-linux-packaging.yml`, `tauri-windows-packaging.yml`, `tauri-macos-packaging.yml`, plus `mt301-fixture-capture.yml`;
- `.github/dependabot.yml` for Tauri npm/Cargo/GitHub Actions updates.

The architecture has not added broad edits under `src/emu`, `src/devices`, or MAME machine-driver trees. The main product implementation is physically separated from upstream MAME source.

### Upstream-origin files intentionally touched

The diff from the recorded merge base shows these notable upstream-origin touches before MT-2200:

- root `.gitattributes` — added LF policy for project-owned runtime-control Lua files;
- `.github/workflows/bgfxshaders.yml` — narrowed self-trigger scope;
- `.github/workflows/ci-linux.yml` — narrowed workflow trigger and excludes `scripts/tauri/**` from native-MAME CI;
- `.github/workflows/ci-macos.yml` — project native `SUBTARGET=tiny` smoke plus path/dispatch routing;
- `.github/workflows/ci-windows.yml` — project native `SUBTARGET=tiny` smoke plus path/dispatch routing;
- `.github/workflows/docs.yml`, `hash.yml`, `includeguards.yml`, and `language.yml` — narrowed broad `.github/workflows/**` triggers so Tauri-only workflow edits do not fan out into unrelated upstream jobs.

### Avoidable vs deliberate divergence

The root `.gitattributes` touch is avoidable because the LF rule can live in a nested project-owned `tauri/.gitattributes`; MT-2200 moves the rule there to remove that root-file patch.

The upstream workflow edits are **deliberate but still maintenance debt**. They keep Tauri-only changes from needlessly triggering unrelated native workflows and provide representative Windows/macOS native validation. A future cleanup could reduce this debt further by moving project-specific native smoke jobs into a dedicated project-owned workflow and accepting or otherwise redesigning upstream workflow trigger behavior. Until then, any upstream sync that touches these workflow files must receive explicit conflict review and native CI.

### Tractability conclusion

Future sync remains tractable because:

1. almost all product code lives in isolated project-owned namespaces;
2. the known upstream-origin patch surface is small and concentrated in CI routing rather than emulation core code;
3. MT-1900 provides a repeatable inventory helper and rehearsal procedure;
4. the first recorded upstream rehearsal integrated 80 upstream commits with no manual conflict resolution and passed Linux, macOS, Windows, docs, include-guard, and XML/JSON validation on rehearsal SHA `603770bbd12036fd2cc7d7c3bf40bf21775396bb`.

## 5. MT-2204 — architecture and maintenance handoff

### Architecture summary

- **Frontend:** React 19 + TypeScript, built by Vite, communicates through typed Tauri commands/events.
- **Trusted application layer:** Rust/Tauri owns process launch, filesystem access, SQLite state, metadata import, configuration, auditing, artwork access, diagnostics, and runtime control.
- **MAME boundary:** MAME executes as a supervised child process. Rendering, PCM audio, and gameplay input remain native to MAME/OS paths and never traverse the WebView IPC hot path.
- **Data:** generated MAME metadata is separated from user-owned favorites, collections, history, settings, audit provenance, artwork configuration, and save-state records.
- **Runtime control:** a versioned, scoped, authenticated local protocol provides bounded pause/resume/reset/exit/save/load/mute/query-state semantics. Unsupported live volume control remains explicitly unsupported rather than emulated through PCM transport or generic evaluation.

### Qualified application/tool versions

The checked-in project declares:

- application package version: `0.1.0`;
- Node.js requirement: `>=22.12.0`;
- `@tauri-apps/api`: `2.11.1`;
- `@tauri-apps/cli`: `2.11.4`;
- React / React DOM: `19.1.0`;
- TypeScript: `6.0.3`;
- Rust edition: 2021, with CI using the stable toolchain and committed `Cargo.lock`.

The metadata regression fixture is sourced from MAME `0.264 (unknown)` / Ubuntu package `0.264+dfsg.1-1`. Packaging CI uses synthetic MAME payloads to validate package topology; it does not certify a public release MAME binary version.

### Known limitations and non-claims

- gameplay remains in an external native MAME window;
- MT-1000/1100/1200 and MT-1705 remain deferred optional research;
- live master-volume setting is intentionally unsupported under the selected native control seam; mute is supported;
- public macOS release signing/notarization still requires real Apple credentials; ad-hoc CI signing is smoke coverage only;
- package CI validates synthetic staged MAME resources; a public bundled-runtime release must stage and qualify a real MAME binary/resource/license set;
- hosted CI does not claim exhaustive physical controller, audio-device, fullscreen/window-manager, or native Wayland coverage;
- external network artwork providers remain disabled by default until a provider is qualified against DG-7.

### Required CI and release workflow

For a release/closure candidate, freeze an exact 40-character SHA and require the applicable exact-head gates:

- `Tauri project` — frontend format/lint/typecheck/tests/build, Rust format/tests/Clippy, qualification ledgers, production Tauri build, Linux development-window smoke;
- `Tauri security` — security policy regression plus JavaScript/Rust advisory checks;
- `Tauri Linux packaging` — `.deb` + AppImage package smoke;
- `Tauri Windows packaging` — NSIS clean install/payload/uninstall smoke;
- `Tauri macOS packaging` — app/DMG layout, ad-hoc signature, relocation/resource smoke and fail-closed release-signing policy;
- `Build documentation` — HTML/PDF documentation build;
- native MAME CI when an upstream-maintained MAME path or native workflow is changed.

Use `scripts/tauri/report-ci-evidence.py` for exact-SHA evidence and `scripts/tauri/mt2004_release_candidate.py` / `mt2100_final_quality_closure.py` for the release-quality ledgers. Do not substitute a green ancestor or descendant for the candidate SHA.

### Upstream synchronization procedure

Follow `MAME_TAURI_MT1900_UPSTREAM_SUSTAINABILITY_2026-09-13.md`:

1. fetch `origin/master` and `mamedev/mame:master` as `upstream/master`;
2. record merge base, divergence counts, and changed paths;
3. run `scripts/tauri/mt1900_upstream_inventory.py` and review every upstream-tree touch;
4. perform the merge first on a throwaway rehearsal branch;
5. record conflicts and their classes;
6. run project CI plus relevant native MAME CI on the exact rehearsal head;
7. promote only the tested integration, never an untested reconstruction;
8. repeat weekly divergence checks during active work and a full rehearsal at least every two weeks or before a release candidate.

### Maintenance priorities after engineering closure

Routine maintenance is expected to focus on dependency updates, upstream synchronization, platform/toolchain drift, advisory review, and release-artifact qualification. New product work should not silently reactivate MT-1000/1100/1200; those tracks require an explicit research decision and their own qualification plan.

## 6. Local closure validation

On the uploaded source archive corresponding to pre-closure `master`, the available offline checks passed:

- all checked-in `scripts/tauri/test-*.py` regression/qualification helpers;
- `scripts/tauri/test-stage-mame-runtime.sh`.

The sandbox had Node.js `v22.16.0` and npm `10.9.2`, but the archive did not contain dependencies initially; dependency installation could not be relied upon as an offline validation path. Rust/Cargo were not installed in the sandbox. Therefore frontend and Rust results are taken only from exact-head CI, not fabricated from the local environment.

## 7. MT-2200 acceptance

MT-2200 is complete when this closure branch has:

- the reconciled TODO with no ambiguous bare unchecked state;
- explicit deferred status for MT-1000, MT-1100, MT-1200, and MT-1705;
- this divergence/handoff record;
- the root `.gitattributes` project-specific rule moved into the Tauri-owned tree;
- a regression test preventing reintroduction of ambiguous TODO state;
- green applicable branch CI, followed by promotion to `master` and post-promotion verification.

Final exact-head run IDs and promoted SHA are appended after branch qualification.
