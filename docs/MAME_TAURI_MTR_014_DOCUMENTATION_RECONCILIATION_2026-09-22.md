# MTR-014 — Documentation reconciliation

**Repository:** `ekkus93/mame`  
**Canonical remediation ledger:** `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md`  
**Companion spec:** `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_SPEC_2026-09-21.md`  
**Historical parity ledger:** `docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md`  
**Historical parity report:** `docs/MAME_TAURI_UI_PARITY_FINAL_REPORT_2026-09-21.md`

## Status and historical record

The 2026-09-21 post-closure review reopened affected original-MAME visual and interaction parity requirements. The original MTP ledger and final parity report are preserved unchanged as historical evidence; they describe the state that was believed closed before the review. They must not be read as overriding the later remediation ledger while any MTR item remains open.

The canonical current status is the MTR ledger named above. Claims in older documentation that there is no unresolved parity work are historical closure claims, not the current remediation status. Final closure is valid only after MTR-000 through MTR-015 are individually reconciled on promoted master.

## Traceability

| Remediation | Original parity area | Reopened requirement |
| --- | --- | --- |
| MTR-001 | machine selection/detail lifecycle | stale detail invalidation and machine-specific transient state |
| MTR-002 | machine activation/launch | immediate double-click/Enter activation identity and races |
| MTR-003 | shell/header visual parity | remove default-visible generic Tauri pseudo-title |
| MTR-004 | shell/footer/status composition | make green machine driver/status region the bottom persistent region |
| MTR-005 | palette/theme parity | remove host-system product colors from parity surfaces |
| MTR-006 | Software Browser | dense MAME palette, selection, focus, and interaction parity |
| MTR-007 | machine list | selected-row visibility after asynchronous result replacement |
| MTR-008 | machine-list keyboard navigation | viewport-aware PageUp/PageDown and documented Home/End semantics |
| MTR-009 | right panel | Images/Infos tab keyboard and focus semantics |
| MTR-010 | artwork | explicit loading/missing/ready/error lifecycle and stale-response rejection |
| MTR-011 | browser architecture | reduce implicit lifecycle coupling only where correctness/testability improves |
| MTR-012 | qualification | replace source-string behavioral claims with executable state/component evidence |
| MTR-013 | rendered parity | re-qualify the reopened visual issues through the strongest available Tauri/WebView path |
| MTR-014 | documentation | reconcile historical closure claims and current behavior contracts |
| MTR-015 | closure | exact-head full qualification, promotion, and post-merge verification |

## Final keyboard semantics

Machine-list `Up` and `Down` move one row within the fetched result page. `PageUp` and `PageDown` use the live viewport height and measured row height to calculate page movement while retaining one row of context; the old arbitrary fixed ten-row jump is not the contract. `Home` and `End` intentionally target the first and last row of the **currently fetched backend page**, not the first/last row of the full matching catalog. This preserves backend pagination authority and avoids pretending that an unloaded catalog boundary is locally navigable. `Enter` activates the currently selected machine through the MTR-002 identity contract. Escape/back behavior remains contextual. Browser shortcuts fail closed while gameplay input owns the surface.

## Secondary tools after footer removal

`MachineDriverStatus` is the final persistent region in the default machine browser. Configure Options, Audit, History, Collections, Diagnostics, and Session/session status remain reachable through the compact `More` secondary-tools affordance or their contextual action path; they are not restored as permanent default footer chrome. Backend/global diagnostics stay out of the green selected-machine driver/status region.

## Intentional visual deviations

The bounded intentional deviations remain those documented by the product UI contract: React/Tauri/WebView renders the selector while MAME remains a separately supervised native process; project-only secondary surfaces remain behind secondary actions; Category/Custom Filter positions may precede full data/authoring support; narrow layouts use an explicit Details affordance; project Icon/System Image artwork extensions coexist with canonical MAME categories; and stable cross-platform screenshot pixel-baseline CI remains deferred while the current tooling lacks a binary WebView screenshot path.

MTR-013 records the strongest currently available rendered/runtime evidence in `docs/MAME_TAURI_MTR_013_RENDERED_VISUAL_REVIEW_2026-09-22.md`. It explicitly distinguishes successful Tauri/WebView window smoke evidence and textual review from unavailable pixel/screenshot evidence.

## Qualification evidence categories

Evidence is classified deliberately:

- **Static/source tripwire:** narrow guards for prohibited strings, composition invariants, and disallowed theme tokens. These are useful regression alarms but are not behavioral proof.
- **Pure unit/state-machine:** deterministic lifecycle, navigation, visibility, keyboard, and artwork-state tests.
- **Component/render:** executable component/server-render tests for rendered state and callbacks where a full WebView is unnecessary.
- **Integration/platform:** Tauri project, security, and platform packaging workflows, including the Linux Tauri development-window smoke path.
- **Rendered visual review:** MTR-013's runtime-backed textual visual review under the present screenshot limitation; it does not claim pixel-baseline evidence.

Behavioral completion must not be justified by source-string assertions alone. Exact-head workflow evidence must correspond to the candidate being promoted.

## Documentation validation

This file is additive. It does not delete, condense, or rewrite the historical MTP evidence or the historical final report. The canonical MTR ledger retains every individual checkbox and receives exact evidence as work is reconciled. Documentation/build validation for this reconciliation is required on the exact PR head before promotion.
