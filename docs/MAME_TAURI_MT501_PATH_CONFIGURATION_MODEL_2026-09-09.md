# MT-501 — Path configuration model

Date: 2026-09-09

## Scope

MT-501 defines the backend configuration contract for content search paths. It does not add the settings UI, write settings to disk, or start audits; those are owned by MT-502 and later tasks.

## Settings schema

The settings schema advances from version 1 to version 2. Existing version-1 settings are migrated in memory by preserving `mameExecutable` and initializing all content path lists to empty. Unknown future schemas continue to fail explicitly.

Version 2 adds ordered content-path lists:

- `contentPaths.romPaths`
- `contentPaths.softwarePaths`
- `contentPaths.chdPaths`

Order is semantically significant and is preserved exactly. The model does not canonicalize, sort, deduplicate, or require a path to exist at parse time. This allows removable media, temporarily unavailable network locations, and user-controlled precedence to remain representable.

ROM, software, and CHD paths remain distinct application concepts even where a later MAME command may compose them into the same MAME search-path option. Keeping the intent separate prevents the configuration model from prematurely baking command-line policy into persistent data.

## Platform-safe path representation

Filesystem paths are represented internally as `PathBuf` through the `PlatformPath` wrapper; application code does not split paths on `/` or `\\` or otherwise assume a platform separator.

For ordinary Unicode paths, JSON remains human-readable and stores the path as a string. A native path that cannot be represented as Unicode is not silently replaced with a lossy string:

- Unix paths use a lossless `unixBytesHex` representation of the native path bytes.
- Windows paths use a lossless `windowsWideHex` representation of the native UTF-16 code units encoded as little-endian bytes.

An encoded path from a different platform is rejected explicitly rather than guessed. This preserves native path identity and prevents corrupted paths from quietly entering launch or audit commands.

## Validation contract

`validate_content_path` reports one of five stable states:

- `accessible` — the path exists, is a directory, and can be opened for enumeration.
- `missing` — the path does not exist.
- `notDirectory` — the path exists but is not a directory.
- `permissionDenied` — the operating system denied access.
- `unreadable` — another filesystem error prevented inspection.

Validation is observational. It does not mutate the path, create directories, or remove invalid entries. MT-502 can therefore show actionable validation feedback while still allowing the user to edit configuration safely.

## Regression coverage

The MT-501 unit tests cover:

- safe defaults for a missing settings file;
- parsing version-2 ROM/software/CHD paths while preserving order;
- migration from version 1 without inventing content paths;
- platform path JSON round trips;
- lossless non-UTF-8 Unix path round trips on Unix CI;
- accessible, missing, and non-directory validation;
- stable permission-denied classification;
- corrupt JSON and unsupported future-schema failures.

## Deferred work

MT-502 owns add/remove/reorder UI, native directory selection, live accessibility feedback, and durable writing of the updated settings. MT-503 and later tasks own supervised verification commands and audit persistence. No MAME process behavior changes in MT-501.

## Qualification evidence

The finalized implementation source tree is `07e5ec9f26bc16eaa49bf813c06365186ce3f3ea`. GitHub Actions Tauri run `34418697878` completed successfully against that exact tree at pre-squash commit `aa066ef753bed2a61a437cd44a09c31d29c571e9`. The clean implementation commit `54582a87053d4a0f9aa631847ae211002bfcc5bb` points to the same tree, with MT-410 closure `611717bc9635202bbb4a073945fbebc4ae582c85` as its single parent.

The successful qualification covered frontend formatting, linting, typechecking, 27 frontend tests, production build, Rust formatting, the full Rust test suite, the inherited 100,000-row MT-410 performance guard, Clippy with warnings denied, and lockfile integrity.

The Linux CI exercises UTF-8 and lossless non-UTF-8 Unix path serialization. The Windows `windowsWideHex` representation is platform-gated source in this task; Windows-target execution should be added when cross-platform CI is introduced.
