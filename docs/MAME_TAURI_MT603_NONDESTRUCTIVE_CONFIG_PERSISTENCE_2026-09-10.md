# MT-603 — Non-Destructive Configuration Persistence

**Date:** 2026-09-10  
**Repository:** `ekkus93/mame`  
**Task:** MT-603 — Implement non-destructive config persistence  
**Parent:** MT-602 closure `f352cf78acffcc7dd9ae4c3cf76566b467a32d8a`  
**Qualified implementation:** `4f791ce388f3abc4d449967ea14d304392e3e4f7`  
**Qualified tree:** `8181ef56a040bbc4e71f0ac890878ee7b329da31`  
**Exact-head CI:** GitHub Actions run `34455467880`

## Scope

MT-603 hardens persistence for the application-owned `settings.json` document. It does **not** begin editing MAME-owned `mame.ini`, machine INI files, controller CFG files, or other upstream/user-managed MAME configuration.

That boundary follows the architecture requirement to keep project-owned configuration separate where practical and to avoid destructive rewrites of unrelated user MAME configuration.

## Implementation

The shared persistence implementation lives in:

- `tauri/src-tauri/src/config_persistence.rs`

The existing content-path configuration flow now calls the shared persistence layer instead of directly truncating and rewriting `settings.json`.

### Unknown/unrelated field preservation

Persistence reads the existing JSON document before writing and validates it with the normal settings parser. The known application fields are merged into the existing JSON object rather than serializing a fresh `SettingsV2` document over the file.

The merge updates only the currently owned fields:

- `schemaVersion`;
- `mameExecutable`;
- known `contentPaths` members (`romPaths`, `softwarePaths`, and `chdPaths`).

Unknown top-level fields and unknown members nested under `contentPaths` are retained. This allows later application versions or adjacent settings owners to coexist without being erased by an older settings mutation.

Existing malformed JSON, malformed known fields, and unsupported future schemas are rejected before any replacement takes place. Persistence never silently rewrites such a document into current defaults.

A schema-v1 document is migrated through the existing migration path and may then be persisted as schema v2 while retaining unrelated JSON members from the original document.

## Backup and recovery policy

When a valid primary settings file already exists, its **exact previous bytes** are written to a sibling recovery file:

```text
settings.json.bak
```

The backup is committed before the new primary file. If the backup cannot be written, the primary file is left untouched and the write fails explicitly.

Recovery is exposed through `recover_settings_from_backup`. Recovery is deliberately explicit rather than an automatic fallback from `load_settings`:

1. a corrupt/unreadable primary remains a visible error;
2. the caller deliberately chooses recovery;
3. the backup is validated as a supported settings document;
4. the exact backup bytes are atomically restored to the primary path;
5. the backup itself is not rotated during recovery, so the recovery source is retained if restoration fails.

Stable recovery-related failures distinguish backup read/validation errors from replacement errors rather than masking the original problem.

## Atomic write behavior

Writes no longer open the final file with `truncate(true)`.

The persistence layer instead:

1. creates the destination directory if necessary;
2. creates a temporary file in the **same directory** as the destination;
3. writes the complete encoded document;
4. calls `sync_all` on the temporary file;
5. atomically persists/replaces the destination using `tempfile::NamedTempFile` persistence semantics;
6. calls `sync_all` on the persisted file;
7. on Unix, also synchronizes the parent directory metadata.

Using a sibling temporary file avoids cross-filesystem rename behavior and removes the previous truncate-before-complete-write failure window.

A first write creates only the primary settings file and does not invent a meaningless backup.

## Tests

MT-603 adds tests for the persistence contract, including:

- preserving unrelated top-level settings across a write;
- preserving unknown nested `contentPaths` members while updating known paths;
- schema-v1-to-v2 persistence without dropping unrelated fields;
- exact previous-primary byte preservation in the `.bak` file;
- explicit restoration of the last-known-good backup;
- preservation of the backup itself during recovery;
- refusing to overwrite malformed existing settings;
- complete first-write behavior with no unnecessary backup;
- no leaked same-directory temporary file after a successful atomic write.

The pre-existing content-path test continues to verify ordered path persistence and preservation of the configured MAME executable.

## CI qualification

Exact implementation SHA:

```text
4f791ce388f3abc4d449967ea14d304392e3e4f7
```

GitHub Actions run:

```text
34455467880
```

The exact-head `Tauri project` `linux-quality` job passed:

- frontend dependency installation;
- frontend format check;
- frontend lint;
- frontend TypeScript typecheck;
- frontend tests;
- frontend production build;
- Rust format;
- Rust tests, including the MT-603 persistence tests;
- inherited 100k-row library UX performance qualification;
- Clippy with warnings denied;
- lockfile integrity.

The branch was normalized before this qualification so the implementation is one product commit directly on the MT-602 closure head, with no temporary helper workflow in its tree.

## MT-603 acceptance

The implementation satisfies all four MT-603 acceptance requirements:

- unrelated user/application settings are preserved instead of overwritten;
- edited files have a defined last-known-good backup and explicit recovery policy;
- writes use same-directory atomic replacement rather than truncating the live file;
- parse/write round-trip and recovery behavior are covered by tests.
