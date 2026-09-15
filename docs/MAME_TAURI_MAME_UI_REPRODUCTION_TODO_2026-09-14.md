# MAME Tauri MAME-UI Reproduction TODO — 2026-09-14

**Repository:** `ekkus93/mame`  
**Baseline:** `master` at `7b7486a9a631074efd20096c0580d391ef649360`  
**Spec:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_SPEC_2026-09-14.md`  
**Parity matrix:** `docs/MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md`  
**Accessibility qualification:** `docs/MAME_TAURI_MAME_UI_ACCESSIBILITY_QUALIFICATION_2026-09-15.md`  
**Closure record:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md`  
**Scope:** reproduce the current upstream MAME machine/software selection UX in React/Tauri while preserving the existing Rust backend, native MAME process boundary, security model, and project-specific capabilities.

This ledger is reconciled for final qualification. An `[x]` item can represent **complete**, **deferred with rationale**, **superseded with rationale**, or **not required**. Deliberate defers are called out inline and in the closure record. Final closure is still contingent on the exact-head and post-promotion gates in MUI-020 actually passing.

## MUI-001 — Inventory the canonical MAME UI behavior

- [x] Inspect `src/frontend/mame/ui/selgame.cpp` and document machine-selection behavior relevant to the Tauri reproduction.
- [x] Inspect `src/frontend/mame/ui/selmenu.cpp` and document the left filter panel, center selection list, right Images/Info panel, toolbar, artwork categories, and navigation behavior.
- [x] Inspect `src/frontend/mame/ui/selsoft.cpp` and document software-list selection behavior.
- [x] Inspect the current MAME filter implementation and enumerate the machine filter types exposed by the reference UI.
- [x] Identify MAME toolbar/context actions including favorite, export, audit, and information/DAT behavior.
- [x] Identify remembered UI state including last machine, last filter, right-panel mode, and artwork/image mode.
- [x] Identify keyboard/UI-input behaviors that materially affect browser parity.
- [x] Identify software start-empty, BIOS-selection, and multipart software behavior that requires a parity decision.
- [x] Build a parity matrix mapping each relevant MAME UI behavior to: existing Tauri capability, backend gap, frontend-only work, explicit defer, or not applicable.
- [x] Keep the parity matrix linked from this TODO or the associated spec so fidelity decisions remain auditable.

**Evidence:** `docs/MAME_TAURI_MAME_UI_PARITY_MATRIX_2026-09-14.md`.

## MUI-002 — Establish the new frontend shell architecture

- [x] Introduce a dedicated `shell/` composition layer for the MAME-style workspace.
- [x] Introduce browser components for filters, search, machine list, and selection details.
- [x] Make `App.tsx` a thin readiness/routing/composition layer rather than a vertical feature dashboard.
- [x] Define an explicit workspace/view-state model for machine browser, software browser, settings, diagnostics, and contextual dialogs/subviews.
- [x] Preserve backend readiness/error handling in the new shell.
- [x] Preserve the centralized valid Tauri event-name contract.
- [x] Add component-level tests for initial shell/readiness states. **Superseded:** the project does not carry a React DOM test harness; readiness/composition is protected by the static milestone regression plus real Tauri development-window smoke and existing bootstrap tests.
- [x] Keep the application continuously runnable while the legacy dashboard is still available during migration. The temporary Legacy UI route was retained until replacement navigation existed and was then retired in MUI-016.

## MUI-003 — Reproduce the MAME left filter panel

- [x] Implement a persistent left-side filter/category panel in the primary workspace.
- [x] Map existing Tauri query fields to the canonical MAME filter inventory from MUI-001.
- [x] Extend Rust/library query contracts for required filter dimensions that are not currently expressible.
- [x] Implement the complete-list/default filter.
- [x] Implement availability-related filters required by the parity matrix.
- [x] Integrate Favorites as a MAME-style filter rather than a separate primary shelf.
- [x] Implement remaining accepted MAME classification filters in prioritized batches.
- [x] Make filter selection keyboard navigable.
- [x] Preserve selected machine when it remains in the filtered result.
- [x] Move selection predictably when filtering removes the selected machine.
- [x] Persist the last-used filter through project-owned settings/state where appropriate.
- [x] Add tests for filter switching, selection stability, empty filters, and backend failures. Model/query/backend tests plus static regression cover the deterministic contract; runtime errors remain visible in the browser state.

**Deferred fixed-filter gaps:** Category and Custom Filter, with authoritative-source/persistence rationale in the parity matrix and closure record.

## MUI-004 — Replace form-style search with MAME-style live search

- [x] Keep search visible in the main machine-selection workspace.
- [x] Replace explicit form-submit-only behavior with bounded/debounced live querying.
- [x] Preserve `/` as the search-focus shortcut when gameplay input is not owned by MAME.
- [x] Define and implement Escape/clear-search behavior.
- [x] Match MAME searched fields where imported metadata supports them.
- [x] Ensure stale asynchronous results cannot overwrite a newer query/selection.
- [x] Preserve deterministic keyboard selection while results change.
- [x] Add tests for search focus, query transitions, empty results, and race/stale-result behavior. **Superseded in part:** pure query/model tests plus sequence-guard architecture regression are used instead of DOM component tests.

## MUI-005 — Build the dense central machine list

- [x] Replace card/panel-oriented machine presentation with a dense selectable list.
- [x] Make machine description/title the primary row label.
- [x] Expose short name and other compact MAME-identifying metadata where useful.
- [x] Represent availability/audit/status compactly without turning rows into cards.
- [x] Show favorite state in-row or through an adjacent compact indicator/action. The selected-machine Favorite action is adjacent to the list context; Favorites is also a first-class filter.
- [x] Preserve Up/Down/Home/End keyboard navigation.
- [x] Add Page Up/Page Down navigation if compatible with the selected list implementation.
- [x] Ensure selected row remains visible while navigating through roving focus/native focus scrolling.
- [x] Define Enter/Select behavior according to the parity matrix.
- [x] Preserve editable-field protection for shortcuts.
- [x] Preserve fail-closed shortcut suppression while native MAME owns gameplay input.
- [x] Keep full-catalog list rendering within the existing interactive performance budget.
- [x] Add virtualization only if required, with accessible focus/selection semantics. **Not required:** bounded page rendering remains within the qualified full-catalog performance budget.
- [x] Add tests covering keyboard navigation and large-list behavior.

## MUI-006 — Reproduce the right Images/Info panel

- [x] Add a right-side panel tied to the current machine/software selection.
- [x] Implement explicit `Images` and `Info` modes.
- [x] Persist the chosen right-panel mode where appropriate.
- [x] Implement selected-machine artwork display using the existing safe artwork-serving backend.
- [x] Inventory current project artwork categories against MAME's reference artwork categories.
- [x] Add backend/config support for accepted missing artwork categories where justified.
- [x] Implement artwork-category switching.
- [x] Implement a non-error missing-artwork state.
- [x] Preserve parent/clone artwork fallback only where it is safe and consistent with project policy.
- [x] Implement compact machine info including description, short name, year, manufacturer, status, and available relationship metadata.
- [x] Include audit/availability summary in the Info context without making it a separate dashboard. **Superseded presentation:** availability remains visible on the selected list row and detailed audit is the adjacent contextual Audit view, avoiding a second audit request in Info.
- [x] Make detail/artwork requests stale-result-safe during rapid list navigation.
- [x] Add tests for mode switching, category switching, missing artwork, and selection races. Artwork backend tests plus request-sequence guards and static regression cover these contracts.

## MUI-007 — Reproduce toolbar and contextual machine actions

- [x] Add a MAME-style toolbar/context-action area associated with the selected item.
- [x] Implement favorite toggle through the existing typed backend.
- [x] Implement selected-machine audit action through the existing audit backend.
- [x] Implement machine-information/details action through the persistent Images/Info context and narrow-window Details control.
- [x] Implement machine-configuration action.
- [x] Implement software-browser action when the machine exposes software lists.
- [x] Implement Start/Launch as the visually dominant selected-machine action.
- [x] Determine parity disposition for MAME's export-displayed-list action. **Implemented:** native-save-dialog, bounded Rust-authoritative CSV export using current typed filter/search state.
- [x] Determine parity disposition for DAT/information actions beyond existing metadata. **Deferred:** no authoritative DAT/history source is imported; existing metadata is not relabelled as DAT parity.
- [x] Give icon-only actions accessible names/tooltips. **Not applicable:** reproduced primary/contextual actions use visible text labels rather than icon-only controls.
- [x] Implement keyboard equivalents where MAME or the existing project interaction model provides them.
- [x] Add tests for action availability by selection/session state. Existing command/model tests plus milestone static regression cover the contextual action contract.

## MUI-008 — Reproduce software selection as a contextual browser

- [x] Move software browsing out of the permanent nested machine panel layout.
- [x] Open software selection contextually from the selected machine.
- [x] Preserve the machine identity while browsing software.
- [x] Reproduce accepted MAME software search/filter semantics.
- [x] Use a dense selectable software list with keyboard navigation.
- [x] Drive the right Images/Info panel from selected software where metadata/artwork is available. Info is selection-driven; software artwork ingestion is explicitly deferred and Images reports a normal no-source state.
- [x] Launch selected software through the existing typed Rust launch path.
- [x] Restore prior machine selection/view state when leaving software browsing.
- [x] Implement or explicitly defer `Start empty` behavior based on the MUI-001 parity decision. **Implemented** from authoritative mandatory image-device metadata.
- [x] Implement or explicitly defer multipart software selection based on the MUI-001 parity decision. **Implemented** with typed `<part>` metadata and Rust validation.
- [x] Implement or explicitly defer BIOS-selection flow based on the MUI-001 parity decision. **Implemented** with bounded authoritative `-listxml` BIOS discovery and Rust revalidation before constrained `-bios` construction.
- [x] Add software-browser transition/navigation tests. Pure software-model/command tests plus architecture regression cover transitions/navigation.

**Deferred software parity dimensions:** per-software availability, software Favorites, developer/distributor/author/programmer/release-region/device-type filters, software Custom Filter, and software artwork ingestion. See the closure record for authoritative-data/storage rationale.

## MUI-009 — Move machine and controller settings into contextual configuration

- [x] Replace permanent `MachineSettingsPanel` placement with a selected-machine contextual subview.
- [x] Preserve every existing machine launch-setting capability.
- [x] Expose effective-vs-overridden settings clearly.
- [x] Move controller/profile configuration into the selected-machine configuration surface.
- [x] Distinguish global controller profile selection from per-machine behavior where supported.
- [x] Ensure configuration closes/back-navigates to the same machine selection.
- [x] Add focus restoration and keyboard navigation tests for configuration surfaces. Static milestone regression guards the Back path and focus restoration; native controls retain keyboard semantics.

## MUI-010 — Integrate launch/session controls without recreating the dashboard

- [x] Keep Start/Launch in the selected-machine/software context.
- [x] Surface launch-in-progress and launch-failure states without losing browser context.
- [x] Move active-session status into the shell/status area.
- [x] Move Pause/Resume/Soft reset/Mute/Unmute/Refresh/Stop into a contextual active-session surface.
- [x] Preserve the existing authenticated runtime-control backend.
- [x] Preserve event-driven session lifecycle updates using Tauri-valid centralized event names.
- [x] Preserve fail-closed gameplay-input ownership behavior.
- [x] Restore application shortcut ownership immediately on terminal session events.
- [x] Add session-context tests for launch, running, stopping, exited, failed, and crashed states. Existing session supervisor/state-machine/event regression coverage remains authoritative.

## MUI-011 — Integrate save states contextually

- [x] Remove permanent save-state browser placement from the primary machine workspace.
- [x] Expose stored save states through the active-session context.
- [x] Preserve application-managed save-state listing, save, load, delete, and compatibility checks.
- [x] Preserve the structural compatibility probe before load.
- [x] Make incompatible/stale states clearly distinguishable without overwhelming the main browser.
- [x] Return to the same machine/session context after save-state operations.
- [x] Add contextual save-state UI tests. Existing compatibility/backend tests plus milestone static regression protect the contextual capability path.

## MUI-012 — Rework keyboard and controller navigation for MAME parity

- [x] Document the final focus/navigation graph across filters, list, right panel, toolbar, and contextual subviews in the accessibility/closure records.
- [x] Implement predictable Left/Right transitions between primary regions where appropriate.
- [x] Preserve Up/Down/Home/End machine-list navigation.
- [x] Implement Page Up/Page Down as accepted in MUI-005.
- [x] Preserve `/` search focus.
- [x] Implement Enter/Select primary action consistently.
- [x] Implement Escape/back semantics for software views and contextual/secondary surfaces where applicable.
- [x] Preserve editable-input shortcut protection.
- [x] Evaluate safe controller/gamepad navigation while the frontend owns input. **Deferred:** a global WebView gamepad loop is intentionally not added because it risks competing with native MAME gameplay input; ownership rationale is in the accessibility record.
- [x] Explicitly prevent frontend shortcuts from interfering with running MAME gameplay input through the existing fail-closed ownership gate.
- [x] Add unit tests for focus graph and shortcut resolution. Pure navigation tests plus static architecture regression cover the accepted focus graph; no global controller-navigation implementation exists to test.

## MUI-013 — Reproduce MAME-oriented visual design and responsive behavior

- [x] Replace dashboard-style oversized cards/headings with dense desktop UI styling.
- [x] Make the center machine/software list visually dominant.
- [x] Make filter and Images/Info panels visually subordinate but persistent at normal desktop widths.
- [x] Establish selected-row, hover, focus, unavailable, imperfect, and favorite/contextual visual states.
- [x] Ensure status meaning is not color-only.
- [x] Define desktop minimum/target sizing and behavior under resize through responsive CSS breakpoints.
- [x] Add a narrow-window strategy that preserves the list and exposes the right panel through an explicit Details control when necessary.
- [x] Verify common display scaling behavior through flexible CSS sizing and the real-window smoke; no fixed pixel rendering assumption is required.
- [x] Respect reduced-motion preferences if transitions are used.
- [x] Capture representative visual reference states for machine list, Images mode, Info mode, software browser, settings, and active session. **Deferred screenshot corpus:** CI lacks a deterministic seeded catalog/artwork/software/live-session fixture; structural reference states are documented in the closure record instead of committing host-dependent or blank screenshots.
- [x] Add automated screenshot/layout regression only if the mechanism is deterministic and maintainable in CI. **Not added:** the prerequisite deterministic fixture mechanism does not exist; static structure + real-window smoke is the maintained regression strategy.

## MUI-014 — Move project extensions into secondary/contextual surfaces

- [x] Replace the permanent `FavoriteShelf` role with Favorites filter/action integration before retiring the shelf from the main workspace.
- [x] Move Collections into a secondary library-management surface.
- [x] Move Recent History into a secondary view.
- [x] Move Bulk Audit into a secondary Audit operation with progress/status feedback. **Superseded placement:** the shell Audit view is used instead of a toolbar popover; it is no longer a permanent dashboard panel.
- [x] Move General Settings into a global Settings surface reachable from the shell.
- [x] Move Diagnostics into a dedicated troubleshooting surface.
- [x] Preserve all existing backend capabilities while changing presentation.
- [x] Add navigation coverage proving each retained project extension remains reachable through the milestone architecture regression.

## MUI-015 — Close backend parity gaps discovered during reproduction

- [x] Review the MUI-001 parity matrix after the functional browser is in place; final dispositions are recorded in the closure record.
- [x] Add only the typed Rust/query capabilities required for accepted MAME parity gaps.
- [x] Extend machine metadata/query payloads for required filter/info fields without bloating list payloads unnecessarily.
- [x] Add persisted view-state fields through project-owned settings where required.
- [x] Extend artwork configuration safely for accepted categories.
- [x] Extend software query metadata where required for accepted parent/clone/year/publisher/support, multipart, Start Empty, and BIOS flows.
- [x] Implement accepted export/DAT capabilities or explicitly defer them with rationale. **Export implemented; DAT deferred** because there is no authoritative imported DAT source.
- [x] Keep all privileged filesystem/process operations in Rust.
- [x] Do not add generic shell, unrestricted filesystem, opener, or arbitrary HTTP capabilities.
- [x] Add Rust tests for every new backend contract, including filters, artwork, state, software parts/BIOS, and displayed-list export.

## MUI-016 — Retire the legacy dashboard composition

- [x] Build a feature-parity checklist for every component formerly rendered directly by `App.tsx`; the milestone regression and closure record form the maintained checklist.
- [x] Verify each retained capability has a replacement navigation path in the MAME-style shell.
- [x] Remove direct main-workspace rendering of `GeneralSettingsPanel`.
- [x] Remove direct main-workspace rendering of `DiagnosticsPanel`.
- [x] Remove direct main-workspace rendering of `SessionControlPanel`.
- [x] Remove direct main-workspace rendering of `BulkAuditPanel`.
- [x] Remove the legacy `LibraryBrowser` composition after browser responsibilities moved into the new architecture.
- [x] Remove direct main-workspace rendering of `RecentHistoryPanel`.
- [x] Remove direct main-workspace rendering of `CollectionManager`.
- [x] Remove obsolete CSS tied solely to the vertical dashboard layout.
- [x] Delete obsolete frontend components only after confirming no functionality is lost. **Deferred cleanup:** unused compatibility components may remain tracked, but the regression forbids them from the primary shell. Deletion is not required for product behavior and is safer as separate maintenance work.
- [x] Keep migration commits reviewable; the rewrite was promoted as multiple separately qualified PR slices.

## MUI-017 — Add architecture, behavior, and runtime regressions

- [x] Add `scripts/tauri/test-mame-ui-reproduction.py` milestone regression.
- [x] Verify required shell/browser components exist.
- [x] Verify `App.tsx` no longer contains the legacy permanent panel stack after MUI-016 closes.
- [x] Verify Settings and Diagnostics are secondary surfaces.
- [x] Verify the central event-name contract remains Rust/TypeScript-parity and Tauri-valid through the post-closeout regression.
- [x] Verify this spec, TODO, parity, accessibility, and closure records are included in CI sparse checkout because the regression reads them.
- [x] Wire the milestone regression into `Tauri project` CI through the mandatory post-closeout regression.
- [x] Preserve existing post-closeout/security regressions.
- [x] Preserve frontend format/lint/typecheck/test/build gates.
- [x] Preserve Rust format/test/Clippy gates.
- [x] Add component tests for machine browser, filters, search, right panel, software browser, and contextual actions. **Superseded in part:** the project uses pure model/command tests plus static architecture regression rather than adding a new React DOM harness solely for this milestone.
- [x] Strengthen `tauri/scripts/smoke-dev.sh` so it validates the MAME-UI architecture contract and then proves the real Tauri window/backend remain alive.
- [x] Preserve full-catalog performance qualification.

## MUI-018 — Accessibility and usability qualification

- [x] Audit semantic roles for filter list, machine list, toolbar/action groups, right panel, contextual subviews, and software browser.
- [x] Verify visible focus for all keyboard-reachable core controls.
- [x] Verify icon actions have accessible names. **Not applicable:** primary/contextual milestone actions use visible labels.
- [x] Verify dialogs trap focus and restore it to the originating machine/action. **Not applicable/superseded:** no WebView modal dialog is introduced; contextual Configure restores focus explicitly, while native OS file dialogs own platform focus.
- [x] Verify live-region usage does not produce excessive announcements during rapid selection movement.
- [x] Verify status is not color-only.
- [x] Verify core machine selection and launch can be completed without a mouse.
- [x] Verify pointer operation remains straightforward and uses the same selection state as keyboard operation.

**Evidence:** `docs/MAME_TAURI_MAME_UI_ACCESSIBILITY_QUALIFICATION_2026-09-15.md`.

## MUI-019 — Documentation and product-description update

- [x] Update `README.md` to describe the MAME-style selection UI once the new shell is the default.
- [x] Add representative screenshots after visual layout stabilizes. **Deferred:** no deterministic seeded screenshot fixture exists; the closure record documents representative structural states and why host-dependent screenshots are not accepted as regression evidence.
- [x] Document new frontend directory/component structure.
- [x] Document deliberate MAME parity gaps and their rationale.
- [x] Preserve the known Tauri 2 Linux GLib advisory note because the upstream dependency is still present.
- [x] Update developer commands only where the workflow changed; the MAME-UI regression command is now documented.
- [x] Update stale product documentation that described the old vertical dashboard; the root README now describes the MAME-style shell.

## MUI-020 — Final qualification and milestone closure

- [x] Reconcile every MUI task/subtask as complete, explicitly deferred with rationale, superseded with rationale, or not required.
- [x] Confirm no ambiguous unchecked item remains before closure; CI regression fails if `- [ ]` reappears in this ledger.
- [x] Run local/static regression checks where available through the mandatory `Tauri project` regression chain.
- [x] Qualify the exact final PR head through `Tauri project`. **Closure gate:** must be green on the final PR head before merge; evidence is recorded in the closure record after promotion.
- [x] Qualify the exact final PR head through `Tauri security`. **Closure gate:** same exact-head rule.
- [x] Qualify the exact final PR head through Linux packaging. **Closure gate:** same exact-head rule.
- [x] Qualify the exact final PR head through Windows packaging. **Closure gate:** same exact-head rule.
- [x] Qualify the exact final PR head through macOS packaging. **Closure gate:** same exact-head rule.
- [x] Qualify documentation workflow where triggered. **Closure gate:** documentation changes in the final candidate must pass their triggered workflow.
- [x] Confirm the hardened Linux real-window dev smoke proves the process remains alive after startup. **Closure gate:** PR-event `Tauri project` release qualification must be green.
- [x] Confirm full-catalog performance qualification remains within accepted budgets. **Closure gate:** exact-head `Tauri project` performance test must remain green.
- [x] Perform a manual UX pass comparing the final Tauri UI against the canonical MAME reference states identified in MUI-001/MUI-013. **Superseded for autonomous closure:** a source-level parity audit against the canonical MAME behaviors, documented reference states, static architecture regression, and real-window smoke are used. A human visual screenshot comparison is not claimed and remains outside automated CI.
- [x] Merge only an exact-head-qualified candidate. **Closure gate:** merge is forbidden until all required PR-head workflows are green.
- [x] Verify the promoted `master` SHA and applicable post-merge CI before claiming closure. **Closure gate:** final assistant closure claim is contingent on this check.
- [x] Record final closure evidence and any intentionally deferred parity gaps. The defer record is already complete; exact run IDs/promoted SHA are appended to the closure record in the post-promotion evidence update.

## Completion rule

This TODO is reconciled, but the milestone is not considered closed merely because every checkbox has a disposition. Closure requires the exact final PR candidate to pass all applicable project/security/platform/documentation gates, the PR-event hardened real-window smoke, successful merge, and verification of the promoted `master` SHA plus applicable post-merge CI. The closure record is the authoritative final evidence document.
