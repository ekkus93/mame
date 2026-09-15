# MAME Tauri UI Post-Closure Remediation TODO — 2026-09-14

**Repository:** `ekkus93/mame`  
**Baseline:** `master` at `b49a2b1652dbf929a71698038a917ff9abeb746f`  
**Spec:** `docs/MAME_TAURI_UI_POST_CLOSURE_REMEDIATION_SPEC_2026-09-14.md`  
**Scope:** remediate the visual-parity, fixed-viewport, and MAME configuration/catalog bootstrap defects discovered by direct comparison with native MAME.

## PCR-001 — Reproduce the MAME visual language

- [ ] Inventory the current MAME browser CSS/theme tokens and identify the generic light-theme rules responsible for the reviewed appearance.
- [ ] Define centralized MAME-derived color tokens for deep navy, content navy, structural blue, selected blue/cyan, yellow accent, primary text, muted text, green success/status, warning, and error.
- [ ] Make the MAME-derived dark presentation canonical for the primary MAME workspace rather than inheriting a generic OS/light-web theme.
- [ ] Restyle the title/application strip to visually belong to the MAME workspace.
- [ ] Restyle primary navigation and search/toolbar chrome.
- [ ] Restyle the left filter rail with clear selected/focus states.
- [ ] Restyle the dense machine list with strong MAME-like selected-row treatment.
- [ ] Restyle the software browser consistently with the machine browser.
- [ ] Restyle Images/Info context with clear active-mode treatment and MAME-like panel boundaries.
- [ ] Restyle the bottom status region so it is visually distinct and persistent.
- [ ] Style unavailable, not-working/preliminary, disabled, favorite, hover, focus, warning, success, and error states.
- [ ] Ensure status meaning remains understandable without color alone.
- [ ] Verify keyboard focus remains clearly visible against the dark palette.
- [ ] Preserve accessibility contrast for text and controls.
- [ ] Remove or isolate stale light-theme rules that can override the canonical MAME workspace palette.
- [ ] Add regression coverage that asserts the canonical MAME theme/token contract exists.

## PCR-002 — Eliminate document-level scrolling in the primary browser

- [ ] Audit `html`, `body`, React root, application shell, `MameShell`, and browser-region sizing/overflow rules.
- [ ] Make the Tauri application root resolve to the usable window height.
- [ ] Set the primary application/document container to avoid normal document-level vertical scrolling.
- [ ] Convert the primary shell to an explicit fixed-height flex/grid composition.
- [ ] Add required `min-height: 0` / `min-width: 0` constraints so nested grid/flex children can shrink instead of forcing page overflow.
- [ ] Keep title/header/navigation visible while browsing machines.
- [ ] Keep search/toolbar visible while browsing machines.
- [ ] Keep bottom status bar visible while browsing machines.
- [ ] Make the machine list own its internal vertical scrolling.
- [ ] Make the software list own its internal vertical scrolling.
- [ ] Make the filter rail own internal scrolling only when necessary for short windows.
- [ ] Make long Info/context content scroll internally when necessary.
- [ ] Ensure the right-side narrow-window Details strategy still works after fixed-height containment.
- [ ] Verify no primary-browser element creates accidental body overflow through margins, intrinsic minimum sizes, or `100vh` plus extra chrome.
- [ ] Add regression coverage for root overflow/fixed shell contracts.

## PCR-003 — Qualify fixed-window behavior at representative desktop sizes

- [ ] Qualify the primary machine browser at 1920×1080.
- [ ] Qualify the primary machine browser at 1366×768.
- [ ] Qualify the primary machine browser at 1280×720.
- [ ] At each size, confirm no document-level vertical scrollbar is present in normal machine browsing.
- [ ] At each size, confirm title/navigation, toolbar/search, main workspace, and bottom status are simultaneously visible.
- [ ] At each size, confirm machine-list internal scrolling works.
- [ ] At each size, confirm filter-list internal scrolling works when the full filter inventory cannot fit.
- [ ] At each size, confirm right-panel content remains usable.
- [ ] Record the qualification evidence in the remediation closure document.

## PCR-004 — Model bootstrap/catalog readiness explicitly

- [ ] Inventory the current frontend readiness, MAME configuration, metadata-generation, and catalog-query states.
- [ ] Define a typed frontend/backend-facing readiness model that distinguishes `MAME not configured` from `metadata absent`, `importing`, `import failed`, `catalog ready`, and `catalog ready but current query empty`.
- [ ] Ensure `0 machines` is only shown as a catalog/query result when an active catalog actually exists.
- [ ] Ensure missing active metadata generation is not presented as an ordinary empty library.
- [ ] Preserve safe error details without exposing sensitive filesystem data unnecessarily.
- [ ] Add pure state-model tests covering every readiness state and legal transition.

## PCR-005 — Add an explicit MAME-not-configured workspace

- [ ] Detect the absence of a usable configured MAME executable before issuing normal library queries.
- [ ] Replace the misleading empty-list presentation with a prominent `MAME is not configured` state.
- [ ] Add a primary `Configure MAME` action in that state.
- [ ] Explain that machine metadata cannot be populated until MAME is configured.
- [ ] Keep secondary diagnostics/settings navigation available.
- [ ] Do not imply that the catalog contains zero machines.
- [ ] Return automatically to readiness evaluation after configuration changes.
- [ ] Add tests for the not-configured presentation and transition out of it.

## PCR-006 — Add metadata-not-imported and import-failed workspaces

- [ ] Detect `MAME configured but no successful active metadata generation` as a separate state.
- [ ] Present a prominent `Import machine list` / `Import MAME metadata` action.
- [ ] Surface configured MAME identity in an appropriate details/settings path.
- [ ] Add an explicit import-in-progress state.
- [ ] Prevent conflicting duplicate imports while one is active.
- [ ] Transition automatically to a populated catalog query after successful import.
- [ ] Add an explicit import-failed state with safe actionable error text.
- [ ] Add `Retry import` to the failed state.
- [ ] Add `Configure MAME` to the failed state when executable/path configuration may be relevant.
- [ ] Link to Diagnostics for deeper import failures where appropriate.
- [ ] Add tests for absent/importing/failed/succeeded transitions.

## PCR-007 — Investigate the reviewed zero-machine installation

- [ ] Reproduce the reviewed state in which the UI shows `MAME not configured`, no active metadata generation, and `0 machines`.
- [ ] Determine whether MAME configuration was never completed, was not persisted, or was not reloaded.
- [ ] Verify configured MAME executable identity validation behavior.
- [ ] Verify whether metadata import is manual, automatic, or currently undiscoverable in the primary workflow.
- [ ] Verify metadata import execution and error propagation using the configured MAME executable.
- [ ] Verify successful generations become the active catalog generation.
- [ ] Verify the first unfiltered catalog query returns machines after activation.
- [ ] If the catalog is active but still returns zero machines, diagnose and fix the Rust query/import defect rather than masking it in the UI.
- [ ] Record the verified root cause of the reviewed zero-machine state.
- [ ] Add regression coverage for the confirmed root cause so it cannot silently recur.

## PCR-008 — Make first-run path lead to a populated browser

- [ ] Define the final first-run flow from app launch to populated machine browser.
- [ ] Make MAME configuration reachable directly from the unavailable primary state.
- [ ] Validate MAME identity immediately after executable selection/configuration.
- [ ] Decide whether metadata import begins automatically after successful configuration or requires an explicit primary action.
- [ ] If automatic, show progress/failure/retry states.
- [ ] If manual, make the import call-to-action impossible to confuse with an ordinary empty library.
- [ ] Refresh readiness and query state after configuration/import without requiring application restart.
- [ ] Verify returning to the Machines view after setup shows the populated catalog.
- [ ] Add transition tests for a fresh configuration → import → ready workflow.

## PCR-009 — Preserve truthful empty-result behavior

- [ ] Distinguish an active-catalog zero search result from missing metadata/catalog state.
- [ ] Distinguish an active-catalog zero filter result from missing metadata/catalog state.
- [ ] Provide clear `No machines match this search/filter` copy for true query-empty states.
- [ ] Preserve current filter/search state while reporting a true zero result.
- [ ] Do not offer import/configuration remediation when the catalog is healthy and only the query is empty.
- [ ] Add tests for healthy-catalog empty search/filter results.

## PCR-010 — Strengthen regression coverage

- [ ] Extend `scripts/tauri/test-mame-ui-reproduction.py` or add a dedicated post-closure remediation regression.
- [ ] Assert the canonical MAME-derived theme/token contract.
- [ ] Assert the fixed root/shell overflow contract.
- [ ] Assert the primary browser contains explicit configuration/import unavailable states.
- [ ] Assert a generic `0 machines` state cannot represent missing configuration/metadata.
- [ ] Preserve existing MAME-UI architecture checks.
- [ ] Preserve event-name, security, post-closeout, and performance regressions.
- [ ] Preserve frontend format/lint/typecheck/test/build gates.
- [ ] Preserve Rust format/test/Clippy gates.
- [ ] Preserve the hardened real Tauri development-window smoke.

## PCR-011 — Add runtime layout qualification where deterministic

- [ ] Evaluate whether the existing Xvfb/xdotool development-window smoke can deterministically inspect viewport/body overflow.
- [ ] If reliable, add a runtime assertion that document scroll height does not exceed the primary viewport in the normal browser state.
- [ ] If reliable, add assertions that top chrome and bottom status are simultaneously present/visible.
- [ ] Do not add brittle pixel-coordinate checks that are likely to fail solely from GTK/WebKit rendering differences.
- [ ] Document any layout property that remains human-qualified rather than automated.

## PCR-012 — Perform explicit visual comparison

- [ ] Capture/review a Tauri screenshot for MAME-not-configured state.
- [ ] Capture/review a Tauri screenshot for metadata-not-imported state.
- [ ] Capture/review a populated unfiltered machine-browser state.
- [ ] Capture/review a selected-machine Images state.
- [ ] Capture/review a selected-machine Info state.
- [ ] Capture/review a 1366×768 fixed-window state with top and bottom chrome simultaneously visible.
- [ ] Compare the final visual hierarchy directly against the supplied native MAME screenshot.
- [ ] Confirm the final app reads as MAME-derived rather than as a generic light web app.
- [ ] Confirm document-level page scrolling is absent from the primary browser.
- [ ] Confirm selected-state, panel boundaries, toolbar, right panel, and bottom status have deliberate MAME-like visual treatment.
- [ ] Record any intentional visual differences and their rationale.
- [ ] Do not close this task from source inspection alone.

## PCR-013 — Update documentation and closure evidence

- [ ] Update `README.md` if setup/bootstrap behavior changes.
- [ ] Document the canonical MAME-derived visual theme.
- [ ] Document fixed-window scroll ownership.
- [ ] Document the first-run Configure MAME → metadata import → populated library workflow.
- [ ] Document each explicit bootstrap/catalog state.
- [ ] Add a remediation closure/evidence document rather than rewriting history in the previous closure record.
- [ ] Record the verified root cause of the reviewed zero-machine state.
- [ ] Record representative visual-review evidence.
- [ ] Record exact final PR-head and promoted-master CI run IDs.

## PCR-014 — Final exact-head qualification and promotion

- [ ] Reconcile every PCR task/subtask as complete, explicitly deferred with rationale, superseded with rationale, or not required.
- [ ] Confirm no ambiguous unchecked item remains before closure.
- [ ] Run the remediation/static architecture regression on the exact final candidate.
- [ ] Qualify the exact final PR head through `Tauri project`.
- [ ] Qualify the exact final PR head through `Tauri security` when applicable.
- [ ] Qualify the exact final PR head through Linux packaging.
- [ ] Qualify the exact final PR head through Windows packaging.
- [ ] Qualify the exact final PR head through macOS packaging.
- [ ] Qualify documentation workflow when triggered.
- [ ] Confirm the PR-event hardened Linux production build smoke passes.
- [ ] Confirm the PR-event real Tauri development-window smoke passes.
- [ ] Confirm full-catalog performance qualification remains within accepted budgets.
- [ ] Complete the explicit visual comparison in PCR-012 before closure.
- [ ] Merge only an exact-head-qualified candidate.
- [ ] Reload this TODO from promoted `master` immediately after merge.
- [ ] Verify applicable post-merge CI on the promoted `master` SHA.
- [ ] Append promoted SHA, run IDs, root-cause result, and visual-review disposition to the remediation closure evidence.

## Completion rule

This remediation is not complete merely because code compiles or CI is green. Closure requires all of the following simultaneously:

- recognizably MAME-derived color/visual hierarchy;
- fixed primary viewport with no document-level vertical scrolling at the required desktop sizes;
- explicit, actionable MAME configuration and metadata bootstrap states;
- a verified explanation and regression for the observed zero-machine condition;
- a successful configuration/import path that leads to a populated machine browser without restart;
- exact-head automated qualification;
- explicit final screenshot/visual comparison against the native MAME reference;
- reconciliation of every PCR task;
- successful merge and post-merge verification on `master`.
