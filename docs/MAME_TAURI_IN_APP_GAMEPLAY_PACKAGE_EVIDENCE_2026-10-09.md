# In-app gameplay — successful full-runtime package qualification (2026-10-09)

## Current qualified full runtime — frame-seam syntax and report fix

**Binary/source commit:** `7d1ed8426e5b3c8f436002edfd79fdf5cf74f3c1` (the subsequent `89fdde8a` commit is documentation/deferral only and does not modify the runtime). [Exact-source full-runtime CI #37983935896](https://github.com/ekkus93/mame/actions/runs/37983935896) passed both jobs, including the real-MAME executable checks, headless `___empty` frame-seam smoke with the corrected shell syntax, full Debian/AppImage packaging, installed-package verification, dependency augmentation, provenance manifest, and mandatory frame-seam enforcement. Tauri project [37983935928](https://github.com/ekkus93/mame/actions/runs/37983935928), Linux [37983935818](https://github.com/ekkus93/mame/actions/runs/37983935818), Windows [37983935859](https://github.com/ekkus93/mame/actions/runs/37983935859), and macOS [37983935806](https://github.com/ekkus93/mame/actions/runs/37983935806) also passed at the same source.

**[Full Debian/AppImage/provenance artifact 11643041469](https://github.com/ekkus93/mame/actions/runs/37983935896/artifacts/11643041469)** expires **2026-10-12 19:59:23 UTC**; preserve its bytes independently if needed after this GitHub Actions retention deadline. Bundled MAME version: `0.289 (unknown)`.

| Packaged file | SHA-256 | Size (bytes) |
| --- | --- | ---: |
| `MAME Tauri Frontend_0.1.1_amd64.AppImage` | `2067f00a13a328ce45468fce5621766fdc2dbdf643bd3563078551d983bee5f5` | 230410744 |
| `MAME Tauri Frontend_0.1.1_amd64.deb` | `3c3df57b21dbba41f2b1a306627559a42b0cef1468f5cbc5bd9e3c52be4cfb0b` | 125453488 |

AppImage build path: `tauri/src-tauri/target/release/bundle/appimage/MAME Tauri Frontend_0.1.1_amd64.AppImage`. Bundle command: `npm run tauri -- build --config src-tauri/tauri.linux-bundle.conf.json --bundles deb,appimage`.

**Scope:** This is *automated package and ROM-less headless frame qualification*, not proof that a real game runs on the user's graphical desktop. The corrected frame report differentiates Lua `screen:pixels()` results that are `not_measured` from real samples. The user has explicitly deferred all remaining graphical-desktop, private-ROM and installed-MAME-configuration tests; the 14 associated TODO items stay unchecked. No tagged release was published by these checks: the `v0.1.1` tag has a different historical source commit.

## Previous qualified full runtime — canvas optimization included

Binary source: `b7c12a82e4d412305dc323168b221970dc99acaa`. The [full-runtime workflow 37969476682](https://github.com/ekkus93/mame/actions/runs/37969476682) passed both jobs: MAME executable/provenance validation, `___empty` headless frame and repeated snapshot checks, full Debian/AppImage packaging, Debian dependency augmentation, installed-package/metadata smoke, manifest recording, and artifact upload. [Tauri project 37969476609](https://github.com/ekkus93/mame/actions/runs/37969476609), [standard Linux 37969476754](https://github.com/ekkus93/mame/actions/runs/37969476754), [Windows 37969476591](https://github.com/ekkus93/mame/actions/runs/37969476591), [macOS 37969476753](https://github.com/ekkus93/mame/actions/runs/37969476753), [security 37969476741](https://github.com/ekkus93/mame/actions/runs/37969476741), and [documentation 37969476504](https://github.com/ekkus93/mame/actions/runs/37969476504) passed at the same source.

The [SHA-pinned full package artifact (ID 11636880295)](https://github.com/ekkus93/mame/actions/runs/37969476682/artifacts/11636880295) is uploaded and currently unexpired. Its GitHub artifact retention deadline is **2026-10-12 17:53:44 UTC**. Download and retain this exact artifact for desktop qualification.

| Packaged item | SHA-256 | Bytes |
| --- | --- | ---: |
| `MAME Tauri Frontend_0.1.1_amd64.AppImage` | `4cba0ff2107b50a12dee2a30021b8d2b91e9370138adb47da891b888ea30a5a2` | 230410744 |
| `MAME Tauri Frontend_0.1.1_amd64.deb` | `57d02fdee6981ebc1d072ac4d927d9baf5c62a6675e925eb11f52bcf2a566d17` | 125453430 |

Bundled real MAME version: `0.289 (unknown)`. Build command: `npm run tauri -- build --config src-tauri/tauri.linux-bundle.conf.json --bundles deb,appimage`. The binaries were produced in the workflow paths `tauri/src-tauri/target/release/bundle/appimage/` and `tauri/src-tauri/target/release/bundle/deb/` and are contained in the artifact.

**Qualification scope limit:** No test has yet established a playable ROM from the user's `/home/phil/mame/roms` directory rendering inside the actual Tauri canvas with functioning audio/gamepad or target presentation performance. Headless `___empty` and installed-package window smoke do **not** count as those observations. Original installed-MAME ROM path, high-resolution/raster/vector capture seam overhead, full desktop interaction, and first-run qualification remain open. The filename `0.1.1` is not proof of release publication: the existing historical `v0.1.1` tag points elsewhere, and release-start run [37970387562](https://github.com/ekkus93/mame/actions/runs/37970387562) refused to move it.

## Previous passing full-runtime build — historical binary

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
