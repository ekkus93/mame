# MAME Tauri UI Post-Remediation Hardening Spec — 2026-09-15

**Repository:** `ekkus93/mame`  
**Baseline:** `master` at `f584e77658a75b1a2393099661af231c07e0c76a`  
**Origin:** Follow-up code review of `docs/MAME_TAURI_UI_POST_CLOSURE_REMEDIATION_TODO_2026-09-14.md` closure  
**Paired TODO:** `docs/MAME_TAURI_UI_POST_REMEDIATION_HARDENING_TODO_2026-09-15.md`

## 1. Purpose

The MAME-style Tauri UI post-closure remediation is promoted and qualified, but follow-up review found several hardening opportunities that should be addressed before treating the UI/browser/session launch path as maintainable long-term. This specification defines those follow-up fixes.

This work is not a reopening of the completed remediation ledger. The prior remediation fixed the core correctness and closure issues. This hardening pass tightens semantics, removes obsolete compatibility scaffolding, reduces stale API surface, and upgrades regression coverage from string-contract checks toward behavior-oriented tests.

## 2. Non-goals

- Do not reopen deliberately deferred MAME parity features such as DAT/category/custom filters or software favorites.
- Do not change the general machine-launch contract unless a specific hardening task requires it.
- Do not add arbitrary launch argument passthroughs from the WebView.
- Do not weaken Rust-side validation or move launch authority into frontend state.
- Do not bypass existing CI, branch protection, or exact-head qualification rules.

## 3. Scope

The scope is limited to issues observed in the code review after the post-closure remediation landed:

1. BIOS override semantics in the software browser.
2. Software launch-state reset behavior during row selection and activation.
3. Obsolete lifecycle-to-focus shim in `App.tsx`.
4. Legacy software browser/API drift.
5. Behavior-oriented regression coverage for the contracts above.
6. Documentation, CI, and closure evidence.

## 4. Issue: BIOS override semantics

### 4.1 Current behavior

`SoftwareBrowser` loads BIOS choices and initializes `selectedBios` to the reported default BIOS when one exists. Software launches and Start Empty then pass that selected BIOS through to Rust. Rust correctly validates explicit BIOS selections and emits `-bios <name>` when a BIOS value is present.

### 4.2 Problem

A reported MAME default BIOS is not necessarily equivalent to a user-selected override. Preselecting the default causes the frontend to pass an explicit BIOS even when the user did not make an override choice. That weakens the intended invariant that omitted BIOS preserves MAME's native default/config-driven behavior.

### 4.3 Required behavior

- The BIOS selector must start in the `MAME default` state unless the user explicitly chooses a BIOS.
- Launch requests must omit BIOS or send `null` when no explicit user override exists.
- Launch requests must carry a concrete BIOS only after explicit user selection.
- Rust must continue to validate any concrete BIOS identifier and membership before launch.
- UI text may still label the reported default BIOS in the dropdown, but it must not be auto-selected as an override.

### 4.4 Acceptance

- A regression test proves software launch does not pass a concrete BIOS immediately after BIOS choices load.
- A regression test proves Start Empty does not pass a concrete BIOS immediately after BIOS choices load.
- A regression test proves selecting a BIOS does pass that BIOS to software launch and Start Empty.
- Existing Rust BIOS validation remains intact.

## 5. Issue: software launch-state reset race

### 5.1 Current behavior

`SoftwareBrowser` resets launch state to idle whenever `selected` changes. `activateItem` sets the selected item and may immediately launch a single-part item.

### 5.2 Problem

For a newly selected single-part item, React can commit the selection change and then the selection effect can reset `launch` to idle after launch has already been set to launching. The eventual launch success/error still arrives, but the in-flight UI can briefly appear idle and may allow duplicate launches.

### 5.3 Required behavior

- Selection changes must not overwrite an active `launching` state.
- Browsing-context changes may clear stale success/error state, but must be scoped so they cannot erase an in-flight launch.
- Single-part activation must transition deterministically to launching and remain launching until the launch promise settles.
- Duplicate launch attempts must remain suppressed while a launch is active.

### 5.4 Acceptance

- A regression test proves activating a newly selected single-part software item leaves the UI in the launching state while the launch promise is pending.
- A regression test proves repeated activation while launching does not enqueue duplicate launch calls.
- Existing part-selection behavior for multi-part software is preserved.

## 6. Issue: obsolete `App.tsx` lifecycle focus shim

### 6.1 Current behavior

`App.tsx` still listens for terminal session lifecycle events and dispatches a synthetic browser `focus` event. Earlier implementations used browser focus to refresh gameplay-input ownership.

### 6.2 Problem

The remediated implementation moved gameplay-input ownership into `MameShell`, driven by session snapshots and lifecycle events. The synthetic focus shim is now obsolete and makes the ownership model harder to reason about.

### 6.3 Required behavior

- Remove the lifecycle-to-focus shim from `App.tsx` if no current behavior depends on it.
- Keep `App.tsx` as thin composition around `MameShell`.
- Keep gameplay-input ownership authoritative in `MameShell`.
- Terminal lifecycle events must still restore browser shortcut ownership through the `MameShell` listener path.

### 6.4 Acceptance

- A regression test or static contract proves `App.tsx` no longer dispatches synthetic focus for session lifecycle events.
- Existing session lifecycle tests and MAME UI regression checks still pass.

## 7. Issue: legacy software browser/API drift

### 7.1 Current behavior

The new MAME UI uses `tauri/src/backend/mameSoftware.ts` and `tauri/src/browser/SoftwareBrowser.tsx`. The repository still contains older software browsing/API types in `tauri/src/backend/commands.ts`, `tauri/src/backend/types.ts`, and `tauri/src/library/SoftwareListBrowser.tsx`.

### 7.2 Problem

The legacy surface lacks newer launch fields and item data such as `softwarePart`, `bios`, and `parts`. It is probably unreachable from the new `MameShell`, but it is stale code that can reintroduce bugs if it becomes reachable or is copied into new work.

### 7.3 Required behavior

Choose one of the following strategies and document the choice:

- **Preferred:** Remove unreachable legacy software browser/API code if no active import path depends on it.
- **Alternative:** Migrate the legacy surface to delegate to the typed `mameSoftware.ts` API and support BIOS/part-aware launch semantics.

Either strategy must leave the repository with a single authoritative frontend software-launch API contract.

### 7.4 Acceptance

- Static import review shows no reachable stale software browser path remains.
- No frontend type exposes a software launch request that omits required modern fields except where it delegates to the authoritative typed API.
- Multi-part software cannot be launched through a stale no-part path.
- BIOS-aware launch semantics cannot be bypassed through an old frontend helper.

## 8. Issue: behavior-oriented regression coverage

### 8.1 Current behavior

`scripts/tauri/test-mame-ui-reproduction.py` is valuable as a contract tripwire, but many checks are string based. It proves important tokens remain present, but it does not prove UI event sequencing or launch-call payload semantics.

### 8.2 Required behavior

Add targeted behavioral tests where the code can support them without excessive harness complexity:

- Shortcut suppression in software mode when `gameplayInputOwned === true`.
- Default BIOS omission for software launch and Start Empty.
- Explicit BIOS propagation after user selection.
- Launch-state behavior while a launch promise is pending.
- Legacy software surface removal or migration contract.

Retain static regression coverage for broad architectural invariants, but avoid treating substring presence as the only proof of behavior for newly hardened code.

### 8.3 Acceptance

- New tests fail on the reviewed buggy behavior and pass after the fixes.
- Existing milestone static regression remains useful and passes.
- CI runs the new tests in the existing Tauri project workflow.

## 9. Documentation and closure requirements

The paired TODO must be updated as work proceeds. Every task and subtask must end in one of these states:

- complete with evidence,
- explicitly deferred with rationale,
- superseded with rationale,
- not required with rationale.

Closure requires:

- exact final PR head qualification through all applicable workflows,
- merge through the gated Ralph Bridge path,
- reload of the TODO from promoted `master`,
- post-merge `master` CI verification,
- closure evidence recorded in the TODO or a linked closure document before the hardening effort is claimed complete.

## 10. Expected deliverables

- Source fixes for BIOS override semantics, launch-state stability, App shim removal, and legacy surface cleanup.
- Frontend/Rust/static regression tests covering the fixed behavior.
- Updated CI or sparse checkout inputs if new docs or tests require it.
- Reconciled TODO and closure evidence.
