#!/usr/bin/env python3
"""Fail-closed, finite-workload native compiler acceptance and speed gate.

This is an external qualification program. Nothing here is installed in the
compiler or its Cargo execution path. The actual candidate runs directly, with
stock Cargo owning scheduling. Rustc is a separately executed reference only.
"""
from __future__ import annotations

import argparse
import ast
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import random
import re
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import time


SCHEMA = "lustr.native-speed-qualification.v1"
MIN_PAIRS = 12
DIRECT_MODES = ("process-cold", "fresh-cache", "warm-noop", "body-edit", "signature-edit", "type-edit", "macro-edit", "cold-invalid")
COLD_DIRECT_MODES = ("process-cold", "fresh-cache", "cold-invalid")
CARGO_MODES = ("fresh-target", "no-change", "body-edit", "signature-edit", "type-edit", "macro-edit", "cold-invalid")
MIXED_MODES = ("fresh-target", "no-change", "body-edit", "signature-edit", "type-edit", "macro-edit")
FULL_MODES = ("fresh-target", "no-change", "check", "release", "compile-plus-test", "doctests", "type-edit", "macro-edit")
COMPILER_NAMES = {"rustc", "rustdoc", "rustup", "lust", "lust-core", "lustr", "lustr-native", "cargo-lustr", "cargo-lust"}


class InfrastructureUnavailable(RuntimeError):
    """Evidence collection failed before the compiler could be evaluated."""


def require(value: bool, message: str) -> None:
    if not value:
        raise ValueError(message)


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def tree_files(root: Path) -> dict[str, str]:
    return {path.relative_to(root).as_posix(): digest(path)
            for path in sorted(root.rglob("*")) if path.is_file()
            and not {"target", ".lust-cache", ".git"}.intersection(path.relative_to(root).parts)}


def load_helper(repository: Path, name: str):
    path = repository / "tools" / name
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), path)
    require(spec is not None and spec.loader is not None, "cannot load " + str(path))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def paired_decision(samples: list[dict], minimum: int = MIN_PAIRS, *, fresh_cache: bool = False) -> dict:
    """No positive regression budget, no discarded outliers, no pooled modes.

    Paired bootstrap uncertainty is conditional on representative independent
    pairs. It is not a claim about arbitrary code or all future executions.
    Every observed slower pair blocks acceptance even if the mean is faster.
    """
    result = {"state": "unproved", "qualified": False, "pairs": len(samples),
              "regression_budget": 0.0, "confidence": 0.95,
              "method": "all observed pairs non-regressing plus one-sided paired log-mean bootstrap",
              "assumptions": "Pairs represent this host/workload; shared-host dependence can invalidate statistical inference."}
    try:
        require(len(samples) >= minimum, f"need at least {minimum} complete pairs")
        require([row["sample"] for row in samples] == list(range(len(samples))), "missing or duplicate sample indices")
        cold_paths, compile_cache_paths = set(), set()
        for row in samples:
            require(row.get("correct") is True, "timed result failed correctness or engine verification")
            require(row.get("source_equal") is True, "paired semantic inputs differ")
            if fresh_cache:
                require(row.get("cold") is True, "fresh-cache row is not a cold compilation")
            if row.get("cold") is True:
                caches = [row.get("baseline_runtime_temp", {}), row.get("candidate_runtime_temp", {})]
                require(all(isinstance(cache, dict) for cache in caches), "cold row has no runtime-cache evidence")
                require(all(cache.get("fresh") is True and cache.get("empty_before_compile") is True for cache in caches),
                        "cold row lacks fresh empty runtime-cache evidence")
                require(caches[0].get("path") and caches[1].get("path") and caches[0]["path"] != caches[1]["path"],
                        "cold lanes shared a runtime temp directory")
                require(row.get("baseline_tmpdir") == caches[0]["path"] and row.get("candidate_tmpdir") == caches[1]["path"],
                        "recorded compiler TMPDIR differs from cold-cache evidence")
                for cache in caches:
                    require(cache["path"] not in cold_paths, "cold samples reused a runtime temp directory")
                    cold_paths.add(cache["path"])
            if fresh_cache:
                caches = [row.get(lane + "_compile_cache") for lane in ("baseline", "candidate")]
                require(all(isinstance(cache, dict) for cache in caches), "fresh-cache row has no compilation-cache evidence")
                for lane, cache in zip(("baseline", "candidate"), caches):
                    require(cache.get("fresh") is True and cache.get("empty_before_compile") is True
                            and cache.get("enabled") is True, "fresh-cache row lacks an enabled, new empty compilation cache")
                    path = cache.get("path")
                    require(isinstance(path, str) and path and path not in compile_cache_paths,
                            "fresh-cache lanes or samples reused a compilation cache")
                    compile_cache_paths.add(path)
                    environment = row.get(lane + "_cache_env")
                    require(isinstance(environment, dict) and all(name in environment for name in
                            ("LUST_CACHE_DIR", "LUST_NO_CACHE", "LUST_NO_DAEMON")), "fresh-cache compiler environment is missing")
                    require(environment["LUST_CACHE_DIR"] == path, "compiler cache directory differs from fresh-cache evidence")
                    require(environment["LUST_NO_CACHE"] in (None, "", "0"), "fresh-cache compilation disabled caching")
                    require(environment["LUST_NO_DAEMON"] == "1", "fresh-cache compilation allowed daemon reuse")
            for key in ("baseline_ns", "candidate_ns"):
                require(type(row.get(key)) is int and row[key] > 0, "timings must be positive integer nanoseconds")
        ratios = [row["candidate_ns"] / row["baseline_ns"] for row in samples]
        logs = [math.log(value) for value in ratios]
        rng = random.Random(0x4C55535452)
        boot = sorted(math.exp(statistics.fmean(rng.choices(logs, k=len(logs)))) for _ in range(10000))
        upper = boot[math.ceil(0.95 * len(boot)) - 1]
        result.update({"raw_ratios_candidate_over_baseline": ratios,
                       "geometric_ratio": math.exp(statistics.fmean(logs)),
                       "upper_95": upper, "worst_observed_ratio": max(ratios),
                       "baseline_median_ns": statistics.median(row["baseline_ns"] for row in samples),
                       "candidate_median_ns": statistics.median(row["candidate_ns"] for row in samples)})
        if any(ratio > 1.0 for ratio in ratios):
            result.update(state="failed", reason="at least one observed candidate invocation is slower; no outliers were discarded")
        elif upper > 1.0:
            result["reason"] = "zero-margin speed evidence is inconclusive"
        else:
            result.update(state="passed", qualified=True)
    except (ValueError, KeyError, TypeError, OverflowError) as error:
        result["reason"] = str(error)
    return result


def expected_series() -> list[str]:
    result = []
    for family, modes in (("lust", DIRECT_MODES), ("cargo", CARGO_MODES), ("mixed", MIXED_MODES), ("full-rust", FULL_MODES)):
        for mode in modes:
            result.append(f"{family}/{mode}")
            if mode not in ("cold-invalid", "check", "compile-plus-test", "doctests"):
                result.append(f"{family}/{mode}/compile-plus-run")
    return result


def final_decision(report: dict) -> dict:
    reasons = []
    for name in ("identities", "provenance", "input-stability", "upstream-preservation", "upstream-accepted-corpus", "engine-trace", "concurrent-cargo", "mixed-compatibility", "full-rust-compatibility", "build-scripts", "doctests"):
        if report.get("requirements", {}).get(name, {}).get("state") != "passed":
            reasons.append(name)
    for name in expected_series():
        if report.get("series", {}).get(name, {}).get("state") != "passed":
            reasons.append(name)
    if report.get("diagnostic_without_trace") or report.get("functional_only"):
        reasons.append("diagnostic collection cannot qualify")
    if report.get("failures"):
        reasons.append("retained execution failures")
    return {"state": "passed" if not reasons else "unqualified", "qualified": not reasons,
            "blocking": reasons, "universal_coverage_proved": False, "universal_speed_proved": False,
            "scope": "Only the recorded workload, source, tools, settings, host and observations. Never a proof for all Rust or future inputs."}


def parse_trace(text: str, command: Path, compiler: Path, oracle: Path,
                candidate: bool, require_overlap: bool = False) -> dict:
    """Read complete timestamped exec/exit records, including unfinished execs.

    Paths are resolved by the kernel-executed absolute path, not substring
    matches. A successful alternate compiler, missing end, wrong status, or
    missing actual source-compilation argv cannot count as native execution.
    """
    pending, active, finished, executions = {}, {}, [], []
    for line in text.splitlines():
        match = re.match(r"^(\d+)\s+(\d+\.\d+)\s+(.*)$", line)
        if not match:
            continue
        pid, stamp, body = int(match[1]), float(match[2]), match[3]
        call = re.match(r'execve\(("(?:[^"\\]|\\.)*"),\s*(\[.*)', body)
        if body.startswith("execveat("):
            raise ValueError("execveat requires a resolved path and argv; this trace cannot qualify")
        if call:
            path = ast.literal_eval(call[1])
            require(Path(path).is_absolute(), "relative exec path cannot qualify")
            item = {"pid": pid, "start": stamp, "path": str(Path(path).resolve()), "argv_trace": call[2]}
            if "<unfinished ...>" in body:
                require(pid not in pending, "overlapping unfinished exec records")
                pending[pid] = item
                continue
            if not re.search(r"= 0$", body):
                continue
        elif "<... execve resumed>" in body:
            item = pending.pop(pid, None)
            require(item is not None, "exec completion has no beginning")
            if not re.search(r"= 0$", body):
                continue
        else:
            exit_match = re.match(r"exit_group\((\d+)\)", body)
            if exit_match and pid in active:
                item = active.pop(pid)
                item.update(end=stamp, exit_code=int(exit_match[1]))
                finished.append(item)
            continue
        require(pid not in active, "compiler exec replaced an active compilation")
        executions.append(item)
        path = Path(item["path"])
        if path != compiler.resolve() and (path == oracle.resolve() or path.name in COMPILER_NAMES):
            raise ValueError("unexpected compiler execution: " + str(path))
        if path == compiler.resolve() and '"--crate-name"' in item["argv_trace"] and re.search(r'"[^"\n]*\.rs"', item["argv_trace"]):
            active[pid] = item
    require(not pending and not active, "incomplete compiler process trace")
    require(str(command.resolve()) in [row["path"] for row in executions], "command missing from process trace")
    require(finished, "no actual source compiler execution completed")
    require(all(row["exit_code"] == 0 and row["end"] >= row["start"] for row in finished), "source compiler failed in trace")
    overlaps = [(left["pid"], right["pid"]) for at, left in enumerate(finished) for right in finished[at + 1:]
                if left["pid"] != right["pid"] and max(left["start"], right["start"]) < min(left["end"], right["end"])]
    if require_overlap:
        require(overlaps, "no overlapping independent Cargo compiler invocations observed")
    return {"state": "passed", "executions": executions, "compilations": finished, "overlapping_compiler_pids": overlaps}


class Campaign:
    def __init__(self, args):
        self.args = args
        self.repository = args.repository.resolve()
        self.output = args.output.resolve()
        require(not self.output.exists(), "output already exists; retain earlier evidence")
        self.output.mkdir(parents=True)
        self.logs = self.output / "logs"
        self.logs.mkdir()
        self.work = self.output / "work"
        self.work.mkdir()
        self.native, self.lust, self.cargo, self.rustc = (path.resolve() for path in (args.native, args.lust, args.cargo, args.rustc))
        self.helpers = {name: load_helper(self.repository, name) for name in ("qualify-native-cargo.py", "build-native-cargo.py", "qualify-native-library.py")}
        self.report = {"schema": SCHEMA, "qualified": False, "full_native_speed_qualified": False,
                       "scope": "Finite Linux host corpus; full-language fixtures are mandatory but cannot prove all Rust.",
                       "runner_sha256": digest(Path(__file__)), "diagnostic_without_trace": args.diagnostic_without_trace,
                       "functional_only": args.functional_only, "samples": args.samples,
                       "host": {"platform": platform.platform(), "cpu_affinity": sorted(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None},
                       "compiler_runtime_instrumentation": False,
                       "timing_policy": "Paired alternating baseline/candidate order; no trace during timing; no retries or discarded samples; no regression allowance.",
                       "cache_policy": "Each cold sample uses empty per-lane artifact/unit and TMPDIR runtime caches. Direct process-cold disables caching; direct fresh-cache keeps it enabled with a new cache directory per sample and no daemon. Warm rows retain per-lane TMPDIR. OS page cache is not flushed.",
                       "configuration": {"edition": "2024", "jobs": 2, "panic": "unwind", "overflow_checks": True,
                                         "debug_assertions": True, "direct_optimization": 0, "direct_debug": 0},
                       "tools": {}, "requirements": {}, "commands": [], "series": {}, "failures": []}
        self.sentinel = self.work / "compiler-sentinels"
        self.sentinel.mkdir()
        for name in COMPILER_NAMES:
            path = self.sentinel / name
            path.write_text('#!/bin/sh\nprintf "%s\\n" "$0" >> "$LUSTR_COMPILER_SENTINEL"\nexit 97\n')
            path.chmod(0o755)
        self.source_before = {}
        self.report["helper_sha256"] = {name: digest(self.repository / "tools" / name) for name in self.helpers}
        self.report["requirements"]["upstream-accepted-corpus"] = {
            "state": "unproved", "reason": "Representative direct fixtures do not cover the complete upstream accepted corpus. No complete preservation claim is authorized."}
        self.save()

    def save(self):
        self.report["decision"] = final_decision(self.report)
        self.report["qualified"] = self.report["decision"]["qualified"]
        self.report["full_native_speed_qualified"] = False
        target = self.output / "report.json"
        temporary = target.with_suffix(".pending")
        temporary.write_text(json.dumps(self.report, indent=2) + "\n")
        temporary.replace(target)

    def attempt(self, name, function):
        try:
            return function()
        except (ValueError, RuntimeError, OSError, KeyError, subprocess.SubprocessError) as error:
            self.report["failures"].append({"scope": name, "error": f"{type(error).__name__}: {error}"})
            self.report["requirements"][name] = {"state": "unproved" if isinstance(error, InfrastructureUnavailable) else "failed", "reason": str(error)}
            return None
        finally:
            self.save()

    def env(self, lane: str, directory: Path) -> dict:
        # Only the reference selects rustc. Cargo receives the actual compiler;
        # no selection wrapper or benchmark code runs inside that compiler.
        compiler = self.native if lane == "candidate" else self.rustc
        result = self.helpers["qualify-native-cargo.py"].environment(directory, compiler, self.sentinel)
        result.update(CARGO_BUILD_JOBS="2", CARGO_TERM_COLOR="never", LANG="C.UTF-8", LC_ALL="C.UTF-8")
        directory.mkdir(parents=True, exist_ok=True)
        result["TMPDIR"] = self.helpers["qualify-native-cargo.py"].prepare_runtime_temp(directory, "warm", False)["path"]
        return result

    def execute(self, label: str, argv, cwd: Path, env: dict, trace: bool = False):
        number = len(self.report["commands"])
        prefix = self.logs / f"{number:05d}"
        original = [str(arg) for arg in argv]
        actual = original
        trace_path = prefix.with_suffix(".trace")
        if trace:
            require(not self.args.diagnostic_without_trace, "process tracing disabled in diagnostic mode")
            tracer = shutil.which("strace", path="/usr/bin:/bin")
            require(tracer is not None, "strace is required for engine/concurrency qualification")
            actual = [tracer, "-f", "-ttt", "-qq", "-s", "65535", "-e", "trace=execve,execveat,exit_group", "-o", str(trace_path), "--", *original]
        timed_out, spawn_error, code = False, None, None
        before = os.getloadavg()
        started = time.monotonic_ns()
        try:
            process = subprocess.Popen(actual, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            try:
                stdout, stderr = process.communicate(timeout=self.args.timeout)
            except subprocess.TimeoutExpired:
                timed_out = True
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                stdout, stderr = process.communicate()
            code = process.returncode
        except OSError as error:
            spawn_error, stdout, stderr = str(error), b"", str(error).encode()
        elapsed = time.monotonic_ns() - started
        prefix.with_suffix(".stdout").write_bytes(stdout)
        prefix.with_suffix(".stderr").write_bytes(stderr)
        row = {"index": number, "label": label, "argv": original, "actual_argv": actual, "cwd": str(cwd),
               "compiler_tmpdir": env.get("TMPDIR"),
               "compiler_cache": {name: env.get(name) for name in ("LUST_CACHE_DIR", "LUST_NO_CACHE", "LUST_NO_DAEMON")},
               "elapsed_ns": elapsed, "exit_code": code, "timed_out": timed_out, "spawn_error": spawn_error,
               "stdout_file": str(prefix.with_suffix(".stdout").relative_to(self.output)),
               "stderr_file": str(prefix.with_suffix(".stderr").relative_to(self.output)),
               "stdout_sha256": hashlib.sha256(stdout).hexdigest(), "stderr_sha256": hashlib.sha256(stderr).hexdigest(),
               "trace": str(trace_path.relative_to(self.output)) if trace_path.exists() else None,
               "host_load_before": before, "host_load_after": os.getloadavg()}
        self.report["commands"].append(row)
        self.save()
        if trace and b"PTRACE" in stderr and b"Operation not permitted" in stderr:
            row["infrastructure_failure"] = "executor denied ptrace; compilation under trace did not run"
            raise InfrastructureUnavailable(row["infrastructure_failure"])
        require(not timed_out and spawn_error is None, label + ": timeout or spawn failure")
        require(not Path(env["LUSTR_COMPILER_SENTINEL"]).exists(), label + ": compiler sentinel was executed")
        return row, stdout, stderr

    def identity(self):
        require(platform.system() == "Linux", "this gate currently requires Linux process tracing")
        require(len({self.native, self.lust, self.cargo, self.rustc}) == 4, "candidate, Lust, Cargo and reference must be distinct paths")
        require(digest(self.native) != digest(self.rustc), "native candidate is the stock reference compiler")
        for name, path, flags, marker in (("native", self.native, ["-vV"], b"lustr-native "),
                                           ("lust", self.lust, ["--version"], b"lust "),
                                           ("cargo", self.cargo, ["-Vv"], b"cargo "),
                                           ("rustc", self.rustc, ["-vV"], b"rustc "),
                                           ("rustdoc", self.rustc.with_name("rustdoc"), ["-Vv"], b"rustdoc ")):
            require(path.is_file() and os.access(path, os.X_OK), name + " is not executable")
            with path.open("rb") as stream:
                require(stream.read(4) == b"\x7fELF", name + " must be an actual ELF executable, not a script or proxy")
            row, stdout, _ = self.execute(name + "-identity", [path, *flags], self.work, self.env("candidate", self.work / name))
            require(row["exit_code"] == 0 and stdout.startswith(marker), name + " returned the wrong compiler identity")
            self.report["tools"][name] = {"path": str(path), "sha256": digest(path), "version": stdout.decode()}
        core = self.lust.with_name("lust-core")
        require(core.is_file(), "upstream Lust core is missing")
        self.report["tools"]["lust-core"] = {"path": str(core), "sha256": digest(core)}
        self.report["requirements"]["identities"] = {"state": "passed"}
        helper = self.helpers["build-native-cargo.py"]
        if self.args.native_build_receipt is not None and self.args.tcc_root is not None:
            receipt = helper.verify_receipt(self.args.native_build_receipt.resolve(), self.repository, self.native, self.args.tcc_root.resolve())
            self.report["native_build_receipt"] = receipt
            self.report["requirements"]["provenance"] = {"state": "passed", "receipt_sha256": digest(self.args.native_build_receipt)}
        else:
            self.report["requirements"]["provenance"] = {"state": "unproved", "reason": "native build receipt and TinyCC inputs required for qualification"}
        pin = (self.repository / "upstream/lust.pin").read_text()
        tree = re.search(r'^tree = "([0-9a-f]{40})"$', pin, re.M)
        require(tree is not None, "Lust pin lacks its exact tree")
        actual = subprocess.check_output(["git", "rev-parse", "HEAD:compiler/native"], cwd=self.repository, text=True).strip()
        dirty = subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=all", "--", "compiler/native"], cwd=self.repository, text=True)
        require(actual == tree[1] and not dirty, "protected upstream Lust source diverged")
        self.report["requirements"]["upstream-preservation"] = {"state": "passed", "pinned_tree": tree[1],
                                                               "scope": "Pinned source identity only; accepted-input preservation has a separate requirement."}
        self.report["source"] = {"commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=self.repository, text=True).strip(),
                                 "tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], cwd=self.repository, text=True).strip()}

    def copy_fixture(self, source: Path, name: str):
        snapshot = self.output / "fixtures" / name
        snapshot.parent.mkdir(exist_ok=True)
        shutil.copytree(source, snapshot)
        identity = tree_files(source)
        require(tree_files(snapshot) == identity, "fixture copy differs")
        self.source_before[str(source)] = identity
        self.report.setdefault("fixture_sources", {})[name] = {"path": str(source), "files": identity}
        return snapshot

    def evaluate_series(self, name: str, observations: list[dict]):
        result = paired_decision(observations, fresh_cache=name in ("lust/fresh-cache", "lust/fresh-cache/compile-plus-run"))
        result["observations"] = observations
        self.report["series"][name] = result
        self.save()

    def inherited_cargo(self):
        path = self.output / "existing-cargo-qualification.json"
        if self.args.functional_only:
            return
        module = self.helpers["qualify-native-cargo.py"]
        failure = None
        try:
            module.qualify(self.native, self.cargo, self.rustc, self.repository / "tests/native-cargo", path, self.args.samples)
        except Exception as error:
            failure = error
        require(path.is_file(), "existing Cargo qualifier did not retain its report")
        data = json.loads(path.read_text())
        self.report["inherited_cargo_report"] = {"file": path.name, "sha256": digest(path), "success": data.get("success")}
        for mode in CARGO_MODES:
            rows = {(row["sample"], row["lane"]): row for row in data.get("timings", []) if row["mode"] == mode}
            for metric, suffix in (("elapsed_ns", ""), ("compile_plus_run_ns", "/compile-plus-run")):
                if mode == "cold-invalid" and suffix:
                    continue
                observations = []
                for sample in range(self.args.samples):
                    baseline, candidate = rows.get((sample, "oracle")), rows.get((sample, "native"))
                    if baseline is None or candidate is None:
                        continue
                    observations.append({"sample": sample, "baseline_ns": baseline.get(metric), "candidate_ns": candidate.get(metric),
                                         "correct": data.get("success") is True, "source_equal": baseline["source_sha256"] == candidate["source_sha256"],
                                         "cold": mode in ("fresh-target", "cold-invalid"),
                                         "baseline_runtime_temp": baseline.get("runtime_temp"), "candidate_runtime_temp": candidate.get("runtime_temp"),
                                         "baseline_tmpdir": data["commands"][baseline["command_index"]].get("compiler_tmpdir"),
                                         "candidate_tmpdir": data["commands"][candidate["command_index"]].get("compiler_tmpdir"),
                                         "baseline_command": baseline["command_index"], "candidate_command": candidate["command_index"]})
                self.evaluate_series("cargo/" + mode + suffix, observations)
        if failure is not None:
            raise failure

    def direct(self):
        snapshot = self.copy_fixture(self.repository / "tests/native-qualification", "direct-inputs")
        original = (snapshot / "existing-generics-traits.rs").read_text()
        family = self.work / "direct"
        family.mkdir()
        for mode in DIRECT_MODES:
            self.attempt("lust/" + mode, lambda mode=mode: self.direct_mode(family, original, mode))

    def direct_mode(self, family: Path, original: str, mode: str):
        cold = mode in COLD_DIRECT_MODES
        directory = family / mode
        directory.mkdir()
        projects, envs = {}, {}
        daemon = None
        socket_directory = None
        try:
            for lane in ("baseline", "candidate"):
                project = directory / lane
                project.mkdir()
                projects[lane] = project
                env = self.env(lane, project / "environment")
                env.update(LUST_CACHE_DIR=str(project / "cache"), LUST_NO_DAEMON="1")
                if mode in ("process-cold", "cold-invalid"):
                    env["LUST_NO_CACHE"] = "1"
                envs[lane] = env
            if not cold:
                # Unix sockets have a short path limit independent of the
                # caller's output directory. Keep only this owned endpoint in
                # a short temporary directory; its logs stay in the report.
                socket_directory = tempfile.TemporaryDirectory(prefix="lustr-speed-")
                socket = Path(socket_directory.name) / "upstream.sock"
                envs["baseline"].pop("LUST_NO_DAEMON")
                envs["baseline"].update(LUST_DAEMON_SOCKET=str(socket), LUST_DAEMON_IDLE="60")
                daemon_log = (directory / "daemon.log").open("wb")
                daemon = subprocess.Popen([str(self.lust.with_name("lust-core")), "daemon"], cwd=directory,
                                          env=envs["baseline"], stdin=subprocess.DEVNULL, stdout=daemon_log,
                                          stderr=subprocess.STDOUT, start_new_session=True)
                deadline = time.monotonic() + 15
                while not socket.exists() and daemon.poll() is None and time.monotonic() < deadline:
                    time.sleep(0.01)
                if not socket.exists() or daemon.poll() is not None:
                    message = (directory / "daemon.log").read_text(errors="replace")
                    if "Operation not permitted" in message:
                        raise InfrastructureUnavailable("executor denied the upstream daemon socket; warm measurements are unavailable")
                    raise ValueError("upstream warm daemon did not become ready: " + message)
            observations, total_observations = [], []
            for sample in range(-1, 2 if self.args.functional_only else self.args.samples):
                source = original
                answer = 14
                if mode == "body-edit" and sample >= 0:
                    source = source.replace("Value(3), Value(4)", f"Value({4 + sample}), Value(4)")
                    answer = 16 + 2 * sample
                elif mode == "signature-edit" and sample >= 0:
                    source = source.replace("i32", "i64" if sample % 2 == 0 else "i32")
                elif mode == "type-edit" and sample >= 0 and sample % 2 == 0:
                    source = source.replace("struct Value(i32);", "struct Value(i32, i32);")
                    source = source.replace("Value(3), Value(4)", "Value(3, 9), Value(4, 10)")
                elif mode == "macro-edit":
                    value = sample + 1
                    source = source.replace("fn main() {", f"macro_rules! adjust {{ ($value:expr) => {{ $value + {value} }}; }}\n\nfn main() {{")
                    source = source.replace("Some(total(&values) * 2)", "Some(adjust!(total(&values)) * 2)")
                    answer += 2 * value
                elif mode == "cold-invalid":
                    source = 'fn main() { let value: u32 = "wrong"; println!("{}", value); }\n'
                source_id = hashlib.sha256(source.encode()).hexdigest()
                retained = directory / (source_id + ".rs")
                if not retained.exists():
                    retained.write_text(source)
                results = {}
                for lane in (("baseline", "candidate") if sample % 2 == 0 else ("candidate", "baseline")):
                    project = projects[lane]
                    path, artifact = project / "main.rs", project / "program"
                    if not path.exists() or path.read_text() != source:
                        path.write_text(source)
                    if cold:
                        artifact.unlink(missing_ok=True)
                    compile_cache = None
                    if mode == "fresh-cache":
                        # Keep caching enabled and include the first admission
                        # and store. Never reuse a previous lane/sample cache.
                        cache_root = project / "compile-caches"
                        require(not cache_root.is_symlink(), "fresh-cache root is a symlink")
                        cache_root.mkdir(exist_ok=True)
                        cache = cache_root / f"sample-{sample + 1}"
                        cache.mkdir()
                        envs[lane]["LUST_CACHE_DIR"] = str(cache)
                        envs[lane].pop("LUST_NO_CACHE", None)
                        compile_cache = {"path": str(cache), "fresh": True, "enabled": True,
                                         "empty_before_compile": not any(cache.iterdir())}
                    elif cold:
                        cache = project / "cache"
                        if cache.exists():
                            shutil.rmtree(cache)
                    runtime_temp = self.helpers["qualify-native-cargo.py"].prepare_runtime_temp(
                        project / "environment", "cold-" + str(sample + 1) if cold else "warm", cold)
                    envs[lane]["TMPDIR"] = runtime_temp["path"]
                    argv = [self.lust, "build", path, "-o", artifact] if lane == "baseline" else [
                        self.native, "--crate-name", "speed_contract", "--edition=2024", "--crate-type", "bin",
                        "-C", "opt-level=0", "-C", "debuginfo=0", "-C", "panic=unwind",
                        "-C", "overflow-checks=yes", "-C", "debug-assertions=yes", path, "-o", artifact]
                    row, _, stderr = self.execute(f"direct-{mode}-{sample}-{lane}", argv, project, envs[lane])
                    runtime = 0
                    if mode == "cold-invalid":
                        require(row["exit_code"] == 1 and b"L0110" in stderr and not artifact.exists(), "type-error rejection did not match upstream semantics")
                    else:
                        require(row["exit_code"] == 0 and artifact.is_file(), "native supported compilation failed")
                        run, stdout, _ = self.execute(f"direct-{mode}-{sample}-{lane}-run", [artifact], project, envs[lane])
                        require(run["exit_code"] == 0 and stdout == f"{answer}\n".encode(), "supported program result differs")
                        runtime = run["elapsed_ns"]
                    results[lane] = (row["elapsed_ns"], runtime, digest(path), runtime_temp, row["compiler_tmpdir"],
                                     compile_cache, row["compiler_cache"])
                if sample >= 0 and not self.args.functional_only:
                    base, candidate = results["baseline"], results["candidate"]
                    entry = {"sample": sample, "baseline_ns": base[0], "candidate_ns": candidate[0],
                             "correct": True, "source_equal": base[2] == candidate[2], "source_sha256": source_id,
                             "cold": cold,
                             "baseline_runtime_temp": base[3], "candidate_runtime_temp": candidate[3],
                             "baseline_tmpdir": base[4], "candidate_tmpdir": candidate[4],
                             "source_file": str(retained.relative_to(self.output))}
                    if mode == "fresh-cache":
                        entry.update(baseline_compile_cache=base[5], candidate_compile_cache=candidate[5],
                                     baseline_cache_env=base[6], candidate_cache_env=candidate[6])
                    observations.append(entry)
                    total_observations.append(dict(entry, baseline_ns=base[0] + base[1], candidate_ns=candidate[0] + candidate[1]))
            self.evaluate_series("lust/" + mode, observations)
            if mode != "cold-invalid":
                self.evaluate_series("lust/" + mode + "/compile-plus-run", total_observations)
        finally:
            if daemon is not None:
                if daemon.poll() is None:
                    os.killpg(daemon.pid, signal.SIGTERM)
                    try:
                        daemon.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        os.killpg(daemon.pid, signal.SIGKILL)
                        daemon.wait()
                daemon_log.close()
            if socket_directory is not None:
                socket_directory.cleanup()

    def workspace(self, family: str, snapshot: Path, package: str, executable: str, expected: bytes, modes):
        for mode in modes:
            self.attempt(family + "/" + mode, lambda mode=mode: self.workspace_mode(family, snapshot, package, executable, expected, mode))

    def edited_workspace_source(self, family: str, original: str, expected: bytes, mode: str, sample: int):
        helper = self.helpers["qualify-native-cargo.py"]
        if family == "mixed":
            require(original.count("value * 7") == 1 and expected.count(b"local=21") == 1,
                    "mixed edit fixture no longer has its exact helper/output contract")
            definition, body, increment = helper.semantic_edit(mode, sample, "value * 7", "i32")
            return definition + original.replace("value * 7", body), expected.replace(b"local=21", f"local={21 + increment}".encode())
        require(family == "full-rust" and original.count("fixture_core::body_probe(20)") == 1
                and expected.count(b"body=42") == 1 and "lustr_edit_probe" not in original,
                "full-Rust edit fixture no longer has its exact body/output contract")
        definition, body, increment = helper.semantic_edit(mode, sample, "value", "u32")
        probe = f"fn lustr_edit_probe(value: u32) -> u32 {{ {body} }}\n"
        source = definition + probe + original.replace("fixture_core::body_probe(20)", "lustr_edit_probe(fixture_core::body_probe(20))")
        return source, expected.replace(b"body=42", f"body={42 + increment}".encode())

    def workspace_mode(self, family: str, snapshot: Path, package: str, executable: str, expected: bytes, mode: str):
        directory = self.work / family / mode
        directory.mkdir(parents=True)
        projects = {}
        envs = {}
        original = (snapshot / "app/src/main.rs").read_text()
        for lane in ("baseline", "candidate"):
            projects[lane] = directory / lane
            shutil.copytree(snapshot, projects[lane])
            envs[lane] = self.env(lane, directory / (lane + "-environment"))
        observations, totals = [], []
        functional = {}
        for sample in range(-1, 2 if self.args.functional_only else self.args.samples):
            results = {}
            for lane in (("baseline", "candidate") if sample % 2 == 0 else ("candidate", "baseline")):
                project = projects[lane]
                target = directory / (lane + "-targets") / (str(sample) if mode in ("fresh-target", "check", "release", "compile-plus-test", "doctests") else "retained")
                if mode in ("fresh-target", "check", "release", "compile-plus-test", "doctests"):
                    for cache in list(project.rglob(".lust-cache")):
                        require(cache.resolve().is_relative_to(project.resolve()) and not cache.is_symlink(), "cache escapes owned fixture")
                        shutil.rmtree(cache)
                cold = mode in ("fresh-target", "check", "release", "compile-plus-test", "doctests")
                runtime_temp = self.helpers["qualify-native-cargo.py"].prepare_runtime_temp(
                    directory / (lane + "-environment"), "cold-" + str(sample + 1) if cold else "warm", cold)
                envs[lane]["TMPDIR"] = runtime_temp["path"]
                source, wanted = original, expected
                if sample >= 0 and mode == "body-edit":
                    source = source.replace("helper(3)", f"helper({4 + sample})")
                    wanted = expected.replace(b"local=21", f"local={(4 + sample) * 7}".encode())
                elif sample >= 0 and mode == "signature-edit":
                    source = source.replace("fn helper(value: i32) -> i32", "fn helper(value: i64) -> i64" if sample % 2 == 0 else "fn helper(value: i32) -> i32")
                elif mode in ("type-edit", "macro-edit"):
                    source, wanted = self.edited_workspace_source(family, original, expected, mode, sample)
                root = project / "app/src/main.rs"
                if sample >= 0 and mode in ("body-edit", "signature-edit", "type-edit", "macro-edit"):
                    require(root.read_text() != source, "edit did not change the previous source")
                if root.read_text() != source:
                    root.write_text(source)
                retained = self.output / "edited-inputs" / family / mode / (digest(root) + ".rs")
                retained.parent.mkdir(parents=True, exist_ok=True)
                if not retained.exists():
                    retained.write_text(source)
                subcommand = "check" if mode == "check" else "test" if mode in ("compile-plus-test", "doctests") else "build"
                argv = [self.cargo, subcommand, "--offline", "--locked", "--jobs", "2", "--target-dir", target, "--message-format=json", "-vv"]
                if mode == "compile-plus-test":
                    argv += ["--workspace", "--all-targets"]
                elif mode == "doctests":
                    argv += ["--workspace", "--doc"]
                    envs[lane]["RUSTDOC"] = str(self.native if lane == "candidate" else self.rustc.with_name("rustdoc"))
                else:
                    argv += ["-p", package]
                if mode == "release":
                    argv += ["--release"]
                trace = sample == -1 and family == "mixed" and mode == "fresh-target" and not self.args.diagnostic_without_trace
                row, stdout, stderr = self.execute(f"{family}-{mode}-{sample}-{lane}", argv, project, envs[lane], trace=trace)
                correct, reason, run_ns = row["exit_code"] == 0, None, 0
                if not correct:
                    reason = "Cargo command failed with exit " + str(row["exit_code"])
                else:
                    events = []
                    for line in stdout.splitlines():
                        if line.startswith(b"{"):
                            events.append(json.loads(line))
                    artifact_events = [event for event in events if event.get("reason") == "compiler-artifact"]
                    if mode == "no-change" and sample >= 0:
                        require(artifact_events and all(event.get("fresh") is True for event in artifact_events), "no-change build performed compilation")
                    if mode in ("body-edit", "signature-edit", "type-edit", "macro-edit", "no-change"):
                        compiler = self.native if lane == "candidate" else self.rustc
                        row["rebuild_evidence"] = self.helpers["qualify-native-cargo.py"].compiler_rebuild_evidence(
                            events, self.helpers["qualify-native-library.py"].compiler_invocations(stderr.decode(errors="replace"), compiler),
                            compiler, project, root, executable, mode == "no-change" and sample >= 0)
                    if family == "full-rust" and sample == -1 and mode == "fresh-target":
                        script_events = [event for event in events if event.get("reason") == "build-script-executed"]
                        require(script_events and any(["FIXTURE_BUILD_VALUE", "7"] in event.get("env", []) for event in script_events),
                                "actual Cargo build-script output is missing")
                        self.report.setdefault("build_script_evidence", {})[lane] = script_events
                    if mode in ("compile-plus-test", "doctests"):
                        require(b"test result: ok" in stdout and re.search(rb"[1-9][0-9]* passed", stdout), "required tests did not execute")
                    if mode not in ("check", "compile-plus-test", "doctests"):
                        artifact = target / ("release" if mode == "release" else "debug") / executable
                        require(artifact.is_file(), "Cargo did not produce the requested executable")
                        run, output, _ = self.execute(f"{family}-{mode}-{sample}-{lane}-run", [artifact], project, envs[lane])
                        correct = run["exit_code"] == 0 and output == wanted
                        run_ns = run["elapsed_ns"]
                        reason = None if correct else "compiled program did not return the required result"
                    if trace and correct:
                        evidence = parse_trace((self.output / row["trace"]).read_text(), self.cargo,
                                               self.native if lane == "candidate" else self.rustc, self.rustc,
                                               candidate=lane == "candidate", require_overlap=True)
                        self.report.setdefault("concurrency_evidence", {})[lane] = evidence
                results[lane] = {"time": row["elapsed_ns"], "runtime": run_ns, "files": tree_files(project), "correct": correct, "reason": reason,
                                 "runtime_temp": runtime_temp, "cold": cold, "compiler_tmpdir": row["compiler_tmpdir"],
                                 "command": row["index"], "source_file": str(retained.relative_to(self.output)),
                                 "expected_stdout": wanted.decode()}
                functional[lane] = {"correct": correct, "reason": reason, "sample": sample, "command": row["index"]}
            baseline, candidate = results["baseline"], results["candidate"]
            equal = baseline["files"] == candidate["files"]
            self.report.setdefault("functional", {})[family + "/" + mode] = dict(functional)
            require(equal, "workspace sources differ between lanes")
            require(baseline["correct"] and candidate["correct"], "unchanged workspace failed: " + json.dumps(functional))
            if sample >= 0 and not self.args.functional_only:
                entry = {"sample": sample, "baseline_ns": baseline["time"], "candidate_ns": candidate["time"],
                         "correct": baseline["correct"] and candidate["correct"], "source_equal": equal,
                         "cold": baseline["cold"] and candidate["cold"],
                         "baseline_runtime_temp": baseline["runtime_temp"], "candidate_runtime_temp": candidate["runtime_temp"],
                         "baseline_tmpdir": baseline["compiler_tmpdir"], "candidate_tmpdir": candidate["compiler_tmpdir"],
                         "baseline_source_files": baseline["files"], "candidate_source_files": candidate["files"],
                         "baseline_command": baseline["command"], "candidate_command": candidate["command"],
                         "source_file": candidate["source_file"], "expected_stdout": candidate["expected_stdout"]}
                observations.append(entry)
                totals.append(dict(entry, baseline_ns=baseline["time"] + baseline["runtime"], candidate_ns=candidate["time"] + candidate["runtime"]))
        self.evaluate_series(family + "/" + mode, observations)
        if mode not in ("check", "compile-plus-test", "doctests"):
            self.evaluate_series(family + "/" + mode + "/compile-plus-run", totals)

    def run(self):
        try:
            self.attempt("identities", self.identity)
            if self.report["requirements"].get("identities", {}).get("state") != "passed":
                return self.report
            mixed = self.copy_fixture(self.repository / "tests/native-library", "mixed")
            full = self.copy_fixture(self.repository / "crates/lustr/fixtures/full-rust", "full-rust")
            # Existing mixed fixture includes safe Rust and an unsafe producer.
            # Its sources are copied exactly, including unsupported requirements.
            self.workspace("mixed", mixed, "lustr_library_contract", "lustr_library_contract", b"left=31 right=109 local=21\n", MIXED_MODES)
            self.workspace("full-rust", full, "fixture-app", "fixture-app",
                           b"derived=41 generated=7 async=42 raw=19 gat=13 const=15 ffi=21 body=42 probe=41 panic=1 drops=1\n", FULL_MODES)
            for family in ("mixed", "full-rust"):
                names = MIXED_MODES if family == "mixed" else FULL_MODES
                passed = all(all(row.get("correct") is True for row in self.report.get("functional", {}).get(family + "/" + mode, {}).values())
                             and set(self.report.get("functional", {}).get(family + "/" + mode, {})) == {"baseline", "candidate"} for mode in names)
                self.report["requirements"][family + "-compatibility"] = {"state": "passed" if passed else "failed"}
            concurrency = self.report.get("concurrency_evidence", {})
            state = "passed" if set(concurrency) == {"baseline", "candidate"} and all(row.get("state") == "passed" for row in concurrency.values()) else "unproved"
            self.report["requirements"]["engine-trace"] = {"state": state}
            self.report["requirements"]["concurrent-cargo"] = {"state": state, "jobs_per_lane": 2}
            self.report["requirements"]["build-scripts"] = {"state": "passed" if set(self.report.get("build_script_evidence", {})) == {"baseline", "candidate"} else "unproved"}
            doctests = self.report.get("functional", {}).get("full-rust/doctests", {})
            self.report["requirements"]["doctests"] = {"state": "passed" if set(doctests) == {"baseline", "candidate"} and all(row.get("correct") for row in doctests.values()) else "unproved"}
            self.attempt("direct-supported-inputs", self.direct)
            self.attempt("existing-cargo-suite", self.inherited_cargo)
            stable = all(digest(Path(value["path"])) == value["sha256"] for value in self.report["tools"].values())
            stable = stable and all(tree_files(Path(path)) == files for path, files in self.source_before.items())
            stable = stable and digest(Path(__file__)) == self.report["runner_sha256"]
            stable = stable and all(digest(self.repository / "tools" / name) == value for name, value in self.report["helper_sha256"].items())
            self.report["requirements"]["input-stability"] = {"state": "passed" if stable else "failed"}
        finally:
            for name in expected_series():
                self.report["series"].setdefault(name, {"state": "unproved", "qualified": False, "reason": "required measurements missing or functional compilation failed"})
            self.save()
        return self.report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=Path(__file__).resolve().parents[1])
    for name in ("native", "lust", "cargo", "rustc", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--native-build-receipt", type=Path)
    parser.add_argument("--tcc-root", type=Path)
    parser.add_argument("--samples", type=int, default=12)
    parser.add_argument("--timeout", type=int, default=180)
    parser.add_argument("--diagnostic-without-trace", action="store_true")
    parser.add_argument("--functional-only", action="store_true", help="collect correctness only; always leaves speed unqualified")
    args = parser.parse_args()
    if args.samples < MIN_PAIRS or args.samples % 2 or args.timeout <= 0:
        parser.error(f"samples must be even and at least {MIN_PAIRS}; timeout must be positive")
    campaign = Campaign(args)
    report = campaign.run()
    print(json.dumps({"report": str(campaign.output / "report.json"), "decision": report["decision"]}))
    return 0 if report["qualified"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
