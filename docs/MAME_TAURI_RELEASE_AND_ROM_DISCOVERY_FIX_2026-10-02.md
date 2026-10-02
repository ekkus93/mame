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

## Publication and source-check evidence

The original v0.1.0 release assets were published successfully by run
`37071196810` at source `36dbaaad936682b1cc6f74b9fb142fc988ff1355`, including
both package formats and SHA256SUMS. They are release assets, independent of
Actions artifact retention. This preserves the original tiny-runtime build.

At source `121880f1becc35ca70678bdcd813b50afa84f92f`, run `37071111113` passed
frontend formatting, lint, typechecking, tests and production build; Rust
formatting, tests and clippy; and locked dependency verification. Its post-job
cache upload was still in progress when these results were observed. The
preexisting sparse-checkout Cargo failure was resolved by retaining the root
ignore and attributes files in the checkout.

Version 0.1.1 updates the application manifests and root lockfile package
versions without changing dependencies. A one-time release workflow waits for
successful Tauri project checks on the current master commit, refuses to move
an existing tag, and explicitly dispatches full-runtime package qualification
on v0.1.1. The existing publisher attaches binaries only after qualification
passes. Full MAME compilation and installed-package checks remain pending;
source-test success alone is not a completed executable release.
