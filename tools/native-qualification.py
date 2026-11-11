#!/usr/bin/env python3
"""Record native Lust coverage separately from the Rust compatibility oracle."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import sys
import time


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def execute(argv, directory, env, timeout, prefix):
    started = time.perf_counter_ns()
    timed_out = False
    spawn_error = None
    try:
        process = subprocess.Popen(
            [str(item) for item in argv], cwd=directory, env=env,
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, start_new_session=(os.name == "posix"),
        )
        try:
            stdout, stderr = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            if os.name == "posix":
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
            else:
                process.kill()
            stdout, stderr = process.communicate()
        exit_code = process.returncode
    except OSError as error:
        stdout, stderr = b"", str(error).encode("utf-8")
        exit_code, spawn_error = None, type(error).__name__
    elapsed = time.perf_counter_ns() - started
    prefix.with_suffix(".stdout").write_bytes(stdout)
    prefix.with_suffix(".stderr").write_bytes(stderr)
    record = {
        "argv": [str(item) for item in argv], "cwd": str(directory),
        "exit_code": exit_code, "timed_out": timed_out,
        "spawn_error": spawn_error, "elapsed_ns": elapsed,
        "stdout_file": prefix.with_suffix(".stdout").name,
        "stderr_file": prefix.with_suffix(".stderr").name,
        "stdout_sha256": hashlib.sha256(stdout).hexdigest(),
        "stderr_sha256": hashlib.sha256(stderr).hexdigest(),
    }
    prefix.with_suffix(".json").write_text(json.dumps(record, indent=2) + "\n")
    return record, stdout


def succeeded(record):
    return record["exit_code"] == 0 and not record["timed_out"]


def rejected(record, artifact):
    return record["exit_code"] == 1 and not record["timed_out"] and not artifact.exists()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lust", required=True, type=Path)
    parser.add_argument("--rustc", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--timeout", default=90, type=int)
    parser.add_argument("--require-all-native", action="store_true")
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")
    compilers = {"native": args.lust.absolute(), "oracle": args.rustc.absolute()}
    for compiler in compilers.values():
        if not compiler.is_file() or not os.access(compiler, os.X_OK):
            parser.error(f"compiler is not executable: {compiler}")
    fixtures = Path(__file__).resolve().parents[1] / "tests/native-qualification"
    manifest = fixtures / "cases.json"
    cases = json.loads(manifest.read_text())
    output = args.output.absolute()
    if output.exists():
        parser.error("--output must not exist; previous evidence is never overwritten")
    output.mkdir(parents=True)
    env = os.environ.copy()
    for key in list(env):
        if key.startswith(("LD_", "DYLD_")) or key in ("LIBPATH", "SHLIB_PATH", "LDR_PRELOAD"):
            env.pop(key)
    for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER",
                "RUSTC_WORKSPACE_WRAPPER", "LUST_STD_SRC", "LUST_LIBRARY",
                "LUST_FRONT_STATE", "LUST_DAEMON_SOCKET"):
        env.pop(key, None)
    env["LUST_NO_DAEMON"] = "1"
    env["CARGO_NET_OFFLINE"] = "true"
    env["PATH"] = str(compilers["oracle"].parent) + os.pathsep + env.get("PATH", "")
    report = {
        "schema": "lustr.native-qualification.v1",
        "host": {"system": platform.system(), "machine": platform.machine()},
        "fixture_manifest_sha256": sha256(manifest),
        "runner_sha256": sha256(Path(__file__)),
        "configuration": {
            "rust_edition": "2024", "panic": "unwind", "overflow_checks": True,
            "debug_assertions": True, "native_daemon": False,
            "native_cache": "fresh directory per case", "timeout_seconds": args.timeout,
        },
        "compilers": {}, "cases": [],
        "scope": "Independent single-file and library fixtures, not full Rust qualification",
        "timing_scope": "One recorded cold observation per command; no speedup conclusion",
        "unassessed": ["arbitrary Cargo projects", "procedural macros", "build scripts",
                       "cross-crate native artifact compatibility", "warm edit invalidation",
                       "other targets", "full Rust test suite", "performance preservation"],
        "full_native_coverage": False,
    }
    for lane, compiler in compilers.items():
        identity, _ = execute([compiler, "--version" if lane == "native" else "-vV"],
                              output, env, args.timeout, output / f"{lane}-identity")
        components = {compiler.name: sha256(compiler)}
        if lane == "native":
            core = compiler.with_name("lust-core")
            if core.is_file():
                components["lust-core"] = sha256(core)
        report["compilers"][lane] = {"path": str(compiler), "sha256": sha256(compiler),
                                    "components": components, "identity": identity}
    for case in cases:
        source = fixtures / case["source"]
        case_dir = output / case["name"]
        case_dir.mkdir()
        source_copy = case_dir / "input.rs"
        shutil.copyfile(source, source_copy)
        result = {"name": case["name"], "category": case["category"],
                  "source": case["source"], "source_sha256": sha256(source_copy),
                  "kind": case.get("kind", "binary"), "lanes": {}}
        if result["kind"] == "library":
            consumer_source = case_dir / "consumer.rs"
            shutil.copyfile(fixtures / case["consumer"], consumer_source)
            result["consumer_source_sha256"] = sha256(consumer_source)
        case_env = env.copy()
        case_env["LUST_CACHE_DIR"] = str(case_dir / "native-cache")
        for lane in ("native", "oracle"):
            artifact = case_dir / (f"lib{lane}.rlib" if result["kind"] == "library" else f"{lane}.bin")
            if lane == "native":
                command = [compilers[lane], "build", source_copy, "-o", artifact,
                           "-C", "panic=unwind", "-C", "overflow-checks=on",
                           "-C", "debug-assertions=on"]
                if result["kind"] == "library":
                    cargo_manifest = case_dir / "Cargo.toml"
                    cargo_manifest.write_text('[package]\nname = "lustr_native_fixture"\n'
                                              'version = "0.1.0"\nedition = "2024"\n'
                                              '[lib]\npath = "input.rs"\n[workspace]\n')
                    command += ["--manifest-path", cargo_manifest, "--lib", "--offline"]
            else:
                command = [compilers[lane], source_copy, "--edition=2024", "-o", artifact,
                           "-C", "panic=unwind", "-C", "overflow-checks=on",
                           "-C", "debug-assertions=on"]
                if result["kind"] == "library":
                    command += ["--crate-type=rlib", "--crate-name=lustr_native_fixture"]
            build, _ = execute(command, case_dir, case_env, args.timeout, case_dir / f"{lane}-build")
            observation = {"build": build, "artifact_present": artifact.is_file()}
            if artifact.is_file():
                observation["artifact_sha256"] = sha256(artifact)
                observation["artifact_bytes"] = artifact.stat().st_size
            if case["category"] == "invalid":
                observation["passed"] = rejected(build, artifact)
            elif succeeded(build) and artifact.is_file() and artifact.stat().st_size > 0:
                if result["kind"] == "library":
                    consumer_binary = case_dir / f"{lane}-consumer.bin"
                    consumer_build, _ = execute(
                        [compilers["oracle"], consumer_source, "--edition=2024", "--extern",
                         f"lustr_native_fixture={artifact}", "-o", consumer_binary,
                         "-C", "panic=unwind", "-C", "overflow-checks=on",
                         "-C", "debug-assertions=on"],
                        case_dir, case_env, args.timeout, case_dir / f"{lane}-consumer-build")
                    observation["consumer_build"] = consumer_build
                    observation["passed"] = False
                    if succeeded(consumer_build) and consumer_binary.is_file():
                        run, stdout = execute([consumer_binary], case_dir, case_env, args.timeout,
                                              case_dir / f"{lane}-run")
                        observation["run"] = run
                        observation["passed"] = succeeded(run) and stdout == case["stdout"].encode()
                else:
                    run, stdout = execute([artifact], case_dir, case_env, args.timeout,
                                          case_dir / f"{lane}-run")
                    observation["run"] = run
                    observation["passed"] = succeeded(run) and stdout == case["stdout"].encode()
            else:
                observation["passed"] = False
            result["lanes"][lane] = observation
        if not result["lanes"]["oracle"]["passed"]:
            result["status"] = "oracle_or_fixture_failure"
        elif result["lanes"]["native"]["passed"]:
            result["status"] = "native_rejection_pass" if case["category"] == "invalid" else "native_fixture_pass"
        elif case["category"] == "invalid":
            result["status"] = "native_invalid_program_gate_failure"
        elif not succeeded(result["lanes"]["native"]["build"]):
            result["status"] = "native_compile_failure_on_valid_rust"
        else:
            result["status"] = "native_artifact_or_runtime_mismatch"
        report["cases"].append(result)
        print(f'{case["name"]}: {result["status"]}', flush=True)
    identity_passed = all(succeeded(entry["identity"]) for entry in report["compilers"].values())
    def admissible_native_result(case):
        native = case["lanes"]["native"]
        if native["passed"]:
            return True
        # A known coverage gap may reject valid Rust. Crashes, timeouts,
        # unusable commands, and accepted miscompilations fail this gate.
        return (case["category"] == "expansion"
                and native["build"]["exit_code"] == 1
                and not native["build"]["timed_out"]
                and not native["artifact_present"])

    required_passed = identity_passed and all(
        case["lanes"]["oracle"]["passed"] and admissible_native_result(case)
        for case in report["cases"]
    )
    all_native = all(case["lanes"]["native"]["passed"] for case in report["cases"])
    report["required_fixture_gates_passed"] = required_passed
    report["all_native_fixtures_passed"] = all_native
    report["native_gaps"] = [case["name"] for case in report["cases"]
                             if case["category"] == "expansion"
                             and case["lanes"]["oracle"]["passed"]
                             and not case["lanes"]["native"]["passed"]]
    report["result"] = "pass" if required_passed and (all_native or not args.require_all_native) else "fail"
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"result": report["result"], "native_gaps": report["native_gaps"],
                      "full_native_coverage": False, "report": str(output / "report.json")}))
    return 0 if report["result"] == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
