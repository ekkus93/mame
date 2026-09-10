# MT-604 — General settings UI

Status: **closed**

## Qualified implementation

- Branch: `ralph/mt-604-general-settings-ui`
- Implementation commit: `642d112d03445e054d30538a4c3a8252258ef4b1`
- Implementation tree: `eade29099a3e9e065a79952374ed120698556ab2`
- Parent MT-603 closure: `632cb668d7cfa350873f43e4188a28d516fba17d`
- Authoritative Tauri qualification: GitHub Actions run `34462363825`

Run `34462363825` passed frontend formatting, lint, typecheck, tests and production build; Rust formatting and tests; the library UX performance qualification; Clippy; and lockfile-integrity verification.

## Delivered behavior

MT-604 adds a general settings surface backed by typed Rust-owned persistence and launch resolution. The UI exposes the configured MAME executable, existing content-path configuration, window/fullscreen preference, bounded renderer preference, and bounded audio preference.

Launch preferences are persisted in the existing settings schema with backward-compatible defaults. Existing settings that lack `launchPreferences` continue to deserialize as inherited/default behavior rather than requiring a destructive settings migration.

Machine and software launches resolve the persisted preferences in the backend before constructing MAME argv. The frontend does not inject arbitrary command-line flags. Renderer and audio values are constrained to the supported choices represented by the MT-601/MT-602 configuration model, and renderer choices are treated as requested preferences because MAME may still perform runtime fallback.

## Regression coverage

Coverage added or updated for:

- general-settings frontend command serialization;
- settings persistence/default compatibility;
- launch argv construction with fullscreen/window, renderer and audio preferences;
- machine/software launch plumbing through the same backend preference path;
- preservation of the existing session-supervisor test API under `#[cfg(test)]` only.

Two qualification failures were resolved before closure. Run `34461393406` exposed a missing `launch_preferences` field in a legacy `SettingsV2` test initializer plus unused compatibility code. A later run exposed legacy supervisor tests that still called `SessionSupervisor::launch`. The final implementation retains that compatibility wrapper only under `#[cfg(test)]`, while production launch paths use the preference-aware API and remain warning-free under Clippy.

## Acceptance criteria

- [x] executable.
- [x] content paths.
- [x] window/fullscreen preference.
- [x] renderer selection where supported.
- [x] audio preference where supported.
