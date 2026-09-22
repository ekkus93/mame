# MAME Tauri UI Parity Review Remediation TODO — 2026-09-21

**Repository:** ekkus93/mame  
**Baseline master:** 3a299632cf41e4d9d313c5880708773cc241d186  
**Spec:** docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_SPEC_2026-09-21.md  
**Original parity TODO:** docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md  
**Goal:** remediate every issue identified by the post-closure code review and re-qualify original-MAME visual/interaction parity without weakening the existing Tauri architecture or launch/catalog correctness.

## Mandatory execution rules

- Use this file as the canonical remediation ledger.
- Do not redesign the application.
- Preserve the Tauri/WebView architecture.
- Preserve catalog authority, BIOS omission semantics, software-part semantics, Start Empty behavior, and gameplay-input ownership safeguards.
- Do not mark behavioral work complete using source-string assertions alone.
- Use executable component/state/unit/integration tests for behavioral claims wherever technically possible.
- Use exact-head CI evidence for every merge candidate.
- After every successful merge, reload this TODO from promoted master, identify the next unchecked item, and continue.
- Do not collapse or delete individual subtasks after completion. Check each subtask and append evidence.
- Do not claim final closure without rendered visual evidence.
- Screenshot pixel-baseline automation may remain deferred only under the documented WebView/tooling limitation; that defer does not waive manual/rendered review.
- A transient tool/API timeout, ordinary test failure, formatting failure, merge conflict, or CI still running is not a user blocker.

---

## MTR-000 — Reopen parity and establish exact baseline

- [x] Read docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_SPEC_2026-09-21.md completely.
- [x] Read docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md completely.
- [x] Read the detailed historical MTP checklist from repository history if current master contains the reconciled/condensed version.
- [x] Read docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md.
- [x] Read docs/MAME_TAURI_UI_PARITY_FINAL_REPORT_2026-09-21.md and identify claims reopened by this remediation.
- [x] Confirm current master SHA before implementation.
- [x] Record the current frontend test/CI baseline before behavior changes.
- [x] Record the exact files currently responsible for selection/detail lifecycle, activation, default shell/footer composition, Software Browser styling, right-panel keyboard behavior, artwork loading, and parity tests.
- [x] Do not modify the historical MTP evidence to hide the prior closure; this remediation must remain additive and auditable.

**Evidence:** Reconciled from promoted master `d11481284ce83724084a0ad874d0fb6dfc27fbae`. The remediation spec, original parity spec, reference, and final report were reread, and the detailed historical MTP checklist was read from pre-condense parent `9a7f1e7441f67709a3231289ed4517df151d2c87`. Promoted-master baseline CI was green: Tauri project `35730623025`, Windows packaging `35730622875`, macOS packaging `35730622891`, Linux packaging `35730622749`, and Tauri security `35730623007`. Responsibility files were recorded across `MameBrowser.tsx`, `machineAsyncIdentity.ts`, `model.ts`, `MachineList.tsx`, `machineListVisibility.ts`, `MameShell`, `MachineDriverStatus`, `SoftwareBrowser`, `MachineRightPanel`, `rightPanelKeyboard.ts`, `artworkState.ts`, `ArtworkAssetFrame.tsx`, and associated tests.

---

## MTR-001 — Fix selected-machine detail invalidation

- [x] Introduce a selection/detail generation, abort mechanism, or equivalent explicit invalidation model.
- [x] Invalidate outstanding detail work whenever selection changes from one machine to another.
- [x] Invalidate outstanding detail work whenever selection changes to null.
- [x] Ensure metadata refresh/import paths invalidate old detail.
- [x] Ensure catalog query failure/empty-result paths invalidate old detail.
- [x] Ensure value-required/deferred-filter transitions that clear selection invalidate old detail.
- [x] Prevent an older successful detail response from replacing a newer selected machine.
- [x] Prevent an older failed detail response from replacing a newer successful state.
- [x] Clear machine-specific pending launch overrides when machine identity changes or selection clears.
- [x] Clear/reset other machine-specific transient action state where required.
- [x] Add deterministic regression test: select A -> detail A pending -> clear selection -> resolve A -> no A detail/status/actions return.
- [x] Add deterministic regression test: select A -> detail A pending -> select B -> resolve B -> resolve A -> B remains authoritative.
- [x] Add deterministic regression test: select A -> detail A pending -> select B -> resolve A -> resolve B -> A is never exposed as B's state.
- [x] Verify right-panel, driver/status, BIOS options, launch actions, software actions, configure action, and audit action all obey current selection identity.
- [x] Run applicable frontend/component tests on exact head.

**Evidence:** Implemented by PR #74 candidate `a6b7f7495a76ce7f7e00b75e07d71be680f2cbca` and promoted as `d11481284ce83724084a0ad874d0fb6dfc27fbae`. `machineAsyncIdentity.ts` adds explicit detail/activation generations, `MameBrowser.tsx` routes selection through identity-aware `selectMachine`, and `machineSelectionIdentity.test.ts` covers stale detail clearing, out-of-order A/B completions, and activation invalidation. Exact candidate passed PR workflows `35729905106`, `35729905090`, `35729905021`, `35729905031`, `35729905103` and push workflows `35725817004`, `35725817012`, `35725817047`, `35725817011`, `35725817089`; promoted master passed `35730623025`, `35730622875`, `35730622891`, `35730622749`, `35730623007`.

---

## MTR-002 — Fix double-click and Enter activation races

- [x] Define the authoritative activation contract for a machine whose detail is not yet loaded.
- [x] Ensure first-selection plus immediate double-click activates that clicked machine exactly once.
- [x] Ensure keyboard selection plus immediate Enter activates that selected machine exactly once.
- [x] Do not silently drop activation merely because detail is loading.
- [x] Ensure activation cannot accidentally use detail from the previously selected machine.
- [x] If activation queues behind detail loading, invalidate the queued activation when its machine identity is no longer current unless the launch has already crossed an explicitly documented authoritative boundary.
- [x] Preserve machine availability guards.
- [x] Preserve BIOS omission unless explicitly selected.
- [x] Preserve Start Empty semantics.
- [x] Preserve software-list/software-part behavior.
- [x] Preserve visible launch-error handling inside the MAME surface.
- [x] Add component/state regression test for rapid double-click.
- [x] Add component/state regression test for immediate Enter after keyboard movement.
- [x] Add regression test covering rapid A activation followed by B selection before A detail completion.
- [x] Run applicable frontend/component tests on exact head.

**Evidence:** Implemented through PR #76 candidate `90505ed09c26c4eec982271e0b34a2c687816bf9`. Activation fetches matching detail when needed and commits only while `activationMayCommit(...)` proves the selected machine/detail/activation generations are current. Tests in `machineSelectionIdentity.test.ts` cover rapid double-click, immediate Enter, and invalidation after selecting another machine. Candidate passed PR workflows `35744067325`, `35744067306`, `35744067377`, `35744067314`, `35744067324` and push workflows `35744059478`, `35744059523`, `35744059446`, `35744059486`, `35744059549`; promoted as `2388e2ad752e3f2be707aa0b73d94dc9b5009a81`.

---

## MTR-003 — Remove generic Tauri branding from default UI

- [x] Remove the default-visible MAME Tauri Frontend pseudo-title from MameShell.css or equivalent.
- [x] Audit default startup/browser surfaces for equivalent generic Tauri rewrite branding.
- [x] Keep implementation/debug branding only in non-default developer/debug contexts if still needed.
- [x] Add a negative static/component regression guard that the prohibited default-visible phrase is absent.
- [x] Render/inspect the default browser and confirm original-like title/search treatment remains intact.
- [x] Confirm removing the pseudo-title does not introduce unwanted vertical whitespace or break density.
- [x] Run applicable frontend tests on exact head.

**Evidence:** PR #80 removed the default-visible pseudo-title row and PR #81 candidate `d409983dbe9352832db055e73c47cde38c728fa3` added `mameShellComposition.source.test.ts` guards proving the prohibited phrase is absent from default browser/shell sources. Exact PR #81 candidate passed PR workflows `35750198886`, `35750198822`, `35750199031`, `35750198809`, `35750198826` and push workflows `35750170743`, `35750170788`, `35750170767`, `35750170817`, `35750170828`; promoted as `2c373015e32744896f74278b9d4f7df478d863de`. Rendered/title/search/density inspection is reconciled by MTR-013 in `docs/MAME_TAURI_MTR_013_RENDERED_VISUAL_REVIEW_2026-09-22.md`, which records the textual rendered review and Tauri/WebView smoke evidence without claiming pixel evidence.

---

## MTR-004 — Make green driver/status region the true bottom region

- [x] Remove the permanent project/global utility footer from below MachineDriverStatus in the default browser composition.
- [x] Ensure MachineDriverStatus is the bottom-most persistent region in default machine-browser mode.
- [x] Preserve Configure Options access without restoring a generic dashboard/tab row.
- [x] Preserve Audit access.
- [x] Preserve History access.
- [x] Preserve Collections access.
- [x] Preserve Diagnostics access.
- [x] Preserve Session or session-status access where required.
- [x] Move secondary/global tools behind an original-compatible secondary affordance, compact menu, command flow, or non-default surface.
- [x] Keep backend/global diagnostics out of the permanent green machine driver/status region.
- [x] Preserve selected-machine metadata in the green region.
- [x] Preserve useful no-selection/loading/error status in the green region.
- [x] Add a component structure test proving no persistent footer exists below the green driver/status region.
- [x] Add a test proving History/Collections/Diagnostics are not permanently visible in default browser chrome.
- [x] Add tests proving required secondary surfaces remain reachable.
- [x] Perform rendered inspection of the full-height default shell.
- [x] Run applicable frontend tests on exact head.

**Evidence:** Implementation promoted via PR #60 as `2d8fbf30d45a597d7646624fbf0321d05109c449`; candidate `4252e1f3b640318275bee73a4fb7a6c25a2167ea` passed Tauri project `35674026992`, Windows `35674027017`, macOS `35674026990`, Linux `35674027023`, and security `35674026976`. Default footer was removed, `MachineDriverStatus` is the final persistent browser region, and secondary tools remain reachable through the compact `More` utility menu/contextual actions. Rendered full-height inspection is reconciled by the MTR-013 rendered review document and Tauri/WebView smoke evidence.

---

## MTR-005 — Complete explicit MAME palette migration

- [x] Audit parity-relevant CSS/TSX for Canvas.
- [x] Audit parity-relevant CSS/TSX for CanvasText.
- [x] Audit parity-relevant CSS/TSX for product uses of currentColor.
- [x] Audit parity-relevant CSS/TSX for unintentional generic white/gray surfaces.
- [x] Include App startup/connect state in the audit.
- [x] Include MameShell/MameBrowser and all default machine-browser components.
- [x] Include SoftwareBrowser.
- [x] Include General Settings/configuration surfaces reachable from the parity flow.
- [x] Replace visible product system-color styling with semantic --mame-* tokens.
- [x] Add tokens where necessary instead of proliferating unrelated literal colors.
- [x] Ensure hover states are intentionally themed.
- [x] Ensure selected states are intentionally themed.
- [x] Ensure focus-visible states retain strong contrast.
- [x] Ensure disabled states are intentionally themed.
- [x] Ensure error/loading/empty states are intentionally themed.
- [x] Extend static theme tests to scan all parity-relevant stylesheets rather than only MameShell.css.
- [x] Add a negative guard for disallowed host-system product styling in parity-relevant files.
- [x] Verify under host light and dark preference where practical that the product palette remains MAME-like.
- [x] Run applicable frontend tests on exact head.

**Evidence:** Implemented in PR #63. Parity audit covered startup/connect, default browser surfaces, Software Browser, and configuration surfaces; visible host-system palette dependencies were replaced with semantic MAME tokens. Exact candidate `eabfb6f48b4b72b3df55bb3bbb8bc9afc960ea24` passed all five push workflows and all five PR workflows.

---

## MTR-006 — Bring Software Browser to MAME visual/interaction parity

- [x] Apply MAME navy/background tokens to Software Browser.
- [x] Apply original-like compact toolbar/header treatment.
- [x] Apply dense software row/list spacing.
- [x] Apply explicit selected-row blue treatment.
- [x] Apply selected primary text treatment consistent with the MAME visual contract.
- [x] Apply muted unavailable/disabled treatment where applicable.
- [x] Apply visible focus treatment.
- [x] Apply thin splitter/border treatment using MAME tokens.
- [x] Remove system-color product styling.
- [x] Keep software errors inside the MAME surface.
- [x] Preserve software-part selection.
- [x] Preserve Start behavior.
- [x] Preserve BIOS semantics.
- [x] Preserve Back/Escape behavior.
- [x] Add component tests for software selection.
- [x] Add component tests for software activation.
- [x] Add theme regression tests covering Software Browser.
- [x] Perform rendered inspection of Software List with representative states.
- [x] Run applicable frontend tests on exact head.

**Evidence:** MTR-006 was promoted through PR #86 and PR #87. PR #86 candidate `bc6b420b9e194a80a12653bdfce22eaf3da05333` moved Software Browser styling to explicit MAME tokens and dense selected/muted/focus treatment. PR #87 candidate `2a04f78eb39c33aaf0942dcd94de15b57e3ce686` added `SoftwareBrowserComponent.test.tsx` server-rendered row coverage for selected ARIA/class state, muted support classes, metadata output, callbacks, and Start disabling. Exact candidate passed push workflows `35762584473`, `35762584480`, `35762584513`, `35762584530`, `35762584472` and PR workflows `35763249390`, `35763249337`, `35763249676`, `35763249324`, `35763249333`. Rendered Software List review is reconciled by MTR-013.

---

## MTR-007 — Fix selected-row visibility and scroll reconciliation

- [x] Define documented scroll behavior for keyboard movement, query/filter changes, search changes, and result replacement.
- [x] Ensure an asynchronously selected row is visible after a search result update.
- [x] Ensure an asynchronously selected row is visible after a filter result update.
- [x] Avoid unnecessary scroll jumps when the selected row is already visible.
- [x] Preserve normal keyboard-driven scrolling.
- [x] Ensure empty results clear selection and machine-specific detail/action state.
- [x] Ensure restored/persisted selection is reconciled safely against the current result set.
- [x] Use a deterministic helper for visibility/page calculations if DOM viewport behavior is difficult to test directly.
- [x] Add a long-list regression test where the list begins deeply scrolled and a new query selects a result near the beginning.
- [x] Add regression test for a new selected result near the end of a long list.
- [x] Add regression test for no-results transition.
- [x] Run applicable frontend/component tests on exact head.

**Evidence:** Promoted through PR #89. Candidate `9799ab0a529ae241d9d22ea349393975f08b3c1a` added `machineListVisibility.ts` and `MachineList.visibility.test.ts` coverage for long-list replacement near beginning/end, no-results selection clearing, absent identity, partially hidden rows, and no-scroll for already-visible rows. Exact candidate passed push workflows `35769741029`, `35769741045`, `35769741017`, `35769740865`, `35769741091` and PR workflows `35773875580`, `35773875456`, `35773875498`, `35773875405`, `35773875514`; promoted as `559c2553cdf8815f04b5f2040f12dd930aceb58d`.

---

## MTR-008 — Correct keyboard page/home/end semantics

- [x] Remove the arbitrary fixed ten-row PageUp/PageDown behavior unless ten is demonstrably the computed visible page size.
- [x] Define viewport-aware PageUp/PageDown behavior for the machine list.
- [x] Implement viewport-aware movement or an equivalent deterministic page-size calculation.
- [x] Define Home behavior for the paged catalog model.
- [x] Define End behavior for the paged catalog model.
- [x] Decide explicitly whether Home/End target the current fetched page or the full matching catalog; document the choice and rationale.
- [x] If full-result Home/End is feasible, implement it without breaking catalog pagination.
- [x] If current-page semantics are retained, ensure the UI/test/docs do not falsely claim full-result semantics.
- [x] Preserve Up/Down behavior.
- [x] Preserve Enter activation after MTR-002.
- [x] Preserve Escape/back behavior.
- [x] Add pure/model tests for page-size calculation.
- [x] Add component/integration tests for PageUp/PageDown.
- [x] Add tests for documented Home/End behavior.
- [x] Ensure shortcuts remain gameplay-input-owner gated.
- [x] Run applicable frontend tests on exact head.

**Evidence:** Promoted through PR #90. Candidate `888d4d4a5618420226c396b7996b1a5dbdf18e2b` replaced fixed ten-row page movement with live viewport/row-height page movement retaining one row of context, and documented current-fetched-page Home/End semantics in `model.ts`. Tests cover page-size calculation, PageUp/PageDown integration, and Home/End behavior. Exact candidate passed push workflows `35780978021`, `35780978108`, `35780977936`, `35780977960`, `35780977948` and PR workflows `35781875893`, `35781875964`, `35781875784`, `35781875965`, `35781875966`; promoted as `dd1cbad6229939f2000099a51e017c47684c2b95`.

---

## MTR-009 — Complete right-panel keyboard semantics

- [x] Treat Images/Infos as a proper keyboard-navigable tablist.
- [x] ArrowRight moves from Images to Infos.
- [x] ArrowLeft moves from Infos to Images.
- [x] Define wrap or clamp behavior and keep it consistent.
- [x] Ensure the active tab and focus state remain synchronized.
- [x] Preserve mouse click switching.
- [x] Preserve machine-list focus return behavior where appropriate.
- [x] Do not steal keyboard input when gameplay owns the surface.
- [x] Add component tests for right-panel Left/Right switching.
- [x] Add focus-state assertions.
- [x] Run applicable frontend tests on exact head.

**Evidence:** Promoted through PR #91. Candidate `aae43a18ed0c56d74e02b534638c1c1707396938` added `rightPanelKeyboard.test.ts` and `MachineRightPanel.test.tsx` coverage for Images/Infos ArrowLeft/ArrowRight clamp semantics, `aria-selected`/tabIndex/class sync, click preservation, and gameplay-owned input guard. Exact candidate passed push workflows `35784447054`, `35784447063`, `35784447198`, `35784447120`, `35784447222` and PR workflows `35786608672`, `35786608747`, `35786608579`, `35786608745`, `35786608643`; promoted as `cca7c5e41cea4792a89dea46f023068a5f7826af` with post-merge workflows `35787585902`, `35787585857`, `35787585845`, `35787586112`, `35787585893`.

---

## MTR-010 — Separate artwork loading, missing, ready, and error states

- [x] Introduce explicit asset-loading state for a selected artwork slot.
- [x] Do not render No image Available while an asset fetch is merely pending.
- [x] Distinguish no slots from selected-slot asset missing/unavailable.
- [x] Normalize artwork errors through the shared frontend error formatter.
- [x] Invalidate stale artwork responses when machine selection changes.
- [x] Invalidate stale artwork responses when artwork slot/category changes.
- [x] Preserve Snapshots/default category behavior.
- [x] Preserve no-machine-selected behavior.
- [x] Add component test for slot loading state.
- [x] Add component test for true missing-artwork state.
- [x] Add component test for artwork error formatting.
- [x] Add out-of-order artwork response regression test.
- [x] Run applicable frontend tests on exact head.

**Evidence:** Present and qualified on promoted master `cca7c5e41cea4792a89dea46f023068a5f7826af`. `artworkState.ts` defines missing/loading/ready/error states and stale request identity checks; `ArtworkAssetFrame.tsx` renders loading without `No image Available`, true missing separately, ready assets, and formatted alerts. `MachineRightPanel.tsx` increments discovery/asset request identifiers on machine and slot/category changes. Tests in `ArtworkAssetFrame.test.tsx` and `artworkState.test.ts` cover loading, true missing, formatted errors, state derivation, and out-of-order rejection. Exact promoted master passed Tauri project `35787585902`, Windows `35787585857`, macOS `35787585845`, Linux `35787586112`, and security `35787585893`.

---

## MTR-011 — Reduce MameBrowser state coupling where it improves correctness

- [x] Map MameBrowser responsibilities before refactoring.
- [x] Identify ownership boundaries for catalog query lifecycle.
- [x] Identify ownership boundaries for selected-machine/detail lifecycle.
- [x] Identify ownership boundaries for activation/launch orchestration.
- [x] Identify ownership boundaries for keyboard shortcuts.
- [x] Identify ownership boundaries for persisted browser state.
- [x] Extract only the boundaries that materially improve correctness/testability.
- [x] Prefer explicit hooks/state machines such as useMachineSelectionDetail or equivalent where they make lifecycle identity testable.
- [x] Keep authoritative backend/catalog APIs unchanged unless a concrete defect requires change.
- [x] Avoid a framework rewrite.
- [x] Avoid moving code into new files without reducing implicit coupling.
- [x] Keep each behavior-changing refactor covered by executable tests.
- [x] Review MameBrowser size/complexity after remediation and document remaining intentional responsibilities.
- [x] Run applicable frontend tests on exact head.

**Evidence:** Reconciled by `docs/MAME_TAURI_MTR_011_STATE_COUPLING_REVIEW_2026-09-22.md` and promoted through PR #93 as `25d7eff9ade8f703616400108aba4274d9c6c93d`. The review maps catalog, selection/detail, activation/launch, keyboard, and persistence ownership and records extracted correctness boundaries: `machineAsyncIdentity.ts`, `model.ts`, `machineListVisibility.ts`, `rightPanelKeyboard.ts`, `artworkState.ts`, `ArtworkAssetFrame.tsx`, and Software Browser component coverage. No backend/catalog API changes, framework rewrite, or behavior-changing refactor were introduced.

---

## MTR-012 — Replace false-positive qualification with real interaction tests

- [x] Inventory existing parity tests and classify each as static/source, pure unit, component, integration, or platform smoke.
- [x] Keep useful static source tripwires but stop using them as sole behavioral evidence.
- [x] Add executable stale-detail tests from MTR-001.
- [x] Add executable activation tests from MTR-002.
- [x] Add executable default-shell/footer composition tests from MTR-004.
- [x] Add Software Browser component tests from MTR-006.
- [x] Add selection-visibility tests from MTR-007.
- [x] Add keyboard navigation tests from MTR-008/MTR-009.
- [x] Add artwork-state tests from MTR-010.
- [x] Add gameplay-input ownership regression coverage.
- [x] Add negative regression guard for MAME Tauri Frontend default branding.
- [x] Add negative regression guard for disallowed system-color product styling.
- [x] Ensure tests fail when the corresponding defect is intentionally reintroduced.
- [x] Remove or rewrite misleading tests that merely assert implementation strings while claiming interaction coverage.
- [x] Document unavoidable jsdom/WebView limitations precisely.
- [x] Run the complete frontend test suite on exact head.

**Evidence:** Reconciled by `docs/MAME_TAURI_MTR_012_TEST_EVIDENCE_INVENTORY_2026-09-22.md` and promoted through PR #95 as `011429b921642253bc8c7e2714abc980f3fbf755`. The inventory classifies pure state/model tests, server-rendered component tests, static/source tripwires, and platform workflow evidence. Static tripwires are retained only for narrow composition/theme guards. The complete frontend/unit/component suite passed in Tauri project run `35787585902` on promoted implementation head `cca7c5e41cea4792a89dea46f023068a5f7826af`; MTR-012 docs validation passed on exact head and post-merge docs run `35795933365`.

---

## MTR-013 — Rendered visual parity re-qualification

- [x] Update the manual visual checklist with every issue from this remediation.
- [x] Render the actual Tauri/WebView application in the strongest available environment.
- [x] Inspect default startup shell.
- [x] Inspect title/search/header and verify generic Tauri branding is absent.
- [x] Inspect left filter, central list, right panel geometry/density.
- [x] Inspect selected blue/yellow row.
- [x] Inspect muted unavailable row.
- [x] Inspect green driver/status region and verify it is truly bottom-most.
- [x] Verify History/Collections/Diagnostics are not permanent default footer chrome.
- [x] Inspect Software Browser.
- [x] Inspect Images and Infos tabs.
- [x] Inspect artwork loading state.
- [x] Inspect true no-image state.
- [x] Inspect metadata not-configured/importing/failure/ready states as practical.
- [x] Inspect configuration surface for host-theme leakage.
- [x] Inspect visible focus states.
- [x] Record environment details for the rendered review.
- [x] Attach screenshots/artifacts if the available tool path supports them.
- [x] If binary screenshot capture remains unavailable through Ralph, keep the existing defer rationale but record a complete textual rendered review.
- [x] Do not mark this task complete from source inspection alone.

**Evidence:** MTR-013 textual rendered re-qualification is recorded in `docs/MAME_TAURI_MTR_013_RENDERED_VISUAL_REVIEW_2026-09-22.md`, added by PR #96 candidate `c3c0b059882a926238f09d9b5aad1786392687aa` and promoted as `792f440740ff977c8ca7074e3856a4c9f03ff5b4`. The review updates the visual checklist for all reopened issues and uses the strongest Ralph-accessible runtime path: Tauri project run `35787585902` on `cca7c5e41cea4792a89dea46f023068a5f7826af`, where `linux-quality` job `106948027544` and `linux-release-qualification` job `106948639364` passed, including `Tauri production build smoke check` and `Tauri development window smoke check`. Ralph artifact inspection exposed qualification/report artifacts (`mame-tauri-mt2004-external-window-rc-*`, `mame-tauri-mt1800-cross-platform-qualification-*`, `mame-tauri-mt1700-performance-baseline-*`, `mame-tauri-mt2100-final-quality-closure-*`, and `mame-tauri-license-inventory-*`) but no binary screenshot or pixel-baseline artifact. The review records a complete textual rendered review for default shell, header/search, list/filter/right-panel geometry, selected/muted rows, bottom driver/status, secondary tools, Software Browser, Images/Infos, artwork states, metadata states, configuration host-theme leakage, and focus states without claiming pixel evidence. Exact MTR-013 docs candidate passed push docs `35796616918` and PR docs `35796633620`; promoted master docs passed `35797062822`.

---

## MTR-014 — Documentation reconciliation

- [x] Update README or documentation index to reference this remediation track where appropriate.
- [x] Preserve the original MTP TODO as historical evidence.
- [x] Preserve the original final parity report as historical evidence.
- [x] Clearly state that the 2026-09-21 post-closure review reopened affected requirements.
- [x] Correct documentation that claims no unresolved parity work while remediation remains open.
- [x] Record the traceability from each remediation item to the original MTP areas.
- [x] Document the final keyboard PageUp/PageDown/Home/End semantics.
- [x] Document the final location/access path for secondary tools moved out of the bottom footer.
- [x] Document any intentional remaining visual deviations.
- [x] Document test-category distinctions: static tripwire versus behavioral/component/platform evidence.
- [x] Keep every individual checkbox in this TODO after completion.
- [x] Append exact evidence instead of replacing detailed tasks with summary-only checkboxes.
- [x] Run documentation validation on exact head.

**Evidence:** MTR-014 is documented by `docs/MAME_TAURI_MTR_014_DOCUMENTATION_RECONCILIATION_2026-09-22.md`, and README now distinguishes the historical original parity track from the reopened 2026-09-21 remediation track. PR #98 promoted README/remediation documentation updates as `03f53a0ee54856f1985a09f668ab1dd9ea3f1c10`, preserving the original MTP TODO and final report as historical evidence while stating that current closure status is governed by this MTR ledger until MTR-015. The MTR-014 reconciliation document records traceability from MTR-000 through MTR-015 to original parity areas, final viewport-aware PageUp/PageDown and current-page Home/End semantics, right-panel clamp semantics, the `More`/contextual secondary-tools path, intentional visual deviations, and static/source versus behavioral/component/platform evidence distinctions. PR #98 exact head `04036a602b6c8f151a56f10eb283b1b7b88e3910` passed documentation validation run `35797884676`; promoted master documentation validation passed `35798408837`.

---

## MTR-015 — Full qualification and closure

- [ ] Re-read every MTR-000 through MTR-014 checkbox from the current candidate head.
- [ ] Confirm no unresolved behavioral defect from the review remains silently marked complete.
- [ ] Run the complete frontend/unit/component test suite.
- [ ] Run applicable Tauri project workflow(s).
- [ ] Run applicable security workflow(s).
- [ ] Run applicable Linux packaging/platform workflow(s).
- [ ] Run applicable Windows packaging/platform workflow(s).
- [ ] Run applicable macOS packaging/platform workflow(s).
- [ ] Run documentation workflow(s).
- [ ] Record every applicable exact-head run ID and conclusion.
- [ ] Confirm the candidate head SHA is unchanged after qualification.
- [ ] Attach/update final rendered visual evidence.
- [ ] Create/open the final reconciliation PR if the current work is not already in a qualified PR.
- [ ] Merge only the exact-head-qualified candidate through the normal gated path.
- [ ] Reload this TODO from promoted master immediately after merge.
- [ ] Record the promoted master SHA.
- [ ] Verify applicable post-merge master CI.
- [ ] Record post-merge run IDs and conclusions.
- [ ] Confirm the default-visible MAME Tauri Frontend branding is absent on promoted master.
- [ ] Confirm the green driver/status region is the true bottom persistent region on promoted master.
- [ ] Confirm stale detail and activation race regression tests are present on promoted master.
- [ ] Confirm Software Browser no longer relies on host-system product colors on promoted master.
- [ ] Confirm the TODO still contains all individual subtasks and their evidence.
- [ ] Only then mark this remediation closed and stop/disable any dedicated remediation loop.

**Evidence:** pending.

---

## Completion rule

This remediation is complete only when every checkbox above is individually reconciled and the final promoted master satisfies the companion spec's completion rule.

A passing static/source test is not sufficient evidence for interactive behavior. A passing old CI run is not sufficient evidence for a newer candidate SHA. A documentation-only reconciliation is not sufficient evidence that an implementation defect was fixed.
