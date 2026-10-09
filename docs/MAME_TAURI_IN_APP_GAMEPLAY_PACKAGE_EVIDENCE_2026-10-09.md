# In-app gameplay — successful full-runtime package qualification (2026-10-09)

The exact-source Linux full-runtime workflow [37923443872](https://github.com/ekkus93/mame/actions/runs/37923443872) **passed** at commit `3da835f8cb7d15910f363448d4d0bc95d1456834`. Both jobs passed: real MAME executable validation and full Debian/AppImage packaging, including headless snapshot capture, repeated frame capture, installed Debian smoke, and metadata bootstrap. MAME version: `0.289 (unknown)`.

The uploaded [full package artifact (ID 11614039521)](https://github.com/ekkus93/mame/actions/runs/37923443872/artifacts/11614039521) includes the AppImage, Debian package, provenance manifest, and frame-seam report. It **expires 2026-10-12 11:23:41 UTC** because repository retention is capped at three days. Preserve a copy before expiration.

| Item | SHA-256 | Size (bytes) |
| --- | --- | ---: |
| `MAME Tauri Frontend_0.1.1_amd64.AppImage` | `0dee88f12cd09c8468ebb70f0c1bcd533a24fb1cf186b243a5fc47f19fcfa835` | 230414840 |
| `MAME Tauri Frontend_0.1.1_amd64.deb` | `edb76812839fd0f9819e3771a5c5321abb8be8535c12b0bbcc175e01d0b60adc` | 125453506 |

Build command: `npm run tauri -- build --config src-tauri/tauri.linux-bundle.conf.json --bundles deb,appimage`.

At the same SHA, Tauri project [37923443908](https://github.com/ekkus93/mame/actions/runs/37923443908), standard Linux packaging [37923443822](https://github.com/ekkus93/mame/actions/runs/37923443822), Windows [37923443874](https://github.com/ekkus93/mame/actions/runs/37923443874), macOS [37923443843](https://github.com/ekkus93/mame/actions/runs/37923443843), and security [37923443888](https://github.com/ekkus93/mame/actions/runs/37923443888) passed.

The existing `v0.1.1` tag refers to older source, not this package; release publishing [37926758341](https://github.com/ekkus93/mame/actions/runs/37926758341) was skipped. The artifact filename version does not establish release publication.

**Desktop qualification remains outstanding.** On the user's real Linux desktop, download the exact artifact, verify its AppImage SHA-256, and run the existing `scripts/tauri/qualify-in-app-gameplay.sh`, `scripts/tauri/qualify-in-app-gameplay-fresh-profile.sh`, `scripts/tauri/capture-original-mame-baseline.sh`, and `scripts/tauri/qualify-mame-frame-seams.sh` with an audited machine from `/home/phil/mame/roms`. See [local qualification instructions](MAME_TAURI_IN_APP_GAMEPLAY_LOCAL_QUALIFICATION_2026-10-08.md). Save diagnostics, raster/vector/rotation performance, keyboard/gamepad, native audio, fullscreen, stop/relaunch and first-run evidence privately. CI's headless `___empty` frames and Xvfb window smoke do not prove in-app playable video.
