# MAME Tauri Browser responsibility map — 2026-09-22

This note records the MTR-011 review of `tauri/src/browser/MameBrowser.tsx`. The goal is not a framework rewrite: extraction is justified only when it makes lifecycle identity, correctness, or executable testing clearer.

## Current ownership boundaries

| Responsibility | Current owner | Lifecycle identity / invariant | Disposition |
| --- | --- | --- | --- |
| Catalog metadata readiness | `MameBrowser` orchestration plus `mameCatalogState.ts` | `catalogSequence` rejects stale metadata work | Keep orchestration in browser; pure state decisions already extracted |
| Catalog query lifecycle | `MameBrowser` plus `model.ts` | `querySequence` identifies the active query; `buildMameBrowserRequest` and `reconcileMachineSelection` are pure | Keep effects local; query construction/reconciliation remain executable model boundaries |
| Selected machine | `MameBrowser` | `selectedRef` is synchronous identity; changing short name invalidates detail and activation generations | Keep one authoritative selection owner |
| Machine detail lifecycle | `MameBrowser` | `detailSequence` prevents stale detail completion from becoming current | Keep adjacent to selected-machine orchestration until a component harness warrants a hook extraction |
| Activation / launch | `MameBrowser` | `activationSequence` invalidates launch work when selection identity changes | Keep with selected-machine owner; backend launch APIs remain authoritative |
| Keyboard navigation | Browser coordination plus `model.ts`, `rightPanelKeyboard.ts`, and list/filter components | gameplay-input ownership gates browser shortcuts; pure movement/tab decisions are extracted | No additional abstraction required for correctness |
| Persisted browser state | `MameBrowser` using typed `mameUiState` backend | hydration is cancellation guarded; persisted panel/artwork/search/filter state is applied centrally | Keep centralized so restore order remains explicit |
| Artwork asset lifecycle | `MachineRightPanel` / `artworkState.ts` | discovery and asset request generations reject stale completions | Correctly outside `MameBrowser`; MTR-010 owns executable artwork coverage |
| Software browser | `SoftwareBrowser` | contextual sub-flow owns software query/selection/activation and Back/Escape | Keep isolated from machine-browser state |

## Coupling review

`MameBrowser.tsx` remains large because it is the composition/orchestration boundary for a stateful desktop browser, but the correctness-sensitive calculations are already split into focused modules (`model.ts`, `mameCatalogState.ts`, `rightPanelKeyboard.ts`) and contextual surfaces (`MachineRightPanel`, `SoftwareBrowser`, list/filter components). Moving effects into new files solely to reduce line count would hide sequencing relationships without reducing state coupling.

The selected-machine, detail, and activation generations intentionally live together: a selection identity change must synchronously invalidate both stale detail and stale activation work. Persisted state also remains at the browser boundary because hydration establishes initial filter, machine, right-panel, and artwork identity together. Backend/catalog API contracts are unchanged.

## Remaining intentional responsibilities

After remediation, `MameBrowser` intentionally remains responsible for: top-level catalog readiness/query orchestration; authoritative selected-machine identity; detail and launch sequencing; browser-level focus/keyboard coordination; persistence hydration/write-back; and composition of machine, software, right-panel, and secondary-action surfaces. Pure calculations and independently meaningful contextual UI remain outside it.

A future extraction such as `useMachineSelectionDetail` should be undertaken only with executable component/integration tests that prove stale-result and identity semantics before and after extraction. MTR-012 is the appropriate qualification gate for that work; MTR-011 does not introduce a speculative hook or change authoritative backend/catalog APIs.
