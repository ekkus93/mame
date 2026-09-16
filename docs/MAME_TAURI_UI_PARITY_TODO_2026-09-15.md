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

- [ ] Read `docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md` completely before changing code.
- [ ] Read `docs/MAME_TAURI_UI_POST_REMEDIATION_HARDENING_TODO_2026-09-15.md` so existing BIOS/launch/shortcut fixes are preserved.
- [ ] Confirm the current Tauri default UI still differs from original MAME before starting implementation.
- [ ] Record a brief baseline note describing the current mismatch: generic light shell, white/gray surfaces, black text, generic tabs, missing dark navy MAME UI, missing blue selected-row/yellow-text treatment, and missing green status region.
- [ ] Identify the default route/component tree responsible for the current shell.
- [ ] Identify all default-visible CSS files that drive the current shell.
- [ ] Identify any generic shell/navigation components that must be replaced, hidden, or visually converted.

**Completion evidence required:** baseline note, component/CSS map, and no code changes that weaken existing launch/BIOS behavior.

---

## MTP-001 — Preserve reference visual evidence in repo-owned form

- [ ] Add a repo-owned visual parity reference document under `docs/` describing the original MAME UI reference screenshot.
- [ ] Document the two known Tauri comparison captures and why they fail parity.
- [ ] Include explicit reference details: dark navy main surface, blue toolbar, white text, blue selected row, yellow selected text, gray disabled rows, left filter list, central machine list, right Images/Infos panel, and green bottom status region.
- [ ] If practical, add or link committed reference screenshots under a documented path.
- [ ] If binary screenshot commits are not practical through the current tool path, state that clearly in the reference document and keep the textual visual contract complete enough to implement from.
- [ ] Add a checklist that future implementers can use for manual visual review.

**Completion evidence required:** committed visual reference/checklist document.

---

## MTP-002 — Replace generic default shell with original-MAME-like shell

- [ ] Audit `MameShell` and related shell CSS for generic app-dashboard patterns.
- [ ] Remove or hide default-visible top navigation tabs that make the app look unlike original MAME.
- [ ] Preserve access to diagnostics/settings/audit/history only through original-compatible menus, secondary affordances, or non-default debug/developer paths.
- [ ] Replace the `MAME Tauri Frontend` default presentation with an original-MAME-like title/search/header treatment.
- [ ] Ensure the default browser screen, not a generic multi-tab dashboard, is the primary user experience.
- [ ] Ensure startup/loading/error states render inside the original-MAME-like shell unless backend failure prevents the shell from loading.

**Completion evidence required:** component changes plus visual/manual evidence that the generic dashboard shell is not the default visible UI.

---

## MTP-003 — Implement explicit MAME palette tokens and remove system-color product styling

- [ ] Add explicit app theme tokens for MAME navy background, toolbar blue, selected-row blue, selected text yellow, white text, muted gray text, splitter lines, and bottom status green.
- [ ] Apply tokens to `html`, `body`, the app root, and all default-visible MAME shell surfaces.
- [ ] Replace primary product uses of bare `Canvas`, `CanvasText`, and `currentColor` with semantic MAME tokens.
- [ ] Keep system colors only as fallback/accessibility escape hatches, not as the visible product palette.
- [ ] Style buttons, inputs, tabs, list rows, hover states, focus states, and disabled states to match original-MAME-like visuals.
- [ ] Confirm the app cannot render as the generic white/gray GTK-like UI shown in the Tauri comparison screenshots.

**Completion evidence required:** theme/CSS diff and automated/static test proving core surfaces use explicit MAME tokens rather than system colors.

---

## MTP-004 — Match original layout geometry and density

- [ ] Rework the browser layout into the original spatial model: top header/search, blue toolbar band, left filter list, central machine list, right Images/Infos panel, bottom status region.
- [ ] Match dense row heights and compact spacing similar to original MAME.
- [ ] Use thin high-contrast splitters between panels.
- [ ] Keep the left filter panel width visually close to the original.
- [ ] Keep the right image/info panel width visually close to the original.
- [ ] Avoid modern dashboard spacing, rounded-card surfaces, and large empty padding in default MAME browser mode.
- [ ] Preserve responsive behavior only to the extent it does not redesign the default desktop UI.

**Completion evidence required:** layout/CSS diff and visual/manual evidence for panel geometry.

---

## MTP-005 — Filter/category panel parity

- [ ] Render the original filter/category list in original order where supported.
- [ ] Add or restore missing original categories where data support exists.
- [ ] Represent unsupported/deferred categories visibly but without breaking original-like layout.
- [ ] Implement original-like selected filter marker/indicator instead of generic gray row selection.
- [ ] Preserve filter keyboard/mouse behavior.
- [ ] Ensure category/custom-filter deferred states are visually integrated into the MAME-like UI.

**Completion evidence required:** filter panel parity checklist and tests or manual evidence for selecting filters.

---

## MTP-006 — Machine list visual and selection parity

- [ ] Render central machine rows with original-like text density and hierarchy.
- [ ] Implement blue selected-row treatment.
- [ ] Implement yellow selected-row primary text treatment.
- [ ] Implement muted gray unavailable/disabled row treatment.
- [ ] Preserve scrolling behavior and visible scroll position.
- [ ] Preserve search-to-selection/list-position behavior.
- [ ] Preserve single-click selection and double-click/activation behavior where implemented.
- [ ] Preserve all previously fixed launch semantics, including BIOS omission unless explicitly selected.

**Completion evidence required:** machine-row CSS/component changes plus automated or manual evidence for selected, disabled, and normal rows.

---

## MTP-007 — Right Images/Infos panel parity

- [ ] Replace generic detail sidebar behavior with original-like `Images` / `Infos` panel structure.
- [ ] Implement original-like tab/header treatment for Images and Infos.
- [ ] Implement original-like image category selector, including `Snapshots` where applicable.
- [ ] Implement original-like no-image placeholder behavior.
- [ ] Ensure image scaling and panel padding resemble original MAME.
- [ ] Ensure Infos content uses original-like text density and layout.
- [ ] Preserve behavior when no machine is selected.

**Completion evidence required:** right-panel component/CSS changes and visual/manual evidence.

---

## MTP-008 — Bottom status/driver region parity

- [ ] Add or restore the original-like green bottom status/driver panel.
- [ ] Populate it with machine metadata when a machine is selected: year, manufacturer, driver parent/clone status, overall status, graphics status, sound status where available.
- [ ] Render useful original-like empty state when no machine is selected.
- [ ] Keep global backend/app diagnostic details out of the default bottom region unless original MAME would show equivalent user-facing status.
- [ ] Ensure status updates follow selection changes.

**Completion evidence required:** bottom status component/CSS changes and evidence with selected/no-selection states.

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

- [ ] Add static CSS/theme tests preventing default shell regression to `Canvas` / `CanvasText` product styling.
- [ ] Add static or component tests proving explicit selected-row blue and selected-text yellow tokens exist and are used.
- [ ] Add static or component tests proving the bottom status green token exists and is used.
- [ ] Add tests or assertions proving the default-visible shell does not expose the generic dashboard tab row as the main UX.
- [ ] If practical in CI, add screenshot tests for at least one stable empty/configured browser state.
- [ ] If screenshot tests are not practical, document why and add the strongest available static/component tripwires.

**Completion evidence required:** automated regression tripwires in CI.

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
