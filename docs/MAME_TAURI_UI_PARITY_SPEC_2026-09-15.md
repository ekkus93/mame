# MAME Tauri UI Parity Spec — 2026-09-15

**Repository:** `ekkus93/mame`  
**Target app:** Tauri frontend for MAME  
**Primary requirement:** the Tauri frontend must look and behave like the original MAME UI closely enough that a normal user should not notice that it is a different frontend.  
**Reason for the Tauri rewrite:** preserve the original MAME UI experience while avoiding the original MAME app's compatibility problems across Linux desktop environments.  
**Companion TODO:** `docs/MAME_TAURI_UI_PARITY_TODO_2026-09-15.md`  
**Prior related work:** `docs/MAME_TAURI_UI_POST_REMEDIATION_HARDENING_TODO_2026-09-15.md`

The previous post-remediation hardening work addressed behavior and launch semantics. It did not close the user-visible UI parity problem. This spec is the canonical product contract for the visual and interaction parity pass.

---

## 1. Problem statement

The current Tauri frontend still looks like a generic desktop/web application: light GTK-like surfaces, black text, gray dividers, browser-style controls, and a top-level tab shell that does not resemble the original MAME UI. That is the wrong product direction.

The user did not ask for a new MAME-inspired app. The user asked for a Tauri implementation of the original MAME UI so the original UI experience remains available on Linux systems where the native MAME UI has desktop-environment problems.

Therefore, the source of truth is not a generic modern desktop design. The source of truth is the original MAME UI.

---

## 2. Reference captures and observed delta

The planning conversation included three key screenshots:

1. Original MAME UI reference capture: `Screenshot from 2026-09-14 20-53-46.png`.
   - Dark navy application background.
   - White text.
   - Center top title/search area.
   - Blue toolbar band with icon actions.
   - Left filter/category panel.
   - Central machine list.
   - Bright blue selected row with yellow selected text.
   - Right images/infos panel.
   - Bottom green status/driver panel.
   - White separator lines and thin vertical splitters.

2. Tauri comparison capture: `Screenshot from 2026-09-14 20-54-16.png`.
   - Generic light shell.
   - Browser-like tabs and toolbar.
   - White/gray list and filter panels.
   - No MAME-like blue toolbar, dark navy surface, or green status region.

3. Tauri comparison capture: `Screenshot from 2026-09-14 20-54-20.png`.
   - Same generic light presentation, including a desktop screenshot overlay.
   - Shows that layout CSS exists, but the product visual identity is wrong.

The implementation must preserve these captures as product evidence. If the image files are not committed to the repo, the implementing agent must either add a textual parity report derived from the screenshots or add repository-owned reference captures before marking visual parity complete.

---

## 3. Non-negotiable product requirement

The Tauri frontend must be a drop-in UI replacement for the original MAME UI:

```text
A user familiar with the original MAME UI should not need to learn a new interface,
and should not immediately notice that the frontend is implemented in Tauri.
```

That requires both visual parity and interaction parity. A dark color theme alone is not enough. A MAME-like layout alone is not enough. Passing TypeScript, Rust, and packaging CI is not enough.

---

## 4. Allowed and prohibited differences

### 4.1 Allowed differences

Differences are allowed only when they serve the Tauri rewrite's purpose or are unavoidable implementation details:

- improved stability across Linux desktop environments;
- consistent rendering across GNOME, KDE, Wayland, and X11;
- accessibility improvements that do not visibly redesign the UI;
- small font-metric, anti-aliasing, DPI, or WebView rendering differences;
- additional internal diagnostics hidden behind a non-default debug/developer surface;
- implementation-specific code structure, as long as the default user experience remains original-MAME-like.

### 4.2 Prohibited differences

The default UI must not introduce these differences:

- generic light GTK/web-form styling;
- system-color-driven `Canvas` / `CanvasText` / `currentColor` theme as the primary product palette;
- a redesigned top tab shell that replaces the original MAME menu/toolbar experience;
- default-visible app sections such as Session, Audit, History, Collections, Settings, or Diagnostics when they make the app look like a different product;
- browser-like buttons, large rounded inputs, or modern web dashboard spacing where the original UI uses dense list/navigation surfaces;
- new terminology that hides original MAME concepts;
- workflows that require more steps than the original UI for common search, filter, select, configure, and launch actions.

---

## 5. Visual parity contract

### 5.1 Global palette

The default application palette must be explicit and MAME-like:

- main background: dark navy / near-black blue;
- primary text: white or near-white;
- disabled/unavailable list items: muted gray;
- selected row: saturated blue gradient or close flat equivalent;
- selected row text: bright yellow, matching the original visual hierarchy;
- splitters/dividers: thin high-contrast light lines;
- status/footer panel: green field with white text where the original UI uses a green status region;
- toolbar band: blue strip distinct from the main navy field.

System colors may be used only for accessibility fallback or OS integration surfaces outside the main MAME UI. They must not drive the primary app appearance.

### 5.2 Window/header/search area

The Tauri window content must match the original MAME structure:

- top title area centered or visually equivalent to original MAME;
- visible MAME version/machine-count summary when available;
- search field/label behavior equivalent to the original search affordance;
- no generic product title bar inside the content that says `MAME Tauri Frontend` as the main visual header if the original MAME UI does not present that way.

### 5.3 Toolbar/actions area

The original-style toolbar strip must be present and recognizable:

- blue horizontal band;
- icon/action affordances corresponding to original MAME actions where implemented;
- visual placement and density similar to original;
- unavailable actions must be visibly disabled in a MAME-like way rather than omitted without explanation.

### 5.4 Main three-column layout

The default browser screen must preserve the original MAME spatial model:

- left filter/category list;
- central machine list;
- right image/info panel;
- thin splitters between panels;
- dense row spacing similar to original;
- no excessive whitespace that makes the app look like a modern dashboard.

### 5.5 Filter/category panel

The filter list must match the original categories and interaction model as closely as practical:

- Unfiltered;
- Available;
- Unavailable;
- Working;
- Not Working;
- Mechanical;
- Not Mechanical;
- Category / category-equivalent behavior;
- Favorites;
- BIOS;
- Not BIOS;
- Parents;
- Clones;
- Manufacturer;
- Year;
- Save Supported;
- Save Unsupported;
- CHD Required;
- No CHD Required;
- Vertical Screen;
- Horizontal Screen;
- Custom Filter if supported.

The selected filter marker must resemble the original diamond/selection indicator or an intentionally equivalent visual, not just a generic gray highlighted row.

### 5.6 Machine list

The machine list must behave and render like original MAME:

- dense list of machine descriptions;
- selected row blue with yellow selected text;
- disabled/unavailable rows muted gray;
- keyboard movement changes selection in-place;
- typing/search narrows or positions the list in the same practical way as original MAME;
- machine metadata must appear in the status/detail areas consistent with original MAME.

### 5.7 Right image/info panel

The right panel must preserve the original Images / Infos interaction:

- top tab/header row with `Images` and `Infos` or exact original labels;
- image-category selector such as `Snapshots`;
- original-like no-image placeholder behavior;
- info panel behavior equivalent to original MAME when information is available;
- right-panel content must not become a generic React detail sidebar.

### 5.8 Bottom status/driver panel

The bottom green status/driver region is part of the recognizable original UI and must be implemented:

- green background region;
- centered or original-like status text;
- manufacturer/year/driver parent/overall/graphics/sound information when available;
- meaningful empty/not-configured behavior that remains visually consistent with original MAME.

### 5.9 Configure/launch/options actions

Original MAME actions must remain recognizable:

- Configure Options;
- Configure Machine;
- launching selected machine/software;
- software-part selection where required;
- BIOS selection behavior without sending a default BIOS override unless explicitly chosen;
- no generic launch panel that replaces original interaction unless it is visually integrated into the original UI model.

---

## 6. Interaction parity contract

The Tauri frontend must preserve original-MAME-like behavior for common user operations.

### 6.1 Keyboard navigation

At minimum:

- Up/Down moves selection.
- PageUp/PageDown moves by page.
- Home/End moves to first/last item where original does so.
- Enter activates/launches or opens the original-equivalent action.
- Escape exits/backtracks according to original behavior.
- Search typing behaves like original search, including visible search state.
- Focus handling must not depend on Linux desktop environment quirks.

### 6.2 Mouse behavior

At minimum:

- clicking a filter changes the filter;
- clicking a machine row selects it;
- double-click or original-equivalent action launches where supported;
- right-panel tabs/actions respond consistently;
- selection and hover states must remain MAME-like, not browser-default-like.

### 6.3 Startup and data behavior

The app must not look broken when MAME metadata is absent or still importing. However, the target default user experience after configuration is a populated MAME browser equivalent to original MAME. The Tauri frontend should not permanently sit at `0 machines` with a generic empty message when enough MAME configuration exists to discover/import metadata.

The implementation must make these states visually and behaviorally explicit:

- MAME executable not configured;
- MAME executable configured but metadata not imported;
- metadata import in progress;
- metadata import failed;
- metadata import complete;
- no ROM paths / availability unknown;
- machine available/unavailable.

Each state must render inside the original-style MAME layout, not as a separate generic app shell.

---

## 7. Compatibility contract

The Tauri version exists to fix Linux desktop-environment compatibility problems. Therefore:

- rendering must be controlled by explicit app CSS/tokens, not host toolkit system colors;
- keyboard ownership/focus must be deterministic inside the WebView;
- Wayland and X11 behavior must be considered;
- desktop-specific quirks must be documented if they affect parity;
- compatibility fixes must not justify a visible redesign unless the original behavior is technically impossible.

---

## 8. Testing and validation contract

### 8.1 Visual acceptance evidence

Do not close this work using only code-level CI. Closure requires visual evidence.

Acceptable evidence includes one or more of:

- committed Playwright/screenshot tests comparing stable reference states;
- generated before/after screenshots from the Tauri app showing original-style palette/layout;
- a documented manual visual parity checklist with attached captures;
- CSS/static tests proving the primary app shell does not fall back to `Canvas`, `CanvasText`, and `currentColor` for core product surfaces.

### 8.2 Required automated tripwires

Add automated regression checks for:

- explicit MAME palette tokens exist and are used by default shell surfaces;
- no primary shell surface uses bare `background: Canvas` or `color: CanvasText` as its product styling;
- selected machine row has a blue selected background and yellow selected text or named equivalent tokens;
- bottom status/driver panel has an explicit green status token;
- default-visible navigation does not expose the generic dashboard shell in the main MAME browser experience;
- keyboard navigation behavior remains covered by frontend tests.

### 8.3 CI expectations

The exact final candidate must pass all applicable CI for changed files. Documentation-only closure commits still require applicable CI evidence before merge.

---

## 9. Implementation approach

The implementation should proceed in this order:

1. Capture/reference baseline: document original MAME visual and interaction requirements in repo-owned form.
2. Replace the generic shell with an original-MAME-like shell or hide generic shell surfaces behind debug/developer affordances.
3. Implement explicit MAME palette/theme tokens and apply them to every default-visible surface.
4. Match layout geometry and density.
5. Match machine/filter/right-panel/status behavior.
6. Add visual and behavioral regression tripwires.
7. Validate on exact head and reconcile the TODO.

Do not start by polishing the existing generic white UI. The existing generic shell should be treated as an implementation scaffold, not the desired product.

---

## 10. Definition of done

This parity pass is complete only when all of the following are true:

- default startup/browser UI is visibly original-MAME-like;
- original screenshot deltas are explicitly reconciled;
- generic light GTK/web-form appearance is gone from default-visible paths;
- core palette/layout/selection/status surfaces are covered by automated tripwires;
- keyboard and mouse behavior for common original UI operations is covered or manually documented with rationale;
- all TODO items are checked or explicitly reconciled with rationale;
- exact final candidate passes applicable CI;
- merged result is verified from promoted `master`;
- post-merge applicable CI is checked before closure is claimed.

If any of these are missing, do not mark the work closed.
