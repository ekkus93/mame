# MAME Tauri MAME-UI Reproduction Specification — 2026-09-14

## 1. Purpose

This specification defines the next major frontend milestone for the project: replace the current dashboard-style Tauri presentation with a Tauri/React reproduction of MAME's existing system/software selection UI and interaction model.

The goal is not to port MAME's C++ rendering implementation line-for-line. The goal is to treat the MAME UI already present in this repository as the canonical product reference and reproduce its information architecture, terminology, selection behavior, navigation model, contextual actions, and overall visual organization in React/Tauri.

The existing Rust/Tauri backend remains the trusted application backend. MAME remains a separately supervised native process. This milestone is therefore primarily a frontend architecture and UX reconstruction, with additive backend work only where an upstream-MAME UI behavior has no existing typed Tauri command or data contract.

Baseline for this specification:

- repository: `ekkus93/mame`;
- baseline `master`: `7b7486a9a631074efd20096c0580d391ef649360`;
- upstream MAME UI source is present in the same repository and is the authoritative behavior reference for this milestone.

## 2. Product decision

The current Tauri frontend must no longer present the application as a vertical engineering dashboard where global settings, diagnostics, session controls, audits, history, collections, save states, and library browsing compete for equal visual priority.

The primary application experience must instead be a MAME-style browser:

```text
+--------------------------------------------------------------------------------+
| MAME                                                        Search [_________] |
+--------------------+--------------------------------------+--------------------+
| FILTERS            | MACHINES / SOFTWARE                  | IMAGE / INFO       |
|                    |                                      |                    |
| All                | 1942                                 | [ artwork ]        |
| Available          | 1943                                 |                    |
| Favorites          | Asteroids                            | Pac-Man            |
| ...                | Centipede                            | Namco, 1980        |
|                    | Donkey Kong                          | Working            |
|                    | Galaga                               |                    |
|                    | Pac-Man                        <      | [ Start ]          |
|                    | ...                                  |                    |
+--------------------+--------------------------------------+--------------------+
| status / count / audit / active session                                         |
+--------------------------------------------------------------------------------+
```

The selected machine or software item is the central context. Configuration, audit, artwork, software, controller settings, save states, and launch/session actions must be attached to that context rather than displayed as unrelated top-level panels.

## 3. Canonical reference implementation

The implementation must be grounded in the upstream MAME UI source in this repository. At minimum, the following areas are reference material:

- `src/frontend/mame/ui/selgame.cpp` — main machine/system selection behavior, remembered selection/filter/right-panel state, machine launch flow, and system-list semantics;
- `src/frontend/mame/ui/selmenu.cpp` — shared system/software selection layout, left filter panel, right Images/Info panel, artwork categories, toolbar actions, selection behavior, and launch-menu interaction;
- `src/frontend/mame/ui/selsoft.cpp` — software-list browsing and software selection behavior;
- related MAME UI files for audit, information/DAT views, options, favorites, selectors, and configuration as discovered during MUI-001 reference inventory.

The current Tauri frontend is implementation material, not the UX reference. Important existing Tauri surfaces include:

- `tauri/src/App.tsx`;
- `tauri/src/library/LibraryBrowser.tsx`;
- `tauri/src/library/SoftwareListBrowser.tsx`;
- `tauri/src/library/MachineAuditPanel.tsx`;
- `tauri/src/settings/MachineSettingsPanel.tsx`;
- `tauri/src/session/SessionControlPanel.tsx`;
- `tauri/src/session/SaveStateBrowser.tsx`;
- `tauri/src/settings/GeneralSettingsPanel.tsx`;
- `tauri/src/settings/DiagnosticsPanel.tsx`;
- existing typed command wrappers and Rust backend modules under `tauri/src-tauri/src/`.

## 4. Fidelity target

This milestone targets **MAME-compatible reproduction**, not merely a MAME-inspired theme.

Required fidelity dimensions:

1. **Information architecture** — machine/software selection is the primary workspace; filters are left-context; Images/Info are right-context; status is subordinate.
2. **Terminology** — labels should follow current MAME terminology unless a deliberate project-specific extension is documented.
3. **Selection semantics** — selected machine/software drives detail, artwork, actions, and launch state.
4. **Navigation semantics** — keyboard-first behavior must remain predictable and MAME-like; pointer input is additive, not required.
5. **Filter/search behavior** — the user can quickly narrow a large catalog without leaving the selection screen.
6. **Contextual actions** — favorite, audit, information, configuration, software, save-state, and launch actions appear in the selected-item context.
7. **State persistence** — last selection, filter, search/right-panel choices where appropriate, and relevant view preferences should survive normal navigation/restart in a manner consistent with MAME's model.
8. **Visual organization** — the three-region selection layout and subordinate status/footer should visibly resemble MAME's current UI even though browser/WebView typography and widgets will not be pixel-identical.

Pixel-perfect emulation of MAME's custom renderer is not required. Structural and behavioral fidelity takes priority over reproducing renderer-specific antialiasing, font metrics, or low-level drawing primitives.

## 5. Non-goals

This milestone does not:

- embed MAME's native renderer into the Tauri WebView;
- port MAME's C++ menu renderer line-for-line;
- implement MT-1000 native-window/embedded-render research;
- implement MT-1100 dedicated Tauri MAME OSD research;
- implement MT-1200 in-process MAME hosting;
- transport gameplay video frames, PCM audio, or live gameplay input through Tauri IPC;
- replace MAME's in-emulation UI/OSD while a game is running;
- require GTK4/Tauri 3 migration;
- require public redistributable bundled MAME binaries;
- remove project-specific enhancements such as collections, history, diagnostics, or save-state management when they can be integrated contextually.

## 6. Core shell architecture

The main React composition should be reorganized around a dedicated MAME-style shell rather than a vertical list of panels.

Target component structure:

```text
tauri/src/
  shell/
    MameShell.tsx
    MameToolbar.tsx
    MameStatusBar.tsx
    WorkspaceRouter.tsx

  browser/
    MachineBrowser.tsx
    MachineFilterPanel.tsx
    MachineSearch.tsx
    MachineList.tsx
    MachineListRow.tsx
    SelectionDetailsPane.tsx
    ArtworkView.tsx
    MachineInfoView.tsx

  software/
    SoftwareBrowser.tsx
    SoftwareList.tsx
    SoftwareDetails.tsx

  dialogs/
    MachineConfigDialog.tsx
    ControllerConfigDialog.tsx
    AuditDialog.tsx
    SaveStateDialog.tsx
    GlobalSettingsDialog.tsx
    DiagnosticsDialog.tsx

  session/
    ...existing session logic, adapted to contextual presentation...
```

Exact filenames may change during implementation, but the architecture must preserve the same separation of responsibilities.

`App.tsx` should become thin application composition/state routing. It should not regain responsibility for rendering all major product features inline.

## 7. Main selection workspace

The default application screen after backend readiness must be the MAME-style machine browser.

Required regions:

- **left panel:** filter/category selection;
- **center panel:** scrollable/selectable machine list;
- **right panel:** Images or Info for the current selection;
- **top/toolbar area:** search and contextual toolbar actions;
- **bottom/status area:** counts, availability/audit summary, active-session state, or transient operation status.

The machine list receives the largest share of workspace. Global settings and diagnostics must not occupy permanent main-workspace vertical space.

The layout should tolerate resizing. On windows too narrow for three simultaneous regions, the right panel may collapse behind an explicit Images/Info/details control, but the machine list must remain the primary view.

## 8. Machine filters

The left filter panel must be derived from MAME's current filter semantics rather than the current Tauri form layout.

MUI-001 must inventory the exact current MAME filter types and identify which can be implemented using existing catalog fields and which require backend/query extensions.

At minimum, the reproduction must support equivalent concepts for commonly used MAME filters such as the complete list, availability, favorites, and other machine classification filters represented by current MAME source. The implementation must not invent a smaller arbitrary filter taxonomy merely because it is easier to wire.

Filter behavior requirements:

- selecting a filter immediately updates the center list;
- selection remains stable when the selected item is still present;
- if the selection disappears, selection moves predictably to the first/nearest item;
- active filter is visually obvious;
- filter navigation works by keyboard;
- last-used filter is persisted where appropriate;
- filter counts may be displayed when inexpensive, but counts must not make interaction sluggish.

## 9. Search behavior

Search is a primary browser interaction, not a separate form submission workflow.

Requirements:

- search field is visible from the main selection workspace;
- typing updates results with bounded/debounced query behavior suitable for a full MAME catalog;
- `/` focuses search when gameplay input is not owned by MAME;
- Escape clears or leaves search according to an explicitly documented rule;
- selection and keyboard navigation remain deterministic while search results change;
- search semantics should reflect the fields MAME searches where those fields are available in the imported metadata;
- the UI must clearly distinguish an empty result from catalog/backend failure.

## 10. Machine list behavior

The center list must be optimized for rapid scanning and keyboard navigation.

Requirements:

- each row exposes the machine description/title prominently and the short name where useful;
- machine status/availability is represented compactly without turning each row into a card;
- favorites can be recognized without opening a separate favorite shelf;
- clone/parent or other MAME-identifying metadata may be represented when useful and available;
- Up/Down/Home/End navigation remains supported;
- Page Up/Page Down behavior should be considered for MAME-like long-list navigation;
- Enter/Select invokes the primary contextual selection/launch behavior;
- scrolling must preserve selection visibility;
- list rendering must remain within the existing interactive performance budget on a full catalog.

The current card/panel-heavy layout must not be reproduced inside every list row.

## 11. Selection persistence and view state

The application should maintain user context similarly to MAME's selection UI.

Required persisted or session-restored state should include, where appropriate:

- last selected machine;
- last filter;
- search text only if deliberately chosen by UX review (default recommendation: do not persist transient search across restart);
- current right-panel mode (`Images` or `Info`);
- current artwork category/image type;
- selected software item when returning from contextual software browsing if feasible;
- user-selected panel visibility/collapse state if introduced.

Persistence must use project-owned typed settings/state; do not use ad-hoc browser local storage for privileged or authoritative state.

## 12. Right Images/Info panel

The current MAME UI's right-side panel is a central reproduction requirement.

### Images mode

The Images mode must support the artwork categories available through MAME/project artwork configuration where data exists. Current upstream MAME categories include concepts such as snapshots, cabinets, control panels, PCBs, flyers, title screens, artwork previews, marquees, covers, and other configured image types.

The Tauri implementation may initially expose only categories supported by the project's artwork backend, but MUI-001 must record the parity gap and MUI-006 must either close it or explicitly defer categories with rationale.

Image behavior must include:

- selected-machine image update without navigating away;
- missing-artwork state that does not look like an error;
- parent/clone fallback where compatible with the backend policy;
- explicit image-category switching;
- no unrestricted filesystem URLs exposed to the WebView.

### Info mode

The Info panel should present machine metadata in a compact MAME-like format rather than as a large independent dashboard section.

Candidate information includes:

- description/title;
- short name;
- year;
- manufacturer;
- source/driver identity where available;
- parent/clone relationship;
- emulation/status summary;
- ROM/software availability/audit summary;
- machine capabilities that are useful for selection.

The panel should be extensible to DAT/history-style information without forcing every information source into the initial milestone.

## 13. Toolbar and contextual actions

MAME's selection UI exposes contextual actions such as favorite, export, audit, and information/DAT access. The Tauri reproduction should preserve the action model while using only supported backend capabilities.

Required first-class actions:

- toggle favorite;
- audit selected machine/media;
- open machine information/details;
- configure selected machine;
- browse/select software when applicable;
- launch/start;
- open selected-machine save states when a compatible running session exists or where browsing stored states makes sense.

Actions that exist in MAME but lack a Tauri backend capability, such as export behavior if not already implemented, must be recorded as a parity gap and either implemented additively or explicitly deferred. They must not silently disappear from the fidelity analysis.

Toolbar buttons require accessible names/tooltips and keyboard equivalents where MAME has them.

## 14. Software selection

Software-list browsing must be integrated as a MAME-like contextual transition from the selected machine, not as a large nested panel permanently rendered under the machine browser.

Requirements:

- machines with software lists expose an obvious software-selection action;
- software browser preserves the selected machine context;
- software search/filter behavior is modeled after the current MAME software selector where feasible;
- selecting software updates the right Images/Info context;
- launch uses the existing typed Rust launch path and software identity;
- returning from software selection restores the prior machine selection and view state;
- start-empty cases and multi-part/software-part cases discovered in the MAME reference inventory must receive an explicit parity decision.

## 15. Configuration UX

Global and machine-specific configuration must stop occupying permanent main-workspace panels.

### Machine configuration

Machine-specific settings should open from the selected-machine context, using a dialog, drawer, or dedicated contextual subview. Existing `MachineSettingsPanel` behavior should be preserved behind the new presentation.

### Controller configuration

Controller/profile configuration should be contextual and should clearly distinguish global profile selection from per-machine overrides where supported.

### Global settings

MAME executable selection, content paths, artwork paths, and global launch preferences should move to a global Settings surface reachable from the shell toolbar/menu.

### Diagnostics

Diagnostics should move to a dedicated Diagnostics surface. It must remain easily reachable for troubleshooting but should not consume primary browser real estate.

## 16. Launch and session behavior

Starting a machine/software item remains the dominant primary action.

Requirements:

- selected runnable item exposes a clear Start/Launch action;
- launch failure is reported in context without destroying browser state;
- once MAME owns gameplay input, application shortcuts remain fail-closed according to the existing ownership model;
- active-session status is represented compactly in the shell/status area;
- pause, resume, reset, mute/unmute, runtime refresh, stop, and save/load-state capabilities remain available through a contextual session surface rather than a permanent dashboard panel;
- terminal session events restore frontend input ownership immediately;
- no gameplay hot path is moved through Tauri.

## 17. Favorites, collections, history, and project extensions

Project-specific features that are useful but not primary upstream MAME selection concepts must be retained without distorting the core reproduction.

- **Favorites:** integrate directly into MAME-style filters and row/action state; remove dependence on a separate permanent Favorite Shelf as the primary favorites UX.
- **Collections:** expose through a secondary filter/category or contextual library-management surface, not a permanent bottom panel.
- **History:** expose as a secondary browser/filter/view or menu surface.
- **Bulk audit:** expose as a toolbar/menu action with progress/status, not a permanent main panel.
- **Diagnostics:** dedicated troubleshooting surface.

A feature may be removed from the always-visible shell while remaining fully supported.

## 18. Keyboard, controller, and pointer navigation

The application must remain usable without a mouse for core browsing and launching.

Required keyboard behaviors include:

- Up/Down selection movement;
- Home/End boundaries;
- Page Up/Page Down where practical;
- `/` search focus;
- Enter/Select primary action;
- Escape/back navigation out of contextual views/dialogs;
- predictable Left/Right movement between filter/list/right-panel contexts where this improves MAME parity;
- favorite/audit/configuration shortcuts where MAME equivalents or existing project shortcuts justify them.

The implementation must preserve editable-field protection and the fail-closed gameplay-input ownership policy.

Controller/gamepad navigation should be evaluated as part of MUI-012. If the Tauri layer cannot safely support it without interfering with native MAME gameplay input, the boundary must be explicit and tested.

## 19. Visual design

The visual objective is recognizable MAME selection UI organization with modern WebView implementation quality.

Requirements:

- dark theme by default unless project settings later expose theme choice;
- restrained use of cards, borders, shadows, and large headings;
- dense list-oriented information presentation suitable for thousands of machines;
- clear selected-row state and keyboard focus state;
- left/filter and right/details panels visually subordinate to the center list;
- typography and spacing optimized for desktop use rather than a web dashboard;
- status colors must not be the sole carrier of meaning;
- layout should remain usable at common desktop sizes and under scaling.

The implementation should avoid gratuitous visual redesign until behavioral parity is established.

## 20. Accessibility

MAME fidelity does not override accessibility.

Requirements:

- correct list/listbox or grid semantics where appropriate;
- visible focus indication;
- accessible names for icon-only actions;
- keyboard operation for all core browser actions;
- status and operation messages surfaced through appropriate live regions without excessive announcements;
- no color-only status encoding;
- dialogs correctly trap/restore focus;
- reduced-motion preferences respected if transitions are introduced.

## 21. Data and backend contract policy

The existing Rust backend should be reused rather than rewritten.

Before adding backend APIs, implementation must first determine whether existing typed commands can supply the required MAME UI behavior. Additive backend work is permitted for genuine parity gaps, for example:

- missing machine filter dimensions;
- missing parent/clone or status metadata needed by the list/right panel;
- artwork categories not yet represented by the artwork backend;
- richer software-list query/filter metadata;
- persisted UI view state;
- export/DAT information if accepted into scope.

Any new backend command must remain typed, bounded, and Rust-authoritative. No generic filesystem/shell bridge may be added for UI convenience.

## 22. Frontend state architecture

The new shell must avoid duplicating authoritative backend state across unrelated components.

Recommended state domains:

- browser query/filter/search state;
- current machine/software selection;
- right-panel view state;
- contextual dialog/subview state;
- current supervised MAME session;
- transient operation/notice state.

State shared broadly across the shell should use an explicit reducer/context/store pattern already compatible with project conventions rather than event chains between distant panels.

Tauri events must use the centralized valid event-name contract introduced after PR #17.

## 23. Migration strategy

The rewrite should be incremental and continuously runnable.

Recommended sequence:

1. establish reference inventory and component shell;
2. build the three-region browser using existing library queries;
3. move filters/search/list/detail into the new shell;
4. integrate Images/Info;
5. migrate favorites/audit/software/configuration actions contextually;
6. migrate session/save-state behavior;
7. move global settings/diagnostics/history/collections/bulk audit into secondary surfaces;
8. remove the legacy dashboard composition only after feature-parity checks pass.

Do not delete an existing feature before its replacement path is implemented and covered.

## 24. Testing and regression requirements

This milestone must add regression coverage specifically for the new product structure.

Required layers:

### Unit/component tests

- filter/search request construction;
- selection stability across result updates;
- keyboard navigation;
- right-panel mode/image-category state;
- contextual action availability;
- software-browser transitions;
- dialog focus/back behavior where testable;
- active-session shortcut ownership.

### Static architecture regression

Add a repository-level regression script for the milestone that verifies, at minimum:

- the MAME shell/browser components exist;
- `App.tsx` no longer renders the legacy dashboard stack directly;
- global Settings and Diagnostics are secondary surfaces;
- centralized event constants remain valid and in parity;
- required spec/TODO linkage exists;
- legacy panels are not silently orphaned before migration is complete.

### Runtime smoke

The Linux Tauri dev smoke must continue to prove the process stays alive after the window appears. Extend runtime smoke coverage where practical to verify that the primary machine browser renders rather than merely detecting a window title.

### Visual regression

Establish deterministic reference screenshots or equivalent rendered-layout assertions for representative states if a stable CI-compatible mechanism can be introduced without excessive maintenance. At minimum, manually captured reference states must be documented during MUI-013 before claiming visual-fidelity closure.

## 25. Performance requirements

The new UI must preserve or improve current full-catalog responsiveness.

Requirements:

- no unbounded rendering of the full machine catalog in the DOM;
- filtering/searching remains interactive at full-catalog scale;
- selection-driven detail/artwork requests are cancellable or stale-result-safe;
- artwork loading does not block list navigation;
- keyboard movement should feel immediate;
- existing library performance qualification continues to pass;
- any virtualization introduced must preserve focus and accessibility semantics.

## 26. Security requirements

All existing security boundaries remain in force.

- WebView remains untrusted relative to privileged filesystem/process operations.
- MAME launch/control remains Rust-authoritative.
- Artwork serving remains bounded to approved local assets.
- No generic shell, opener, unrestricted filesystem, or HTTP plugin is added.
- Runtime-control tokens and bootstrap protections remain unchanged.
- Save-state paths remain application-owned and validated.
- Event names remain Tauri-valid and centralized.

## 27. Documentation requirements

On completion:

- update `README.md` screenshots/description so the product is described as a MAME-style Tauri frontend rather than the current dashboard composition;
- update developer documentation for the new component layout;
- document any deliberate upstream-MAME parity gaps;
- preserve the existing GLib advisory note unless the underlying dependency is actually removed;
- record final exact-head CI evidence and promoted `master` SHA.

## 28. Acceptance definition

This milestone is complete only when all of the following are true:

1. Application launch lands in a MAME-style machine selection workspace.
2. Machine list is the dominant primary surface.
3. MAME-derived filter and search semantics are available through the left/top browser context.
4. Selection drives a right-side Images/Info context.
5. Favorite, audit, configuration, software, and launch actions are contextual to the selection.
6. Global settings and diagnostics are no longer permanent main-screen panels.
7. Existing project features remain reachable or are explicitly deferred with rationale.
8. Keyboard-first browsing and launch behavior works and remains safe around gameplay input ownership.
9. Full-catalog performance remains qualified.
10. Linux real-window startup smoke passes with the hardened survival check.
11. Applicable Linux/macOS/Windows packaging workflows remain green.
12. The new TODO contains no ambiguous unfinished item: each task is complete, explicitly deferred, or superseded with rationale.

## 29. Qualification protocol

Every implementation batch must be qualified against the exact candidate SHA. At milestone closure, the final PR head must pass all applicable workflows, including:

- `Tauri project`;
- `Tauri security`;
- `Tauri Linux packaging`;
- `Tauri Windows packaging`;
- `Tauri macOS packaging`;
- `Build documentation` when documentation paths trigger it.

After merge, the promoted `master` SHA must be inspected before closure is claimed.

This milestone must not use a green ancestor as evidence for a changed candidate.
