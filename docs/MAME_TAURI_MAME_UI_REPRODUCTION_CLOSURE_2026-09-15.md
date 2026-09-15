# MAME Tauri MAME-UI Reproduction Closure — 2026-09-15

**Milestone:** `MUI-001` through `MUI-020`  
**Specification:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md`  
**TODO ledger:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_TODO_2026-09-14.md`  
**Parity inventory:** `docs/MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md`  
**Accessibility qualification:** `docs/MAME_TAURI_MAME_UI_ACCESSIBILITY_QUALIFICATION_2026-09-15.md`

## Closure statement

The dashboard-style Tauri application composition has been replaced by a MAME-oriented machine/software selection experience. The selected machine is now the primary application context; filters, live search, a dense list, Images/Info, software selection, configuration, audit, favorites, export, launch, active-session controls, save states, and retained project extensions are organized around that context.

MAME remains a separately supervised native process. The WebView does not transport gameplay frames, PCM audio, timing, or gameplay input. Privileged filesystem, database, process, metadata, artwork, export, and runtime-control operations remain behind typed Rust/Tauri commands.

## Implemented product structure

The default ready-state composition is `App.tsx` → `MameShell` → `MameBrowser`.

The primary machine browser provides:

- persistent canonical left-side machine filters;
- bounded/debounced live search;
- a dense paged machine list with keyboard navigation;
- deterministic selection preservation and last-machine restoration;
- persistent Images/Info state and artwork category state;
- all 17 canonical MAME artwork categories plus the existing project-specific Icon/System Image extensions;
- safe clone-to-parent artwork fallback excluding BIOS parents;
- contextual Favorite, Audit, Export, Configure, Software, and Start actions;
- an explicit narrow-window Details control;
- fail-closed frontend shortcut ownership while native MAME owns gameplay input.

Secondary surfaces retain Session, bulk Audit, History, Collections, global Settings, and Diagnostics without returning to the former permanent-panel dashboard.

## Machine-filter parity

Implemented canonical fixed filters are:

- Unfiltered;
- Available / Unavailable;
- Working / Not Working;
- Mechanical / Not Mechanical;
- Favorites;
- BIOS / Not BIOS;
- Parents / Clones;
- Manufacturer;
- Year;
- Source File;
- Save Supported / Save Unsupported;
- CHD Required / No CHD Required using authoritative imported disk presence;
- Vertical Screen / Horizontal Screen using imported display rotation.

### Deliberate machine-filter defers

**Category** is deferred because the project does not ingest an authoritative MAME category/INI source. Inventing categories would be false parity.

**Custom Filter** is deferred because upstream MAME's persisted multi-clause custom-filter model would require a separate filter-expression persistence domain. The complete accepted fixed-filter inventory is implemented first.

## Software-selection parity

Implemented software behavior includes:

- contextual transition from the selected machine;
- machine context preserved on return;
- live text search;
- dense keyboard-navigable result list;
- software-list selection;
- Parents / Clones / Year / Publisher / Supported / Partially Supported / Unsupported filters;
- typed multipart software metadata and required part selection;
- authoritative Start Empty legality from mandatory image-device metadata;
- bounded authoritative BIOS discovery and Rust-side BIOS revalidation;
- typed BIOS launch option with no arbitrary argv field;
- persistent software Images/Info preference;
- non-error missing-software-artwork state when no authoritative software artwork source exists.

### Deliberate software defers

**Available / Unavailable software filters** are deferred because the project does not have trustworthy per-software audit/content provenance equivalent to its machine audit provenance. Unknown is preferred to guessed availability.

**Software Favorites** are deferred because the existing favorites domain is machine-keyed. Adding software favorites requires a durable `(machine, software-list, software)` identity/lifecycle contract and migration rather than overloading machine favorites.

**Developer / Distributor / Author / Programmer / Release Region / Device Type** filters are deferred because those dimensions are not currently imported as authoritative bounded fields by the project software metadata contract. They may be added when the parser/storage contract imports them explicitly.

**Software Custom Filter** is deferred for the same persistence/expression-model reason as machine Custom Filter.

**Software artwork ingestion** is deferred. The contextual Images mode remains present and reports a normal no-source state rather than fabricating artwork paths or exposing unrestricted filesystem URLs.

## Toolbar and information parity

Favorite, selected-machine audit, machine information, machine configuration, software selection, launch, and displayed-list export are implemented.

Displayed-list export is Rust-authoritative: the frontend supplies only the typed current search/filter state, the native dialog selects the destination, results are streamed page-by-page from the catalog with a hard row cap, CSV is escaped, and the file is atomically persisted. No generic filesystem capability was added.

**DAT/history pages are deferred** because no authoritative DAT/history source is imported by this project. Existing machine metadata is not relabelled as DAT parity.

## Remembered state

A versioned project-owned MAME UI state file persists:

- last machine;
- last filter and bounded filter value;
- machine Images/Info mode;
- machine artwork category;
- software Images/Info mode;
- software artwork preference.

The catalog query supports a preferred-machine hint so reopening can return the page containing the remembered machine without loading an unbounded catalog into the frontend.

## Contextual configuration/session/save-state behavior

Machine launch settings and controller-profile assignment are contextual to the selected machine. Global-vs-machine controller selection is explicitly distinguished. Configuration returns to the same machine context and restores focus to the originating action.

Active-session state appears in the shell status area. Pause, Resume, Soft Reset, Mute/Unmute, runtime refresh, Stop, and Save State remain available through the Session surface using the previously qualified authenticated runtime-control backend.

Application-managed save-state listing, save/load/delete, and structural compatibility checks remain in the Session context rather than the primary machine browser.

## Navigation and accessibility

The final focus graph is:

`filter region ↔ machine list ↔ right Images/Info context`, with contextual transitions to Software and Configure and explicit Back/Escape paths.

Up/Down/Home/End/Page Up/Page Down, `/` search focus, Enter activation, Escape search-clear/back semantics, editable-field protection, visible focus, textual status labels, reduced-motion handling, and fail-closed gameplay-input ownership are implemented and regression-guarded.

Global WebView gamepad navigation is deliberately deferred to avoid competing with native MAME gameplay input. See the accessibility qualification record for the ownership rationale.

## Visual reference disposition

The implemented reference states are structurally defined and regression-guarded for:

1. machine list with left filters and right Images mode;
2. machine list with Info mode;
3. contextual software browser;
4. contextual machine configuration;
5. active-session shell/status plus Session surface;
6. narrow-window machine list with explicit Details reveal.

A committed screenshot corpus is **deferred**. Current CI has no deterministic seeded machine catalog, artwork corpus, software list, and live-session fixture capable of producing stable representative screenshots across GTK/WebKit rendering versions. Committing blank or host-dependent screenshots would provide misleading regression value. The hardened real-window smoke instead validates the actual Tauri window/backend lifecycle, while the milestone regression validates the required structure. A future deterministic seeded-fixture harness may add screenshot regression without changing this milestone's behavior contract.

## Architecture/runtime regression coverage

`scripts/tauri/test-mame-ui-reproduction.py` protects the milestone architecture and interaction contracts and is invoked from the mandatory post-closeout regression inside `Tauri project`.

The hardened development smoke validates the MAME-UI architecture contract before launching the Tauri development app under Xvfb and then proves the real native window/backend remain alive through startup.

Frontend format, lint, TypeScript, Vitest, production build, Rustfmt, Rust tests, Clippy, full-catalog performance qualification, security checks, and platform packaging remain part of exact-head qualification.

## Product extensions retained

The migration retained existing capabilities while changing placement:

- Favorites → machine filter/contextual action;
- Collections → secondary surface;
- Recent History → secondary surface;
- Bulk Audit → secondary Audit surface with progress;
- General Settings → secondary Settings surface;
- Diagnostics → secondary troubleshooting surface;
- active session/save states → Session context.

The temporary Legacy UI route and legacy dashboard-only CSS have been removed.

## Security invariants preserved

- Rust crate remains `#![deny(unsafe_code)]`.
- No generic shell command bridge was added.
- No unrestricted filesystem, opener, or HTTP capability was added.
- MAME execution remains supervised and Rust-authoritative.
- Artwork and export filesystem access remain explicitly bounded.
- Runtime-control authentication and session scoping remain unchanged.
- Browser shortcuts continue to fail closed around gameplay ownership.
- Unknown audit/media provenance is not relabelled as known availability.

## Final qualification protocol

Closure requires an exact final PR head to pass:

- `Tauri project`;
- `Tauri security`;
- `Tauri Linux packaging`;
- `Tauri Windows packaging`;
- `Tauri macOS packaging`;
- documentation workflow when triggered;
- PR-event hardened Linux production/development-window qualification.

The final promoted `master` SHA and run IDs are recorded in a post-promotion evidence update before the milestone is claimed closed.
