# Bundled MAME Linux Release Qualification

**Date:** 2026-09-27  
**Canonical TODO:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`

## Two-tier packaging policy

`Tauri Linux packaging` deliberately separates fast structural validation from release qualification.

The structural job stages a tiny synthetic executable and fixture hash/BGFX/legal files. It proves Tauri resource mapping, Debian/AppImage topology, install/uninstall mechanics, desktop integration, and basic frontend launch. It is **not** acceptable release evidence.

The `real-mame-deb-qualification` job is the release-grade path. It runs for pull requests, `master`, tags, and manual dispatch and checks out the complete repository.

## Real-runtime build

The real job builds `./mame` from the same Git commit that packages the Tauri frontend. The package staging step runs with:

```text
MAME_RUNTIME_QUALIFICATION=real
MAME_SOURCE_SHA=<exact GitHub SHA>
MAME_BUILD_TARGET=mame
MAME_BUILD_PROFILE=release
```

The stager refuses real qualification unless `validate-real-mame-runtime.sh` observes a genuine numeric MAME version and a bounded `-listxml pacman` result. Synthetic fixture markers are rejected. The staged runtime contains MAME, hash XML, BGFX resources, COPYING/legal resources, and provenance.

## Ubuntu installed-package evidence

The job builds one `.deb` containing Tauri plus the staged real MAME runtime. Runtime shared-library ownership is derived from `ldd`/dpkg on the build host and merged into the Debian Depends field; the package may not depend on the distro `mame` package.

The Ubuntu smoke installs that `.deb`, verifies the installed package-owned runtime is executable and non-writable by the normal user, executes real `-version` and `-listxml` from an arbitrary working directory, and launches the installed Tauri app under Xvfb with fresh XDG config/data roots. Qualification waits until the application-created catalog contains an active generation with:

- `source_kind = bundled`;
- `trust = qualifiedBundled`;
- a real machine count greater than 1000.

That is the end-to-end first-run proof that no executable path is required and automatic metadata bootstrap reaches a ready catalog.

The same smoke performs a package reinstall/upgrade check while preserving the generated catalog, a user-data sentinel, and a settings document containing content paths, launch preferences, and an external override. Package-owned runtime files are reinstalled while user-owned state is unchanged.

## Debian evidence

A separate Debian Bookworm container installs the same built `.deb` using its declared dependencies, locates the bundled package runtime, executes `-version`, performs bounded `-listxml pacman`, and removes the package. A full Debian graphical WebView session is not currently automated; Ubuntu Xvfb is the graphical installed-application qualification, while Debian is package/dependency/runtime qualification.

## AppImage

The real job also builds and extracts the AppImage, verifies the real runtime/resources/provenance, and executes the runtime validator. The Debian/Ubuntu `.deb` remains the primary product contract.

## Upgrade behavior

Runtime identity is recorded in metadata generations. A changed bundled executable identity makes an existing generation stale; the frontend startup state machine automatically refreshes missing/stale metadata. Package reinstall/upgrade smoke proves package operations do not erase the separate XDG user-data/config trees. Intentional external overrides are persisted settings and remain explicit until the user chooses **Use bundled MAME**.

## Closure evidence

Final BMR closure must record the exact PR head, real-runtime workflow run ID, uploaded artifact identity, `provenance.json` contents, promoted master SHA, and post-merge qualification.
