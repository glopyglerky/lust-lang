#!/usr/bin/env python3
"""Run fixed safe-library probes without replacing the full required native gate."""
from __future__ import annotations

import argparse
import importlib.util
import json
from pathlib import Path
import shutil


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--cargo", type=Path, required=True)
    parser.add_argument("--oracle", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--diagnostic-without-trace", action="store_true")
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[1]
    runner = repository / "tools/qualify-native-library.py"
    spec = importlib.util.spec_from_file_location("native_library_gate", runner)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    try:
        module.qualifier(args.native, args.cargo, args.oracle,
                         repository / "tests/native-library-safe", args.output,
                         diagnostic_without_trace=args.diagnostic_without_trace,
                         scope="safe-primitives-supplement")
    finally:
        report_path = args.output / "report.json"
        if report_path.is_file():
            report = json.loads(report_path.read_text())
            report["supplemental_runner_sha256"] = module.digest(Path(__file__).resolve())
            report["retained_supplemental_runner"] = "safe-runner-source.py"
            shutil.copy2(Path(__file__).resolve(), args.output / "safe-runner-source.py")
            report_path.write_text(json.dumps(report, indent=2) + "\n")


if __name__ == "__main__":
    main()
