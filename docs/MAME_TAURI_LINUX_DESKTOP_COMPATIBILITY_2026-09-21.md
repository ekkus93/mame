# MAME Tauri Linux desktop compatibility — 2026-09-21

This note records the compatibility boundary for the original-MAME UI parity pass. The Tauri frontend preserves the recognizable MAME selector experience while deliberately avoiding a return to the original native selector implementation that motivated the Linux compatibility work.

## Rendering path

The default application path is React -> `App.tsx` -> `MameShell` / `MameBrowser`, rendered in the Tauri WebView configured by `tauri/src-tauri/tauri.conf.json`. The frontend does not embed or call the original native MAME selector UI. MAME itself remains a separately supervised process for emulation, rendering, audio, timing, and gameplay input.

This separation reduces dependence on the original selector's desktop-toolkit behavior. The selector chrome, palette, list selection, focus treatment, and panel geometry are controlled by repository-owned HTML/CSS rather than host toolkit system colors.

## GNOME, KDE, Wayland, and X11

The project does not assume a particular GNOME or KDE theme. `mameTheme.css` defines the product palette explicitly and sets `color-scheme: dark`; the default browser shell uses semantic `--mame-*` tokens. Host light/dark mode therefore does not choose the selector's primary colors.

Keyboard behavior is handled inside the WebView with explicit focus targets. Global browser shortcuts fail closed when `document.hasFocus()` is false, when an editable element owns the event, or when native MAME owns gameplay input. Machine-list navigation moves DOM focus deliberately rather than depending on window-manager traversal conventions.

Wayland and X11 can differ in compositor/window activation behavior, but the parity implementation does not use X11-only APIs in the product UI. The Linux CI development-window smoke currently runs under Xvfb/X11 because it is deterministic and available on hosted runners. That proves the Tauri/WebView window launches and remains interactive in the CI environment; it does not claim exhaustive compositor certification for every GNOME/KDE/Wayland combination.

## Theme and focus invariants

- `mameTheme.css` must load after base CSS.
- The default MAME shell must use explicit MAME background/text/selection/status/focus tokens.
- The transient backend-connect screen must also use the MAME tokens rather than generic `Canvas` / `CanvasText` presentation.
- Keyboard shortcuts must honor WebView focus and gameplay-input ownership.
- `:focus-visible` outlines must remain high-contrast and must not be removed for visual parity.

`linuxDesktopCompatibility.source.test.ts` and the existing `mameParityTheme.test.ts` provide static regression tripwires for these invariants.

## Practical manual check

1. Start the app with `cd tauri && npm run tauri -- dev`.
2. Confirm the startup screen and machine browser stay dark/navy regardless of the host light/dark theme.
3. Tab through search, filter, machine, right-panel, and command controls and confirm the yellow focus outline remains visible.
4. Verify Up/Down/PageUp/PageDown/Home/End selection movement and Escape/back behavior while the window is focused.
5. Move focus to another desktop window and confirm app-level shortcuts do not act on the unfocused selector.
6. If both sessions are available, repeat on Wayland and X11. Record compositor-specific problems as compatibility defects rather than redesigning the selector.

No known parity requirement requires the original native selector code path.
