# MT-410 Library UX Performance Qualification

**Date:** 2026-09-09  
**Repository:** `ekkus93/mame`  
**Branch:** `ralph/mt-410-library-ux-performance`  
**Qualified implementation:** `8bb2e862ffd6f19e23311487183c3c31e248a937`  
**Authoritative CI:** Tauri project run `34413919454`

## Scope

MT-410 qualifies the existing library architecture against four requirements:

1. measure startup catalog load;
2. measure search latency;
3. exercise a full-scale catalog;
4. confirm that high-cardinality catalog data cannot expand the frontend render workload without bound.

This task is a qualification and regression-gate task. It does not change the library query semantics or replace the existing pagination model with virtualization.

## Repeatable qualification harness

The Rust metadata test suite now contains an ignored test named `full_catalog_library_queries_meet_interactive_budgets`. The normal Rust unit-test pass leaves this deliberately expensive test ignored, while the Tauri `linux-quality` job invokes it explicitly with `--ignored --nocapture` so its measurements are visible in CI and performance regressions fail the build.

The harness creates a temporary file-backed SQLite catalog through the normal `CatalogRepository`, inserts one active metadata generation and **100,000 synthetic machine rows** in a single seed transaction, closes the seeding scope, and then exercises the same paged machine-query path used by the application. The synthetic catalog is intentionally larger than needed for a smoke test; it is a controlled stress corpus, not a claim about the exact number of machines in any particular MAME release.

The timed operations are:

- database reopen plus the first 100-row machine page;
- five case-insensitive substring searches across short name, description, and manufacturer, reporting median and maximum latency;
- the final 100-row page at offset 99,900.

Catalog construction is not included in the interactive latency measurements because metadata generation/import is a separate background workflow. The qualification verifies query-time behavior after a usable catalog exists.

## Budgets and observed results

Run `34413919454` executed on GitHub-hosted Ubuntu 24.04 with Rust 1.98.1. The exact qualification output was:

```text
MT-410 library-performance catalog_rows=100000 page_size=100 startup_ms=50 search_median_ms=114 search_max_ms=115 tail_page_ms=226
```

| Measurement | Budget | Observed | Result |
| --- | ---: | ---: | --- |
| Catalog reopen + first page | 2,000 ms | 50 ms | PASS |
| Full-catalog substring search, median of 5 | 1,000 ms | 114 ms | PASS |
| Full-catalog substring search, maximum of 5 | informational | 115 ms | PASS |
| Last 100-row page | 2,000 ms | 226 ms | PASS |

The qualification also asserts that the catalog reports exactly 100,000 matching rows, that first/tail queries return only the configured 100-row page, and that a unique search target placed at the end of the synthetic catalog is found correctly.

## Bounded frontend render workload

The current UI is structurally bounded by pagination rather than by rendering the full catalog and relying on the browser to cope with it:

- `LIBRARY_PAGE_SIZE` is 100, so the primary machine browser requests at most 100 machine items for its normal page.
- The Rust query layer independently rejects machine page sizes above 200, providing a backend defense against an accidentally unbounded frontend request.
- `LibraryBrowser` maps only `page.items` into machine rows; it never maps the full catalog result set into React elements.
- The MT-409 software browser uses a separate page size of 25 and likewise renders only its current page of software items.

Therefore a 100,000-row catalog does not create 100,000 machine DOM rows. The high-cardinality catalog workload remains in SQLite, while the frontend receives and renders a bounded page. No additional list virtualization is required for the current paged UX. If the design later moves to infinite scrolling, large client-side aggregation, or substantially larger page sizes, virtualization should be re-evaluated and this qualification updated.

This bound applies specifically to high-cardinality machine/software result rows. It is not a claim that every element in the entire application DOM is capped at 100 or 125 nodes; detail metadata, controls, and other UI elements are separate from the catalog-row bound.

## CI qualification

Tauri project run `34413919454` qualified exact implementation commit `8bb2e862ffd6f19e23311487183c3c31e248a937` and passed:

- frontend Prettier check;
- ESLint;
- TypeScript typecheck;
- 27 frontend tests across 8 test files;
- frontend production build;
- `cargo fmt --check`;
- standard Rust tests: 76 passed, 0 failed, with the MT-410 performance test intentionally ignored in that ordinary pass;
- explicit MT-410 100,000-row performance qualification;
- Clippy with warnings denied;
- lockfile-integrity check.

The explicit performance test passed in 2.41 seconds including synthetic catalog construction after the test binary had compiled. Its interactive measurements are the per-operation figures shown above.

## Interpretation and limitations

The result establishes a repeatable regression gate at the Rust/SQLite query boundary and confirms that the current paged frontend architecture prevents full-catalog row rendering. It is deliberately conservative about what it proves:

- the corpus is synthetic rather than a captured contemporary MAME catalog;
- the measurement covers SQLite open/query work in the Rust backend, not Tauri IPC serialization, WebView layout/paint, or GPU rendering;
- metadata XML generation/import time is excluded because it is not the interactive catalog-query path;
- GitHub-hosted runner timing is useful for regression qualification, not a promise that every end-user machine will reproduce the same millisecond values.

Those limitations do not weaken the central MT-410 conclusion: the application can query a 100,000-row catalog within explicit interactive budgets while keeping machine/software result rendering bounded by pagination.

## MT-410 conclusion

**PASS.** Startup catalog access, search latency, full-scale catalog behavior, and bounded high-cardinality render workload are qualified at implementation commit `8bb2e862ffd6f19e23311487183c3c31e248a937` by Tauri project run `34413919454`.
