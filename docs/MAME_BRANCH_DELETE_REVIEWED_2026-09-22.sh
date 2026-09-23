#!/usr/bin/env bash
set -euo pipefail

# Reviewed delete script generated from docs/MAME_BRANCH_REVIEW_TODO_2026-09-22.md.
# Run from a checkout with origin pointing at git@github.com:ekkus93/mame.git after inspecting.

branches=(
  ralph/fix-tauri-event-names
  ralph/mt-1900-upstream-sync-rehearsal
  ralph/post-closeout-hardening-v4
  ralph/refactor-control-split
  ralph/refactor-large-tauri-files
  ralph/theme-color-restoration
  ralph/vs1-stop-control
  ralph/vs1-stop-flow
  ralph/vs1-stop-idle
  ralph/mame-ui-parity-mtp000-004
  ralph/mame-ui-parity-mtp007-008
  ralph/mame-ui-parity-mtp009-010
  ralph/mame-ui-parity-mtp011
  ralph/mame-ui-post-closure-remediation-docs
  ralph/mt-000-foundations
  ralph/mt-100-closure
  ralph/mt-100-scaffold
  ralph/mt-1404-native-tiny
  ralph/mt-2200-engineering-closure
  ralph/mt-410-docs-stage
  ralph/mt-501-docs-stage
  ralph/mt-502-formatgen
  ralph/mt-502-lockgen
  ralph/mt-701-characterize-mame-control
  ralph/mt-800-local-artwork
  ralph/mtp-011-startup-metadata-parity
  ralph/mtp-013-linux-compatibility
  ralph/mtp-014-documentation
  ralph/mame-parity-mtr000-2026-09-22
  ralph/mame-parity-mtr002-2026-09-22
  ralph/mame-parity-mtr002-ledger-2026-09-22
  ralph/mame-parity-mtr003-2026-09-22
  ralph/mame-parity-mtr003-ledger-2-2026-09-22
  ralph/mame-parity-mtr003-ledger-2026-09-22
  ralph/mame-parity-mtr003-reconcile-2026-09-22
  ralph/mame-parity-mtr003-startup-branding-2026-09-21
  ralph/mame-parity-mtr004-2026-09-21
  ralph/mame-parity-mtr005-006-2026-09-21
  ralph/mame-parity-mtr005-2026-09-21
  ralph/mame-parity-mtr006-2026-09-21
  ralph/mame-parity-mtr007-2026-09-22
  ralph/mame-parity-mtr008-2026-09-21
  ralph/mame-parity-mtr008-2026-09-22
  ralph/mame-parity-mtr009-2026-09-22
  ralph/mame-parity-mtr009-component-tests-2026-09-21
  ralph/mame-parity-mtr010-2026-09-21
  ralph/mame-parity-mtr010-tests-2026-09-21
  ralph/mame-parity-mtr012-2026-09-22
  ralph/mame-parity-mtr012-ready-2026-09-22
  ralph/mame-parity-remediation-branding-2026-09-21
  ralph/mame-parity-remediation-state-2026-09-21
  ralph/mtr-004-bottom-region-2026-09-21
  ralph/mtr-006-software-browser-component-tests
  ralph/mtr-006-software-browser-parity
  ralph/mtr-007-ledger-reconcile
  ralph/mtr-007-selected-row-visibility
  ralph/mtr-008-keyboard-page-semantics
  ralph/mtr-011-state-coupling-review
  ralph/mtr-012-test-evidence-inventory
  ralph/mtr-012-test-inventory-reconcile
  ralph/mtr-013-014-ledger-checkboxes
  ralph/mtr-013-ledger-reconcile
  ralph/mtr-014-doc-reconciliation
  ralph/mtr-015-final-closure-20260922
  ralph/muh-001-002-behavior
  ralph/muh-001-002-launch-semantics
  ralph/muh-001-003-core-hardening
  ralph/muh-003-app-focus-shim
  ralph/muh-003-reconcile
  ralph/muh-003-remove-focus-shim
  ralph/muh-004-migrate-legacy-browser
  ralph/muh-004-retire-legacy-browser
  ralph/muh-005-006-closure
  ralph/mui-002-006-browser-shell
  ralph/mui-002-007-browser-shell
  ralph/mui-003-015-parity-completion
  ralph/mui-003-015-parity-state
  ralph/mui-008-software-parity
  ralph/mui-009-014-contextual-surfaces
  ralph/mui-015-020-final-closure
  ralph/mui-post-closure-remediation
  ralph/post-closeout-hardening
  ralph/post-closeout-hardening-v2
  ralph/post-closeout-hardening-v3
)

for branch in "${branches[@]}"; do
  git push origin --delete "$branch"
done
