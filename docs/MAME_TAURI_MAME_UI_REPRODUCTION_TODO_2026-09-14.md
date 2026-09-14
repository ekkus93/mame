# MAME Tauri MAME-UI Reproduction TODO — 2026-09-14

**Repository:** `ekkus93/mame`  
**Baseline:** `master` at `7b7486a9a631074efd20096c0580d391ef649360`  
**Spec:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md`  
**Scope:** reproduce the current upstream MAME machine/software selection UX in React/Tauri while preserving the existing Rust backend, native MAME process boundary, security model, and project-specific capabilities.

## MUI-001 — Inventory the canonical MAME UI behavior

- [ ] Inspect `src/frontend/mame/ui/selgame.cpp` and document machine-selection behavior relevant to the Tauri reproduction.
- [ ] Inspect `src/frontend/mame/ui/selmenu.cpp` and document the left filter panel, center selection list, right Images/Info panel, toolbar, artwork categories, and navigation behavior.
- [ ] Inspect `src/frontend/mame/ui/selsoft.cpp` and document software-list selection behavior.
- [ ] Inspect the current MAME filter implementation and enumerate the machine filter types exposed by the reference UI.
- [ ] Identify MAME toolbar/context actions including favorite, export, audit, and information/DAT behavior.
- [ ] Identify remembered UI state including last machine, last filter, right-panel mode, and artwork/image mode.
- [ ] Identify keyboard/UI-input behaviors that materially affect browser parity.
- [ ] Identify software start-empty, BIOS-selection, and multipart software behavior that requires a parity decision.
- [ ] Build a parity matrix mapping each relevant MAME UI behavior to: existing Tauri capability, backend gap, frontend-only work, explicit defer, or not applicable.
- [ ] Keep the parity matrix linked from this TODO or the associated spec so fidelity decisions remain auditable.

## MUI-002 — Establish the new frontend shell architecture

- [ ] Introduce a dedicated `shell/` composition layer for the MAME-style workspace.
- [ ] Introduce browser components for filters, search, machine list, and selection details.
- [ ] Make `App.tsx` a thin readiness/routing/composition layer rather than a vertical feature dashboard.
- [ ] Define an explicit workspace/view-state model for machine browser, software browser, settings, diagnostics, and contextual dialogs.
- [ ] Preserve backend readiness/error handling in the new shell.
- [ ] Preserve the centralized valid Tauri event-name contract.
- [ ] Add component-level tests for initial shell/readiness states.
- [ ] Keep the application continuously runnable while the legacy dashboard is still available during migration.

## MUI-003 — Reproduce the MAME left filter panel

- [ ] Implement a persistent left-side filter/category panel in the primary workspace.
- [ ] Map existing Tauri query fields to the canonical MAME filter inventory from MUI-001.
- [ ] Extend Rust/library query contracts for required filter dimensions that are not currently expressible.
- [ ] Implement the complete-list/default filter.
- [ ] Implement availability-related filters required by the parity matrix.
- [ ] Integrate Favorites as a MAME-style filter rather than a separate primary shelf.
- [ ] Implement remaining accepted MAME classification filters in prioritized batches.
- [ ] Make filter selection keyboard navigable.
- [ ] Preserve selected machine when it remains in the filtered result.
- [ ] Move selection predictably when filtering removes the selected machine.
- [ ] Persist the last-used filter through project-owned settings/state where appropriate.
- [ ] Add tests for filter switching, selection stability, empty filters, and backend failures.

## MUI-004 — Replace form-style search with MAME-style live search

- [ ] Keep search visible in the main machine-selection workspace.
- [ ] Replace explicit form-submit-only behavior with bounded/debounced live querying.
- [ ] Preserve `/` as the search-focus shortcut when gameplay input is not owned by MAME.
- [ ] Define and implement Escape/clear-search behavior.
- [ ] Match MAME searched fields where imported metadata supports them.
- [ ] Ensure stale asynchronous results cannot overwrite a newer query/selection.
- [ ] Preserve deterministic keyboard selection while results change.
- [ ] Add tests for search focus, query transitions, empty results, and race/stale-result behavior.

## MUI-005 — Build the dense central machine list

- [ ] Replace card/panel-oriented machine presentation with a dense selectable list.
- [ ] Make machine description/title the primary row label.
- [ ] Expose short name and other compact MAME-identifying metadata where useful.
- [ ] Represent availability/audit/status compactly without turning rows into cards.
- [ ] Show favorite state in-row or through an adjacent compact indicator/action.
- [ ] Preserve Up/Down/Home/End keyboard navigation.
- [ ] Add Page Up/Page Down navigation if compatible with the selected list implementation.
- [ ] Ensure selected row remains visible while navigating.
- [ ] Define Enter/Select behavior according to the parity matrix.
- [ ] Preserve editable-field protection for shortcuts.
- [ ] Preserve fail-closed shortcut suppression while native MAME owns gameplay input.
- [ ] Keep full-catalog list rendering within the existing interactive performance budget.
- [ ] Add virtualization only if required, with accessible focus/selection semantics.
- [ ] Add tests covering keyboard navigation and large-list behavior.

## MUI-006 — Reproduce the right Images/Info panel

- [ ] Add a right-side panel tied to the current machine/software selection.
- [ ] Implement explicit `Images` and `Info` modes.
- [ ] Persist the chosen right-panel mode where appropriate.
- [ ] Implement selected-machine artwork display using the existing safe artwork-serving backend.
- [ ] Inventory current project artwork categories against MAME's reference artwork categories.
- [ ] Add backend/config support for accepted missing artwork categories where justified.
- [ ] Implement artwork-category switching.
- [ ] Implement a non-error missing-artwork state.
- [ ] Preserve parent/clone artwork fallback only where it is safe and consistent with project policy.
- [ ] Implement compact machine info including description, short name, year, manufacturer, status, and available relationship metadata.
- [ ] Include audit/availability summary in the Info mode without making it a separate dashboard.
- [ ] Make detail/artwork requests stale-result-safe during rapid list navigation.
- [ ] Add tests for mode switching, category switching, missing artwork, and selection races.

## MUI-007 — Reproduce toolbar and contextual machine actions

- [ ] Add a MAME-style toolbar/context-action area associated with the selected item.
- [ ] Implement favorite toggle through the existing typed backend.
- [ ] Implement selected-machine audit action through the existing audit backend.
- [ ] Implement machine-information/details action.
- [ ] Implement machine-configuration action.
- [ ] Implement software-browser action when the machine exposes software lists.
- [ ] Implement Start/Launch as the visually dominant selected-machine action.
- [ ] Determine parity disposition for MAME's export-displayed-list action.
- [ ] Determine parity disposition for DAT/information actions beyond existing metadata.
- [ ] Give icon-only actions accessible names/tooltips.
- [ ] Implement keyboard equivalents where MAME or the existing project interaction model provides them.
- [ ] Add tests for action availability by selection/session state.

## MUI-008 — Reproduce software selection as a contextual browser

- [ ] Move software browsing out of the permanent nested machine panel layout.
- [ ] Open software selection contextually from the selected machine.
- [ ] Preserve the machine identity while browsing software.
- [ ] Reproduce accepted MAME software search/filter semantics.
- [ ] Use a dense selectable software list with keyboard navigation.
- [ ] Drive the right Images/Info panel from selected software where metadata/artwork is available.
- [ ] Launch selected software through the existing typed Rust launch path.
- [ ] Restore prior machine selection/view state when leaving software browsing.
- [ ] Implement or explicitly defer `Start empty` behavior based on the MUI-001 parity decision.
- [ ] Implement or explicitly defer multipart software selection based on the MUI-001 parity decision.
- [ ] Implement or explicitly defer BIOS-selection flow based on the MUI-001 parity decision.
- [ ] Add software-browser transition/navigation tests.

## MUI-009 — Move machine and controller settings into contextual configuration

- [ ] Replace permanent `MachineSettingsPanel` placement with a selected-machine configuration dialog/drawer/subview.
- [ ] Preserve every existing machine launch-setting capability.
- [ ] Expose effective-vs-overridden settings clearly.
- [ ] Move controller/profile configuration into an appropriate contextual configuration surface.
- [ ] Distinguish global controller profile selection from per-machine behavior where supported.
- [ ] Ensure configuration closes/back-navigates to the same machine selection.
- [ ] Add focus restoration and keyboard navigation tests for configuration surfaces.

## MUI-010 — Integrate launch/session controls without recreating the dashboard

- [ ] Keep Start/Launch in the selected-machine/software context.
- [ ] Surface launch-in-progress and launch-failure states without losing browser context.
- [ ] Move active-session status into the shell/status area.
- [ ] Move Pause/Resume/Soft reset/Mute/Unmute/Refresh/Stop into a contextual active-session surface.
- [ ] Preserve the existing authenticated runtime-control backend.
- [ ] Preserve event-driven session lifecycle updates using Tauri-valid centralized event names.
- [ ] Preserve fail-closed gameplay-input ownership behavior.
- [ ] Restore application shortcut ownership immediately on terminal session events.
- [ ] Add session-context tests for launch, running, stopping, exited, failed, and crashed states.

## MUI-011 — Integrate save states contextually

- [ ] Remove permanent save-state browser placement from the primary machine workspace.
- [ ] Expose stored save states through the selected machine and/or active-session context.
- [ ] Preserve application-managed save-state listing, save, load, delete, and compatibility checks.
- [ ] Preserve the structural compatibility probe before load.
- [ ] Make incompatible/stale states clearly distinguishable without overwhelming the main browser.
- [ ] Return to the same machine/session context after save-state operations.
- [ ] Add contextual save-state UI tests.

## MUI-012 — Rework keyboard and controller navigation for MAME parity

- [ ] Document the final focus/navigation graph across filters, list, right panel, toolbar, and dialogs.
- [ ] Implement predictable Left/Right transitions between primary regions where appropriate.
- [ ] Preserve Up/Down/Home/End machine-list navigation.
- [ ] Implement Page Up/Page Down if accepted in MUI-005.
- [ ] Preserve `/` search focus.
- [ ] Implement Enter/Select primary action consistently.
- [ ] Implement Escape/back semantics for software views, dialogs, and secondary surfaces.
- [ ] Preserve editable-input shortcut protection.
- [ ] Evaluate safe controller/gamepad navigation while the frontend owns input.
- [ ] Explicitly prevent frontend controller shortcuts from interfering with running MAME gameplay input.
- [ ] Add unit tests for focus graph and shortcut resolution.

## MUI-013 — Reproduce MAME-oriented visual design and responsive behavior

- [ ] Replace dashboard-style oversized cards/headings with dense desktop UI styling.
- [ ] Make the center machine/software list visually dominant.
- [ ] Make filter and Images/Info panels visually subordinate but persistent at normal desktop widths.
- [ ] Establish selected-row, hover, focus, unavailable, imperfect, and favorite visual states.
- [ ] Ensure status meaning is not color-only.
- [ ] Define desktop minimum/target sizing and behavior under resize.
- [ ] Add a narrow-window strategy that preserves the list and exposes the right panel through an explicit control when necessary.
- [ ] Verify common display scaling behavior.
- [ ] Respect reduced-motion preferences if transitions are used.
- [ ] Capture representative visual reference states for machine list, Images mode, Info mode, software browser, settings, and active session.
- [ ] Add automated screenshot/layout regression only if the mechanism is deterministic and maintainable in CI.

## MUI-014 — Move project extensions into secondary/contextual surfaces

- [ ] Replace the permanent `FavoriteShelf` role with Favorites filter/action integration before retiring the shelf from the main workspace.
- [ ] Move Collections into a secondary library/filter management surface.
- [ ] Move Recent History into a secondary view/filter/menu surface.
- [ ] Move Bulk Audit into a toolbar/menu operation with progress/status feedback.
- [ ] Move General Settings into a global Settings surface reachable from the shell.
- [ ] Move Diagnostics into a dedicated troubleshooting surface.
- [ ] Preserve all existing backend capabilities while changing presentation.
- [ ] Add navigation coverage proving each retained project extension remains reachable.

## MUI-015 — Close backend parity gaps discovered during reproduction

- [ ] Review the MUI-001 parity matrix after the first functional browser is in place.
- [ ] Add only the typed Rust/query capabilities required for accepted MAME parity gaps.
- [ ] Extend machine metadata/query payloads for required filter/info fields without bloating list payloads unnecessarily.
- [ ] Add persisted view-state fields through project-owned settings where required.
- [ ] Extend artwork configuration safely for accepted categories.
- [ ] Extend software query metadata where required.
- [ ] Implement accepted export/DAT capabilities or explicitly defer them with rationale.
- [ ] Keep all privileged filesystem/process operations in Rust.
- [ ] Do not add generic shell, unrestricted filesystem, opener, or arbitrary HTTP capabilities.
- [ ] Add Rust tests for every new backend contract.

## MUI-016 — Retire the legacy dashboard composition

- [ ] Build a feature-parity checklist for every component currently rendered directly by `App.tsx`.
- [ ] Verify each capability has a replacement navigation path in the MAME-style shell.
- [ ] Remove direct main-workspace rendering of `GeneralSettingsPanel`.
- [ ] Remove direct main-workspace rendering of `DiagnosticsPanel`.
- [ ] Remove direct main-workspace rendering of `SessionControlPanel`.
- [ ] Remove direct main-workspace rendering of `BulkAuditPanel`.
- [ ] Remove the legacy `LibraryBrowser` composition only after its browser responsibilities have moved into the new architecture.
- [ ] Remove direct main-workspace rendering of `RecentHistoryPanel`.
- [ ] Remove direct main-workspace rendering of `CollectionManager`.
- [ ] Remove or repurpose obsolete CSS tied solely to the vertical dashboard layout.
- [ ] Delete obsolete frontend components only after confirming no functionality is lost.
- [ ] Keep migration commits reviewable; avoid one giant deletion/rewrite commit if smaller qualified slices are possible.

## MUI-017 — Add architecture, behavior, and runtime regressions

- [ ] Add `scripts/tauri/test-mame-ui-reproduction.py` or equivalent milestone regression.
- [ ] Verify required shell/browser components exist.
- [ ] Verify `App.tsx` no longer contains the legacy permanent panel stack after MUI-016 closes.
- [ ] Verify Settings and Diagnostics are secondary surfaces.
- [ ] Verify the central event-name contract remains Rust/TypeScript-parity and Tauri-valid.
- [ ] Verify this spec and TODO are included in CI sparse checkout if the regression reads them.
- [ ] Wire the milestone regression into `Tauri project` CI.
- [ ] Preserve existing post-closeout/security regressions.
- [ ] Preserve frontend format/lint/typecheck/test/build gates.
- [ ] Preserve Rust format/test/Clippy gates.
- [ ] Add component tests for machine browser, filters, search, right panel, software browser, and contextual actions.
- [ ] Strengthen `tauri/scripts/smoke-dev.sh` or companion runtime smoke to verify the primary browser renders and the backend remains alive.
- [ ] Preserve full-catalog performance qualification.

## MUI-018 — Accessibility and usability qualification

- [ ] Audit semantic roles for filter list, machine list, toolbar, right panel, dialogs, and software browser.
- [ ] Verify visible focus for all keyboard-reachable controls.
- [ ] Verify icon actions have accessible names.
- [ ] Verify dialogs trap focus and restore it to the originating machine/action.
- [ ] Verify live-region usage does not produce excessive announcements during rapid selection movement.
- [ ] Verify status is not color-only.
- [ ] Verify core machine selection and launch can be completed without a mouse.
- [ ] Verify pointer operation remains straightforward and does not fight keyboard selection.

## MUI-019 — Documentation and product-description update

- [ ] Update `README.md` to describe the MAME-style selection UI once the new shell is the default.
- [ ] Add representative screenshots after visual layout stabilizes.
- [ ] Document new frontend directory/component structure.
- [ ] Document deliberate MAME parity gaps and their rationale.
- [ ] Preserve the known Tauri 2 Linux GLib advisory note unless the upstream dependency is genuinely removed.
- [ ] Update developer commands only if the workflow changes.
- [ ] Update any stale documentation that still describes the old vertical dashboard as the product UI.

## MUI-020 — Final qualification and milestone closure

- [ ] Reconcile every MUI task/subtask as complete, explicitly deferred with rationale, or superseded with rationale.
- [ ] Confirm no ambiguous unchecked item remains before closure.
- [ ] Run local regression/static checks where available.
- [ ] Qualify the exact final PR head through `Tauri project`.
- [ ] Qualify the exact final PR head through `Tauri security`.
- [ ] Qualify the exact final PR head through Linux packaging.
- [ ] Qualify the exact final PR head through Windows packaging.
- [ ] Qualify the exact final PR head through macOS packaging.
- [ ] Qualify documentation workflow where triggered.
- [ ] Confirm the hardened Linux real-window dev smoke proves the process remains alive after startup.
- [ ] Confirm full-catalog performance qualification remains within accepted budgets.
- [ ] Perform a manual UX pass comparing the final Tauri UI against the canonical MAME reference states identified in MUI-001/MUI-013.
- [ ] Merge only an exact-head-qualified candidate.
- [ ] Verify the promoted `master` SHA and applicable post-merge CI before claiming closure.
- [ ] Record final closure evidence and any intentionally deferred parity gaps.

## Completion rule

This TODO is not complete merely because the application looks more like MAME. Closure requires structural and behavioral reproduction of the main machine/software selection workflow, preservation of the existing backend/security boundaries, migration of all current project capabilities to sensible contextual/secondary surfaces, regression coverage, exact-head qualification, and explicit disposition of every parity gap identified in MUI-001.
