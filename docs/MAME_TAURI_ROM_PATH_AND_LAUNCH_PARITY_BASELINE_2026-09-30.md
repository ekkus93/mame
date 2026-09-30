# MAME Tauri ROM Path and Launch Parity Baseline

**Date:** 2026-09-30  
**Roadmap:** `docs/MAME_TAURI_ROM_PATH_AND_LAUNCH_PARITY_TODO_2026-09-30.md`  
**Spec:** `docs/MAME_TAURI_ROM_PATH_AND_LAUNCH_PARITY_SPEC_2026-09-30.md`  
**Baseline repo head inspected:** `2e524aef61f17905fb406b3d96ea519d3662a0bb`

This baseline records the current source-level evidence for the observed installed-package UX issues: the native MAME UI showed `88 Games`, the Tauri frontend showed `1-59 of 59`, Tauri rows had ROM availability `Unknown`, pressing Start surfaced `The MAME runtime-control channel closed before it became ready`, and the optional artwork/detail tabs were visually prominent even without artwork or audited ROM availability.

The completed bundled-runtime roadmap remains closed. This document is a post-BMR inventory for ROM path parity, launch diagnostics, native-MAME list/filter parity, and artwork/details UX.

---

## Installed package and artifact baseline

The relevant package-bearing baseline from the completed BMR work is:

- package implementation head: `818426f90aa15fdcbff5c9dd55fc614ee7681934`;
- real-runtime package workflow: `Tauri Linux real runtime package` run `36660846147`;
- artifact: `real-bundled-mame-linux-packages` artifact `11075618123`, `156,292,632` bytes;
- artifact contents: `.deb`, `.AppImage`, and `runtime-provenance.txt`;
- final BMR reconciliation head: `1e7fa175e349c69f69e33f9e6bbf5c0aa521524e`, with exact-head Tauri project and documentation CI green.

The RPL roadmap was created at head `2e524aef61f17905fb406b3d96ea519d3662a0bb`.

---

## Source inventory

### Settings content paths

`tauri/src-tauri/src/config.rs` defines:

- `SettingsV2`, whose default sets `mame_executable: None` and `content_paths: ContentPathsV1::default()`;
- `ContentPathsV1`, whose default has empty `rom_paths`, `software_paths`, and `chd_paths`;
- `PlatformPath` serialization/deserialization and path validation primitives.

This confirms that a fresh install can have no explicit Tauri ROM/software/CHD paths.

### Content path configuration UI/API

`tauri/src-tauri/src/path_configuration.rs` exposes:

- `get_content_path_configuration`;
- `set_content_path_configuration`;
- `pick_content_directory`.

It validates persisted configured paths, but it does not currently report a richer effective path contract such as configured paths plus discovered/default paths and their source reasons.

### Audit inputs

`tauri/src-tauri/src/library/audit.rs` resolves audit context from the active metadata generation, effective MAME executable, and persisted settings. It passes `load_settings(settings_path)?.content_paths` to `audit_machine`.

`tauri/src-tauri/src/mame/audit_runner.rs` builds audit argv with:

- `-noreadconfig`;
- optional `-rompath <media-search-path>` if any configured ROM/software/CHD path exists;
- `-verifyroms <machine>`.

The audit search path is composed from `rom_paths`, then `software_paths`, then `chd_paths`, preserving user order within each group. If the configured path list is empty, the audit argv does not include `-rompath` and the existing test calls this "MAME builtin media default" behavior.

### Launch argv construction

`tauri/src-tauri/src/mame/argv.rs` builds launch argv from machine/software/BIOS and typed `ProjectPathArgument` values. It validates each project-controlled path as one absolute path and rejects path-list separators such as `;`.

This is a deliberate safety boundary, but it means a future `rompath` implementation needs an explicit path-list contract. Simply stuffing a semicolon-delimited path list into the existing `ProjectPathArgument.path` field would violate the current validator.

### Bundled launch project paths

`tauri/src-tauri/src/sessions.rs` appends bundled package/resource paths only when launching the bundled runtime. It currently adds:

- `hashpath` pointing at the package-owned bundled hash directory;
- `bgfx_path` pointing at the package-owned bundled BGFX directory;
- user-writable state/config directories such as `inipath`, `cfg_directory`, `nvram_directory`, `input_directory`, `state_directory`, `snapshot_directory`, `diff_directory`, and `comment_directory`.

The bundled launch path does not currently append a `rompath` from `SettingsV2.content_paths`, and its unit test asserts that bundled launch paths contain no `rompath`.

### Runtime-control startup failure

`tauri/src-tauri/src/sessions/supervisor_runtime.rs` returns `CONTROL_CHANNEL_CLOSED` with message `The MAME runtime-control channel closed before it became ready.` when stdout/control closes before the ready marker is observed.

That error accurately describes the supervisor protocol state, but it is not a good primary user-facing diagnosis for ordinary early MAME exit, especially missing/invalid ROM content.

---

## Confirmed current failure class

The source inventory confirms a concrete audit/launch parity gap:

1. Audit uses `SettingsV2.content_paths` to build an optional `-rompath`.
2. Bundled Start launch appends package resource/user-state paths but does not append configured ROM/software/CHD paths as an explicit `rompath`.
3. Therefore a path that is used for audit is not guaranteed to be used for Start.
4. If no Tauri content paths are configured, both audit and launch have no explicit Tauri ROM path model to show the user.
5. If MAME exits early during Start, the frontend can expose the internal runtime-control closed-before-ready message instead of an actionable ROM/content diagnostic.

The next implementation slice should not bypass the existing argv safety model. It should introduce an effective content-path/path-list contract that both audit and launch share, and it should make `rompath` list validation explicit rather than treating a semicolon-delimited list as an ordinary single path.

---

## Baseline conclusion

RPL should proceed in this order:

1. Define a backend effective content-path contract and path-list validation model.
2. Refactor audit and launch to share that contract.
3. Add Start gating for `Unknown`/`Unavailable` ROM availability.
4. Map early MAME exit to actionable launch diagnostics with stderr/stdout tail and effective path evidence.
5. Investigate the native `88 Games` vs Tauri `59` count mismatch and the optional artwork/details panel UX.
