# MT-506 — Per-machine audit action

Date: 2026-09-09

## Status

**Qualified for closure.**

Implementation commit:

`0a94f425749ca49b970fbf21ef5a4abe5d97e477`

Implementation tree:

`a44c54bb9e7e2e62ff181e19a8a59c00631a5dd8`

Authoritative Tauri qualification:

- workflow run: `34439484296`
- result: **PASS**
- exact tested head: `0a94f425749ca49b970fbf21ef5a4abe5d97e477`

The exact normalized implementation passed frontend formatting, ESLint, strict TypeScript, frontend tests, production build, Rust formatting, Rust tests, the inherited 100,000-row MT-410 performance qualification, Clippy with warnings denied, and lockfile-integrity verification.

## Scope

MT-506 adds an explicit per-machine media audit operation to the library detail view. It composes the MT-503 command semantics, the MT-504 structured parser, and the MT-505 provenance-aware persistence layer without introducing a second source of audit truth.

## Backend execution contract

The Rust runner invokes MAME directly through `std::process::Command`; no shell command string is constructed.

For a machine audit it constructs the effective argument sequence as:

```text
-noreadconfig [-rompath <configured-multipath>] -verifyroms <machine-short-name>
```

The configured MAME media search path is built deterministically in this order:

1. ROM paths, preserving user order;
2. software paths, preserving user order;
3. CHD paths, preserving user order.

Paths remain OS-native `OsString` values through composition. Non-UTF-8 paths therefore do not require lossy display conversion. A configured path that cannot safely participate in MAME's semicolon-delimited multipath is rejected with a structured error rather than silently altered.

The audit process has bounded stdout/stderr retention and a bounded execution timeout. stdout and stderr are drained concurrently to avoid pipe deadlock. A normal MAME audit failure exit such as exit code `2` is passed to the MT-504 parser as audit evidence; it is not misreported as a process-transport failure.

## Authority and provenance

Before an audit runs or a persisted result is loaded, the backend:

- validates the requested machine identifier;
- confirms the machine belongs to the active catalog generation;
- resolves the MAME executable source from that generation;
- re-inspects the executable identity and rejects stale metadata identity;
- loads the current ordered content-path configuration.

A completed audit is then persisted through MT-505 with the exact MAME identity and content-path provenance. Loading the current result uses MT-505's stale-result invalidation rules.

## Frontend behavior

The machine detail view includes a dedicated `MachineAuditPanel` with:

- an `Audit media` / `Audit again` action;
- an explicit `Auditing…` progress state while the backend command is active;
- the MT-504 structured aggregate classification;
- fine-grained facts such as optional missing, required missing, incorrect checksum/length, known-undumped, or redump-needed content;
- audit timestamp and MAME process exit code;
- a bounded raw MAME diagnostic excerpt for non-complete results when diagnostic output is available;
- explicit error state for command/transport/configuration failures.

The panel loads only a result that is current for the active MAME identity and path configuration. It does not infer availability from filesystem presence.

## Qualification coverage

Automated coverage includes:

- typed frontend command envelopes for loading and running per-machine audits;
- exact argument construction for no configured paths;
- deterministic ROM → software → CHD path ordering;
- rejection of ambiguous multipath separators;
- lossless non-UTF-8 path composition on Unix;
- execution of a controlled fake MAME process returning exit `2`, proving the result is parsed as missing required media rather than treated as a transport error;
- full existing Rust and frontend regression suites;
- inherited MT-410 performance qualification.

## Failure found during qualification

An intermediate candidate registered child-module Tauri commands through a parent-module Rust re-export. Tauri's `#[command]` macro-generated registration symbols remain at the defining module path, so that candidate failed Rust compilation in workflow run `34438888471`.

The implementation was corrected to register the commands at `library::audit::get_library_machine_audit` and `library::audit::run_library_machine_audit`. The normalized implementation above contains that fix and passed exact-head qualification in run `34439484296`.

## MT-506 acceptance

- Run audit: **satisfied**.
- Show progress: **satisfied**.
- Show structured result: **satisfied**.
- Show raw diagnostic excerpt when useful: **satisfied**.

## Boundary to MT-507

MT-506 deliberately performs one machine audit per user action. It does not add bulk scheduling, cancellation, parallel execution, or restart/resume semantics. Those belong to MT-507 so concurrency policy remains explicit and separately qualified.
