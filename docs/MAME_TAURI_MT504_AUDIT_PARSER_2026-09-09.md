# MT-504 — MAME audit parser

Date: 2026-09-09

## Scope

MT-504 implements the pure Rust parser that converts a completed MAME audit process result into conservative structured availability information. It consumes stdout, stderr, and the process exit code together. It does not launch MAME, persist audit results, or inspect the filesystem for media presence.

The parser implements the contract established in `docs/MAME_TAURI_MT503_AUTHORITATIVE_MAME_AUDIT_COMMANDS_2026-09-09.md`.

## Structured result

`MameAuditClassification` represents these aggregate states:

- `Complete` — MAME explicitly reports the set as good and the process exits successfully.
- `BestAvailable` — MAME reports an acceptable but degraded/best-available result, including optional missing media, known-undumped media, or media needing redump.
- `MissingRequired` — required media is explicitly absent.
- `Incorrect` — present media has an explicitly incorrect checksum or length.
- `MixedFailure` — the same audit reports both required-missing and incorrect media.
- `Unknown` — target errors, unsupported exit codes, incomplete output, contradictory evidence, or output that cannot be classified safely.

`MameAuditFacts` preserves useful detail independently of the aggregate classification:

- `optionalMissing`
- `noGoodDumpKnown`
- `needsRedump`
- `missingRequired`
- `incorrectChecksum`
- `incorrectLength`
- `targetNotFound`

This prevents a best-available result from erasing why it is not fully correct.

## Exit-code handling

The parser does not classify from exit status alone.

For exit `0`, it requires positive MAME summary evidence before returning `Complete` or `BestAvailable`. Contradictory failure diagnostics force `Unknown`.

For exit `2` (`EMU_ERR_MISSING_FILES`), detailed diagnostics decide between required-missing, incorrect, mixed failure, or unknown. A bare `romset ... is bad` is intentionally insufficient because MAME uses exit `2` for multiple failure causes.

Exit `5` (`EMU_ERR_NO_SUCH_SYSTEM`), absent process status, and currently unmodeled exit codes remain `Unknown`.

## Diagnostic distinctions

Ordering in the diagnostic parser is deliberate:

- `NOT FOUND BUT OPTIONAL` records `optionalMissing` and does not become required-missing.
- `NOT FOUND - NO GOOD DUMP KNOWN` records `noGoodDumpKnown` and does not become required-missing.
- ordinary `- NOT FOUND` records `missingRequired`.
- `INCORRECT CHECKSUM:` records incorrect checksum content.
- `INCORRECT LENGTH:` records incorrect length content.
- `NEEDS REDUMP` and `NO GOOD DUMP KNOWN` remain degradation facts rather than hard missing failures.

The parser therefore preserves the MT-503 distinction between fully correct and best available.

## Conservative compatibility behavior

MT-504 intentionally fails closed. It returns `Unknown` instead of guessing when:

- the process exit code is unavailable;
- an unknown nonzero exit code is returned;
- output does not contain a recognized positive or failure form;
- exit `0` conflicts with explicit missing/incorrect/bad-set diagnostics;
- exit `2` is present without diagnostics sufficient to identify the failure type;
- a target-not-found diagnostic is present.

No filename, archive, CHD, configured path, or directory-presence heuristic participates in the parser.

## Raw diagnostics

The result retains a combined stdout/stderr diagnostic excerpt for later UI and compatibility debugging. The excerpt is bounded to 16 KiB, records whether truncation occurred, and truncates only at a UTF-8 character boundary.

This supports MT-506 diagnostic display without allowing unbounded command output to accumulate in the structured result.

## Test coverage

Rust unit tests cover:

- confirmed-good media;
- optional missing media;
- known-undumped media;
- media needing redump;
- required missing media;
- whole-set-not-found output;
- incorrect checksum;
- incorrect length;
- mixed missing and incorrect failures;
- bare exit-2/bad-summary ambiguity;
- unknown target handling;
- contradictory success status and failure output;
- unrecognized successful output;
- absent process exit status;
- bounded UTF-8-safe raw diagnostic retention.

## Qualification evidence

The normalized implementation commit is `c247a2b1d390c6df5630b45df843264dd1570f8d`, with tree `3e102f0c9cd43faa963eb6d13b5de423d31258c8`.

GitHub Actions Tauri project run `34437228743` completed successfully against that exact normalized implementation SHA. The successful gate included:

- frontend formatting;
- ESLint;
- TypeScript typecheck;
- frontend tests;
- frontend production build;
- Rust formatting;
- Rust tests, including MT-504 parser tests;
- inherited 100,000-row library UX performance qualification;
- Rust Clippy with warnings denied;
- lockfile integrity.

The earlier formatted candidate tree also passed run `34436928076`; the normalized exact-head run above is the authoritative qualification.

## Acceptance conclusion

MT-504 represents every acceptance category in the MT-500 backlog: MAME-confirmed complete media, required missing content, incorrect content, optional-content distinction, and unknown/error behavior. It also preserves mixed failures and best-available degradation without promoting them to fully correct.

MT-504 is complete. MT-505 — Persist audit results with provenance — is next.
