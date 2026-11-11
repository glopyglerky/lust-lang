#!/usr/bin/env python3
"""Compare the real native Cargo entry point with pinned Lust on unchanged fixtures.

This is a bounded functional audit, not a speed qualification or proof of full
upstream coverage. Every selected case attempts both compilers once. Runtime
comparison uses only the masking rules in the pinned upstream differential gate.
"""
from __future__ import annotations

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
from collections import Counter
import errno
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import shutil
import signal
import subprocess
import time
import tomllib


ROOT = Path(__file__).resolve().parents[1]
PROFILES = {
    "pass": {"name": "dev-unwind", "panic": "unwind", "overflow-checks": "on", "debug-assertions": "on"},
    "unwind": {"name": "dev-unwind", "panic": "unwind", "overflow-checks": "on", "debug-assertions": "on"},
    "release": {"name": "release-semantics-unwind", "panic": "unwind", "overflow-checks": "off", "debug-assertions": "off"},
}
GAPS = [
    "pass and unwind fixtures under release semantics",
    "panic=abort profiles",
    "fail and reject diagnostic corpora",
    "external C compiler, LLVM, sanitizer and cross-target variants",
    "other upstream gates, Cargo dependency graphs and retained-state behavior",
    "all cold and warm performance requirements",
]


def sha(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def write_json(path: Path, value) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    temporary.replace(path)


def storage_snapshot(path: Path) -> dict:
    status = os.statvfs(path)
    return {"available_bytes": status.f_bavail * status.f_frsize, "free_bytes": status.f_bfree * status.f_frsize,
            "free_inodes": status.f_ffree, "monotonic_ns": time.monotonic_ns()}


def inventory(donor: Path) -> tuple[list[dict], dict[str, str]]:
    cases, files = [], {}
    for suite in PROFILES:
        directory = donor / "tests" / suite
        sources = sorted(directory.glob("*.rs"))
        if not sources:
            raise ValueError(f"missing or empty upstream fixture suite: {directory}")
        for path in sorted(directory.rglob("*")):
            if path.is_file():
                files[str(path.relative_to(donor))] = sha(path)
        cases.extend({"id": f"{suite}/{path.name}", "suite": suite,
                      "source": str(path.resolve()), "source_sha256": sha(path),
                      "profile": PROFILES[suite]} for path in sources)
    return cases, files


def select_cases(cases: list[dict], requested: list[str]) -> list[dict]:
    if not requested:
        return cases
    if len(requested) != len(set(requested)):
        raise ValueError("duplicate --case selectors are not allowed")
    available = {case["id"] for case in cases}
    unknown = sorted(set(requested) - available)
    if unknown:
        raise ValueError(f"unknown exact case IDs: {unknown}")
    return [case for case in cases if case["id"] in requested]


def profile_flags(suite: str) -> list[str]:
    return [arg for key in ("panic", "overflow-checks", "debug-assertions")
            for arg in ("-C", f"{key}={PROFILES[suite][key]}")]


def mask_output(data: bytes, source: Path, donor: Path, *, stderr: bool) -> bytes:
    """Port mask_stdout/mask_stderr/lust_stderr/no_frames, without new masks."""
    if not stderr:
        return data.replace(os.fsencode(source), b"@CASE@").replace(os.fsencode(donor), b"@ROOT@")
    data = re.sub(rb"^thread '([^']*)' \([0-9]+\) ", rb"thread '\1' (TID) ", data, flags=re.M)
    data = re.sub(rb"(/rustc/[0-9a-f]+|/[^ :\n]*/lib/rustlib/src/rust)/library/", b"@STD@/library/", data)
    lines = []
    for line in re.findall(rb"[^\n]*\n|[^\n]+$", data):
        if (line.removesuffix(b"\n") == b"timeout: the monitored command dumped core"
                or re.search(rb": line [0-9]+: +[0-9]+ .* timeout ", line)
                or re.match(rb"^ +(?:[0-9]+: |at )", line)):
            continue
        lines.append(line)
    return b"".join(lines).replace(os.fsencode(source), b"@CASE@")


def execute(argv: list[str], cwd: Path, env: dict[str, str], logs: Path,
            label: str, timeout: float, root: Path) -> dict:
    stdout, stderr = logs / f"{label}.stdout", logs / f"{label}.stderr"
    record = {"argv": argv, "cwd": str(cwd), "timeout_seconds": timeout,
              "exit_code": None, "timed_out": False, "error": None}
    start = time.monotonic_ns()
    with stdout.open("wb") as out, stderr.open("wb") as err:
        try:
            process = subprocess.Popen(argv, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                       stdout=out, stderr=err, start_new_session=True)
            try:
                record["exit_code"] = process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                record["timed_out"] = True
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                record["exit_code"] = process.wait()
        except OSError as error:
            record["error"] = str(error)
    record["elapsed_ns"] = time.monotonic_ns() - start
    record["signal"] = -record["exit_code"] if record["exit_code"] is not None and record["exit_code"] < 0 else None
    for name, path in (("stdout", stdout), ("stderr", stderr)):
        record[name] = str(path.relative_to(root))
        record[name + "_sha256"] = sha(path)
    return record


def complete_command(command: dict) -> bool:
    return not command["timed_out"] and command["error"] is None and command["exit_code"] is not None


def compare_runs(lanes: dict, root: Path, source: Path, donor: Path, panic: bytes | None) -> list[str]:
    failures = []
    runs = [lanes[name]["run"] for name in ("lust", "native")]
    if runs[0]["exit_code"] != runs[1]["exit_code"]:
        failures.append("runtime exit status or signal differs")
    for channel in ("stdout", "stderr"):
        outputs = [(root / run[channel]).read_bytes() for run in runs]
        normalized = [mask_output(data, source, donor, stderr=channel == "stderr") for data in outputs]
        for lane, run, data in zip(("lust", "native"), runs, normalized):
            path = root / (run[channel] + ".masked")
            path.write_bytes(data)
            lanes[lane][channel + "_masked"] = str(path.relative_to(root))
            lanes[lane][channel + "_masked_sha256"] = sha(path)
        if normalized[0] != normalized[1]:
            failures.append(f"runtime {channel} differs after upstream masking")
        if channel == "stderr" and panic:
            for name, data in zip(("lust", "native"), outputs):
                if panic not in data:
                    failures.append(f"{name} stderr lacks the fixture's panic expectation")
    return failures


def artifact_info(path: Path) -> dict:
    with path.open("rb") as stream:
        if stream.read(4) != b"\x7fELF" or not os.access(path, os.X_OK):
            raise ValueError(f"not an executable ELF: {path}")
    return {"path": str(path), "sha256": sha(path), "size_bytes": path.stat().st_size}


def snapshot_compiler(source: Path, root: Path, lane: str) -> dict:
    """Freeze actual ELF files, retaining an installed compiler's support layout."""
    source = source.resolve(strict=True)
    artifact_info(source)
    destination = root / "compilers" / lane
    (destination / "bin").mkdir(parents=True)
    paths = [source]
    if lane == "lust":
        paths.append(source.with_name("lust-core"))
    records = []
    for path in paths:
        identity = artifact_info(path)
        target = destination / "bin" / path.name
        shutil.copy2(path, target)
        if sha(target) != identity["sha256"] or sha(path) != identity["sha256"]:
            raise ValueError(f"compiler changed while snapshotting: {path}")
        records.append(dict(identity, executed_path=str(target)))
    support = source.parent.parent / "lib/lust"
    support_files = {}
    if support.is_dir():
        shutil.copytree(support, destination / "lib/lust")
        for path in sorted(support.rglob("*")):
            if path.is_file():
                relative = path.relative_to(support)
                digest = sha(path)
                if sha(destination / "lib/lust" / relative) != digest:
                    raise ValueError(f"compiler support changed while snapshotting: {path}")
                support_files[str(relative)] = digest
    return {"files": records, "executable": records[0]["executed_path"],
            "installed_support": str(support) if support_files else None,
            "support_sha256": support_files,
            "build_time_support": not bool(support_files)}


def package_provenance(compiler: dict, pin: dict, root: Path) -> dict:
    source = Path(compiler["files"][0]["path"]).parent.parent / "lustr-toolchain.json"
    if not source.is_file():
        return {"state": "unproved", "reason": "No packaged source declaration accompanies the measured upstream ELF files."}
    manifest = json.loads(source.read_text())
    if manifest.get("lust_revision") != pin["revision"] or manifest.get("lust_tree") != pin["tree"]:
        raise ValueError("upstream package declares a different Lust source pin")
    payload = {entry["path"]: entry["sha256"] for entry in manifest["payload"]}
    for record in compiler["files"]:
        if payload.get("bin/" + Path(record["executed_path"]).name) != record["sha256"]:
            raise ValueError("upstream compiler ELF does not match its package manifest")
    for relative, digest in compiler["support_sha256"].items():
        if payload.get("lib/lust/" + relative) != digest:
            raise ValueError("upstream runtime support does not match its package manifest")
    target = root / "upstream-package-manifest.json"
    shutil.copy2(source, target)
    return {"state": "passed", "kind": "Package source declaration bound to actual ELF and support hashes; not a compiler build receipt.",
            "path": str(target.relative_to(root)), "sha256": sha(target)}


def case_environment(sandbox: Path) -> dict[str, str]:
    # The complete declared environment is identical for both lanes. No ambient
    # Rust/Cargo/Lust settings or credentials are passed through.
    return {"PATH": "/usr/bin:/bin", "HOME": str(sandbox / "home"),
            "TMPDIR": str(sandbox / "tmp"), "LANG": "C", "LC_ALL": "C", "TZ": "UTC",
            "LUST_NO_DAEMON": "1", "LUST_NO_CACHE": "1",
            "LUST_CACHE_DIR": str(sandbox / "cache")}


def reset_sandbox(path: Path) -> None:
    if path.exists():
        shutil.rmtree(path)
    for name in ("home", "tmp", "cache"):
        (path / name).mkdir(parents=True)


def run_case(case: dict, index: int, compilers: dict, root: Path, donor: Path,
             compile_timeout: float, run_timeout: float) -> dict:
    source = Path(case["source"])
    directory = root / "cases" / case["id"][:-3]
    directory.mkdir(parents=True)
    logs = directory / "logs"
    logs.mkdir()
    sandbox = directory / "sandbox"
    env = case_environment(sandbox)
    write_json(directory / "environment.json", env)
    result = dict(case, state="unproved", issues=[], lanes={},
                  environment=str((directory / "environment.json").relative_to(root)))
    flags = profile_flags(case["suite"])
    # Alternate the first lane without claiming that this functional pool is a
    # performance measurement. No case is retried or dropped.
    order = ("lust", "native") if index % 2 == 0 else ("native", "lust")
    result["lane_order"] = list(order)
    for lane in order:
        row = result["lanes"][lane] = {}
        phase = "setup"
        try:
            reset_sandbox(sandbox)
            artifacts = directory / lane
            artifacts.mkdir()
            executable = artifacts / source.stem
            if lane == "native":
                argv = [compilers[lane]["executable"], "--crate-name", source.stem,
                        "--crate-type", "bin", "--edition=2024", "--emit=link,dep-info",
                        "--out-dir", str(artifacts), *flags, str(source)]
            else:
                argv = [compilers[lane]["executable"], "build", *flags, str(source), "-o", str(executable)]
            row["compile"] = execute(argv, sandbox, env, logs, lane + "-compile", compile_timeout, root)
            if not complete_command(row["compile"]) or row["compile"]["exit_code"] != 0:
                continue
            phase = "artifact"
            row["artifact"] = artifact_info(executable)
            if lane == "native":
                dep_info = executable.with_suffix(".d")
                content = dep_info.read_bytes()
                escaped = os.fsencode(source).replace(b" ", b"\\ ")
                target = os.fsencode(executable).replace(b" ", b"\\ ") + b": "
                first = content.split(b"\n", 1)[0]
                if (not first.startswith(target) or b" " + escaped + b" " not in b" " + first[len(target):] + b" "
                        or escaped + b":\n" not in content):
                    raise ValueError("native dep-info does not declare the source and requested artifact")
                row["dep_info"] = {"path": str(dep_info.relative_to(root)), "sha256": sha(dep_info)}
            # Compilers may leave temporary files. Programs start with the same
            # recreated cwd, executable name, HOME, TMPDIR and environment.
            phase = "runtime setup"
            reset_sandbox(sandbox)
            shutil.copy2(executable, sandbox / "program")
            row["run"] = execute(["./program"], sandbox, env, logs, lane + "-run", run_timeout, root)
        except (OSError, ValueError) as error:
            row["error"] = str(error)
            row["error_phase"] = phase
            row["error_kind"] = ("artifact contract" if phase == "artifact"
                                  and (isinstance(error, ValueError) or error.errno in (errno.ENOENT, errno.EISDIR, errno.EACCES))
                                  else "infrastructure")
    lanes = result["lanes"]
    if any("error" in row for row in lanes.values()):
        result["issues"].append("artifact or runner error; see lane evidence")
        result["state"] = "failed" if any(row.get("error_kind") == "artifact contract" for row in lanes.values()) else "unproved"
    elif any(not complete_command(row["compile"]) for row in lanes.values()):
        result["issues"].append("compiler timeout or process-launch failure")
    elif lanes["lust"]["compile"]["exit_code"] != 0:
        result["issues"].append("upstream rejected or crashed on an expected accepted fixture")
    elif lanes["native"]["compile"]["exit_code"] != 0:
        result["state"] = "failed"
        result["issues"].append("native rejected or crashed on an upstream-accepted fixture")
    elif any(not complete_command(row["run"]) for row in lanes.values()):
        result["issues"].append("runtime timeout or process-launch failure")
    else:
        expectations = re.findall(rb"// panic: (.*)", source.read_bytes())
        panic = expectations[0].rstrip() if expectations else None
        result["issues"] = compare_runs(lanes, root, source, donor, panic)
        result["state"] = "failed" if result["issues"] else "passed"
    try:
        if sha(source) != case["source_sha256"]:
            raise ValueError("fixture source changed during the audit")
    except (OSError, ValueError) as error:
        result["issues"].append(str(error))
        result["state"] = "unproved"
    # Keep every raw log, dep-info and artifact hash. Thousands of per-case
    # binaries would cost gigabytes; neither passing nor failing ones are kept.
    try:
        for lane, row in lanes.items():
            executable = directory / lane / source.stem
            if executable.is_file():
                row.setdefault("artifact", {"path": str(executable), "sha256": sha(executable),
                                             "size_bytes": executable.stat().st_size})
                executable.unlink()
                row["artifact"]["retained"] = False
        if sandbox.exists():
            shutil.rmtree(sandbox)
        write_json(directory / "result.json", result)
    except OSError as error:
        result["issues"].append(f"evidence cleanup/write failed: {error}")
        result["state"] = "unproved"
    return result


def summarize(rows: list[dict], selected: list[dict], total: int, stable: bool) -> dict:
    expected = {case["id"] for case in selected}
    observed = [row["id"] for row in rows]
    counts = dict(Counter(row["state"] for row in rows))
    missing, unexpected = sorted(expected - set(observed)), sorted(set(observed) - expected)
    duplicates = sorted(name for name, count in Counter(observed).items() if count != 1)
    attempted = [row["id"] for row in rows
                 if all("compile" in row.get("lanes", {}).get(lane, {}) for lane in ("lust", "native"))]
    complete = not missing and not unexpected and not duplicates and set(attempted) == expected and stable
    passed = complete and counts.get("passed", 0) == len(selected)
    return {"state": "passed" if passed else "failed" if counts.get("failed", 0) else "unproved",
            "selected_cases": len(selected), "completed_cases": len(rows), "total_corpus_cases": total,
            "scope": "full bounded matrix" if len(selected) == total else "explicit partial selection",
            "counts": counts, "missing": missing, "unexpected": unexpected, "duplicates": duplicates,
            "compiler_pairs_attempted": len(attempted), "compiler_pairs_missing": sorted(expected - set(attempted)),
            "inputs_stable": stable, "complete": complete, "success": passed,
            "qualified": False, "upstream_accepted_corpus": "unproved"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--lust", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--repository", type=Path, default=ROOT)
    parser.add_argument("--case", action="append", default=[], help="exact suite/name.rs; selects an explicitly partial pilot")
    parser.add_argument("--workers", type=int, choices=(1, 2), default=2)
    parser.add_argument("--compile-timeout", type=float, default=30)
    parser.add_argument("--run-timeout", type=float, default=10)
    args = parser.parse_args()
    if args.compile_timeout <= 0 or args.run_timeout <= 0:
        parser.error("timeouts must be positive")
    repository, root = args.repository.resolve(), args.output.resolve()
    if root.is_relative_to(repository):
        parser.error("--output must be outside the source checkout")
    root.mkdir(parents=True, exist_ok=False)
    shutil.copy2(Path(__file__), root / "runner.py")
    donor = repository / "compiler/native"
    rows = []
    report = {"schema": "lustr.native-upstream-corpus.v1", "success": False, "qualified": False,
              "performance_claim": False, "functional_only": True,
              "scope": "Unchanged pass/unwind/release sources with the explicit profile matrix; direct native Cargo CLI versus upstream Lust.",
              "profiles": PROFILES, "coverage_gaps": GAPS, "workers": args.workers,
              "compile_timeout_seconds": args.compile_timeout, "run_timeout_seconds": args.run_timeout,
              "retry_count": 0, "compiler_snapshot": "Copied executable bytes, no compiler wrappers or source rewrites.",
              "runner_sha256": sha(root / "runner.py"), "failures": [], "results": rows,
              "storage": [{"completed_cases": 0, **storage_snapshot(root)}],
              "candidate_build_recipe": "unproved; this runner records an ELF hash and local source context, not an observed compiler build receipt"}
    write_json(root / "report.json", report)
    start = time.monotonic_ns()
    try:
        pin = tomllib.loads((repository / "upstream/lust.pin").read_text())
        report["upstream_pin"] = pin
        revision = subprocess.run(["git", "-C", str(repository), "rev-parse", "HEAD"], check=True, capture_output=True, text=True).stdout.strip()
        donor_tree = subprocess.run(["git", "-C", str(repository), "rev-parse", "HEAD:compiler/native"], check=True, capture_output=True, text=True).stdout.strip()
        donor_changes = subprocess.run(["git", "-C", str(repository), "status", "--porcelain", "--", "compiler/native"], check=True, capture_output=True, text=True).stdout
        if donor_tree != pin["tree"] or donor_changes:
            raise ValueError("donor tree differs from the pinned unchanged upstream reference")
        report["source_commit"] = revision
        patch = subprocess.run(["git", "-C", str(repository), "diff", "--binary", "HEAD", "--", "crates/lustr-native"], check=True, capture_output=True).stdout
        (root / "native-working-tree.patch").write_bytes(patch)
        report["native_working_tree_patch_sha256"] = sha(root / "native-working-tree.patch")
        cases, inputs = inventory(donor)
        selected = select_cases(cases, args.case)
        extra = [repository / "upstream/lust.pin", donor / "scripts/case-pool.sh", donor / "scripts/difftest.sh"]
        extra_hashes = {str(path): sha(path) for path in extra}
        write_json(root / "manifest.json", {"all_cases": cases, "fixture_files": inputs,
                                           "additional_inputs": extra_hashes, "selected_ids": [case["id"] for case in selected]})
        report["manifest_sha256"] = sha(root / "manifest.json")
        report["suite_counts"] = dict(Counter(case["suite"] for case in cases))
        compilers = {name: snapshot_compiler(path, root, name) for name, path in (("lust", args.lust), ("native", args.native))}
        report["compilers"] = compilers
        report["upstream_package_provenance"] = package_provenance(compilers["lust"], pin, root)
        # Avoid large signal-generated core files without changing exit codes.
        core_limit = resource.getrlimit(resource.RLIMIT_CORE)
        resource.setrlimit(resource.RLIMIT_CORE, (0, core_limit[1]))
        report["core_dump_soft_limit"] = 0
        logs = root / "identity-logs"
        logs.mkdir()
        identity_cwd = root / "identity-sandbox"
        reset_sandbox(identity_cwd)
        for lane, flag in (("lust", "--version"), ("native", "-vV")):
            command = execute([compilers[lane]["executable"], flag], identity_cwd, case_environment(identity_cwd), logs, lane, 10, root)
            compilers[lane]["version_command"] = command
            output = (root / command["stdout"]).read_text(errors="replace")
            compilers[lane]["version"] = output
            prefix = "lust " if lane == "lust" else "lustr-native "
            # LUST_BUILD_HASH is a source/runtime content hash, not a git ID.
            # Versions identify the program; source provenance is separate.
            if not complete_command(command) or command["exit_code"] != 0 or not output.startswith(prefix):
                raise ValueError(f"{lane} ELF does not report the expected compiler identity")
        report["summary"] = summarize([], selected, len(cases), True)
        write_json(root / "report.json", report)
        with (root / "results.jsonl").open("w", buffering=1) as journal:
            with ThreadPoolExecutor(max_workers=args.workers) as pool:
                futures = {pool.submit(run_case, case, index, compilers, root, donor, args.compile_timeout, args.run_timeout): case
                           for index, case in enumerate(selected)}
                for future in as_completed(futures):
                    case = futures[future]
                    try:
                        result = future.result()
                    except Exception as error:
                        result = dict(case, state="unproved", issues=[f"runner exception: {type(error).__name__}: {error}"], lanes={})
                    rows.append(result)
                    journal.write(json.dumps(result, sort_keys=True) + "\n")
                    if len(rows) % 50 == 0 or len(rows) == len(selected):
                        progress = summarize(rows, selected, len(cases), True)
                        report["storage"].append({"completed_cases": len(rows), **storage_snapshot(root)})
                        write_json(root / "progress.json", progress)
                        print(json.dumps({key: progress[key] for key in ("completed_cases", "selected_cases", "counts")}), flush=True)
        stable = inventory(donor)[1] == inputs and all(sha(Path(path)) == digest for path, digest in extra_hashes.items())
        for compiler in compilers.values():
            stable &= all(sha(Path(record["executed_path"])) == record["sha256"] for record in compiler["files"])
            for path, digest in compiler["support_sha256"].items():
                stable &= sha(Path(compiler["executable"]).parent.parent / "lib/lust" / path) == digest
        report["summary"] = summarize(rows, selected, len(cases), stable)
        report["success"] = report["summary"]["success"]
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        report["failures"].append(f"{type(error).__name__}: {error}")
    finally:
        report["elapsed_ns"] = time.monotonic_ns() - start
        report["storage"].append({"completed_cases": len(rows), **storage_snapshot(root)})
        report["results"] = sorted(rows, key=lambda row: row["id"])
        write_json(root / "report.json", report)
    print(json.dumps({"report": str(root / "report.json"), "success": report["success"],
                      "qualified": False, "failures": report["failures"]}), flush=True)
    return 0 if report["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
