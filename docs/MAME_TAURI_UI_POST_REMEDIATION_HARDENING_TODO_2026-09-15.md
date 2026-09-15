# MAME Tauri UI Post-Remediation Hardening TODO — 2026-09-15

**Repository:** `ekkus93/mame`  
**Baseline:** `master` at `f584e77658a75b1a2393099661af231c07e0c76a`  
**Spec:** `docs/MAME_TAURI_UI_POST_REMEDIATION_HARDENING_SPEC_2026-09-15.md`  
**Predecessor remediation:** `docs/MAME_TAURI_UI_POST_CLOSURE_REMEDIATION_TODO_2026-09-14.md`

This ledger tracks the follow-up hardening issues found during review of the completed MAME Tauri UI post-closure remediation. The predecessor remediation remains closed. These tasks tighten semantics, remove obsolete/stale surfaces, and add behavioral coverage so future changes cannot regress the remediated contracts.

## MUH-001 — Preserve omitted-BIOS semantics unless the user explicitly selects an override

- [ ] Change `SoftwareBrowser` so BIOS choices load without auto-selecting the reported default BIOS as an explicit override.
- [ ] Keep the `MAME default` selector state as the initial state after BIOS choices load.
- [ ] Ensure `launchMameSoftware` receives `bios: null` or an omitted BIOS field when the user has not explicitly selected a BIOS.
- [ ] Ensure `launchMameEmpty` receives `bios: null` or an omitted BIOS field when the user has not explicitly selected a BIOS.
- [ ] Preserve explicit BIOS propagation when the user chooses a concrete BIOS option.
- [ ] Preserve Rust-side BIOS identifier validation and authoritative membership validation before both software launch and Start Empty launch.
- [ ] Add regression coverage proving default BIOS choices are displayed but not sent as explicit launch overrides.
- [ ] Add regression coverage proving an explicitly selected BIOS is sent for both software launch and Start Empty.

## MUH-002 — Prevent software launch-state resets from racing single-part activation

- [ ] Refactor the `selected` effect in `SoftwareBrowser` so it updates part selection without blindly resetting active launch state.
- [ ] Move stale launch-message clearing to explicit browsing-context transitions or another scoped path that cannot overwrite `launching`.
- [ ] Preserve the current automatic launch behavior for single-part software activation.
- [ ] Preserve the part-required path for multi-part software.
- [ ] Ensure duplicate launch attempts remain suppressed while `launch.status === "launching"`.
- [ ] Add behavioral regression coverage proving a newly selected single-part item remains in the launching state while the launch promise is pending.
- [ ] Add behavioral regression coverage proving repeated activation while launching does not produce duplicate launch calls.

## MUH-003 — Remove obsolete lifecycle-to-focus ownership shim from `App.tsx`

- [x] Remove the session lifecycle listeners in `App.tsx` that dispatch synthetic `focus` events.
- [x] Keep `App.tsx` as thin composition around `MameShell`.
- [x] Verify gameplay-input ownership still flows from `MameShell` session snapshot and lifecycle-event handling.
- [x] Verify terminal session events still restore browser shortcut ownership without relying on browser focus.
- [x] Add or extend regression coverage so the removed shim is not reintroduced accidentally.

**MUH-003 evidence:** Implemented and merged by PR #31 as `b92d961e17ee3f84ab66bccaf2331f4dfabc0b01` (`MUH-003: remove obsolete session focus shim`). Exact promoted `master` CI passed all five applicable workflows: Tauri project run 34976771661, Tauri security 34976771835, Windows packaging 34976771744, macOS packaging 34976771639, and Linux packaging 34976771712.

## MUH-004 — Eliminate or migrate stale legacy software browser/API surface

- [x] Audit imports and reachability for `tauri/src/library/SoftwareListBrowser.tsx` and the older software request/response helpers in `tauri/src/backend/commands.ts` and `tauri/src/backend/types.ts`.
- [x] Decide and document whether the legacy surface is removed or migrated.
- [x] If removed, delete unreachable legacy software UI/API types and update any affected exports/imports.
- [x] If migrated instead, delegate to `tauri/src/backend/mameSoftware.ts` and support `parts`, `softwarePart`, `bios`, and typed Start Empty semantics.
- [x] Ensure no reachable frontend software launch path can bypass part validation by launching multi-part software without selecting a part.
- [x] Ensure no reachable frontend software launch path can bypass BIOS-aware typed launch semantics.
- [x] Add regression coverage for the chosen removal/migration contract.

**MUH-004 evidence:** Implemented and merged by PR #36 as promoted `master` `7f88a440641079f2567c5a0fa237ec8e8a5edc4b` (`MUH-004: retire legacy software browser implementation`). The retained `SoftwareListBrowser` is compatibility-only and delegates to the authoritative `SoftwareBrowser`; legacy frontend software commands were removed and command tests migrated to the authoritative API. Promoted exact-head CI passed Tauri project run 34998188566, Tauri security run 34998188645, Windows packaging run 34998188669, macOS packaging run 34998188613, and Linux packaging run 34998188577.

## MUH-005 — Add behavioral coverage for remediated UI contracts

- [ ] Add frontend tests proving software-browser global shortcuts ignore `/` and `Escape` while `gameplayInputOwned === true`.
- [ ] Add frontend tests proving the same shortcuts work when gameplay input is not owned and focus is outside editable fields.
- [ ] Add frontend tests for default BIOS omission and explicit BIOS propagation.
- [ ] Add frontend tests for launch-state behavior during pending launch promises.
- [ ] Add Rust or command-boundary tests proving omitted BIOS remains omitted in launch arguments where practical.
- [ ] Keep the existing static milestone regression as a broad architectural tripwire.
- [ ] Prefer behavioral tests over adding more substring-only checks for sequencing or payload semantics.

## MUH-006 — CI, documentation, and closure reconciliation

- [ ] Ensure any new tests are run by the existing `Tauri project` workflow.
- [ ] Update sparse checkout or workflow path triggers if new hardening docs/tests need to be present in CI jobs.
- [ ] Reconcile every task/subtask in this TODO as complete, explicitly deferred with rationale, superseded with rationale, or not required with rationale.
- [ ] Confirm no ambiguous unchecked item remains before closure.
- [ ] Qualify the exact final PR head through all applicable project/security/platform/documentation workflows.
- [ ] Merge only an exact-head-qualified candidate through the gated Ralph Bridge path.
- [ ] Reload this TODO from promoted `master` after merge and verify the promoted SHA.
- [ ] Verify applicable post-merge `master` CI before claiming hardening closure.
- [ ] Record closure evidence in this TODO or a linked closure document.

## Completion rule

This hardening effort is not complete until all MUH tasks are implemented or explicitly reconciled, exact-head CI passes on the final candidate, the work is merged to `master`, the TODO is reloaded from promoted `master`, post-merge CI is verified, and closure evidence is recorded. A green ancestor does not qualify a changed candidate.
