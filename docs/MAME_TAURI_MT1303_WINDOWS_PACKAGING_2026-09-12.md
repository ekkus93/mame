# MAME Tauri MT-1303 — Windows packaging

**Date:** 2026-09-12  
**Task:** MT-1303 — Windows packaging  
**Status:** Implementation complete; CI qualification pending

## Package choice

The initial direct-download Windows package is **NSIS** (`*-setup.exe`) built by the Tauri v2 bundler.

NSIS is preferred for the first Windows qualification because it is a first-class Tauri target, supports current-user installation without elevation, supports unattended installation, and gives the CI workflow a deterministic install/uninstall surface. MSI remains a future distribution option if enterprise deployment requirements justify a second Windows package format; MT-1303 does not create two independently qualified installer formats without a product requirement.

The Windows-specific configuration lives in `tauri/src-tauri/tauri.windows.conf.json`. It is merged only for Windows builds and therefore does not enable bundling on Linux/macOS. The configuration:

- enables bundling on Windows;
- selects `nsis` as the only initial Windows target;
- uses `currentUser` install mode;
- uses the Tauri WebView2 download-bootstrapper policy;
- maps the MT-1302 staged `bundle-resources/mame-runtime` tree to installed `mame-runtime` resources.

## Bundled MAME boundary

The Windows package consumes the exact MT-1302 runtime topology:

```text
mame-runtime/
  bin/mame.exe
  hash/
  bgfx/
  licenses/COPYING
  licenses/legal/
```

The Rust `BundledRuntimeLayout` test suite is compiled and executed on the Windows runner before bundling. This validates the platform-specific `mame.exe` path and the backend-owned `QualifiedBundled` source boundary.

The installer smoke additionally validates the installed resource paths from the actual NSIS installation directory recorded by Windows. It does not guess a hard-coded `%LOCALAPPDATA%` path.

## Clean-install and clean-uninstall qualification

`.github/workflows/tauri-windows-packaging.yml` performs a Windows package smoke on a GitHub-hosted clean runner:

1. create a bounded synthetic MAME source/runtime fixture;
2. stage it through the same `stage-mame-runtime.sh` contract used by release packaging;
3. run the Windows bundled-runtime Rust tests;
4. build a real Tauri NSIS installer;
5. install it unattended;
6. locate the actual installation via the Windows uninstall registry entry;
7. verify the application executable, bundled `mame.exe`, hash, BGFX, `COPYING`, and legal resources in the installed tree;
8. run the generated NSIS uninstaller unattended;
9. verify both the installation directory and uninstall registry entry are removed.

The PowerShell verifier is `scripts/tauri/test-windows-nsis-install.ps1`.

## Synthetic-payload safety rule

The CI package deliberately contains a **synthetic file named `mame.exe`**, not a runnable or release-qualified MAME build. Its only purpose is to prove Tauri/NSIS resource placement and uninstall cleanup. The workflow does not upload or publish this installer artifact.

A distributable Windows release must replace that fixture with a separately qualified real MAME Windows executable and the matching MT-1302 resource/license tree. Passing the MT-1303 installer smoke therefore proves packaging mechanics, not native MAME emulation compatibility or release signing.

## Follow-on release concerns

Code signing, release provenance, final native MAME version qualification, and publication policy are release-hardening concerns and must be satisfied before public Windows distribution. They must not be inferred from this unsigned synthetic package smoke.
