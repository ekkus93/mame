# MAME Tauri UI Post-Closure Remediation TODO — 2026-09-14

**Repository:** `ekkus93/mame`  
**Recovered baseline:** `master` at `b49a2b1652dbf929a71698038a917ff9abeb746f`  
**Related milestone:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_TODO_2026-09-14.md`  
**Closure record:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md`

This ledger captures defects found immediately after the MAME-style Tauri UI milestone was closed. The original requested post-closure TODO path was not present on the promoted `master`, so this file reconstructs the remediation work from the exact closed implementation and its stated invariants. The remediation does not reopen deliberately deferred parity features; it fixes correctness, lifecycle, accessibility, and regression gaps in behavior already claimed by the closed milestone.

## MUR-001 — Make session/gameplay-input ownership authoritative and event-driven

- [ ] Remove the browser-local polling source of truth for gameplay-input ownership.
- [ ] Make `MameShell` own fail-closed gameplay-input ownership from the authoritative session snapshot and lifecycle events.
- [ ] Pass ownership into the machine browser and software browser so both suppress global shortcuts while native MAME owns gameplay input.
- [ ] Update ownership immediately when machine or software launch returns a session, without waiting for a later focus event.
- [ ] Restore browser shortcut ownership immediately on exited/failed/crashed lifecycle events.
- [ ] Preserve fail-closed behavior when initial session discovery or lifecycle-listener setup fails.
- [ ] Add/extend regression coverage so software-mode shortcuts cannot bypass gameplay ownership.

## MUR-002 — Make software launch revalidation exact rather than fuzzy/paginated

- [ ] Stop revalidating a selected software item through generic substring search with `limit=1`.
- [ ] Add a bounded parser/backend path that locates the exact software short name while still validating the complete MAME software-list XML structure.
- [ ] Preserve multipart validation against the exact item returned by authoritative MAME metadata.
- [ ] Add a regression fixture where an earlier fuzzy match would otherwise hide the exact selected item.

## MUR-003 — Clear stale artwork errors across category transitions

- [ ] Clear asset-read errors whenever artwork category/slot selection changes.
- [ ] Ensure switching from a failed asset read to a category with no configured asset shows the normal missing-artwork state, not the prior category's error.
- [ ] Preserve request-sequence guards so stale asset responses cannot overwrite a newer category.
- [ ] Add regression coverage for the error-to-missing transition where practical.

## MUR-004 — Refresh selected-machine state from authoritative query results

- [ ] Replace retained selected-row objects with the fresh matching row returned by each successful catalog query.
- [ ] Ensure catalog/availability refreshes do not leave stale selected-machine metadata in browser state.
- [ ] Refresh selected-machine detail when the authoritative result generation/revision changes while preserving selection by short name.
- [ ] Preserve deterministic reselection when the selected machine disappears from a result set.
- [ ] Add regression coverage for selection-refresh behavior in pure browser model helpers where practical.

## MUR-005 — Correct software-list selection semantics and roving keyboard focus

- [ ] Expose the software result list with listbox/option selection semantics consistent with the machine browser.
- [ ] Use a single roving tab stop tied to the selected software row.
- [ ] Preserve pointer selection, Enter activation, and Up/Down/Home/End/Page navigation behavior.
- [ ] Ensure selection state is represented with `aria-selected`, not `aria-current`.
- [ ] Extend static regression coverage for the software list accessibility contract.

## MUR-006 — Apply typed BIOS selection consistently to Start Empty

- [ ] Extend the typed machine-launch request with an optional BIOS identifier rather than adding arbitrary arguments.
- [ ] Validate any selected BIOS against authoritative MAME BIOS choices in Rust before launch.
- [ ] Pass the software browser's selected BIOS through the Start Empty path.
- [ ] Preserve existing machine-launch callers by defaulting BIOS to MAME's default when none is supplied.
- [ ] Add Rust validation tests for invalid/unavailable BIOS selections and frontend typing coverage.

## MUR-007 — Qualification, documentation, and closure reconciliation

- [ ] Extend `scripts/tauri/test-mame-ui-reproduction.py` to guard the remediated ownership/software-list contracts and include this TODO in the milestone regression.
- [ ] Ensure the main CI sparse checkout includes this remediation ledger if the regression reads it.
- [ ] Reconcile every task/subtask in this ledger as complete, explicitly deferred with rationale, superseded with rationale, or not required.
- [ ] Confirm no ambiguous unchecked item remains before closure.
- [ ] Qualify the exact final PR head through all applicable Tauri project/security/platform/documentation workflows.
- [ ] Merge only an exact-head-qualified candidate.
- [ ] Verify the promoted `master` SHA and applicable post-merge CI before claiming remediation closure.

## Completion rule

The remediation is complete only when every item above has an explicit disposition, the exact final candidate is green in all applicable required workflows, the qualified candidate is merged to `master`, and the promoted `master` SHA is rechecked. A green ancestor does not qualify a changed candidate.
