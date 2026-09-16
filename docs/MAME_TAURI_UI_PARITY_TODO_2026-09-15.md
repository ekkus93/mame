# MAME Tauri UI Parity TODO — 2026-09-15

**Repository:** `ekkus93/mame`  
**Spec:** `docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md`  
**Goal:** make the Tauri frontend look and behave like the original MAME UI closely enough that a normal user should not notice the frontend was rewritten in Tauri.  
**Prior hardening ledger:** `docs/MAME_TAURI_UI_POST_REMEDIATION_HARDENING_TODO_2026-09-15.md`

This TODO is the canonical backlog for original-MAME visual and interaction parity. The prior hardening TODO closed behavior/launch semantics; it did not close visual parity.

## Mandatory execution rules

- Do not redesign the app.
- Treat the original MAME UI as the product spec.
- Do not mark a task complete just because unit tests pass.
- Do not close the parity pass without visual evidence.
- Use exact-head CI evidence for each PR/candidate.
- After every merge, reload this TODO from promoted `master`, identify the next unchecked item, and continue.
- If screenshots/reference captures are unavailable in the implementation environment, create a textual visual parity report from the known reference details and mark screenshot-dependent subtasks blocked only if they cannot be completed without new user-provided assets.

---

## MTP-000 — Baseline and source-of-truth reset

- [x] Read `docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md` completely before changing code.
- [x] Read `docs/MAME_TAURI_UI_POST_REMEDIATION_HARDENING_TODO_2026-09-15.md` so existing BIOS/launch/shortcut fixes are preserved.
- [x] Confirm the current Tauri default UI still differs from original MAME before starting implementation.
- [x] Record a brief baseline note describing the current mismatch: generic light shell, white/gray surfaces, black text, generic tabs, missing dark navy MAME UI, missing blue selected-row/yellow-text treatment, and missing green status region.
- [x] Identify the default route/component tree responsible for the current shell.
- [x] Identify all default-visible CSS files that drive the current shell.
- [x] Identify any generic shell/navigation components that must be replaced, hidden, or visually converted.

**MTP-000 evidence:** The parity spec and prior hardening TODO were re-read from promoted `master` before implementation. Baseline mismatch is recorded in `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md`: the old default-visible Tauri shell was a generic light tabbed application rather than the original dark/navy MAME interface. The default route is `App.tsx` -> `MameShell` -> `MameBrowser`; the primary CSS surfaces are `index.css`, `MameShell.css`, and `ContextualSurfaces.css`. The generic default-visible tab shell in `MameShell` was identified as the shell surface to hide/convert while preserving secondary panels behind non-default controls. Prior BIOS/software-launch hardening files were not weakened by this slice.

---

## MTP-001 — Preserve reference visual evidence in repo-owned form

- [x] Add a repo-owned visual parity reference document under `docs/` describing the original MAME UI reference screenshot.
- [x] Document the two known Tauri comparison captures and why they fail parity.
- [x] Include explicit reference details: dark navy main surface, blue toolbar, white text, blue selected row, yellow selected text, gray disabled rows, left filter list, central machine list, right Images/Infos panel, and green bottom status region.
- [x] If practical, add or link committed reference screenshots under a documented path.
- [x] If binary screenshot commits are not practical through the current tool path, state that clearly in the reference document and keep the textual visual contract complete enough to implement from.
- [x] Add a checklist that future implementers can use for manual visual review.

**MTP-001 evidence:** Added `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md`. It documents the original MAME screenshot, the two failed light Tauri captures, the expected visual landmarks, and a manual parity checklist. Binary screenshot commits were not practical through the current Ralph text-file write path, so the document explicitly records that limitation and keeps the textual visual contract complete.

---

## MTP-002 — Replace generic default shell with original-MAME-like shell

- [x] Audit `MameShell` and related shell CSS for generic app-dashboard patterns.
- [x] Remove or hide default-visible top navigation tabs that make the app look unlike original MAME.
- [x] Preserve access to diagnostics/settings/audit/history only through original-compatible menus, secondary affordances, or non-default debug/developer paths.
- [x] Replace the `MAME Tauri Frontend` default presentation with an original-MAME-like title/search/header treatment.
- [x] Ensure the default browser screen, not a generic multi-tab dashboard, is the primary user experience.
- [x] Ensure startup/loading/error states render inside the original-MAME-like shell unless backend failure prevents the shell from loading.

**MTP-002 evidence:** `MameShell.tsx` no longer renders the default-visible `mame-shell-nav` / `Application views` tab row. The library/browser view is the default primary surface. Session, settings, audit, history, collections, and diagnostics remain reachable from the bottom status/action strip and render as secondary surfaces with an explicit return to Machine Selection. `MameShell.css` and `mameTheme.css` convert the default presentation to dark/navy MAME-like surfaces with a compact blue toolbar and green bottom status region.

---

## MTP-003 — Implement explicit MAME palette tokens and remove system-color product styling

- [x] Add explicit app theme tokens for MAME navy background, toolbar blue, selected-row blue, selected text yellow, white text, muted gray text, splitter lines, and bottom status green.
- [x] Apply tokens to `html`, `body`, the app root, and all default-visible MAME shell surfaces.
- [x] Replace primary product uses of bare `Canvas`, `CanvasText`, and `currentColor` with semantic MAME tokens.
- [x] Keep system colors only as fallback/accessibility escape hatches, not as the visible product palette.
- [x] Style buttons, inputs, tabs, list rows, hover states, focus states, and disabled states to match original-MAME-like visuals.
- [x] Confirm the app cannot render as the generic white/gray GTK-like UI shown in the Tauri comparison screenshots.

**MTP-003 evidence:** Added `tauri/src/mameTheme.css` and imported it after `index.css` from `main.tsx`. `index.css` now uses `var(--mame-text)` and `var(--mame-bg)` instead of `CanvasText` / `Canvas`. `MameShell.css` uses explicit MAME tokens for shell, toolbar, selected rows, muted disabled rows, splitters, and the green bottom status bar. `mameParityTheme.test.ts` adds static tripwires for theme import order, required tokens, default shell token use, and absence of default-shell `Canvas` / `CanvasText` product styling.

---

## MTP-004 — Match original layout geometry and density

- [x] Rework the browser layout into the original spatial model: top header/search, blue toolbar band, left filter list, central machine list, right Images/Infos panel, bottom status region.
- [x] Match dense row heights and compact spacing similar to original MAME.
- [x] Use thin high-contrast splitters between panels.
- [x] Keep the left filter panel width visually close to the original.
- [x] Keep the right image/info panel width visually close to the original.
- [x] Avoid modern dashboard spacing, rounded-card surfaces, and large empty padding in default MAME browser mode.
- [x] Preserve responsive behavior only to the extent it does not redesign the default desktop UI.

**MTP-004 evidence:** `MameShell.css` now drives the default browser as a compact original-MAME-like spatial model: centered title/header via the browser shell, blue toolbar, left filters, central machine list, right panel, high-contrast splitters, dense rows, and green bottom status strip. The old dashboard spacing, generic rounded controls, and tab shell were removed from the default-visible browser path. Narrow-screen hooks from the prior milestone regression were preserved in `ContextualSurfaces.css`.

---

## MTP-005 — Filter/category panel parity

- [x] Render the original filter/category list in original order where supported.
- [x] Add or restore missing original categories where data support exists.
- [x] Represent unsupported/deferred categories visibly but without breaking original-like layout.
- [x] Implement original-like selected filter marker/indicator instead of generic gray row selection.
- [x] Preserve filter keyboard/mouse behavior.
- [x] Ensure category/custom-filter deferred states are visually integrated into the MAME-like UI.

**MTP-005 evidence:** `model.ts` now exposes `MAME_BROWSER_FILTER_NAV_ITEMS`, inserting visible deferred `Category` and `Custom Filter` entries into the original-like filter order while retaining the supported authoritative filters. `MachineFilterPanel.tsx` renders the full navigation list with a visible diamond indicator, deferred styling, and preserved listbox/option keyboard semantics including right-arrow movement to the machine list. `mameParityTheme.test.ts` asserts the filter indicator and deferred landmarks remain present.

---

## MTP-006 — Machine list visual and selection parity

- [x] Render central machine rows with original-like text density and hierarchy.
- [x] Implement blue selected-row treatment.
- [x] Implement yellow selected-row primary text treatment.
- [x] Implement muted gray unavailable/disabled row treatment.
- [x] Preserve scrolling behavior and visible scroll position.
- [x] Preserve search-to-selection/list-position behavior.
- [x] Preserve single-click selection and double-click/activation behavior where implemented.
- [x] Preserve all previously fixed launch semantics, including BIOS omission unless explicitly selected.

**MTP-006 evidence:** `MachineList.tsx` now classifies rows as selected and unavailable; `MameShell.css` applies dense row sizing, blue selected-row treatment, yellow selected text, and muted unavailable rows. Existing selection, scrolling, search-to-selection, click, double-click activation, and launch/BIOs semantics were left intact rather than rewritten in this visual slice.

---

## MTP-007 — Right Images/Infos panel parity

- [x] Replace generic detail sidebar behavior with original-like `Images` / `Infos` panel structure.
- [x] Implement original-like tab/header treatment for Images and Infos.
- [x] Implement original-like image category selector, including `Snapshots` where applicable.
- [x] Implement original-like no-image placeholder behavior.
- [x] Ensure image scaling and panel padding resemble original MAME.
- [x] Ensure Infos content uses original-like text density and layout.
- [x] Preserve behavior when no machine is selected.

**MTP-007 evidence:** `MachineRightPanel.tsx` now exposes original-MAME-style `Images` / `Infos` tab labels and `Machine Images and Infos` tablist semantics. `ArtworkPane` always preserves the `Snapshots` selector even when discovery returns no slots, and the no-artwork path renders an original-like `No image Available` placeholder. `EmptyMachineRightPanel` preserves the same Images/Snapshots/no-image structure when no machine is selected, while loading and error states remain inside the right panel. `MameShell.css` tightens the right-panel tabs, artwork category strip, image frame, no-image placeholder, and info typography/padding to match the dense dark MAME layout. `mameParityTheme.test.ts` asserts the Images/Infos/Snapshots/no-image landmarks remain present.

---

## MTP-008 — Bottom status/driver region parity

- [x] Add or restore the original-like green bottom status/driver panel.
- [x] Populate it with machine metadata when a machine is selected: year, manufacturer, driver parent/clone status, overall status, graphics status, sound status where available.
- [x] Render useful original-like empty state when no machine is selected.
- [x] Keep global backend/app diagnostic details out of the default bottom region unless original MAME would show equivalent user-facing status.
- [x] Ensure status updates follow selection changes.

**MTP-008 evidence:** Added `MachineDriverStatus.tsx` and routed selected machine/detail state from `MameBrowser.tsx` into a green `mame-driver-status` footer. The status string now updates with selected-machine detail and includes description/short name, year, manufacturer, parent/clone/BIOS status, overall driver status, graphics/orientation, sound, and save-state status. Empty, loading, and detail-error states render useful original-like status text. `MameShell.tsx` no longer fills the default bottom strip with app/backend diagnostics; those controls remain secondary affordances, while the selected-machine driver/status line is rendered in the default browser. Static tripwires assert `MachineDriverStatus`, `Selected machine driver status`, and the driver status class remain wired.

---

## MTP-009 — Original-like command/actions flow

- [ ] Restore or implement original-like `Configure Options` and `Configure Machine` actions in the main browser experience.
- [ ] Place launch/configure actions where original users expect them, not in a generic toolbar/dashboard pattern.
- [ ] Keep software-part selection and BIOS selection visually integrated into the original-MAME-like UI.
- [ ] Preserve Start Empty and software launch behavior from the prior hardening pass.
- [ ] Ensure disabled/unavailable actions have original-like visual disabled states.
- [ ] Ensure errors are visible without turning the whole UI into a generic alert/card page.

**Completion evidence required:** action-flow code changes and behavioral tests or manual evidence for launch/configure flows.

---

## MTP-010 — Keyboard and mouse behavior parity

- [ ] Add or extend tests for Up/Down selection movement.
- [ ] Add or extend tests for PageUp/PageDown movement.
- [ ] Add or extend tests for Home/End where supported.
- [ ] Add or extend tests for Enter activation.
- [ ] Add or extend tests for Escape/back behavior.
- [ ] Add or extend tests for search typing/focus behavior.
- [ ] Add or extend tests for clicking filters, selecting rows, and right-panel tabs.
- [ ] Ensure gameplay-input ownership and shortcut fixes from prior hardening are preserved.

**Completion evidence required:** frontend behavioral tests and passing Tauri project CI.

---

## MTP-011 — Startup, metadata, and empty-state parity

- [ ] Replace generic `0 machines` / metadata inactive empty state with original-MAME-like visual presentation.
- [ ] Distinguish not-configured, metadata-import-needed, import-in-progress, import-failed, metadata-loaded, and no-ROMs/unknown-availability states.
- [ ] Ensure all states remain inside the original-MAME-like layout.
- [ ] Provide a clear path to configure/import metadata without leaving the user in a generic blank white shell.
- [ ] Verify that a configured installation reaches a populated machine list rather than a permanent generic empty state.

**Completion evidence required:** state rendering tests or manual evidence for each state that can be produced locally/CI.

---

## MTP-012 — Visual regression tripwires

- [x] Add static CSS/theme tests preventing default shell regression to `Canvas` / `CanvasText` product styling.
- [x] Add static or component tests proving explicit selected-row blue and selected-text yellow tokens exist and are used.
- [x] Add static or component tests proving the bottom status green token exists and is used.
- [x] Add tests or assertions proving the default-visible shell does not expose the generic dashboard tab row as the main UX.
- [ ] If practical in CI, add screenshot tests for at least one stable empty/configured browser state.
- [x] If screenshot tests are not practical, document why and add the strongest available static/component tripwires.

**MTP-012 evidence:** Added `mameParityTheme.test.ts`, which verifies theme import order, palette tokens, selected-row/yellow-text and green-status token usage, absence of default shell `Canvas` / `CanvasText` styling, removal of the default-visible generic dashboard tab row, Images/Infos/Snapshots/no-image right-panel landmarks, and selected-machine driver-status wiring. Screenshot tests remain open for a later pass if a stable WebView/screenshot harness is added; the current strongest available tripwires are static/source tests and the textual visual checklist.

---

## MTP-013 — Linux desktop-environment compatibility preservation

- [ ] Verify the parity changes do not reintroduce reliance on the original native UI code path that had Linux desktop-environment issues.
- [ ] Document how the Tauri/WebView path avoids or reduces the original desktop-environment problem.
- [ ] Check focus/keyboard assumptions against GNOME/KDE, Wayland/X11 considerations where practical.
- [ ] Ensure explicit theme tokens render consistently regardless of host light/dark mode.
- [ ] Ensure high-contrast/focus affordances remain usable.

**Completion evidence required:** compatibility note and relevant tests/manual checks.

---

## MTP-014 — Documentation and user-facing explanation cleanup

- [ ] Update README or relevant docs to state that the Tauri frontend aims to preserve original MAME UI behavior while improving Linux desktop-environment compatibility.
- [ ] Document any intentional deviations from original MAME UI.
- [ ] Document how to run the app and verify visual parity locally.
- [ ] Document how to gather/update reference screenshots.
- [ ] Remove or revise docs that describe the generic Tauri shell as the intended product direction.

**Completion evidence required:** docs diff with clear parity statement and verification instructions.

---

## MTP-015 — Final reconciliation and closure

- [ ] Reconcile every task/subtask in this TODO as complete, explicitly deferred with rationale, superseded with rationale, or blocked with concrete user-required input.
- [ ] Attach final visual evidence: screenshots, screenshot-test artifacts, or a complete manual visual parity report.
- [ ] Qualify the exact final PR head through all applicable project/security/platform/documentation workflows.
- [ ] Merge only an exact-head-qualified candidate through the gated Ralph Bridge path.
- [ ] Reload this TODO from promoted `master` after merge and verify the promoted SHA.
- [ ] Verify applicable post-merge `master` CI before claiming closure.
- [ ] Do not disable or stop any related scheduled work until this visual parity pass is actually closed.

**Completion evidence required:** exact PR/head SHA, CI run IDs, promoted master SHA, post-merge CI evidence, and final visual parity evidence.

---

## Completion rule

This parity effort is not complete until the default Tauri frontend visibly and behaviorally matches original MAME, every task above is reconciled, final exact-head CI passes, the qualified candidate is merged, the TODO is reloaded from promoted `master`, post-merge CI is verified, and final visual evidence is recorded.
