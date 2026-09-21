# MAME Tauri frontend fork

This repository is a fork of upstream [MAME](https://www.mamedev.org/) with a project-owned React/Tauri desktop frontend under `tauri/`.

The product scope is a **production-useful external-window MAME frontend that preserves the recognizable layout and interaction model of the original MAME selector while improving Linux desktop-environment compatibility**. MAME remains a separately supervised native process and owns emulation, rendering, audio, timing, and gameplay input. The Tauri application owns machine/software browsing, configuration, metadata, audit/status surfaces, artwork, save-state management, session control, and project-specific library features.

## Current product UI

The default ready-state UI reproduces the information architecture and interaction model of MAME's machine/software selector rather than the older engineering-dashboard composition.

The primary workspace is organized around the selected machine:

```text
+--------------------------------------------------------------------------------+
| MAME / navigation                                  Search [__________________] |
+--------------------+--------------------------------------+--------------------+
| FILTERS            | MACHINES                             | IMAGES / INFO      |
|                    |                                      |                    |
| Unfiltered         | 1942                                 | selected artwork   |
| Available          | 1943                                 | or machine info    |
| Favorites          | Asteroids                            |                    |
| Working            | Centipede                            |                    |
| Parents            | Donkey Kong                          |                    |
| ...                | Galaga                               |                    |
|                    | Pac-Man                              |                    |
+--------------------+--------------------------------------+--------------------+
| active session / MAME version / app status                                      |
+--------------------------------------------------------------------------------+
```

Machine filters are persistent on the left, the dense machine list is the dominant center surface, and Images/Info is the right-side context. Search is live and bounded. Favorite, Audit, Export, Configure, Software, and Start are contextual to the selected machine. Software selection uses its own dense contextual browser and returns to the same machine state.

At narrow desktop widths, the machine list remains primary and an explicit **Details** control reveals the right-side context.

The old temporary **Legacy UI** route and vertical dashboard composition have been retired.

## Current project status

The main engineering phase and post-closeout hardening phase are complete. The current original-MAME UI parity pass is tracked by:

- `docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md` — current visual/interaction parity contract;
- `docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md` — current execution and reconciliation ledger;
- `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md` — repo-owned visual contract and review checklist;
- `docs/MAME_TAURI_LINUX_DESKTOP_COMPATIBILITY_2026-09-21.md` — Linux/WebView compatibility boundary and manual verification guidance.

The earlier MAME-UI reproduction milestone remains useful historical context:

- `docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md` — reproduction specification;
- `docs/MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md` — canonical upstream behavior inventory and initial dispositions;
- `docs/MAME_TAURI_MAME_UI_REPRODUCTION_TODO_2026-09-14.md` — milestone reconciliation ledger;
- `docs/MAME_TAURI_MAME_UI_ACCESSIBILITY_QUALIFICATION_2026-09-15.md` — accessibility/usability audit;
- `docs/MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md` — final implementation/defer record and qualification protocol.

Earlier engineering closure records remain authoritative for the completed external-window architecture:

- `docs/MAME_TAURI_TODO_2026-09-08.md`;
- `docs/MAME_TAURI_MT2200_ENGINEERING_CLOSURE_2026-09-13.md`;
- `docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_SPEC_2026-09-13.md`;
- `docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_TODO_2026-09-13.md`.

The deliberately deferred architecture research tracks remain MT-1000 native-window/embedded-render integration, MT-1100 dedicated Tauri MAME OSD, MT-1200 in-process MAME hosting, and MT-1705 embedded-render performance qualification. They are not part of the external-window product claim.

## MAME-style browser capabilities

Implemented browser capabilities include:

- imported `-listxml` machine metadata stored in the local catalog;
- canonical fixed machine filters for availability, working state, mechanical state, BIOS state, parent/clone, manufacturer, year, source file, save-state support, CHD presence, orientation, and Favorites;
- debounced live machine search;
- dense paged keyboard-navigable machine list;
- durable last-machine/filter/right-panel/artwork state;
- all 17 canonical MAME artwork categories plus project Icon/System Image extensions;
- safe clone-to-parent artwork fallback;
- compact machine Info context;
- contextual selected-machine audit;
- bounded displayed-list CSV export through a native save dialog;
- machine-specific launch settings and controller-profile overrides;
- contextual software-list browser;
- software search plus parent/clone/year/publisher/support filters;
- typed multipart software-part selection;
- authoritative Start Empty legality;
- authoritative bounded BIOS discovery and validated BIOS launch;
- supervised machine/software launch;
- active-session status and runtime controls;
- application-managed save-state listing/save/load/delete/compatibility checks;
- secondary Collections, Recent History, bulk Audit, global Settings, and Diagnostics surfaces.

Deliberate MAME-parity gaps are documented in the closure record. They are not silently represented as implemented features.

## Intentional deviations from original MAME UI

The default selector intentionally stays close to original MAME, but this frontend is not a byte-for-byte or toolkit-for-toolkit clone. Current intentional differences are limited to implementation or compatibility constraints:

- the selector chrome is rendered in a Tauri WebView rather than the original native MAME UI toolkit;
- secondary project tools such as Diagnostics, Collections, Recent History, and bulk Audit remain available behind non-default secondary surfaces;
- narrow desktop widths use an explicit **Details** affordance instead of forcing the full three-column layout off-screen;
- Category and Custom Filter remain visible in the original-like filter order but are marked deferred until authoritative data/persisted composite-filter support exists;
- small font-metric, anti-aliasing, DPI, and WebView compositor differences are accepted when they do not alter the original interaction model;
- automated screenshot baselines are deferred because hosted CI does not currently provide a stable cross-platform WebView pixel-baseline harness. Static/component tripwires and the repo-owned textual visual contract are the qualified fallback.

These deviations must not be used as permission to redesign the default browser into a generic dashboard.

## Local visual-parity verification

Run the desktop app:

```bash
cd tauri
npm ci
npm run tauri -- dev
```

For a configured-installation parity check, configure a MAME executable, import metadata, and verify the default browser against `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md`. In particular, confirm the dark/navy surface, compact blue toolbar, left filters, dense center list, blue/yellow selection, right Images/Infos panel, green driver/status region, visible focus treatment, and original-like keyboard/mouse navigation.

Also exercise the not-configured, metadata-import-needed, import-in-progress/failure, empty-catalog, and unknown-ROM-availability states; they must remain inside the same MAME-style shell rather than falling back to a generic light page.

### Capturing or updating reference screenshots

Use the host desktop's normal screenshot tool while the Tauri window is at a stable size. Capture at minimum:

1. a configured browser with a selected machine and the Images/Infos panel visible;
2. a startup/metadata empty state;
3. any intentional-deviation state that materially changes layout.

When a binary-capable repository path is available, place captures under a documented `docs/reference/` path and update `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md` with the exact filenames, capture date, window size, desktop environment, display server (Wayland/X11 where applicable), and the source commit SHA. Do not replace the textual checklist with screenshots; keep both so future diffs can be reviewed even when image tooling is unavailable.

## Repository layout

Important project-owned paths:

```text
README.md
tauri/
  src/App.tsx                         # thin readiness/bootstrap composition
  src/shell/                          # MAME shell and contextual shell styling
  src/browser/                        # machine/software browsers and models
  src/backend/                        # typed TypeScript Tauri-command wrappers
  src/library/                        # retained audit/favorites/history/collection logic
  src/settings/                       # global/machine/controller configuration surfaces
  src/session/                        # session/runtime/save-state surfaces
  src-tauri/src/                      # Rust authority: catalog, launch, filesystem, export, control
docs/MAME_TAURI_*                     # specs, ledgers, qualification and closure records
scripts/tauri/                        # regression/qualification helpers
.github/workflows/tauri-project.yml   # main quality + Linux runtime qualification
.github/workflows/tauri-security.yml
.github/workflows/tauri-*-packaging.yml
```

`App.tsx` must remain a thin readiness/composition layer. Primary UX belongs to `MameShell`/browser components; privileged authority belongs to Rust.

## Security/process boundary

The WebView is not treated as a privileged filesystem/process environment.

Key invariants:

- Rust crate uses `#![deny(unsafe_code)]`;
- MAME runs as a supervised child process;
- executable/catalog/audit/export/artwork/save-state operations remain typed Rust commands;
- no generic shell command bridge is exposed;
- no unrestricted filesystem, opener, or HTTP capability is added for UI convenience;
- displayed-list export supplies only typed browser query state, uses a native save dialog, and streams bounded catalog results in Rust;
- artwork serving remains bounded to configured local roots with containment checks;
- runtime-control frames remain authenticated, bounded, and session-scoped;
- gameplay video, PCM audio, timing, and gameplay input are not proxied through Tauri IPC;
- frontend keyboard shortcuts fail closed whenever native MAME owns gameplay input or ownership cannot be established.

## Developer prerequisites

### All platforms

Install:

- Node.js `>=22.12.0`;
- npm matching the checked-in `tauri/package-lock.json` workflow;
- Rust stable with `cargo`, `rustfmt`, and `clippy`;
- the normal Tauri 2 platform toolchain;
- a MAME executable for real interactive use.

The JavaScript dependency graph is locked by `tauri/package-lock.json`; the Rust graph is locked by `tauri/src-tauri/Cargo.lock`.

### Linux prerequisites

The CI Linux jobs install:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  xvfb \
  xdotool
```

`xvfb` and `xdotool` are used by the real Tauri development-window smoke path.

### Known Tauri 2 Linux GLib advisory

The checked-in Tauri 2 Linux GTK3 dependency graph currently resolves through `tauri 2.11.5`, `gtk 0.18.2`, and `webkit2gtk 2.0.2` to Rust `glib 0.18.5`. RustSec reports `RUSTSEC-2024-0429` as an informational `unsound` advisory affecting `glib::VariantStrIter` iterator methods.

The application does not directly call the affected `VariantStrIter` API. Do **not** try to fix the advisory by forcing an incompatible newer `glib` crate alongside the current GTK3 graph. Keep the advisory visible until the supported upstream Tauri Linux stack migrates to compatible newer GTK/GLib bindings. Reassess sooner if a reproducible Linux GLib/GVariant runtime failure appears.

### macOS and Windows

Use the normal Tauri 2 platform prerequisites plus Node.js and Rust stable. CI validates macOS app/DMG mechanics and Windows NSIS install/payload/uninstall behavior. Public release signing/notarization requires real platform credentials outside this repository.

## Initial setup and development

```bash
cd tauri
npm ci
```

Run the browser development server:

```bash
npm run dev
```

Run the Tauri desktop app:

```bash
npm run tauri -- dev
```

Build the frontend:

```bash
npm run build
```

Run a production Tauri build smoke without packaging:

```bash
npm run tauri -- build --no-bundle
```

Build platform bundles where the current host supports them:

```bash
npm run tauri -- build
```

## Validation

Frontend checks:

```bash
cd tauri
npm run format:check
npm run lint
npm run typecheck
npm test
npm run build
```

Rust checks:

```bash
cd tauri/src-tauri
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

Repository-level qualification helpers include:

```bash
./scripts/tauri/test-stage-mame-runtime.sh
python3 scripts/tauri/test-report-ci-evidence.py
python3 scripts/tauri/test-mt1800-qualification.py
python3 scripts/tauri/test-mt1900-upstream-inventory.py
python3 scripts/tauri/test-mt2004-release-candidate.py
python3 scripts/tauri/test-mt2100-final-quality-closure.py
python3 scripts/tauri/test-mt2200-todo-reconciliation.py
python3 scripts/tauri/test-post-closeout-hardening.py
python3 scripts/tauri/test-mame-ui-reproduction.py
python3 scripts/tauri/test-security-policy.py
python3 scripts/tauri/test-performance-qualification.py
```

The `Tauri project` workflow is the authoritative combined quality gate. On pull requests/master it also runs the hardened Linux production build and real development-window smoke under Xvfb.

## CI workflows

The Tauri/frontend CI matrix includes:

- `Tauri project` — Linux quality, performance, regression, production build, and development-window qualification;
- `Tauri security` — static security-policy and advisory checks;
- `Tauri Linux packaging` — `.deb`/AppImage smoke;
- `Tauri Windows packaging` — NSIS build/install/payload/uninstall smoke;
- `Tauri macOS packaging` — app/DMG layout, relocation/resource, ad-hoc signature, and signing-policy smoke;
- `Build documentation` — documentation build.

Use exact-head CI evidence for release or closure claims. A green ancestor or unrelated workflow is not evidence for a changed candidate.

## Packaging and release notes

Packaging CI uses synthetic staged MAME runtime payloads to validate installer topology and bundled-resource mechanics. It does not certify a public redistributable MAME binary bundle.

A public bundled-MAME release must separately qualify the real runtime binary/resources/licenses for each target platform. Public macOS signing/notarization likewise requires real Apple credentials.

## Upstream MAME

MAME is a multi-purpose emulation framework for documenting and preserving hardware/software history through executable emulation.

Useful upstream resources:

- [Official MAME Development Team Site](https://www.mamedev.org/)
- [MAME documentation](https://docs.mamedev.org/)
- [MAME Testers](https://mametesters.org/)
- [MAMEdev Forum](https://forum.mamedev.org/)
- [MAMEdev Discussions](https://github.com/orgs/mamedev/discussions)

To build upstream MAME itself, follow upstream documentation. Typical commands such as `make` or `make SUBTARGET=tiny` build MAME, not the Tauri frontend.

## License

This fork retains upstream MAME licensing. MAME as a whole is available under GNU GPL version 2 or later, with many source files under GPL-compatible licenses such as BSD-3-Clause, LGPL-2.1, or GPL-2.0.

The Tauri frontend Rust crate declares `GPL-2.0-only` package metadata.

See `COPYING` and `docs/legal/` for license texts and upstream licensing details.

MAME is a registered trademark of Gregory Ember, and permission is required to use the "MAME" name, logo, or wordmark.
