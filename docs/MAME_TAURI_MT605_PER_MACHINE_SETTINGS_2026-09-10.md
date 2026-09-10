# MT-605 — Per-machine settings

**Date:** 2026-09-10  
**Repository:** `ekkus93/mame`  
**Task:** MT-605 — Add per-machine settings  
**Parent:** MT-604 closure `e815ba6ec699ede34c262dcab1b0e47eabd5a697`  
**Qualified implementation:** `0da2499ba6972e1547f136fdd2425efef33b20ee`  
**Implementation tree:** `35a487094d9ca6afb745b0d9ec68dcd5e7f52f03`  
**Exact-head Tauri qualification:** `34465366766`

## Result

MT-605 adds durable per-machine launch preferences for the bounded window, renderer, and audio settings introduced by MT-604.

The machine override model uses the same typed preference enums as general settings. At the machine layer, `inherit` means that the corresponding general preference remains effective. An explicit machine value overrides only that field; unrelated fields continue to inherit.

## Override storage

Per-machine overrides are user-owned state in the catalog SQLite database. Schema version 3 adds `machine_launch_preferences`, keyed by machine short name, with the bounded preference object serialized as JSON.

The table intentionally has no foreign key into generated MAME metadata. Metadata refresh or regeneration therefore cannot cascade-delete user settings for a machine that temporarily disappears or is renamed upstream.

A fully inherited preference set is storage-empty: saving all three fields as `inherit`, or using the dedicated reset command, deletes the machine row rather than persisting a redundant override.

## Launch integration

The supervised launch path now resolves launch preferences in this order for each field:

```text
machine override, when explicit
    > general launch preference
    > MAME/native behavior when the resulting value is inherit
```

Resolution happens in the shared launch path, so both direct machine launches and software launches associated with that machine receive the same machine-specific settings.

No arbitrary argv strings are accepted and no MAME-owned INI/CFG file is edited.

## UI and typed commands

The machine-detail panel now exposes a **Machine launch settings** section with:

- per-machine window mode, renderer, and audio override selectors;
- an explicit **Reset to inherited** action;
- the currently resolved values that will be requested for new launches.

The frontend uses dedicated typed commands:

- `get_machine_launch_settings`
- `set_machine_launch_settings`
- `reset_machine_launch_settings`

## Tests

Coverage added for:

- persistence and reload of a machine override;
- physical deletion of the override row on reset;
- field-by-field effective-value resolution against general preferences;
- storage-empty handling for an all-inherited override;
- frontend command names and request shapes;
- migration to schema version 3 while preserving existing user state.

## Qualification

Exact-head run `34465366766` passed on `0da2499ba6972e1547f136fdd2425efef33b20ee`:

- frontend format check;
- frontend lint;
- TypeScript typecheck;
- frontend tests;
- frontend production build;
- Rust format;
- Rust tests;
- inherited 100,000-row library performance qualification;
- Clippy with warnings denied;
- lockfile integrity.

## Scope boundary

MT-605 displays resolved preference values but deliberately does not yet provide full configuration provenance/explainability. Source-layer display and pending transient launch override explanation remain MT-606.