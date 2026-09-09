# MT-408 — Controller-friendly frontend navigation research

Date: 2026-09-09

## Decision

Use the browser Gamepad API only as a frontend navigation adapter while the Tauri window owns UI input. Do not forward controller events to MAME and do not poll controller navigation while a MAME gameplay session is active or while the Tauri document is unfocused.

MT-408 is a research/prototype task. It deliberately does not wire gamepad polling into `LibraryBrowser` yet. Production integration should happen only after the focus model below is accepted and after controller ownership can reuse the same fail-closed Rust session-state check introduced by MT-407.

## Desired standard-gamepad behavior

The initial target is the W3C `standard` gamepad mapping. Non-standard mappings remain unsupported until explicit per-controller mapping exists.

| Control | Desired frontend behavior |
| --- | --- |
| D-pad Up / left stick up | Move to the previous machine in the active machine list. |
| D-pad Down / left stick down | Move to the next machine in the active machine list. |
| South / A button | Activate the currently focused frontend control. |
| D-pad Left / Right | Reserved for moving between major UI regions after the region model is implemented. |
| East / B button | Reserved for returning to the previous region/layer. |
| Start / Guide | Not consumed by the frontend prototype. |

The prototype emits edge-triggered actions only. Holding a D-pad direction or the primary button does not generate an action on every poll. The left stick uses an enter threshold of `0.70` and a release threshold of `0.35` so analog noise near center does not cause repeated navigation.

## Input-ownership safety model

Controller navigation must fail closed.

The frontend may consume a controller sample only when all of these are true:

1. Rust has established that no MAME session is in `created`, `starting`, `running`, or `stopping` state.
2. The Tauri document has focus.
3. The gamepad is connected.
4. The gamepad reports the W3C `standard` mapping.

When any condition becomes false, the navigation prototype resets to a de-armed state. When ownership later returns, the controller must first produce one neutral sample before new input is accepted. This prevents a direction or button held during gameplay from becoming an accidental frontend action when focus returns to Tauri.

This is intentionally stricter than simply checking `document.hasFocus()`: the Tauri window can regain focus while a supervised MAME session is still alive, so the Rust session state remains authoritative.

## Focus transitions

The production focus model should preserve one visible focus target and keep keyboard and controller selection synchronized.

### Machine-list region

- Entry: first controller navigation action focuses the already-selected machine row; if no row is selected, focus the first visible row.
- Up/Down: move both selection and DOM focus to the adjacent row, bounded at the first/last visible machine.
- South/A on a row: activate that row using the same semantic action as mouse/keyboard activation; do not synthesize raw keyboard events.
- Pagination: when moving to another page, selection/focus should land on the first corresponding visible row after data loads.

### Detail/action region

- A future Left/Right region transition may move focus between the machine list and the detail action group.
- South/A activates the focused button such as Launch or Favorite.
- East/B returns focus to the selected machine row.
- Launch must continue to update the MT-407 gameplay-ownership gate immediately when Rust returns an active session state.

### Filter/search region

- A future region transition may move focus into the search/filter controls.
- Controller text entry is out of scope for MT-408; platform/on-screen keyboard behavior should be evaluated separately.
- East/B returns focus to the selected machine row without mutating filter contents.

## Prototype implementation

`tauri/src/library/controllerNavigation.ts` contains a pure state machine that can be polled by a future Gamepad API adapter. It provides:

- explicit frontend-vs-gameplay ownership gating;
- standard-mapping/connectivity gating;
- neutral re-arming after ownership loss;
- D-pad vertical edge detection;
- left-stick threshold and hysteresis handling;
- primary-button activation edge detection;
- no automatic repeat while an input remains held.

`tauri/src/library/controllerNavigation.test.ts` covers the ownership, neutral re-arm, edge, hysteresis, activation, and mapping/disconnect cases.

## Integration recommendation

A later implementation should poll `navigator.getGamepads()` with `requestAnimationFrame` only while the frontend gate is open. It should translate the first accepted standard gamepad into the pure state machine above and then invoke semantic React actions (`setSelected`, `.focus()`, or an existing button callback). It should never dispatch synthetic keyboard events and should stop/reset polling actions as soon as MT-407's session ownership check reports active gameplay.

Multiple simultaneously connected controllers, controller identity persistence, remapping, horizontal region transitions, autorepeat timing, and controller-driven text entry are intentionally deferred until real hardware UX testing establishes requirements.
