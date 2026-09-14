# MAME Tauri MAME-UI Parity Matrix — 2026-09-14

**Milestone:** MAME UI reproduction (`MUI-001` through `MUI-020`)  
**Reference baseline:** repository `master` at `0c95e4451406e44af6b530de40c875e5c0881966`  
**Primary upstream references:** `src/frontend/mame/ui/selgame.cpp`, `selmenu.cpp`, `selsoft.cpp`, and `utils.cpp`  
**Implementation specification:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md`

## Purpose

This document is the auditable behavior inventory required by MUI-001. The current upstream MAME machine/software selection UI is the product-behavior reference. The React/Tauri implementation does not copy MAME's C++ renderer; it reproduces the selection model, information architecture, important interaction semantics, and accepted contextual actions while preserving the project's existing Rust authority boundary and external native MAME process.

The disposition vocabulary is:

- **existing** — the Tauri project already has the backend/domain capability and it can be reused;
- **frontend** — backend data/capability is sufficient and the remaining work is presentation/state/navigation;
- **backend gap** — accepted for this milestone and requires a typed Rust/storage/query extension;
- **defer** — deliberately excluded from this milestone with rationale recorded here;
- **N/A** — behavior belongs to MAME's in-process UI/runtime and is not appropriate to reproduce in the external Tauri frontend.

## Canonical machine-selection behavior

`menu_select_game` is the root machine-selection surface. It builds the machine list, restores the last-used machine and filter, restores the right-panel mode and image category, and keeps the main selection state while filters/search change when possible. The root browser is not a dashboard of independent feature panels: the selected machine is the center of the interaction model.

Material behavior to reproduce:

- a persistent machine-selection list is the dominant center surface;
- a persistent left filter panel drives the list;
- a persistent right panel switches between **Images** and **Info**;
- search is part of the selection surface and immediately affects displayed results;
- the current selection drives artwork, information, toolbar/context actions, software browsing, configuration, audit, favorites, and launch;
- selection is remembered/reselected where possible when the result set changes;
- favorites are a filter plus a contextual toggle, not a separate primary dashboard shelf;
- selecting a machine launches it or enters software selection when software selection is required;
- audit failure is surfaced before launch rather than silently ignored;
- machine configuration is contextual to the selected machine;
- the root selection UI remembers last machine, filter, right-panel mode, and right-image mode.

## Canonical left machine filters

`utils.cpp` defines the current machine-filter inventory. The accepted Tauri disposition is below.

| MAME filter | Tauri disposition | Notes |
| --- | --- | --- |
| Unfiltered | frontend | Existing machine query is sufficient. |
| Available | existing/frontend | Existing audit-derived availability query. |
| Unavailable | existing/frontend | Existing `missing` availability classification; `unknown` remains separately visible rather than falsely unavailable. |
| Working | existing/frontend | Existing `driverStatus=good`. |
| Not Working | backend gap | Implement accepted combined imperfect/preliminary semantics in typed query contract. |
| Mechanical | backend gap | Metadata already records machine mechanical state; expose query predicate. |
| Not Mechanical | backend gap | Complement of mechanical predicate. |
| Category | defer | Upstream MAME category behavior depends on category/INI data not currently imported by the project. Defer until a defined category-data ingestion source exists rather than inventing categories. |
| Favorites | backend gap | Favorite storage exists; integrate it into machine query/filter semantics. |
| BIOS | backend gap | `is_bios` exists in machine detail/storage; expose query predicate. |
| Not BIOS | backend gap | Complement of BIOS predicate. |
| Parents | existing/frontend | Existing `parentsOnly`. |
| Clones | existing/frontend | Existing `clonesOnly`. |
| Manufacturer | existing/frontend | Existing exact manufacturer query; UI must expose choices or bounded text semantics. |
| Year | existing/frontend | Existing exact year query. |
| Source File | backend gap | Source file is imported/listed but not currently filterable. |
| Save Supported | backend gap | `driver_savestate` is imported; expose supported predicate. |
| Save Unsupported | backend gap | Complement of save-supported predicate. |
| CHD Required | backend gap | Implement only from imported disk/ROM metadata with deterministic semantics; do not infer from filenames. |
| No CHD Required | backend gap | Complement of accepted CHD predicate. |
| Vertical Screen | backend gap | Derive from imported display orientation/rotation. |
| Horizontal Screen | backend gap | Complement of vertical-screen predicate for display-bearing systems. |
| Custom Filter | defer | Upstream supports persisted multi-clause custom filter composition. This is not required for the first reproduced primary workflow and would introduce a separate filter-expression persistence model. The fixed canonical filters above remain required. |

The Tauri UI may expose `Unknown availability` as a project-specific status/filter because audit provenance can legitimately be unavailable. It must not misrepresent unknown as MAME's unavailable state.

## Canonical center list behavior

The MAME list is dense and selection-oriented. The Tauri reproduction must therefore avoid card-per-machine presentation.

Accepted behavior:

- description/title is the primary visible label;
- short name is available as compact secondary identifying metadata;
- parent/clone/status/availability/favorite state may use compact indicators;
- Up/Down move one row;
- Page Up/Page Down move by a viewport-sized increment;
- Home/End move to first/last selectable row;
- the selected row remains visible;
- pointer selection and keyboard selection remain synchronized;
- Enter/Select invokes the primary action for the selection;
- list navigation must remain deterministic as live search/filter results replace the current result set;
- the existing fail-closed gameplay-input ownership rule remains stronger than browser shortcuts while native MAME is running.

## Canonical search behavior

MAME search is inline with the browser and changes the displayed set without a submit button. Machine search uses normalized approximate/search fields from the machine database; software search considers short name, long name, and alternate titles.

Tauri decision:

- implement debounced live machine search using imported short name, description and manufacturer immediately;
- include additional imported searchable names only if they can be added without making list queries unbounded;
- retain `/` as the project shortcut to focus search;
- Escape clears a non-empty search before backing out of a secondary view;
- use request sequencing/cancellation guards so stale asynchronous responses never replace a newer search result.

## Canonical right Images/Info panel

`selmenu.cpp` defines two right-panel modes: **Images** and **Info**. The image side supports these canonical artwork categories:

1. Snapshots
2. Cabinet
3. Control Panel
4. PCB
5. Flyer
6. Title Screen
7. Ending
8. Artwork Preview
9. Bosses
10. Logo
11. Versus
12. Game Over
13. HowTo
14. Scores
15. Select
16. Marquee
17. Covers

Tauri disposition:

- **Images/Info modes:** frontend, required;
- **current project artwork:** existing, reused through the safe local artwork backend;
- **missing accepted artwork categories:** backend/config gap, extend only with bounded configured local roots and the same path-containment policy;
- **missing artwork:** normal non-error state;
- **clone-to-parent fallback:** accepted when the existing safe artwork lookup can prove both names are catalog identities and containment checks remain intact;
- **Info:** frontend from existing machine detail plus accepted compact audit availability summary;
- **DAT pages beyond imported metadata:** defer. MAME's DAT action can expose external/history DAT sources the project does not currently ingest. Do not fabricate equivalent data.

Right-panel mode and selected image category are remembered UI state in MAME and are accepted persistence requirements for the Tauri milestone.

## Canonical toolbar/context actions

The machine toolbar in `selmenu.cpp` includes favorite, export displayed list, audit media, and information/DAT actions. Software selection includes favorite and information/DAT actions.

| Action | Tauri disposition | Decision |
| --- | --- | --- |
| Favorite toggle | existing/frontend | Required through typed favorite backend. |
| Audit media | existing/frontend | Required through selected-machine audit backend; bulk audit remains a secondary operation. |
| Export displayed list | backend gap | Implement a bounded project-owned export of the current machine result set through an explicit save destination; no generic filesystem capability. |
| Information | frontend | Required contextual machine/software info surface. |
| DAT view | defer | No authoritative DAT/history source is currently imported. Existing metadata/info is not to be relabeled as DAT parity. |
| Configure machine | existing/frontend | Required contextual machine settings. |
| Software browser | existing/frontend plus gaps | Required for systems with software lists. |
| Launch/Start | existing/frontend | Required and visually dominant. |

## Remembered UI state

MAME persists the following relevant selection state:

- last-used machine;
- last-used filter (including filter value/subfilter where applicable);
- machine right-panel mode;
- machine right-image category;
- software right-panel mode;
- software right-image category;
- remember-last/reselection behavior.

Tauri disposition: **backend gap** for durable project-owned UI state. Add an explicit versioned UI-state structure to project settings rather than scattering persistence across browser local storage. State-loading failure must preserve a usable default browser.

## Keyboard/focus behavior

`menu_select_launch::handle_keys` establishes the material navigation semantics:

- Select activates the focused filter or primary list selection;
- Back exits a non-root selection context when search is empty;
- Cancel/Escape clears search first, then performs root/back behavior;
- Left/Right changes right-panel mode/page when that region owns focus;
- Up/Down navigate the focused filter/list/info region;
- Page Up/Page Down page the focused region;
- Home/End jump within the focused region;
- focus-next/focus-previous rotate among focus regions;
- paste/search character input feeds the browser search;
- pointer input can select rows, filters, toolbar items, panel tabs/categories, and dividers.

Tauri decision: reproduce the focus graph semantically rather than emulating MAME's raw input subsystem. Native MAME owns gameplay input while a session is active, and the existing fail-closed ownership check remains mandatory.

## Software-selection behavior

`selsoft.cpp` builds software entries from compatible software lists, groups parents/clones, supports filtering/search, remembers right-panel/image state, exposes software favorites/DAT behavior, and can prepend a start-without-software entry when the machine can boot without required media.

Reference software filters are:

- Unfiltered;
- Available;
- Unavailable;
- Favorites;
- Parents;
- Clones;
- Year;
- Publisher;
- Developer;
- Distributor;
- Author;
- Programmer;
- Supported;
- Partially Supported;
- Unsupported;
- Release Region;
- Device Type;
- Software List;
- Custom Filter.

Milestone decisions:

- contextual software browser: required;
- dense list + live text search: required;
- parent/clone, year, publisher, support and software-list filters: accepted, adding typed backend query fields as needed;
- developer/distributor/author/programmer/region/device-type filters: accepted only where values are already imported or can be added from listxml/software-list metadata without external data sources;
- software availability: backend gap; implement only with trustworthy project audit/content provenance, otherwise expose Unknown rather than guessing;
- software favorites: backend gap; accepted after a stable machine/list/software composite identity is used;
- software custom composite filter: defer for the same reason as machine custom filters;
- software DAT pages: defer until an authoritative DAT source exists.

## Start-empty, multipart software and BIOS selection

These are real launch-flow behaviors, not visual decoration.

### Start empty

**Accepted — backend gap.** MAME exposes `[Start empty]` when no required image device prevents boot without media. Add an explicit contextual Start Empty action when the imported/runtime metadata can establish that it is legal. It launches the selected machine without a software argument through the existing typed launch path.

### Multipart software

**Accepted — backend gap.** MAME asks the user to choose a software package part when more than one compatible part can be launched. Extend the software metadata contract to carry launchable parts and show a bounded part-selection dialog before launch. Never concatenate arbitrary frontend strings into a command line.

### BIOS selection

**Accepted — backend gap.** MAME can require/offer BIOS selection before launch. Add typed BIOS choices from authoritative MAME metadata and pass the selected BIOS through a constrained launch option owned by Rust. Do not expose a generic arbitrary-argument field to achieve parity.

## Project-extension placement decisions

Existing project capabilities remain in scope but must stop competing with the machine browser as peer dashboard panels:

- General Settings → global secondary Settings surface;
- Diagnostics → dedicated troubleshooting secondary surface;
- active session/runtime controls → shell status plus contextual session surface;
- save states → selected-machine/active-session contextual surface;
- bulk audit → toolbar/menu operation with progress;
- Recent History → secondary library view/filter/menu;
- Collections → secondary library/filter-management surface;
- Favorites shelf → retire after Favorites filter/action replacement is complete.

## Parity matrix by behavior

| Behavior | Current Tauri capability | Disposition / implementation track |
| --- | --- | --- |
| Three-region machine browser | Current `LibraryBrowser` contains pieces but uses dashboard/panel composition | frontend — MUI-002/003/005/006 |
| Dense machine list | Existing paged machine data | frontend — MUI-005 |
| Live search | Submit-driven search exists | frontend — MUI-004 |
| Filter panel | Form fields exist, not MAME filter inventory | frontend + backend gaps — MUI-003/015 |
| Availability | Audit-derived classification exists | existing — MUI-003 |
| Favorites | Machine favorite CRUD exists | query/frontend gap — MUI-003/007/015 |
| Images panel | Safe artwork discovery/serving exists | frontend + category gaps — MUI-006/015 |
| Info panel | Machine detail exists | frontend — MUI-006 |
| Selected-machine audit | Existing audit backend/panel | frontend relocation — MUI-007 |
| Machine configuration | Existing machine settings | frontend relocation — MUI-009 |
| Controller configuration | Existing controller settings | frontend relocation — MUI-009 |
| Machine launch | Existing typed supervised launch | frontend relocation — MUI-007/010 |
| Session controls | Existing authenticated runtime controls | frontend relocation — MUI-010 |
| Save states | Existing managed state backend/UI | frontend relocation — MUI-011 |
| Software browser | Existing software list query/launch | frontend rewrite + metadata gaps — MUI-008/015 |
| Start empty | Machine-only launch exists but legality metadata not modeled | backend/UI gap — MUI-008/015 |
| Multipart software | Not modeled | backend/UI gap — MUI-008/015 |
| BIOS selection | Not modeled as typed contextual choice | backend/UI gap — MUI-008/015 |
| Export displayed list | No bounded export command | backend/UI gap — MUI-007/015 |
| DAT/history pages | No authoritative source | defer |
| Last machine/filter/panel/image persistence | No complete versioned MAME-style view state | backend gap — MUI-003/006/008/015 |
| Page navigation/focus regions | Partial list keyboard support exists | frontend — MUI-005/012 |
| Project extensions | All capabilities exist as dashboard panels | frontend relocation — MUI-014/016 |
| Security/process boundary | Rust authority + external supervised MAME already established | existing; invariant across all tracks |

## Invariants for every implementation batch

1. MAME remains a separately supervised native process for this milestone.
2. Video frames, PCM audio, timing and gameplay input do not move through the WebView/Tauri IPC.
3. Privileged filesystem/process/database operations remain in typed Rust commands.
4. No generic shell, arbitrary filesystem, opener or unrestricted HTTP capability is added for UI parity.
5. Tauri event names remain centralized and valid.
6. Browser keyboard/controller shortcuts fail closed whenever native MAME owns gameplay input.
7. Unknown audit/media state is never relabeled as available or unavailable without provenance.
8. New list payload fields must be bounded; expensive detail/artwork data remains selection-driven.
9. Exact-head CI remains authoritative for implementation claims.

## MUI-001 closure

The canonical machine UI, software UI, filter inventories, toolbar/context actions, remembered state, keyboard behavior, and special launch flows have been inventoried above. Every behavior required by MUI-001 now has an explicit disposition. Later tracks must update this matrix if implementation evidence changes a decision; they must not silently drop a parity gap.
