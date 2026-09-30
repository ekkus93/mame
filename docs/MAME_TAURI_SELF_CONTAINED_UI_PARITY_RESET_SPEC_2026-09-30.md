# MAME Tauri Self-Contained UI Parity Reset Spec

**Date:** 2026-09-30  
**Status:** Draft reset spec — authoritative direction for getting the Tauri MAME project back on track.  
**Companion TODO:** `docs/MAME_TAURI_SELF_CONTAINED_UI_PARITY_RESET_TODO_2026-09-30.md`  
**Supersedes for execution priority:** scattered post-BMR/RPL work where that work conflicts with the product invariants below.  
**Related roadmaps:**

- `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`
- `docs/MAME_TAURI_ROM_PATH_AND_LAUNCH_PARITY_SPEC_2026-09-30.md`
- `docs/MAME_TAURI_ROM_PATH_AND_LAUNCH_PARITY_TODO_2026-09-30.md`

## Problem statement

The project drifted away from the original product goal: replace the existing MAME UI with a Tauri UI while keeping normal packaged use self-contained, sane, and recognizable as MAME.

The project must not require a normal user to separately install upstream MAME and point the Tauri app at that binary before the packaged app can run. The packaged application must own the bundled runtime path for the normal user flow. External/development MAME selection may remain as an advanced/developer escape hatch, but it must not be the default product path and must not be presented as a prerequisite for normal use.

The current product also accumulated custom launcher/catalog/control-plane behavior before native-MAME-equivalent behavior was locked down. That produced several user-visible problems:

1. The UI diverges from native MAME and contains prominent Tauri-only controls and optional artwork/media tabs that are not in the original UI.
2. Audit and launch have historically been able to use different ROM/content path contracts.
3. Start can be offered when ROM availability is unknown or unavailable.
4. Missing-ROM launches can surface backend/runtime-control internals instead of actionable product guidance.
5. Native MAME and Tauri list/filter counts can differ without a clear tested explanation.

This reset spec makes those failures explicit and narrows the project back to the original product contract.

## Product invariants

These invariants are non-negotiable for the reset work.

### 1. Packaged app is self-contained by default

A release/package build of the Tauri app must include and resolve the packaged MAME runtime and required runtime resources by default.

Normal packaged use must not require the user to:

- install upstream MAME separately;
- locate an external MAME binary;
- point the Tauri app at an external MAME binary;
- understand development-tree runtime paths;
- manually wire package runtime resource directories.

The packaged app may still allow external/development MAME sources for advanced debugging, development, or comparison, but that path must be visually and behaviorally secondary.

### 2. Native-MAME UI parity comes before feature growth

The main Tauri UI should visually and behaviorally track native MAME as closely as practical.

Any feature that is not present in the original/native UI must be one of:

- removed from the primary path;
- hidden behind an advanced/debug affordance;
- de-emphasized so it does not look like a required game control;
- explicitly documented as an intentional Tauri addition with tests.

Optional artwork/media categories must not appear to be required launch controls, ROM availability evidence, or proof that a game is runnable.

### 3. One effective content-path contract

The app must have one backend contract for effective content paths.

The same effective path set must drive:

- ROM/software/CHD availability audit;
- Start/launch argv construction;
- launch diagnostics;
- diagnostics/support output;
- user-facing configuration explanation.

Audit and launch must never silently use different path sets.

### 4. Start must not blindly launch unknown or unavailable content

The normal Start path must not launch a machine when the current app state says the ROM availability is `Unknown`, `Unavailable`, stale, or unaudited unless there is an explicit, tested override with clear copy.

The default behavior must be:

- `Available`: Start is enabled and launches with the exact effective path set used for audit.
- `BestAvailable` or equivalent acceptable audit state: Start is enabled only if explicitly mapped as playable by tested policy.
- `Unknown` / unaudited / stale audit: Start is gated and routes the user to audit/configuration guidance.
- `Unavailable` / missing required ROMs / incorrect required ROMs: Start is gated and explains missing/incorrect content.

### 5. Missing-ROM failures must be product errors, not backend internals

If MAME exits before runtime-control readiness because ROMs/content are missing or invalid, the primary user-facing error must explain the missing/invalid content problem and show useful next steps.

Internal runtime-control phrases such as a control channel closing before readiness may remain in diagnostics, but they must not be the primary visible message for ordinary missing-ROM or early-exit failures.

### 6. Native list/filter parity must be explained or fixed

The project must resolve the observed native-vs-Tauri count discrepancy. Native MAME reported `88 Games` for the observed view while Tauri showed `1-59 of 59` in a comparable view.

The reset must identify whether the difference comes from:

- parent/clone filtering;
- BIOS/device filtering;
- working/imperfect/preliminary status filtering;
- mechanical or non-game filtering;
- CHD-required filtering;
- category/metadata gaps;
- local ROM availability vs full catalog semantics;
- a Tauri query/import bug.

If the difference is intentional, it must be documented and tested. If not intentional, it must be fixed.

### 7. Package/integration behavior beats internal green checks

Rust unit tests and frontend tests are necessary but not sufficient. The final acceptance evidence must include package-level or integration behavior showing the packaged app starts from its bundled runtime path and behaves sanely with no ROM paths, configured ROM paths, missing ROMs, and available ROMs.

## Intended product flow

### First launch / no ROM paths configured

The app opens without asking for a MAME binary. It uses the bundled runtime for metadata/catalog behavior and clearly explains that ROM/content directories are not configured yet.

The user can browse catalog entries, but the UI must not imply that catalog entries are runnable until availability is established.

Start on unknown content should be gated or route to audit/configuration guidance.

### ROM paths configured

The user configures ROM/software/CHD paths in the app.

The app computes effective content paths, validates them, and uses the same ordered set for audit and launch.

### Available content

When audit establishes that a machine is available/playable, Start launches packaged MAME with the same effective content path set used by audit.

### Missing or unavailable content

When audit establishes that content is missing or incorrect, Start is blocked by default and the UI explains what is missing and how to configure content directories or run audit again.

### Advanced external MAME path

An external or development MAME executable path may remain available for developers and advanced users. It must be clearly separate from the normal packaged app path and must not be required for normal use.

## Architecture requirements

### Bundled runtime ownership

The package layer owns resolution of the bundled MAME executable and packaged runtime resources. The frontend must not self-assert bundled trust by passing an arbitrary path.

The backend must distinguish:

- packaged bundled runtime;
- development-tree runtime;
- user-configured external runtime.

Only the packaged bundled runtime is the normal release path.

### Effective content paths

The backend must expose or internally maintain an effective content-path model with, at minimum:

- schema version;
- path kind: ROM, software, CHD, artwork/media where applicable;
- path value;
- source: configured, discovered/default, package-provided, or user-state as applicable;
- validation status: accessible, missing, inaccessible, invalid, unknown;
- reason/details safe for diagnostics;
- deterministic ordering.

Initially, the reset can keep discovered/default ROM paths explicit as unsupported/deferred if package investigation shows no safe default should be honored. What is not acceptable is pretending the path set is unknown while audit/launch silently use different path logic.

### Audit and launch parity

Audit and launch must call into the same effective content-path contract. If MAME needs a semicolon-delimited path list for `-rompath`, construction and validation of that path list must be a typed backend operation, not arbitrary frontend string concatenation.

The launch target must include the effective ROM path list explicitly whenever required by the package/runtime model.

### Start gating

The backend or frontend must prevent normal launch for unknown, stale, or unavailable content. Prefer backend enforcement so tests can prove no MAME process is spawned for gated cases.

Acceptable states and transitions must be documented and tested.

### Diagnostics

Launch diagnostics must include enough evidence to debug missing ROMs without exposing unrelated filesystem data.

Required diagnostics include:

- selected machine;
- selected software/BIOs when applicable;
- runtime source kind and trust;
- effective content paths or redacted/safe path summaries;
- audit state and audit freshness;
- launch argv option summary;
- MAME stdout/stderr tail for early exits, bounded and redacted according to existing policy;
- internal runtime-control error only as secondary diagnostic detail.

## UI requirements

### Main library/list view

The main library/list view should match native MAME as closely as practical before custom Tauri features are promoted.

Required work:

- inventory every current Tauri-only control visible in the primary view;
- classify each as remove, hide, advanced, or justified addition;
- align list columns and labels with native-MAME semantics where practical;
- distinguish driver/emulation status from local ROM availability;
- avoid styling `Unknown` as playable.

### Details panel and artwork/media

Artwork/media categories such as Snapshots, Cabinet, Control Panel, PCB, Flyer, and Title Screen are optional assets. They must not appear as required controls or imply ROM availability.

When no artwork exists, empty categories should be hidden, disabled, or visually de-emphasized.

The details panel should prioritize:

1. title/description and core machine metadata;
2. ROM availability/audit state;
3. effective content path/configuration guidance;
4. Start/action state;
5. optional artwork/media.

## Testing requirements

The reset must add or update tests for:

- packaged runtime resolution without external MAME configuration;
- advanced external/development runtime path remaining non-default;
- effective content path construction and ordering;
- audit and launch receiving the same effective ROM path list;
- Start gating for unknown/stale/unavailable content;
- no MAME spawn when launch is gated;
- available content launch uses expected `-rompath`;
- missing-ROM/early-exit error classification and user-facing copy;
- native-vs-Tauri list/filter parity decisions;
- empty artwork/media handling;
- diagnostics containing effective content-path and early-exit evidence.

## Qualification requirements

Final closure requires exact-head evidence for:

- `Tauri project` green;
- package workflows affected by changed files green;
- security workflow green when Rust/packaging paths change;
- package-level smoke showing bundled runtime run-in-place behavior;
- no-ROM-path first-run behavior;
- configured-ROM-path audit/launch parity;
- unavailable/unknown Start gating;
- missing-ROM diagnostics;
- UI parity inventory and resulting UI changes.

## Explicit non-goals until parity is restored

Until this reset is complete, do not prioritize:

- additional custom catalog features;
- new artwork/media features;
- new external-MAME configuration surfaces;
- new diagnostics bundles beyond missing-ROM and launch failure evidence;
- new package distribution formats not needed for the self-contained app path;
- broad refactors unrelated to packaged runtime, UI parity, content paths, Start gating, diagnostics, or count parity.

## Acceptance criteria

This reset is complete only when all of the following are true:

1. A normal packaged user can launch the Tauri app without installing or selecting external MAME.
2. The primary UI is recognizably aligned with native MAME and any intentional differences are documented and tested.
3. Audit and Start use the same effective ROM/content path contract.
4. Unknown/unavailable content is not blindly launched by normal Start.
5. Missing-ROM or early-exit failures show actionable product copy instead of primary runtime-control internals.
6. The native-vs-Tauri count discrepancy is fixed or explained with tests.
7. Package-level CI/smoke evidence proves the self-contained run-in-place path.
8. The companion TODO is fully reconciled with exact SHAs, run IDs, and evidence.
