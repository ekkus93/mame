# MAME Tauri Bundled Runtime Security Review

**Date:** 2026-09-28  
**Specification:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_SPEC_2026-09-27.md`  
**Checklist:** `docs/MAME_TAURI_BUNDLED_MAME_DISTRIBUTION_TODO_2026-09-27.md`  
**Reviewed SHA:** `203f01af0526ff241d080cede0d36578c2c29d38`

This review covers the security-relevant changes made by the bundled-runtime default path. It focuses on runtime selection, package resource containment, WebView authority, process launch, diagnostics, and user-writable MAME state.

## Security boundaries preserved

### Runtime authority remains Rust-owned

The frontend can still request only typed external/development-tree executable sources through the existing launch boundary. A release-qualified bundled runtime is not exposed as a frontend-selectable arbitrary path. Bundled trust is created only by the Rust/package layer after resolving Tauri's resource directory and validating the `BundledRuntimeLayout`.

Relevant paths:

- `tauri/src-tauri/src/effective_runtime.rs`
- `tauri/src-tauri/src/bundled_runtime.rs`
- `tauri/src-tauri/src/app.rs`
- `tauri/src-tauri/src/sessions.rs`

### No generic shell/process API was added

The bundled-runtime work reuses existing typed launch operations and MAME argument builders. It does not add a WebView command that accepts arbitrary shell fragments, arbitrary argv vectors, or untyped process execution requests. BIOS, machine, software, and project-path values remain validated Rust-side before launch.

Relevant paths:

- `tauri/src-tauri/src/mame.rs`
- `tauri/src-tauri/src/mame_ui_launch.rs`
- `tauri/src-tauri/src/software.rs`
- `tauri/src-tauri/src/sessions.rs`

### External override remains explicit and validated

External runtime selection remains an advanced override. Clearing the override returns to bundled/default semantics rather than disabling runtime resolution. Invalid external override state fails closed and presents recovery to bundled runtime instead of silently launching an unvalidated path.

Relevant paths:

- `tauri/src-tauri/src/config.rs`
- `tauri/src-tauri/src/general_settings.rs`
- `tauri/src/settings/GeneralSettingsPanel.tsx`

## Package resource containment

Bundled runtime resolution uses Tauri's package resource directory and `BundledRuntimeLayout`; it does not depend on current working directory, `PATH`, or a developer checkout. The layout validator enforces the package-owned executable, `hash`, `bgfx`, and legal-resource structure before runtime use.

For bundled launches, package-owned `hash` and `bgfx` resources are supplied as MAME project paths, while writable MAME state is redirected to the platform application data directory under `mame-state`. This avoids writes under `/usr`, the Tauri resource directory, or an AppImage mount for ordinary generated state such as INI, CFG, NVRAM, input, state, snapshot, diff, and comments.

Relevant paths:

- `tauri/src-tauri/src/bundled_runtime.rs`
- `tauri/src-tauri/src/sessions.rs`
- `scripts/tauri/stage-mame-runtime.sh`
- `scripts/tauri/prepare-bundled-mame-runtime.sh`

## Synthetic versus real runtime evidence

Synthetic MAME fixtures remain explicitly structural. Release-qualified real-runtime staging requires real-mode inputs, rejects synthetic-looking version output, runs a bounded `-version` and `-listxml pacman` validation, and records runtime provenance. The real-runtime `.deb` qualification workflow exists but is not counted as final release evidence until it is actually dispatched and passes for the target candidate.

Relevant paths:

- `scripts/tauri/validate-real-mame-runtime.sh`
- `scripts/tauri/prepare-bundled-mame-runtime.sh`
- `scripts/tauri/build-linux-bundled-mame-runtime.sh`
- `.github/workflows/tauri-linux-real-runtime-qualification.yml`

## Diagnostics and data exposure

Runtime diagnostics now classify unavailable runtime state as bundled-package, external-override, configuration, or legacy no-runtime domains. The diagnostic snapshot surfaces useful runtime provenance and stable error categories, but path-like context is still sanitized by the existing diagnostics sanitization path where general log/event details are recorded.

Relevant path:

- `tauri/src-tauri/src/diagnostics.rs`

## Residual open security/compliance work

The following items remain open until real-runtime `.deb` qualification is executed on a final candidate:

- verify the final production package contains the required MAME `COPYING` and legal resources from the real source tree;
- verify the installed real-runtime package operates with package-owned resources read-only;
- verify dependency augmentation does not introduce a distro `mame` dependency;
- record the final package artifact identity, bundled MAME version/provenance, and install/uninstall evidence.

## Conclusion

The bundled-default runtime path does not weaken the existing Rust-owned process/filesystem boundary. The main remaining risk is not a new WebView/process capability; it is incomplete release evidence until the real-runtime `.deb` workflow is run on the final production candidate.
