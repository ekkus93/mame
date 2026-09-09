# MAME Tauri DG-2 — Release Executable Policy

**Document date:** 2026-09-08  
**Repository:** `ekkus93/mame`  
**Decision gate:** DG-2 / MT-201  
**Status:** Accepted implementation policy

## Decision

The default packaged-release policy is a **project-qualified bundled MAME executable** when the target platform/package format permits redistribution under MAME's licensing requirements.

The application also supports an **explicitly configured external MAME executable** as a user opt-in. External executables are never promoted to bundled/qualified trust merely because they answer the expected MAME CLI probes.

A **development-tree executable** is a separate development/test source and is not a release-default source.

## Source and trust model

| Source | Intended use | Trust classification |
| --- | --- | --- |
| bundled | normal packaged release | `qualifiedBundled` |
| external | explicit user configuration | `userConfigured` |
| development tree | contributor/test workflow | `development` |

`qualifiedBundled` means only that the executable is the build qualified with that application release. It is not a cryptographic attestation and does not imply that third-party content is trusted.

## Identity contract

Before use, the backend validates that the selected path is a regular usable executable and probes the current MAME CLI with the discrete argument vector:

```text
-noreadconfig
-version
```

Current MAME implements `-version` as a zero-argument frontend command that emits `emulator_info::get_build_version()`. `-noreadconfig` prevents user INI state from influencing this identity probe.

The recorded executable identity contains:

- source class;
- trust classification;
- canonical executable path;
- version token;
- optional build descriptor;
- raw bounded version line.

## No silent fallback

The application must not silently fall back from a missing/broken bundled executable to a PATH/system executable or to another configured binary. A different executable may be selected only through an explicit, surfaced configuration action.

Likewise, an external executable remains classified as `userConfigured` even if its reported version matches the bundled build.

## Packaging and content

Bundling MAME must preserve the licensing/notices already documented by the project. This decision does not authorize bundling ROMs, software images, CHDs, or other copyrighted machine/software content.

## Consequences

This policy makes release behavior reproducible while preserving an escape hatch for users who deliberately want another MAME build. Metadata and later audit caches can key their provenance to the executable identity without conflating release-qualified and arbitrary external binaries.
