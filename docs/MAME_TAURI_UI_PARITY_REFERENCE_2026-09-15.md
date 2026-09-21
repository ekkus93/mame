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

The failed Tauri captures showed a generic light desktop/web application: white and gray surfaces, black text, browser-like tabs, generic top navigation, and no MAME-like blue toolbar, navy surface, selected-row treatment, or green status region.

## Current implementation note

The parity implementation now keeps the default selector on explicit MAME palette tokens and the original three-region spatial model: left filters, dense central machine list, right `Images` / `Infos`, compact blue command/search treatment, blue/yellow selected rows, muted unavailable rows, and the green driver/status region. Startup/metadata states, command labels, keyboard/mouse navigation, right-panel behavior, and Linux focus/theme compatibility are covered by source/component tests and are reconciled in the parity TODO. Category and Custom Filter remain intentionally deferred but visible in original-like order.

## Manual visual checklist

Before final closure, compare a fresh Tauri capture against the original reference and verify:

- the default screen is dark/navy, not white/gray;
- there is no default-visible generic Session/Audit/History/Collections/Settings/Diagnostics tab bar;
- toolbar/action surfaces are blue and compact;
- filters render in original order with an original-like indicator;
- machine rows are dense;
- selected rows are blue with yellow selected text;
- disabled/unavailable rows are muted gray;
- the right panel uses `Images` / `Infos` terminology;
- missing images resemble the original placeholder;
- the bottom status/driver area is green;
- keyboard and mouse behavior remains original-MAME-like.

## Screenshot storage

The current Ralph text-file write path does not commit binary screenshots, and hosted CI does not provide a stable cross-platform WebView pixel-baseline harness. This textual contract plus static/component tripwires is therefore the qualified parity evidence for this pass.

When binary screenshot commits are available, capture a stable configured browser and at least one startup/metadata empty state. Store them under a documented `docs/reference/` path and record the filename, date, window size, desktop environment, display server, and exact source SHA here. Keep the textual checklist even after screenshots are added.
