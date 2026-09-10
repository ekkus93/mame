# MT-507 — Bounded Bulk Audit Qualification

**Document date:** 2026-09-09  
**Repository:** `ekkus93/mame`  
**Task:** MT-507 — Add bounded bulk audit  
**Implementation branch:** `ralph/mt-507-bounded-bulk-audit`  
**Qualified implementation commit:** `a5d076b806686635950096b6922677f3b547ec98`  
**Parent / MT-506 closure:** `4d4d3eedf6f6e238a569d0bc7e233357b059222e`  
**Exact-head qualification run:** `34442886244`

## Scope

MT-507 adds a bounded whole-library audit operation on top of the per-machine MAME audit and persistence contracts completed in MT-504 through MT-506.

The implementation deliberately chooses **safe restartability** rather than a second persistent bulk-job checkpoint format. Every successful per-machine audit is independently persisted through the MT-505 provenance-aware store. Cancelling, application interruption, or a later restart therefore does not create a partially committed batch transaction or require trusting separate resume metadata.

## Acceptance behavior

### Cancellation

- Exactly one bulk audit may be active at a time.
- A cancel request transitions the job from `running` to `cancelling` and prevents workers from dequeuing new machines.
- MAME audit processes already in flight are allowed to finish or reach the existing bounded per-machine audit timeout.
- When workers drain after cancellation, the job becomes `cancelled`.
- The frontend exposes Cancel while a job is active and explains the cooperative in-flight behavior.

Cancellation is therefore bounded and deterministic without introducing a second process-kill path that could conflict with the existing per-machine audit runner.

### Progress

The typed backend status contract exposes:

- schema version;
- job ID;
- state;
- configured maximum parallelism;
- total runnable machines in the prepared queue;
- successfully persisted result count;
- failed machine count;
- currently active worker count;
- last processed machine;
- bounded last-error text.

The frontend polls status only while the job is `running` or `cancelling`, renders a progress element, and reports processed/total, persisted, failed, and active counts.

### Bounded parallelism

- Default concurrency: `2` MAME audit processes.
- User-selectable concurrency: `1` through `4`.
- Values outside that range are rejected by the Rust backend.
- The work queue contains only runnable, non-device machines from the active catalog.
- Catalog enumeration is deterministic by machine short name.
- Catalog size is guarded at 100,000 entries.
- Query paging is bounded to 200 catalog rows per query.
- Only the selected worker count may execute per-machine MAME audits concurrently.

### Safe restartability

A completed per-machine audit is persisted immediately using the MT-505 provenance contract before the worker advances. If a bulk audit is cancelled or interrupted, already committed machine results remain usable only while their recorded MAME identity and content-path provenance are still current.

Starting another bulk audit rebuilds the deterministic runnable-machine queue and audits it again. This avoids hidden or stale bulk-job resume state while satisfying MT-507's requirement that expensive work be safely restartable.

## Failure behavior

- An overlapping start is rejected with `MAME_BULK_AUDIT_ALREADY_RUNNING`.
- Invalid parallelism is rejected with `MAME_BULK_AUDIT_PARALLELISM_INVALID`.
- Catalog setup, paging, queue coordination, or status coordination failures become explicit failed-job states.
- Worker panics are surfaced as `MAME_BULK_AUDIT_WORKER_PANICKED` rather than silently losing work.
- A per-machine audit failure increments the machine failure count and records a bounded last-error summary; it does not silently disappear or abort unrelated machines.
- Last-error status text is capped at 512 characters.

## Tests and qualification

Exact-head GitHub Actions run `34442886244` qualified commit `a5d076b806686635950096b6922677f3b547ec98` successfully.

The run passed:

- frontend Prettier format check;
- ESLint;
- strict TypeScript typecheck;
- frontend unit tests, including typed bulk-audit command envelopes;
- frontend production build;
- `cargo fmt --check`;
- Rust unit tests, including concurrency-bound validation, overlapping-job rejection, cancellation transition, and bounded status errors;
- inherited MT-410 100,000-row library performance qualification;
- Clippy with warnings denied;
- lockfile integrity.

The release-only qualification job was skipped as expected for this normal branch push.

## Implementation note

Earlier intermediate runs exposed formatting-only failures in Prettier and rustfmt. Their canonical formatter output was materialized on the branch, the temporary formatter workflows were removed, and the implementation was normalized back to one commit directly on the MT-506 closure parent before final qualification. No formatter helper workflow exists in the qualified implementation tree.

## Result

MT-507 acceptance criteria are satisfied by the exact qualified implementation commit above. The central TODO may be reconciled to mark cancellation, progress, bounded parallelism, and safely restartable behavior complete. MT-508 remains the next active task and is intentionally untouched by this qualification.
