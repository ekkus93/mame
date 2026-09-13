# MT-1700 — Performance qualification

Date: 2026-09-13
Branch: `ralph/mt-1700-performance-qualification`
Base: `97a490112613cd775ded9b16ac5bae425586f4dd`

## Scope

MT-1700 makes performance qualification reproducible instead of anecdotal. The implementation adds a checked artifact schema that records the benchmark corpus, current CI-enforced measurements, and release-lab measurements that require a real external MAME runtime, host hardware, or ROM-bearing test setup that cannot be committed to this repository.

## CI-enforced baselines

The `Tauri project / linux-quality` workflow now enforces two MT-1700 gates:

1. `Library UX performance qualification` runs the existing ignored Rust full-catalog performance test against a synthetic 100,000-row catalog. It covers library-ready behavior, full-catalog search latency, and tail-page responsiveness.
2. `MT-1700 performance artifact schema regression` runs `scripts/tauri/test-performance-qualification.py`, proving that the generated MT-1700 artifact still covers every canonical TODO metric before the workflow uploads the baseline artifact.

The workflow also writes and uploads:

`artifacts/performance/mt1700-performance-baseline.json`

The artifact is intentionally schema-checked. A future change that drops a benchmark category, removes a measurement method, or mislabels an enforcement state will fail CI before the artifact can be published.

## Release-lab baselines

The artifact separates CI-enforced synthetic measurements from release-protocol measurements that require a real machine/host pairing. The release-protocol entries cover:

- representative launch benchmarking;
- raster-machine frame/audio/fullscreen behavior;
- demanding-machine overhead;
- sidecar CPU, memory, frame-behavior, and launch-latency comparisons against direct MAME;
- metadata generation time, peak memory, database import time, and stale-refresh behavior.

Those measurements are represented in the same artifact schema so release notes can record real values without changing the CI contract.

## Embedded rendering

MT-1705 remains conditional by design. The current architecture launches MAME as a sidecar/native window instead of embedding the renderer. The MT-1700 artifact therefore marks embedded-render metrics as `deferred_until_mt1000` rather than pretending that frame pacing, input latency, GPU overhead, or A/V sync have been qualified for an embedded renderer that is not active.

## Exact-head trigger note

The branch records Tauri evidence on `e2955db96c15cc9eb260fb6fcd5a6c6a2891cc66`, where the workflow file and performance scripts changed together and the Tauri project workflow passed. The final branch head is a documentation-qualification head so the docs workflow also verifies the full MT-1700 documentation set before promotion.

## Files

- `.github/workflows/tauri-project.yml`
- `scripts/tauri/performance_qualification.py`
- `scripts/tauri/test-performance-qualification.py`
- `docs/MAME_TAURI_MT1700_PERFORMANCE_QUALIFICATION_2026-09-13.md`

## Local validation

The new Python scripts were syntax-checked and executed locally in the sandbox:

```text
python3 /tmp/mt1700/test-performance-qualification.py
python3 /tmp/mt1700/performance_qualification.py --write /tmp/mt1700.json --format pretty
```

Both completed successfully before the implementation was pushed.
