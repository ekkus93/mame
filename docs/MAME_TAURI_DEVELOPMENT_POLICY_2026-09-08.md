# MAME Tauri Development Policy

**Document date:** 2026-09-08  
**Repository:** `ekkus93/mame`  
**Applies to:** Tauri modernization work tracked by `docs/MAME_TAURI_TODO_2026-09-08.md`  
**Status:** Active

---

## 1. Purpose

This document freezes the project-level development, repository-layout, coding, licensing, and upstream-synchronization policies required by MT-000.

The policies are intentionally designed to keep Tauri-specific work isolated from upstream MAME wherever possible, preserve exact-head validation, and make routine synchronization with `mamedev/mame` tractable.

---

## 2. Architecture baseline audit

Planning baseline:

```text
7cc3033a50b00801240f020b7098e22ffffdcd44
```

At the MT-000 audit, fork `master` was:

```text
c45b774487a97423a97e84a9a84f388752d1da8e
```

The fork was exactly two commits ahead of the planning baseline, and both commits were project planning documents. No MAME C/C++ source, build-system source, OSD implementation, machine driver, device implementation, renderer, input module, or upstream workflow had changed relative to the planning baseline.

Therefore there is **no implementation-level architectural drift** to reconcile at MT-001.

The following architecture decisions remain confirmed:

1. **Sidecar first.** The first production-useful architecture supervises a native MAME child process.
2. **Native hot paths.** Video frames, PCM audio, emulation timing, and gameplay input do not traverse JavaScript/WebView IPC.
3. **Embedded rendering is optional.** MT-1000 is a research track and is not required for the external-window product milestone.
4. **Dedicated Tauri OSD is gated.** MT-1100 begins only after an evidence-backed embedded-render decision.
5. **In-process hosting is optional and gated.** MT-1200 requires a documented product justification; technical curiosity is not sufficient.
6. **MAME remains authoritative.** Machine metadata, audit semantics, emulation behavior, and state serialization remain MAME-owned.

---

## 3. Repository layout — DG-1

The final project-owned application namespace is:

```text
tauri/
├── package.json
├── package-lock.json
├── index.html
├── tsconfig*.json
├── vite.config.ts
├── eslint.config.*
├── src/                     React + TypeScript frontend
├── public/                  frontend static resources only
└── src-tauri/               Rust/Tauri backend
    ├── Cargo.toml
    ├── Cargo.lock
    ├── build.rs
    ├── tauri.conf.json
    ├── capabilities/
    ├── icons/
    ├── src/
    └── tests/
```

Additional project-owned namespaces are reserved as follows:

```text
scripts/tauri/               repository-level integration/generation tooling
tests/tauri/                 repository-level fixtures/integration data when a test cannot live in tauri/
docs/MAME_TAURI_*            architecture, decisions, operations, qualification, and handoff documents
src/osd/tauri/               reserved; created only if MT-1100 is approved
```

### 3.1 Rationale

Using a single top-level `tauri/` workspace keeps the standard Tauri/Vite layout intact while avoiding Node/Rust project files at the MAME repository root. It also minimizes collisions with upstream MAME's existing `src/frontend` and `src/osd` namespaces.

No new generic top-level `frontend/`, `ui/`, `app/`, or `rust/` directory is introduced.

### 3.2 Ownership rule

Files under `tauri/`, `scripts/tauri/`, `tests/tauri/`, and `docs/MAME_TAURI_*` are project-owned unless explicitly documented otherwise. Changes elsewhere in the MAME tree require an explicit rationale in the PR and must be counted as upstream-divergence surface.

---

## 4. Branch and merge policy

### 4.1 Integration branch

`master` is the fork integration branch. Normal executable development does not occur directly on `master`.

### 4.2 Feature branches

Ralph-loop branches use:

```text
ralph/mt-<task-or-range>-<short-slug>
```

Examples:

```text
ralph/mt-000-foundations
ralph/mt-100-scaffold
ralph/mt-200-sidecar
ralph/mt-300-metadata
ralph/mt-1000-native-window-research
```

A branch may close a tightly coupled task range when splitting it would create artificial intermediate states, but the PR must state exactly which TODO items it claims.

### 4.3 Pull requests

Executable behavior changes must reach `master` through a PR. Documentation changes that accompany executable work stay on the same branch/PR.

Documentation-only direct commits to `master` are permissible only when all of the following hold:

- no executable behavior changes;
- no build or CI behavior changes;
- no security or release policy is weakened;
- the change is planning, handoff, evidence recording, or typo/copy maintenance;
- direct commit is deliberate rather than used to avoid review/CI.

Once executable implementation exists, PRs remain preferred even for substantive documentation changes.

### 4.4 Exact-head rule

For executable changes, the commit merged must be the same PR head SHA that passed all required checks. If the branch head changes after a green run, the new head must be requalified.

A merge must not rely on a green run from an ancestor commit when later executable changes are present.

### 4.5 Merge method

Prefer squash merge for project feature branches unless preserving a multi-commit history has a clear maintenance benefit. Upstream synchronization PRs are an exception and should normally preserve the upstream merge relationship.

### 4.6 Branch cleanup

After merge and post-merge verification:

- delete merged `ralph/mt-*` branches;
- delete obsolete superseded experiment branches;
- retain a research branch only when an active decision gate explicitly depends on it;
- do not use stale branches as long-term evidence storage—record durable evidence in `docs/`.

---

## 5. Upstream synchronization procedure

The canonical upstream is:

```text
https://github.com/mamedev/mame.git
```

Recommended local procedure:

```bash
git remote add upstream https://github.com/mamedev/mame.git  # first time only
git fetch upstream
git switch master
git pull --ff-only origin master
git switch -c sync/upstream-YYYYMMDD
git merge --no-edit upstream/master
```

Then:

1. resolve conflicts with preference for preserving upstream MAME behavior;
2. review every conflict touching project-owned or patched MAME files;
3. run the project validation required by the changed paths;
4. push the sync branch;
5. open a PR to `master` documenting upstream old/new SHAs and any conflicts;
6. merge only after required exact-head validation;
7. verify `master` and delete the sync branch.

### 5.1 Rebase policy

Do not routinely rebase or rewrite shared `master` history to follow upstream. Project feature branches may be rebased before merge when useful. Upstream integration should normally be represented as a merge into a sync branch so fork history remains auditable.

### 5.2 Conflict policy

Broad recurring conflicts in MAME core files are an architectural warning. Repeated conflicts should trigger an effort to move project code behind a narrower extension point rather than accepting permanent conflict cost.

---

## 6. Rust coding standard

Rust code under `tauri/src-tauri` follows:

- stable Rust unless a documented requirement needs otherwise;
- `rustfmt` canonical formatting;
- `cargo fmt --check` in CI;
- `cargo clippy --all-targets --all-features -- -D warnings` unless a narrowly documented platform exception exists;
- `cargo test` for unit/integration tests;
- explicit error types at subsystem boundaries;
- no ignored `Result` from operations whose failure changes behavior;
- no `unwrap()`/`expect()` in normal runtime paths unless the invariant is local, proven, and documented;
- bounded queues/buffers for child output and event payloads;
- no generic shell execution command exposed to the frontend.

### 6.1 Rust modules

The initial backend module responsibilities are:

```text
app          Tauri composition/root lifecycle
config       settings schema, migration, effective configuration
errors       stable command-facing error taxonomy/envelope
mame         executable identity, argv construction, invocation
metadata     MAME metadata generation/parsing/import
library      catalog query/domain operations
storage      SQLite/filesystem persistence primitives
sessions     supervised MAME process lifecycle/state machine
platform     OS-specific paths/process/window integration
```

Dependencies should point inward toward narrow domain interfaces. Platform code must not become the default home for application logic.

---

## 7. TypeScript/React coding standard

Frontend code under `tauri/src` follows:

- TypeScript strict mode;
- `noUncheckedIndexedAccess` enabled unless a demonstrated library/tooling incompatibility prevents it;
- ESLint with no ignored type/lint failures in CI;
- Prettier or an equivalent deterministic formatter once configured;
- functional React components by default;
- hooks for local UI behavior;
- explicit boundary modules for Tauri commands/events;
- backend-owned session/catalog state is not independently re-invented as contradictory frontend truth;
- asynchronous views have explicit loading, empty, success, and error states;
- lists representing the full MAME catalog must be paginated or virtualized.

A React class component is acceptable where the platform API still requires one, such as a dependency-free error boundary.

### 7.1 Tests

Frontend unit/component tests are named `*.test.ts` / `*.test.tsx` and normally live next to the module they test. Repository-scale fixtures may live under `tests/tauri/`.

---

## 8. Serialization and command conventions

Rust structures exposed to the frontend use Serde with explicit naming policy. The command boundary uses JavaScript-friendly `camelCase` field names unless an external protocol dictates otherwise.

All command failures use a stable envelope conceptually equivalent to:

```json
{
  "code": "MAME_EXECUTABLE_NOT_FOUND",
  "message": "The configured MAME executable does not exist.",
  "details": {},
  "retryable": false
}
```

Rules:

- `code` is stable and machine-readable;
- `message` is concise and user-presentable;
- `details` is bounded structured diagnostic context, not an arbitrary exception dump;
- `retryable` is explicit where meaningful;
- internal paths/secrets are not exposed unless necessary for the user to resolve the problem.

Backend events use namespaced lowercase names such as `session.started` and include an explicit payload schema version once they become persistent product contracts.

---

## 9. C/C++ policy

Future changes to MAME C/C++ code must:

- follow surrounding MAME style;
- preserve upstream license headers;
- minimize whitespace-only churn;
- avoid broad refactors bundled with Tauri work;
- prefer `src/osd/tauri/` or existing module interfaces over edits to `src/emu`, `src/devices`, or machine drivers;
- include the relevant native MAME validation for the touched subsystem.

---

## 10. Licensing and trademark constraints

This section records engineering constraints, not legal advice.

### 10.1 MAME licensing

Repository `COPYING` states that MAME as a whole is distributed under GNU GPL version 2, while individual source files may carry less restrictive licenses and full license texts are stored under `docs/legal`.

Engineering consequences:

- a release that redistributes a MAME binary must preserve all required notices/license material;
- release engineering must satisfy the GPL source-availability obligations applicable to the exact redistributed MAME binary and project modifications;
- third-party components included in MAME must retain any required notices documented by the source/distribution;
- the `hash` directory has its own CC0 dedication as documented by MAME and must not be assumed to share the licensing of unrelated files;
- no release process may strip upstream license headers or `COPYING`/required legal material.

Before a public bundled-MAME release, MT-1307 must produce a concrete license inventory for the exact artifacts being shipped.

### 10.2 Trademark

MAME's repository states that **MAME is a registered trademark of Gregory Ember** and that permission is required to use the MAME name, logo, or wordmark in ways covered by those rights.

Therefore:

- the working project may use descriptive repository/document language necessary to explain interoperability;
- release branding, application name, iconography, store listing, and marketing must not assume permission to use the MAME logo/wordmark;
- a final product name/branding review is required before distribution.

### 10.3 Tauri/Rust/JavaScript dependencies

Dependency lockfiles are mandatory once the scaffold exists. CI/release tooling must be capable of producing an inventory covering:

- Rust crates and licenses;
- npm dependencies and licenses;
- Tauri runtime/CLI components;
- redistributed native libraries/resources.

No dependency is considered redistributable merely because it installed successfully.

### 10.4 ROM/software content

The project does not bundle copyrighted ROM images, disk images, CHDs, console/computer software, firmware not already lawfully redistributable with MAME, or other game/software content.

The application manages user-selected local paths and reports MAME's audit results; it does not become a content-distribution mechanism.

---

## 11. Validation policy by change class

### Documentation only

At minimum:

- inspect Markdown rendering/content;
- verify referenced paths/task IDs;
- no executable CI is required unless the documentation changes CI/release semantics.

### Tauri frontend only

Required baseline once MT-108 exists:

- formatter check;
- ESLint;
- TypeScript typecheck;
- unit/component tests;
- production frontend build.

### Rust/Tauri backend

Required baseline once MT-109 exists:

- `cargo fmt --check`;
- clippy policy above;
- `cargo test`;
- release/smoke compile appropriate to the target.

### MAME-side changes

Run the smallest representative MAME validation that covers the change, plus project Tauri checks. Changes to OSD/core/build integration may require `SUBTARGET=tiny`, relevant platform builds, or upstream-equivalent CI according to scope.

---

## 12. Foundation decisions frozen by MT-000

| Decision | Result |
| --- | --- |
| Architecture | sidecar-first |
| Real-time video path | native only; no WebView framebuffer IPC |
| Real-time audio path | native only; no WebView PCM IPC |
| Gameplay input path | native MAME path during sidecar phases |
| Embedded rendering | optional research gate |
| Dedicated OSD | optional, gated by MT-1010/DG-4 |
| In-process MAME | optional, gated by explicit justification |
| Application workspace | `tauri/` |
| Frontend | `tauri/src/` |
| Rust backend | `tauri/src-tauri/` |
| Project integration scripts | `scripts/tauri/` |
| Shared project fixtures | `tests/tauri/` |
| MAME-side Tauri OSD namespace | `src/osd/tauri/`, reserved only |
| Development branches | `ralph/mt-<task-or-range>-<slug>` |
| Master policy | integration branch; executable changes via PR |
| Upstream integration | sync branch + reviewed merge into `master` |

These decisions may be revised only by a later durable decision record that states the evidence and consequences.