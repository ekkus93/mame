# MTR-014 — Documentation Reconciliation

**Repository:** `ekkus93/mame`  
**TODO:** `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md`  
**Reconciliation base:** `792f440740ff977c8ca7074e3856a4c9f03ff5b4`

## Purpose

MTR-014 reconciles the public/documentation story after the 2026-09-21 post-closure review reopened a bounded set of original-MAME visual and interaction parity claims. It preserves the original parity TODO and final report as historical evidence while making the reopened remediation track discoverable and auditable.

## Documentation index update requirement

The README must identify both tracks:

- the original parity track: `docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md`, `docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md`, `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md`, and `docs/MAME_TAURI_UI_PARITY_FINAL_REPORT_2026-09-21.md`;
- the reopened remediation track: `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_SPEC_2026-09-21.md`, `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md`, `docs/MAME_TAURI_MTR_012_TEST_EVIDENCE_INVENTORY_2026-09-22.md`, `docs/MAME_TAURI_MTR_013_RENDERED_VISUAL_REVIEW_2026-09-22.md`, and this reconciliation document.

The original final report is intentionally not rewritten to erase its prior closure claim. It remains historical evidence for the state of the repo at the time it was written. Current status must be read from the MTR remediation ledger until MTR-015 closes.

## Reopened requirements

The post-closure review reopened the following evidence areas:

- stale selected-machine detail invalidation;
- double-click and Enter activation race handling;
- default-visible generic Tauri branding removal;
- default footer/driver-status composition;
- explicit MAME palette migration;
- Software Browser visual and interaction parity;
- selected-row visibility and scroll reconciliation;
- keyboard PageUp/PageDown/Home/End semantics;
- right-panel Images/Infos keyboard semantics;
- artwork loading/missing/ready/error separation;
- MameBrowser state-coupling review;
- replacement of false-positive/static-only qualification with classified executable evidence;
- rendered visual re-qualification under the strongest available WebView validation path;
- final documentation and closure reconciliation.

## Traceability to original MTP areas

| Remediation item | Original parity area / reopened claim |
| --- | --- |
| MTR-000 | Reopened the original MTP closure and established the exact baseline. |
| MTR-001 | Selected-machine detail lifecycle and current-selection identity. |
| MTR-002 | Machine activation, double-click, Enter, Start/Start Empty, launch error handling. |
| MTR-003 | Default shell/title/header branding and generic Tauri rewrite branding removal. |
| MTR-004 | Bottom driver/status region and removal of permanent utility/footer chrome. |
| MTR-005 | MAME palette, host-theme leakage, selected/focus/disabled/error styling. |
| MTR-006 | Software Browser styling, selection, activation, BIOS, software-part, Back/Escape behavior. |
| MTR-007 | Machine-list selected-row visibility and result-replacement scrolling. |
| MTR-008 | Machine-list keyboard page movement and current-page Home/End semantics. |
| MTR-009 | Right-panel Images/Infos tab keyboard and focus semantics. |
| MTR-010 | Artwork asset loading, missing, ready, stale response, and error states. |
| MTR-011 | MameBrowser responsibility map and correctness-oriented extraction boundaries. |
| MTR-012 | Test evidence classification; static tripwires versus executable behavior coverage. |
| MTR-013 | Rendered/textual visual re-qualification and screenshot-artifact limitation. |
| MTR-014 | Documentation index, historical preservation, and final semantics documentation. |
| MTR-015 | Final full qualification and promoted-master closure. |

## Final keyboard semantics

Machine-list PageUp/PageDown are viewport-aware. The UI calculates a page step from the live list viewport height and measured row height, retaining one row of context. Up/Down remain row-by-row within the currently loaded list.

Home and End intentionally target the current fetched catalog page, not the full matching catalog. The explicit Previous/Next pager remains the authority for crossing backend page boundaries. This avoids false full-result semantics and preserves bounded catalog pagination.

Right-panel Images/Infos keyboard semantics clamp instead of wrapping: ArrowRight moves Images to Infos, ArrowLeft moves Infos to Images, ArrowRight at Infos stays on Infos, and ArrowLeft at Images returns focus toward the machine list.

## Secondary tools location

Project-only secondary tools were moved out of permanent footer chrome. Configure Options, Audit, History, Collections, Diagnostics, and Session remain reachable through the compact `More` utility menu and contextual action buttons. The green `MachineDriverStatus` region remains the bottom-most persistent default browser region and carries selected-machine status rather than global diagnostics.

## Intentional remaining visual deviations

The documented intentional deviations remain bounded:

- the selector is implemented in React/Tauri/WebView rather than the original native selector;
- project-only secondary tools remain behind secondary actions;
- Category and Custom Filter positions remain represented while full data/authoring support is deferred;
- narrow desktop windows use a Details affordance;
- project Icon/System Image artwork categories coexist with canonical MAME categories;
- screenshot/pixel-baseline capture remains deferred when Ralph/CI do not expose binary WebView screenshot artifacts.

## Test-category distinctions

The project now distinguishes:

- **static/source tripwires** for prohibited strings, composition order, token usage, and shortcut-owner wiring;
- **pure unit/state tests** for selection identity, activation generation, request construction, keyboard movement, and artwork request identity;
- **server-rendered component tests** for Software Browser and artwork state output;
- **platform/workflow evidence** for Tauri build, development-window smoke, packaging, security, and documentation validation.

Static/source tripwires are no longer treated as sole proof of interactive behavior. Rendered visual parity remains separately documented in MTR-013 and final closure remains in MTR-015.

## Preservation rule

Do not delete or rewrite the original MTP TODO/final report to hide the earlier closure. Keep this remediation additive: the historical documents explain what was claimed, and the MTR ledger explains what was reopened, fixed, re-qualified, and finally closed.
