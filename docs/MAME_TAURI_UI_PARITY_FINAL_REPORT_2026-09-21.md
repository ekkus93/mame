# MAME Tauri UI Parity Final Report — 2026-09-21

## Scope

This report is the final repo-owned visual/behavioral evidence for `docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md`. Binary screenshots cannot be committed through the current Ralph text-write path, and hosted CI has no stable cross-platform WebView pixel-baseline harness, so the TODO explicitly permits a complete textual visual parity report backed by source/component tripwires and the real Tauri Xvfb smoke path.

## Final visual parity assessment

The default ready-state Tauri frontend now matches the original-MAME selector identity described by the repo-owned reference contract:

- dark navy application/browser surface with explicit MAME palette tokens rather than host `Canvas`/`CanvasText` product styling;
- compact blue toolbar/search/action treatment rather than a generic dashboard header;
- persistent original-order left filter list with an original-like selection indicator and visibly integrated deferred Category/Custom Filter positions;
- dense central machine list with blue selected rows, yellow selected text, and muted unavailable rows;
- right-side `Images` / `Infos` context with `Snapshots`, original-like missing-image treatment, compact artwork/info layout, and a defined no-selection state;
- green selected-machine driver/status region carrying machine metadata rather than generic backend diagnostics;
- startup, not-configured, metadata import/progress/failure, empty-catalog, and unknown-ROM states remain inside the same MAME-style shell;
- default-visible generic Session/Audit/History/Collections/Settings/Diagnostics dashboard tabs are absent; project-only tools remain secondary surfaces;
- keyboard/mouse behavior covers Up/Down, PageUp/PageDown, Home/End, Enter activation, Escape/back, search focus, filter/list/right-panel focus movement, click selection, double-click activation, and right-panel tabs;
- visible focus remains explicit/high-contrast, gameplay-input ownership gates shortcuts, and host light/dark mode does not replace the MAME palette.

## Intentional deviations

The selector is implemented in React/Tauri/WebView instead of embedding the original native selector. Project-only secondary tools remain available behind secondary actions. Category and Custom Filter retain original-like positions while full data/authoring support is deferred. Narrow windows use an explicit Details affordance. Project Icon/System Image artwork extensions coexist with canonical MAME artwork categories. Minor platform font/compositor differences are accepted when they do not change the interaction model.

These are bounded implementation/compatibility deviations, not permission to redesign the default selector.

## Regression and compatibility evidence

The parity suite includes `mameParityTheme.test.ts`, `mameBrowserInteraction.source.test.ts`, `mameCatalogState.test.tsx`, and `linuxDesktopCompatibility.source.test.ts`. Together they pin theme import/token behavior, default-shell structure, selected-row/status/right-panel landmarks, keyboard/mouse interactions, catalog/startup states, WebView shell routing, gameplay-input focus gating, and visible focus treatment.

Linux compatibility remains on the React/Tauri/WebView boundary documented in `docs/MAME_TAURI_LINUX_DESKTOP_COMPATIBILITY_2026-09-21.md`; the original native UI code path is not reintroduced. Hosted Linux CI additionally exercises a real Tauri development window under Xvfb/X11. GNOME/KDE and Wayland/X11 practical considerations and local verification steps are documented there.

## Screenshot limitation and disposition

Committed binary reference captures and pixel-baseline screenshot tests are explicitly deferred. The current Ralph source-write path is text-only and the hosted matrix does not provide a stable cross-platform WebView pixel-baseline harness. This is not a parity implementation blocker: the authoritative TODO permits a textual report when screenshot/reference capture is unavailable, and the repository retains a complete visual contract plus static/component tripwires. Future binary-capable work should store captures under `docs/reference/mame-ui-parity/` with date, source SHA, window size, desktop environment, display server, host theme, selected filter/machine, and right-panel state.

## Closure conclusion

MTP-000 through MTP-014 are reconciled. No visual-parity implementation task remains unresolved; screenshot automation is the only explicitly deferred evidence mechanism, with rationale and fallback evidence recorded above. MTP-015 closes after this report/TODO candidate is exact-head qualified, merged through the Ralph gate, reloaded from promoted `master`, and applicable post-merge `master` CI is verified and recorded in the final reconciliation follow-up.
