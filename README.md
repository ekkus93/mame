# MAME Tauri frontend fork

This repository is a fork of upstream [MAME](https://www.mamedev.org/) with a project-owned React/Tauri desktop frontend under `tauri/`.

The current product scope is a **Tauri-hosted in-app gameplay frontend backed by a separately supervised MAME process**. On the first supported gameplay path (Linux little-endian/AppImage), MAME runs headless with `-video none`; MAME's native snapshot renderer supplies RGB32 frames over a private authenticated session FIFO, Rust validates and exposes only the latest bounded frame, and a TypeScript-owned canvas renders the game inside the main Tauri window. MAME continues to own emulation, timing, and native audio output. Focus-owned keyboard/W3C-standard gamepad state is forwarded through a bounded typed Tauri command into the authenticated runtime-control shim rather than through the video queue.

The earlier external-native-window release remains useful historical engineering evidence, but it is no longer the intended gameplay product experience. The authoritative current work is `docs/MAME_TAURI_IN_APP_GAMEPLAY_SPEC_2026-10-08.md` and `docs/MAME_TAURI_IN_APP_GAMEPLAY_TODO_2026-10-08.md`.

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

- the selector and gameplay surface are rendered by React in a Tauri WebView; MAME remains a separately supervised native process rather than an in-process emulator library;
- project-only secondary surfaces such as Diagnostics, Collections, and bulk Audit remain available behind secondary actions;
- Category and Custom Filter positions are represented in the original-like filter list where full data/authoring support remains deferred;
- narrow-window behavior uses an explicit **Details** control instead of allowing the desktop three-panel layout to become unusable;
- project Icon/System Image artwork extensions coexist with the canonical MAME artwork categories;
- screenshot pixel-baseline CI is deferred because the hosted matrix has no stable cross-platform WebView screenshot harness; source/component tripwires plus the repo-owned visual checklist are the current regression guard.

The canonical visual contract is `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md`. Linux desktop compatibility boundaries and GNOME/KDE/Wayland/X11 considerations are recorded in `docs/MAME_TAURI_LINUX_DESKTOP_COMPATIBILITY_2026-09-21.md`.

## Current project status

The main engineering phase and post-closeout hardening phase are complete. The original-MAME visual/interaction parity pass remains historically tracked by:

- `docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md` — parity specification;
- `docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md` — original parity backlog and evidence ledger;
- `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md` — visual reference contract and manual checklist;
- `docs/MAME_TAURI_UI_PARITY_FINAL_REPORT_2026-09-21.md` — original closure report and historical evidence;
- `docs/MAME_TAURI_LINUX_DESKTOP_COMPATIBILITY_2026-09-21.md` — Linux desktop compatibility boundary and manual checks.

A 2026-09-21 post-closure review reopened a bounded remediation track for parity claims that required stronger behavioral, visual, and documentation evidence. The current reopened remediation track is:

- `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_SPEC_2026-09-21.md` — post-closure remediation specification;
- `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md` — canonical MTR remediation ledger and evidence record;
- `docs/MAME_TAURI_MTR_012_TEST_EVIDENCE_INVENTORY_2026-09-22.md` — static/source versus behavioral/component/platform evidence inventory;
- `docs/MAME_TAURI_MTR_013_RENDERED_VISUAL_REVIEW_2026-09-22.md` — textual rendered visual re-qualification with Tauri/WebView smoke evidence and screenshot-artifact limitations;
- `docs/MAME_TAURI_MTR_014_DOCUMENTATION_RECONCILIATION_2026-09-22.md` — documentation reconciliation, traceability, and final semantics record.

The original MTP TODO and final report are preserved as historical evidence. Do not treat their closure language as the current status while the reopened MTR remediation ledger remains open; current closure status is governed by `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md` until MTR-015 is promoted.

Earlier reproduction and engineering closure records remain useful historical evidence:

- `docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md`;
- `docs/MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md`;
- `docs/MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md`;
- `docs/MAME_TAURI_TODO_2026-09-08.md`;
- `docs/MAME_TAURI_MT2200_ENGINEERING_CLOSURE_2026-09-13.md`;
- `docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_TODO_2026-09-13.md`.

The September MT-1000/MT-1705 embedded-render work was deferred for the historical external-window release. That product assumption was superseded on 2026-10-08 by the in-app gameplay specification. The current implementation deliberately uses a narrow sidecar integration—MAME's existing snapshot renderer plus a private bounded frame transport—rather than a dedicated Tauri MAME OSD or in-process MAME hosting. Dedicated OSD and in-process hosting remain separate research tracks.

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

Deliberate MAME-parity gaps and remediation status are documented in the parity TODO, the reopened remediation ledger, and the historical closure record. They are not silently represented as implemented features.

## In-app gameplay architecture and current scope

The in-app gameplay path keeps MAME out of the WebView and out of the Tauri process while replacing the visible external SDL gameplay window:

1. Rust launches one supervised MAME child with the normal effective ROM/content paths and native audio configuration, but forces `-video none`.
2. The session bootstrap configures a private mode-`0600` FIFO and unpredictable per-session authentication token on supported Linux little-endian hosts.
3. MAME's Lua callback calls `manager.machine.video:snapshot_pixels()`. That API renders MAME's snapshot target through the native C++ software renderer and returns RGB32 pixels, avoiding the palette-index ambiguity of `screen:pixels()`.
4. Rust validates protocol version, session/token identity, dimensions, stride, payload caps, sequence monotonicity, and supported format before replacing a one-slot latest-frame mailbox.
5. The WebView fetches only the newest frame through a typed binary command and presents it in a TypeScript-owned canvas. Frame receipt/drop/delivery/presentation metrics and stall state are separately observable.
6. Keyboard and W3C-standard Gamepad API controls are translated to bounded MAME logical input tokens and applied through MAME's normal I/O-port fields. Saved browser-standard controller profiles pin gameplay to the recorded Gamepad API identity when connected.
7. MAME audio remains native. The currently bundled MAME Lua surface exposes UI mute but not a live master-volume setter, so the product exposes mute/unmute without claiming WebView PCM or unsupported volume control.

The first supported in-app video transport is explicitly **Linux little-endian**. Unsupported platforms fail with `MAME_IN_APP_GAMEPLAY_UNSUPPORTED` rather than silently reopening an external native gameplay window. The initial screen-topology scope is MAME's composed snapshot target: changing target dimensions are accepted frame-by-frame, the canvas handles 0/90/180/270-degree orientation, and multi-screen layouts are presented as the single layout MAME composes into that target rather than as independently controllable per-screen WebView surfaces. Current known gaps include final full-AppImage desktop qualification with real ROMs, performance evidence across representative raster/vector/high-resolution machines, independently addressable multi-screen surfaces, and parity with renderer-specific BGFX/artwork effects. See the in-app gameplay spec/TODO for closure evidence.

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
  src/gameplay/                       # in-app canvas, frame protocol, input mapping
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
- runtime-control messages remain authenticated, bounded, and session-scoped;
- gameplay video uses a separate private authenticated binary FIFO from MAME to Rust; frame bytes never share stdout/runtime-control diagnostics and the Rust mailbox keeps only the latest frame;
- the WebView receives gameplay video only through a typed bounded binary command; it has no generic socket or filesystem capability;
- gameplay input is focus-owned in the WebView and forwarded as bounded logical MAME input tokens through a typed command; high-rate updates are coalesced and never share the video queue;
- PCM audio is not proxied through the WebView: MAME retains native audio output, with native UI mute exposed through the authenticated control protocol;
- MAME remains authoritative for emulation timing; the WebView does not drive the emulation clock;
- focus loss, fullscreen transitions, stop, and component teardown release desired/accepted gameplay input to prevent stuck controls.

## Developer prerequisites

### All platforms

Install:

- Node.js `>=22.12.0`;
- npm matching the checked-in `tauri/package-lock.json` workflow;
- Rust stable with `cargo`, `rustfmt`, and `clippy`;
- the normal Tauri 2 platform toolchain;
- a locally built or installed MAME executable only when testing development-tree/external-override flows. Normal Debian/Ubuntu production packages include a bundled package-owned MAME runtime and do not require users to supply an executable.

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

## Packaged runtime and content setup

Normal packaged installs include the package-owned MAME runtime. A user does **not** need to install upstream MAME separately or select a MAME executable before browsing and configuring content. The external executable selector is an advanced/development override: open the Settings surface, expand **Advanced override** under **Bundled MAME runtime**, and clear the override to return to the package-owned runtime.

### Configure ROM, software, and CHD paths

Open the app's Settings surface and use **Content configuration → ROM and software paths**. The panel has separate **ROM paths**, **Software paths**, and **CHD paths** groups with **Add … directory** actions. Paths are searched in the order shown, can be moved up/down, and remain visible with an explicit validation status when missing, unreadable, or inaccessible.

When no ROM path is configured, the app also detects the standard per-user `~/mame/roms` directory when it exists. Explicitly configured paths take precedence. Audit, catalog Start, and diagnostics use the same effective paths, so the local availability filter reflects the directories MAME searches.

### Local content availability

Catalog metadata and local content availability are separate:

- **Available Locally** means the current audit for the current MAME runtime and current effective content paths classified the machine as complete or best-available. Normal Start is enabled.
- **Missing Content** means the current audit found required content missing, incorrect, or a mixture of both. Normal Start is gated; fix the content/path configuration and audit again.
- **Not Audited** means there is no current audit result for the active runtime/path identity. This includes results made stale by changing the runtime or effective paths. Normal Start is gated until an audit establishes availability.

The machine catalog comes from MAME metadata such as `-listxml`, so catalog entries can exist before any local ROMs are configured. Seeing a machine in the browser does not imply that its ROM/CHD/software content is installed.

Artwork directories are optional presentation media. Missing snapshots, cabinets, marquees, icons, and other artwork never make a machine unlaunchable and are not part of the ROM/content availability gate.

### Native list/count semantics

The **Unfiltered** machine view follows native MAME's unfiltered system-list semantics: it does not silently exclude device rows. Parent/clone, BIOS, driver-status, mechanical, CHD, orientation, and other dimensions are explicit filters rather than hidden base-query restrictions. The RESET-007 regression fixture and native-source comparison are recorded in `docs/MAME_TAURI_NATIVE_UI_PARITY_INVENTORY_2026-09-30.md`.

## Visual parity verification

For a local visual/interaction check, start from a bundled/default runtime in an installed package or an explicit development/external MAME override plus a metadata catalog, then run:

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
- `Tauri Linux packaging` — synthetic `.deb`/AppImage smoke for fast installer-topology regression;
- `Tauri Linux real runtime package` — release-grade Linux `.deb` and AppImage qualification with a real bundled MAME runtime, package install/uninstall, installed bundled-source verification, read-only package resource checks, metadata bootstrap/catalog qualification, and uploaded package artifacts;
- `Tauri Windows packaging` — NSIS build/install/payload/uninstall smoke;
- `Tauri macOS packaging` — app/DMG layout, relocation/resource, ad-hoc signature, and signing-policy smoke;
- `Build documentation` — documentation build.

Use exact-head CI evidence for release or closure claims. A green ancestor or unrelated workflow is not evidence for a changed candidate.

## Packaging and release notes

Fast packaging CI may use synthetic staged MAME runtime payloads to validate installer topology and bundled-resource mechanics only. Synthetic payloads do not certify a production bundled-MAME release.

The release-grade Linux path is the `Tauri Linux real runtime package` workflow. It builds a real MAME runtime from the repository source, stages the runtime with `hash`, `bgfx`, COPYING, and legal resources, augments Debian shared-library dependencies, builds both `.deb` and AppImage bundles, installs and qualifies the `.deb`, extracts and qualifies the AppImage runtime tree, verifies installed backend bundled-source resolution, verifies installed metadata bootstrap/catalog readiness, verifies read-only package-resource behavior, and uploads the `real-bundled-mame-linux-packages` artifact.

Public macOS signing/notarization requires real Apple credentials outside this repository.

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
