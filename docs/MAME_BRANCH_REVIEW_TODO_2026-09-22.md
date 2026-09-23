# MAME Branch Salvage Review TODO — 2026-09-22

**Repository:** `ekkus93/mame`  
**Purpose:** review the 84 remaining remote branches after the safe-delete pass and decide whether each branch should be deleted, abandoned, kept temporarily, or salvaged/ported into current `master`.  
**Source audit:** `docs/MAME_BRANCH_REVIEW_BEFORE_DELETE_2026-09-22.tsv`  
**Current branch-cleanup state:** the previously generated safe-delete script has already been run; this TODO covers only branches that still had unique patches according to the audit.

## Ground rules

- Use Ralph Bridge for all GitHub, branch, PR, merge, and CI operations.
- Do **not** merge stale branches directly into `master` unless a branch is proven current, narrow, conflict-free, and exactly desired. Prefer porting useful work onto a fresh branch from current `master`.
- Do **not** trust `ahead`/`behind` alone. Many Ralph branches were squash-merged; topology can report unique commits even when content is already represented on `master`.
- For each branch, inspect content against current `master`, decide whether anything is still valuable, and record the decision in this file.
- If useful work exists, create a fresh `ralph/salvage-...` branch from current `master`, port only the useful hunks/files, run exact-head CI, open a PR, merge normally, reload this TODO, and continue.
- If the branch is superseded, duplicate, abandoned, or no longer desirable, mark it delete-ready and include a short rationale.
- Preserve every branch checkbox until it has an explicit decision and evidence.

## Decision labels

- `DELETE_SUPERSEDED` — branch work is replaced by current `master` or a later completed remediation.
- `DELETE_DUPLICATE` — branch is another copy/attempt of work already represented on `master`.
- `DELETE_ABANDONED` — branch contains stale/partial work that should not be carried forward.
- `SALVAGE_PORT` — branch contains useful work; port selected changes to a fresh branch from current `master`.
- `KEEP_TEMPORARILY` — decision depends on another branch/PR or requires a separate follow-up decision.
- `MERGE_DIRECTLY_RARE` — branch can be merged as-is; use only if it is current, narrow, exact-head-qualified, and still desired.

## Per-branch review procedure

1. Resolve current `master` and branch head with Ralph Bridge.
2. Compare branch content against current `master`.
3. Inspect the branch detail file from `docs/MAME_BRANCH_DELETE_AUDIT_DETAILS_2026-09-22/` when available.
4. Decide one of the labels above.
5. If `SALVAGE_PORT`, create a fresh branch from current `master`, port only useful changes, qualify with exact-head CI, and merge through a normal PR.
6. If delete-ready, do not delete immediately unless this review pass is explicitly authorizing deletion; record it for the final delete script.

## Progress summary

- [ ] Review all 84 remaining branches.
- [ ] Create a delete script for all branches marked `DELETE_*`.
- [ ] Create fresh salvage PRs for every branch marked `SALVAGE_PORT`.
- [ ] Merge all accepted salvage PRs through exact-head CI.
- [ ] Re-run branch inventory after deletion/salvage to confirm only `origin/master` and intentional active branches remain.

## P0 — high-priority salvage review

- [ ] Review branch `ralph/fix-tauri-event-names`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-1900-upstream-sync-rehearsal`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/post-closeout-hardening-v4`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/refactor-control-split`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/refactor-large-tauri-files`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/theme-color-restoration`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/vs1-stop-control`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/vs1-stop-flow`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/vs1-stop-idle`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.

## P1 — roadmap/feature branch review

- [ ] Review branch `ralph/mame-ui-parity-mtp000-004`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-ui-parity-mtp007-008`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-ui-parity-mtp009-010`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-ui-parity-mtp011`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-ui-post-closure-remediation-docs`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-000-foundations`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-100-closure`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-100-scaffold`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-1404-native-tiny`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-2200-engineering-closure`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-410-docs-stage`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-501-docs-stage`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-502-formatgen`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-502-lockgen`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-701-characterize-mame-control`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mt-800-local-artwork`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtp-011-startup-metadata-parity`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtp-013-linux-compatibility`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtp-014-documentation`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.

## P2 — parity/remediation supersession review

- [ ] Review branch `ralph/mame-parity-mtr000-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr002-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr002-ledger-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr003-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr003-ledger-2-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr003-ledger-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr003-reconcile-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr003-startup-branding-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr004-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr005-006-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr005-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr006-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr007-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr008-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr008-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr009-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr009-component-tests-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr010-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr010-tests-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr012-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-mtr012-ready-2026-09-22`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-remediation-branding-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mame-parity-remediation-state-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-004-bottom-region-2026-09-21`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-006-software-browser-component-tests`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-006-software-browser-parity`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-007-ledger-reconcile`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-007-selected-row-visibility`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-008-keyboard-page-semantics`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-011-state-coupling-review`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-012-test-evidence-inventory`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-012-test-inventory-reconcile`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-013-014-ledger-checkboxes`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-013-ledger-reconcile`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-014-doc-reconciliation`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mtr-015-final-closure-20260922`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/muh-001-002-behavior`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/muh-001-002-launch-semantics`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/muh-001-003-core-hardening`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/muh-003-app-focus-shim`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/muh-003-reconcile`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/muh-003-remove-focus-shim`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/muh-004-migrate-legacy-browser`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/muh-004-retire-legacy-browser`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/muh-005-006-closure`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mui-002-006-browser-shell`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mui-002-007-browser-shell`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mui-003-015-parity-completion`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mui-003-015-parity-state`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mui-008-software-parity`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mui-009-014-contextual-surfaces`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mui-015-020-final-closure`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/mui-post-closure-remediation`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/post-closeout-hardening`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/post-closeout-hardening-v2`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.
- [ ] Review branch `ralph/post-closeout-hardening-v3`. Decision: `PENDING`. Rationale: _pending_. Follow-up: _pending_.

## Final cleanup checklist

- [ ] Every branch above has a non-`PENDING` decision.
- [ ] Every `SALVAGE_PORT` item has either a merged PR or a follow-up blocker recorded.
- [ ] Every `KEEP_TEMPORARILY` item has an owner/reason and an expiration condition.
- [ ] Branches marked `DELETE_*` are written to a final reviewed delete script.
- [ ] Final delete script is inspected before execution.
- [ ] Final delete script is executed.
- [ ] Fresh branch inventory confirms cleanup result.
