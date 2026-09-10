# MT-606 — Configuration explainability closure

Date: 2026-09-10

## Status

MT-606 is implemented and qualified.

Implementation commit:

`420391dd6d98ae1c6666f94be368a8868ee15690`

Implementation tree:

`21925fd38b4f173fa5a67999c3facd8a4aa7a109`

Authoritative Tauri qualification:

`34469435315`

The exact-head workflow completed successfully, including frontend formatting, lint, TypeScript type checking, frontend tests, production build, Rust formatting, Rust tests, the inherited 100k-row library UX performance qualification, Clippy, and lockfile-integrity verification.

## Explainability contract

MT-606 reuses the MT-602 deterministic precedence engine rather than defining a second configuration-ordering policy. For each bounded launch preference, the backend reports:

- the effective requested value for the next launch;
- the precedence layer that supplied that value; and
- any pending one-shot launch override.

The app-owned layers map as follows:

- general launch settings -> `profileDefaults`;
- persisted per-machine settings -> `machineOverrides`;
- one-shot next-launch settings -> `transientLaunchOverrides`.

When none of those application-owned layers supplies a value, the explanation reports `mameGlobalDefaults` with the existing `inherit` sentinel. `inherit` means the application emits no command-line override and leaves resolution to MAME's native configuration stack. It does not claim that the frontend has observed MAME's eventual concrete renderer, audio provider, or other runtime-selected value.

## Real transient launch layer

The pending launch override is executable behavior, not display-only state.

The machine-detail UI can stage bounded one-shot window, renderer, and audio values. The same object is sent to the Rust backend and participates in the same MT-602 precedence calculation used to build the actual launch preferences.

The transient layer has higher precedence than persisted per-machine and general settings. It is sent through both machine and software-list launch paths.

A pending override is cleared by the frontend after a successful launch. A failed launch retains it so the user can correct the failure and retry without silently losing the requested one-shot configuration. Selecting a different machine clears the machine-scoped pending state.

## Requested versus observed runtime state

Renderer and audio provider values are described as requested launch configuration. MAME may perform its own provider fallback at runtime. MT-606 therefore does not label an unobserved fallback provider as active or effective runtime state.

This distinction preserves the configuration subsystem's explainability guarantee: the UI explains what the application will request and why that value wins, while remaining explicit about values that are ultimately controlled or resolved inside MAME.

## Implementation surface

The implementation adds a reusable Rust explainability module and integrates it into per-machine settings and launch resolution. The frontend exposes the returned explanation next to the persistent and transient controls.

The implementation also extends the bounded launch request envelopes with optional transient launch preferences. No arbitrary command-line argument string is accepted from the WebView.

## Tests

Backend coverage verifies:

- a machine override wins over general settings and reports `machineOverrides`;
- an inherited machine field resolves to the general value and reports `profileDefaults`;
- a transient value wins over all lower application layers and reports `transientLaunchOverrides`;
- the pending value is explicitly returned when present;
- an all-inherited configuration reports MAME/global ownership without inventing a concrete runtime value; and
- effective launch preferences reconstructed from the explanation match the values passed to launch argument construction.

Frontend transport coverage verifies that bounded pending launch preferences are included in the dedicated machine-settings explanation request when present.

## Acceptance

MT-606 acceptance requirements are satisfied:

- show effective value;
- show source layer; and
- show pending launch override.
