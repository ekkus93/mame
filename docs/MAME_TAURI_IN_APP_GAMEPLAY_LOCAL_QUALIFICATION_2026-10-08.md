# MAME Tauri in-app gameplay local qualification

**Date:** 2026-10-08  
**Scope:** full Linux AppImage desktop acceptance for the in-app gameplay specification

This procedure is the repeatable host-side evidence path for the acceptance items that cannot be established by unit tests or GitHub-hosted package smoke tests. It does not replace exact-head CI. It complements it with a real desktop session, a real locally available ROM, real keyboard/gamepad input, and native audio.

## Preconditions

Use the full real-runtime AppImage produced after the source changes being qualified. Do not substitute the synthetic fast-package image, an unpacked AppDir, or an ancestor build. The normal user ROM directory should be present; the conventional path is `~/mame/roms` unless explicit settings override it.

The repository must be at the source SHA corresponding to the AppImage. Record any local working-tree changes rather than silently ignoring them.

## Run

From the repository root:

```bash
scripts/tauri/qualify-in-app-gameplay.sh \
  /absolute/path/to/MAME-Tauri-Frontend.AppImage \
  <machine-short-name> \
  "$HOME/mame/roms"
```

The script performs the non-visual evidence collection automatically:

- records repository HEAD and dirty/clean state;
- records the exact AppImage path, size, file type, and SHA-256;
- extracts the AppImage and identifies the packaged `mame-runtime/bin/mame`;
- records the bundled MAME `-version` result and packaged runtime provenance;
- records desktop/session identity and the ROM path under test;
- captures the AppImage process stdout/stderr;
- creates a dated qualification checklist;
- after the application closes, copies the newest diagnostics export created during the run when available.

The operator still performs the user-facing interactions inside the actual AppImage: audit/start, gameplay, keyboard/gamepad, native sound/mute, resize, fullscreen, stop/relaunch, negative content/path cases, and diagnostics export. Those observations are deliberately explicit because an Xvfb window-existence check does not prove playable gameplay.

## Isolated normal-first-run review

To qualify first-run behavior without relying on an existing developer catalog,
audit database, or settings profile, use the companion wrapper from a **real**
desktop session:

```bash
bash scripts/tauri/qualify-in-app-gameplay-fresh-profile.sh \
  /absolute/path/to/MAME-Tauri-Frontend.AppImage \
  <machine-short-name> \
  "$HOME/mame/roms"
```

It creates a temporary HOME and separate XDG config/data/cache directories,
exposes the selected existing ROM directory through the conventional
`~/mame/roms` path by symlink (without copying ROMs), and runs the same full
AppImage capture script. The temporary profile is deleted on exit; the evidence
directory retains `fresh-profile-manifest.txt`,
`fresh-profile-instructions.txt`, and the ordinary qualification files.
The user's existing application profile is not read or changed by the wrapper.
Run without custom runtime/path environment overrides, and confirm first-run
catalog behavior, ROM discovery, audit, launch, controls, and diagnostics in
the actual desktop UI. This wrapper prepares a reproducible review; it does
**not** itself satisfy the user-facing Phase 7 acceptance checkbox.

## Required evidence

A qualifying evidence directory contains at least:

- `manifest.txt`;
- `qualification.md`;
- `app.stdout.log`;
- `app.stderr.log`;
- `runtime-provenance.txt` when the package provides it;
- `diagnostics.json` from the gameplay session.

The diagnostics snapshot is expected to retain the terminal session, effective argv and content paths, bounded stdout/stderr, frame receipt/drop/delivery/presentation metrics, last frame dimensions/error state, video transport, input-bridge state, runtime identity, and content-path validation.

## Acceptance interpretation

A green package workflow proves the full AppImage contains a release-qualified real MAME runtime and that the application window starts. It does **not** prove in-app gameplay. The desktop run closes that gap only when a real machine reaches first-frame-presented in the TypeScript canvas, no separate native MAME gameplay window appears, controls and native audio work, and stop/relaunch leaves no stale frame/input state.

If the desktop run fails, retain the complete evidence directory and fix the source before rebuilding. Any change to launch, frame transport, pixel conversion, packaging, or runtime options invalidates the previous gameplay artifact and requires a fresh run.

## Original installed-MAME ROM-path baseline (Phase 0 / Phase 4)

Before changing any application settings, capture how the **original installed**
MAME executable resolves the same content. This is host-specific evidence, not
something that a CI container or a source-level unit test can establish.

From a normal terminal on the user's desktop, without replacing HOME or
`XDG_CONFIG_HOME`:

```bash
command -v mame
mame -version
mame -showconfig > original-mame-showconfig.txt
grep -E '^[[:space:]]*(rompath|inipath|cfg_directory|nvram_directory)[[:space:]]' original-mame-showconfig.txt
find "$HOME/mame/roms" -maxdepth 1 -type f -printf '%f\n' | sort > original-mame-rom-directory-files.txt
```

Keep `original-mame-showconfig.txt` with the qualification evidence. Record
the installed MAME executable's absolute path, version, and the *effective*
`rompath` after its INI precedence has been applied. If `mame` is not on
`PATH`, record that fact instead of substituting the bundled runtime and
calling it the original installed MAME. Do not change `rompath` just to make
the comparison pass.

Choose a short name that is **actually installed and audited** in this
directory. For a reproducible native-MAME baseline, run that machine with the
original MAME configuration and retain its exit status and stdout/stderr;
repeat with an intentionally missing-content machine. Compare the original
MAME's effective paths with the Tauri Settings/Diagnostics effective paths and
the launch argv. Differences are findings, not reasons to overwrite the
original configuration.

The directory filename inventory can reveal user-supplied ROM names and should
remain in the private local evidence folder; do not publish it as a CI artifact.

## Frame-seam qualification matrix (Phase 1 / Phase 3)

The headless `___empty` CI test establishes that the **real** MAME binary can
produce correctly sized snapshot bytes without a visible SDL window. It is not
a ROM-game test and does not qualify orientation, raster/vector parity,
native audio, controller behavior, or a playable Tauri canvas.

On the desktop, run the repository's benchmark against the **same real MAME
version bundled in the AppImage**, using machine short names confirmed
available by the local audit. Record one report per representative machine:

```bash
scripts/tauri/qualify-mame-frame-seams.sh \
  /absolute/path/to/extracted/mame-runtime/bin/mame \
  <audited-machine-short-name> \
  "$HOME/mame/roms" \
  artifacts/in-app-gameplay/<machine>-frame-seam.txt
```

The benchmark measures Lua callback CPU time for the diagnostic
`screen:pixels()` seam and the selected `snapshot_pixels()` seam
independently. A diagnostic-seam `screen_status=unsupported` is **not** a
production snapshot failure. The benchmark runs with `-nothrottle`, so its
CPU timings alone do not establish normal-speed emulation, end-to-end frame
age, or the 60 FPS Tauri canvas acceptance target.

Use this coverage matrix; do not substitute `___empty` for any of the
real-machine rows:

| Machine category | Required observation | Evidence |
| --- | --- | --- |
| Native-resolution raster, approximately 60 Hz | Snapshot payload, first in-app frame, sustained presented FPS, frame age and dropped frames | Frame-seam report, diagnostics, screenshot |
| Vector | Snapshot validity and known vector-effects limitations, controls and in-app rendering | Frame-seam report, diagnostics, screenshot |
| Rotated screen | Reported orientation, correct canvas rotation and aspect ratio after resize/fullscreen | Diagnostics, screenshot |
| Different/high resolution or aspect ratio | Payload dimensions, bounded capture CPU time, canvas scaling, emulation speed | Frame-seam report, diagnostics |
| Available/best-available content | Audit and launch agree on paths and machine identity | Audit result, launch argv, diagnostics |
| Known missing content | Pre-spawn audit/content diagnosis and no misleading control-channel message | Diagnostics |
| Normal fresh profile | First-run catalog, ROM path, audit, gameplay, return to library | Fresh-profile manifest, completed qualification note |

For each real machine, compare native MAME execution with and without frame
capture under otherwise identical settings. Capture emulation speed, CPU,
memory, native audio, keyboard/gamepad behavior, stop/relaunch, and any
startup/stream failure separately. A report is *evidence pending* until the
operator supplies the measurements and desktop observations. Keep the TODO
checkboxes open until then.

## Artifact-to-source reconciliation

Before checking any full-AppImage item complete, confirm that the AppImage
manifest's source SHA is the intended tested revision, that the file hash and
bundled runtime version were recorded from that exact AppImage, and that the
desktop run used the executable AppImage rather than an unpacked AppDir.
Source changes to launch, transport, pixel conversion, or packaging require a
fresh package and a repeat qualification. An in-progress, cancelled, or skipped
CI run is not a passing artifact.

