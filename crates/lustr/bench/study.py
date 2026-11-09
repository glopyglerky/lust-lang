#!/usr/bin/env python3
"""Controlled BlueFe experiments. No toolchain installs or source edits.

subset compares explicit whole-target Lust with LLVM on the real log analyzer.
frontend compares one versus N frontend jobs on unchanged lust-core metadata.
Neither experiment authorizes automatic native routing or proves Rust semantics.
"""
import argparse
import datetime as dt
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import statistics
import sys
import tempfile

import measure as common
from paired import paired_summary


def summary(rows):
    results = {}
    for scenario in sorted({row["scenario"] for row in rows}):
        pairs = {}
        routes = {}
        for row in rows:
            if row["scenario"] != scenario:
                continue
            value = row["build"]["elapsed_ns"]
            pairs.setdefault(row["sample"], {})[row["route"]] = value
            routes.setdefault(row["route"], []).append(value)
        results[scenario] = {
            "routes": {route: {"samples": len(values), "median_ns": statistics.median(values), "p95_ns": sorted(values)[math.ceil(.95 * len(values)) - 1]} for route, values in routes.items()},
            "paired": paired_summary([(pair["baseline"], pair["candidate"]) for _, pair in sorted(pairs.items())]),
        }
    return results


def input_hash(project):
    files = [project / name for name in ("Cargo.toml", "Cargo.lock", "build.rs")]
    for directory in ("src", "library"):
        files.extend(path for path in (project / directory).rglob("*") if path.is_file())
    records = [(str(path.relative_to(project)), common.file_digest(path)) for path in sorted(files)]
    return hashlib.sha256(json.dumps(records, separators=(",", ":")).encode()).hexdigest()


def study(args, report):
    project = Path(args.project).resolve(strict=True)
    toolchain = Path(args.bluefe_root)
    if not toolchain.is_absolute():
        raise common.BenchmarkError("--bluefe-root must be absolute")
    tools = {name: common.executable(str(toolchain / "bin" / name), name) for name in ("cargo", "rustc", "rustdoc")}
    coordinator = common.executable(args.coordinator, "lustr")
    compiler_inputs = input_hash(project)
    with tempfile.TemporaryDirectory(prefix="lustr-study-") as temporary:
        work = Path(temporary)
        env = os.environ.copy()
        # All deliberate build settings apply identically to both routes.
        removed = []
        for name in list(env):
            if name.startswith(("CARGO_PROFILE_", "CARGO_TARGET_", "LUST_")) or name in {"RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTC_BOOTSTRAP", "CARGO_BUILD_TARGET", "CARGO_MAKEFLAGS", "MAKEFLAGS", "MFLAGS"}:
                removed.append(name)
                del env[name]
        # TCC is an explicit compiler input, never an inferred replacement.
        if "LUST_TCC_HOME" in os.environ:
            env["LUST_TCC_HOME"] = os.environ["LUST_TCC_HOME"]
        env.update({"CARGO": tools["cargo"], "LUST_CARGO": tools["cargo"], "RUSTC": tools["rustc"], "RUSTDOC": tools["rustdoc"], "CARGO_NET_OFFLINE": "true", "CARGO_TERM_COLOR": "never", "CARGO_INCREMENTAL": "0", "LUST_NO_DAEMON": "1", "LUST_NO_CACHE": "1", "LUST_CACHE_DIR": str(work / "cache"), "LUST_DAEMON_SOCKET": str(work / "absent.sock")})
        report["environment"] = {"removed_keys": sorted(removed), "overrides": {name: env[name] for name in ("CARGO_INCREMENTAL", "LUST_NO_DAEMON", "LUST_NO_CACHE")}, "tcc_home": env.get("LUST_TCC_HOME"), "cargo_home": env.get("CARGO_HOME")}
        report["identities"] = common.identify(tools["cargo"], tools["rustc"], coordinator, env, work, args)
        report["identities"]["binaries"] = {name: {"path": path, "sha256": common.file_digest(Path(path))} for name, path in tools.items()}
        reference = common.executable(args.reference_rustc, "rustc") if args.reference_rustc else tools["rustc"]
        report["identities"]["reference_rustc"] = {"path": reference, "sha256": common.file_digest(Path(reference)), "version": common.process([reference, "-vV"], work, env, args.timeout)}
        report["source"] = {"compiler_inputs_sha256": compiler_inputs, "git_head": common.process(["git", "-C", str(project), "rev-parse", "HEAD"], work, env, args.timeout)}
        report["scripts"] = {path.name: common.file_digest(path) for path in (Path(__file__), Path(common.__file__), Path(__file__).with_name("paired.py"))}

        def run(argv, purpose, selected_env=None):
            row = common.process(argv, work, env if selected_env is None else selected_env, args.timeout)
            row["purpose"] = purpose
            return row

        def guard():
            if input_hash(project) != compiler_inputs:
                raise common.BenchmarkError("compiler inputs changed during measurement")

        def checkpoint():
            common.write_report(report, args.output)

        if args.kind == "subset":
            lust = common.executable(args.lust, "lust")
            env["LUSTC"] = lust
            report["identities"]["lust"] = {"path": lust, "sha256": common.file_digest(Path(lust)), "version": run([lust, "--version"], "identify native compiler")}
            source = work / "main.rs"
            shutil.copyfile(project / "bench/real/main.rs", source)
            report["workload"] = {"name": "upstream real log analyzer", "source_sha256": common.file_digest(source), "lines": len(source.read_text().splitlines()), "panic": "abort", "overflow_checks": True, "edition": "2024", "debug": args.debug, "optimization": 0, "cache": "disabled for both compilers", "limits": "one qualified subset program; no general Rust, daemon, incremental, or Cargo-library claim"}
            baseline_flags = ["--edition=2024", "-Cpanic=abort", "-Coverflow-checks=on", f"-Cdebuginfo={args.debug}", "-Copt-level=0"]
            scenarios = ("check", "build", "compile_and_run")

            def operation(route, scenario, index):
                output = work / f"program-{route}"
                if route == "baseline":
                    argv = [reference, *baseline_flags, str(source), "-o", str(output)]
                    if scenario == "check":
                        argv.extend(["--emit=metadata"])
                else:
                    argv = [coordinator, "--engine", "lust", "check" if scenario == "check" else "build", str(source)]
                    if scenario != "check":
                        argv.extend(["-o", str(output), *(["-g"] if args.debug == 2 else [])])
                build = run(argv, f"{route} {scenario}")
                common.require_success(build, build["purpose"])
                row = {"scenario": scenario, "sample": index, "route": route, "build": build}
                if scenario != "check":
                    runtime = run([str(output)], "execute the built log analyzer")
                    common.require_success(runtime, runtime["purpose"])
                    row["runtime"] = runtime
                    row["artifact"] = {"bytes": output.stat().st_size, "sha256": common.file_digest(output)}
                    if scenario == "compile_and_run":
                        build["compile_only_ns"] = build["elapsed_ns"]
                        build["elapsed_ns"] += runtime["elapsed_ns"]
                return row
        else:
            if args.frontend_jobs < 2:
                raise common.BenchmarkError("frontend candidate needs at least two jobs")
            report["workload"] = {"name": "unchanged lust-core full Rust metadata check", "frontend_jobs": ["reference compiler default" if args.reference_rustc else 1, args.frontend_jobs], "cargo_jobs": args.cargo_jobs, "incremental": False, "profile": "unchanged dev", "limits": "root metadata only; dependencies primed outside timing; native code generation, linking and tests excluded; an optional reference compiler changes the compiler revision and cannot isolate a frontend-setting effect"}
            scenarios = ("root_metadata",)
            targets = {route: work / route for route in ("baseline", "candidate")}

            def operation(route, scenario, index):
                # Dependencies stay cached; remove only this experiment's root
                # package output. This forces actual root compilation each time.
                cleanup = run([tools["cargo"], "clean", "--manifest-path", str(project / "Cargo.toml"), "--target-dir", str(targets[route]), "-p", "lustr-native"], "remove owned root output outside timing")
                common.require_success(cleanup, cleanup["purpose"])
                prefix = [tools["cargo"]] if route == "baseline" else [coordinator, "--bluefe-root", str(toolchain)]
                jobs = 1 if route == "baseline" else args.frontend_jobs
                argv = [*prefix, "check", "--manifest-path", str(project / "Cargo.toml"), "--bin", "lust-core", "--target-dir", str(targets[route]), "--offline", "--locked", f"-j{args.cargo_jobs}", "-vv"]
                selected_env = env.copy()
                selected_env["RUSTC"] = reference if route == "baseline" else tools["rustc"]
                flags = "" if route == "baseline" and args.reference_rustc else f"-Zunstable-options\x1f--jobs-frontend={jobs}"
                selected_env["CARGO_ENCODED_RUSTFLAGS"] = flags
                build = run(argv, f"{route} fresh root metadata", selected_env)
                build["environment_overrides"] = {name: selected_env[name] for name in ("RUSTC", "CARGO_ENCODED_RUSTFLAGS")}
                common.require_success(build, build["purpose"])
                commands = list(common.cargo_verbose_commands(build["stderr"]))
                expected = f"--jobs-frontend={jobs}"
                verified = [tokens for _, tokens in commands if "--crate-name" in tokens and "lust_core" in tokens and (expected in tokens if flags else not any(token.startswith(("--jobs-frontend", "-Zthreads")) for token in tokens)) and selected_env["RUSTC"] in tokens and "--emit=dep-info,metadata" in tokens and not any("link" in token for token in tokens if token.startswith("--emit="))]
                if len(verified) != 1:
                    raise common.BenchmarkError("expected one fresh root compiler command with exact frontend selection")
                return {"scenario": scenario, "sample": index, "route": route, "build": build, "preparation": cleanup, "compiler_verified": True}

        # One untimed round per scenario, then balanced measured pairs.
        for scenario in scenarios:
            for index in range(-1, args.samples):
                guard()
                order = ["baseline", "candidate"] if index % 2 == 0 else ["candidate", "baseline"]
                pair = []
                for route in order:
                    print(f"{scenario} {index + 1}/{args.samples} {route}", file=sys.stderr, flush=True)
                    row = operation(route, scenario, index)
                    pair.append(row)
                    report["warmups" if index < 0 else "samples"].append(row)
                    checkpoint()
                if args.kind == "subset" and scenario != "check" and pair[0]["runtime"]["stdout"] != pair[1]["runtime"]["stdout"]:
                    raise common.BenchmarkError("native program outputs differ; no speed ratio is valid")
                guard()
        report["passed"] = True
        report["statistics"] = summary(report["samples"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kind", choices=("subset", "frontend"), required=True)
    parser.add_argument("--project", required=True)
    parser.add_argument("--bluefe-root", required=True)
    parser.add_argument("--coordinator", required=True)
    parser.add_argument("--lust", help="Release-built native Lust client, required for subset")
    parser.add_argument("--reference-rustc", help="Optional explicit stock baseline rustc; record version differences separately from coordinator effects")
    parser.add_argument("--frontend-jobs", type=int, default=4)
    parser.add_argument("--cargo-jobs", type=int, default=4)
    parser.add_argument("--debug", type=int, choices=(0, 2), default=0)
    parser.add_argument("--samples", type=int, default=10)
    parser.add_argument("--timeout", type=float, default=900)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    if args.samples < 2 or args.samples % 2 or not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error("samples must be even and at least two; timeout must be positive")
    if args.kind == "subset" and not args.lust:
        parser.error("subset requires --lust")
    if args.cargo_jobs < 1:
        parser.error("cargo jobs must be positive")
    output = Path(args.output).absolute()
    if output.exists() or output.is_relative_to(Path(args.project).resolve()):
        parser.error("write a new report outside the source checkout")
    report = {"format_version": 1, "created_utc": dt.datetime.now(dt.timezone.utc).isoformat(), "hardware": common.hardware(), "method": {"pairs": args.samples, "order": "alternating AB/BA", "warmups": "one excluded pair per scenario", "statistics": "paired log ratio, seeded percentile bootstrap, exploratory exact two-sided sign-flip test; no multiple-comparison adjustment", "cold": "compiler processes and requested output; installed tools and OS page cache warm"}, "samples": [], "warmups": [], "statistics": {}, "errors": [], "passed": False}
    try:
        study(args, report)
    except (common.BenchmarkError, OSError, ValueError, KeyboardInterrupt) as error:
        report["errors"].append(str(error) or "interrupted")
        report["passed"] = False
        report["statistics"] = {}
    common.write_report(report, args.output)
    print(json.dumps({"passed": report["passed"], "errors": report["errors"], "statistics": report["statistics"]}), file=sys.stderr)
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
