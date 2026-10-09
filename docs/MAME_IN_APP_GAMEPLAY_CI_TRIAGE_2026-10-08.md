# In-app gameplay CI triage — 2026-10-08

At `bea14e376d9491cfb7a38b51d11371bb57a431fe`, `Tauri project` run 37824508988 failed frontend Prettier checking of `src/backend/commands.ts`, `src/backend/types.ts`, and `src/gameplay/GameSurface.tsx`.

Linux, macOS and Windows packaging runs 37824509053, 37824508935 and 37824509024 failed Rust compilation: `src/sessions/supervisor.rs:523` calls undefined `frame_error_with_metrics`. Preserve `AppError` code/message/retryable and merge `sessionId` and `metrics` into its details before returning it.

Do not infer full AppImage gameplay qualification from green documentation or security workflows. Recheck exact-head CI after fixes and retain full AppImage desktop proof.

## 2026-10-09 — Preserve active real-MAME compilation and repair cache key after completion

At source head `d82dc7f28c79f458a39742a08bfbdd1538f50261`, real-runtime workflow [37906371265](https://github.com/ekkus93/mame/actions/runs/37906371265) is still compiling the actual native MAME executable. The workflow defines `cancel-in-progress: true` for the same branch ref. **Do not make a source, script, or workflow commit merely to improve CI while this long-running build remains active:** such a push would start another qualifying run and cancel the current compilation. A docs-only commit is outside this workflow's `on.push.paths` filter and is not a substitute for binary testing.

Static review found a compiler-cache restore-prefix mismatch in `.github/workflows/tauri-linux-real-runtime-package.yml`:

- Exact cache key: `${{ runner.os }}-real-mame-${{ github.sha }}-${{ env.MAME_TAURI_MAME_SUBTARGET }}`
- Specific restore prefix: `${{ runner.os }}-real-mame-${{ env.MAME_TAURI_MAME_SUBTARGET }}-`

Because the exact key puts the commit SHA before the subtarget but the specific restore prefix expects the subtarget immediately after `real-mame-`, cross-commit reuse **cannot match this specific prefix**. The generic `${{ runner.os }}-real-mame-` fallback can still restore cache entries, so do not claim that all caching is broken. After the active long-running run finishes, change the cache key layout to put the subtarget before the SHA, preserving a specific restore prefix that really matches and the generic fallback where appropriate. Requalify any workflow change at its new exact source SHA. Avoid unnecessary repeated six-hour builds.

Full-runtime packaging and real-ROM desktop qualification are still pending; a compiling runtime is neither a passing artifact nor a user-playable AppImage.
