# MTR-012 — Parity Test Evidence Inventory

**Repository:** `ekkus93/mame`  
**TODO:** `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md`  
**Reviewed head:** `25d7eff9ade8f703616400108aba4274d9c6c93d`

## Purpose

This inventory separates static/source tripwires from executable behavior coverage. Static tripwires remain useful for default-surface composition and theme leakage, but they are not used as the sole evidence for interaction behavior when a pure, component, state-machine, or platform test exists.

## Test classification

| File | Category | Evidence role |
| --- | --- | --- |
| `tauri/src/browser/machineSelectionIdentity.test.ts` | Pure state-machine/unit | Executable stale-detail and activation identity coverage for MTR-001 and MTR-002. |
| `tauri/src/browser/model.test.ts` | Pure unit | Executable browser request, filter, selection reconciliation, PageUp/PageDown, and Home/End behavior coverage for MTR-007 and MTR-008. |
| `tauri/src/browser/MachineList.visibility.test.ts` | Pure/unit with test doubles | Executable selected-row visibility and no-results transition coverage for MTR-007. |
| `tauri/src/browser/SoftwareBrowserComponent.test.tsx` | Component/server-render | Executable Software Browser row rendering, selected state, muted unsupported/partial state, and row metadata coverage for MTR-006. |
| `tauri/src/browser/rightPanelKeyboard.test.ts` | Pure unit | Executable right-panel Images/Infos arrow/clamp semantics for MTR-009. |
| `tauri/src/browser/artworkState.test.ts` | Pure unit | Executable artwork loading/missing/error state and stale request identity coverage for MTR-010. |
| `tauri/src/browser/ArtworkAssetFrame.test.tsx` | Component/server-render | Executable artwork loading, true missing, ready/error presentation coverage for MTR-010. |
| `tauri/src/browser/SoftwareBrowserParity.source.test.ts` | Static/source tripwire | Useful guard for Software Browser MAME palette tokens and wiring, but not sole interaction evidence. |
| `tauri/src/shell/mameShellComposition.source.test.ts` | Static/source tripwire | Useful guard for default footer absence, default branding absence, and secondary-tool reachability; must be paired with rendered review for visual claims. |
| GitHub Actions `Tauri project` workflow | Platform/workflow | Runs format, lint, typecheck, frontend tests, production build, Rust format/tests/clippy, and release qualification checks for exact implementation heads. |
| Packaging/security workflows | Platform/workflow | Cross-platform and security qualification evidence for exact implementation heads. |

## Reopened review items and evidence mapping

- **MTR-001 stale selected-machine detail:** `machineSelectionIdentity.test.ts` verifies detail request invalidation when selection clears, out-of-order A/B detail rejection, and stale activation rejection.
- **MTR-002 activation races:** `machineSelectionIdentity.test.ts` verifies rapid double-click activation, immediate Enter activation after keyboard selection, and rapid A activation rejected after B selection.
- **MTR-004 footer/default shell composition:** `mameShellComposition.source.test.ts` guards no persistent footer below `MachineDriverStatus` and secondary-tool reachability. This remains a source composition tripwire, not rendered visual evidence.
- **MTR-006 Software Browser:** `SoftwareBrowserComponent.test.tsx` gives component-level row rendering evidence; `SoftwareBrowserParity.source.test.ts` remains a theme/wiring tripwire.
- **MTR-007 selection visibility:** `MachineList.visibility.test.ts` and `model.test.ts` cover selected result visibility, no-results clearing, and selection reconciliation.
- **MTR-008 keyboard navigation:** `model.test.ts` covers viewport-aware page-step calculation, PageUp/PageDown movement, and current-fetched-page Home/End behavior.
- **MTR-009 right-panel keyboard:** `rightPanelKeyboard.test.ts` covers ArrowRight/ArrowLeft/clamp semantics. The production `MachineRightPanel` retains the `gameplayInputOwned` guard, which is also verified by exact implementation workflow coverage.
- **MTR-010 artwork state:** `artworkState.test.ts` and `ArtworkAssetFrame.test.tsx` cover loading vs missing state, formatted errors, and out-of-order stale response rejection.
- **Negative branding guard:** `mameShellComposition.source.test.ts` asserts the prohibited `MAME Tauri Frontend` phrase is absent from default shell/browser sources.
- **Negative host-system color guard:** `SoftwareBrowserParity.source.test.ts` and the MTR-005 theme scans guard against host-system product colors in parity-relevant surfaces.

## Static tripwire policy

Static/source tests are retained when they cheaply guard architectural or styling regressions that are difficult to express in jsdom. They are no longer treated as sufficient behavioral proof for user interactions. Each interaction claim above is supported by a pure, state-machine, component, or platform workflow test where technically available.

## Misleading-test review

The tests that previously looked like closure evidence only through source strings are now classified as static tripwires. They are retained only for narrow composition/theme invariants and are explicitly paired with executable tests or deferred rendered review where needed. No source tripwire is recorded in this inventory as sole evidence for interactive behavior.

## Known jsdom/WebView limitations

- Server-rendered component tests can assert ARIA/state/classes/text output, but they cannot validate WebView layout metrics or pixel-level density.
- Pure helper tests validate deterministic interaction rules, but they do not replace rendered WebView inspection for visual parity.
- The shell and theme source tripwires can catch prohibited strings/tokens, but they cannot prove the final rendered appearance.
- Rendered visual parity remains owned by MTR-013.

## Exact-head qualification references

The implementation sections MTR-001 through MTR-010 record exact push/PR workflow IDs for the commits that introduced the behavioral and component evidence. The latest promoted master before this inventory, `25d7eff9ade8f703616400108aba4274d9c6c93d`, contains all of those promoted tests and passed documentation validation after MTR-011. This MTR-012 inventory is documentation-only; applicable exact-head qualification is documentation validation for this branch plus the previously recorded exact-head frontend/platform workflows for the tested implementation changes.
