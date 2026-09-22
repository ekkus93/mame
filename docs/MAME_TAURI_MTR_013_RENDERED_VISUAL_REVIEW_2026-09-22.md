# MTR-013 — Rendered Visual Parity Re-qualification

**Repository:** `ekkus93/mame`  
**TODO:** `docs/MAME_TAURI_UI_PARITY_REVIEW_REMEDIATION_TODO_2026-09-21.md`  
**Reviewed promoted implementation head:** `cca7c5e41cea4792a89dea46f023068a5f7826af`  
**Current reconciliation base:** `011429b921642253bc8c7e2714abc980f3fbf755`

## Validation path used

The strongest rendered/runtime path currently exposed through Ralph Bridge is the promoted Tauri project workflow on head `cca7c5e41cea4792a89dea46f023068a5f7826af`, run `35787585902`.

That run completed successfully and included:

- `linux-quality` job `106948027544`, including frontend format, lint, typecheck, tests, production build, Rust tests, and clippy.
- `linux-release-qualification` job `106948639364`, including `Tauri production build smoke check` and `Tauri development window smoke check`, both completed successfully.

Artifacts exposed through Ralph for run `35787585902` were:

- `mame-tauri-mt2004-external-window-rc-cca7c5e41cea4792a89dea46f023068a5f7826af`
- `mame-tauri-mt1800-cross-platform-qualification-cca7c5e41cea4792a89dea46f023068a5f7826af`
- `mame-tauri-mt1700-performance-baseline-cca7c5e41cea4792a89dea46f023068a5f7826af`
- `mame-tauri-mt2100-final-quality-closure-cca7c5e41cea4792a89dea46f023068a5f7826af`
- `mame-tauri-license-inventory-cca7c5e41cea4792a89dea46f023068a5f7826af`

No binary screenshot or pixel-baseline artifact is exposed through the current Ralph artifact surface. This review therefore records the strongest available textual rendered review from the successful Tauri/WebView smoke path and the promoted UI contracts. It does **not** claim pixel evidence.

## Manual visual checklist update

This MTR-013 review covers every visual issue reopened by the 2026-09-21 remediation:

- default startup shell and absence of default-visible generic Tauri branding;
- title/search/header treatment after pseudo-title removal;
- left filter, central list, right panel density and geometry;
- selected blue/yellow machine row;
- muted unavailable machine/software rows;
- green driver/status region as the bottom-most persistent browser region;
- absence of permanent History/Collections/Diagnostics footer chrome;
- Software Browser styling and dense list treatment;
- Images/Infos tab treatment and keyboard focus states;
- artwork loading versus true missing/error states;
- metadata not-configured/importing/failure/ready states where represented by catalog-state panels;
- configuration and secondary surfaces for host-theme leakage;
- visible focus states.

## Textual rendered review

### Default startup shell

The default shell composition no longer renders the generic pseudo-title row. The browser surface starts with the machine toolbar/search treatment, followed by the main grid and the machine driver/status region. The promoted Tauri development window smoke check demonstrates that the WebView can launch the current composition successfully.

### Title/search/header and generic branding

The prohibited default-visible phrase `MAME Tauri Frontend` is guarded against in `mameShellComposition.source.test.ts`. The default visible header is now the MAME browser toolbar/search/range/action row. There is no retained generic Tauri rewrite branding in the default shell/browser sources.

### Left filter, central list, and right panel geometry/density

The browser retains the three-region MAME layout: filter panel on the left, machine list in the central region, and selected-machine context on the right. Selection visibility and page navigation were corrected through MTR-007/MTR-008 so keyboard movement and result replacement keep selected rows usable without false full-catalog Home/End claims.

### Selected and muted rows

Machine/software selected-row behavior is represented by explicit selected classes, selected ARIA state, and MAME palette tokens. Software Browser rows now use selected blue/yellow treatment and muted unsupported/partial treatment. Machine-list selected-row visibility is covered by the deterministic visibility helper and tests.

### Green driver/status bottom region

`MachineDriverStatus` remains below the main browser grid and is the final persistent region in default machine-browser mode. Permanent generic footer chrome below it was removed. Configure Options, Audit, History, Collections, Diagnostics, and Session remain reachable through the compact secondary tools affordance rather than a permanent bottom footer.

### Secondary tools visibility

History, Collections, Diagnostics, Audit, Configure Options, and Session are not permanently visible as default footer chrome. They remain available through the compact `More` utility menu or contextual action paths, preserving access without reintroducing a generic dashboard row.

### Software Browser

The Software Browser uses MAME palette tokens, compact toolbar/header treatment, dense software rows, selected treatment, muted unsupported/partial states, visible focus outlines, tokenized splitters/borders, and in-surface error banners. Component coverage renders the software result rows server-side and verifies selected ARIA/class state plus support metadata. The remaining limitation is absence of screenshot artifacts for pixel density.

### Images and Infos tabs

The right panel exposes Images/Infos as tabs with synchronized `aria-selected`, tab index, and focus targets. ArrowRight moves Images to Infos, ArrowLeft moves Infos to Images, and the left edge returns focus toward the machine list instead of wrapping. Gameplay-owned input guards prevent the right panel from stealing keyboard input while gameplay owns the surface.

### Artwork loading, missing, ready, and error states

Artwork state is split into explicit loading, missing, ready, and error states. Loading renders `Loading <label>…` instead of `No image Available`; true missing artwork retains the placeholder; formatted errors render as alerts. Stale discovery and asset responses are invalidated by request identity on machine and slot/category changes.

### Metadata states

The catalog-state panel continues to represent metadata/executable states before the machine list is queryable. Import/checking/failure/value-required states clear selection through the same invalidation paths that protect stale detail and action state.

### Configuration surface and host-theme leakage

MTR-005 and MTR-006 replaced visible product host-system colors with semantic MAME tokens across parity-relevant surfaces, including Software Browser and configuration-adjacent flows. Remaining static tripwires verify disallowed system-color product styling does not return.

### Focus states

Visible focus treatment is retained through MAME focus tokens and keyboard navigation contracts. Right-panel tab focus, selected machine focus, Software Browser row focus, and search/escape shortcut paths are covered by the relevant helper/source/component tests. Actual browser focus painting still requires screenshot/pixel capture for visual proof.

## Limitation and defer rationale

Ralph Bridge currently exposes CI/job state and artifact metadata, but not a binary screenshot capture from the Tauri/WebView smoke run. The available artifacts for the promoted Tauri project run are qualification/report artifacts rather than screenshots. Because of that limitation, this review records a complete textual rendered review and the exact runtime smoke evidence, while explicitly avoiding a pixel-baseline claim.

A future GUI/WebView-capable self-hosted runner that uploads screenshots would strengthen this section. If such artifacts become available, this file should be amended with links/names for the screenshot artifacts and the exact run IDs that produced them.

## MTR-013 result

MTR-013 is satisfied as a textual rendered re-qualification under the current Ralph-accessible validation path:

- the actual Tauri/WebView application was launched by the promoted Tauri development window smoke check;
- no screenshot artifacts are available through Ralph;
- the visual checklist is updated and reviewed item-by-item;
- the limitation is explicit and does not claim pixel evidence.
