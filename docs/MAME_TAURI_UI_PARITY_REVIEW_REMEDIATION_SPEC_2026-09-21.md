# MAME Tauri UI Parity Review Remediation Spec — 2026-09-21

**Repository:** ekkus93/mame  
**Baseline master:** 3a299632cf41e4d9d313c5880708773cc241d186  
**Original parity spec:** docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md  
**Original parity TODO:** docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md  
**Reference contract:** docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md  
**Prior final report:** docs/MAME_TAURI_UI_PARITY_FINAL_REPORT_2026-09-21.md  
**Companion TODO:** docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md

## 1. Purpose

This remediation reopens the Tauri UI parity effort after a source-level review of promoted master found concrete behavioral defects, visual-parity regressions, incomplete keyboard/navigation behavior, theme leakage, and qualification gaps that were not caught by the previous closure process.

The goal is not to redesign the application. The goal is to make the default Tauri frontend behave and present itself like the original MAME UI while preserving the existing Tauri/WebView architecture, Linux desktop-environment compatibility work, catalog authority, software/BIOS launch correctness, gameplay-input ownership safeguards, and other previously qualified behavior.

This spec is authoritative for the remediation work described below. It supplements the original parity spec rather than replacing it.

## 2. Baseline review findings

The review of master 3a299632cf41e4d9d313c5880708773cc241d186 identified the following issues.

### 2.1 High-priority behavioral defect: stale selected-machine detail

MameBrowser currently sequence-guards detail responses only after a non-null machine is selected. Clearing selection does not invalidate an already in-flight detail request. An older request can therefore complete after selection has been cleared and repopulate detail, right-panel, status, or action state for a machine that is no longer selected.

This can occur during metadata refresh, query failure, filter transitions, or any path that clears selection while a detail request is in flight.

### 2.2 Activation race

Machine activation depends on detailState already being ready for the activated machine. A rapid double-click or immediate Enter after selection can reach activation while detail is still loading and silently do nothing.

Activation semantics must belong to the machine the user activated, not to incidental timing of a separate detail request.

### 2.3 Forbidden generic Tauri branding remains visible

The default machine browser still injects the text MAME Tauri Frontend through MameShell.css. The original parity specification explicitly requires replacement of this presentation.

### 2.4 Green driver/status region is not the true bottom shell region

MachineDriverStatus exists and is visually green, but MameShell renders an additional permanent utility/footer strip after the browser. That footer exposes project/global controls such as Session, Configure Options, Audit, History, Collections, and Diagnostics.

The default original-MAME composition must end in the selected-machine driver/status region. Project/global secondary surfaces must remain accessible without occupying the permanent bottom status position.

### 2.5 Explicit MAME palette migration is incomplete

The main machine browser uses explicit MAME palette tokens, but normal user flows such as Software Browser still use system-driven product styling such as Canvas and currentColor. General Settings also retains system-color styling.

Primary user-facing MAME surfaces must not depend on host light/dark system colors for their product palette.

### 2.6 Software List visual parity is incomplete

Software selection is functionally integrated, but its palette, selection treatment, focus treatment, density, disabled states, and error presentation are not fully aligned with the machine browser/original-MAME visual contract.

### 2.7 Machine-list viewport behavior is incomplete

Asynchronous search/filter reconciliation can select a machine without ensuring the selected row is visible. Scroll preservation and reselection behavior need explicit semantics and tests.

### 2.8 Page navigation semantics are too weak

PageUp/PageDown currently use a fixed row delta rather than a viewport-aware page movement. Home/End behavior is limited to the currently loaded result page rather than clearly defined against the full matching catalog.

The remediation must define and implement deterministic semantics that match original-MAME expectations as closely as the paged data model permits.

### 2.9 Right-panel keyboard semantics are incomplete

The Images/Infos region supports mouse tab changes but does not implement complete arrow-key tab navigation. The parity matrix requires keyboard navigation when the region owns focus.

### 2.10 Artwork loading is conflated with missing artwork

Artwork selection clears the current asset while an asynchronous read occurs. During that interval the UI can report No image Available even though an image is merely loading. Artwork errors also need the same normalized error formatting used elsewhere in the frontend.

### 2.11 MameBrowser orchestration is too state-coupled

MameBrowser has accumulated metadata lifecycle, catalog query, selection, machine detail, artwork mode, launch, export, shortcuts, responsive state, software mode, and persistence responsibilities. The stale-detail and activation races are evidence that lifecycle ownership needs to be made more explicit.

Refactoring is required only where it materially reduces correctness risk. This is not permission for a broad rewrite.

### 2.12 Qualification tests over-rely on source-string assertions

Several existing parity tests assert that source text contains particular strings or callback names. Such tests are useful static tripwires but are not behavioral qualification.

The prior closure treated source tripwires as stronger evidence than they provide. The remediation must add real component/state interaction tests for the critical behaviors identified above.

### 2.13 Documentation overstates closure

The original TODO/final report marked affected items complete. Those historical files should remain as historical records, but current documentation must explicitly record the reopened requirements and must not claim closure until this remediation passes its completion rule.

## 3. Goals

The remediation shall:

1. Eliminate stale machine-detail and stale-action state after selection invalidation.
2. Make double-click and Enter activation reliable regardless of detail-fetch timing.
3. Remove remaining generic Tauri branding from the default MAME browser.
4. Make the green selected-machine driver/status region the true bottom region of the default MAME composition.
5. Move project/global secondary controls out of the permanent bottom status position while preserving access.
6. Complete explicit MAME palette usage across all normal machine/software/configuration flows relevant to parity.
7. Bring Software Browser visual behavior into the same MAME visual system.
8. Make search/filter reselection and scrolling deterministic and keep the selected machine visible.
9. Implement defensible PageUp/PageDown and Home/End semantics.
10. Complete keyboard semantics for the Images/Infos panel.
11. Separate artwork loading, missing, ready, and error states.
12. Reduce state-coupling where necessary to make asynchronous ownership explicit.
13. Replace critical source-text-only qualification with actual interaction/state tests.
14. Reconcile parity documentation accurately and preserve a durable audit trail.
15. Close only after exact-head qualification, gated merge, promoted-master reload, and post-merge CI verification.

## 4. Non-goals

This work must not:

- Reintroduce the original native UI implementation path.
- Replace the Tauri/WebView architecture.
- Redesign the app into a modern dashboard.
- Change catalog authority or invent availability/driver metadata.
- Change BIOS semantics so a BIOS override is supplied when the user did not explicitly choose one.
- Break Start Empty or software-part selection behavior.
- Weaken gameplay-input ownership protections.
- Introduce host-dependent system colors as the primary product palette.
- Use a broad refactor as a substitute for targeted behavior fixes.
- Mark visual parity complete solely because unit tests pass.

## 5. Architectural invariants

### 5.1 Selection/detail identity invariant

At any observable point, machine detail, right-panel content, driver/status content, launch actions, software actions, BIOS options, and configure/audit actions must correspond to the currently selected machine identity.

If selectedMachine is null, machine-specific detail/action state must not become ready due to a previously started request.

Every asynchronous machine-detail completion must prove that it still belongs to the current selection generation before mutating visible state.

### 5.2 Activation invariant

An activation gesture must target the machine associated with that gesture. If additional detail is required before launch, the UI must either fetch/await the matching detail and continue the activation if the same activation is still valid, or use an already-authoritative launch path that does not depend on unrelated detail readiness.

It must not silently discard valid double-click/Enter activation merely because detail loading has not finished.

### 5.3 Default composition invariant

The default desktop browser composition must contain:

- original-like title/search treatment without generic Tauri product branding;
- compact blue toolbar/command band;
- left filter/category region;
- central dense machine list;
- right Images/Infos region;
- green selected-machine driver/status region as the bottom-most persistent region of the default composition.

Project-specific history, collections, diagnostics, and other secondary surfaces may remain accessible but must not redefine the permanent default MAME composition.

### 5.4 Theme invariant

Default and normal user-facing MAME flows must use explicit semantic MAME tokens. Canvas, CanvasText, and currentColor may be used only where they are an intentional accessibility/OS integration fallback and cannot become the visible primary product palette.

The audit scope includes at minimum App startup/connect state, MameShell, MameBrowser, MachineFilterPanel, MachineList, MachineRightPanel, MachineDriverStatus, MameCatalogStatePanel, SoftwareBrowser, machine/software launch controls, and General Settings or configuration surfaces reachable from the parity flow.

### 5.5 Gameplay ownership invariant

Browser/navigation shortcuts must remain disabled whenever gameplay input owns the relevant keyboard surface. Fail-closed behavior for uncertain session ownership must be preserved.

## 6. Functional requirements

### RPR-001 — Invalidate stale detail and action state

- Introduce an explicit selection/detail generation or equivalent cancellation mechanism.
- Increment/invalidate the generation whenever selection changes, including transitions to null.
- Prevent stale responses from repopulating machine detail after selection clear/change.
- Clear machine-specific launch overrides and transient action state when identity changes.
- Ensure metadata refresh/import/query failure cannot resurrect prior detail.
- Ensure a stale failure cannot overwrite a newer successful machine detail state.
- Add deterministic tests with deliberately reordered promises.

Acceptance criteria:

- Select A, start detail A, clear selection, resolve A: UI remains unselected with no A actions/status/detail.
- Select A, start detail A, select B, resolve B then A: B remains authoritative.
- Select A, start detail A, select B, resolve A then B: transient A completion never becomes visible for B.
- Selection clear removes pending machine-specific launch overrides.

### RPR-002 — Reliable activation

- Double-click must activate the clicked machine even when its detail was not already loaded.
- Enter must activate the keyboard-selected machine under the same condition.
- Repeated activation must not launch a stale previously selected machine.
- If activation queues behind detail resolution, a later selection change must invalidate the old activation.
- Existing availability guards, BIOS semantics, software selection, and launch-error reporting must be preserved.

Acceptance criteria:

- First selection plus immediate double-click yields one activation for that machine.
- Keyboard move plus immediate Enter yields one activation for that machine.
- Activate A then immediately select B before A detail returns: A must not launch unless the activation had already crossed the authoritative launch boundary by design and that behavior is documented/tested.
- Unavailable machines remain non-launchable.

### RPR-003 — Remove generic Tauri default branding

- Remove the visible MAME Tauri Frontend pseudo-title from the default composition.
- Prevent equivalent generic rewrite branding from appearing in other default-visible surfaces.
- Keep any implementation/debug branding in non-default developer/debug contexts only.
- Add a negative regression test for the prohibited visible default branding.

### RPR-004 — Make driver/status the true bottom region

- MachineDriverStatus must be the bottom-most persistent region of the default machine browser composition.
- Remove the permanent project/global utility strip from below it.
- Preserve access to Configure Options, Audit, History, Collections, Diagnostics, Session, or similar tools through a compact secondary affordance, menu, command path, or non-default surface.
- Do not reintroduce a generic dashboard/tab row.
- MachineDriverStatus must continue to update with selection and provide a useful no-selection state.

Acceptance criteria:

- A rendered default shell has no persistent footer below the green driver/status region.
- History/Collections/Diagnostics are not permanently visible in the default browser chrome.
- All supported secondary surfaces remain reachable.

### RPR-005 — Complete explicit theme migration

- Audit all normal parity-relevant frontend CSS for Canvas, CanvasText, currentColor, generic light-surface assumptions, and host-dependent colors.
- Replace product styling with semantic --mame-* tokens.
- Extend the token set only where needed; avoid one-off literal color proliferation.
- Ensure hover, selected, focused, disabled, error, loading, and empty states are intentionally themed.
- Keep focus-visible contrast strong.

Acceptance criteria:

- Static audit rejects disallowed system-color product styling in parity-relevant files.
- Main browser, software browser, configuration path, and transient startup path remain recognizably MAME-themed under both host light and dark modes.

### RPR-006 — Software Browser parity

- Apply MAME navy/background, toolbar, text, selected-row, disabled-row, splitter, focus, and error semantics to Software Browser.
- Preserve dense row/list behavior.
- Keep software-part selection and Start behavior correct.
- Preserve all existing BIOS/launch semantics.
- Keep software errors inside the MAME surface.
- Add component tests for software selection and activation styling/behavior.

### RPR-007 — Deterministic selection visibility and scroll behavior

- Define when scroll position is preserved versus reset.
- Whenever an async query/filter/search chooses a selected item, ensure the selected row is visible.
- Avoid unnecessary scroll jumps when the selection is already visible.
- Preserve keyboard-driven native scrolling behavior.
- Test a list long enough to require scrolling.

Acceptance criteria:

- Searching/filtering from a deeply scrolled list cannot leave the selected result off-screen.
- Reconciliation after query completion produces one visible selected row when results are non-empty.
- Empty results clear selection and machine-specific detail correctly.

### RPR-008 — Keyboard navigation semantics

Machine list:

- Up/Down move one selectable row.
- PageUp/PageDown move approximately one visible viewport page rather than a hard-coded arbitrary count.
- Home/End semantics must be explicitly defined for the paged catalog model and implemented consistently.
- Enter activates the selected machine reliably.
- Escape/back behavior remains consistent.

Filter panel:

- Preserve listbox semantics and filter-to-machine focus movement.

Right panel:

- Images/Infos tabs implement standard keyboard tablist behavior.
- Left/Right changes the selected tab while the tablist owns focus.
- Focus management must not steal gameplay input.

Search:

- Existing search focus shortcut remains gameplay-owner gated.
- Typing/search reconciliation preserves the visibility invariant.

### RPR-009 — Artwork loading/error correctness

Represent at least these artwork states distinctly:

- no selected machine;
- loading slot discovery;
- no artwork slots;
- loading selected asset;
- asset ready;
- asset missing/unavailable;
- asset error.

Requirements:

- Do not show No image Available while a selected asset is merely loading.
- Normalize errors through the shared frontend error formatter.
- Prevent stale artwork responses from replacing a newer slot/machine selection.
- Preserve Snapshots/default category semantics.

### RPR-010 — Reduce asynchronous state coupling

Refactor only where necessary to make ownership/testability clear.

Preferred extraction boundaries include selected-machine/detail lifecycle, catalog query lifecycle, browser shortcut handling, browser persistence, and launch/activation orchestration.

Requirements:

- Each extracted unit must have an explicit input/output responsibility.
- No broad framework rewrite.
- No behavior change without corresponding tests.
- MameBrowser should become easier to reason about, not merely split into files while retaining implicit shared state.

### RPR-011 — Upgrade qualification tests

Retain useful static/source tripwires, but classify them as static guards rather than behavioral evidence.

Add executable component/state tests for at minimum:

- stale detail invalidation on null selection;
- A-to-B out-of-order detail completion;
- rapid double-click activation;
- immediate Enter activation;
- selection visibility after async search/filter;
- right-panel Left/Right keyboard switching;
- true bottom-region composition;
- absence of default-visible MAME Tauri Frontend branding;
- Software Browser token usage and normal selection flow;
- artwork loading versus missing state;
- gameplay-input ownership gating.

Where jsdom cannot model viewport behavior faithfully, isolate scroll/page calculations into pure tested helpers and add the strongest available DOM integration assertion around their use.

### RPR-012 — Visual qualification

- Preserve the existing textual parity reference.
- Update the manual checklist to cover the newly found regressions.
- Perform real rendered Tauri/WebView inspection where the execution environment supports it.
- At minimum inspect default shell, a selected available machine, an unavailable machine, Software List, Images/Infos, no-image state, metadata startup/import states, and a configuration surface.
- Pixel-baseline automation may remain deferred only with the existing concrete tooling rationale.
- A textual report may supplement but must not contradict source/rendered behavior.

### RPR-013 — Documentation reconciliation

- Add this remediation spec/TODO to the documentation index/README where appropriate.
- Preserve the original MTP TODO/final report as historical evidence.
- Explicitly state that the review reopened affected parity requirements.
- Correct current claims that imply no unresolved parity work.
- Record issue-to-original-MTP traceability.
- Never collapse completed subtasks out of the canonical remediation TODO; check them individually and append evidence instead.

## 7. Traceability to original parity work

| Review finding | Original MTP areas reopened |
| --- | --- |
| Stale selected-machine detail | MTP-006, MTP-008, MTP-009, MTP-010 |
| Activation race | MTP-006, MTP-009, MTP-010 |
| Visible MAME Tauri Frontend title | MTP-002 |
| Utility footer below green status | MTP-002, MTP-004, MTP-008, MTP-009 |
| Canvas/currentColor theme leakage | MTP-003, MTP-009, MTP-013 |
| Software Browser visual mismatch | MTP-003, MTP-009 |
| Search-selected row can be off-screen | MTP-006, MTP-010 |
| Fixed PageUp/PageDown and page-local Home/End | MTP-010 |
| Missing right-panel arrow navigation | MTP-007, MTP-010 |
| False no-image during load | MTP-007 |
| Source-string tests treated as behavior | MTP-010, MTP-012 |
| Documentation/closure overclaim | MTP-012, MTP-014, MTP-015 |

## 8. Validation strategy

### 8.1 Per-slice validation

Each implementation slice must run the applicable frontend/unit/component tests and exact-head CI before merge.

Changes affecting launch/catalog/Rust boundaries must also run applicable Rust/project/security workflows.

Changes affecting packaging/platform behavior must run applicable Linux, Windows, and macOS workflows when configured.

### 8.2 Exact-head rule

Qualification evidence must be tied to the exact candidate head SHA. A passing run from an older head does not qualify a later commit.

### 8.3 No source-tripwire substitution

Static source assertions may prove that a prohibited token/string is absent or a required token exists. They do not prove interactive behavior.

Behavioral completion requires executable state/component/integration evidence unless the environment makes that category impossible, in which case the TODO must record the precise limitation and the strongest available alternative.

### 8.4 Manual/rendered evidence

The final pass must include a rendered parity review, not solely source inspection.

The report must explicitly inspect:

- default startup shell;
- machine filter/list/right-panel geometry;
- selected row blue/yellow treatment;
- unavailable row treatment;
- green status as actual bottom persistent region;
- absence of generic Tauri branding;
- absence of permanent History/Collections/Diagnostics footer;
- Software Browser;
- Images and Infos;
- artwork loading and missing states;
- keyboard focus visibility;
- configuration/metadata states.

## 9. Documentation and evidence rules

The remediation TODO is a durable ledger.

- Never replace individual subtasks with a single all-subtasks-complete line.
- Mark each checkbox only when that exact subtask is satisfied.
- Append evidence beneath the relevant task.
- Record exact commit SHA and CI run IDs for qualification.
- If a subtask is deferred, leave its text present and record the reason plus the condition for revisiting it.
- If a requirement is superseded, identify the replacement requirement and rationale.
- A test passing is evidence only for the behavior the test actually exercises.

## 10. Completion rule

This remediation is complete only when all of the following are true:

1. Every subtask in docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md is checked complete, explicitly deferred with defensible rationale, superseded with traceability, or blocked by concrete user-required input.
2. The stale-detail race is covered by deterministic executable regression tests.
3. Double-click and Enter activation are covered by executable regression tests.
4. Default-visible MAME Tauri Frontend branding is absent.
5. The green driver/status region is the true bottom-most persistent default region.
6. Normal parity-relevant flows no longer use host system colors as the visible primary product palette.
7. Software Browser meets the explicit MAME visual/interaction contract.
8. Search/filter reselection keeps the selected row visible.
9. Keyboard page and right-panel navigation semantics are implemented and tested.
10. Artwork loading/missing/error states are distinct and tested.
11. Critical behavior is qualified with real interaction/state tests rather than source-string assertions alone.
12. Documentation accurately reflects remaining deviations and evidence.
13. A final rendered parity review is recorded.
14. The final implementation candidate passes all applicable exact-head CI.
15. The qualified candidate is merged through the normal gated path.
16. The remediation TODO is reloaded from promoted master and the promoted SHA is recorded.
17. Applicable post-merge master CI is verified before closure is claimed.
