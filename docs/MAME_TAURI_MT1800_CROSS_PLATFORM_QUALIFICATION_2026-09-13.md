# MT-1800 — Cross-platform qualification

Date: 2026-09-13  
Branch: `ralph/mt-1800-cross-platform-qualification`

## Scope

MT-1800 closes the supported cross-platform qualification ledger for the external-window Tauri frontend. The goal is not to claim embedded rendering, synthetic audio playback, or physical controller coverage in CI. The goal is to make every platform claim explicit, tie automated claims to GitHub Actions evidence, and record runtime-boundary claims where MAME or the operating system owns the behavior.

## Added executable evidence

This branch adds `scripts/tauri/mt1800_qualification.py`, which generates a schema-versioned MT-1800 qualification artifact. It also adds `scripts/tauri/test-mt1800-qualification.py`, which proves offline that the ledger:

- includes every MT-1801 through MT-1804 requirement;
- records Linux, Windows, macOS, and cross-platform coverage counts;
- distinguishes `automated-ci`, `delegated-runtime-boundary`, and `documented-boundary` coverage;
- rejects malformed candidate SHAs;
- can write the exact JSON artifact uploaded by CI.

The Tauri workflow now runs the regression test and uploads `mame-tauri-mt1800-cross-platform-qualification-${{ github.sha }}` from `artifacts/cross-platform/`.

## Coverage model

| Coverage class | Meaning |
| --- | --- |
| `automated-ci` | A GitHub Actions workflow or checked-in regression test exercises the behavior. |
| `delegated-runtime-boundary` | The external-window architecture deliberately delegates this behavior to MAME or OS runtime APIs; CI verifies Tauri does not route or intercept that data path. |
| `documented-boundary` | The limitation or unsupported claim is explicitly documented so release qualification cannot silently overclaim it. |

## Linux qualification

| Requirement | Closure |
| --- | --- |
| Development build | Covered by `Tauri project / linux-release-qualification / Tauri development window smoke check` under Xvfb. |
| Packaged build | Covered by `Tauri Linux packaging` with Debian and AppImage build plus install/smoke coverage. |
| External MAME executable | Covered by path/executable model tests and no-shell execution policy. |
| Bundled sidecar | Covered by MT-1305 synthetic runtime staging, Linux bundled path validation, package install, and uninstall smoke. |
| Controller input | Delegated to the launched MAME process. CI does not claim physical controller validation. |
| Audio | Delegated to MAME and the OS audio stack. Tauri does not route PCM. |
| Fullscreen | Delegated to MAME/window-manager behavior for the external-window product line. |
| X11 | Covered by Xvfb-backed smoke in Linux release qualification. |
| Wayland | Documented as an OS/display-server boundary unless a future workflow explicitly claims native Wayland smoke coverage. |

## Windows qualification

| Requirement | Closure |
| --- | --- |
| Development build | Windows-specific Rust/runtime layout tests build under the Windows toolchain before packaging. |
| Packaged build | NSIS package is built, installed, payload-validated, uninstalled, and checked for residual payload/registry state. |
| Controller input | Delegated to MAME/Windows input APIs; CI does not claim physical controller validation. |
| Audio | Delegated to MAME/Windows audio APIs; Tauri packaging validates installation, not synthetic audio playback. |
| Fullscreen | Delegated to MAME/windowing behavior for the external-window product line. |
| Renderer behavior | Renderer ownership stays with MAME. Embedded/native-renderer claims remain outside MT-1800. |

## macOS qualification

| Requirement | Closure |
| --- | --- |
| Development build | macOS-specific Rust/runtime layout tests build under the macOS toolchain before packaging. |
| Signed packaged build | Ad-hoc signed app bundle and DMG are built and smoke-tested without Apple release credentials. |
| Controller input | Delegated to MAME/macOS input APIs; CI does not claim physical controller validation. |
| Audio | Delegated to MAME/CoreAudio; CI does not synthesize audio output. |
| Fullscreen | Delegated to MAME/windowing behavior for the external-window product line. |
| Notarization/install behavior | CI proves credentialless release fails closed and ad-hoc installation works; real notarization remains gated on Apple credentials. |

## Cross-platform configuration portability audit

| Requirement | Closure |
| --- | --- |
| Path differences | Platform-native path handling is covered by MT-501 and OS-specific bundled-runtime validation. |
| Settings migration | Settings schema migration is tested in Rust and uses native path representation. |
| Controller identity differences | Tauri does not normalize or persist controller identities in the external-window release; future controller profiles must add their own migration tests. |
| Artwork paths | Local artwork roots use configured filesystem paths with missing-art fallback and bounded UI behavior. |

## Closure

MT-1800 is closed when the branch head has a green Tauri project run showing the MT-1800 ledger regression and artifact upload, plus the platform packaging workflows remain green on the same candidate head or on the promoted master head.
