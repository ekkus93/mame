# Tagged release assets and local ROM discovery correction

The 2026-10-02 installed-app report exposed gaps in the historical reset acceptance:

- `v0.1.0` package run `37046484748` succeeded but uploaded only an Actions artifact,
  scheduled to expire on 2026-10-09. No GitHub Release was created.
- The package workflow built `SUBTARGET=tiny`, a genuine MAME executable with an
  example driver subset. The user's Pac-Man, Galaga, Street Fighter II, Contra,
  Asteroids, Gauntlet, Robotron, Tapper and R-Type ROM collections are outside it.
- Initial content settings were empty; automatic home-directory discovery was
  explicitly deferred. Available Locally also requires a current successful audit.

## Corrections

`Publish Tauri Linux release` promotes the exact qualified tagged package artifact
to persistent `.deb`, `.AppImage`, and `SHA256SUMS` release assets. It validates the
source repository, workflow, conclusion, tag commit, artifact identity, expiration,
and package completeness. Existing assets are never silently overwritten. The
first installation of this workflow backfills original `v0.1.0` build run
`37046484748`; its release notes explicitly disclose its tiny-runtime limitation.
Later successful tagged package builds are published automatically.

The package workflow now builds full `SUBTARGET=mame`. A release coverage check
requires the arcade drivers named above before packaging and checks the extracted
package runtime again. Structural fixtures remain separate from release builds.

On first run, settings discover accessible `~/mame/roms` and `~/.mame/roms` folders
in that order. Discovery goes through the settings loader, so configuration,
audits, diagnostics and launch see the same paths. Existing settings, including
explicitly empty lists and user ordering, take precedence. The empty Available
Locally view offers Configure ROM folders and Audit ROMs actions. Archive presence
alone is not treated as proof of complete or version-compatible ROM contents.

## Existing installations

In More > Configure Options, add `/home/phil/mame/roms` under ROM paths and save.
Then use More > Audit. A compatible full-runtime package and refreshed metadata
are required for machines absent from the original tiny runtime. Updating source
does not change an already installed executable or the existing `v0.1.0` tag.

## Qualification

- Local publisher tests exercise valid lightweight/annotated tags and reject
  mismatched commits, failed runs, foreign repositories, branch builds, expired
  or duplicate artifacts, and missing package formats before release writes.
- Local driver-check probes accept exact machine names and reject a tiny-style
  failure or a prefix-only match. Shell syntax and Python compilation are checked.
- Backend regression covers first-run discovery, invalid candidates, and saved
  explicitly empty paths. Exact-head frontend/backend and package CI are required
  before declaring the installed-app correction qualified.
