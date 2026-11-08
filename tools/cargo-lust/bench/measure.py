#!/usr/bin/env python3
"""Measure the Cargo coordinator on the copied, path-only full-Rust fixture.

No project sources, Cargo configuration, or toolchain installation are changed.
Every measurement owns its source copy, Cargo home, target dirs, and run dirs.
"""

from __future__ import annotations

import argparse
import base64
import datetime as dt
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import time


FIXTURE = Path(__file__).resolve().parent.parent / "fixtures" / "full-rust"
PROBE_FILE = Path("core/src/lib.rs")
PROBE = "pub fn benchmark_value() -> u32 { 41 } // BENCH_PROBE"
SCENARIOS = ("cold", "warm_noop", "body_edit", "signature_edit", "rejected_edit")
PROFILE = {
    "CARGO_PROFILE_DEV_OPT_LEVEL": "0",
    "CARGO_PROFILE_DEV_DEBUG": "2",
    "CARGO_PROFILE_DEV_DEBUG_ASSERTIONS": "true",
    "CARGO_PROFILE_DEV_OVERFLOW_CHECKS": "true",
    "CARGO_PROFILE_DEV_PANIC": "unwind",
    "CARGO_PROFILE_DEV_INCREMENTAL": "true",
    "CARGO_INCREMENTAL": "1",
}


class BenchmarkError(Exception):
    pass


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def tree_files(root: Path) -> list[dict]:
    """Hash relative names and contents; do not follow symlinks out of a tree."""
    result = []
    for path in sorted(root.rglob("*")):
        name = path.relative_to(root).as_posix()
        if path.is_symlink():
            result.append({"path": name, "symlink": os.readlink(path)})
        elif path.is_file():
            result.append({"path": name, "bytes": path.stat().st_size, "sha256": file_digest(path)})
    return result


def tree_digest(root: Path) -> str:
    return digest(json.dumps(tree_files(root), sort_keys=True, separators=(",", ":")).encode())


def stream_record(data: bytes) -> dict:
    record = {"bytes": len(data), "sha256": digest(data)}
    try:
        record["text"] = data.decode("utf-8")
    except UnicodeDecodeError:
        record["base64"] = base64.b64encode(data).decode("ascii")
    return record


def process(argv: list[str], cwd: Path, env: dict[str, str], timeout: float) -> dict:
    start = time.perf_counter_ns()
    child = subprocess.Popen(
        argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=os.name == "posix",
    )
    timed_out = False
    try:
        stdout, stderr = child.communicate(timeout=timeout)
    except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
        if os.name == "posix":
            try:
                os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        else:
            child.kill()
        stdout, stderr = child.communicate()
        if isinstance(error, KeyboardInterrupt):
            raise
        timed_out = True
    return {
        "argv": argv, "cwd": str(cwd), "returncode": child.returncode,
        "elapsed_ns": time.perf_counter_ns() - start, "timed_out": timed_out,
        "stdout": stream_record(stdout), "stderr": stream_record(stderr),
    }


def successful(result: dict) -> bool:
    return result["returncode"] == 0 and not result["timed_out"]


def require_success(result: dict, purpose: str) -> None:
    if not successful(result):
        raise BenchmarkError(f"{purpose} failed, exit={result['returncode']}, timeout={result['timed_out']}")


def executable(value: str | None, name: str) -> str:
    found = shutil.which(value or name)
    if found is None:
        raise BenchmarkError(f"Cannot find {name}; provide its executable path.")
    # Do not resolve symlinks: rustup proxies depend on their executable name.
    return os.path.abspath(found)


def clean_environment(cargo_home: Path) -> tuple[dict[str, str], list[str]]:
    env = os.environ.copy()
    removed = []
    exact = {
        "RUSTC", "RUSTDOC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTC_BOOTSTRAP",
        "RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS",
        "CARGO", "CARGO_HOME", "CARGO_TARGET_DIR", "CARGO_INCREMENTAL", "CARGO_BUILD_JOBS",
        "CARGO_BUILD_TARGET", "CARGO_MAKEFLAGS", "MAKEFLAGS", "MFLAGS",
        "LUST_CARGO", "LUSTC",
    }
    for key in list(env):
        if key in exact or key.startswith(("CARGO_PROFILE_", "CARGO_BUILD_", "CARGO_TARGET_", "CARGO_LUST_")):
            removed.append(key)
            del env[key]
    env.update(PROFILE)
    env["CARGO_HOME"] = str(cargo_home)
    env["CARGO_TERM_COLOR"] = "never"
    env["CARGO_NET_OFFLINE"] = "true"
    env["LC_ALL"] = "C"
    cargo_home.mkdir()
    return env, sorted(removed)


def choose_tools(args: argparse.Namespace, env: dict[str, str], root: Path, report: dict) -> tuple[str, str, str]:
    if args.toolchain:
        rustup = executable(args.rustup, "rustup")
        resolved = []
        for name in ("cargo", "rustc", "rustdoc"):
            result = process([rustup, "which", "--toolchain", args.toolchain, name], root, env, args.timeout)
            report["setup"].append(result)
            require_success(result, f"locating {name} for {args.toolchain}")
            resolved.append(executable(result["stdout"].get("text", "").strip(), name))
        env["RUSTUP_TOOLCHAIN"] = args.toolchain
        return tuple(resolved)
    return (
        executable(args.cargo or os.environ.get("CARGO"), "cargo"),
        executable(args.rustc or os.environ.get("RUSTC"), "rustc"),
        executable(args.rustdoc or os.environ.get("RUSTDOC"), "rustdoc"),
    )


def identify(cargo: str, rustc: str, coordinator: str, env: dict[str, str], root: Path, args: argparse.Namespace) -> dict:
    identities = {}
    for name, command in (("cargo", [cargo, "--version", "--verbose"]), ("rustc", [rustc, "-vV"]), ("coordinator", [coordinator, "--version"])):
        result = process(command, root, env, args.timeout)
        identities[name] = result
        require_success(result, f"identifying {name}")
    identities["coordinator"]["executable_sha256"] = file_digest(Path(coordinator))
    sysroot = process([rustc, "--print", "sysroot"], root, env, args.timeout)
    identities["sysroot"] = sysroot
    require_success(sysroot, "identifying the Rust sysroot")
    host = next((line.removeprefix("host: ") for line in identities["rustc"]["stdout"].get("text", "").splitlines() if line.startswith("host: ")), None)
    if host:
        backend_dir = Path(sysroot["stdout"].get("text", "").strip()) / "lib" / "rustlib" / host / "codegen-backends"
        identities["cranelift_backend_files"] = [
            {"path": str(path), "bytes": path.stat().st_size, "sha256": file_digest(path)}
            for path in sorted(backend_dir.glob("*cranelift*")) if path.is_file()
        ]
    return identities


def cargo_messages(result: dict) -> list[dict]:
    messages = []
    for line in result["stdout"].get("text", "").splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError as error:
            raise BenchmarkError(f"Cargo stdout contains a non-JSON line: {line[:200]}") from error
        if not isinstance(message, dict):
            raise BenchmarkError("Cargo stdout contains a non-object JSON message.")
        messages.append(message)
    return messages


def cargo_arguments(project: Path, target: Path, args: argparse.Namespace) -> list[str]:
    return [
        args.operation, "--manifest-path", str(project / "Cargo.toml"),
        "--package", "fixture-app", "--bin", "fixture-app", "--target-dir", str(target),
        "--jobs", str(args.jobs), "--offline", "--locked", "--message-format=json",
    ]


def selected_plan(coordinator: str, cargo: str, project: Path, root: Path, env: dict[str, str],
                  args: argparse.Namespace, report: dict) -> None:
    command = [coordinator, "--engine", args.engine, "plan"] + cargo_arguments(project, root / "plan-target", args)
    result = process(command, project, env, args.timeout)
    report["selection"] = {"command": result}
    require_success(result, "obtaining the coordinator plan")
    plan = json.loads(result["stdout"].get("text", ""))
    report["selection"]["plan"] = plan
    expected = "rustc" if args.operation == "check" else args.engine
    if plan.get("selected_engine") != expected:
        raise BenchmarkError(f"Expected engine {expected}, but coordinator plan selected {plan.get('selected_engine')}.")
    if plan.get("program") != cargo:
        raise BenchmarkError("The candidate plan does not use the baseline's exact Cargo executable.")
    if expected == "cranelift" and plan.get("environment", {}).get("CARGO_PROFILE_DEV_CODEGEN_BACKEND") != "cranelift":
        raise BenchmarkError("The candidate plan does not select Cranelift for the measured dev profile.")


def sample(route: str, prefix: list[str], project: Path, target: Path, run_dir: Path, source_hash: str,
           expected_probe: int, scenario: str, index: int, order: int, env: dict[str, str], args: argparse.Namespace) -> dict:
    command = prefix + cargo_arguments(project, target, args)
    start = time.perf_counter_ns()
    build = process(command, project, env, args.timeout)
    entry = {
        "route": route, "scenario": scenario, "sample": index, "order_in_pair": order,
        "source_sha256": source_hash, "build": build, "runtime": None,
        "artifacts": [], "diagnostic_codes": [], "validation_errors": [],
    }
    try:
        messages = cargo_messages(build)
        artifacts = [message for message in messages if message.get("reason") == "compiler-artifact"]
        entry["artifacts"] = [{"name": item["target"]["name"], "kind": item["target"]["kind"], "fresh": item["fresh"]} for item in artifacts]
        entry["diagnostic_codes"] = [
            message["message"]["code"]["code"] for message in messages
            if message.get("reason") == "compiler-message" and message.get("message", {}).get("code")
        ]
        if successful(build) and args.operation == "build":
            binaries = [item["executable"] for item in artifacts if item["target"]["name"] == "fixture-app" and item.get("executable")]
            if len(binaries) != 1:
                raise BenchmarkError("Expected exactly one fixture-app executable in Cargo's JSON output.")
            binary = Path(binaries[0]).resolve()
            if not binary.is_relative_to(target.resolve()):
                raise BenchmarkError("Cargo reported an executable outside the owned target directory.")
            run_dir.mkdir(parents=True)
            entry["runtime"] = process([str(binary)], run_dir, env, args.timeout)
            entry["build_and_run_ns"] = time.perf_counter_ns() - start
            entry["runtime"]["files"] = tree_files(run_dir)
            entry["artifact_bytes"] = binary.stat().st_size
            words = entry["runtime"]["stdout"].get("text", "").split()
            entry["runtime_probe_matches"] = all(word in words for word in (f"probe={expected_probe}", "panic=1", "drops=1"))
    except (BenchmarkError, KeyError, TypeError, OSError) as error:
        # Keep the raw compiler output even when its artifact protocol is wrong.
        entry["validation_errors"].append(str(error))
    entry["source_unchanged"] = tree_digest(project) == source_hash
    return entry


def pair_verdict(entries: list[dict], scenario: str, operation: str) -> dict:
    reasons = [f"{item['route']}: {error}" for item in entries for error in item["validation_errors"]]
    if any(not item["source_unchanged"] for item in entries):
        reasons.append("The compiler or build script changed the source snapshot.")
    if any(item["build"]["timed_out"] for item in entries):
        reasons.append("A compilation timed out.")
    if scenario == "rejected_edit":
        for item in entries:
            if successful(item["build"]) or "E0308" not in item["diagnostic_codes"]:
                reasons.append(f"{item['route']} did not reject the deliberate type error with E0308.")
        return {"passed": not reasons, "reasons": reasons, "acceptance_parity": not reasons, "runtime_parity": None}
    for item in entries:
        if not successful(item["build"]):
            reasons.append(f"{item['route']} did not compile successfully.")
        elif scenario == "warm_noop" and any(not artifact["fresh"] for artifact in item["artifacts"]):
            reasons.append(f"{item['route']} rebuilt an artifact in the warm no-op scenario.")
        elif scenario in ("body_edit", "signature_edit"):
            rebuilt = {artifact["name"].replace("-", "_") for artifact in item["artifacts"] if not artifact["fresh"]}
            if not {"fixture_core", "fixture_app"}.issubset(rebuilt):
                reasons.append(f"{item['route']} did not rebuild the edited library and its dependent binary.")
    runtime_parity = None
    if operation == "build":
        runtime_parity = False
        if all(item["runtime"] is not None for item in entries):
            left, right = (item["runtime"] for item in entries)
            runtime_parity = (
                successful(left) and successful(right)
                and left["stdout"] == right["stdout"]
                and left["stderr"] == right["stderr"]
                and left["files"] == right["files"]
            )
            if not all(item.get("runtime_probe_matches", False) for item in entries):
                reasons.append("Runtime output did not match the edited value or panic/drop probes.")
        if not runtime_parity:
            reasons.append("Executable exit status, stdout, stderr, or written files differed or execution failed.")
    return {"passed": not reasons, "reasons": reasons, "acceptance_parity": all(successful(item["build"]) for item in entries), "runtime_parity": runtime_parity}


def statistics_for(samples: list[dict], include_run: bool) -> dict:
    result = {}
    for scenario in SCENARIOS:
        rows = [item for item in samples if item["scenario"] == scenario]
        if not rows:
            continue
        metric = "build_and_run_ns" if include_run and scenario != "rejected_edit" else "build.elapsed_ns"
        summary = {"metric": metric, "routes": {}}
        for route in ("cargo", "coordinator"):
            values = [item["build_and_run_ns"] if metric == "build_and_run_ns" else item["build"]["elapsed_ns"] for item in rows if item["route"] == route]
            if not values:
                continue
            ordered = sorted(values)
            summary["routes"][route] = {
                "samples": len(values), "median_ns": statistics.median(values),
                "p95_ns": ordered[math.ceil(0.95 * len(ordered)) - 1],
                "min_ns": ordered[0], "max_ns": ordered[-1],
            }
        if len(summary["routes"]) == 2:
            summary["baseline_over_coordinator_median"] = summary["routes"]["cargo"]["median_ns"] / summary["routes"]["coordinator"]["median_ns"]
        result[scenario] = summary
    return result


def hardware() -> dict:
    data = {"system": platform.system(), "release": platform.release(), "machine": platform.machine(), "logical_cpus": os.cpu_count(), "python": platform.python_version()}
    if hasattr(os, "sched_getaffinity"):
        data["available_cpus"] = len(os.sched_getaffinity(0))
    cpuinfo = Path("/proc/cpuinfo")
    if cpuinfo.exists():
        data["cpu_model"] = next((line.split(":", 1)[1].strip() for line in cpuinfo.read_text().splitlines() if line.startswith("model name")), None)
    return data


def benchmark(args: argparse.Namespace, report: dict) -> None:
    coordinator = executable(args.coordinator, "cargo-lust")
    with tempfile.TemporaryDirectory(prefix="cargo-lust-bench-") as directory:
        root = Path(directory).resolve()
        project = root / "project"
        if any(path.is_symlink() for path in FIXTURE.rglob("*")):
            raise BenchmarkError("The benchmark fixture must not contain symlinks.")
        shutil.copytree(FIXTURE, project, ignore=shutil.ignore_patterns("target", ".git"))
        original = (project / PROBE_FILE).read_text()
        if original.count(PROBE) != 1:
            raise BenchmarkError("The fixture must contain exactly one documented BENCH_PROBE line.")
        env, removed = clean_environment(root / "cargo-home")
        cargo, rustc, rustdoc = choose_tools(args, env, root, report)
        env.update({"CARGO": cargo, "LUST_CARGO": cargo, "RUSTC": rustc, "RUSTDOC": rustdoc})
        report["removed_environment_keys"] = removed
        report["identities"] = identify(cargo, rustc, coordinator, env, root, args)
        selected_plan(coordinator, cargo, project, root, env, args, report)
        lock = process([cargo, "generate-lockfile", "--offline", "--manifest-path", str(project / "Cargo.toml")], project, env, args.timeout)
        report["setup"].append(lock)
        require_success(lock, "preparing the copied fixture's lockfile")
        report["fixture"] = {"path": str(FIXTURE), "files": tree_files(project), "base_sha256": tree_digest(project)}
        prefixes = {"cargo": [cargo], "coordinator": [coordinator, "--engine", args.engine]}

        def run_pair(scenario: str, index: int, source: str, expected_probe: int, warmup: bool = False) -> None:
            # A no-op must preserve mtimes too: rewriting identical bytes can make
            # Cargo rebuild and would measure an edit instead of an unchanged tree.
            if (project / PROBE_FILE).read_text() != source:
                (project / PROBE_FILE).write_text(source)
            source_hash = tree_digest(project)
            order = ["cargo", "coordinator"] if (index + SCENARIOS.index(scenario)) % 2 == 0 else ["coordinator", "cargo"]
            entries = []
            for position, route in enumerate(order):
                target = root / "targets" / scenario / route
                if scenario == "cold":
                    target = target / str(index)
                entry = sample(route, prefixes[route], project, target, root / "runs" / scenario / str(index) / route,
                               source_hash, expected_probe, scenario, index, position, env, args)
                entries.append(entry)
                report["warmups" if warmup else "samples"].append(entry)
            # A warmup starts from an empty target directory, so freshness expectations do not apply.
            verdict = pair_verdict(entries, "cold" if warmup else scenario, args.operation)
            verdict.update({"scenario": scenario, "sample": index, "warmup": warmup, "source_sha256": source_hash, "order": order})
            report["pairs"].append(verdict)
            print(f"{scenario} {'warmup' if warmup else index + 1}: {'pass' if verdict['passed'] else 'FAIL'}", file=sys.stderr, flush=True)
            if not verdict["passed"]:
                raise BenchmarkError("; ".join(verdict["reasons"]))

        for scenario in args.scenarios:
            if scenario != "cold":
                run_pair(scenario, -1, original, 41, warmup=True)
            for index in range(args.samples):
                source, expected_probe = original, 41
                if scenario == "body_edit" and index % 2 == 0:
                    source, expected_probe = original.replace(PROBE, PROBE.replace("{ 41 }", "{ 42 }")), 42
                elif scenario == "signature_edit" and index % 2 == 0:
                    source = original.replace(PROBE, PROBE.replace("-> u32", "-> u64"))
                elif scenario == "rejected_edit":
                    # Each rejected snapshot differs, avoiding a repeated-error cache benchmark.
                    source = original.replace(PROBE, PROBE.replace("{ 41 }", f'{{ "type_error_{index}" }}'))
                run_pair(scenario, index, source, expected_probe)


def write_report(report: dict, output: str) -> None:
    text = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if output == "-":
        sys.stdout.write(text)
    else:
        path = Path(output).absolute()
        path.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent, prefix=path.name + ".", delete=False) as stream:
            temporary = Path(stream.name)
            stream.write(text)
        try:
            os.replace(temporary, path)
        finally:
            temporary.unlink(missing_ok=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--coordinator", help="Built cargo-lust executable; otherwise locate cargo-lust on PATH.")
    parser.add_argument("--engine", choices=("rustc", "cranelift"), default="rustc")
    parser.add_argument("--operation", choices=("build", "check"), default="build")
    parser.add_argument("--samples", type=int, default=6, help="Measured pairs per scenario, default 6; even counts balance ordering.")
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--timeout", type=float, default=120.0, help="Maximum seconds per child command.")
    parser.add_argument("--scenarios", nargs="+", choices=SCENARIOS, default=list(SCENARIOS))
    parser.add_argument("--build-and-run", action="store_true", help="Use build plus executable runtime as the primary metric; runtime parity is always checked for builds.")
    parser.add_argument("--toolchain", help="Installed rustup toolchain; resolve Cargo, rustc and rustdoc from that same toolchain.")
    parser.add_argument("--rustup", help="rustup path, used only with --toolchain.")
    parser.add_argument("--cargo", help="Cargo executable; default CARGO or PATH. Mutually exclusive with --toolchain.")
    parser.add_argument("--rustc", help="rustc executable; default RUSTC or PATH. Mutually exclusive with --toolchain.")
    parser.add_argument("--rustdoc", help="rustdoc executable; default RUSTDOC or PATH. Mutually exclusive with --toolchain.")
    parser.add_argument("--output", default="-", help="JSON report path, or - for stdout.")
    args = parser.parse_args()
    if args.samples < 1 or args.jobs < 1 or not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error("samples, jobs and timeout must be positive")
    if args.toolchain and any((args.cargo, args.rustc, args.rustdoc)):
        parser.error("--toolchain cannot be combined with explicit Cargo/rustc/rustdoc paths")
    if args.build_and_run and args.operation != "build":
        parser.error("--build-and-run requires --operation build")
    if len(args.scenarios) != len(set(args.scenarios)):
        parser.error("scenarios must not contain duplicates")
    report = {
        "schema_version": 1, "created_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "hardware": hardware(), "engine": args.engine, "operation": args.operation,
        "settings": {"samples_per_scenario": args.samples, "jobs": args.jobs, "profile": PROFILE, "features": "fixture defaults, unchanged", "target": "host", "scenarios": args.scenarios, "primary_metric": "build_and_run" if args.build_and_run else args.operation},
        "method": {"clock": "time.perf_counter_ns", "interleaving": "alternate the first route in each pair", "cold": "empty Cargo target directory; installed sysroot and OS page cache are not cold", "warm": "one excluded paired warmup; target directories retained", "source": "one shared copied source snapshot per pair, separate target directories", "p95": "nearest-rank percentile; at small sample counts this is the maximum", "limits": "small synthetic fixture; not evidence of broad full-project acceleration; no peak-memory or compiler-stage profiling"},
        "setup": [], "warmups": [], "samples": [], "pairs": [], "errors": [], "passed": False,
    }
    code = 0
    try:
        benchmark(args, report)
        report["passed"] = True
    except KeyboardInterrupt:
        report["errors"].append("Interrupted.")
        code = 130
    except (BenchmarkError, OSError, ValueError) as error:
        report["errors"].append(str(error))
        print(f"benchmark failed: {error}", file=sys.stderr)
        code = 1
    # Failed qualification never receives a speed ratio, even if some timed samples exist.
    report["statistics"] = statistics_for(report["samples"], args.build_and_run) if report["passed"] else {}
    write_report(report, args.output)
    for scenario, summary in report["statistics"].items():
        left, right = (summary["routes"][route] for route in ("cargo", "coordinator"))
        print(f"{scenario}: Cargo {left['median_ns'] / 1e6:.3f} ms, coordinator {right['median_ns'] / 1e6:.3f} ms, ratio {summary['baseline_over_coordinator_median']:.3f}x", file=sys.stderr)
    return code


if __name__ == "__main__":
    raise SystemExit(main())
