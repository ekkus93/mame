# MAME Tauri frontend fork

This repository is a fork of upstream [MAME](https://www.mamedev.org/) with a project-owned desktop frontend under `tauri/`.

The current product scope is a **production-useful external-window Tauri frontend**. The React/TypeScript UI provides library browsing, configuration, audit/status surfaces, artwork/save-state management, and session controls. The Rust/Tauri layer owns trusted filesystem, process, SQLite, metadata, and runtime-control authority. MAME itself remains a separately supervised native process and owns emulation, rendering, audio, and gameplay input.

## Current project status

The main engineering phase and post-closeout hardening batch are complete on `master`.

Primary closure records:

- `docs/MAME_TAURI_TODO_2026-09-08.md` — authoritative engineering TODO ledger.
- `docs/MAME_TAURI_MT2200_ENGINEERING_CLOSURE_2026-09-13.md` — engineering-phase closure and maintenance handoff.
- `docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_SPEC_2026-09-13.md` — post-closeout hardening specification.
- `docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_TODO_2026-09-13.md` — completed post-closeout hardening TODO.

The deliberately deferred optional research tracks are:

- MT-1000 — native-window / embedded-render integration research.
- MT-1100 — dedicated Tauri MAME OSD research.
- MT-1200 — in-process MAME hosting research.
- MT-1705 — embedded-render performance qualification, dependent on MT-1000.

Those tracks are not part of the completed external-window product claim.

## What the Tauri frontend does

Implemented frontend/backend capabilities include:

- MAME executable selection and identity reporting.
- MAME metadata import from `-listxml` into the local application database.
- Library search, filtering, favorites, machine details, and software-list browsing.
- ROM/software audit surfaces with stored provenance.
- User settings and launch-preference persistence.
- Supervised external MAME process launch and stop behavior.
- Authenticated runtime controls for supported live commands:
  - pause;
  - resume;
  - soft reset;
  - mute/unmute;
  - query/refresh runtime state.
- Local artwork discovery and safe local artwork serving.
- Save-state listing, save/load/delete, and compatibility warnings.
- Diagnostics and CI-backed release/qualification ledgers.

The WebView does **not** transport gameplay video frames, PCM audio, or live gameplay input. Those remain on native MAME/OS paths.

## Repository layout

Important project-owned paths:

```text
README.md                                      # this overview
tauri/                                        # React + Tauri frontend project
tauri/src/                                    # TypeScript/React frontend code
tauri/src-tauri/                              # Rust/Tauri backend code
scripts/tauri/                                # project validation and qualification helpers
docs/MAME_TAURI_*                             # project specs, closure records, handoffs, ledgers
.github/workflows/tauri-project.yml           # main Tauri quality/release-qualification workflow
.github/workflows/tauri-security.yml          # Tauri security checks
.github/workflows/tauri-*-packaging.yml        # platform packaging smoke workflows
```

Most upstream MAME source remains outside the Tauri project surface.

## Developer prerequisites

### All platforms

Install:

- Node.js `>=22.12.0`;
- npm matching the checked-in `package-lock.json` workflow;
- Rust stable with `cargo`, `rustfmt`, and `clippy`;
- a platform toolchain capable of building Tauri 2 applications;
- a MAME executable, either from an external install or a development tree, for real interactive use.

The frontend project pins its JavaScript dependencies in `tauri/package-lock.json`. The Rust backend pins dependency resolution in `tauri/src-tauri/Cargo.lock`.

### Linux prerequisites

The CI Linux jobs install these Tauri prerequisites:

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

`xvfb` and `xdotool` are used by the development-window smoke check. They are not normally required just to edit code, but they are required to reproduce the full Linux CI smoke path locally.

#### Known Tauri 2 Linux GLib advisory

At the time of this README update, the checked-in Rust dependency graph resolves the Tauri Linux GTK3 stack through `tauri 2.11.5`, `gtk 0.18.2`, and `webkit2gtk 2.0.2` to the Rust `glib 0.18.5` crate. RustSec reports `RUSTSEC-2024-0429` for that GLib crate as an informational `unsound` advisory affecting `glib::VariantStrIter` iterator methods. The application does not directly call the affected `VariantStrIter` API, and the project security audit currently reports no Rust vulnerabilities while still surfacing this informational warning.

This dependency is upstream in the supported Tauri 2 Linux stack. Do **not** try to fix it by adding or forcing `glib 0.20` alongside the current GTK3 dependency graph, and do not create separate GLib-specific builds of the application solely for this advisory. A proper removal requires the upstream Tauri Linux stack to move to compatible newer GTK/GLib bindings.

Until that migration is available in a supported Tauri release, keep the advisory visible and monitor Linux runtime behavior. If a Linux build shows reproducible GLib/GVariant crashes or behavior that could involve this advisory, capture the application logs and a minimal reproducer and reassess the dependency decision rather than silently suppressing the warning.

### macOS and Windows

Use the normal Tauri 2 platform prerequisites for macOS or Windows, plus Node.js and Rust stable. The CI packaging workflows validate macOS app/DMG layout and Windows NSIS packaging behavior, but release signing/notarization still requires real platform credentials outside this repository.

## Initial setup

From a fresh checkout:

```bash
cd tauri
npm ci
```

For real use, launch the app and configure the MAME executable/content paths through the UI. The app expects MAME to remain an external native process for the completed product scope.

## Development commands

Run the frontend development server:

```bash
cd tauri
npm run dev
```

Run the Tauri desktop app in development mode:

```bash
cd tauri
npm run tauri -- dev
```

Build the frontend only:

```bash
cd tauri
npm run build
```

Run a Tauri production build smoke check without bundling installers:

```bash
cd tauri
npm run tauri -- build --no-bundle
```

Build platform bundles/installers where the local platform supports them:

```bash
cd tauri
npm run tauri -- build
```

## Formatting, linting, typechecking, and tests

Frontend checks:

```bash
cd tauri
npm run format:check
npm run lint
npm run typecheck
npm test
npm run build
```

Rust backend checks:

```bash
cd tauri/src-tauri
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

Repository-level Tauri validation helpers:

```bash
./scripts/tauri/test-stage-mame-runtime.sh
python3 scripts/tauri/test-report-ci-evidence.py
python3 scripts/tauri/test-mt1800-qualification.py
python3 scripts/tauri/test-mt1900-upstream-inventory.py
python3 scripts/tauri/test-mt2004-release-candidate.py
python3 scripts/tauri/test-mt2100-final-quality-closure.py
python3 scripts/tauri/test-mt2200-todo-reconciliation.py
python3 scripts/tauri/test-post-closeout-hardening.py
python3 scripts/tauri/test-security-policy.py
python3 scripts/tauri/test-performance-qualification.py
```

The GitHub Actions `Tauri project` workflow is the authoritative combined project gate. It runs the Python qualification helpers, frontend format/lint/typecheck/tests/build, Rust format/tests/Clippy, performance qualification, release-candidate ledger generation, and Linux Tauri smoke checks.

## CI workflows

The current Tauri/frontend-specific CI matrix includes:

- `Tauri project` — main Linux quality and release-qualification gate.
- `Tauri security` — static security policy regression and advisory checks.
- `Tauri Linux packaging` — `.deb` and AppImage package smoke checks.
- `Tauri Windows packaging` — NSIS package, clean install, payload validation, and uninstall smoke checks.
- `Tauri macOS packaging` — app/DMG layout, ad-hoc signature, relocation/resource smoke, and release-signing policy checks.
- `Build documentation` — upstream/project documentation build.

For release or closure claims, use exact-head CI evidence. Do not substitute a green ancestor, descendant, or unrelated workflow run for the candidate SHA.

## Packaging and release notes

Packaging CI uses synthetic staged MAME runtime payloads to validate installer topology, bundled-resource paths, and install/uninstall behavior. That proves package mechanics; it does not certify a public redistributable MAME binary bundle.

A public release that bundles MAME must separately qualify the real runtime binary/resource/license set for the target platforms.

macOS CI uses smoke-appropriate signing behavior. Public macOS release signing and notarization require real Apple credentials and are outside the checked-in open repository state.

## Security model

The Tauri frontend intentionally keeps privileged operations in Rust. The WebView calls typed Tauri commands rather than invoking generic shell, filesystem, HTTP, or opener plugins.

Key constraints:

- Rust crate uses `#![deny(unsafe_code)]`.
- Tauri capabilities remain narrowly scoped.
- MAME runs as a supervised child process.
- Runtime-control frames are authenticated and bounded.
- Runtime-control bootstrap files are hardened on Unix and documented on non-Unix platforms.
- Gameplay video, PCM audio, and gameplay input are not proxied through Tauri IPC.

See `scripts/tauri/test-security-policy.py` and `scripts/tauri/test-post-closeout-hardening.py` for regression coverage of these assumptions.

## Upstream MAME information

MAME is a multi-purpose emulation framework whose purpose is to preserve decades of software history by documenting hardware and providing executable validation of that documentation.

Useful upstream resources:

- [Official MAME Development Team Site](https://www.mamedev.org/)
- [MAME documentation](https://docs.mamedev.org/)
- [MAME Testers](https://mametesters.org/)
- [MAMEdev Forum](https://forum.mamedev.org/)
- [MAMEdev Discussions](https://github.com/orgs/mamedev/discussions)

To build upstream MAME itself, follow the upstream documentation. Typical upstream builds use commands such as:

```bash
make
make SUBTARGET=tiny
```

Those commands build MAME, not the Tauri frontend app in `tauri/`.

## License

This fork retains upstream MAME licensing. The MAME project as a whole is made available under the GNU General Public License, version 2 or later, with many source files available under GPL-compatible licenses such as BSD-3-Clause, LGPL-2.1, or GPL-2.0.

The Tauri frontend Rust crate declares `GPL-2.0-only` package metadata.

Please see `COPYING` and `docs/legal/` for the full license texts and upstream licensing details.

MAME is a registered trademark of Gregory Ember, and permission is required to use the "MAME" name, logo, or wordmark.
