#!/usr/bin/env python3
"""Build and qualify the unchanged root Lust compiler with LLVM and Cranelift.

Uses only Python's standard library and the adjacent measure.py recording
helpers. The pinned backend and tcc must already be built. Cargo fetch is the
only network-capable build operation and runs before any timed samples.
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
import re
import shutil
import statistics
import subprocess
import sys
import tempfile

import measure as common


SCENARIOS = ("cold", "warm_noop")
BINS = {"lust", "lust-core"}
FIXTURES = Path(__file__).resolve().parent.parent / "fixtures" / "subset"
TEST_SUMMARY = re.compile(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;")


class ProjectError(common.BenchmarkError):
    pass


def checkpoint(report: dict, output: str) -> None:
    if output != "-":
        common.write_report(report, output)


def validate_output(args: argparse.Namespace) -> None:
    if args.output == "-":
        return
    project = Path(args.project).resolve()
    output = Path(args.output).resolve()
    inputs = [Path(args.coordinator), Path(__file__), Path(common.__file__)]
    if os.environ.get("LUST_CRANELIFT_BACKEND"):
        inputs.append(Path(os.environ["LUST_CRANELIFT_BACKEND"]))
    if output in {path.resolve() for path in inputs}:
        raise ProjectError("The report cannot overwrite a compiler, backend, or benchmark script.")
    if output.is_relative_to(project):
        if output.parent != project:
            raise ProjectError("Write the report outside the source tree or as a new file in the checkout root.")
        result = subprocess.run([common.executable(None, "git"), "-C", str(project), "ls-files", "--error-unmatch", "--", output.name], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if result.returncode != 1:
            raise ProjectError("The report cannot overwrite a tracked project file, or Git could not verify its location.")
    tcc = Path(os.environ.get("LUST_TCC_HOME", str(project / "third_party" / "tcc")))
    if not tcc.is_absolute():
        tcc = project / tcc
    if output.is_relative_to(tcc.resolve()):
        raise ProjectError("The report cannot be written into the tcc input tree.")


def encoded(data: dict) -> bytes:
    return data["text"].encode("utf-8") if "text" in data else base64.b64decode(data["base64"], validate=True)


def fingerprint(records: list[dict]) -> str:
    return hashlib.sha256(json.dumps(records, sort_keys=True, ensure_ascii=True, separators=(",", ":")).encode()).hexdigest()


def tracked_files(project: Path, git: str) -> list[dict]:
    result = subprocess.run([git, "-C", str(project), "ls-files", "--stage", "-z"], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    records = []
    for entry in result.stdout.split(b"\0"):
        if not entry:
            continue
        metadata, separator, name = entry.partition(b"\t")
        if not separator:
            raise ProjectError("Cannot parse the project's tracked file inventory.")
        mode, object_id, stage = metadata.decode("ascii").split()
        if stage != "0":
            raise ProjectError("The project contains unresolved index conflicts.")
        relative = os.fsdecode(name)
        path = project / relative
        item = {"path": relative, "index_mode": mode, "index_object": object_id}
        if mode == "160000":
            item["kind"] = "submodule_gitlink"
        elif path.is_symlink():
            item.update({"kind": "symlink", "link": os.readlink(path)})
            if path.is_file():
                item["target_sha256"] = common.file_digest(path)
        elif path.is_file():
            item.update({"kind": "file", "bytes": path.stat().st_size, "sha256": common.file_digest(path), "mode": path.stat().st_mode & 0o777})
        else:
            raise ProjectError(f"Tracked source is missing or is not a regular file: {relative}")
        records.append(item)
    if not records:
        raise ProjectError("The project has no tracked files.")
    return records


def backend_environment() -> tuple[dict[str, str], list[str]]:
    env = os.environ.copy()
    removed = []
    for name in list(env):
        if (name.startswith("CARGO_PROFILE_") and name.endswith("_CODEGEN_BACKEND")) or name == "LUST_CRANELIFT_BACKEND":
            removed.append(name)
            del env[name]
    for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS"):
        if "codegen-backend" in env.get(name, ""):
            raise ProjectError(f"{name} selects a backend; remove that override so the baseline can use its ordinary backend.")
    return env, sorted(removed)


def command_record(argv: list[str], cwd: Path, env: dict[str, str], timeout: float, rows: list[dict], purpose: str) -> dict:
    row = common.process(argv, cwd, env, timeout)
    row["purpose"] = purpose
    rows.append(row)
    return row


def require(row: dict, purpose: str) -> None:
    common.require_success(row, purpose)


def resolve_tools(project: Path, env: dict[str, str], report: dict, timeout: float) -> dict[str, str]:
    rustup = common.executable(None, "rustup")
    selected = env.get("RUSTUP_TOOLCHAIN")
    result = {}
    for name in ("cargo", "rustc", "rustdoc"):
        arguments = [rustup, "which"]
        if selected:
            arguments.extend(["--toolchain", selected])
        arguments.append(name)
        row = command_record(arguments, project, env, timeout, report["setup"], f"resolve {name}")
        require(row, f"resolve installed {name}")
        path = Path(row["stdout"].get("text", "").strip())
        if not path.is_absolute() or not path.is_file():
            raise ProjectError(f"rustup did not resolve an installed absolute {name} executable.")
        result[name] = str(path)
    if len({str(Path(path).parent) for path in result.values()}) != 1:
        raise ProjectError("Cargo, rustc, and rustdoc must come from the same installed toolchain directory.")
    env.update({"CARGO": result["cargo"], "LUST_CARGO": result["cargo"], "RUSTC": result["rustc"], "RUSTDOC": result["rustdoc"]})
    report["tools"] = {"paths": result, "requested_toolchain": selected, "versions": {}}
    for name, arguments in (("cargo", ["--version", "--verbose"]), ("rustc", ["-vV"]), ("rustdoc", ["--version", "--verbose"])):
        row = command_record([result[name], *arguments], project, env, timeout, report["setup"], f"identify {name}")
        require(row, f"identify {name}")
        report["tools"]["versions"][name] = row["stdout"]
    compiler_version = report["tools"]["versions"]["rustc"].get("text", "")
    if "nightly" not in compiler_version:
        raise ProjectError("The complementary backend requires its matching installed nightly toolchain; source setup's env.sh first.")
    return result


def arguments(operation: str, project: Path, target: Path, jobs: int, messages: bool = True) -> list[str]:
    args = [operation, "--manifest-path", str(project / "Cargo.toml"), "--bins", "--target-dir", str(target), "--jobs", str(jobs), "--locked", "--offline"]
    if messages:
        args.append("--message-format=json")
    return args


def artifacts(row: dict, target: Path, warm: bool) -> dict[str, dict]:
    messages = common.cargo_messages(row)
    found = {}
    for message in messages:
        if message.get("reason") != "compiler-artifact":
            continue
        name = message.get("target", {}).get("name")
        if name not in BINS or "bin" not in message.get("target", {}).get("kind", []) or not message.get("executable"):
            continue
        path = Path(message["executable"])
        if not path.is_absolute() or not path.is_file() or not path.resolve().is_relative_to(target.resolve()):
            raise ProjectError(f"Cargo returned a missing or external {name} executable.")
        if name in found:
            raise ProjectError(f"Cargo returned more than one {name} executable.")
        if warm and message.get("fresh") is not True:
            raise ProjectError(f"Unchanged warm build unexpectedly rebuilt {name}.")
        if not warm and message.get("fresh") is not False:
            raise ProjectError(f"Empty-target build unexpectedly reused {name}.")
        profile = message.get("profile", {})
        if profile.get("opt_level") != "0" or profile.get("debuginfo") != 2 or profile.get("debug_assertions") is not True or profile.get("overflow_checks") is not True:
            raise ProjectError(f"{name} was not built with the expected unchanged default dev settings: {profile}")
        found[name] = {"path": str(path), "bytes": path.stat().st_size, "sha256": common.file_digest(path), "fresh": message["fresh"], "profile": profile}
    if set(found) != BINS:
        raise ProjectError("The root project must build both lust and lust-core binaries.")
    if Path(found["lust"]["path"]).parent != Path(found["lust-core"]["path"]).parent:
        raise ProjectError("Lust's client and core must be sibling executables.")
    return found


def runtime_probe(executable: Path, directory: Path, env: dict[str, str], timeout: float, result: dict) -> None:
    # Both routes receive identical absolute source paths, but never share a
    # prior executable, daemon, or Lust compile cache. In particular, these two
    # builds can have the same upstream LUST_BUILD_HASH despite different Rust
    # codegen backends, so simply disabling the daemon would not be sufficient.
    if directory.exists():
        shutil.rmtree(directory)
    directory.mkdir()
    for name in ("supported.rs", "unsupported.rs"):
        shutil.copyfile(FIXTURES / name, directory / name)
    probe_env = env.copy()
    probe_env.update({"LUST_NO_DAEMON": "1", "LUST_NO_CACHE": "1", "LUST_CACHE_DIR": str(directory / ".lust-cache")})
    for name in ("LUST_STD_SRC", "LUST_LIBRARY", "LUST_SOURCES_READY", "LUST_DAEMON_SOCKET"):
        probe_env.pop(name, None)
    cases = {
        "version": ["--version"],
        "capabilities": ["capabilities", "--json"],
        "supported": ["run", str(directory / "supported.rs")],
        "unsupported": ["check", str(directory / "unsupported.rs")],
    }
    for name, args in cases.items():
        row = common.process([str(executable), *args], directory, probe_env, timeout)
        result["commands"][name] = row
        if name == "unsupported":
            if row["timed_out"] or row["returncode"] == 0 or b"L0405" not in encoded(row["stderr"]):
                raise ProjectError("The built Lust compiler must reject the async fixture with L0405.")
        else:
            require(row, f"built Lust {name} probe")
        if name == "supported" and encoded(row["stdout"]) != b"42\n":
            raise ProjectError("The built Lust compiler produced incorrect supported-program output.")
        if name == "capabilities":
            result["capabilities"] = json.loads(encoded(row["stdout"]))
        normalized = {}
        for stream in ("stdout", "stderr"):
            content = encoded(row[stream]).replace(os.fsencode(executable.parent), b"<compiler-dir>").replace(os.fsencode(directory), b"<probe-dir>")
            if name == "version" and stream == "stdout":
                content = re.sub(rb"\(build [0-9a-f]+\)", b"(build <id>)", content)
            normalized[stream] = common.stream_record(content)
        normalized["returncode"] = row["returncode"]
        result["normalized"][name] = normalized
    result["passed"] = True


def summaries(samples: list[dict]) -> dict:
    result = {}
    for scenario in SCENARIOS:
        item = {"metric": "Cargo build wall time", "routes": {}}
        for route in ("cargo", "coordinator"):
            values = [sample["build"]["elapsed_ns"] for sample in samples if sample["scenario"] == scenario and sample["route"] == route]
            ordered = sorted(values)
            item["routes"][route] = {"samples": len(values), "median_ns": statistics.median(values), "p95_ns": ordered[math.ceil(len(ordered) * 0.95) - 1], "min_ns": ordered[0], "max_ns": ordered[-1]}
        item["baseline_over_coordinator_median"] = item["routes"]["cargo"]["median_ns"] / item["routes"]["coordinator"]["median_ns"]
        result[scenario] = item
    return result


def benchmark(args: argparse.Namespace, report: dict) -> None:
    project = Path(args.project).resolve(strict=True)
    coordinator = Path(args.coordinator)
    if not coordinator.is_absolute() or not coordinator.is_file():
        raise ProjectError("--coordinator must name an existing absolute cargo-lust executable.")
    configured = os.environ.get("LUST_CRANELIFT_BACKEND", "")
    backend = Path(configured)
    if not configured or not backend.is_absolute() or not backend.is_file():
        raise ProjectError("LUST_CRANELIFT_BACKEND must name the existing absolute unwinding-enabled backend library.")
    git = common.executable(None, "git")
    env, removed = backend_environment()
    root_row = command_record([git, "-C", str(project), "rev-parse", "--show-toplevel"], project, env, args.timeout, report["setup"], "locate Git project root")
    require(root_row, "locate project root")
    if Path(root_row["stdout"].get("text", "").strip()).resolve() != project:
        raise ProjectError("--project must be the root Lust Git checkout, not a nested crate.")
    files = tracked_files(project, git)
    source_digest = fingerprint(files)
    if not {"Cargo.toml", "Cargo.lock", "src/client.rs", "src/main.rs"}.issubset({item["path"] for item in files}):
        raise ProjectError("The tracked project is missing root Lust source or its lockfile.")
    if args.output != "-":
        output = Path(args.output).absolute()
        if output.is_relative_to(project) and str(output.relative_to(project)) in {item["path"] for item in files}:
            raise ProjectError("The report cannot overwrite a tracked project file.")
    report["source"] = {"project": str(project), "sha256": source_digest, "files": files, "scope": "tracked working files and submodule gitlink identities; no application source edits"}
    for name, command in (("commit", ["rev-parse", "HEAD"]), ("status", ["status", "--porcelain=v1", "--untracked-files=no"])):
        row = command_record([git, "-C", str(project), *command], project, env, args.timeout, report["setup"], f"record Git {name}")
        require(row, f"record Git {name}")
        report["source"][name] = row["stdout"].get("text", "").strip()
    tcc = Path(env.get("LUST_TCC_HOME", str(project / "third_party" / "tcc")))
    if not tcc.is_absolute():
        tcc = project / tcc
    tcc = tcc.resolve()
    for required in ("bin/tcc", "lib/libtcc.a", "lib/tcc/libtcc1.a"):
        if not (tcc / required).is_file():
            raise ProjectError("Build root scripts/build-tcc.sh before this benchmark; tcc is a caller prerequisite.")
    tcc_files = common.tree_files(tcc)
    tcc_digest = fingerprint(tcc_files)
    backend_digest = common.file_digest(backend)
    coordinator_digest = common.file_digest(coordinator)
    report["inputs"] = {
        "backend": {"path": str(backend), "sha256": backend_digest, "bytes": backend.stat().st_size},
        "coordinator": {"path": str(coordinator), "sha256": coordinator_digest},
        "tcc": {"path": str(tcc), "sha256": tcc_digest, "files": tcc_files},
        "probes": {name: common.file_digest(FIXTURES / name) for name in ("supported.rs", "unsupported.rs")},
        "scripts": {path.name: common.file_digest(path) for path in (Path(__file__), Path(common.__file__))},
    }
    report["environment"] = {"removed_backend_variables": removed, "preserved_build_flags": {name: env[name] for name in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_INCREMENTAL", "CARGO_PROFILE_DEV_PANIC", "CARGO_PROFILE_DEV_OPT_LEVEL", "CARGO_PROFILE_DEV_DEBUG", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER") if name in env}, "profile": "project's default dev profile; no optimization, panic, debug, feature, or source overrides"}
    tools = resolve_tools(project, env, report, args.timeout)
    candidate_env = env.copy()
    candidate_env["LUST_CRANELIFT_BACKEND"] = str(backend)

    def guard(label: str) -> None:
        unchanged = fingerprint(tracked_files(project, git)) == source_digest and fingerprint(common.tree_files(tcc)) == tcc_digest and common.file_digest(backend) == backend_digest and common.file_digest(coordinator) == coordinator_digest
        report["input_checks"].append({"stage": label, "unchanged": unchanged})
        if not unchanged:
            raise ProjectError(f"Source, tcc, compiler, or backend changed during {label}; measurements are invalid.")

    guard("before prefetch")
    fetch = command_record([tools["cargo"], "fetch", "--manifest-path", str(project / "Cargo.toml"), "--locked"], project, env, args.timeout, report["setup"], "prefetch locked dependencies outside timing")
    require(fetch, "prefetch locked dependencies")
    guard("after prefetch")
    env["CARGO_NET_OFFLINE"] = "true"
    candidate_env["CARGO_NET_OFFLINE"] = "true"
    with tempfile.TemporaryDirectory(prefix="cargo-lust-project-bench-") as directory:
        scratch = Path(directory).resolve()
        report["temporary_root"] = str(scratch)
        plan_row = command_record([str(coordinator), "--engine", "cranelift", "plan", *arguments("build", project, scratch / "plan-target", args.jobs)], project, candidate_env, args.timeout, report["setup"], "verify complementary backend selection")
        require(plan_row, "verify coordinator selection")
        plan = json.loads(encoded(plan_row["stdout"]))
        if plan.get("selected_engine") != "cranelift" or plan.get("program") != tools["cargo"] or plan.get("environment", {}).get("CARGO_PROFILE_DEV_CODEGEN_BACKEND") != str(backend):
            raise ProjectError("The coordinator plan does not select the exact baseline Cargo and configured Cranelift library.")
        report["plan"] = plan
        warm_targets = {route: scratch / ("warm-" + route) for route in ("cargo", "coordinator")}

        def pair(scenario: str, index: int, targets: dict[str, Path], warmup: bool = False) -> None:
            order = ["cargo", "coordinator"] if index % 2 == 0 else ["coordinator", "cargo"]
            check = {"scenario": scenario, "sample": index, "warmup": warmup, "order": order, "passed": False}
            report["pairs"].append(check)
            entries = {}
            for position, route in enumerate(order):
                print(f"{scenario} {'warmup' if warmup else str(index + 1) + '/' + str(args.samples)}: {route}", file=sys.stderr, flush=True)
                guard(f"before {scenario} {index} {route}")
                prefix = [tools["cargo"]] if route == "cargo" else [str(coordinator), "--engine", "cranelift"]
                selected_env = env if route == "cargo" else candidate_env
                build = common.process([*prefix, *arguments("build", project, targets[route], args.jobs)], project, selected_env, args.timeout)
                entry = {"scenario": scenario, "sample": index, "warmup": warmup, "route": route, "order_in_pair": position, "build": build, "runtime": None, "artifacts": {}}
                report["warmups" if warmup else "samples"].append(entry)
                checkpoint(report, args.output)
                require(build, f"{route} root Lust build")
                entry["artifacts"] = artifacts(build, targets[route], scenario == "warm_noop" and not warmup)
                guard(f"after {scenario} {index} {route} build")
                entry["runtime"] = {"commands": {}, "normalized": {}, "passed": False}
                runtime_probe(Path(entry["artifacts"]["lust"]["path"]), scratch / "runtime-probes", selected_env, args.timeout, entry["runtime"])
                entries[route] = entry
                guard(f"after {scenario} {index} {route} runtime")
                checkpoint(report, args.output)
            left, right = entries["cargo"]["runtime"], entries["coordinator"]["runtime"]
            if left["normalized"] != right["normalized"] or left["capabilities"] != right["capabilities"]:
                raise ProjectError("The two built Lust compilers differ in probe status, diagnostics, capabilities, or program output.")
            check["passed"] = True
            checkpoint(report, args.output)

        for index in range(args.samples):
            # Bound disk usage even when debug compiler executables are large.
            with tempfile.TemporaryDirectory(prefix=f"cold-{index}-", dir=scratch) as cold:
                pair("cold", index, {route: Path(cold) / route for route in ("cargo", "coordinator")})
        pair("warm_noop", 0, warm_targets, warmup=True)
        for index in range(args.samples):
            pair("warm_noop", index, warm_targets)

        # This qualification is deliberately outside compilation statistics.
        # Both commands compile and execute the root compiler's real bin tests.
        for route in ("cargo", "coordinator"):
            guard(f"before {route} unit suite")
            prefix = [tools["cargo"]] if route == "cargo" else [str(coordinator), "--engine", "cranelift"]
            row = command_record([*prefix, *arguments("test", project, warm_targets[route], args.jobs, messages=False)], project, env if route == "cargo" else candidate_env, args.timeout, report["test_qualification"], f"{route} actual root cargo test --bins")
            checkpoint(report, args.output)
            require(row, f"{route} root Lust unit suite")
            row["summaries"] = [list(map(int, match)) for match in TEST_SUMMARY.findall(row["stdout"].get("text", ""))]
            if not row["summaries"]:
                raise ProjectError("The root bin test command returned no passing Rust test summaries.")
            guard(f"after {route} unit suite")
        if report["test_qualification"][0]["summaries"] != report["test_qualification"][1]["summaries"]:
            raise ProjectError("The two compiler routes executed different root test counts.")
    guard("final verification")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--coordinator", required=True, help="Absolute path to the built cargo-lust executable.")
    parser.add_argument("--project", required=True, help="Unchanged root Lust Git checkout.")
    parser.add_argument("--samples", type=int, default=2, help="Even measured pairs per scenario; default 2 for an initial low-sample run.")
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--timeout", type=float, default=900.0, help="Maximum seconds per command, including an entire test suite.")
    parser.add_argument("--output", required=True, help="JSON evidence file; use - for one final report on stdout.")
    args = parser.parse_args()
    if args.samples < 2 or args.samples % 2 or args.jobs < 1 or not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error("samples must be positive and even with a minimum of 2; jobs and timeout must be positive")
    try:
        validate_output(args)
    except (common.BenchmarkError, OSError, ValueError) as error:
        parser.error(str(error))
    report = {
        "schema_version": 1, "created_utc": dt.datetime.now(dt.timezone.utc).isoformat(), "workload": "unchanged root Lust compiler", "hardware": common.hardware(),
        "settings": {"samples_per_scenario": args.samples, "jobs": args.jobs, "low_sample": args.samples < 6, "scenarios": list(SCENARIOS)},
        "method": {"timed_operation": "cargo build --bins, default dev profile", "prefetch": "locked dependencies before timing; subsequent Cargo commands offline", "cold": "empty per-route target directories; installed compiler, dependency sources, and OS caches remain warm", "warm": "one excluded pair, then unchanged builds with retained targets", "interleaving": "alternate first route in each measured pair", "runtime": "outside build timing; fresh identical-path probe directory and disabled Lust daemon/cache", "normalization": "only compiler/probe directory prefixes and version build ID; raw output is retained", "test_suite": "one untimed cargo test --bins per route, with retained targets; require both success and equal test counts", "p95": "nearest-rank; with 2 samples, p95 is the maximum", "limits": "one real compiler project and limited probes; no broad conformance or production performance claim"},
        "setup": [], "input_checks": [], "warmups": [], "samples": [], "pairs": [], "test_qualification": [], "errors": [], "statistics": {}, "passed": False,
    }
    status = 0
    try:
        benchmark(args, report)
        report["statistics"] = summaries(report["samples"])
        report["passed"] = True
    except KeyboardInterrupt:
        report["errors"].append("Interrupted; completed observations retained.")
        status = 130
    except (common.BenchmarkError, OSError, ValueError, subprocess.CalledProcessError) as error:
        report["errors"].append(str(error))
        print(f"real-project benchmark failed: {error}", file=sys.stderr)
        status = 1
    report["finished_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    # A failed build, runtime probe, source check, or unit suite yields no ratios.
    if not report["passed"]:
        report["statistics"] = {}
    common.write_report(report, args.output)
    for scenario, result in report["statistics"].items():
        left, right = (result["routes"][route]["median_ns"] / 1e9 for route in ("cargo", "coordinator"))
        print(f"{scenario}: Cargo {left:.3f}s, coordinator {right:.3f}s, median ratio {result['baseline_over_coordinator_median']:.3f}x", file=sys.stderr)
    return status


if __name__ == "__main__":
    raise SystemExit(main())
