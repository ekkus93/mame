#!/usr/bin/env python3
"""Generate and validate the MT-1700 performance qualification artifact."""

from __future__ import annotations

import argparse
import copy
import json
import os
import platform
import time
from pathlib import Path
from typing import Any

SCHEMA_VERSION = 1
TASK_ID = "MT-1700"

METRICS: dict[str, dict[str, dict[str, str]]] = {
    "benchmark_corpus": {
        "full_catalog_search_benchmark": {
            "enforcement": "ci_enforced",
            "measurement": "100,000-row synthetic catalog startup, search, and tail-page latency.",
            "ci_gate": "Tauri project / linux-quality / Library UX performance qualification",
        },
        "representative_launch_benchmark": {
            "enforcement": "release_protocol",
            "measurement": "Synthetic runtime launch smoke plus release-lab external-MAME launch-to-ready timing.",
            "ci_gate": "Tauri release and package smoke workflows for synthetic runtime paths",
        },
        "representative_raster_machine": {
            "enforcement": "release_protocol",
            "measurement": "Release-lab raster machine frame pacing, audio, fullscreen, and responsiveness.",
            "ci_gate": "Manual release-lab capture; ROM-bearing assets are not committed",
        },
        "representative_demanding_machine": {
            "enforcement": "release_protocol",
            "measurement": "Release-lab demanding machine CPU, memory, launch latency, and frame behavior.",
            "ci_gate": "Manual release-lab capture; host capability dependent",
        },
    },
    "frontend_performance_baseline": {
        "cold_app_startup": {
            "enforcement": "ci_enforced",
            "measurement": "Tauri production build smoke and development-window smoke startup.",
            "ci_gate": "Tauri project / linux-release-qualification",
        },
        "library_ready_time": {
            "enforcement": "ci_enforced",
            "measurement": "Catalog open plus first-page query against the 100,000-row corpus.",
            "ci_gate": "Tauri project / linux-quality / Library UX performance qualification",
        },
        "search_latency": {
            "enforcement": "ci_enforced",
            "measurement": "Median repeated full-catalog exact text-search latency.",
            "ci_gate": "Tauri project / linux-quality / Library UX performance qualification",
        },
        "large_list_responsiveness": {
            "enforcement": "ci_enforced",
            "measurement": "Tail-page query latency against the 100,000-row corpus.",
            "ci_gate": "Tauri project / linux-quality / Library UX performance qualification",
        },
        "artwork_loading_behavior": {
            "enforcement": "release_protocol",
            "measurement": "Release-lab local/missing/cached artwork behavior without privileged remote navigation.",
            "ci_gate": "Manual release-lab capture using this artifact schema",
        },
    },
    "sidecar_overhead_baseline": {
        "cpu_usage": {
            "enforcement": "release_protocol",
            "measurement": "Direct MAME CPU versus Tauri-launched equivalent on the same host.",
            "ci_gate": "Manual release-lab capture using this artifact schema",
        },
        "memory_usage": {
            "enforcement": "release_protocol",
            "measurement": "Direct MAME RSS/working-set versus Tauri-launched equivalent.",
            "ci_gate": "Manual release-lab capture using this artifact schema",
        },
        "frame_behavior": {
            "enforcement": "release_protocol",
            "measurement": "Direct MAME versus Tauri-launched frame pacing, fullscreen, and stutter notes.",
            "ci_gate": "Manual release-lab capture using this artifact schema",
        },
        "launch_latency": {
            "enforcement": "release_protocol",
            "measurement": "Direct MAME launch-to-ready versus Tauri launch-to-ready latency.",
            "ci_gate": "Manual release-lab capture using this artifact schema",
        },
    },
    "metadata_generation_performance": {
        "generation_time": {
            "enforcement": "release_protocol",
            "measurement": "External MAME -listxml capture time and fixture parse time.",
            "ci_gate": "Captured listxml fixture provenance plus release-lab capture",
        },
        "peak_memory": {
            "enforcement": "release_protocol",
            "measurement": "Peak RSS during -listxml capture, parser traversal, and SQLite import.",
            "ci_gate": "Manual release-lab capture using this artifact schema",
        },
        "db_import_time": {
            "enforcement": "release_protocol",
            "measurement": "SQLite metadata import elapsed time for representative and full-catalog listxml.",
            "ci_gate": "Manual release-lab capture using this artifact schema",
        },
        "incremental_stale_refresh_behavior": {
            "enforcement": "release_protocol",
            "measurement": "Refresh decision latency for unchanged and stale metadata identities.",
            "ci_gate": "Manual release-lab capture using this artifact schema",
        },
    },
    "embedded_render_qualification": {
        "frame_pacing_comparison": {
            "enforcement": "deferred_until_mt1000",
            "measurement": "Only applicable if MT-1000 embedded rendering proceeds.",
            "ci_gate": "Not applicable to the current sidecar/windowed architecture",
        },
        "input_latency_comparison": {
            "enforcement": "deferred_until_mt1000",
            "measurement": "Only applicable if MT-1000 embedded rendering proceeds.",
            "ci_gate": "Not applicable to the current sidecar/windowed architecture",
        },
        "cpu_gpu_overhead_comparison": {
            "enforcement": "deferred_until_mt1000",
            "measurement": "Only applicable if MT-1000 embedded rendering proceeds.",
            "ci_gate": "Not applicable to the current sidecar/windowed architecture",
        },
        "audio_video_sync": {
            "enforcement": "deferred_until_mt1000",
            "measurement": "Only applicable if MT-1000 embedded rendering proceeds.",
            "ci_gate": "Not applicable to the current sidecar/windowed architecture",
        },
    },
}

ENFORCEMENT_LEVELS = {"ci_enforced", "release_protocol", "deferred_until_mt1000"}
BUDGETS = {
    "full_catalog_rows": 100_000,
    "library_page_size": 100,
    "first_page_startup_query_ms": 2_000,
    "full_catalog_search_median_ms": 1_000,
    "tail_page_query_ms": 2_000,
    "tauri_development_window_smoke_seconds": 30,
}


def build_baseline() -> dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "task_id": TASK_ID,
        "status": "qualification_harness_defined",
        "ci_context": {
            "github_sha": os.environ.get("GITHUB_SHA"),
            "github_ref": os.environ.get("GITHUB_REF"),
            "github_run_id": os.environ.get("GITHUB_RUN_ID"),
            "generated_at_epoch_ms": int(time.time() * 1000),
            "python_version": platform.python_version(),
            "platform": platform.platform(),
        },
        "budgets": BUDGETS,
        **copy.deepcopy(METRICS),
    }


def validate_baseline(baseline: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if baseline.get("schema_version") != SCHEMA_VERSION:
        errors.append(f"schema_version must be {SCHEMA_VERSION}")
    if baseline.get("task_id") != TASK_ID:
        errors.append(f"task_id must be {TASK_ID}")
    for key, value in baseline.get("budgets", {}).items():
        if not isinstance(value, int) or value <= 0:
            errors.append(f"budget {key} must be a positive integer")
    for section, metrics in METRICS.items():
        actual = baseline.get(section)
        if not isinstance(actual, dict):
            errors.append(f"{section} must be an object")
            continue
        missing = sorted(set(metrics) - set(actual))
        for metric in missing:
            errors.append(f"{section}.{metric} must be an object")
        for metric in metrics:
            item = actual.get(metric)
            if not isinstance(item, dict):
                errors.append(f"{section}.{metric} must be an object")
                continue
            if item.get("enforcement") not in ENFORCEMENT_LEVELS:
                errors.append(f"{section}.{metric}.enforcement has an invalid value")
            for field in ("measurement", "ci_gate"):
                if not isinstance(item.get(field), str) or not item[field].strip():
                    errors.append(f"{section}.{metric}.{field} must be a non-empty string")
    return errors


def write_baseline(path: Path, *, pretty: bool) -> None:
    baseline = build_baseline()
    errors = validate_baseline(baseline)
    if errors:
        raise SystemExit("invalid MT-1700 baseline:\n" + "\n".join(errors))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(baseline, indent=2 if pretty else None, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--write",
        type=Path,
        default=Path("artifacts/performance/mt1700-performance-baseline.json"),
    )
    parser.add_argument("--format", choices=("compact", "pretty"), default="compact")
    args = parser.parse_args()
    write_baseline(args.write, pretty=args.format == "pretty")


if __name__ == "__main__":
    main()
