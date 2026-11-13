#!/usr/bin/env python3
"""Compare two native Lust builds on the inherited realish benchmark.

The clients and their real daemons run directly. Every source state is checked
against rustc and cold native checks; built programs must agree. Raw interleaved
samples are retained. An inconclusive zero-regression comparison does not pass.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import random
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import time

import speed_slice


ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_identity(path):
    if not (path / "library/core/src/lib.rs").is_file() or not (path / "tcc/lib/tcc/libtcc1.a").is_file():
        raise ValueError("not a complete install-relative native runtime: " + str(path))
    files = {str(p.relative_to(path)): sha(p) for p in sorted(path.rglob("*")) if p.is_file()}
    return {"path": str(path), "files": files,
            "sha256": hashlib.sha256(json.dumps(files, sort_keys=True).encode()).hexdigest()}


def decide(samples):
    if len(samples) < 12 or len(samples) % 2:
        return {"state": "unproved", "reason": "at least twelve balanced pairs are required"}
    ratios = [row["candidate_ns"] / row["upstream_ns"] for row in samples]
    logs = [math.log(ratio) for ratio in ratios]
    randomizer = random.Random(73401)
    draws = sorted(math.exp(sum(randomizer.choices(logs, k=len(logs))) / len(logs)) for _ in range(10000))
    baseline = [row["upstream_ns"] for row in samples]
    candidate = [row["candidate_ns"] for row in samples]
    median_ratio = statistics.median(candidate) / statistics.median(baseline)
    tail_ratio = speed_slice.pct(candidate, 95) / speed_slice.pct(baseline, 95)
    upper = draws[9499]
    passed = upper <= 1.0 and median_ratio <= 1.0 and tail_ratio <= 1.0
    return {"state": "passed" if passed else "unproved" if draws[499] <= 1.0 <= upper else "regressed",
            "pairs": len(samples), "median_ratio": median_ratio, "p95_ratio": tail_ratio,
            "paired_geometric_ratio": math.exp(statistics.mean(logs)),
            "paired_ratio_upper_95": upper,
            "baseline_median_ns": statistics.median(baseline),
            "candidate_median_ns": statistics.median(candidate),
            "threshold": 1.0, "regression_allowance": 0.0}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--upstream", type=Path, required=True)
    parser.add_argument("--candidate", type=Path, required=True)
    parser.add_argument("--rustc", type=Path, required=True)
    parser.add_argument("--linker", type=Path, help="explicit linker for the independent Rust oracle")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--pairs", type=int, default=24)
    parser.add_argument("--modules", type=int, default=8)
    parser.add_argument("--enforce", action="store_true")
    args = parser.parse_args()
    if args.pairs < 12 or args.pairs % 2 or args.modules < 1:
        parser.error("use at least twelve even pairs and a positive module count")
    compilers = {name: path.resolve() for name, path in (("upstream", args.upstream), ("candidate", args.candidate))}
    runtimes = {name: binary.parent.parent / "lib/lust" for name, binary in compilers.items()}
    oracle, output = args.rustc.resolve(), args.output.resolve()
    if output.exists() or len(set(compilers.values()) | {oracle}) != 3:
        parser.error("use a new output directory and distinct compiler paths")
    for binary in [*compilers.values(), oracle, *(p.with_name("lust-core") for p in compilers.values())]:
        if not binary.is_file() or not os.access(binary, os.X_OK):
            parser.error("missing compiler executable: " + str(binary))
    if args.linker is not None and (not args.linker.is_file() or not os.access(args.linker, os.X_OK)):
        parser.error("--linker must name an executable")
    output.mkdir(parents=True)
    source = output / "main.rs"
    report = {"schema": "lust.native-comparison.v1", "qualified": False, "correct": False,
              "scope": "The recorded native clients, source states and host. No universal speed or full-Rust claim.",
              "host": {"system": platform.system(), "architecture": platform.machine()},
              "compiler_build_requirement": "Both compiler executables must be built with the same upstream release profile.",
              "tools": {}, "commands": [], "series": {}, "failures": [], "daemons": {},
              "generator": {"file": "bench/realish/make.py", "sha256": sha(ROOT / "bench/realish/make.py"),
                            "input_sha256": sha(ROOT / "bench/realish/warehouse.rs"), "modules": args.modules},
              "runner_sha256": sha(Path(__file__)), "samples": args.pairs}
    base_env = {key: value for key, value in os.environ.items()
                if not key.startswith(("LUST_", "RUST", "CARGO_", "LD_", "DYLD_"))}
    base_env["CARGO_NET_OFFLINE"] = "true"
    daemons, daemon_logs, environments, visit_logs = {}, [], {}, {}
    socket_directory = Path(tempfile.mkdtemp(prefix="lust-compare-", dir="/tmp"))
    counts = {"upstream": 0, "candidate": 0}
    expected = {}

    def save():
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")

    def execute(label, argv, env, timeout=60):
        command = [str(value) for value in argv]
        start = time.perf_counter_ns()
        process = subprocess.Popen(command, cwd=output, env=env, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, start_new_session=True)
        timed_out = False
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            stdout, stderr = process.communicate()
        row = {"label": label, "argv": command, "elapsed_ns": time.perf_counter_ns() - start,
               "exit_code": process.returncode, "stdout": stdout.decode(errors="replace"),
               "stderr": stderr.decode(errors="replace"), "timed_out": timed_out}
        report["commands"].append(row)
        if timed_out:
            raise RuntimeError(label + ": timed out")
        return row

    def check_oracle(state, valid):
        identity = sha(source)
        if identity in expected:
            return expected[identity]
        binary = output / ("oracle-" + identity[:16])
        command = [oracle, "--edition=2024", source, "-o", binary]
        if args.linker is not None:
            command.extend(["-C", "linker=" + str(args.linker.absolute())])
        row = execute(state + "-rustc", command, base_env, timeout=180)
        if (row["exit_code"] == 0) != valid or (not valid and row["exit_code"] != 1):
            raise RuntimeError(state + ": oracle disagrees with the fixture's declared validity")
        result = None
        if valid:
            runtime = execute(state + "-rustc-run", [binary], base_env)
            if runtime["exit_code"] != 0:
                raise RuntimeError(state + ": oracle program failed")
            result = (runtime["exit_code"], runtime["stdout"], runtime["stderr"])
        expected[identity] = result
        return result

    def compile_one(lane, verb, label, valid=True, cold=False):
        env = dict(environments[lane])
        visits_before = 0
        if not cold:
            if daemons[lane].poll() is not None or not Path(report["daemons"][lane]["socket"]).is_socket():
                raise RuntimeError(label + ": " + lane + " daemon is not available")
            visits_before = visit_logs[lane].stat().st_size if visit_logs[lane].exists() else 0
        if cold:
            counts[lane] += 1
            directory = output / lane / ("cold-" + str(counts[lane]))
            directory.mkdir()
            env.update(LUST_NO_DAEMON="1", LUST_NO_CACHE="1", TMPDIR=str(directory),
                       LUST_CACHE_DIR=str(directory / "cache"))
        binary = output / (lane + "-program")
        command = [compilers[lane], verb, "--timings", source]
        if verb == "build":
            command.extend(["-o", binary])
        row = execute(label + "-" + lane, command, env)
        if (row["exit_code"] == 0) != valid or (not valid and row["exit_code"] != 1):
            raise RuntimeError(label + ": " + lane + " returned the wrong acceptance result")
        if not valid and not row["stderr"].strip():
            raise RuntimeError(label + ": " + lane + " rejected without a diagnostic")
        row["phases_ms"] = speed_slice.parse_timings(row["stderr"])
        row["visits"] = speed_slice.parse_visits(row["stderr"])
        if not cold:
            visits_after = visit_logs[lane].stat().st_size if visit_logs[lane].exists() else 0
            row["daemon_visits_bytes"] = {"before": visits_before, "after": visits_after}
            if visits_after <= visits_before:
                raise RuntimeError(label + ": " + lane + " request produced no daemon visit record")
        if verb == "build" and valid:
            row["artifact_sha256"] = sha(binary)
            runtime = execute(label + "-" + lane + "-run", [binary], env)
            if (runtime["exit_code"], runtime["stdout"], runtime["stderr"]) != expected[sha(source)]:
                raise RuntimeError(label + ": " + lane + " runtime differs from rustc")
            row["compile_plus_run_ns"] = row["elapsed_ns"] + runtime["elapsed_ns"]
        if cold:
            shutil.rmtree(directory)
        return row

    def pair(label, verb, index, valid=True, cold=False):
        order = ("upstream", "candidate") if index % 2 == 0 else ("candidate", "upstream")
        rows = {lane: compile_one(lane, verb, label, valid, cold) for lane in order}
        item = {"pair": index, "order": list(order), "source_sha256": sha(source),
                **{lane + "_ns": rows[lane]["elapsed_ns"] for lane in rows}}
        report["series"].setdefault(label, {"samples": []})["samples"].append(item)
        if verb == "build" and valid:
            combined = {**item, **{lane + "_ns": rows[lane]["compile_plus_run_ns"] for lane in rows}}
            report["series"].setdefault(label + "+run", {"samples": []})["samples"].append(combined)
        if not cold:
            for lane in order:
                control = compile_one(lane, "check", label + "-cold-control", valid, cold=True)
                if control["exit_code"] != rows[lane]["exit_code"]:
                    raise RuntimeError(label + ": warm and cold native results differ")
        save()

    try:
        report["runtime_inputs"] = {name: tree_identity(path) for name, path in runtimes.items()}
        if args.linker is not None:
            report["oracle_linker"] = {"path": str(args.linker.absolute()), "sha256": sha(args.linker)}
        generated = subprocess.check_output([sys.executable, ROOT / "bench/realish/make.py", str(args.modules)], text=True)
        anchor = "fn main() {\n"
        if generated.count(anchor) != 1:
            raise ValueError("benchmark main anchor changed")
        probe = ("struct Probe { value: i64 }\n"
                 "fn probe(value: i64) -> i64 { value }\n"
                 "macro_rules! extra { () => { 0i64 }; }\n")
        base = probe + generated.replace("    let mut total: i64 = 0;", "    let mut total: i64 = probe(Probe { value: 1 }.value) + extra!();", 1)
        states = {
            "body": base.replace("-> i64 { value }", "-> i64 { value + 1 }", 1),
            "signature": base.replace("fn probe(value: i64)", "fn probe(value: i64, extra: i64)", 1)
                             .replace("-> i64 { value }", "-> i64 { value + extra }", 1)
                             .replace("probe(Probe { value: 1 }.value)", "probe(Probe { value: 1 }.value, 1)", 1),
            "type": base.replace("struct Probe { value: i64 }", "struct Probe { value: i64, other: u64 }", 1)
                        .replace("Probe { value: 1 }", "Probe { value: 1, other: 2 }", 1),
            "macro": base.replace("() => { 0i64 }", "() => { 2i64 }", 1),
            "invalid": base.replace("-> i64 { value }", '-> i64 { "wrong type" }', 1),
            "recovery": base,
        }
        if len(set(states.values()) | {base}) != 6:
            raise ValueError("benchmark edit anchors changed")
        report["source_states"] = {}
        for state, text in {"base": base, **states}.items():
            path = output / ("source-" + state + ".rs")
            path.write_text(text)
            report["source_states"][state] = {"file": path.name, "sha256": sha(path)}
        for name, binary in {**compilers, "rustc": oracle}.items():
            version = execute(name + "-identity", [binary, "--version"], base_env)
            report["tools"][name] = {"path": str(binary), "sha256": sha(binary), "version": version["stdout"].strip()}
            prefix = ("rustc ",) if name == "rustc" else ("lust ", "lustr ")
            if version["exit_code"] != 0 or not version["stdout"].startswith(prefix):
                raise RuntimeError(name + ": compiler identity command failed")
            if name in compilers:
                report["tools"][name]["core_sha256"] = sha(binary.with_name("lust-core"))
        for lane, binary in compilers.items():
            directory = output / lane
            directory.mkdir()
            temp = directory / "tmp"
            temp.mkdir()
            socket = socket_directory / (lane + ".sock")
            env = dict(base_env, LUST_DAEMON_SOCKET=str(socket),
                       LUST_DAEMON_IDLE="3600", LUST_CACHE_DIR=str(directory / "cache"),
                       LUST_STATS="1", TMPDIR=str(temp))
            environments[lane] = env
            visit_logs[lane] = directory / "daemon-visits.log"
            daemon_env = dict(env, LUST_VISITS_LOG=str(visit_logs[lane]))
            log = (directory / "daemon.log").open("wb")
            daemon_logs.append(log)
            process = subprocess.Popen([binary, "daemon"], cwd=output, env=daemon_env,
                                       stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            daemons[lane] = process
            report["daemons"][lane] = {"pid": process.pid, "socket": str(socket), "established": False}
            deadline = time.monotonic() + 10
            while not socket.is_socket():
                if process.poll() is not None or time.monotonic() > deadline:
                    detail = (directory / "daemon.log").read_text(errors="replace")[-2000:]
                    raise RuntimeError(lane + ": native daemon failed to establish a socket: " + detail)
                time.sleep(0.01)
            report["daemons"][lane]["established"] = True
        source.write_text(base)
        check_oracle("base", True)
        for lane in compilers:
            for verb in ("check", "build"):
                compile_one(lane, verb, "warmup")
            compile_one(lane, "check", "retained-check-first")
            retained = compile_one(lane, "check", "retained-check-repeated")
            if retained["visits"].get("path") != "reuse":
                raise RuntimeError(lane + ": repeated check did not reuse a retained session")
            report["daemons"][lane]["retained_check_reused"] = True
        for index in range(args.pairs):
            source.write_text(base)
            for verb in ("check", "build"):
                pair("cold-" + verb, verb, index, cold=True)
                pair("noop-" + verb, verb, index)
            for state, text in states.items():
                source.write_text(text)
                valid = state != "invalid"
                check_oracle(state, valid)
                pair(state + "-check", "check", index, valid)
                if valid:
                    pair(state + "-build", "build", index)
        for lane, binary in compilers.items():
            if sha(binary) != report["tools"][lane]["sha256"] or sha(binary.with_name("lust-core")) != report["tools"][lane]["core_sha256"]:
                raise RuntimeError(lane + ": compiler executable changed during measurement")
        if sha(oracle) != report["tools"]["rustc"]["sha256"]:
            raise RuntimeError("oracle executable changed during measurement")
        if any(tree_identity(path) != report["runtime_inputs"][name] for name, path in runtimes.items()):
            raise RuntimeError("native runtime input changed during measurement")
        if args.linker is not None and sha(args.linker) != report["oracle_linker"]["sha256"]:
            raise RuntimeError("oracle linker changed during measurement")
        report["correct"] = True
        for series in report["series"].values():
            series["decision"] = decide(series["samples"])
        report["qualified"] = all(row["decision"]["state"] == "passed" for row in report["series"].values())
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as error:
        report["failures"].append(str(error))
    finally:
        for process in daemons.values():
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGTERM)
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL)
                    process.wait()
        for stream in daemon_logs:
            stream.close()
        shutil.rmtree(socket_directory, ignore_errors=True)
        save()
    print(json.dumps({"correct": report["correct"], "qualified": report["qualified"],
                      "failures": report["failures"], "report": str(output / "report.json")}, indent=2))
    return 0 if report["correct"] and (report["qualified"] or not args.enforce) else 1


if __name__ == "__main__":
    raise SystemExit(main())
