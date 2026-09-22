# MTR-012 — Test Evidence Inventory

**Repository:** `ekkus93/mame`  
**TODO:** `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md`  
**Reviewed head:** `25d7eff9ade8f703616400108aba4274d9c6c93d`

## Purpose

MTR-012 replaces false-positive parity qualification with a traceable inventory of real behavioral evidence, while keeping static/source tripwires only where they are useful as regression guards and not as the sole proof of user-visible interaction behavior.

## Classification

| Evidence file | Classification | Coverage role |
| --- | --- | --- |
| `tauri/src/browser/machineSelectionIdentity.test.ts` | Pure unit/state | Stale selected-detail invalidation, activation generation guards, rapid double-click/Enter activation identity, and queued activation rejection after selection changes. |
| `tauri/src/browser/model.test.ts` | Pure unit/model | Browser request construction, filter value requirements, selection reconciliation, viewport-aware page movement, and current-fetched-page Home/End semantics. |
| `tauri/src/browser/MachineList.visibility.test.ts` | Pure unit with DOM-adapter test doubles | Selected-row visibility reconciliation, long-list replacement near beginning/end, no-results selection clearing, absent identity no-op, and nearest-scroll behavior. |
| `tauri/src/browser/rightPanelKeyboard.test.ts` | Pure unit/model | Images/Infos ArrowLeft/ArrowRight semantics and clamp/return-to-list edge behavior. |
| `tauri/src/browser/ArtworkAssetFrame.test.tsx` | Server-rendered component | Loading artwork state, true missing-artwork state, and formatted error rendering without false `No image Available` copy while loading. |
| `tauri/src/browser/artworkState.test.ts` | Pure unit/state | Loading-vs-missing derivation, shared formatter wiring for artwork errors, and out-of-order artwork request rejection. |
| `tauri/src/browser/SoftwareBrowserComponent.test.tsx` | Server-rendered component | Software row selected ARIA/class state, muted unsupported treatment, support metadata, and row rendering shape. |
| `tauri/src/browser/SoftwareBrowserParity.source.test.ts` | Static/source tripwire plus theme scan | Software Browser MAME palette tokens, dense row styling hooks, prohibited host-system color strings, and source-level wiring for launch/BIOS/software-part/error/back behavior. |
| `tauri/src/shell/mameShellComposition.source.test.ts` | Static/source tripwire | Permanent-footer removal, secondary-tool reachability, and negative default-visible `MAME Tauri Frontend` branding guard. |
| `tauri/src/browser/gameplayInputOwnership.source.test.ts` | Static/source tripwire | Guard that top-level browser, right-panel tabs, and Software Browser shortcut handlers remain gated when gameplay owns input. |

## Behavioral evidence added by remediation area

- **MTR-001 stale detail:** covered by `machineSelectionIdentity.test.ts` with executable state regressions for selection clear and out-of-order A/B detail completions.
- **MTR-002 activation races:** covered by `machineSelectionIdentity.test.ts` with executable state regressions for rapid double-click, immediate Enter, and activation invalidation after selecting a different machine.
- **MTR-004 default shell/footer composition:** covered primarily by `mameShellComposition.source.test.ts`; this remains a static composition tripwire because the rendered inspection is explicitly reserved for MTR-013.
- **MTR-006 Software Browser:** covered by `SoftwareBrowserComponent.test.tsx` for component row output plus `SoftwareBrowserParity.source.test.ts` for theme/static wiring tripwires. Rendered Software List inspection remains deferred to MTR-013.
- **MTR-007 selection visibility:** covered by `MachineList.visibility.test.ts` and `machineListVisibility.ts` helper behavior, including long-list and empty-result regressions.
- **MTR-008 keyboard navigation:** covered by `model.test.ts` for page-size calculation, PageUp/PageDown, and current-page Home/End semantics.
- **MTR-009 right-panel keyboard:** covered by `rightPanelKeyboard.test.ts` and the promoted right-panel component assertions recorded in the MTR-009 TODO evidence.
- **MTR-010 artwork state:** covered by `artworkState.test.ts` and `ArtworkAssetFrame.test.tsx` for loading/missing/error/out-of-order state separation.
- **Gameplay input ownership:** covered by `gameplayInputOwnership.source.test.ts` to guard shortcut ownership in the browser, right panel, and Software Browser surfaces.
- **Branding/theme negative guards:** covered by `mameShellComposition.source.test.ts` for default-visible `MAME Tauri Frontend` and by `SoftwareBrowserParity.source.test.ts` for disallowed host-system color strings on the Software Browser surface. Earlier MTR-005 evidence records the broader parity stylesheet theme scan.

## Static tripwire limitations

Static/source tests remain useful for guarding prohibited strings, composition order, palette-token usage, and shortcut-owner wiring, but they are not treated as proof of rendered interaction behavior. The rendered visual/density/focus review remains assigned to MTR-013. jsdom/WebView gaps remain unavoidable for pixel density, actual browser focus painting, native WebView scrollbar behavior, and screenshot/pixel-baseline validation unless a GUI/WebView-capable validation runner supplies rendered artifacts.

## Misleading-test cleanup assessment

No existing source/static test is marked as the sole behavioral proof for stale detail, activation, selected-row visibility, keyboard page semantics, right-panel tab semantics, or artwork loading/missing/error behavior. Those areas now have pure state, helper, or server-rendered component coverage. The remaining source/static tests are retained and classified as tripwires only.

## Qualification expectation

MTR-012 changes one source tripwire test, adds this inventory, and reconciles the TODO. The candidate must pass the complete frontend/unit/component suite through the Tauri project workflow on the exact candidate head. Documentation validation should also pass for the inventory and TODO update.
