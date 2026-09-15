# MAME Tauri UI Post-Closure Remediation Specification — 2026-09-14

**Repository:** `ekkus93/mame`  
**Baseline:** `master` at `b49a2b1652dbf929a71698038a917ff9abeb746f`  
**Related milestone:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_TODO_2026-09-14.md`  
**Related closure record:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md`

## 1. Purpose

The completed MAME-UI reproduction milestone correctly migrated the application architecture toward a MAME-style machine/software browser, but direct visual review against native MAME exposed three material product defects that were not adequately represented by the previous closure criteria:

1. the Tauri application does not reproduce the recognizable MAME visual language;
2. the primary application viewport can scroll as a document instead of behaving like a fixed desktop UI;
3. an unconfigured/unimported MAME catalog presents primarily as an empty machine browser, creating ambiguity between normal empty state, missing configuration, failed metadata import, and an actual catalog bug.

This remediation reopens those product-quality areas without discarding the previously qualified backend/security/runtime work.

The objective is not to recreate MAME pixel-for-pixel. The objective is to make the Tauri frontend visibly and behaviorally read as a MAME-derived desktop selector, keep its primary chrome fixed inside the window, and make MAME configuration/catalog bootstrap states explicit and actionable.

## 2. Reference observations

The review compared a native MAME selector screenshot with the Tauri frontend running on Ubuntu.

### Native MAME reference characteristics

The native reference visibly uses:

- a dark navy application background;
- saturated blue structural bars and selection treatment;
- bright yellow highlights/icons for important actions and current selection;
- white/light text with muted gray for disabled/unavailable entries;
- a strongly colored status region at the bottom;
- fixed top/search/toolbar/main/status regions;
- a left filter rail, central dense machine list, and right Images/Info region that remain spatially stable;
- scrolling inside variable list content rather than scrolling the whole application page.

### Tauri review findings

The reviewed Tauri build visibly showed:

- predominantly white/light-gray browser-like styling;
- weak visual correspondence with MAME despite similar three-region information architecture;
- document-level vertical scrolling, allowing top chrome to leave the viewport;
- an empty machine list with `0 machines`;
- the message `No successfully imported MAME metadata generation is active.`;
- footer state `MAME not configured`.

These observations are treated as acceptance evidence for this remediation.

## 3. Scope

This remediation covers three primary workstreams.

### 3.1 Visual parity remediation

Restyle the MAME browser shell so its visual hierarchy is recognizably derived from the native MAME selector.

This includes:

- shell background and region colors;
- top application/title area;
- navigation and toolbar treatment;
- machine-filter rail;
- central machine/software lists;
- selection, focus, hover, unavailable, preliminary/imperfect, favorite, and disabled states;
- Images/Info panel;
- bottom status bar;
- contextual actions and configuration surfaces insofar as they appear inside the MAME browser workflow.

The design must preserve accessibility and contrast. Fidelity to the native MAME visual language is more important than preserving the current generic light desktop-app theme.

### 3.2 Fixed-viewport layout remediation

The primary MAME browser must behave like a fixed desktop application, not a vertically scrolling web document.

The application shell must fit the usable Tauri window height. Only genuinely variable content regions may scroll internally.

### 3.3 Configuration/catalog bootstrap remediation

The machine browser must make configuration/import state unambiguous.

The application must distinguish at least these states:

1. MAME executable not configured;
2. MAME configured but no successful metadata generation exists;
3. metadata import in progress;
4. metadata import failed;
5. metadata import succeeded and an active catalog exists;
6. active catalog exists but the current search/filter returns zero machines.

The user must not have to infer those states from a small footer message.

## 4. Non-goals

This remediation does not require:

- embedding MAME rendering inside the Tauri WebView;
- changing the external supervised-process architecture;
- proxying gameplay audio/video/input through Tauri IPC;
- adding unrestricted filesystem, shell, opener, or HTTP capabilities;
- implementing previously deferred DAT/history ingestion;
- implementing previously deferred MAME Category/Custom Filter semantics unless independently required for one of the three remediation goals;
- matching every native-MAME pixel, font rasterization artifact, or platform-specific decoration.

## 5. Visual design contract

### 5.1 Palette direction

The final palette must be intentionally MAME-derived rather than neutral/light by default.

A suitable token family should include at minimum:

- deep navy application/background surface;
- darker navy content surface;
- saturated MAME blue structural/selection surface;
- brighter blue/cyan selected-row edge or gradient treatment;
- bright yellow accent for selected/high-value actions where appropriate;
- white/off-white primary text;
- muted gray disabled/unavailable text;
- green success/working/status treatment;
- red error/destructive treatment.

Exact hex values may be tuned during implementation. They should be centralized as CSS custom properties or equivalent theme tokens instead of being scattered as one-off literals.

### 5.2 Structural hierarchy

At normal desktop sizes the browser must visually read, from top to bottom, as:

1. title/application status strip;
2. search/toolbar strip;
3. fixed three-region browser body;
4. fixed bottom status strip.

Within the browser body:

- filter rail remains visually distinct at left;
- machine/software list is dominant at center;
- Images/Info remains a bounded context region at right;
- separators are strong enough to mirror the native MAME panel structure;
- selected state is immediately obvious without relying on subtle background tint.

### 5.3 Interaction states

Required styled states include:

- selected;
- keyboard focus;
- hover;
- disabled;
- unavailable/missing;
- preliminary/not-working/imperfect where present;
- favorite where represented;
- active tab/mode;
- success/info/warning/error status.

Status must remain understandable without color alone.

### 5.4 Theme policy

For this remediation, the MAME browser is not required to inherit the host operating system's light appearance. The MAME-derived dark theme is the canonical product presentation for this workspace unless a future explicitly designed alternate theme is added.

## 6. Fixed viewport contract

### 6.1 Root containment

For the primary Tauri window:

- `html`, `body`, the React root, application shell, and MAME shell must resolve to the available viewport height;
- the document/root must not become the normal vertical scroll container;
- the shell must use an explicit flex/grid sizing model with `min-height: 0`/`min-width: 0` where required so children can shrink correctly;
- fixed chrome must not be pushed below the fold by content.

### 6.2 Scroll ownership

Allowed scroll containers include variable collections such as:

- filter list if the window is too short for all filters;
- machine list;
- software list;
- long Info content;
- long secondary settings/history/diagnostic content when those surfaces intrinsically require scrolling.

The following must not scroll away during normal machine browsing:

- application/title strip;
- primary navigation;
- search/toolbar strip;
- main region boundaries;
- bottom status strip.

### 6.3 Desktop size qualification

At minimum, validate the primary machine browser at:

- 1920×1080;
- 1366×768;
- 1280×720.

For each size:

- no document-level vertical scrollbar may appear in the normal primary browser state;
- toolbar and status bar remain visible simultaneously;
- central content shrinks rather than forcing page overflow;
- internal list scrolling remains usable;
- the right-side context follows the existing explicit narrow-window strategy when width is insufficient.

## 7. MAME configuration and catalog bootstrap contract

### 7.1 State model

The frontend must derive a typed bootstrap/catalog state rather than flattening all absence cases into `0 machines`.

Required conceptual states:

- `mame-not-configured`;
- `metadata-not-imported`;
- `metadata-importing`;
- `metadata-import-failed`;
- `catalog-ready`;
- `catalog-ready-empty-result`.

Names may differ in code, but the distinctions must exist in the data/UI model.

### 7.2 MAME not configured

When no usable MAME executable is configured:

- the central workspace must prominently state that MAME is not configured;
- present a primary `Configure MAME` action;
- explain that machine metadata cannot be loaded until a MAME executable is configured;
- do not present `0 machines` as if it were a successfully queried catalog;
- filters/search may remain visible if desired for layout stability, but they must not imply a usable catalog.

### 7.3 MAME configured, metadata absent

When a MAME executable is configured but there is no successful active metadata generation:

- prominently state that machine metadata has not been imported successfully;
- present a primary `Import machine list` / `Import MAME metadata` action;
- expose the configured MAME identity/path in an appropriate detail or settings surface;
- make any prior import failure visible if known.

### 7.4 Import in progress

During import:

- show an explicit progress/busy state;
- prevent duplicate conflicting imports;
- keep the window responsive;
- on success transition automatically to `catalog-ready` and query the first page;
- on failure transition to `metadata-import-failed` with actionable error text.

### 7.5 Import failure

An import failure must include:

- clear failure status;
- the user-actionable reason where safe;
- `Retry import`;
- `Configure MAME` when executable/path configuration may be the cause;
- diagnostics linkage if deeper troubleshooting is needed.

### 7.6 Catalog ready

Once an active successful catalog exists:

- the browser should show the machine total returned by the catalog/query contract;
- the initial unfiltered machine query should populate the central list when machines exist;
- a zero-result search/filter must be visually distinct from missing catalog state;
- returning from configuration/import must refresh readiness and list state without requiring application restart.

## 8. Investigation requirement for the observed zero-machine state

The screenshot does not by itself prove a parser/query defect because it explicitly reports both `MAME not configured` and no active successful metadata generation.

Implementation must therefore determine which of these is true on the reviewed installation:

1. configuration was never completed;
2. configuration exists but was not loaded/persisted correctly;
3. MAME executable discovery/validation rejected a valid installation;
4. metadata import was never automatically or manually initiated;
5. metadata import failed;
6. metadata import succeeded but activation/persistence failed;
7. active catalog exists but the machine query is defective.

Do not fix this by hiding the message or hard-coding a machine count. The state transition and catalog provenance must remain truthful.

## 9. UX requirements for first run

A first-time user opening the app must have an obvious path to a populated library.

The expected workflow is:

1. app opens;
2. if MAME is not configured, primary workspace offers `Configure MAME`;
3. user selects/configures MAME executable and required paths;
4. app validates MAME identity;
5. app offers or automatically begins metadata import according to the final interaction design;
6. import status is visible;
7. successful import activates the catalog;
8. machine browser populates without requiring restart.

If automatic import is chosen, it must still expose progress/failure/retry states. If manual import is chosen, the call to action must be prominent enough that an empty machine browser cannot be mistaken for a working configured state.

## 10. Testing contract

### 10.1 Frontend/model tests

Add tests for:

- bootstrap-state derivation;
- no-MAME state;
- metadata-absent state;
- import-in-progress state;
- import-failed state;
- catalog-ready state;
- catalog-ready zero search/filter result;
- transition from successful import to populated query;
- fixed shell region semantics and scroll ownership where testable without a full browser.

### 10.2 Rust/backend tests

Preserve and extend tests for:

- MAME executable identity validation;
- metadata generation success/failure activation;
- catalog active-generation lookup;
- import error propagation;
- post-import query visibility where feasible at repository/test-fixture level.

### 10.3 Static architecture regression

Extend `scripts/tauri/test-mame-ui-reproduction.py` or add a dedicated remediation regression to fail if:

- the primary shell loses its fixed-viewport containment contract;
- root/document overflow is reintroduced intentionally in the MAME workspace;
- the MAME color-token/theme contract is removed;
- bootstrap states collapse back into a generic zero-machine message;
- primary Configure/Import actions disappear from the appropriate unavailable states.

### 10.4 Runtime smoke

The real Tauri development-window smoke must continue passing.

If practical within the existing harness, add viewport assertions using the running window or browser-side instrumentation. Do not claim this acceptance item from source inspection alone if a deterministic runtime check can be added safely.

## 11. Visual qualification contract

This remediation specifically exists because prior static/architectural qualification did not catch obvious visual mismatch.

Final qualification therefore requires a human-readable visual comparison against the supplied native MAME reference.

At minimum capture or inspect these Tauri states:

1. MAME not configured;
2. MAME configured but metadata absent;
3. populated unfiltered machine browser;
4. selected machine with Images mode;
5. selected machine with Info mode;
6. 1366×768 primary browser showing fixed top and bottom chrome simultaneously.

Acceptance questions:

- Does the app immediately read as MAME-derived rather than a generic light web app?
- Are navy/blue/yellow/green hierarchy and strong selected-state treatment present?
- Is there any document-level vertical scrolling in the primary browser?
- Are title/toolbar/status regions all visible at once?
- Is an unconfigured/import-missing state obvious and actionable?
- After successful import, are machines visibly populated?

Do not close this remediation solely because lint/tests/CI are green.

## 12. Security and architecture invariants

The remediation must preserve:

- supervised external MAME process architecture;
- typed Tauri commands;
- `#![deny(unsafe_code)]`;
- no generic shell bridge;
- no unrestricted filesystem/opener/HTTP surface;
- bounded metadata/catalog operations;
- existing runtime-control authentication/session scoping;
- gameplay input ownership remaining with native MAME while a session is active.

## 13. Documentation requirements

Update the relevant README/closure documents after implementation so they describe:

- the canonical MAME-derived dark visual theme;
- fixed-window scroll ownership;
- first-run MAME configuration/import workflow;
- explicit bootstrap/catalog state handling;
- any remaining deliberate visual or catalog limitations.

The old closure record should not be rewritten to imply that these defects were never present. Add a post-closure remediation record or amendment with exact evidence.

## 14. Acceptance definition

This remediation is complete only when all of the following are true:

- the primary MAME workspace has a recognizably MAME-derived dark navy/blue/yellow/green visual system;
- the primary browser does not use document-level vertical scrolling at the required desktop sizes;
- top chrome, main workspace, and bottom status remain simultaneously visible;
- lists/panels own their own overflow where necessary;
- MAME-not-configured is explicit and actionable;
- metadata-not-imported/importing/import-failed states are explicit and actionable;
- successful import activates and populates the machine browser without restart;
- zero search/filter results are distinguishable from missing catalog state;
- the observed installation's zero-machine condition is explained by verified configuration/import/catalog evidence;
- exact-head frontend/Rust/security/platform qualification remains green;
- the real-window smoke remains green;
- a final screenshot/visual review against the native MAME reference is performed and recorded;
- every remediation TODO item is reconciled before merge.

## 15. Qualification protocol

The final remediation candidate must pass the repository's applicable exact-head workflows, including at least:

- `Tauri project`;
- `Tauri security` when triggered/applicable;
- Linux packaging;
- Windows packaging;
- macOS packaging;
- documentation workflow when docs change;
- PR-event Linux production/development-window qualification.

The final closure evidence must record the exact candidate SHA, promoted `master` SHA, relevant run IDs, the verified explanation for the reviewed zero-machine state, and the result of the visual comparison.
