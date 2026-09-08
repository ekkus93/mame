# MAME Tauri Decision Record — DG-1 Repository Layout

**Decision ID:** DG-1  
**Date:** 2026-09-08  
**Repository branch:** `ralph/mt-000-foundations`  
**Baseline master SHA:** `c45b774487a97423a97e84a9a84f388752d1da8e`  
**Status:** Accepted

## Question

Where should the Tauri/React/Rust application live inside the MAME fork so that the project can use a conventional Tauri workspace while minimizing collisions and merge cost with upstream `mamedev/mame`?

## Options evaluated

### A. Root-level `src-tauri/` + `frontend/`

Advantages:

- close to the initial architecture sketch;
- obvious separation between frontend and Rust backend.

Disadvantages:

- adds multiple generic project roots to an already very large upstream repository;
- puts Node tooling at/near the MAME repository root;
- increases namespace/collision risk over a long-lived upstream fork;
- makes the Tauri application less self-contained.

### B. Root-level `ui/` + `src-tauri/`

Advantages:

- compact naming.

Disadvantages:

- generic `ui` naming is ambiguous in a repository that already contains substantial MAME frontend/UI code;
- same root pollution concerns as option A.

### C. Self-contained `tauri/` workspace

Layout:

```text
tauri/
├── package.json
├── src/
└── src-tauri/
```

Advantages:

- matches the normal Tauri/Vite relationship between frontend and `src-tauri`;
- keeps npm, Vite, TypeScript, React, Cargo, and Tauri configuration in one project-owned root;
- avoids collision with upstream `src/frontend`;
- limits normal project changes to one top-level namespace;
- improves path-based CI and divergence auditing;
- can be removed or moved without touching MAME core source.

Disadvantages:

- documentation must make clear that `tauri/src` is a frontend source directory and not MAME C++ source;
- commands are normally run from `tauri/`, not repository root.

## Evidence

At the planning/audit baseline there was no existing top-level `tauri/` namespace in the fork. The standard Tauri 2 project model places frontend sources and `src-tauri/` in one application directory. The project also has a standing requirement to isolate Tauri-specific work from upstream MAME.

## Performance data

Not applicable. Repository placement has no runtime-path effect.

## Cross-platform data

The selected layout is platform-neutral and is compatible with standard Tauri/Vite tooling on Linux, Windows, and macOS.

## Upstream-maintenance impact

Option C provides the smallest expected recurring merge surface because ordinary frontend/backend work remains under one project-owned root and does not modify upstream MAME source.

## Security impact

Neutral-to-positive. A self-contained workspace makes it easier to scope CI, Tauri capabilities, dependency inventories, and project-specific filesystem permissions.

## Decision

Adopt **Option C: `tauri/` self-contained workspace**.

Reserved related namespaces:

```text
scripts/tauri/
tests/tauri/
docs/MAME_TAURI_*
src/osd/tauri/   # only if the dedicated OSD phase is approved
```

## Rejected alternatives

Root-level `frontend/`, `ui/`, and `src-tauri/` layouts are rejected for the initial implementation because they spread project-owned files across generic top-level namespaces without providing a material tooling advantage.

## Revisit conditions

Revisit only if:

- a supported Tauri tool requires an incompatible layout;
- packaging/build tooling demonstrably cannot operate cleanly from `tauri/`;
- an upstream MAME directory collision is introduced;
- a future monorepo/workspace architecture provides measurable maintenance benefits.