# In-app gameplay CI triage — 2026-10-08

At `bea14e376d9491cfb7a38b51d11371bb57a431fe`, `Tauri project` run 37824508988 failed frontend Prettier checking of `src/backend/commands.ts`, `src/backend/types.ts`, and `src/gameplay/GameSurface.tsx`.

Linux, macOS and Windows packaging runs 37824509053, 37824508935 and 37824509024 failed Rust compilation: `src/sessions/supervisor.rs:523` calls undefined `frame_error_with_metrics`. Preserve `AppError` code/message/retryable and merge `sessionId` and `metrics` into its details before returning it.

Do not infer full AppImage gameplay qualification from green documentation or security workflows. Recheck exact-head CI after fixes and retain full AppImage desktop proof.
