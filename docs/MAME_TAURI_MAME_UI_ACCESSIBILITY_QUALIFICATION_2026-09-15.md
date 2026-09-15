# MAME Tauri MAME-UI Accessibility and Usability Qualification — 2026-09-15

**Milestone:** MAME UI reproduction (`MUI-018`)  
**Reference spec:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md`  
**Reference parity matrix:** `docs/MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md`

## Scope

This qualification records the accessibility and core-usability audit of the reproduced MAME-style machine/software browser. It covers semantics, keyboard-only operation, focus behavior, status communication, pointer behavior, reduced-motion behavior, and the boundary between frontend input and native MAME gameplay input.

The audit is intentionally based on the shipped React/Tauri structure and its automated regressions. It does not claim pixel-level assistive-technology certification or substitute for platform-specific screen-reader testing.

## Semantic structure

### Machine filters

`MachineFilterPanel.tsx` exposes the canonical filter collection as a labelled `listbox` with `option` children, `aria-selected`, and roving `tabIndex`. The active option is the single normal Tab stop. Up/Down/Home/End update both selection and focus, and Arrow Right transfers focus to the machine region.

### Machine results

`MachineList.tsx` exposes the machine collection as a labelled `listbox` with `option` rows and `aria-selected`. The selected row is the normal Tab stop. Text labels carry description, short name, year, manufacturer, driver status, and availability; availability and status are therefore not color-only.

### Software results

`SoftwareBrowser.tsx` uses a semantic unordered list of native buttons for software rows. Selection is represented textually/visually and keyboard navigation is implemented on the row buttons. This is a deliberate semantic-list pattern rather than an ARIA listbox because each row remains a directly actionable native button.

### Right-side context

Machine and software Images/Info selectors use labelled `tablist`/`tab` semantics with `aria-selected`. Artwork-category controls are native buttons with `aria-pressed`. Machine information uses semantic description lists. Missing artwork is presented as normal text, not as an alert.

### Contextual actions and secondary surfaces

Start, Favorite, Audit, Export, Configure, Software, Session, Settings, Diagnostics, History, Collections, and bulk-audit navigation use native buttons/links with visible text labels. There are no icon-only primary actions in the reproduced browser, so no unlabeled icon-button exception exists.

## Keyboard-only path

The core launch path is operable without a pointer:

1. use the filter list with Up/Down/Home/End;
2. use Arrow Right to enter the machine list;
3. navigate machines with Up/Down/Home/End/Page Up/Page Down;
4. press Enter to activate the selected machine, or use Ctrl/Cmd+Enter from the browser context;
5. use `/` to focus live search when the frontend owns input;
6. press Escape in a non-empty search to clear it before backing out;
7. use Arrow Left/Right to move between filter, machine, and right-panel regions;
8. enter Software context from the selected machine, use its dense keyboard list, and Escape/Back to return to the same machine context;
9. enter Configure and use the explicit `← Machine details` control to return, with focus restored to the originating Configure action.

The milestone regression asserts the material region-navigation and shortcut tokens so these paths cannot silently disappear.

## Focus visibility and restoration

`ContextualSurfaces.css` supplies explicit `:focus-visible` outlines for shell navigation, machine rows, filters, right-panel tabs, contextual actions, artwork categories, and status-bar controls.

No project-owned modal dialog is introduced by this milestone. Machine configuration is a contextual subview, not a modal; it focuses the Back control on entry and restores focus to the Configure button on exit. Native operating-system file/folder pickers are provided by the Tauri dialog plugin and own their platform focus behavior outside the WebView.

Because there are no WebView modal dialogs in the reproduced primary workflow, a DOM focus trap is **not applicable** to this milestone. The relevant requirement is deterministic subview focus restoration, which is implemented and regression-guarded.

## Live regions and announcements

Live regions are limited to bounded status changes such as result ranges, persisted-state errors, launch/export completion, launch errors, and BIOS/query failures. Rapid movement between individual machine rows does not emit a live-region announcement for every selection change. This avoids turning keyboard scanning into excessive status chatter.

## Status communication

Driver status and media availability are emitted as text (`Working`, `Preliminary`, `Available`, `Missing`, `Unknown`, and related labels) in addition to CSS classes. Launch, audit, export, software, and save-state errors are represented by textual messages and alert/status roles. No required state depends on color alone.

## Pointer behavior

Pointer and keyboard selection share the same selected-item state. Clicking a filter or row updates the same state used by keyboard navigation; double-click activation invokes the same primary activation path. Right-panel tabs, artwork categories, pager controls, and contextual actions are native buttons, so pointer input does not maintain a separate hidden selection model.

## Reduced motion

The shell applies a `prefers-reduced-motion: reduce` rule that removes smooth scrolling behavior from the MAME shell subtree. The reproduced UI does not depend on animation for state meaning.

## Controller/gamepad navigation disposition

Global frontend gamepad-navigation shortcuts are **deliberately deferred**. The application already captures browser-reported controller identity for configuration, but native MAME owns gameplay input while a session is active. Adding a second global WebView gamepad-navigation loop would create an avoidable risk of duplicated/interfering input.

For this milestone, keyboard and pointer navigation are the qualified frontend input paths. Browser shortcuts fail closed whenever gameplay input ownership is active or cannot be established. A future controller-navigation implementation must prove an explicit ownership handoff before it can replace this defer.

## Qualification evidence

The following automated layers cover the audited behavior:

- frontend Vitest model/command suites;
- `scripts/tauri/test-mame-ui-reproduction.py` static architecture/interaction regression;
- `scripts/tauri/test-post-closeout-hardening.py`, which invokes the MAME-UI regression inside the mandatory `Tauri project` gate;
- hardened Linux Tauri development-window smoke under Xvfb;
- exact-head frontend format/lint/typecheck/test/build and Rust format/test/Clippy gates.

Platform-specific screen-reader certification remains outside this repository's automated CI scope. Nothing in this document claims such certification.
