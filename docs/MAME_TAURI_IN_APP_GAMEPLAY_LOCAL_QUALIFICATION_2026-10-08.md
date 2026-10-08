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
scripts/tauri/qualify-in-app-gameplay-fresh-profile.sh \
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
