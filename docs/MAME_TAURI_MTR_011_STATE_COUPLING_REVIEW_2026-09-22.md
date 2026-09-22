# MTR-011 — MameBrowser State Coupling Review

**Repository:** `ekkus93/mame`  
**TODO:** `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md`  
**Reviewed head:** `a19824882dd52fba0eb964654512e19331b19797`  
**Primary file:** `tauri/src/browser/MameBrowser.tsx` (`964` lines at review time)

## Goal

MTR-011 is a bounded coupling review, not a framework rewrite. The rule for this pass is to extract only boundaries that materially improve correctness or testability and to leave authoritative backend/catalog APIs unchanged unless a concrete defect requires changing them.

## Responsibility map

### Catalog query lifecycle

`MameBrowser.tsx` remains the orchestration point for catalog readiness, metadata import/refresh, debounced filters/search, backend query requests, pagination offset, and displayed-list export.

Boundaries already extracted:

- `tauri/src/browser/mameCatalogState.ts` owns metadata/catalog state derivation, catalog query eligibility, executable request mapping, availability notices, and range-label fallback.
- `tauri/src/browser/model.ts` owns browser request construction, value-required filter checks, selection reconciliation, keyboard index movement, and deterministic page-step calculation.

Intentional retained responsibility:

- The component keeps the actual React effects because they coordinate UI hydration, catalog state, query sequencing, and selected-machine invalidation in one place. Splitting this further without a new correctness defect would move lifecycle coupling rather than reduce it.

### Selected-machine/detail lifecycle

`MameBrowser.tsx` owns the selected row state and renders selected-machine-dependent surfaces, but selected-machine identity and stale async safety are no longer implicit component-local conventions.

Boundaries already extracted:

- `tauri/src/browser/machineAsyncIdentity.ts` owns the selected-machine async identity model, including detail and activation generations.
- `tauri/src/browser/machineSelectionIdentity.test.ts` verifies stale detail rejection, detail clearing, and activation invalidation scenarios.

Intentional retained responsibility:

- `selectMachine` stays inside `MameBrowser.tsx` because it coordinates React state (`selected`), refs (`selectedRef`, generation refs), and machine-specific transient state (`pendingLaunchOverrides`, `launchState`). Extracting it into a hook would require threading the same setters and refs through another layer without improving the existing executable identity coverage.

### Activation and launch orchestration

`MameBrowser.tsx` owns actual launch dispatch and user-visible launch status because launch behavior crosses selected-machine identity, machine detail readiness, launch overrides, session callback delivery, and in-surface error rendering.

Boundaries already extracted:

- `machineAsyncIdentity.ts` owns the activation-generation commit guard.
- Existing activation tests cover double-click/Enter activation behind detail loading and invalidation after selection changes.

Intentional retained responsibility:

- The launch command remains in `MameBrowser.tsx` so BIOS omission, Start Empty semantics, launch overrides, and session-start callbacks remain in a single authoritative path. No backend API change is required for MTR-011.

### Keyboard shortcuts

`MameBrowser.tsx` owns cross-region shortcut orchestration because it coordinates search focus, list focus, narrow-details state, selected-machine launch, and gameplay-input ownership.

Boundaries already extracted:

- `tauri/src/browser/model.ts` owns pure list navigation and page-step semantics.
- `tauri/src/browser/machineListVisibility.ts` owns selected-row visibility reconciliation.
- `tauri/src/browser/rightPanelKeyboard.ts` owns right-panel primary tab arrow semantics.

Intentional retained responsibility:

- The global `keydown` listener remains component-local because it depends on live refs, current detail state, current software mode, and `gameplayInputOwned`. Moving it out without a dedicated interaction harness would make ownership less explicit.

### Persisted browser state

`MameBrowser.tsx` owns hydration and persistence effects for saved UI state because those effects coordinate backend storage with current filter/search/right-panel/artwork selections.

Boundaries already extracted:

- Backend persistence API types and commands remain in `tauri/src/backend/mameUiState.ts`.
- Persisted machine selection reconciliation flows through `reconcileMachineSelection` in `model.ts` before being committed to current UI state.

Intentional retained responsibility:

- Hydration/persistence effects remain in `MameBrowser.tsx` for now because they are the only place all persisted UI fields and their corresponding React setters are available. There is no behavioral defect requiring a new persistence hook in this remediation pass.

## Extracted boundaries that materially improved correctness/testability

This remediation track has already extracted or relied on these correctness boundaries:

- `machineAsyncIdentity.ts`: explicit stale-detail and activation generations.
- `model.ts`: pure request, filter, selection reconciliation, list navigation, and page-size logic.
- `machineListVisibility.ts`: deterministic selected-row visibility behavior.
- `rightPanelKeyboard.ts`: deterministic right-panel Images/Infos arrow behavior.
- `artworkState.ts`: explicit loading/missing/ready/error artwork asset state and stale request identity checks.
- `ArtworkAssetFrame.tsx`: isolated rendered artwork-state output for component testing.
- `SoftwareBrowserComponent.test.tsx` coverage around exported Software Browser row rendering.

These extractions reduce implicit coupling where the review identified concrete correctness or testability problems. Further splitting of `MameBrowser.tsx` is intentionally deferred unless a new defect appears, because additional movement would mostly pass setters/refs through a hook facade while leaving the same lifecycle coupling.

## Backend and architecture constraints

- No backend/catalog API changes are required for MTR-011.
- The Tauri/WebView architecture is unchanged.
- No framework rewrite is introduced.
- No files were moved solely to reduce line count.
- No behavior-changing refactor is introduced by this review document.

## Remaining intentional responsibilities in `MameBrowser.tsx`

`MameBrowser.tsx` remains the composition/orchestration component for:

- catalog readiness and query dispatch;
- selected machine and detail rendering;
- launch/session orchestration;
- cross-region keyboard shortcuts;
- persisted browser UI state;
- layout-level composition of toolbar, filter panel, list, right panel, and driver/status region.

That size is still high, but at this point the remaining responsibilities are cohesive for a top-level browser surface. The correctness-sensitive state machines and pure interaction rules are covered outside the component, so further size-only decomposition is intentionally not part of this remediation pass.

## Qualification expectation

Because this review is documentation-only and records existing extracted correctness boundaries, applicable qualification is documentation validation plus the existing exact-head frontend/platform evidence already recorded for the implementation sections that introduced the extracted state machines and tests.
