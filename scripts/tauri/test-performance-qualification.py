#!/usr/bin/env python3
"""Regression tests for the MT-1700 performance qualification artifact."""

from __future__ import annotations

import importlib.util
import json
import tempfile
from pathlib import Path

SCRIPT = Path(__file__).with_name("performance_qualification.py")


def load_module():
    spec = importlib.util.spec_from_file_location("performance_qualification", SCRIPT)
    if spec is None or spec.loader is None:
        raise AssertionError(f"unable to load {SCRIPT}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def assert_valid(module, baseline) -> None:
    errors = module.validate_baseline(baseline)
    if errors:
        raise AssertionError("MT-1700 baseline validation failed:\n" + "\n".join(errors))


def test_canonical_baseline_covers_every_todo_requirement() -> None:
    module = load_module()
    baseline = module.build_baseline()
    assert_valid(module, baseline)
    assert baseline["benchmark_corpus"]["full_catalog_search_benchmark"]["enforcement"] == "ci_enforced"
    assert baseline["frontend_performance_baseline"]["search_latency"]["enforcement"] == "ci_enforced"
    assert baseline["embedded_render_qualification"]["audio_video_sync"]["enforcement"] == "deferred_until_mt1000"


def test_validation_rejects_missing_metric() -> None:
    module = load_module()
    baseline = module.build_baseline()
    del baseline["sidecar_overhead_baseline"]["memory_usage"]
    errors = module.validate_baseline(baseline)
    assert any("memory_usage" in error for error in errors), errors


def test_written_artifact_is_valid_json() -> None:
    module = load_module()
    with tempfile.TemporaryDirectory() as directory:
        output = Path(directory) / "baseline.json"
        module.write_baseline(output, pretty=True)
        baseline = json.loads(output.read_text(encoding="utf-8"))
    assert_valid(module, baseline)


if __name__ == "__main__":
    test_canonical_baseline_covers_every_todo_requirement()
    test_validation_rejects_missing_metric()
    test_written_artifact_is_valid_json()
