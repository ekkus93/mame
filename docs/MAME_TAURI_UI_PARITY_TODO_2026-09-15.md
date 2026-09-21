# MAME Tauri UI Parity TODO — 2026-09-15

**Repository:** `ekkus93/mame`  
**Spec:** `docs/MAME_TAURI_UI_PARITY_SPEC_2026-09-15.md`  
**Goal:** make the Tauri frontend look and behave like the original MAME UI closely enough that a normal user should not notice the frontend was rewritten in Tauri.  
**Prior hardening ledger:** `docs/MAME_TAURI_UI_POST_REMEDIATION_HARDENING_TODO_2026-09-15.md`

This TODO is the canonical backlog for original-MAME visual and interaction parity. The prior hardening TODO closed behavior/launch semantics; it did not close visual parity.

## Mandatory execution rules

- Do not redesign the app.
- Treat the original MAME UI as the product spec.
- Do not mark a task complete just because unit tests pass.
- Do not close the parity pass without visual evidence.
- Use exact-head CI evidence for each PR/candidate.
- After every merge, reload this TODO from promoted `master`, identify the next unchecked item, and continue.
- If screenshots/reference captures are unavailable in the implementation environment, create a textual visual parity report from the known reference details and mark screenshot-dependent subtasks blocked only if they cannot be completed without new user-provided assets.

## Reconciliation summary

MTP-000 through MTP-014 are complete. Their detailed implementation/evidence history is preserved in repository history through promoted master `9a7f1e7441f67709a3231289ed4517df151d2c87` and the referenced parity documents. The final visual evidence is `docs/MAME_TAURI_UI_PARITY_FINAL_REPORT_2026-09-21.md`. Screenshot pixel-baseline automation remains explicitly deferred because the hosted matrix lacks a stable cross-platform WebView screenshot harness and the Ralph source-write path is text-only; static/component tripwires, the textual visual contract, and real Tauri Xvfb smoke are the qualified fallback.

---

## MTP-000 — Baseline and source-of-truth reset

- [x] All subtasks complete and reconciled. Baseline mismatch, route/component tree, default-visible CSS, and generic-shell replacement target were identified before implementation.

## MTP-001 — Preserve reference visual evidence in repo-owned form

- [x] All subtasks complete and reconciled in `docs/MAME_TAURI_UI_PARITY_REFERENCE_2026-09-15.md`; binary screenshot commits are explicitly deferred with rationale and a complete textual visual contract is retained.

## MTP-002 — Replace generic default shell with original-MAME-like shell

- [x] All subtasks complete. The default-visible generic dashboard/tab shell was removed; the MAME-style browser is primary and secondary project surfaces remain accessible outside the default composition.

## MTP-003 — Implement explicit MAME palette tokens and remove system-color product styling

- [x] All subtasks complete. Explicit MAME palette tokens cover default-visible surfaces and regression tests prevent host system-color product styling from returning.

## MTP-004 — Match original layout geometry and density

- [x] All subtasks complete. Header/search, blue toolbar, left filters, dense machine list, right Images/Infos, splitters, and green status region follow the original spatial model.

## MTP-005 — Filter/category panel parity

- [x] All subtasks complete. Supported filters retain original-like ordering/behavior; Category and Custom Filter are visibly integrated deferred entries where authoritative support is not yet implemented.

## MTP-006 — Machine list visual and selection parity

- [x] All subtasks complete. Dense rows, blue/yellow selection, muted unavailable rows, scrolling/search/click/activation behavior, and prior BIOS launch semantics are preserved.

## MTP-007 — Right Images/Infos panel parity

- [x] All subtasks complete. Images/Infos, Snapshots, missing-image treatment, image/info density, scaling/padding, and no-selection behavior are reconciled.

## MTP-008 — Bottom status/driver region parity

- [x] All subtasks complete. The green selected-machine driver/status region follows selection and carries machine metadata rather than generic app diagnostics.

## MTP-009 — Original-like command/actions flow

- [x] All subtasks complete. Start, Start Empty, Configure Machine/Options, Software List, Audit, disabled/error behavior, software parts, and BIOS selection remain integrated into the MAME browser flow.

## MTP-010 — Keyboard and mouse behavior parity

- [x] All subtasks complete. Up/Down, PageUp/PageDown, Home/End, Enter, Escape/back, search focus, filters, row selection/activation, right-panel tabs, and gameplay-input ownership are covered by implementation/tests.

## MTP-011 — Startup, metadata, and empty-state parity

- [x] All subtasks complete. Not-configured, executable-unavailable, metadata checking/import/progress/failure/ready, empty catalog, and ROM-availability states remain inside the MAME-style shell and configured installs return to the machine query path.

## MTP-012 — Visual regression tripwires

- [x] All implementation/static/component tripwire subtasks complete.
- [x] Screenshot tests explicitly deferred: hosted CI has no stable cross-platform WebView pixel-baseline harness and the Ralph source-write path is text-only. The textual visual contract, final report, static/component tripwires, and real Tauri Xvfb smoke are the qualified fallback.

## MTP-013 — Linux desktop-environment compatibility preservation

- [x] All subtasks complete. The React/Tauri/WebView rendering boundary, GNOME/KDE and Wayland/X11 considerations, explicit host-independent theme, focus treatment, and practical verification sequence are documented and regression-tested.

## MTP-014 — Documentation and user-facing explanation cleanup

- [x] All subtasks complete. README/reference docs state the parity/Linux-compatibility goal, intentional deviations, local visual verification, screenshot capture/update procedure, and retirement of the generic dashboard product direction.

## MTP-015 — Final reconciliation and closure

- [x] Reconcile every task/subtask in this TODO as complete, explicitly deferred with rationale, superseded with rationale, or blocked with concrete user-required input.
- [x] Attach final visual evidence: `docs/MAME_TAURI_UI_PARITY_FINAL_REPORT_2026-09-21.md` is the complete textual manual parity report permitted by the execution rules when binary captures are unavailable.
- [ ] Qualify the exact final PR head through all applicable project/security/platform/documentation workflows.
- [ ] Merge only an exact-head-qualified candidate through the gated Ralph Bridge path.
- [ ] Reload this TODO from promoted `master` after merge and verify the promoted SHA.
- [ ] Verify applicable post-merge `master` CI before claiming closure.
- [x] Do not disable or stop any related scheduled work until this visual parity pass is actually closed.

**MTP-015 pre-merge evidence:** Final report attached. All prior task/subtask states are reconciled above. The closure candidate is documentation-only; exact-head qualification, gated merge, promoted-master reload, and post-merge CI evidence are intentionally recorded only after those operations actually occur.

---

## Completion rule

This parity effort is complete only after the remaining MTP-015 qualification/merge/promoted-master/post-merge checks are performed and recorded. The final reconciliation follow-up must include exact closure PR/head SHA, applicable CI run IDs, promoted master SHA, and post-merge CI evidence.
