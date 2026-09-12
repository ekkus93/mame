# MAME Tauri MT-1302 — Bundled MAME runtime resources

**Date:** 2026-09-12  
**Task:** MT-1302 — Bundle required MAME resources correctly  
**Status:** Qualified

MT-1302 defines and validates the package-owned runtime layout used by later platform-specific installers. It does not enable bundling by itself and does not check a MAME executable into Git.

## Qualification evidence

The exact implementation head `2977fe8e2b4a9841dd980bee21db109f3dd548b5` passed GitHub Actions Tauri project run `34712104197` on 2026-09-12. The green gate set included the MT-1302 synthetic staging contract, frontend format/lint/typecheck/tests/production build, Rust format/tests, the library UX performance qualification, Clippy with warnings denied, and lockfile-integrity verification.

The synthetic staging contract proves both a successful complete runtime stage and the failure-safety rule that an invalid source tree is rejected during preflight without destroying an already staged destination.

## Required staged layout

Platform packaging must stage the following tree beneath the application's Tauri resource directory:

```text
mame-runtime/
  bin/
    mame            # Linux/macOS
    mame.exe        # Windows
  hash/
    *.xml           # complete MAME software-list hash directory
  bgfx/
    ...             # complete MAME BGFX runtime resources
  licenses/
    COPYING
    legal/
      ...            # complete docs/legal directory from the same MAME source tree
```

The package layer resolves the executable only from this package-owned layout. Missing resources are fatal for bundled mode; there is no fallback to a PATH/system MAME or to a user-configured executable.

## Resource inventory

The bundled baseline includes:

- the qualified MAME executable;
- the complete `hash/` tree so software-list metadata and software launch semantics remain available;
- the complete `bgfx/` tree required by MAME renderer configurations that depend on packaged shaders/chains;
- top-level `COPYING` and the complete `docs/legal/` directory from the same MAME source tree.

The baseline deliberately does **not** redistribute ROMs, software images, CHDs, user artwork, samples, cheats, controller files, user INI files, or other copyrighted/user content. Optional MAME runtime features that later require additional redistributable project files must be added explicitly through a follow-up task and license review rather than copied implicitly.

## Staging contract

`scripts/tauri/stage-mame-runtime.sh` accepts a MAME source root, a built executable, and an output directory. It preflights every required source component before deleting/replacing the destination, stages into a temporary sibling directory, validates the staged result, then renames it into place.

The intended package staging target is:

```text
tauri/src-tauri/bundle-resources/mame-runtime
```

That generated directory is ignored by Git. Platform packaging jobs are responsible for running the staging script before invoking the Tauri bundler.

## Installed-layout verification

`bundled_runtime::BundledRuntimeLayout` constructs the expected installed paths from the Tauri resource directory and validates them before a bundled executable can be promoted to `MameExecutableSource::bundled` / `QualifiedBundled`.

Validation is fail-closed and checks:

- package resource directory and `mame-runtime/` root exist as directories;
- the platform-specific executable exists as a regular file and is executable on Unix;
- `hash/` exists and contains software-list XML;
- `bgfx/` exists and contains runtime files;
- `licenses/COPYING` exists;
- `licenses/legal/` exists and contains license files;
- canonicalized runtime components remain beneath the package-owned resource root, preventing symlink escape.

Unit tests cover a valid installed layout, missing hash resources, missing license material, and Unix symlink escape. The valid-layout test also proves bundled resolution receives `QualifiedBundled` trust and never derives its path from renderer input.

## Packaging handoff

MT-1303, MT-1304, and MT-1305 own Windows, macOS, and Linux package-format integration respectively. They should stage this exact resource layout, add it to the platform package, install on a clean machine/VM, and run the installed-path validator as part of their smoke qualification.
