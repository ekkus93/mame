# MAME Tauri ROM Path and Launch Parity Specification

**Date:** 2026-09-30  
**Status:** Draft implementation specification  
**Related completed roadmap:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`  
**Companion TODO:** `docs/MAME_TAURI_ROM_PATH_AND_LAUNCH_PARITY_TODO_2026-09-30.md`

This specification defines the follow-up work required after bundled-runtime packaging closure. The bundled Linux package now ships a real MAME runtime and can import metadata, but user-visible testing against a normal installed package exposed product gaps in ROM path discovery, launch gating, launch-error diagnosis, catalog/filter parity, and artwork/details-panel clarity.

This work must not reopen the bundled-runtime packaging roadmap. It starts from the premise that the real bundled runtime exists and focuses on making the installed frontend behave predictably for a normal user who has ROMs in MAME-compatible locations or needs clear guidance when ROMs are missing.

---

## Observed user-facing failures and mismatches

### 1. Catalog entries appear although ROMs are not known to be available

The Tauri frontend imports MAME machine metadata and displays catalog rows before ROM availability has been audited. In the observed installed package, the machine list shows many rows with availability `Unknown` and a banner saying that ROM availability has not been audited. Pressing **Start** on such a row can still attempt a supervised MAME launch.

The intended product distinction is:

- **Catalog presence** means bundled MAME knows the machine driver from metadata such as `-listxml`.
- **Driver status** such as Working, Imperfect, Preliminary, or Not Working is an emulation/driver property, not proof that ROM content is present.
- **ROM availability** is a separate local-content result and must be explicit before the UI implies a game is runnable.

### 2. Tauri launch can show an internal control-channel error instead of the real startup cause

The observed error was:

```text
The MAME runtime-control channel closed before it became ready.
```

Repository inspection shows this is the `CONTROL_CHANNEL_CLOSED` startup path in the supervised runtime-control protocol. It means the frontend launched a MAME process, but stdout/control closed before the authenticated ready marker was observed. That is a backend lifecycle fact, but it is not a useful primary diagnosis for a normal user.

For ordinary launch failures, especially missing or invalid ROM content, the frontend must surface the underlying MAME startup failure using the captured stderr/stdout tail and a user-actionable message.

### 3. Tauri may not honor MAME's normal ROM search behavior by default

Repository inspection indicates `SettingsV2::default()` initializes empty content paths and `ContentPathsV1::default()` initializes empty `rom_paths`, `software_paths`, and `chd_paths`. The bundled launch path appends package-owned runtime resources such as `hashpath` and `bgfx_path`, and user-writable state directories such as `cfg`, `nvram`, `state`, and `snap`, but the inspected bundled launch path did not show an automatic default `rompath` equivalent.

The product must either honor MAME-compatible default ROM locations, import/resolve MAME's effective `rompath`, or make it unmistakably clear that no ROM paths are configured. Silent launch attempts with no effective ROM path are not acceptable.

### 4. Native MAME UI and Tauri frontend counts/filters differ

Observed screenshots showed native MAME reporting `88 Games` while the Tauri frontend reported `1-59 of 59` for a visually comparable view. The native MAME UI and the Tauri frontend do not need identical layout, but filter labels must have clear semantics and any intentional count divergence must be explainable.

Potential divergence dimensions include parent/clone handling, BIOS vs non-BIOS rows, mechanical and non-mechanical filters, Working/Imperfect/Preliminary status mapping, category data availability, and whether the view is catalog-first or playable-library-first.

### 5. Right-side artwork/details tabs are unclear

The Tauri UI shows tabs/buttons such as **Snapshots**, **Cabinet**, **Control Panel**, **PCB**, **Flyer**, and **Title Screen**. These are optional artwork/media categories, not required runtime controls and not proof of ROM availability. When no artwork exists, the empty panel and prominent tab styling can distract from the launch/availability workflow.

---

## Goals

1. Make the installed Tauri package find ROMs in the same practical locations a MAME user expects, or clearly state which effective ROM paths are active.
2. Prevent `Unknown` or `Unavailable` ROM availability from looking equivalent to a playable game.
3. Replace internal runtime-control startup errors with actionable launch diagnostics for ordinary user failures.
4. Reconcile or document native-MAME vs Tauri catalog/filter count differences.
5. Clarify optional artwork/detail tabs and reduce empty/noisy UI in normal usage.
6. Preserve the completed BMR invariants: bundled runtime by default, no ROM redistribution, explicit advanced external override, Rust-owned runtime resolution and launch, package-owned resources read-only, user state outside package resources.

---

## Non-goals

- Do not bundle ROMs, CHDs, BIOS images, game software, or undistributable firmware.
- Do not weaken executable/source validation or let the webview self-assert bundled-runtime trust.
- Do not remove the metadata/catalog browser; catalog browsing is valid when clearly labeled.
- Do not make optional artwork mandatory for launch.
- Do not require exact pixel-for-pixel parity with native MAME's internal UI.
- Do not reopen the completed bundled-MAME packaging roadmap except by cross-reference.

---

## Requirements

### RPL-001 — Effective ROM path discovery and visibility

The frontend/backend must compute an explicit effective content-path model for the installed package. It must include configured user paths and any supported default/discovered ROM paths. The UI must show the effective ROM path list, validation status, and whether the list is empty.

The implementation must investigate at least these sources:

- persisted Tauri `contentPaths.romPaths`;
- MAME's default or reported `rompath` behavior for the bundled runtime;
- platform-appropriate user data/config locations used by MAME-style installs;
- current working directory assumptions that native MAME might use but the packaged Tauri app should not rely on silently.

If a path is included by default, the reason/source must be reportable, for example `configured`, `mameDefault`, `legacyUserLocation`, or `packageDetected`.

### RPL-002 — Audit and launch must use the same content paths

ROM audit and Start launch must use the same effective ROM path contract. A machine may not audit against one set of paths and launch against another unless the UI explicitly reports the difference.

The launch argv must include the effective ROM path when required rather than relying on an uncontrolled process working directory.

### RPL-003 — Start gating by availability state

The Start action must distinguish these states:

- **Available:** launch may proceed.
- **Unknown:** default action should prompt to audit/configure paths before launch, with an explicit override only if product owners intentionally allow attempting anyway.
- **Unavailable:** do not launch blindly; show missing/unavailable content diagnostics and configured paths.
- **Audit stale:** prompt to re-audit before launch if the configured/effective ROM path set changed after the last audit.

Any override path must be explicit and tested; the normal flow should not produce a runtime-control failure for known missing content.

### RPL-004 — Launch failure diagnostics

If MAME exits before the control channel is ready, the user-facing error must include:

- a high-level message such as `MAME exited before startup completed`;
- likely cause classification when available, especially missing ROMs or invalid ROM path configuration;
- captured stderr/stdout tail from the failed MAME process;
- the selected machine short name;
- the effective ROM paths used for the launch;
- a diagnostic error code distinct from the raw control protocol state.

The raw `CONTROL_CHANNEL_CLOSED` code may remain in developer diagnostics, but it must not be the primary user-facing explanation for ordinary startup failure.

### RPL-005 — Native-MAME catalog/filter parity investigation

The implementation must compare native MAME's list/count behavior with the Tauri catalog for at least one reproducible bundled-runtime metadata snapshot. It must record which differences are intentional and fix unintentional mismatches.

Minimum parity dimensions:

- parent vs clone inclusion;
- BIOS vs non-BIOS rows;
- Working, Imperfect, Preliminary, and Not Working status mapping;
- mechanical vs non-mechanical rows;
- CHD-required and save-supported filters;
- catalog count vs playable/available count.

### RPL-006 — Clear catalog vs playable-library modes

The UI must make it clear whether the current list is a catalog view or a playable-library view.

Acceptable designs include:

- a visible mode label such as `Catalog` vs `Playable`;
- defaulting the first-run view to a guided audit/configuration state;
- making the `Available` filter the primary playable view once audit data exists;
- explaining that `Working` means emulation-driver status, not local ROM availability.

### RPL-007 — Artwork/details panel clarity

The right-side tabs/buttons must be clearly labeled as optional artwork/media. Empty categories should be hidden, disabled, or explicitly marked as missing rather than presented as important controls.

The details panel should not obscure launch prerequisites. It should prioritize ROM availability, effective ROM paths, and audit status before optional artwork in first-run/unaudited states.

### RPL-008 — Diagnostics and support bundle

The diagnostics bundle must contain enough information to diagnose launch failures without requiring the user to infer internal protocol state:

- app version and bundled MAME identity;
- selected machine/software/bios;
- effective content paths and their validation status;
- launch argv with sensitive/user paths redacted only as required by existing policy;
- MAME stdout/stderr tail;
- audit state and freshness;
- raw internal error code and mapped user-facing diagnostic.

### RPL-009 — Regression coverage

Add automated tests and/or package smoke checks covering:

- empty content paths report unaudited/unconfigured state;
- default/discovered ROM paths are included or explicitly absent with a clear reason;
- audit and launch share the same effective path model;
- Unknown/Unavailable launch gating;
- MAME exits before control readiness maps to actionable user-facing diagnostics with stderr tail;
- filter/count parity fixtures or documented intentional divergence;
- artwork tabs are optional and do not imply ROM availability.

---

## Acceptance criteria

This roadmap is complete when a normal installed-package user can understand and control ROM discovery without reading source code:

1. The app shows which ROM paths it will audit and launch with.
2. The app either discovers sensible MAME-compatible default ROM paths or clearly explains that none are configured.
3. Starting a machine with missing/unknown ROM content produces an audit/configuration path, not an internal control-channel error.
4. Native-MAME vs Tauri count/filter differences are either fixed or explicitly documented with tests.
5. Optional artwork buttons are clearly optional and do not distract from ROM availability/launch readiness.
6. Exact-head CI passes for implementation, tests, documentation, and any affected package smoke checks.
