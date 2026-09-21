# MAME Tauri frontend fork

This repository is a fork of upstream [MAME](https://www.mamedev.org/) with a project-owned React/Tauri desktop frontend under `tauri/`.

The product scope is a **production-useful external-window MAME frontend**. MAME remains a separately supervised native process and owns emulation, rendering, audio, timing, and gameplay input. The Tauri application owns machine/software browsing, configuration, metadata, audit/status surfaces, artwork, save-state management, session control, and project-specific library features.

## Current product UI

The default ready-state UI is intentionally an original-MAME-style machine/software selector, not a new dashboard design. The parity target is to preserve the original selector's visual hierarchy and keyboard/mouse behavior while using the React/Tauri/WebView path to reduce dependence on the native desktop-toolkit behavior that caused Linux desktop-environment compatibility problems.

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
| selected-machine driver/status                                                   |
+--------------------------------------------------------------------------------+
```

Machine filters are persistent on the left, the dense machine list is the dominant center surface, and Images/Info is the right-side context. Search is live and bounded. Favorite, Audit, Export, Configure, Software, and Start are contextual to the selected machine. Software selection uses its own dense contextual browser and returns to the same machine state.

At narrow desktop widths, the machine list remains primary and an explicit **Details** control reveals the right-side context.

The old temporary **Legacy UI** route and vertical engineering-dashboard composition have been retired and are not the intended product direction. Settings, audit, history, collections, diagnostics, and session controls remain available as secondary surfaces rather than a default dashboard tab row.

### Intentional deviations from original MAME

Parity does not mean embedding the original native selector. The intentional deviations are bounded and documented:

- the selector is rendered by React in a Tauri WebView; MAME itself remains a separately supervised native process;
- project-only secondary surfaces such as Diagnostics, Collections, and bulk Audit remain available behind secondary actions;
- Category and Custom Filter positions are represented in the original-like filter list where full data/authoring support remains deferred;
- narrow-window behavior uses an explicit **Details** control instead of allowing the desktop three-panel layout to become unusable;
- project Icon/System Image artwork extensions coexist with the canonical MAME artwork categories;
- screenshot pixel-baseline CI is deferred because the hosted matrix has no stable cross-platform WebView screenshot harness; source/component tripwires plus the repo-owned visual checklist are the current regression guard.

The canonical visual contract is `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md`. Linux desktop compatibility boundaries and GNOME/KDE/Wayland/X11 considerations are recorded in `docs/MAME_TAURI_LINUX_DESKTOP_COMPATIBILITY_2026-09-21.md`.

## Current project status

The main engineering phase and post-closeout hardening phase are complete. The original-MAME visual/interaction parity pass is tracked by:

- `docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md` — current parity specification;
- `docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md` — canonical parity backlog and evidence ledger;
- `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md` — visual reference contract and manual checklist;
- `docs/MAME_TAURI_LINUX_DESKTOP_COMPATIBILITY_2026-09-21.md` — Linux desktop compatibility boundary and manual checks.

Earlier reproduction and engineering closure records remain useful historical evidence:

- `docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md`;
- `docs/MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md`;
- `docs/MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md`;
- `docs/MAME_TAURI_TODO_2026-09-08.md`;
- `docs/MAME_TAURI_MT2200_ENGINEERING_CLOSURE_2026-09-13.md`;
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

Deliberate MAME-parity gaps are documented in the parity TODO and historical closure record. They are not silently represented as implemented features.

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

## Visual parity verification

For a local visual/interaction check, start from a configured MAME executable and metadata catalog, then run:

```bash
cd tauri
npm ci
npm run tauri -- dev
```

Compare the default machine-selection window against `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md`. Verify the dark/navy main surface, compact blue toolbar, original-order left filters, dense central list, blue selected row with yellow selected text, muted unavailable rows, `Images` / `Infos` right panel, and green selected-machine status region. Exercise Up/Down, PageUp/PageDown, Home/End, Enter, Escape, search focus, filter selection, machine selection, right-panel tabs, Configure, Software List, Audit, and Start/Start Empty. On Linux, also follow `docs/MAME_TAURI_LINUX_DESKTOP_COMPATIBILITY_2026-09-21.md` for host-theme and focus checks.

### Updating visual references

When the intended upstream-MAME visual target changes, capture the original/reference selector and the Tauri comparison at the same practical window size and state. Record the date, platform/session, window dimensions, selected filter, selected machine, right-panel tab/category, and any host theme/compositor details. Store binary captures under `docs/reference/mame-ui-parity/` when using a binary-capable repository path, and update `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md` with filenames and any intentional differences. Do not replace the textual checklist: it remains the portable source of truth when screenshots cannot be consumed by CI or the Ralph text-file path.

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
