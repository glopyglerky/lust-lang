#!/usr/bin/env python3
"""Diagnostic phase attribution for the pre-namespace-fix 5b98ffb6 candidate.

The instrumented executable emits one timing record after compiler work.
Cache-enabled and disabled comparisons also change the pre-existing unit
cache policy, so their whole-process difference is not a request-cache cost.
The paired uninstrumented control estimates observer/build-layout effects.
Nothing here is release qualification or an optimized candidate.
"""
from pathlib import Path
import hashlib
import json
import os
import platform
import shutil
import statistics
import subprocess
import tempfile
import time


ROOT = Path(__file__).resolve().parent
REPOSITORY = ROOT.parent / "lustr-cache-profile"
BINARIES = {
    "uninstrumented": ROOT.parent / "native-cache-experiments/lazy-native",
    "instrumented": ROOT / "native-profile",
}
SAMPLES = 12
PREFIX = b"LUSTR_CACHE_PHASES "
work = Path(tempfile.mkdtemp(prefix="lustr-cache-phases-", dir="/tmp"))
(ROOT / "logs").mkdir()
(ROOT / "fixtures").mkdir()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


report = {
    "schema": "lustr.native-cache-phase-diagnostic.v1",
    "qualified": False,
    "production_performance_claim": False,
    "scope": __doc__,
    "source_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPOSITORY, text=True).strip(),
    "runner_sha256": sha(Path(__file__)),
    "host": platform.platform(),
    "binaries": {name: {"path": str(path), "sha256": sha(path)} for name, path in BINARIES.items()},
    "source_sha256": {path.name: sha(path) for path in sorted((REPOSITORY / "crates/lustr-native/src").glob("*.rs"))},
    "profile": {"name": "gate", "opt_level": 1, "lto": "off", "codegen_units": 16},
    "phase_policy": "Instant wall-clock nanoseconds with fixed atomic counters. One stderr record after compile_total. Nested subphases must not be added to their containing phases.",
    "sampling_policy": "Twelve timed pairs plus retained preflight per case/mode. Alternating binary and mode order. No dropped samples. Fresh cache and TMPDIR for every cold compile, OS page cache retained.",
    "cache_work_directory": str(work),
    "commands": [],
    "summaries": {},
    "manifest_probe": {},
    "failures": [],
}


def save():
    (ROOT / "report.json").write_text(json.dumps(report, indent=2) + "\n")


def execute(case, sample, mode, lane, source, expected, directory, env, *, warm=False):
    directory.mkdir(parents=True, exist_ok=warm)
    cache, runtime = directory / "cache", directory / "runtime"
    if not warm:
        cache.mkdir()
        runtime.mkdir()
        assert not any(cache.iterdir()) and not any(runtime.iterdir())
    artifact = directory / "phase_probe"
    dep_info = directory / "phase_probe.d"
    artifact.unlink(missing_ok=True)
    dep_info.unlink(missing_ok=True)
    env = dict(env, LUST_CACHE_DIR=str(cache), TMPDIR=str(runtime), LUST_NO_DAEMON="1")
    if mode == "uncached":
        env["LUST_NO_CACHE"] = "1"
    else:
        env.pop("LUST_NO_CACHE", None)
    argv = [str(BINARIES[lane]), "--crate-name", "phase_probe", "--crate-type", "bin", "--edition=2024",
            "-C", "opt-level=0", "-C", "debuginfo=0", "-C", "panic=unwind",
            "-C", "overflow-checks=yes", "-C", "debug-assertions=yes", "--emit=link,dep-info",
            "--out-dir", str(directory), str(source)]
    start = time.perf_counter_ns()
    process = subprocess.run(argv, cwd=directory, env=env, capture_output=True, timeout=120)
    elapsed = time.perf_counter_ns() - start
    index = len(report["commands"])
    stdout_path, stderr_path = ROOT / "logs" / f"{index:04d}.stdout", ROOT / "logs" / f"{index:04d}.stderr"
    stdout_path.write_bytes(process.stdout)
    stderr_path.write_bytes(process.stderr)
    lines = [line[len(PREFIX):] for line in process.stderr.splitlines() if line.startswith(PREFIX)]
    assert len(lines) == (1 if lane == "instrumented" else 0), process.stderr
    phases = json.loads(lines[0]) if lines else None
    runtime_result = subprocess.run([artifact], cwd=directory, env=env, capture_output=True, timeout=30) if process.returncode == 0 else None
    correct = (process.returncode == 0 and runtime_result is not None and runtime_result.returncode == 0
               and runtime_result.stdout == expected and dep_info.is_file())
    markers = list(cache.glob("req-native-*"))
    row = {"index": index, "case": case, "sample": sample, "mode": mode, "lane": lane,
           "warm": warm, "argv": argv, "cwd": str(directory), "source_sha256": sha(source),
           "elapsed_ns": elapsed, "exit_code": process.returncode, "correct": correct,
           "stdout": str(stdout_path.relative_to(ROOT)), "stderr": str(stderr_path.relative_to(ROOT)),
           "runtime_stdout": runtime_result.stdout.decode() if runtime_result else None,
           "cache_dir": str(cache), "runtime_tmp": str(runtime), "fresh_before_compile": not warm,
           "cache_disabled": env.get("LUST_NO_CACHE"), "admissions_after_compile": len(markers),
           "artifact_bytes": artifact.stat().st_size if artifact.exists() else None,
           "environment_entries": len(env), "environment_bytes": sum(len(k) + len(v) for k, v in env.items()),
           "phases": phases}
    report["commands"].append(row)
    save()
    assert correct, row
    assert len(markers) == (0 if mode == "uncached" else 1), row
    if phases is not None:
        assert phases["compile_total"]["count"] == 1
        assert phases["native_compile"]["count"] == (0 if warm else 1)
        assert phases["store_upstream"]["count"] == (0 if mode == "uncached" or warm else 1)
        assert phases["lookup_marker_read"]["count"] == (0 if mode == "uncached" else 1)
    return row


def summary(values):
    ordered = sorted(values)
    return {"count": len(values), "median_ns": statistics.median(values),
            "mean_ns": statistics.fmean(values), "min_ns": ordered[0], "max_ns": ordered[-1],
            "p95_ns": ordered[min(len(ordered) - 1, int(len(ordered) * 0.95))]}


base_env = {key: value for key, value in os.environ.items() if not key.startswith(("LUST_", "LUSTR_"))}
sources = {
    "generics-traits": ((REPOSITORY / "tests/native-qualification/existing-generics-traits.rs").read_text(), b"14\n"),
    "unsafe-scalar": ('unsafe fn add(value: i32) -> i32 { value + 1 }\nfn main() { println!("{}", unsafe { add(41) }); }\n', b"42\n"),
    "unsafe-generic": ('unsafe fn id<T>(value: T) -> T { value }\nfn main() { println!("{}", unsafe { id(42) }); }\n', b"42\n"),
}
functions = "".join(f"fn f{i}(x: i64) -> i64 {{ x + {i} }}\n" for i in range(16))
calls = " + ".join(f"f{i}(1)" for i in range(16))
sources["unsafe-many-functions"] = (f'unsafe fn id(value: i64) -> i64 {{ value }}\n{functions}\nfn main() {{ println!("{{}}", unsafe {{ id({calls}) }}); }}\n', b"136\n")

try:
    assert report["source_commit"] == "5b98ffb63da0139d778ef4a26de28b1d38fd4a0e"
    assert not subprocess.check_output(["git", "status", "--porcelain", "--", "compiler/native"], cwd=REPOSITORY, text=True)
    for case, (text, expected) in sources.items():
        source = ROOT / "fixtures" / f"{case}.rs"
        source.write_text(text)
        assert not any((parent / "Cargo.toml").exists() for parent in source.parents)
        for sample in range(-1, SAMPLES):
            modes = ("fresh-cache", "uncached") if sample % 2 == 0 else ("uncached", "fresh-cache")
            for mode in modes:
                lanes = ("uninstrumented", "instrumented") if sample % 2 == 0 else ("instrumented", "uninstrumented")
                for lane in lanes:
                    execute(case, sample, mode, lane, source, expected,
                            work / case / mode / lane / str(sample + 1), base_env)
        for mode in ("fresh-cache", "uncached"):
            rows = [row for row in report["commands"] if row["case"] == case and row["mode"] == mode and row["sample"] >= 0]
            lanes = {lane: [row for row in rows if row["lane"] == lane] for lane in BINARIES}
            measured = lanes["instrumented"]
            phases = {name: summary([row["phases"][name]["ns"] for row in measured]) for name in measured[0]["phases"]}
            for row in measured:
                row["request_cache_work_ns"] = sum(row["phases"][name]["ns"] for name in ("cache_prepare_total", "lookup_total", "store_total"))
            result = {"wall": {lane: summary([row["elapsed_ns"] for row in values]) for lane, values in lanes.items()},
                      "phases": phases, "request_cache_work": summary([row["request_cache_work_ns"] for row in measured])}
            report["summaries"][case + "/" + mode] = result
            print(json.dumps({"case": case, "mode": mode, "instrumented_median_ms": result["wall"]["instrumented"]["median_ns"] / 1e6,
                              "request_cache_work_median_ms": result["request_cache_work"]["median_ns"] / 1e6,
                              "native_compile_median_ms": phases["native_compile"]["median_ns"] / 1e6}), flush=True)
        save()

    manifest = work / "manifest-probe"
    manifest.mkdir()
    (manifest / "Cargo.toml").write_text('[package]\nname = "phase_manifest"\nversion = "0.1.0"\nedition = "2024"\nbuild = "build.rs"\n')
    (manifest / "build.rs").write_text('fn main() { println!("cargo:rerun-if-changed=build.rs"); }\n')
    source = manifest / "main.rs"
    source.write_text(sources["unsafe-scalar"][0])
    sentinel = manifest / "sentinel"
    sentinel.mkdir()
    cargo = sentinel / "cargo"
    cargo.write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$LUSTR_PHASE_CARGO_LOG"\nexit 97\n')
    cargo.chmod(0o755)
    cargo_log = manifest / "cargo.log"
    env = dict(base_env, PATH=str(sentinel) + ":/usr/bin:/bin", LUSTR_PHASE_CARGO_LOG=str(cargo_log))
    records = []
    for warm in (False, True):
        before = len(cargo_log.read_text().splitlines()) if cargo_log.exists() else 0
        row = execute("manifest-probe", 0 if not warm else 1, "fresh-cache", "instrumented", source, b"42\n",
                      manifest / "output", env, warm=warm)
        calls = cargo_log.read_text().splitlines() if cargo_log.exists() else []
        row["nested_cargo_calls"] = calls[before:]
        records.append(row["index"])
    report["manifest_probe"] = {"command_indices": records,
                                "calls": cargo_log.read_text().splitlines() if cargo_log.exists() else [],
                                "scope": "Logging shell sentinel exits 97; no real Cargo/build script runs. Instrumented source deliberately retains the pre-fix cargo-native cache namespace."}
    (ROOT / "manifest-cargo.log").write_text(cargo_log.read_text() if cargo_log.exists() else "")
    report["success"] = True
except Exception as error:
    report["success"] = False
    report["failures"].append(repr(error))
    raise
finally:
    save()

print(json.dumps({"report": str(ROOT / "report.json"), "commands": len(report["commands"]),
                  "manifest_probe": report["manifest_probe"], "qualified": False}), flush=True)
