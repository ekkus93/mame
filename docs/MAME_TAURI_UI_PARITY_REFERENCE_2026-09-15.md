# MAME Tauri UI Parity Reference — 2026-09-15

This document preserves the visual target for the parity pass. The Tauri app is not a redesign; it is a WebView/Tauri replacement for the original MAME UI so Linux desktop-environment compatibility improves without making users learn a new frontend.

## Original MAME reference

The original reference screenshot shows:

- dark navy main surface;
- white foreground text;
- centered top title/search area;
- blue toolbar band;
- left filter/category list with the original filter order;
- dense central machine list;
- blue selected row with yellow selected text;
- muted gray disabled/unavailable rows;
- right `Images` / `Infos` panel;
- original-like `No image Available` placeholder;
- green bottom driver/status region;
- thin bright panel splitters.

## Failed Tauri comparison captures

The failed Tauri captures showed a generic light desktop/web application: white and gray surfaces, black text, browser-like tabs, generic top navigation, and no MAME-like blue toolbar, navy surface, selected-row treatment, or green status region. That generic dashboard composition is historical failure evidence, not the intended product direction.

## Current implementation

The default Tauri selector now uses the original-MAME spatial model: explicit dark/navy palette tokens, compact blue toolbar, original-order left filter list, dense center machine list, blue/yellow selected-row treatment, muted unavailable rows, `Images` / `Infos` right context, and green selected-machine driver/status region. Startup and metadata states remain inside the same MAME-like shell rather than falling back to a generic light application page.

Intentional deviations are bounded: the selector is React/Tauri/WebView rather than the original native selector; project-only secondary tools remain behind secondary actions; Category/Custom Filter positions remain visible where full support is deferred; narrow windows use an explicit Details affordance; and project artwork extensions coexist with canonical MAME categories. These deviations must not change the default desktop visual identity.

## Manual visual checklist

Before final closure, compare a fresh Tauri run against the original reference and verify:

- the default screen is dark/navy, not white/gray;
- there is no default-visible generic Session/Audit/History/Collections/Settings/Diagnostics tab bar;
- toolbar/action surfaces are blue and compact;
- filters render in original order with an original-like indicator;
- machine rows are dense;
- selected rows are blue with yellow selected text;
- disabled/unavailable rows are muted gray;
- the right panel uses `Images` / `Infos` terminology;
- missing images resemble the original placeholder;
- the bottom selected-machine status/driver area is green;
- startup/not-configured/import/error states retain the same palette and shell;
- keyboard and mouse behavior remains original-MAME-like;
- visible focus remains high-contrast and host light/dark mode does not replace the MAME palette.

## Capturing and updating references

For a reproducible comparison, record the date, operating system/desktop session, Wayland/X11 where applicable, host theme, window dimensions, selected filter, selected machine, and right-panel tab/category. Capture the original/reference selector and Tauri selector at the same practical window size and state. When a binary-capable repository path is available, store captures under `docs/reference/mame-ui-parity/` and list the filenames here with the environment metadata.

Do not update the reference merely to make a regression look intentional. A visual target change must be reconciled against the parity specification and TODO.

## Screenshot automation status

The hosted CI matrix does not currently provide a stable cross-platform WebView pixel-baseline harness, and the Ralph source-write path is text-only. Screenshot regression tests are therefore explicitly deferred for this pass. The repo-owned textual contract, source/component theme tripwires, real Tauri Xvfb development-window smoke, and this complete manual visual report are the qualified fallback until a stable screenshot harness is introduced.
