# MAME Tauri Post-Closeout Hardening Specification — 2026-09-13

## 1. Purpose

This specification defines the post-closeout hardening batch created after MT-2200 engineering closure. It does not reopen the production-useful external-window Tauri frontend engineering phase. It tightens several review findings that were identified after MT-2200 was promoted to `master`.

The hardening batch is intentionally bounded. It improves user-visible session controls, lifecycle synchronization, runtime-control security documentation/regression coverage, generated-source guardrails, and frontend/backend session typing.

## 2. Non-goals

This batch does not implement or reclassify the optional research tracks:

- MT-1000 native-window / embedded-rendering research;
- MT-1100 dedicated Tauri MAME OSD;
- MT-1200 in-process MAME hosting;
- MT-1705 embedded-render performance qualification.

It also does not add PCM transport through Tauri, generic shell execution, public bundled MAME release artifacts, Apple notarization credentials, or any broad upstream MAME source-tree refactor.

## 3. Review findings addressed

### PCH-001 — Visible runtime-control UI

The backend and typed command wrappers already expose pause, resume, soft reset, mute/unmute, and runtime-state query. Before this batch, the session panel exposed only stop and save-state operations. The UI must surface all supported runtime-control operations through explicit controls while keeping the WebView outside the video/audio/gameplay hot path.

Acceptance criteria:

- The session panel imports and uses `pauseMame`, `resumeMame`, `resetMame`, `setMameMute`, and `queryMameRuntimeState`.
- Controls are disabled unless the current supervised session is running.
- Pause/resume buttons reflect the observed runtime pause state when available.
- Mute/unmute reflects the observed native user-mute/effective mute state when available.
- Every operation reports success/failure through the existing UI notice/error pattern.
- No command constructs shell strings or bypasses the typed Tauri command wrappers.

### PCH-002 — Shortcut/gameplay ownership refresh after terminal session events

Library keyboard shortcuts deliberately fail closed while MAME owns gameplay input. Before this batch, ownership refreshed on mount/focus but could remain stale after terminal session events until focus changed.

Acceptance criteria:

- `LibraryBrowser` listens for `session.started`, `session.exited`, `session.crashed`, and `session.failed`.
- `session.started` refreshes ownership from Rust instead of guessing.
- Terminal session events immediately release application shortcuts locally.
- Event-listener setup failures preserve fail-closed behavior.

### PCH-003 — Bootstrap-file permission semantics

The runtime-control bootstrap script contains an unguessable per-session frame token. Unix builds restrict the file to `0600`. Non-Unix builds cannot apply POSIX mode bits through portable Rust stdlib APIs and previously used a silent no-op.

Acceptance criteria:

- Unix `0600` behavior remains intact.
- Non-Unix behavior is explicitly documented in code as relying on per-user temporary-directory ACLs, per-session token entropy, and RAII cleanup.
- Regression coverage prevents this from silently degrading into an undocumented no-op.

### PCH-004 — Runtime-control source-composition guardrails

The runtime-control shim is generated from Rust/Lua fragments. The build already fails closed on missing anchors, but the post-closeout review identified the composition path as brittle.

Acceptance criteria:

- Regression coverage verifies the build script still uses exact fail-closed anchors.
- Regression coverage verifies generated fragment names and Lua/Rust source fragments are still represented.
- The generated-source mechanism remains bounded and auditable until a larger refactor is justified.

### PCH-005 — Frontend `SessionSnapshot` type parity

The Rust backend returns a rich `SessionSnapshot`, but the TypeScript type previously declared only a narrow subset. That hid diagnostics already returned by the backend.

Acceptance criteria:

- TypeScript `SessionSnapshot` includes executable identity, effective argv/config, lifecycle timestamps, exit/termination fields, stdout/stderr diagnostic tails, truncation flags, and diagnostic error state.
- Existing save-state compatibility logic can consume the typed executable identity without local ad-hoc widening.
- Tests that construct snapshots use realistic full fixtures.

### PCH-006 — CI regression coverage

Acceptance criteria:

- Add `scripts/tauri/test-post-closeout-hardening.py`.
- Wire the regression into `.github/workflows/tauri-project.yml`.
- The regression verifies the user-visible controls, lifecycle listeners, non-Unix bootstrap documentation, generated-source guardrails, type parity, and TODO/spec linkage.

### PCH-007 — Qualification protocol

Acceptance criteria:

- Local static validation must run where the archive environment permits.
- Exact-head PR CI must qualify the branch before merge.
- Post-merge `master` CI must be inspected before claiming closure.

## 4. Files expected to change

Primary implementation files:

- `tauri/src/session/SessionControlPanel.tsx`
- `tauri/src/library/LibraryBrowser.tsx`
- `tauri/src/backend/types.ts`
- `tauri/src/session/saveStateCompatibility.test.ts`
- `tauri/src-tauri/src/sessions/control.rs`

Regression and CI files:

- `scripts/tauri/test-post-closeout-hardening.py`
- `scripts/tauri/test-security-policy.py`
- `.github/workflows/tauri-project.yml`

Planning files:

- `docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_SPEC_2026-09-13.md`
- `docs/MAME_TAURI_POST_CLOSEOUT_HARDENING_TODO_2026-09-13.md`

## 5. Validation protocol

A branch implementing this specification must pass, at minimum:

```text
python3 scripts/tauri/test-post-closeout-hardening.py
python3 scripts/tauri/test-security-policy.py
python3 scripts/tauri/test-mt2200-todo-reconciliation.py
```

For merge qualification, use exact-head GitHub Actions evidence for the same applicable workflows used at MT-2200 closeout:

- Tauri project;
- Build documentation;
- Tauri security;
- Tauri Windows packaging;
- Tauri macOS packaging;
- Tauri Linux packaging.

The hardening branch must be merged only after the exact PR head is green. After merge, the promoted `master` SHA must also be verified through the same post-merge matrix.
