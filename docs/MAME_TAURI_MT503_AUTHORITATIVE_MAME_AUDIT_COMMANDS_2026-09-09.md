# MT-503 — Authoritative MAME audit commands

Date: 2026-09-09

## Scope

MT-503 determines which MAME commands are authoritative for media auditing and records the output and exit-code semantics that MT-504 must parse. It does not implement the parser, persist audit results, add UI actions, or infer availability from filesystem presence.

The contract is grounded in the MAME source in this repository rather than in filename heuristics. The relevant implementation is primarily in:

- `src/frontend/mame/clifront.cpp`
- `src/frontend/mame/audit.cpp`
- `src/frontend/mame/audit.h`
- `src/emu/main.h`
- `src/emu/emuopts.cpp`

## Selected commands

### Machine ROM/media audit

The authoritative command for auditing one machine/system is:

```text
mame -verifyroms <machine>
```

`verifyroms` calls `media_auditor::audit_media(AUDIT_VALIDATE_FAST)` for the selected driver and therefore uses MAME's own media definitions, parent/clone relationships, checksums, disk definitions, optional flags, and search rules.

For MT-506 per-machine auditing, this is the primary command.

The command also accepts patterns and can audit multiple matching systems, but MT-506 should initially invoke it with one exact machine short name so the result has an unambiguous target and bounded output.

### Software-list audit associated with a system

The authoritative command for auditing software attached to a system is:

```text
mame -verifysoftware <system>
```

`verifysoftware` enumerates the original software lists attached to matching systems and audits each software item with `media_auditor::audit_software(..., AUDIT_VALIDATE_FAST)`.

This command audits the software lists associated with a system; it is not a single-software-item command.

### Software-list-wide audit

MAME also exposes:

```text
mame -verifysoftlist <software-list>
```

This audits a named software list. It can be useful for later bulk software-list workflows, but it is not the primary MT-506 per-machine command and still audits a list rather than one selected software item.

## MAME audit status model

`media_auditor` defines five aggregate summary states:

```text
CORRECT
NONE_NEEDED
BEST_AVAILABLE
INCORRECT
NOTFOUND
```

The audit record layer also exposes finer substatus values including:

```text
GOOD
GOOD_NEEDS_REDUMP
FOUND_NODUMP
FOUND_BAD_CHECKSUM
FOUND_WRONG_LENGTH
NOT_FOUND
NOT_FOUND_NODUMP
NOT_FOUND_OPTIONAL
UNVERIFIED
```

MT-504 must preserve enough of this distinction to avoid collapsing materially different outcomes.

## Representative output forms

The following are canonical output forms derived directly from the MAME `print_summary` and `media_auditor::summarize` format strings. They are grammar examples for the parser rather than a claim that this repository snapshot contains a prebuilt MAME executable that was run locally for MT-503.

### Correct set

```text
romset <name> is good
1 romsets found, 1 were OK.
```

`CORRECT` is the strongest positive result and should be the basis for a future `complete`/confirmed-good state.

### Best available

```text
romset <name> is best available
1 romsets found, 1 were OK.
```

`BEST_AVAILABLE` is considered acceptable by MAME's aggregate command accounting, but it is not equivalent to `CORRECT`. Reasons can include known-undumped media, media needing redump, or optional media not being present.

Representative detail lines include:

```text
<set>: <file> (...) - NEEDS REDUMP
<set>: <file> (...) - NO GOOD DUMP KNOWN
<set>: <file> (...) - NOT FOUND - NO GOOD DUMP KNOWN
<set>: <file> (...) - NOT FOUND BUT OPTIONAL
```

MT-504 must therefore retain the distinction between fully correct and best-available/optional-degraded results.

### Missing required content

Representative detail lines include:

```text
<set>: <file> (...) - NOT FOUND
```

For a requested set for which MAME finds no usable media, `verifyroms` can terminate with:

```text
romset "<name>" not found!
```

A missing required file is an audit failure and must not be treated as runnable merely because a similarly named archive or directory exists.

### Incorrect content

Representative detail forms include:

```text
<set>: <file> (...) - INCORRECT LENGTH: <actual> bytes
```

and:

```text
<set>: <file> (...) - INCORRECT CHECKSUM:
EXPECTED: <expected hashes>
   FOUND: <actual hashes>
```

The aggregate set summary then uses:

```text
romset <name> is bad
```

This is semantically distinct from required content being absent.

### No ROMs needed

`media_auditor::NONE_NEEDED` is a separate internal summary. `print_summary` records it as `is best available` when the caller requests a summary for it. MT-504 should not invent a missing-media failure solely because no ROM files are required.

### Unknown system

For an exact target that does not match a known system, `verifyroms` throws a no-such-system error such as:

```text
No matching systems found for '<name>'
```

This is not a missing-ROM result. It is a command/target error.

## Exit-code contract

MAME defines the relevant process exit codes in `src/emu/main.h`:

```text
0  EMU_ERR_NONE
2  EMU_ERR_MISSING_FILES
5  EMU_ERR_NO_SUCH_SYSTEM
```

Other MAME fatal/configuration/device/identification errors use other nonzero values and must remain `unknown/error` unless explicitly modeled later.

### Exit 0

For the selected audit commands, exit `0` means the command completed without a fatal audit failure. The parser must still inspect the output to distinguish `CORRECT` from `BEST_AVAILABLE`/`NONE_NEEDED` and to retain optional/undumped diagnostics.

### Exit 2 is not synonymous with "missing"

This is a critical MT-504 requirement.

`verifyroms` throws `EMU_ERR_MISSING_FILES` not only when a requested set is absent but also when one or more audited sets are `INCORRECT`. `verifysoftware` and `verifysoftlist` use the same code for failed media audits.

Therefore:

```text
exit_code == 2
```

means that the media audit failed, but the detailed output must determine whether the structured result is:

- missing required content;
- incorrect length/checksum;
- a mixed failure containing both;
- or another explicit media-audit failure.

MT-504 must never implement `2 => missing` as a complete classification rule.

### Exit 5

`EMU_ERR_NO_SUCH_SYSTEM` means the requested system or software-list target did not resolve. It must map to an unknown/error target condition, not to missing content.

## Search-path semantics

MAME exposes a single media search-path option:

```text
-rompath
```

Internally this is `OPTION_MEDIAPATH`, a `MULTIPATH` option described by MAME as the path to ROM sets and hard disk images. Software auditing also uses MAME's media/search-path machinery.

MT-501 intentionally stores ROM, software, and CHD locations as separate application concepts. MT-503 does not redefine that persistent model and does not choose a cross-group precedence policy. When MT-506 constructs an audit invocation, it must compose the configured application paths into MAME's media search path deterministically and without lossy path conversion. That composition policy must preserve the user's configured ordering semantics and must be covered by tests.

## Why filesystem presence is not authoritative

The application must not infer audit success from any of the following alone:

- `<rompath>/<machine>.zip` exists;
- a directory named after the machine exists;
- a CHD file exists;
- a configured path is accessible;
- the machine has a metadata entry;
- a software-list archive exists.

MAME's auditor understands information that simple presence checks do not, including expected lengths and hashes, parent/clone sharing, devices, disks, software-list hierarchy, optional media, known-undumped media, and bad/redump status.

The authoritative availability result therefore comes from MAME's audit command and parsed diagnostics, not from a pre-scan of filesystem names.

## MT-504 parser requirements derived from MT-503

MT-504 must parse stdout/stderr and process exit status together. At minimum it must represent:

- confirmed correct/complete (`CORRECT`);
- best available/acceptable without falsely calling it fully correct (`BEST_AVAILABLE` and `NONE_NEEDED` where applicable);
- missing required content (`NOT_FOUND` / required set absence);
- incorrect content (`FOUND_BAD_CHECKSUM`, `FOUND_WRONG_LENGTH`, aggregate `INCORRECT`);
- optional missing content (`NOT_FOUND_OPTIONAL`);
- known-undumped/redump conditions where exposed;
- unknown target (`EMU_ERR_NO_SUCH_SYSTEM`);
- unrecognized or contradictory output as `unknown/error` rather than guessing.

Raw bounded diagnostic text should be retained alongside the structured result for MT-506 UI use and future parser compatibility.

## Source-derived acceptance conclusions

MT-503 selects `-verifyroms <machine>` as the authoritative per-machine audit command, with `-verifysoftware <system>` and `-verifysoftlist <list>` documented for software-list auditing. MAME's own output/status grammar and exit-code behavior are sufficient to implement MT-504 without filesystem heuristics.

The most important compatibility rules are:

1. Do not equate exit code `2` with missing files only.
2. Do not equate `BEST_AVAILABLE` with fully `CORRECT`.
3. Preserve optional-missing and known-undumped distinctions where MAME exposes them.
4. Treat no-such-system and unrecognized failures as unknown/error.
5. Never promote file presence alone to audited/runnable status.


## Qualification evidence

The source-grounded MT-503 contract commit is `b8f03258ee917322668b67c7f3308a9b2d054b44`. GitHub Actions documentation run `34432467797` completed successfully against that exact commit, including dependency installation, HTML documentation build, PDF documentation build, and artifact upload.

MT-503 is therefore closed. MT-504 — Implement audit parser — is the next active task.
