# MAME Tauri Bundled Runtime Distribution Specification

**Date:** 2026-09-27  
**Status:** Proposed implementation contract  
**Repository:** `ekkus93/mame`  
**Primary platform scope:** Debian/Ubuntu `.deb` distribution  
**Canonical implementation checklist:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`

## 1. Purpose

The production application must install as one complete MAME desktop application. A normal Debian or Ubuntu user must not install MAME separately, locate a MAME executable, type a filesystem path, or use a file picker before the application can browse machines, import metadata, or launch MAME.

The production `.deb` is therefore required to contain both:

1. the Tauri/React frontend and Rust backend; and
2. a project-qualified MAME runtime with the runtime resources required by that MAME executable.

The package-owned MAME runtime is the normal product runtime. Selection of an external MAME executable is an advanced override for development and expert use only.

This specification closes the gap between the package plumbing that already supports `mame-runtime` resources and the current application behavior that still treats an externally configured executable as mandatory.

## 2. Problem statement

Current master contains substantial bundled-runtime infrastructure:

- `tauri/src-tauri/src/bundled_runtime.rs` defines and validates a package-owned `mame-runtime` layout.
- `tauri/src-tauri/tauri.linux-bundle.conf.json` includes `bundle-resources/mame-runtime` in Linux bundles.
- `scripts/tauri/stage-mame-runtime.sh` stages the executable, `hash`, `bgfx`, and legal resources.
- `scripts/tauri/test-linux-packages.sh` verifies that Debian/AppImage payloads contain a bundled MAME executable and resources.

However, normal startup still resolves MAME from `settings.mame_executable`. If that optional setting is absent, `app.rs` reports `MameVersionReport::NotConfigured`. The General Settings UI therefore presents a prominent “MAME executable” path field and Browse/Save/Clear controls.

The Linux packaging workflow also stages a synthetic shell-script fixture as `mame`. That is useful for package-layout tests but does not prove that a shipping `.deb` contains a functional MAME runtime.

The resulting product behavior is internally inconsistent: the package can carry MAME, but the application still asks the user to provide MAME.

## 3. Product contract

### 3.1 One-install experience

For the supported Debian/Ubuntu production path, the required user flow is:

```text
download .deb
    ↓
install .deb
    ↓
launch application
    ↓
package-owned MAME runtime is discovered automatically
    ↓
metadata is initialized/refreshed automatically when required
    ↓
MAME-style machine browser becomes usable
```

There must be no required “choose MAME executable” step.

The application may still require the user to configure ROM, CHD, software, artwork, controller, or other personal content paths. MAME software/ROM content is explicitly outside this distribution contract and is not to be bundled merely to make first run appear populated with playable content.

### 3.2 Bundled runtime is authoritative by default

The runtime resolution rule must be:

```text
explicit valid external override
    → use external runtime

otherwise
    → use valid package-owned bundled runtime

if neither is usable
    → report a package/runtime installation failure
```

“No external override configured” must never mean “MAME not configured” for a correctly installed production package.

### 3.3 External runtime is an advanced override

An external MAME executable may remain supported for developers and expert users, but:

- it is not required for first run;
- it is not the default;
- its path field/file picker must not dominate normal General Settings;
- clearing the override means “return to bundled MAME,” not “disable MAME”;
- the UI must always make the active runtime source visible;
- an invalid external override must fail closed and offer a clear way to return to the bundled runtime.

## 4. Runtime model

### 4.1 Typed source identity

The backend must treat runtime selection as a typed source, not as an optional path whose absence implies no runtime.

At minimum, the effective runtime source must distinguish:

- `bundled`: package-owned, validated MAME runtime;
- `external`: explicit user/developer override.

The existing `MameExecutableSource`, source-kind, and trust concepts should be reused rather than creating a second incompatible abstraction.

### 4.2 Settings semantics

The persisted configuration must encode an override preference rather than whether MAME exists.

A recommended logical model is:

```text
runtime preference:
  bundled/default
  external override(path)
```

The exact wire/schema representation may differ, but these semantics are mandatory.

Migration requirements:

- existing settings with no `mameExecutable` migrate to bundled/default;
- existing settings with a valid explicit `mameExecutable` may migrate to an external override so existing power-user behavior is not silently discarded;
- invalid legacy external paths must not make a correctly packaged application permanently unusable; the UI must provide a one-action return to bundled runtime;
- schema migration must be deterministic and covered by tests.

### 4.3 Package-owned resource discovery

Bundled runtime discovery must use Tauri’s package resource directory and `BundledRuntimeLayout`. It must not depend on:

- the process current working directory;
- a hard-coded developer checkout path;
- `PATH`;
- a distro-installed `mame`;
- a user-entered executable path.

The resolved runtime root must remain contained in the package-owned resource directory.

### 4.4 Runtime immutability

The installed runtime under the package resource path is read-only application content.

User-writable MAME state must live in appropriate user-scoped directories. The application must not require mutation of files installed under `/usr`, the Tauri resource directory, or an AppImage mount.

The implementation must preserve MAME-owned INI/CFG semantics while ensuring writes go to user-writable locations.

## 5. Production MAME payload

### 5.1 Required payload

A production Linux package must contain, at minimum:

- the actual MAME executable built/qualified for the release;
- MAME `hash` software-list XML resources;
- MAME `bgfx` runtime resources required by supported renderer paths;
- required license/legal files;
- any other non-system runtime resource that a clean installation requires for the supported product behavior.

The package must not merely contain a filename named `mame`; the executable must be a functional MAME binary.

### 5.2 Runtime provenance

The package build must record enough evidence to answer:

- which MAME source revision produced the bundled binary;
- which build target/profile produced it;
- which frontend/backend revision packaged it;
- which MAME version the binary reports at runtime.

Because this repository is a MAME fork, the preferred production path is to build the bundled MAME runtime from the same qualified source tree/revision used for the package. If a different source/artifact pipeline is used, it must be explicit, reproducible, pinned, and documented.

### 5.3 Synthetic fixtures

Synthetic MAME fixtures may remain in fast CI for isolated layout/unit tests.

They must not be accepted as qualification evidence for a shipping production package.

Any workflow that claims release/package qualification must clearly distinguish:

- synthetic fixture package-layout validation; from
- real MAME runtime package qualification.

## 6. Debian/Ubuntu packaging

### 6.1 Single `.deb`

The release artifact for Debian/Ubuntu must be a single user-installable `.deb` that includes the frontend and bundled MAME runtime.

The user must not need a separate `apt install mame` or a manually downloaded MAME binary.

### 6.2 Dependencies

The Debian package may declare normal shared-library dependencies needed by Tauri/WebKit/GTK and by the bundled MAME executable.

It must not declare the distro `mame` package as a functional dependency for normal operation.

The package workflow must detect missing runtime shared-library dependencies before release.

### 6.3 Installed payload

Package qualification must verify that after installation:

- the frontend executable exists and is executable;
- the desktop entry launches the frontend;
- the bundled MAME executable exists and is executable;
- `hash`, `bgfx`, and legal resources are present;
- the frontend resolves the installed bundled runtime without a user override;
- the bundled runtime can execute `-version`;
- the bundled runtime can perform a bounded metadata/list operation representative of application startup;
- uninstall removes package-owned frontend/runtime files while preserving user data according to normal Debian semantics.

### 6.4 AppImage

The existing AppImage path should use the same bundled-runtime contract where practical. It is secondary to the `.deb` requirement, but it must not regress into requiring an external MAME path.

## 7. Startup and first-run behavior

### 7.1 Normal installed startup

On application startup, the backend must resolve and validate the effective runtime before reporting application readiness to the browser.

For a clean production install with no settings file:

- bundled runtime should resolve automatically;
- the frontend should not display “MAME not configured”;
- runtime identity should report `bundled`;
- metadata readiness should be checked against the bundled runtime identity.

### 7.2 Automatic metadata bootstrap

If metadata is missing because this is first run, the application should automatically begin metadata generation/import using the bundled runtime.

If metadata is stale because the installed bundled MAME version changed during an application upgrade, the application should automatically refresh it.

The normal user should not have to click “Import Metadata” merely because the application was freshly installed or upgraded.

The UI must show bounded, MAME-style progress while initialization is occurring.

### 7.3 Failure behavior

If bundled runtime validation or metadata generation fails:

- remain inside the MAME-like shell;
- state clearly that the installed MAME runtime/package is unhealthy;
- expose diagnostic details suitable for a bug report;
- provide Retry where meaningful;
- provide the advanced external override as an escape hatch, not as the primary recovery instruction;
- do not silently search arbitrary system paths.

## 8. Launch/runtime consistency

The MAME executable used to generate metadata, audit machines, discover BIOS/software information, and launch sessions must be the same effective runtime identity unless an explicit, validated transition occurs.

A runtime switch must invalidate or stale any catalog generation tied to the old executable identity.

Launch code must continue to revalidate all security-sensitive values in Rust.

No WebView-provided path may become an unsanitized shell command.

## 9. General Settings UX

### 9.1 Normal view

The current prominent “MAME executable” configuration group must be removed from the normal settings experience.

The normal runtime section should instead present status similar to:

```text
MAME runtime
Bundled MAME 0.xxx
Installed with this application
[details]
```

No editable filesystem path should be necessary.

### 9.2 Advanced override

If external-runtime support remains, it must be behind an explicit advanced/developer action such as “Use custom MAME executable.”

The advanced surface may provide:

- Browse;
- explicit validation;
- active source/version display;
- “Use bundled MAME” reset action.

The reset action must immediately restore bundled-runtime semantics.

### 9.3 Copy and terminology

User-facing copy must describe the external executable as an override.

It must not tell normal users that they need to “configure the MAME executable.”

## 10. Upgrade behavior

When a package upgrade replaces the bundled MAME runtime:

- startup must discover the new bundled identity;
- stale metadata must be detected;
- metadata refresh must happen automatically;
- settings must continue to refer to bundled/default unless the user deliberately selected an external override;
- user content paths, favorites, collections, controller settings, and other user state must survive.

An external override must remain explicit across upgrades unless the override becomes invalid, in which case the application must make recovery to bundled runtime straightforward and visible.

## 11. Security requirements

Existing security boundaries remain mandatory:

- Rust owns executable/resource resolution and process launch.
- Bundled paths are resolved from Tauri package resources, canonicalized, and checked for containment.
- The package-owned executable must pass executable-file validation.
- External overrides remain explicitly validated before persistence/use.
- No generic shell-command bridge is introduced.
- Runtime switching does not weaken metadata/launch identity validation.
- Bundled resources are not writable through arbitrary frontend IPC.
- Diagnostics must not leak secrets or unrelated user filesystem data.

## 12. Licensing and redistribution

The production packaging workflow must preserve the upstream/project legal material required to redistribute the bundled MAME runtime.

Release qualification must verify that required COPYING/license/legal resources are included.

If release artifacts are built from a specific MAME source revision, the repository/release documentation must retain adequate source/provenance information for applicable redistribution obligations.

ROMs, CHDs, copyrighted game software, BIOS dumps not distributable with MAME, and other user-owned content are not part of this bundled-runtime payload.

## 13. CI and qualification strategy

### 13.1 Unit/backend tests

Tests must prove:

- missing external override resolves to bundled runtime when a valid bundled layout exists;
- explicit valid external override wins;
- clearing override returns to bundled;
- invalid bundled layout produces a package/runtime error rather than “not configured”;
- legacy settings migrate deterministically;
- bundled resource containment remains enforced;
- runtime identity changes stale metadata as intended.

### 13.2 Frontend/component/source tests

Tests must prove:

- normal General Settings does not require an executable path;
- bundled runtime status is visible;
- external path controls are hidden behind advanced override affordance;
- “Use bundled MAME” clears/reset override semantics;
- first-run state is initialization/import progress, not generic “MAME not configured.”

### 13.3 Real package qualification

At least one release-grade Linux CI path must build/stage a real MAME binary and produce the actual `.deb`.

Qualification must install that package into a clean environment and verify:

1. package installation succeeds;
2. bundled runtime is present;
3. bundled MAME `-version` succeeds;
4. bundled MAME metadata/list invocation succeeds;
5. the Tauri app starts without an executable override;
6. application/backend identity reports bundled runtime;
7. metadata bootstrap can reach a ready catalog;
8. no distro `mame` package is required.

The existing synthetic fixture package smoke may remain as a fast structural test but cannot substitute for this qualification.

### 13.4 Platform matrix

The primary release gate must cover supported Ubuntu and Debian environments to the extent CI infrastructure permits. Where a full graphical Debian environment is unavailable, the Debian package install/runtime checks must still be automated and any remaining graphical check must be explicitly documented rather than silently omitted.

## 14. Documentation requirements

The README and user-facing install documentation must state:

- the Debian/Ubuntu `.deb` contains MAME;
- users do not separately install or locate MAME;
- ROM/content paths remain user-supplied;
- an external MAME executable is an advanced override only;
- first-run metadata initialization may take time;
- how to diagnose a broken bundled runtime/package.

Developer documentation must explain how to build and stage the production MAME runtime and how synthetic test fixtures differ from release artifacts.

## 15. Non-goals

This work does not:

- bundle ROMs, CHDs, proprietary game software, or undistributable firmware;
- embed MAME emulation inside the Tauri process;
- move gameplay rendering/audio/input through Tauri IPC;
- eliminate advanced external-runtime support if it remains useful;
- require rewriting MAME’s own configuration subsystem;
- make package-owned runtime resources user-writable;
- bypass existing launch validation or session supervision.

## 16. Required architectural outcomes

The implementation is complete only when all of the following are true:

1. A clean production `.deb` installation contains a real functional MAME runtime.
2. First launch with no settings resolves bundled MAME automatically.
3. The normal UI never asks the user to locate MAME.
4. Metadata initializes or refreshes automatically for the bundled runtime.
5. The machine browser becomes usable without executable-path setup.
6. External MAME selection is clearly an advanced override.
7. Clearing an override restores bundled MAME.
8. Metadata/audit/launch operations consistently use the effective runtime identity.
9. Release CI validates a real MAME package, not only a synthetic fixture.
10. Debian/Ubuntu installation and uninstall semantics are qualified.
11. Legal/provenance material is preserved.
12. Documentation describes the one-package product accurately.

## 17. Completion rule

This effort is not complete merely because a bundled executable exists in an archive.

Completion requires an exact-head-qualified production candidate whose `.deb` installs the Tauri application and a real MAME runtime together, launches on a clean supported Debian/Ubuntu system without executable-path configuration, initializes metadata from the bundled runtime, reaches the machine browser, preserves the advanced override path, passes project/security/package/documentation qualification, is merged to `master`, and passes applicable post-merge `master` CI.
