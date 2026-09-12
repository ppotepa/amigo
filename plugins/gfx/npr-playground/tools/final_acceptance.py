"""Build a machine-readable Drawing Studio acceptance report from benchmark artifacts."""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def load_jsonl(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--matrix", type=Path, default=Path("target/viewport-layer-matrix-brush-defaults-full.jsonl"))
    parser.add_argument("--soak", type=Path, default=Path("target/viewport-brush-defaults-soak-300.jsonl"))
    parser.add_argument("--output", type=Path, default=Path("target/drawing-studio-acceptance.json"))
    args = parser.parse_args()

    matrix = load_jsonl(args.matrix)
    cases = [row for row in matrix if row.get("kind") == "case" and row.get("valid")]
    expected = {
        (model, look, layers, size, mode, scenario)
        for model in ("cube", "sphere", "suzanne", "multi")
        for look in ("comic-ink", "pencil-study")
        for layers in (4, 8, 16)
        for size in ("640x360", "960x540", "1280x720")
        for mode in ("native_gpu", "local_rgba", "jpeg")
        for scenario in ("orbit", "spin", "pause")
    }
    actual = {
        (row.get("model"), row.get("look"), row.get("layers"), row.get("size"), row.get("mode"), row.get("scenario"))
        for row in cases
    }
    interval_p95 = [row["interval_p95_ms"] for row in cases if isinstance(row.get("interval_p95_ms"), (int, float))]
    max_interval_p95 = max(interval_p95, default=None)
    worst_case = max(
        (row for row in cases if isinstance(row.get("interval_p95_ms"), (int, float))),
        key=lambda row: row["interval_p95_ms"],
        default=None,
    )
    drop_counts = [
        row["diagnostics"]["stale_readback_drops"]
        for row in cases
        if isinstance(row.get("diagnostics"), dict)
        and isinstance(row["diagnostics"].get("stale_readback_drops"), int)
    ]
    if not drop_counts:
        drop_counts = [
            frame["header"]["stages"]["stale_readback_drops"]
            for row in cases
            for frame in row.get("raw", [])
            if isinstance(frame.get("header", {}).get("stages", {}).get("stale_readback_drops"), int)
        ]
    missing = sorted(expected - actual)
    unexpected = sorted(actual - expected)
    soak_rows = load_jsonl(args.soak)
    soak_candidates = [row for row in soak_rows if row.get("kind") == "soak"]
    soak = max(soak_candidates, key=lambda row: row.get("seconds", 0.0), default=None)
    close = next((row for row in reversed(soak_rows) if row.get("kind") == "close"), None)
    report = {
        "artifact": "drawing-studio-acceptance",
        "matrix": {
            "path": str(args.matrix),
            "valid_case_records": len(cases),
            "unique_expected_cases": len(expected),
            "unique_actual_cases": len(actual),
            "complete": not missing and not unexpected,
            "missing": missing,
            "unexpected": unexpected,
        },
        "performance": {
            "max_interval_p95_ms": max_interval_p95,
            "worst_case": {key: worst_case.get(key) for key in ("model", "look", "layers", "size", "mode", "scenario", "interval_p95_ms")} if worst_case else None,
            "target_interval_ms": 20.0,
            "within_target": max_interval_p95 is None or max_interval_p95 <= 20.0,
            "note": "Values above target require an explicit baseline decision before final performance sign-off.",
        },
        "transport": {
            "stale_readback_drops": sum(drop_counts) if drop_counts else None,
            "diagnostic_samples": len(drop_counts),
            "note": "Null means the benchmark artifact predates the diagnostic field; runtime diagnostics expose it for new runs.",
        },
        "soak": {
            "path": str(args.soak),
            "present": soak is not None,
            "seconds": soak.get("seconds") if soak else None,
            "engine_exited": close.get("engine_exited") if close else None,
            "error": soak.get("error") if soak else "missing soak record",
        },
    }
    report["accepted"] = bool(
        report["matrix"]["complete"]
        and report["soak"]["present"]
        and (report["soak"]["seconds"] or 0) >= 300
        and report["soak"]["engine_exited"] is True
        and not report["soak"]["error"]
    )
    report["functional_acceptance"] = report["accepted"]
    report["performance_signoff"] = report["performance"]["within_target"]
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
