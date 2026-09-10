# MT-608 — Controller Configuration UI Qualification

**Date:** 2026-09-10  
**Repository:** `ekkus93/mame`  
**Task:** MT-608 — Controller configuration UI  
**Parent closure:** MT-607 `8892a237e76c14202e09795254fb640d724edb1a`

## Qualified implementation

- Implementation SHA: `7c38ef7139f66e7fceaf7584d17777ad5ab4da25`
- Implementation tree: `273f76452151621c3ff4258b368ecdc898527dca`
- Exact-head Tauri workflow: `34483298119`
- Result: PASS

The exact-head workflow passed frontend formatting, lint, TypeScript typecheck, frontend tests, production build, Rust formatting, Rust tests, the inherited MT-410 100k-row library performance qualification, Clippy with warnings denied, and lockfile integrity.

An earlier candidate `ddc04cf721aa4b5cfff6cdd2a6ae747d11d8a519` passed frontend, Rust tests, and performance but failed Clippy solely because `controller_settings.rs` retained an unused `get_controller_profile` import. The final qualified SHA removes that import and otherwise preserves the intended MT-608 product tree.

## Implemented behavior

MT-608 adds a typed controller-configuration surface backed by the MT-607 controller-profile model.

### Inspect selected/effective/active state

The backend response distinguishes three concepts rather than conflating them:

- the profile assigned to the requested scope;
- the effective selected profile after scope precedence is resolved;
- the profile actually active in MAME.

Machine-scoped selection takes precedence over the global selection; a machine with no machine-specific assignment inherits the global selection.

The current build does not yet translate saved controller profiles into MAME gameplay configuration through `-ctrlr`, `controller_map`, or another native mapping mechanism. Consequently `activeProfile` remains `null` and the UI reports that no saved profile is active in MAME. This is intentional truthfulness, not an incomplete status display.

### Assign/select profile

The controller settings backend supports selecting one profile for global scope or for an exact machine scope. Re-selecting a scope replaces its prior assignment rather than accumulating conflicting active selections. The General Settings surface exposes global controller configuration; the per-machine settings surface exposes machine-scoped configuration.

### Controller capture and mapping support

The frontend can inspect connected browser Gamepad API devices and create a controller profile from devices reporting the W3C `standard` mapping. Non-standard browser mappings are displayed as unsupported for capture and are rejected by the backend if submitted directly.

This does not route gameplay input through the WebView. Browser gamepad inspection is configuration metadata only; gameplay input remains native to MAME.

## Tests and acceptance evidence

Backend tests cover:

- global selection inherited by a machine with no machine override;
- machine selection overriding global selection while remaining explicitly not applied to MAME;
- replacement of an existing exact-scope selection;
- rejection of non-standard browser Gamepad mappings.

Frontend command-contract tests cover the typed controller-profile commands. The component presents selected/effective state independently from the explicit active-in-MAME state, preventing unsupported or unapplied mappings from being represented as active.

## MT-608 acceptance

The implementation satisfies the task requirements:

- inspect active profile: the UI/backend explicitly inspect and distinguish assigned, effective-selected, and actually-active state;
- assign/select profile: global and machine-scoped selections are persisted and resolved deterministically;
- avoid pretending unsupported mappings are active: unsupported browser mappings cannot be captured as supported, and no saved profile is reported active in MAME until a native application mechanism exists and is evidenced.
