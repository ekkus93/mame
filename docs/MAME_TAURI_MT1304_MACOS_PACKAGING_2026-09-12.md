# MAME Tauri MT-1304 — macOS packaging

**Date:** 2026-09-12  
**Task:** MT-1304 — macOS packaging  
**Status:** Implementation complete; CI qualification pending

## Package model

MT-1304 uses the native Tauri macOS application bundle plus DMG direct-download package:

- `.app` is the executable macOS application bundle;
- `.dmg` is the initial direct-download installer container;
- `tauri/src-tauri/tauri.macos.conf.json` enables only the macOS `app` and `dmg` targets and maps the MT-1302 `mame-runtime` resource tree into the application resource directory.

The package preserves the same package-owned MAME topology used by the Windows package:

```text
MAME Tauri Frontend.app/
  Contents/
    MacOS/<frontend executable>
    Resources/
      mame-runtime/
        bin/mame
        hash/
        bgfx/
        licenses/COPYING
        licenses/legal/
```

The Rust `BundledRuntimeLayout` validation is run on the macOS runner before packaging, including the Unix executable-bit contract for `mame-runtime/bin/mame`.

## Clean-runner bundle and DMG smoke

`.github/workflows/tauri-macos-packaging.yml` runs on a fresh GitHub-hosted macOS runner and:

1. creates a bounded synthetic MAME executable/resource fixture;
2. stages it through the shared MT-1302 `stage-mame-runtime.sh` contract;
3. runs the macOS `bundled_runtime` Rust tests;
4. proves the release script rejects absent credentials and rejects ad-hoc signing as a release identity;
5. builds an ad-hoc-signed `.app` and DMG with Tauri;
6. validates `Info.plist`, the application executable, bundled MAME executable, hash resources, BGFX resources, and license material inside the produced `.app`;
7. verifies the application code signature with `codesign --verify --deep --strict`;
8. mounts the actual DMG read-only and repeats the package/resource/signature checks on the app contained in the disk image;
9. copies the mounted app to a clean temporary `Applications` directory and validates it again, proving the package-owned resource layout survives relocation;
10. verifies package-lock and Cargo lock files remain unchanged.

The verifier is `scripts/tauri/test-macos-bundle.sh`.

The synthetic MAME payload is never a release artifact and the workflow does not publish the generated package.

## Signing boundary

The ordinary CI smoke uses the macOS ad-hoc signing identity `-`. This is sufficient to exercise Tauri's signing/bundle mechanics and to verify a structurally signed application on a clean runner, but it is explicitly **not** public-release signing.

`scripts/tauri/build-macos-release.sh` implements the release boundary and fails closed unless:

- `APPLE_SIGNING_IDENTITY` is set to a real Apple signing identity and is not `-`;
- a fully staged, executable MT-1302 MAME runtime is present;
- either App Store Connect API notarization credentials are complete (`APPLE_API_ISSUER`, `APPLE_API_KEY`, `APPLE_API_KEY_PATH`) or Apple-ID notarization credentials are complete (`APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`).

The release script then runs lockfile-based dependency installation, macOS bundled-runtime validation, and the Tauri `app,dmg` build. Tauri performs signing and notarization when the Apple certificate/identity and notarization credentials are available to the build environment.

For CI/CD release signing, the Apple certificate must also be imported into the runner keychain. The repository intentionally does not contain the certificate, certificate password, private notarization key, Apple password, or other Apple credentials.

## Notarization and publication rule

A public direct-download macOS package must not be described as release-qualified merely because the ad-hoc CI smoke passes. Publication requires a real Developer ID signing identity and successful Apple notarization/stapling using private credentials supplied by the release environment.

MT-1304 therefore qualifies the deterministic packaging mechanics, app/DMG layout, bundled runtime placement, ad-hoc signing mechanics, and fail-closed notarization process without fabricating evidence that private Apple release credentials were exercised.
