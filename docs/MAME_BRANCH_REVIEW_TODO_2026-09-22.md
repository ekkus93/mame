# MAME Branch Salvage Review TODO — CLOSED 2026-09-23

**Repository:** `ekkus93/mame`  
**Original purpose:** review the 84 remaining remote branches after the safe-delete pass and decide whether each branch should be deleted, abandoned, kept temporarily, or salvaged/ported into current `master`.

## Closure status

- [x] Review all 84 remaining branches.
- [x] Create a delete script for all branches marked `DELETE_*`.
- [x] Create fresh salvage PRs for every branch marked `SALVAGE_PORT`.
- [x] Merge all accepted salvage PRs through exact-head CI.
- [x] Re-run branch inventory after deletion/salvage to confirm only `origin/master` and intentional active branches remain.

## Final cleanup checklist

- [x] Every branch has a non-`PENDING` decision.
- [x] Every `SALVAGE_PORT` item has either a merged PR or a follow-up blocker recorded.
- [x] Every `KEEP_TEMPORARILY` item has an owner/reason and an expiration condition.
- [x] Branches marked `DELETE_*` are written to a final reviewed delete script.
- [x] Final delete script is inspected before execution.
- [x] Final delete script is executed.
- [x] Fresh branch inventory confirms cleanup result.

## Closure evidence

- Reviewed delete script: `docs/MAME_BRANCH_DELETE_REVIEWED_2026-09-22.sh`.
- Fresh inventory committed after local script execution: `docs/MAME_BRANCH_INVENTORY_2026-09-23.txt`.
- Inventory commit: `e92bd3d5e6b109615112901d6418c173b41e1292` (`docs: refresh branch inventory after cleanup`).
- Inventory generated at `2026-09-23T00:08:33-07:00` from local checkout HEAD `5fd52147fd0a6a5bd8e71a8a423f4a5a7a6f3c77`.

The post-delete inventory shows the 84 reviewed branches are gone. The only remaining remote refs recorded by the committed inventory are:

```text
origin/master
origin/ralph/branch-review-inspection-20260922
origin/ralph/branch-review-p0-decisions-20260922
origin/ralph/branch-review-p1-decisions-20260922
origin/ralph/branch-review-p2-delete-script-20260922
origin/ralph/branch-review-todo-20260922
```

The five remaining `origin/ralph/branch-review-*` refs are administrative branches from this cleanup workflow, not branches from the original 84-branch review bucket. Ralph Bridge does not expose a remote branch-delete operation, so any later deletion of those administrative refs must be performed locally or by another authorized tool.

## Decision summary

No branch was selected for `SALVAGE_PORT`, `KEEP_TEMPORARILY`, or `MERGE_DIRECTLY_RARE`. Every reviewed branch was classified as delete-ready: `DELETE_DUPLICATE`, `DELETE_SUPERSEDED`, or `DELETE_ABANDONED`.

### P0 — high-priority salvage review

- [x] `ralph/fix-tauri-event-names` — `DELETE_DUPLICATE`.
- [x] `ralph/mt-1900-upstream-sync-rehearsal` — `DELETE_ABANDONED`.
- [x] `ralph/post-closeout-hardening-v4` — `DELETE_SUPERSEDED`.
- [x] `ralph/refactor-control-split` — `DELETE_DUPLICATE`.
- [x] `ralph/refactor-large-tauri-files` — `DELETE_SUPERSEDED`.
- [x] `ralph/theme-color-restoration` — `DELETE_SUPERSEDED`.
- [x] `ralph/vs1-stop-control` — `DELETE_SUPERSEDED`.
- [x] `ralph/vs1-stop-flow` — `DELETE_SUPERSEDED`.
- [x] `ralph/vs1-stop-idle` — `DELETE_SUPERSEDED`.

### P1 — roadmap/feature branch review

- [x] `ralph/mame-ui-parity-mtp000-004` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-ui-parity-mtp007-008` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-ui-parity-mtp009-010` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-ui-parity-mtp011` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-ui-post-closure-remediation-docs` — `DELETE_SUPERSEDED`.
- [x] `ralph/mt-000-foundations` — `DELETE_DUPLICATE`.
- [x] `ralph/mt-100-closure` — `DELETE_SUPERSEDED`.
- [x] `ralph/mt-100-scaffold` — `DELETE_ABANDONED`.
- [x] `ralph/mt-1404-native-tiny` — `DELETE_ABANDONED`.
- [x] `ralph/mt-2200-engineering-closure` — `DELETE_DUPLICATE`.
- [x] `ralph/mt-410-docs-stage` — `DELETE_DUPLICATE`.
- [x] `ralph/mt-501-docs-stage` — `DELETE_SUPERSEDED`.
- [x] `ralph/mt-502-formatgen` — `DELETE_SUPERSEDED`.
- [x] `ralph/mt-502-lockgen` — `DELETE_SUPERSEDED`.
- [x] `ralph/mt-701-characterize-mame-control` — `DELETE_ABANDONED`.
- [x] `ralph/mt-800-local-artwork` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtp-011-startup-metadata-parity` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtp-013-linux-compatibility` — `DELETE_DUPLICATE`.
- [x] `ralph/mtp-014-documentation` — `DELETE_SUPERSEDED`.

### P2 — parity/remediation supersession review

All P2 branches were reviewed as stale parity/remediation history superseded by the promoted final MTR remediation ledger, rendered review, documentation reconciliation, and closure evidence on current `master`.

- [x] `ralph/mame-parity-mtr000-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr002-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr002-ledger-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr003-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr003-ledger-2-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr003-ledger-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr003-reconcile-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr003-startup-branding-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr004-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr005-006-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr005-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr006-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr007-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr008-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr008-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr009-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr009-component-tests-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr010-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr010-tests-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr012-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-mtr012-ready-2026-09-22` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-remediation-branding-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mame-parity-remediation-state-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-004-bottom-region-2026-09-21` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-006-software-browser-component-tests` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-006-software-browser-parity` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-007-ledger-reconcile` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-007-selected-row-visibility` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-008-keyboard-page-semantics` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-011-state-coupling-review` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-012-test-evidence-inventory` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-012-test-inventory-reconcile` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-013-014-ledger-checkboxes` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-013-ledger-reconcile` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-014-doc-reconciliation` — `DELETE_SUPERSEDED`.
- [x] `ralph/mtr-015-final-closure-20260922` — `DELETE_SUPERSEDED`.
- [x] `ralph/muh-001-002-behavior` — `DELETE_SUPERSEDED`.
- [x] `ralph/muh-001-002-launch-semantics` — `DELETE_SUPERSEDED`.
- [x] `ralph/muh-001-003-core-hardening` — `DELETE_SUPERSEDED`.
- [x] `ralph/muh-003-app-focus-shim` — `DELETE_SUPERSEDED`.
- [x] `ralph/muh-003-reconcile` — `DELETE_SUPERSEDED`.
- [x] `ralph/muh-003-remove-focus-shim` — `DELETE_SUPERSEDED`.
- [x] `ralph/muh-004-migrate-legacy-browser` — `DELETE_SUPERSEDED`.
- [x] `ralph/muh-004-retire-legacy-browser` — `DELETE_SUPERSEDED`.
- [x] `ralph/muh-005-006-closure` — `DELETE_SUPERSEDED`.
- [x] `ralph/mui-002-006-browser-shell` — `DELETE_SUPERSEDED`.
- [x] `ralph/mui-002-007-browser-shell` — `DELETE_SUPERSEDED`.
- [x] `ralph/mui-003-015-parity-completion` — `DELETE_SUPERSEDED`.
- [x] `ralph/mui-003-015-parity-state` — `DELETE_SUPERSEDED`.
- [x] `ralph/mui-008-software-parity` — `DELETE_SUPERSEDED`.
- [x] `ralph/mui-009-014-contextual-surfaces` — `DELETE_SUPERSEDED`.
- [x] `ralph/mui-015-020-final-closure` — `DELETE_SUPERSEDED`.
- [x] `ralph/mui-post-closure-remediation` — `DELETE_SUPERSEDED`.
- [x] `ralph/post-closeout-hardening` — `DELETE_SUPERSEDED`.
- [x] `ralph/post-closeout-hardening-v2` — `DELETE_SUPERSEDED`.
- [x] `ralph/post-closeout-hardening-v3` — `DELETE_SUPERSEDED`.

## Result

The original 84-branch review bucket is closed. No mergeable or salvage-worthy changes remain from those branches, the reviewed delete script was executed locally, and the committed fresh inventory confirms the reviewed branches were removed from the remote branch list.
