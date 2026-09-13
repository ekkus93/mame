# MT-2004 External-Window Release Candidate

**Date:** 2026-09-13  
**Repository:** `ekkus93/mame`  
**Scope:** production-useful Tauri MAME frontend with supervised external-window MAME execution.  
**Previous qualified base:** `a93c959166a6e5c9d35e1fa6e43fe1d81fb954af`

## Release-candidate definition

MT-2004 closes the useful-product milestone without claiming embedded rendering,
dedicated OSD integration, or in-process MAME hosting.  The release candidate is
an external-window frontend that can configure executable/content paths, audit
content, browse metadata, launch MAME as a supervised process, report actionable
errors, and package the Tauri application for supported desktop platforms.

The exact qualified commit for an MT-2004 candidate is the commit SHA passed to
`scripts/tauri/mt2004_release_candidate.py --sha`.  CI generates an artifact
named `mame-tauri-mt2004-external-window-rc-${{ github.sha }}` from that helper
so the candidate SHA, required gates, supported artifacts, and known limitations
are recorded as machine-checkable evidence for the exact workflow head.

## Supported packages

The release-candidate package set is intentionally limited to the desktop targets
that already have automated package smoke coverage:

| Platform | Package formats | Required workflow/job |
| --- | --- | --- |
| Linux | Debian package, AppImage | `Tauri Linux packaging` / `linux-deb-appimage-smoke` |
| Windows | NSIS installer | `Tauri Windows packaging` / `windows-nsis-smoke` |
| macOS | app bundle, DMG | `Tauri macOS packaging` / `macos-app-dmg-smoke` |

## Required smoke coverage

The exact candidate SHA must have these gates green:

- `Build documentation` / `build-docs` renders the documentation set to HTML and PDF.
- `Tauri project` / `linux-quality` runs frontend format/lint/typecheck/tests,
  production frontend build, Rust format/tests/clippy, license inventory,
  performance qualification, MT-1800 qualification, and MT-2004 manifest
  regression tests.
- `Tauri project` / `linux-release-qualification` builds the production Tauri
  binary and smokes the development window under Xvfb.
- `Tauri Linux packaging` / `linux-deb-appimage-smoke` stages a synthetic MAME
  runtime, validates bundled runtime paths, builds `.deb` and AppImage packages,
  and smokes Debian install/uninstall plus AppImage execution.
- `Tauri Windows packaging` / `windows-nsis-smoke` stages a synthetic MAME
  runtime, validates bundled runtime paths, builds the NSIS installer, and smokes
  clean install, packaged-path behavior, and clean uninstall.
- `Tauri macOS packaging` / `macos-app-dmg-smoke` stages a synthetic MAME
  runtime, validates bundled runtime paths, proves missing Apple credentials are
  rejected for public release signing, builds ad-hoc app/DMG smoke artifacts, and
  checks clean-runner app, DMG, resources, relocation, and signature behavior.

## Known limitations

- **External window only:** embedded rendering is not complete and is not claimed
  by this release candidate.
- **Runtime/content ownership:** hosted CI uses synthetic packaged-runtime
  fixtures.  A public release with real MAME runtime files must stage a reviewed
  MAME executable/source payload with `scripts/tauri/stage-mame-runtime.sh` and
  honor the relevant MAME/content licenses.  Users remain responsible for their
  own ROM and software content.
- **macOS public signing/notarization:** CI proves credentialless public signing
  fails closed and uses ad-hoc signing only for smoke coverage.  A public macOS
  release requires real Apple signing and notarization credentials.
- **Physical-device coverage:** hosted CI does not exhaustively prove every
  controller, audio device, display manager, or fullscreen/window-manager
  permutation.  Run a manual device smoke on target hardware before broad
  distribution.

## Acceptance mapping

- Package on supported platforms: Linux `.deb`/AppImage, Windows NSIS, and macOS
  app/DMG workflows are required gates.
- Complete smoke test: package build, install/uninstall, bundled-runtime,
  production-build, dev-window, and platform-specific package smoke gates are
  required.
- Document known limitations: this note and the generated MT-2004 manifest list
  the product boundaries explicitly.
- Record exact qualified commit: the generated manifest records `candidate_sha`,
  and the final handoff must cite the exact green commit and workflow run IDs.

MT-2004 is complete only after the manifest regression, documentation gate, Tauri
project gate, and all three platform packaging gates pass on the candidate SHA.
