# MAME Tauri MT-1305 — Linux packaging

**Date:** 2026-09-12  
**Task:** MT-1305 — Linux packaging  
**Status:** Implementation complete; CI qualification pending

## Initial package formats

The first Linux distribution targets are:

- Debian package (`.deb`) for native package-manager installation on Debian/Ubuntu-family systems;
- AppImage for a portable direct-download package.

RPM/Flatpak/Snap are intentionally deferred. They can be added later without changing the MT-1302 package-owned MAME runtime contract.

`tauri/src-tauri/tauri.linux-bundle.conf.json` is an explicit package-build overlay. It enables only `deb` and `appimage`, uses the validated Tauri icon, and maps `bundle-resources/mame-runtime` to the package resource directory as `mame-runtime`. The Linux packaging workflow supplies this file through Tauri CLI `--config`; it is deliberately not named `tauri.linux.conf.json`, because that filename is automatically merged into every Linux Tauri/Cargo build and would make generated package resources mandatory for ordinary developer and unit-test builds.

## Runtime dependency strategy

The Debian package relies on Tauri's generated native dependency metadata for the GTK/WebKit desktop stack. CI fails if the generated package does not declare both WebKitGTK 4.1 and GTK 3 runtime dependencies.

The package-owned MAME executable and its runtime data are not system dependencies. They remain under Tauri's generated package resource root, with `mame-runtime` as the stable relative subtree. CI discovers the concrete Debian resource root from the package payload instead of hard-coding Tauri's product-name/path normalization, then proves that the package contains:

```text
mame-runtime/
  bin/mame
  hash/
  bgfx/
  licenses/COPYING
  licenses/legal/
```

The same discovery is repeated against the files actually installed by `dpkg`, so the package inspection and install-time assertions must agree on the bundled runtime topology.

The AppImage contains the same package-owned MAME resource tree. `bundleMediaFramework` remains disabled because the frontend does not require Tauri to redistribute a GStreamer multimedia stack for this packaging milestone.

Linux packages are built on GitHub's Ubuntu 22.04 image rather than the newest available distro image. This deliberately keeps the build-system glibc baseline older and reduces accidental minimum-runtime-version inflation.

## X11 and Wayland qualification

### X11

X11 is executable-qualified in CI. The Debian-installed frontend and the AppImage are both launched under `Xvfb`; `xdotool` must observe a visible window named `MAME Tauri Frontend`. A process that exits before showing the window or never exposes the window fails qualification.

### Wayland

Wayland uses the same GTK/WebKitGTK application stack and package payload. MT-1305 does **not** claim a compositor-level Wayland runtime qualification from GitHub-hosted CI because the qualification runner does not provide a production Wayland desktop session. This is recorded explicitly rather than treating an X11/Xvfb result as Wayland evidence.

A later desktop matrix may add compositor-specific Wayland runtime testing without changing the package formats or resource topology.

## Package smoke contract

`.github/workflows/tauri-linux-packaging.yml` runs on a fresh Ubuntu 22.04 runner and:

1. stages a bounded synthetic MAME executable/resource fixture through the MT-1302 stager;
2. installs Linux build and smoke dependencies;
3. runs the Rust bundled-runtime validator on Linux without applying the packaging overlay;
4. builds `.deb` and AppImage packages with the explicit `tauri.linux-bundle.conf.json` overlay;
5. reads Debian control metadata and inventories the actual filesystem payload, failing with that inventory if an expected path is absent;
6. discovers and validates the generated package resource root instead of assuming a specific product-name normalization;
7. installs the `.deb` with the native package manager;
8. validates the installed frontend executable, desktop entry, executable bundled MAME, hash data, BGFX data, and license material;
9. launches the installed frontend under X11/Xvfb and requires its real application window;
10. removes the package and proves the executable and bundled runtime are removed;
11. launches the AppImage using its extract-and-run path, avoiding a FUSE-only qualification dependency;
12. extracts the AppImage and validates its `AppRun`, desktop entry, executable bundled MAME, resources, and licenses;
13. verifies npm and Cargo lockfiles are unchanged.

The verifier is `scripts/tauri/test-linux-packages.sh`. Every package-layout assertion emits an explicit error, and Debian layout failures dump the actual payload inventory so CI failures remain diagnosable.

The synthetic MAME executable exists only to prove package topology and executable preservation. It is not a release MAME payload and no CI package from this workflow is published.
