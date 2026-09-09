# MT-300 listxml fixtures

`listxml-representative.xml` is a reduced schema fixture used to exercise the MT-300 parser and database importer while the implementation is bootstrapped. Its element and attribute shapes are derived directly from `src/frontend/mame/infoxml.cpp` at the MT-300 base SHA.

Before MT-301 is closed, this bootstrap fixture must be replaced by (or accompanied by) a reduced capture generated from a pinned, known MAME executable. The capture command is provided by `scripts/tauri/capture-listxml-fixture.sh`, and the producing executable's `-version` identity must be recorded here with the final fixture.

## Current validation state

The MT-300 implementation branch is based on MAME/Tauri integration commit `52b145c77b5f5af72b85ac01cf31ca4fab6b23d8`. The reduced bootstrap fixture is intentionally not represented as an authoritative captured MAME catalog; its provenance remains explicit until a pinned executable capture is recorded. Parser/import regression tests consume this fixture, while `capture-listxml-fixture.sh` is the required path for producing the final MT-301 capture without hand-editing generated XML.
