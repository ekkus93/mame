# MT-1306 — Version reporting qualification

**Qualified date:** 2026-09-13  
**Implementation head:** `e79b13e6df5877effc190a180b34dd1cae6c779a`

## Scope

MT-1306 requires UI/diagnostics reporting for:

- application version;
- Git/build identity where appropriate;
- MAME version/build identity;
- database schema version; and
- runtime protocol version.

## Implementation

The typed `get_app_info` backend contract now reports:

- the application package version from `CARGO_PKG_VERSION`;
- build profile, target architecture/OS, and `GITHUB_SHA` when supplied by CI/build infrastructure;
- catalog database schema version;
- settings schema version;
- runtime-control protocol version; and
- configured MAME executable identity when available.

MAME identity lookup is deliberately diagnostic rather than fatal: an unconfigured or currently unavailable executable is represented explicitly without preventing the rest of the application/build identity from being reported.

The frontend diagnostics surface consumes the typed response and presents the version/build/schema/protocol fields. Tests cover the version-reporting contract and the frontend state shape.

## Exact-head qualification

All executable/package workflows for `e79b13e6df5877effc190a180b34dd1cae6c779a` completed successfully:

- Tauri project: `34740868964` — **success**
- Tauri macOS packaging: `34740868972` — **success**
- Tauri Linux packaging: `34740868988` — **success**
- Tauri Windows packaging: `34740869003` — **success**

## Result

MT-1306 is complete. All five required version-reporting dimensions are implemented and exact-head qualified.