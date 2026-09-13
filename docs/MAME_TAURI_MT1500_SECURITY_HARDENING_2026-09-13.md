# MAME Tauri MT-1500 Security Hardening

**Date:** 2026-09-13  
**Branch:** `ralph/mt-1500-security-hardening`  
**Status:** implementation complete; exact-head qualification pending

## Scope

This record closes MT-1501 through MT-1506 by making the existing privileged boundaries explicit, adding adversarial process-input coverage, adding static policy regression, and establishing automated dependency advisory/update handling.

## MT-1501 — Tauri capabilities and permissions

The main WebView capability remains deliberately minimal:

- capability is scoped only to the `main` window;
- the only granted capability is `core:default`;
- the application does not install or grant the generic Tauri shell plugin;
- the application does not install or grant the generic Tauri filesystem plugin;
- the application does not install or grant generic HTTP/opener plugins;
- native MAME execution remains Rust-owned through bounded `std::process::Command` call sites and validated argv/path models rather than a frontend-accessible generic process API.

`scripts/tauri/test-security-policy.py` fails CI if these assumptions drift.

## MT-1502 — Adversarial filesystem behavior

The current filesystem boundary already fails closed:

- project-controlled MAME paths must be absolute and reject parent traversal;
- MAME path-list separator injection and environment-variable expansion are rejected;
- artwork asset IDs are path-blind identifiers rather than host paths supplied by the WebView;
- artwork roots and candidate files are canonicalized before authorization;
- canonical candidates outside the configured canonical root are rejected;
- Unix symlink-escape regression coverage proves an artwork symlink cannot escape its configured root;
- malformed asset IDs, unsupported extensions, and stale/unauthorized root indices are rejected without disclosing host paths;
- Unicode and spaces remain supported where they are safe data rather than identifier syntax.

The Rust test suite exercises these behaviors on every normal Tauri CI run.

## MT-1503 — Process argument injection

`tauri/src-tauri/tests/mt1500_security.rs` adds an explicit adversarial contract for machine/software inputs. It rejects:

- semicolons;
- `&&` and pipes;
- command substitution syntax;
- backticks;
- embedded single/double quotes;
- backslashes in identifiers;
- spaces where identifiers require bounded MAME syntax;
- option-looking values such as `-help`;
- parent traversal, path-list injection, and Unix environment expansion in project-controlled MAME paths.

Legitimate filesystem paths containing spaces, Unicode, and non-shell metacharacters remain discrete argv entries; no shell-string concatenation is introduced.

## MT-1504 — IPC/control-protocol audit

The runtime-control protocol remains local to the supervised child process and uses per-session authentication material. Existing MT-711 adversarial tests cover:

- authenticated malformed JSON;
- decoded and encoded oversized frames;
- unsupported protocol versions;
- stale-session IDs;
- unknown command completions;
- generated Lua-side rejection of unknown commands, versions, sessions, and oversized requests;
- channel drops during commands;
- bounded response deadlines.

The security model does not treat an arbitrary local process as an authorized peer merely because it can emit text: frames must satisfy the current session/version/token protocol contract.

## MT-1505 — Remote-content policy

The privileged Tauri WebView is local-content only in production:

- `frontendDist` is the bundled `../dist` output;
- the development URL is fixed to local Vite at `http://localhost:1420`;
- application windows do not specify arbitrary remote URLs;
- CSP is pinned by regression to same-origin application content plus the Tauri local IPC/dev transport allowances;
- generic HTTP/opener plugins are absent;
- local artwork is read by the Rust backend and returned as bounded data URLs, so artwork does not require privileged WebView navigation to remote origins.

Any future remote artwork/provider work must remain isolated from privileged page navigation and requires explicit security-policy review.

## MT-1506 — Dependency and supply-chain policy

The repository now enforces the following process:

- npm and Cargo lockfiles are committed and normal CI rejects lockfile drift;
- `.github/dependabot.yml` schedules weekly npm, Cargo, and GitHub Actions update PRs;
- `.github/workflows/tauri-security.yml` runs the static privileged-boundary regression;
- production npm dependencies are checked with `npm audit --omit=dev --audit-level=high`;
- Rust dependencies are checked against RustSec advisories with `rustsec/audit-check@v2.0.0`;
- the security workflow also runs weekly so newly published advisories are reported without waiting for a source change;
- MT-1307 generates the exact dependency/license inventory retained as a CI artifact for release review;
- release evidence must identify the exact source SHA and exact green security/package CI runs used for qualification.

A vulnerability may be temporarily accepted only by an explicit documented decision identifying the advisory, affected component/path, impact analysis, mitigation, owner, and removal/review condition. Silent advisory suppression is not an accepted policy.

## Qualification requirements

Before closure/promotion, the exact executable head must pass:

1. `Tauri project`, including the MT-1503 Rust integration tests;
2. `Tauri security`, including static policy, npm advisory audit, and RustSec audit;
3. affected packaging workflows required by the exact-head CI policy;
4. documentation build for the final evidence-only closure commit.

Exact run IDs and SHAs are appended when those gates complete.
