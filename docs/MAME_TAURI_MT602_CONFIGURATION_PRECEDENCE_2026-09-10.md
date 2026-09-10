# MT-602 — Deterministic Configuration Precedence

**Date:** 2026-09-10  
**Repository:** `ekkus93/mame`  
**Task:** MT-602 — Define deterministic configuration precedence  
**Parent:** MT-601 closure `c673f14d100d0248a387c88f44e2c5bd78456b72`  
**Qualified implementation:** `5e0a2aa77771021c6f8bbc7f19d0737d55f4d31c`  
**Exact-head Tauri qualification:** `34453141798`

## Purpose

MT-602 defines the deterministic application-level precedence used to calculate a requested MAME option value. It builds on the source inventory in MT-601 without replacing MAME's own native INI and runtime fallback behavior.

The executable policy lives in:

- `tauri/src-tauri/src/configuration_precedence.rs`

and is exported through:

- `tauri/src-tauri/src/lib.rs`

## Precedence contract

From lowest to highest priority:

```text
application defaults
MAME/global defaults
profile defaults
machine overrides
transient launch overrides
```

The Rust enum is `ConfigurationLayer`, with explicit `LOW_TO_HIGH` and `HIGH_TO_LOW` constants so the order is reviewable and testable rather than implied by collection order.

For an option with values present at several layers, the highest present layer wins:

```text
TransientLaunchOverrides
  > MachineOverrides
  > ProfileDefaults
  > MameGlobalDefaults
  > ApplicationDefaults
```

If no layer supplies the option, resolution returns no effective application value.

## MAME/native boundary

`MameGlobalDefaults` is the application model's input representing the effective value supplied by MAME's native compiled/default and standard INI machinery. MT-602 does not flatten MAME's internal INI priorities into application-owned profile layers.

MAME still owns the native ordering documented by MT-601, including main MAME INI, orientation/output/source/parent/machine INIs and platform OSD options.

Transient application launch overrides are intended to become real argv options. MAME's own command-line priority therefore remains authoritative once the process is launched.

## Conflict behavior

Cross-layer conflicts are expected and deterministic: the higher layer wins.

Within a single layer, duplicate assignments for the same option are rejected with:

```text
CONFIG_PRECEDENCE_CONFLICT
```

MT-602 deliberately does not use same-layer last-write-wins semantics. That prevents effective values from changing merely because an unordered or differently ordered collection was supplied.

## Requested versus observed effective value

`EffectiveConfigurationValue<T>` records:

- the requested value selected by application precedence;
- the `source_layer` that won.

This is application-level explainability, not proof that a MAME backend accepted the value exactly. Renderer/audio providers may reject an unsupported requested provider and fall back to an automatic provider. Later configuration explainability must therefore keep these concepts distinct:

```text
requested value + application source layer
observed runtime effective value/fallback, when available
```

MT-602 does not silently rewrite a requested value to match an assumed backend fallback.

## Effective-value calculation tests

The resolver test suite covers:

- explicit stable low-to-high and high-to-low ordering;
- highest present layer wins regardless of assignment input order;
- removing higher layers reveals the next lower value;
- MAME/global defaults override application defaults;
- duplicate assignments within one layer are rejected;
- an option absent from every layer has no resolved value.

## Qualification

Initial candidate CI `34452664081` passed every frontend gate but stopped at `cargo fmt --check`. The rustfmt-only correction was validated on candidate run `34452881676`, which passed all substantive gates.

The implementation was then normalized to one clean commit directly on the MT-601 closure head:

```text
5e0a2aa77771021c6f8bbc7f19d0737d55f4d31c
```

Exact-head run `34453141798` passed:

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

MT-602 defines policy only. It does not yet persist profiles or machine overrides, edit MAME-owned configuration files, or add settings UI. Those concerns remain in MT-603 and later tasks.
