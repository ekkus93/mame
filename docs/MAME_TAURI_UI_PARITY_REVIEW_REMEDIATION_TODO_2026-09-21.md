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

**Evidence:** MTR-000 baseline reconciliation was performed from promoted master `d11481284ce83724084a0ad874d0fb6dfc27fbae`. The remediation spec `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_SPEC_2026-09-21.md`, original parity spec `docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md`, current reference `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md`, and final report `docs/MAME_TAURI_UI_PARITY_FINAL_REPORT_2026-09-21.md` were reread. Because current `docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md` is condensed, the detailed historical checklist was read from pre-condense parent commit `9a7f1e7441f67709a3231289ed4517df151d2c87`, where MTP-000 through MTP-015 remained expanded. The claims reopened by this remediation are the prior final report's no-unresolved-parity-work closure, source/static tripwire sufficiency for interaction behavior, complete keyboard/mouse parity, Software Browser visual parity, default shell footer disposition, host-system color removal, selected-detail/activation race safety, selected-row visibility, right-panel tab keyboard handling, and artwork loading/missing/error separation. Current promoted-master CI baseline for `d11481284ce83724084a0ad874d0fb6dfc27fbae` was all green: Tauri project run `35730623025`, Windows packaging `35730622875`, macOS packaging `35730622891`, Linux packaging `35730622749`, and Tauri security `35730623007`. Current responsibility files are: `tauri/src/browser/MameBrowser.tsx` plus `tauri/src/browser/machineAsyncIdentity.ts` for selection/detail and activation identity; `tauri/src/browser/model.ts`, `tauri/src/browser/MachineList.tsx`, and `tauri/src/browser/machineListVisibility.ts` for query selection, keyboard movement, and selected-row visibility; `tauri/src/shell/MameShell.tsx`, `tauri/src/shell/MameShell.css`, and `tauri/src/browser/MachineDriverStatus.tsx` for default shell, secondary-surface access, and bottom driver/status composition; `tauri/src/browser/SoftwareBrowser.tsx` and `tauri/src/browser/SoftwareBrowser.css` for Software Browser behavior/styling; `tauri/src/browser/MachineRightPanel.tsx`, `tauri/src/browser/rightPanelKeyboard.ts`, and `tauri/src/browser/artworkState.ts`, and `tauri/src/browser/ArtworkAssetFrame.tsx` for right-panel keyboard and artwork lifecycle; and the parity/interaction tests under `tauri/src/browser/*test*` plus `tauri/src/shell/mameShellComposition.source.test.ts` for regression coverage. This update is additive: the historical MTP TODO and final report remain intact as evidence, while this remediation ledger records the reopened requirements.

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

**Evidence:** MTR-001 was implemented by the PR #74 candidate head `a6b7f7495a76ce7f7e00b75e07d71be680f2cbca` and promoted to master as `d11481284ce83724084a0ad874d0fb6dfc27fbae`. `tauri/src/browser/machineAsyncIdentity.ts` introduces explicit selected-machine async identity with `detailGeneration` and `activationGeneration`; `selectMachineIdentity` increments both generations whenever the selected short name changes, including transitions to `null`. `tauri/src/browser/MameBrowser.tsx` routes all selection changes through `selectMachine`, synchronously updates `selectedRef`, advances the async identity, resets `detailSequence`/`activationSequence`, clears `pendingLaunchOverrides`, and resets `launchState` when machine identity changes. Metadata refresh/import, catalog-not-queryable, value-required, query failure, and empty-result reconciliation paths all call `selectMachine(null)` or `selectMachine(reconcileMachineSelection(...))`, so stale detail work is invalidated across those paths. Detail success and failure continuations both use `detailMayCommit(asyncIdentityRef.current, sequence, shortName)` before mutating `detailState`, preventing stale success or stale error from replacing the current selected-machine state. Right-panel, driver/status, BIOS/configuration/software/audit actions, and launch controls are all rendered from `detailState.status === "ready" ? detailState.detail : null`, so they cannot observe a stale detail once the identity guard rejects it. `tauri/src/browser/machineSelectionIdentity.test.ts` adds deterministic executable identity regressions for pending detail invalidation after selection clear, out-of-order A/B detail completion, and activation invalidation when identity changes. Exact candidate `a6b7f7495a76ce7f7e00b75e07d71be680f2cbca` passed all five PR workflows: Tauri project `35729905106`, Windows packaging `35729905090`, macOS packaging `35729905021`, Linux packaging `35729905031`, and Tauri security `35729905103`; the same head also had passing push workflows `35725817004`, `35725817012`, `35725817047`, `35725817011`, and `35725817089`. Promoted master `d11481284ce83724084a0ad874d0fb6dfc27fbae` then passed post-merge push workflows `35730623025`, `35730622875`, `35730622891`, `35730622749`, and `35730623007`.

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

**Evidence:** MTR-002 activation race coverage was implemented and promoted through PR #76. The authoritative activation contract is: an activation gesture targets the currently selected machine identity for that gesture; if matching detail is already ready, launch may proceed immediately, otherwise activation fetches matching detail and may commit only while `activationMayCommit(asyncIdentityRef.current, detailGeneration, activationGeneration, shortName)` still proves the selected machine, detail generation, and activation generation are current. `tauri/src/browser/MameBrowser.tsx` preserves machine availability guards by returning early when `!machine.runnable` or when the activated row is no longer the selected short name. It preserves BIOS omission, Start Empty, software-list/software-part behavior, and visible in-surface launch errors by continuing to route actual launches through the existing `launchDetail`, `pendingLaunchOverrides`, `SoftwareBrowser`, and `mame-browser-banner is-error` paths. `tauri/src/browser/machineSelectionIdentity.test.ts` now includes executable state regressions for rapid double-click activation waiting for current machine detail, immediate Enter activation after keyboard selection waiting for selected detail, and rapid A activation invalidated when B is selected before A detail returns. Exact candidate head `90505ed09c26c4eec982271e0b34a2c687816bf9` passed all five PR workflows: Tauri project `35744067325`, Windows packaging `35744067306`, macOS packaging `35744067377`, Linux packaging `35744067314`, and Tauri security `35744067324`; the same head also passed all five push workflows: Tauri project `35744059478`, Windows packaging `35744059523`, macOS packaging `35744059446`, Linux packaging `35744059486`, and Tauri security `35744059549`. The qualified candidate was promoted to master as `2388e2ad752e3f2be707aa0b73d94dc9b5009a81`.

---

## MTR-003 — Remove generic Tauri branding from default UI

- [x] Remove the default-visible MAME Tauri Frontend pseudo-title from MameShell.css or equivalent.
- [x] Audit default startup/browser surfaces for equivalent generic Tauri rewrite branding.
- [x] Keep implementation/debug branding only in non-default developer/debug contexts if still needed.
- [x] Add a negative static/component regression guard that the prohibited default-visible phrase is absent.
- [ ] Render/inspect the default browser and confirm original-like title/search treatment remains intact.
- [ ] Confirm removing the pseudo-title does not introduce unwanted vertical whitespace or break density.
- [x] Run applicable frontend tests on exact head.

**Evidence:** MTR-003 implementation was split across PR #80 and PR #81. PR #80 promoted master `8b558daff71baa089b89b8a73c6a62b45fb2d3cd` removed the default-visible pseudo-title container from `tauri/src/shell/MameShell.css`; the default `.mame-browser` composition now starts with the toolbar/search row and uses `grid-template-rows: auto auto minmax(0, 1fr) auto` with no retained pseudo-title row. PR #81 promoted master `2c373015e32744896f74278b9d4f7df478d863de` added `tauri/src/shell/mameShellComposition.source.test.ts` coverage importing `MameShell.tsx`, `MameBrowser.tsx`, and `MameShell.css` as raw sources and asserting that the prohibited default-visible phrase `MAME Tauri Frontend` is absent from each default browser/shell source. The audit found no remaining default startup/browser generic Tauri rewrite branding in those default surface files; no developer/debug-context branding was retained. Exact PR #81 candidate head `d409983dbe9352832db055e73c47cde38c728fa3` passed all required PR workflows: Tauri project `35750198886`, Windows packaging `35750198822`, macOS packaging `35750199031`, Linux packaging `35750198809`, and Tauri security `35750198826`; it also passed all push workflows for the same exact head: Tauri project `35750170743`, Windows packaging `35750170788`, macOS packaging `35750170767`, Linux packaging `35750170817`, and Tauri security `35750170828`. Rendered browser/title/search/density inspection remains intentionally unchecked here and must be resolved by the rendered visual re-qualification pass in MTR-013 rather than being claimed from source inspection alone.

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
- [ ] Perform rendered inspection of the full-height default shell.
- [x] Run applicable frontend tests on exact head.

**Evidence:** implementation promoted to master as squash commit `2d8fbf30d45a597d7646624fbf0321d05109c449` via PR #60. The default footer was removed; `MachineDriverStatus` is now the final persistent browser region; Configure Options, Audit, History, Collections, Diagnostics, and Session/session status remain reachable through the compact `More` utility menu; regression tripwires cover shell composition and utility reachability. Exact candidate head `4252e1f3b640318275bee73a4fb7a6c25a2167ea` passed PR-triggered Tauri project run 35674026992, Windows packaging 35674027017, macOS packaging 35674026990, Linux packaging 35674027023, and Tauri security 35674026976. Rendered full-height inspection remains intentionally unchecked until actual rendered evidence is recorded under MTR-013; source/CI evidence is not substituted for that requirement.

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

**Evidence:** implementation promoted to master in PR #63. The parity audit covers startup/connect styling, default browser surfaces, Software Browser, and configuration surfaces. Visible host-system palette dependencies were replaced with semantic MAME palette tokens, and the static theme guard scans the parity stylesheets for prohibited host-system product styling. Exact candidate eabfb6f48b4b72b3df55bb3bbb8bc9afc960ea24 passed all five push workflows and all five PR workflows.

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
- [ ] Perform rendered inspection of Software List with representative states.
- [x] Run applicable frontend tests on exact head.

**Evidence:** MTR-006 implementation was promoted through PR #86 and PR #87. PR #86 candidate `bc6b420b9e194a80a12653bdfce22eaf3da05333` updated `tauri/src/browser/SoftwareBrowser.css` so the Software Browser uses explicit MAME palette tokens (`--mame-bg`, `--mame-toolbar`, `--mame-border`, `--mame-border-muted`, `--mame-selected`, `--mame-selected-text`, `--mame-disabled`, `--mame-focus`, `--mame-muted`, `--mame-warning`), compact toolbar/header sizing, dense 1.45rem software rows, selected blue/yellow treatment, muted unsupported/partial support treatment, visible focus outlines, and tokenized splitters/borders without `Canvas`, `CanvasText`, `ButtonFace`, `ButtonText`, or product `currentColor` reliance. PR #86 added/strengthened `tauri/src/browser/SoftwareBrowserParity.source.test.ts` theme guards. PR #87 candidate `2a04f78eb39c33aaf0942dcd94de15b57e3ce686` added `tauri/src/browser/SoftwareBrowserComponent.test.tsx`, which renders the exported `SoftwareResultsList` with `react-dom/server` and asserts selected-row class/ARIA state, muted support classes, software metadata output, row support state data, selection callback identity, activation callback identity, and multi-part/no-part Start disabling through executable tests rather than source-string checks alone. Production `tauri/src/browser/SoftwareBrowser.tsx` preserves software-part selection, Start behavior, BIOS omission/selection semantics, Back/Escape behavior, and MAME-surface error banners while routing rows through the tested list component. Exact candidate `2a04f78eb39c33aaf0942dcd94de15b57e3ce686` passed all five push workflows: Tauri project `35762584473`, Windows packaging `35762584480`, macOS packaging `35762584513`, Linux packaging `35762584530`, and Tauri security `35762584472`; it also passed all five PR workflows: Tauri project `35763249390`, Windows packaging `35763249337`, macOS packaging `35763249676`, Linux packaging `35763249324`, and Tauri security `35763249333`. The rendered Software List inspection remains intentionally unchecked here and must be completed under MTR-013; source/static/component evidence is not substituted for the manual rendered review requirement.

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

**Evidence:** MTR-007 implementation was promoted through PR #89. Candidate `9799ab0a529ae241d9d22ea349393975f08b3c1a` added deterministic selected-row visibility reconciliation in `tauri/src/browser/machineListVisibility.ts` and executable coverage in `tauri/src/browser/MachineList.visibility.test.ts`. The production `MachineList` effect documents and applies the contract: query/filter/search result replacement may reveal a newly selected row with nearest scrolling, an already-visible row is left stationary, and keyboard navigation continues to use the existing focus-driven path. The helper computes viewport/row geometry where available and falls back to nearest scrolling when geometry is unavailable, preserving existing behavior under non-DOM test doubles. The tests cover long-list replacement from a deeply scrolled position to a selected row near the beginning, selected rows near the end of a long list, no-results selection clearing, absent selected identity, partially hidden rows, and the no-scroll case for already-visible rows. Restored/persisted selection reconciliation remains routed through `reconcileMachineSelection(...)` before the list effect runs, and empty result paths call the same selection-clear flow already guarded by MTR-001 identity/action invalidation. Exact candidate `9799ab0a529ae241d9d22ea349393975f08b3c1a` passed all five push workflows: Tauri project `35769741029`, Windows packaging `35769741045`, macOS packaging `35769741017`, Linux packaging `35769740865`, and Tauri security `35769741091`; it also passed all five PR workflows: Tauri project `35773875580`, Windows packaging `35773875456`, macOS packaging `35773875498`, Linux packaging `35773875405`, and Tauri security `35773875514`. The qualified candidate was promoted to master as `559c2553cdf8815f04b5f2040f12dd930aceb58d`.

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

**Evidence:** MTR-008 implementation was promoted through PR #90. Candidate `888d4d4a5618420226c396b7996b1a5dbdf18e2b` removed the fixed ten-row PageUp/PageDown behavior from the machine-list interaction path and uses live viewport height plus measured row height to compute page movement while retaining one row of context. `tauri/src/browser/model.ts` defines deterministic page-size calculation and documents Home/End as current-fetched-page semantics rather than full matching-catalog jumps, preserving backend catalog pagination authority. Full-result Home/End was therefore intentionally not implemented because current-page semantics were retained to avoid falsely crossing backend page boundaries; the UI/tests/docs now assert the current-page contract rather than claiming full-result semantics. Existing Up/Down, Enter activation, Escape/back handling, and gameplay-input ownership checks remain on the guarded machine-list keyboard path. Tests cover page-size calculation, viewport PageUp/PageDown integration, and the documented current-page Home/End behavior. Exact candidate `888d4d4a5618420226c396b7996b1a5dbdf18e2b` passed all five push workflows: Tauri project `35780978021`, Windows packaging `35780978108`, macOS packaging `35780977936`, Linux packaging `35780977960`, and Tauri security `35780977948`; it also passed all five PR workflows: Tauri project `35781875893`, Windows packaging `35781875964`, macOS packaging `35781875784`, Linux packaging `35781875965`, and Tauri security `35781875966`. The qualified candidate was promoted to master as `dd1cbad6229939f2000099a51e017c47684c2b95`.

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

**Evidence:** MTR-009 implementation and coverage were promoted through PR #91. Candidate `aae43a18ed0c56d74e02b534638c1c1707396938` covers the `Images`/`Infos` tablist semantics with `tauri/src/browser/rightPanelKeyboard.test.ts` and server-rendered component assertions in `tauri/src/browser/MachineRightPanel.test.tsx`. `nextPrimaryRightView` defines the clamp behavior: ArrowRight moves Images to Infos, ArrowLeft moves Infos to Images, and ArrowLeft from Images returns `null` so focus may return to the machine list instead of wrapping. `MachineRightPanel` keeps `aria-selected`, `tabIndex`, selected class, and focus targets synchronized through `selectPrimaryTab`, preserves mouse click switching, and ignores right-panel arrow shortcuts when `gameplayInputOwned` is true. Exact candidate `aae43a18ed0c56d74e02b534638c1c1707396938` passed all five push workflows: Tauri project `35784447054`, Windows packaging `35784447063`, macOS packaging `35784447198`, Linux packaging `35784447120`, and Tauri security `35784447222`; it also passed all five PR workflows: Tauri project `35786608672`, Windows packaging `35786608747`, macOS packaging `35786608579`, Linux packaging `35786608745`, and Tauri security `35786608643`. The qualified candidate was promoted to master as `cca7c5e41cea4792a89dea46f023068a5f7826af`, whose post-merge push workflows also passed: Tauri project `35787585902`, Windows packaging `35787585857`, macOS packaging `35787585845`, Linux packaging `35787586112`, and Tauri security `35787585893`.

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

**Evidence:** MTR-010 behavior is present and qualified on promoted master `cca7c5e41cea4792a89dea46f023068a5f7826af`. `tauri/src/browser/artworkState.ts` defines explicit `missing`, `loading`, `ready`, and `error` asset states, `initialArtworkAssetState(Boolean(descriptor))` distinguishes selected-slot loading from true missing artwork, `artworkAssetError` routes artwork errors through the shared formatter, and `isCurrentArtworkRequest` rejects stale/out-of-order responses. `tauri/src/browser/ArtworkAssetFrame.tsx` renders loading as `Loading <label>…` without `No image Available`, renders true missing artwork separately, renders ready assets as images, and renders formatted errors inside an alert. `tauri/src/browser/MachineRightPanel.tsx` increments discovery and asset request identifiers on machine changes and slot/category changes so stale discovery or asset responses cannot commit after identity changes. It also preserves the `Snapshots` default category by seeding category kinds with `screenshot`, and `EmptyMachineRightPanel` preserves no-machine-selected behavior with the same default Images/Snapshots surface. Executable tests cover loading without false missing copy, true missing artwork, formatted error rendering, loading-vs-missing state derivation, shared error formatting, and out-of-order request rejection in `ArtworkAssetFrame.test.tsx` and `artworkState.test.ts`. Exact promoted master `cca7c5e41cea4792a89dea46f023068a5f7826af` passed all applicable workflows: Tauri project `35787585902`, Windows packaging `35787585857`, macOS packaging `35787585845`, Linux packaging `35787586112`, and Tauri security `35787585893`.

---

## MTR-011 — Reduce MameBrowser state coupling where it improves correctness

- [ ] Map MameBrowser responsibilities before refactoring.
- [ ] Identify ownership boundaries for catalog query lifecycle.
- [ ] Identify ownership boundaries for selected-machine/detail lifecycle.
- [ ] Identify ownership boundaries for activation/launch orchestration.
- [ ] Identify ownership boundaries for keyboard shortcuts.
- [ ] Identify ownership boundaries for persisted browser state.
- [ ] Extract only the boundaries that materially improve correctness/testability.
- [ ] Prefer explicit hooks/state machines such as useMachineSelectionDetail or equivalent where they make lifecycle identity testable.
- [ ] Keep authoritative backend/catalog APIs unchanged unless a concrete defect requires change.
- [ ] Avoid a framework rewrite.
- [ ] Avoid moving code into new files without reducing implicit coupling.
- [ ] Keep each behavior-changing refactor covered by executable tests.
- [ ] Review MameBrowser size/complexity after remediation and document remaining intentional responsibilities.
- [ ] Run applicable frontend tests on exact head.

**Evidence:** pending.

---

## MTR-012 — Replace false-positive qualification with real interaction tests

- [ ] Inventory existing parity tests and classify each as static/source, pure unit, component, integration, or platform smoke.
- [ ] Keep useful static source tripwires but stop using them as sole behavioral evidence.
- [ ] Add executable stale-detail tests from MTR-001.
- [ ] Add executable activation tests from MTR-002.
- [ ] Add executable default-shell/footer composition tests from MTR-004.
- [ ] Add Software Browser component tests from MTR-006.
- [ ] Add selection-visibility tests from MTR-007.
- [ ] Add keyboard navigation tests from MTR-008/MTR-009.
- [ ] Add artwork-state tests from MTR-010.
- [ ] Add gameplay-input ownership regression coverage.
- [ ] Add negative regression guard for MAME Tauri Frontend default branding.
- [ ] Add negative regression guard for disallowed system-color product styling.
- [ ] Ensure tests fail when the corresponding defect is intentionally reintroduced.
- [ ] Remove or rewrite misleading tests that merely assert implementation strings while claiming interaction coverage.
- [ ] Document unavoidable jsdom/WebView limitations precisely.
- [ ] Run the complete frontend test suite on exact head.

**Evidence:** pending.

---

## MTR-013 — Rendered visual parity re-qualification

- [ ] Update the manual visual checklist with every issue from this remediation.
- [ ] Render the actual Tauri/WebView application in the strongest available environment.
- [ ] Inspect default startup shell.
- [ ] Inspect title/search/header and verify generic Tauri branding is absent.
- [ ] Inspect left filter, central list, right panel geometry/density.
- [ ] Inspect selected blue/yellow row.
- [ ] Inspect muted unavailable row.
- [ ] Inspect green driver/status region and verify it is truly bottom-most.
- [ ] Verify History/Collections/Diagnostics are not permanent default footer chrome.
- [ ] Inspect Software Browser.
- [ ] Inspect Images and Infos tabs.
- [ ] Inspect artwork loading state.
- [ ] Inspect true no-image state.
- [ ] Inspect metadata not-configured/importing/failure/ready states as practical.
- [ ] Inspect configuration surface for host-theme leakage.
- [ ] Inspect visible focus states.
- [ ] Record environment details for the rendered review.
- [ ] Attach screenshots/artifacts if the available tool path supports them.
- [ ] If binary screenshot capture remains unavailable through Ralph, keep the existing defer rationale but record a complete textual rendered review.
- [ ] Do not mark this task complete from source inspection alone.

**Evidence:** pending.

---

## MTR-014 — Documentation reconciliation

- [ ] Update README or documentation index to reference this remediation track where appropriate.
- [ ] Preserve the original MTP TODO as historical evidence.
- [ ] Preserve the original final parity report as historical evidence.
- [ ] Clearly state that the 2026-09-21 post-closure review reopened affected requirements.
- [ ] Correct documentation that claims no unresolved parity work while remediation remains open.
- [ ] Record the traceability from each remediation item to the original MTP areas.
- [ ] Document the final keyboard PageUp/PageDown/Home/End semantics.
- [ ] Document the final location/access path for secondary tools moved out of the bottom footer.
- [ ] Document any intentional remaining visual deviations.
- [ ] Document test-category distinctions: static tripwire versus behavioral/component/platform evidence.
- [ ] Keep every individual checkbox in this TODO after completion.
- [ ] Append exact evidence instead of replacing detailed tasks with summary-only checkboxes.
- [ ] Run documentation validation on exact head.

**Evidence:** pending.

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
