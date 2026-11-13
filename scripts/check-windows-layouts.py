#!/usr/bin/env python3
"""Compare Windows target layout checks with pinned Rust metadata compilation.

This checks frontend layout constants. It does not link or run Windows code.
All targets and cases run even when a previous comparison fails.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
TARGETS = (
    "x86_64-pc-windows-gnu",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
)
CASES = (
    ("tests/pass/layout_sync_target.rs", None),
    ("tests/pass/layout_mutex_target.rs", None),
    ("tests/fail/layout_mutex_target_extent.rs", "L0110"),
    ("tests/fail/layout_mutex_target_alignment.rs", "L0110"),
    ("tests/fail/layout_mutex_target_body_extent.rs", "L0110"),
)


def diagnostics(output):
    records = []
    for line in output.splitlines():
        record = json.loads(line)
        if not isinstance(record, dict):
            raise ValueError("diagnostic is not a JSON object")
        records.append(record)
    return records


def rust_rejection(records, source):
    """Require a located type mismatch, not a target setup error or crash."""
    errors = []
    for record in records:
        if record.get("level") != "error":
            continue
        code, spans = record.get("code"), record.get("spans", [])
        if code is None and not spans and record.get("message", "").startswith("aborting due to "):
            continue
        if not isinstance(code, dict) or code.get("code") != "E0308":
            return False
        if not any(span.get("is_primary") and Path(span.get("file_name", "")).resolve() == source
                   for span in spans):
            return False
        errors.append(record)
    return bool(errors)


def native_rejection(records, source, expected):
    errors = [record for record in records if record.get("severity") == "error"]
    return bool(errors) and all(record.get("kind") == "diagnostic" and record.get("code") == expected
                                and Path(record.get("file", "")).resolve() == source for record in errors)


def check(command, compiler, source, expected, env):
    print("$ " + shlex.join(map(str, command)), flush=True)
    try:
        result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=60)
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"FAIL {compiler}: {error}", flush=True)
        return False
    for output in (result.stdout, result.stderr):
        if output:
            print(output, end="" if output.endswith("\n") else "\n", flush=True)
    try:
        records = diagnostics(result.stdout) + diagnostics(result.stderr)
    except (ValueError, TypeError) as error:
        print(f"FAIL {compiler}: invalid diagnostic output: {error}", flush=True)
        return False
    if expected is None:
        errors = [record for record in records
                  if record.get("level") == "error" or record.get("severity") == "error"]
        ok = result.returncode == 0 and not errors
    else:
        rejected = (rust_rejection(records, source) if compiler == "Rust"
                    else native_rejection(records, source, expected))
        ok = result.returncode == 1 and rejected
    verdict = "accept" if expected is None else "reject with a type mismatch"
    print(f"{'PASS' if ok else 'FAIL'} {compiler}: expected {verdict}; exit={result.returncode}", flush=True)
    return ok


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("lust", type=Path)
    args = parser.parse_args()
    lust = args.lust.resolve()
    env = dict(os.environ, LUST_NO_DAEMON="1")
    failed = 0
    checks = 0
    with tempfile.TemporaryDirectory(prefix="lust-windows-layout-") as directory:
        for target in TARGETS:
            for relative, expected in CASES:
                source = ROOT / relative
                print(f"Checking {target}: {relative}", flush=True)
                if not source.is_file() or (expected is not None and
                        not re.match(r"// expect: " + expected + r"\b", source.read_text())):
                    print("FAIL: missing fixture or changed native diagnostic expectation", flush=True)
                    failed += 1
                    continue
                output = Path(directory) / (target + "-" + source.stem + ".rmeta")
                commands = (
                    ("Rust", ["rustup", "run", "1.99.0", "rustc", "--edition=2024", "--target", target,
                              "--emit=metadata", "--error-format=json", source, "-o", output]),
                    ("Lust", [lust, "check", source, "--target", target, "--json"]),
                )
                for compiler, command in commands:
                    checks += 1
                    if not check(command, compiler, source, expected, env):
                        failed += 1
    print(f"Windows layout frontend checks: {checks} checks, {failed} failures", flush=True)
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
