# MAME Tauri UI Post-Closure Remediation TODO — 2026-09-14

**Repository:** `ekkus93/mame`  
**Recovered baseline:** `master` at `b49a2b1652dbf929a71698038a917ff9abeb746f`  
**Related milestone:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_TODO_2026-09-14.md`  
**Closure record:** `docs/MAME_TAURI_MAME_UI_REPRODUCTION_CLOSURE_2026-09-15.md`

This ledger captures defects found immediately after the MAME-style Tauri UI milestone was closed. The original requested post-closure TODO path was not present on the promoted `master`, so this file reconstructs the remediation work from the exact closed implementation and its stated invariants. The remediation does not reopen deliberately deferred parity features; it fixes correctness, lifecycle, accessibility, and regression gaps in behavior already claimed by the closed milestone.

## MUR-001 — Make session/gameplay-input ownership authoritative and event-driven

- [x] Remove the browser-local polling source of truth for gameplay-input ownership.
- [x] Make `MameShell` own fail-closed gameplay-input ownership from the authoritative session snapshot and lifecycle events.
- [x] Pass ownership into the machine browser and software browser so both suppress global shortcuts while native MAME owns gameplay input.
- [x] Update ownership immediately when machine or software launch returns a session, without waiting for a later focus event.
- [x] Restore browser shortcut ownership immediately on exited/failed/crashed lifecycle events.
- [x] Preserve fail-closed behavior when initial session discovery or lifecycle-listener setup fails.
- [x] Add/extend regression coverage so software-mode shortcuts cannot bypass gameplay ownership. The milestone regression now requires the shell-owned ownership prop and the software-browser ownership gate.

## MUR-002 — Make software launch revalidation exact rather than fuzzy/paginated

- [x] Stop revalidating a selected software item through generic substring search with `limit=1`.
- [x] Add a bounded parser/backend path that locates the exact software short name while still validating the complete MAME software-list XML structure. `parse_software_item` streams the complete document and retains at most the exact matching item.
- [x] Preserve multipart validation against the exact item returned by authoritative MAME metadata. `launch_library_software` now runs the existing part-required/part-membership checks against `parse_software_item` output.
- [x] Add a regression fixture where an earlier fuzzy match would otherwise hide the exact selected item. The fixture proves fuzzy `archon` returns `archon2` first while exact lookup still returns `archon`.

## MUR-003 — Clear stale artwork errors across category transitions

- [x] Clear asset-read errors whenever artwork category/slot selection changes.
- [x] Ensure switching from a failed asset read to a category with no configured asset shows the normal missing-artwork state, not the prior category's error.
- [x] Preserve request-sequence guards so stale asset responses cannot overwrite a newer category.
- [x] Add regression coverage for the error-to-missing transition where practical. The maintained static milestone regression guards the error reset immediately before the no-descriptor branch; the project still does not carry a React DOM harness solely for this surface.

## MUR-004 — Refresh selected-machine state from authoritative query results

- [x] Replace retained selected-row objects with the fresh matching row returned by each successful catalog query.
- [x] Ensure catalog/availability refreshes do not leave stale selected-machine metadata in browser state.
- [x] Refresh selected-machine detail when the authoritative result generation/revision changes while preserving selection by short name. A successful refreshed page replaces the selected object with its fresh authoritative row, retriggering the existing selected-detail request without changing the short-name selection.
- [x] Preserve deterministic reselection when the selected machine disappears from a result set.
- [x] Add regression coverage for selection-refresh behavior in pure browser model helpers where practical. `model.test.ts` covers fresh-row replacement, preferred fallback, first-row fallback, and empty results.

## MUR-005 — Correct software-list selection semantics and roving keyboard focus

- [x] Expose the software result list with listbox/option selection semantics consistent with the machine browser.
- [x] Use a single roving tab stop tied to the selected software row.
- [x] Preserve pointer selection, Enter activation, and Up/Down/Home/End/Page navigation behavior.
- [x] Ensure selection state is represented with `aria-selected`, not `aria-current`.
- [x] Extend static regression coverage for the software list accessibility contract. The milestone regression requires listbox/option semantics, `aria-selected`, and the selected-row tab stop.

## MUR-006 — Apply typed BIOS selection consistently to Start Empty

- [x] Keep Start Empty on a typed Rust command (`launch_mame_empty`) carrying only machine identity, optional BIOS, and typed launch preferences; no arbitrary argv field is exposed.
- [x] Validate any selected BIOS against authoritative MAME BIOS choices in Rust before launch.
- [x] Pass the software browser's selected BIOS through the Start Empty path.
- [x] Preserve existing machine-launch callers: the dedicated Start Empty command leaves the existing machine-launch request contract unchanged, and omitted BIOS continues to use MAME's default.
- [x] Add Rust validation tests for invalid/unavailable BIOS selections and frontend typing coverage. `mame_ui_launch.rs` covers unsafe and unreported BIOS values; the milestone regression requires the typed frontend invocation and Rust validation path.
- [x] Revalidate Start Empty legality in Rust (`can_start_empty`) before launch rather than relying on frontend visibility alone.

## MUR-007 — Qualification, documentation, and closure reconciliation

- [x] Extend `scripts/tauri/test-mame-ui-reproduction.py` to guard the remediated ownership, exact software lookup, listbox, artwork, and typed Start Empty contracts and include this TODO in the milestone regression.
- [x] Ensure the main CI sparse checkout includes this remediation ledger wherever the regression runs.
- [x] Reconcile every task/subtask in this ledger as complete, explicitly deferred with rationale, superseded with rationale, or not required.
- [x] Confirm no ambiguous unchecked item remains before closure; the milestone regression fails if `- [ ]` reappears in this ledger.
- [x] Qualify the exact final PR head through all applicable Tauri project/security/platform/documentation workflows. Exact-head evidence is recorded below.
- [x] Merge only an exact-head-qualified candidate. PR #27 was squash-merged only after the exact head qualified; merge identity is recorded below.
- [x] Verify the promoted `master` SHA and applicable post-merge CI before claiming remediation closure. Promoted-master evidence is recorded below.

## Closure evidence — 2026-09-15

**Exact final candidate:** `5c4d2d990587350573e9859f6f5e601fd1c5d261` on `ralph/mui-post-closure-remediation`.

Exact-head qualification passed before merge:

- PR `Tauri project` run `34944888085` — success, including the PR-only Linux release/development-window qualification.
- PR `Tauri security` run `34944888202` — success.
- PR `Tauri Linux packaging` run `34944888119` — success.
- PR `Tauri macOS packaging` run `34944888066` — success.
- PR `Tauri Windows packaging` run `34944888050` — success.
- PR `Build documentation` run `34944888102` — success.
- Push `Tauri project` run `34944357289` — success.
- Push `Tauri security` run `34944357292` — success.
- Push `Tauri Linux packaging` run `34944357323` — success.
- Push `Tauri macOS packaging` run `34944357298` — success.
- Push `Tauri Windows packaging` run `34944357303` — success.

**Merge:** PR #27 was squash-merged through the gated Ralph Bridge path. The promoted merge commit was `2de62fb13f1b328b84b49d8acf5261cde077045e`.

Post-merge `master` qualification for `2de62fb13f1b328b84b49d8acf5261cde077045e` passed completely:

- `Tauri project` run `34948876525` — success, including `linux-quality` and `linux-release-qualification`.
- `Tauri security` run `34948876484` — success.
- `Tauri Linux packaging` run `34948876728` — success.
- `Tauri macOS packaging` run `34948876586` — success.
- `Tauri Windows packaging` run `34948876587` — success.
- `Build documentation` run `34948876643` — success.

The remediation implementation is therefore promoted and qualified. This documentation-only evidence commit records the already-completed closure without reopening implementation scope.

## Completion rule

This ledger is fully reconciled. Remediation closure required the exact final candidate to pass all applicable project/security/platform/documentation gates, successful merge, and verification of the promoted `master` SHA plus applicable post-merge CI. Those conditions are satisfied by the evidence above. A green ancestor was not used to qualify a changed candidate.
