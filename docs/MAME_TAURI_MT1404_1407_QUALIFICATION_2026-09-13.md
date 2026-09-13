# MAME Tauri MT-1404 through MT-1407 Qualification

**Qualified source SHA:** `a3c97334062a22207ba40386f5daf065176cb828`  
**Promoted to `master`:** 2026-09-13

All cumulative MT-1404 through MT-1407 changes were qualified on the same exact source SHA through PR #6.

| Workflow | Run | Result |
| --- | ---: | --- |
| CI (Windows) | `34744599205` | success |
| CI (macOS) | `34744599224` | success |
| Tauri project | `34744599202` | success |
| Tauri Linux packaging | `34744599203` | success |
| Tauri Windows packaging | `34744599229` | success |
| Tauri macOS packaging | `34744599234` | success |
| Build documentation | `34744599215` | success |

The qualified head was six commits ahead and zero commits behind the previous `master` closure SHA, so `master` was fast-forwarded directly to the exact qualified head rather than creating a different merge SHA.
