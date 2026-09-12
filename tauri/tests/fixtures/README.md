# MT-300 / MT-301 listxml fixtures

`listxml-representative.xml` is the original reduced bootstrap fixture used to exercise the MT-300 parser and database importer. Its element and attribute shapes were derived directly from `src/frontend/mame/infoxml.cpp` at the MT-300 base SHA. It is retained as a compact schema-focused regression fixture and is not represented as captured catalog output.

MT-301 fixture provenance is complete through a separate captured fixture:

- captured fixture: `listxml-representative-captured.xml`;
- provenance manifest: `listxml-representative.provenance.json`;
- capture workflow: `.github/workflows/mt301-fixture-capture.yml`;
- successful capture run: GitHub Actions `34688122578`;
- pinned package: Ubuntu `mame=0.264+dfsg.1-1`;
- executable identity reported by the capture: `0.264 (unknown)`;
- captured fixture SHA-256: `85a4bfa0f881389afcf897f1c5a2c7c90ca4bec6f58af3b8441e6e7ea9cf336a`.

The captured file is a bounded projection of actual `-listxml` output rather than hand-authored XML. It retains the representative machine identities and parser fields needed by the regression suite while excluding catalog noise that is irrelevant to the exercised schema. `tauri/src-tauri/src/metadata/provenance_fixture.rs` locks the recorded MAME/package identity and representative captured shape into the Rust test suite.

The generator integration test verifies imported metadata through the canonical MAME short name `apple2e`; it intentionally does not invent a display-name alias for the authoritative `Apple //e` description.
