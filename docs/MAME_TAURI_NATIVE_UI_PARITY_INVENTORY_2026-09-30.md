# MAME Tauri Native UI Parity Inventory

**Date:** 2026-09-30  
**Reset TODO:** `docs/MAME_TAURI_SELF_CONTAINED_UI_PARITY_RESET_TODO_2026-09-30.md`  
**Reset section:** RESET-002  
**Inventory source:** current `master` after RESET-001 (`39172ed8cb3f0e6e4da2d7027e838952368b3d9a`) plus local inspection of the uploaded `mame-master-2609301143.zip` source snapshot.

## Native/reference view captured for this reset

The reset baseline is the native-MAME screenshot/view and comparable Tauri screenshot/view discussed on 2026-09-30.

Native reference facts:

- The native MAME view is primarily a machine/game selection browser.
- The observed native view reports `88 Games`.
- Native flow prioritizes list selection, machine metadata, and image/info context.
- Native MAME does not present Tauri support/control-plane tools as primary launch controls.

Comparable Tauri facts:

- The comparable Tauri view reports `1-59 of 59`.
- Rows expose `Unknown` local availability.
- The primary toolbar/action area exposes custom Tauri controls beyond native selection.
- The right panel prominently exposes Images/Infos and optional artwork categories.
- Start can be visible before local content availability has been established.

No binary screenshots are added here; this inventory records the view facts and source locations needed for reset implementation.

## RESET-007 count/filter parity resolution

The fixed regression baseline uses the checked-in `listxml-reset-count-parity.xml` fixture, a synthetic MAME identity reported as version `0.288` / build `reset-count-parity`, one active imported metadata generation containing nine machines, no search text, and the `Unfiltered` (`all`) machine filter. The fixture deliberately spans parent/clone, BIOS, device, preliminary/not-working, mechanical, CHD, and non-runnable dimensions so each can be tested independently.

Current native MAME source establishes the unfiltered reference semantics directly. `src/frontend/mame/ui/systemlist.cpp` builds the sorted system list from the driver list and removes only `___empty`. `src/frontend/mame/ui/selgame.cpp` copies that complete sorted list into the displayed list when no machine filter is active. Native availability scanning annotates entries; it is not an unconditional device-row exclusion.

The Tauri catalog query had an extra unconditional `m.is_device = 0` predicate before the selected filter was applied. That hidden base predicate could reduce the unfiltered count independently of every visible filter and was inconsistent with the native source contract. RESET-007 removes it. The regression baseline now returns all nine fixture machines, including the explicit `deviceonly` row.

The same regression suite exercises the formerly suspected count dimensions as explicit filters: parent/clone, BIOS/not-BIOS, working/not-working, mechanical/not-mechanical, and CHD/no-CHD. Category support remains deferred and does not participate in the `Unfiltered` SQL predicate; metadata gaps therefore cannot silently change the base count through a category filter. The observed screenshot values (`88 Games` native versus `1-59 of 59` Tauri) remain historical capture evidence because that exact transient metadata database was not retained, but the hidden semantic difference identified in the production query is removed and the chosen unfiltered semantics now match the native source with a fixed regression fixture.


## Source files inspected

Primary frontend sources:

- `tauri/src/shell/MameShell.tsx`
- `tauri/src/browser/MameBrowser.tsx`
- `tauri/src/browser/MachineFilterPanel.tsx`
- `tauri/src/browser/MachineList.tsx`
- `tauri/src/browser/MachineRightPanel.tsx`
- `tauri/src/browser/SoftwareBrowser.tsx`
- `tauri/src/browser/model.ts`
- `tauri/src/library/libraryQuery.ts`
- `tauri/src/settings/GeneralSettingsPanel.tsx`
- `tauri/src/settings/PathConfigurationPanel.tsx`
- `tauri/src/settings/DiagnosticsPanel.tsx`
- `tauri/src/library/BulkAuditPanel.tsx`
- `tauri/src/library/CollectionManager.tsx`
- `tauri/src/library/FavoriteToggleButton.tsx`
- `tauri/src/session/SessionControlPanel.tsx`

Parity-adjacent tests/tripwires:

- `tauri/src/shell/mameShellComposition.source.test.ts`
- `tauri/src/shell/linuxDesktopCompatibility.source.test.ts`
- `tauri/src/browser/mameBrowserInteraction.source.test.ts`
- `tauri/src/browser/MachineRightPanel.tabs.test.tsx`
- `tauri/src/browser/mameParityTheme.test.ts`
- `tauri/src/browser/MachineRightPanel.test.ts`
- `tauri/src/browser/model.test.ts`
- `tauri/src/browser/mameCatalogState.test.tsx`

## Current Tauri UI inventory

### Shell and secondary tools

`MameShell.tsx` keeps the machine browser as the primary `library` view and moves `session`, `settings`, `audit`, `history`, `collections`, and `diagnostics` into secondary surfaces. That is directionally correct. The problem is that `MameBrowser.tsx` still exposes those secondary tools from a visible primary-toolbar `More` menu.

### Browser toolbar

The machine browser toolbar currently exposes:

- search input: `Search systems...`;
- range label, e.g. `1-59 of 59`;
- `Export`;
- responsive `Details` toggle;
- `More` menu containing Session, Configure Options, Audit, History, Collections, Diagnostics.

### Selected machine actions

The selected-machine action row currently exposes:

- `Start` or `Start Empty`;
- favorite toggle;
- `Configure Machine`;
- `Software List` when software lists exist;
- `Audit`.

This makes the primary action row look like a custom launcher/control panel rather than a native-MAME-like browser.

### Filter rail

`model.ts` currently defines these visible/deferred machine filters:

- `Unfiltered`, `Available`, `Unavailable`, `Working`, `Not Working`, `Mechanical`, `Not Mechanical`;
- deferred `Category`;
- `Favorites`, `BIOS`, `Not BIOS`, `Parents`, `Clones`, `Manufacturer`, `Year`, `Source File`;
- `Save Supported`, `Save Unsupported`, `CHD Required`, `No CHD Required`, `Vertical Screen`, `Horizontal Screen`;
- deferred `Custom Filter`.

This mixes native-style catalog filters, Tauri local-availability filters, and Tauri user-data/power-user filters in one prominent rail.

### Machine list

`MachineList.tsx` row fields include:

- description/title;
- short name;
- year;
- manufacturer;
- driver status;
- local availability.

`libraryQuery.ts` maps local availability to `Available`, `Missing`, and `Unknown`. It maps driver/emulation state to `Working`, `Imperfect`, `Preliminary`, `Runnable`, or `Not runnable`.

The issue is semantic, not merely visual: `Working` is driver/emulation status, while `Available` is local audited content state. In the current layout they have comparable weight, and users can reasonably read `Working` as playable.

### Right panel and artwork/media

`MachineRightPanel.tsx` exposes primary tabs:

- `Images`;
- `Infos`.

The images pane exposes artwork categories:

- `Snapshots`, `Cabinet`, `Control Panel`, `PCB`, `Flyer`, `Title Screen`, `Artwork Preview`.

The info pane exposes:

- Year, Manufacturer, Status, Parent, Source, Save states, Graphics, Sound, Orientation, Software lists;
- flags: `Requires artwork`, `Unofficial`, `No sound hardware`, `Incomplete`.

The current right panel defaults to optional artwork/media presentation and includes copy such as `No image Available`. That reuses availability language and makes optional artwork feel more important than ROM availability/audit state.

### Software browser

`SoftwareBrowser.tsx` exposes:

- `← Machines`;
- software search;
- BIOS selector;
- `Start Empty`;
- `Start`;
- software-list selector;
- software filters;
- software pager;
- software Images/Info panel.

Software-list navigation is legitimate MAME behavior, but Start/Start Empty must follow the same local-content policy, path contract, and missing-content diagnostics as machine Start.

## Tauri-only controls and RESET-003 disposition

| Element | Disposition |
|---|---|
| `Export` | Move to advanced/secondary tools or hide from primary toolbar. |
| `More` menu | Keep visually quiet if retained; diagnostics/history/collections should not feel primary. |
| Session tool | Advanced/contextual only when a session exists. |
| `Configure Options` | Keep, but route unknown/missing-content guidance directly to ROM/content setup. |
| Global `Audit` | Keep as content-verification action; de-emphasize unless needed by unknown/unavailable guidance. |
| `History` | Defer/hide deeper until primary parity is accepted. |
| `Collections` | Defer/hide deeper until primary parity is accepted. |
| `Diagnostics` | Advanced/support-only. |
| Favorite toggle | De-emphasize or move out of the primary launch cluster. |
| `Configure Machine` | Hide/de-emphasize as advanced per-machine launch options. |
| `Software List` | Keep for software-list machines, but present as native-style navigation. |
| Per-machine `Audit` | Keep only as availability guidance or secondary action. |
| `Available` filter | Rename/clarify as local audited content, not driver status. |
| `Unavailable` filter | Harmonize with row copy; prefer missing/unavailable local content wording. |
| `Working` / `Not Working` | Rename/annotate as driver/emulation status. |
| `Favorites` filter | Defer or keep lower-priority; it is custom user data, not reset-critical. |
| Source/save/CHD/screen filters | De-emphasize unless proven native-equivalent for the target view. |
| `Images`/`Infos` tabs | Keep, but availability/configuration guidance must take priority when content is unknown/missing. |
| Artwork category buttons | Hide/de-emphasize empty categories; label as optional media. |
| `No image Available` copy | Replace with optional-artwork wording such as `No artwork image configured`. |
| `Requires artwork` flag | Keep as driver metadata, but separate from local ROM/content availability. |
| Software `Start Empty` | Keep only under explicit policy and same gating/diagnostics as machine Start. |

## Confusing driver/emulation status with ROM availability

The current UI has these hazards:

1. `Working` and `Available` appear in the same filter rail but answer different questions.
2. Row driver-status and local-availability badges have comparable visual weight.
3. Details use `Status` without clarifying that it is driver/emulation status.
4. `Unknown` means unaudited/unverified, not necessarily broken or unplayable.
5. Start is currently disabled for non-runnable driver metadata but not fully gated on unknown/missing local content.

RESET-003 and RESET-005 must split these concepts in copy, visual hierarchy, and backend launch policy.

## Optional artwork/media hazards

The current UI can make optional artwork/media look required because:

1. The right panel defaults to `Images`.
2. `Snapshots` is always included even when no screenshot exists.
3. Artwork category buttons can stay visible even when empty.
4. `No image Available` uses availability language.
5. The `Requires artwork` driver flag appears near optional artwork controls.

RESET-003 must hide/de-emphasize empty artwork categories and rewrite copy so artwork is clearly optional media.

## Intended primary UI shape after reset

The primary UI should be:

1. A native-like machine selection browser with search, filter rail, list, and details/images context.
2. A primary Start action that is enabled only for launchable content under tested policy.
3. Local content state copy that says what it means: `Available locally`, `Not audited`, or `Missing content`.
4. Driver/emulation status labeled separately from local content state.
5. ROM/content setup guidance surfaced when content state requires it.
6. Optional artwork/media visually secondary and hidden/de-emphasized when empty.
7. Support/advanced tools reachable but not dominant: diagnostics, collections, history, export, per-machine overrides, external/development runtime overrides.

## RESET-003 implementation order

1. Rename/annotate driver-status vs local-content labels and filters.
2. Change `Unknown` local availability copy to unaudited/not-audited language.
3. Move or de-emphasize Favorite, Configure Machine, and Audit from the selected-machine action cluster.
4. Hide/de-emphasize optional artwork categories when empty and fix optional-artwork copy.
5. Keep software-list navigation but make Start/Start Empty follow the same availability policy as machine launch.
6. Add source/component tests that lock the revised copy and primary action cluster.

## RESET-002 acceptance mapping

This document captures the native reference facts, the comparable current Tauri UI facts, every visible primary-path Tauri-only control class, the current list/filter/status/details inventory, the driver-status/local-availability confusion, the optional-artwork confusion, and the intended primary UI shape after reset.
