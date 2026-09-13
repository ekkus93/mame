# MT-1900 Upstream Sustainability

**Date:** 2026-09-13  
**Repository:** `ekkus93/mame`  
**Project head used for rehearsal:** `6abd8f57cff3905a574dee3c2b0635646eea2fff`  
**Upstream repository:** `mamedev/mame`  
**Upstream head observed:** `5346408d5efb5054b84f0015ed18c4e59db99a1c`  
**Merge base:** `7cc3033a50b00801240f020b7098e22ffffdcd44`  
**MT-1900 tooling branch:** `ralph/mt-1900-upstream-sustainability`

## Scope

MT-1900 makes upstream synchronization auditable and repeatable.  The Tauri
frontend must remain isolated from upstream-maintained MAME source unless a task
explicitly requires a MAME patch and records its maintenance cost.

## MT-1901 — Upstream remote and sync procedure

Use these canonical remotes:

```bash
git remote add upstream https://github.com/mamedev/mame.git
git fetch origin master
git fetch upstream master
```

Before syncing, record the divergence:

```bash
git merge-base origin/master upstream/master
git rev-list --left-right --count upstream/master...origin/master
git diff --name-only "$(git merge-base origin/master upstream/master)"...origin/master \
  > artifacts/upstream/project-changed-paths.txt
```

Create an isolated rehearsal branch first:

```bash
git switch -c ralph/mt-1900-upstream-sync-rehearsal origin/master
git merge --no-ff upstream/master
```

If conflicts appear, do not resolve them directly on `master`.  Resolve them on
the rehearsal branch and record each conflict class in the MT-1900 note.  After a
clean rehearsal, either:

1. keep the rehearsal as evidence only; or
2. open a dedicated upstream-sync PR to promote the same merge to `master`.

Promotion to `master` requires exact-head CI evidence from both project-specific
Tauri workflows and any native MAME workflows triggered by upstream-tree changes.

## MT-1902 — Project-owned patch inventory

The helper `scripts/tauri/mt1900_upstream_inventory.py` classifies changed paths
as either project-owned or upstream-tree touches.

Project-owned paths are intentionally scoped to:

- `tauri/**`
- `scripts/tauri/**`
- `tests/tauri/**`
- `docs/MAME_TAURI*`
- `docs/legal/MAME_TAURI*`
- project-specific Tauri/docs/security packaging workflows under `.github/`

All other paths are treated as upstream-maintained MAME tree touches and require
an explanation, an owner, and native MAME validation.

Example usage:

```bash
python3 scripts/tauri/mt1900_upstream_inventory.py \
  --from-git "$(git merge-base upstream/master HEAD)" \
  --pretty \
  --write artifacts/upstream/mt1900-patch-inventory.json
```

For a project-only branch that must not touch upstream-maintained MAME files:

```bash
python3 scripts/tauri/mt1900_upstream_inventory.py \
  --from-git "$(git merge-base upstream/master HEAD)" \
  --expect-no-upstream-touches
```

## MT-1903 — First upstream-sync rehearsal

A real GitHub rehearsal PR was created from upstream into a throwaway project
branch:

- PR: `#8` — `MT-1900 upstream sync rehearsal`
- Base branch: `ralph/mt-1900-upstream-sync-rehearsal`
- Base SHA: `6abd8f57cff3905a574dee3c2b0635646eea2fff`
- Head repository: `mamedev/mame`
- Head SHA: `5346408d5efb5054b84f0015ed18c4e59db99a1c`
- Upstream commits integrated: `80`
- Changed files reported by GitHub: `318`
- Additions/deletions reported by GitHub: `28,937` / `8,613`
- Merge commit on rehearsal branch: `603770bbd12036fd2cc7d7c3bf40bf21775396bb`

Initial GitHub mergeability was temporarily unknown/false while GitHub computed
the test merge.  A subsequent metadata refresh reported the PR mergeable, and
the merge completed cleanly into the rehearsal branch.  No manual conflict
resolution was needed for this sampled upstream head.

The merge is intentionally **not** promoted to `master` by this document.  It is
a rehearsal artifact that proves the current fork can integrate the sampled
upstream head cleanly and identifies the CI surface triggered by such a sync.

Observed rehearsal workflow results on `603770bbd12036fd2cc7d7c3bf40bf21775396bb`:

- `34772403450` — Check `#include` guards — **success**
- `34772403447` — XML/JSON validation — **success**
- `34772403430` — Build documentation — **success**
- `34772403422` — CI (macOS) — **success**
- `34772403428` — CI (Windows) — **success**
- `34772403427` — CI (Linux) — **success**

The Linux rehearsal run completed successfully after building and validating both
`build-linux (gcc)` and `build-linux (clang)`.  The macOS and Windows rehearsal
runs completed their representative tiny-target build and validation jobs.

Conclusion: the fork can cleanly integrate the sampled upstream head into a
throwaway rehearsal branch, and the native MAME validation surface triggered by
that rehearsal passed.  This is sync-readiness evidence only; the upstream merge
commit remains on the rehearsal branch and is not promoted to `master` here.

## Production MT-1900 branch evidence

The production MT-1900 branch `ralph/mt-1900-upstream-sustainability` contains
only project-owned upstream-sustainability tooling, regression wiring, and this
documentation.  It does not promote the upstream rehearsal merge.

Observed production-branch gates before this evidence-closing documentation
update:

- `34772322124` — Tauri project — **success** at `337a56e53b4643e4cecd32d96a48b628cb19c0bf`
- `34772511111` — Build documentation — **success** at `337a56e53b4643e4cecd32d96a48b628cb19c0bf`

Because this commit is documentation-only, promotion requires the documentation
workflow for the evidence-closing head to pass before fast-forwarding `master`.

## MT-1904 — Recurring cadence

Recommended cadence:

- Run the non-mutating divergence check weekly while active Tauri development is
  ongoing.
- Run a full upstream-sync rehearsal at least every two weeks, or immediately
  before any release candidate.
- Do not allow drift to exceed one month without a documented reason.
- Promote an upstream sync only from an exact rehearsal SHA or an equivalent
  repeated merge that has passed the required project and native CI matrix.

## Acceptance status

- MT-1901: complete — upstream remote and sync procedure documented.
- MT-1902: complete — inventory helper added and wired into Tauri linux-quality
  CI.
- MT-1903: complete — real upstream rehearsal branch created, cleanly merged,
  and validated across include guards, XML/JSON, docs, Linux, macOS, and
  Windows workflows.
- MT-1904: complete — recurring cadence documented.

MT-1900 is complete after the evidence-closing documentation workflow passes on
this branch head and `master` is fast-forwarded to that exact production head.
