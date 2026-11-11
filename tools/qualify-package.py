#!/usr/bin/env python3
"""Exercise the installed package after relocation and retain raw timing records."""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import statistics
import subprocess
import tempfile
import time


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def clean_environment(package: Path, scratch: Path) -> dict[str, str]:
    environment = dict(os.environ)
    for name in list(environment):
        if name.startswith(("CARGO_", "RUST", "LUST_", "LUSTR_", "LD_", "DYLD_")) or name in {
            "CARGO", "LUSTC", "LIBPATH", "SHLIB_PATH", "LDR_PRELOAD"
        }:
            environment.pop(name)
    environment.update({
        "PATH": str(package / "bin") + os.pathsep + "/usr/bin:/bin",
        "CARGO_HOME": str(scratch / "cargo-home"),
        "RUSTUP_HOME": str(scratch / "empty-rustup-home"),
        "CARGO_NET_OFFLINE": "true",
    })
    return environment


def fixture_source(answer: int) -> str:
    return (
        "fn add(left: u32, right: u32) -> u32 { left + right }\n"
        f'fn main() {{ println!("{{}}", add(20, {answer - 20})); }}\n'
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--repository", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--baseline-native", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=7)
    args = parser.parse_args()
    if args.samples < 3:
        parser.error("at least three paired observations are required")
    if args.output.exists():
        parser.error("output already exists; retain it and select a new evidence path")
    repository = args.repository.resolve()
    baseline = args.baseline_native.resolve()
    original = args.package.resolve()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    report = {
        "schema": "lustr.package-qualification", "schema_version": 1,
        "runner_sha256": digest(Path(__file__).resolve()),
        "native_full_rust": False, "full_native_speed_qualified": False,
        "runtime_coordinator": False,
        "platform": platform.platform(), "cpu_count": os.cpu_count(),
        "source_revision": None, "commands": [], "timings": [],
        "checks": [], "success": False,
        "timing_scope": "One supported source; paired process cold, daemon no-op and body-edit compilation. Runtime checked separately. No universal speed conclusion.",
    }
    daemons: list[subprocess.Popen] = []
    log_streams = []
    hidden_sources: list[tuple[Path, Path]] = []
    hidden_package: Path | None = None
    with tempfile.TemporaryDirectory(prefix="lustr-package-") as temporary:
        scratch = Path(temporary)
        native_cargo_evidence = scratch / "native-cargo-qualification.json"
        package = scratch / "relocated package"
        shutil.copytree(original, package, symlinks=True)
        environment = clean_environment(package, scratch)

        def execute(label: str, argv: list[str | Path], env: dict[str, str] | None = None,
                    expected: int | None = 0, cwd: Path | None = None) -> subprocess.CompletedProcess:
            command = [str(item) for item in argv]
            start = time.monotonic_ns()
            process = subprocess.Popen(command, cwd=cwd or scratch, env=env or environment,
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
            timed_out = False
            try:
                stdout, stderr = process.communicate(timeout=180)
            except subprocess.TimeoutExpired:
                timed_out = True
                os.killpg(process.pid, signal.SIGKILL)
                stdout, stderr = process.communicate()
            result = subprocess.CompletedProcess(command, process.returncode, stdout, stderr)
            elapsed = time.monotonic_ns() - start
            report["commands"].append({
                "label": label, "argv": command, "exit_code": result.returncode, "elapsed_ns": elapsed,
                "stdout": result.stdout.decode("utf-8", errors="replace"),
                "stderr": result.stderr.decode("utf-8", errors="replace"),
                "timed_out": timed_out,
            })
            if timed_out:
                raise TimeoutError(f"{label}: exceeded 180 seconds; complete invocation record retained")
            if expected is not None and result.returncode != expected:
                raise AssertionError(f"{label}: exit {result.returncode}, expected {expected}; {result.stderr.decode(errors='replace')}")
            return result

        try:
            manifest = json.loads((package / "lustr-toolchain.json").read_text())
            report["package_manifest_sha256"] = digest(package / "lustr-toolchain.json")
            report["source_revision"] = manifest["source_revision"]
            require(manifest["schema"] == "lustr.toolchain-package" and manifest["schema_version"] == 1, "unsupported package schema")
            require(manifest["native_lust"] is True and manifest["native_full_rust"] is False, "incorrect package coverage claims")
            require(manifest.get("runtime_coordinator") is False, "package must use Cargo selection without a coordinator")
            require(manifest.get("native_cargo", {}).get("executable") == "bin/lustr-native"
                    and manifest["native_cargo"].get("native_full_rust") is False,
                    "package must identify the bounded in-process native compiler")
            for name in ("lustr", "cargo-lustr", "cargo-lust"):
                path = package / "bin" / name
                require(not path.exists() and not path.is_symlink(), "unexpected coordinator in package: " + name)
            report["checks"].append("no coordinator executable in the package")
            for entry in manifest["payload"]:
                path = package / entry["path"]
                if "symlink" in entry:
                    require(path.is_symlink() and os.readlink(path) == entry["symlink"], "package symlink differs from manifest")
                    require(path.resolve().is_relative_to(package), "package symlink escapes root")
                else:
                    require(digest(path) == entry["sha256"], f"payload mismatch: {entry['path']}")
            report["checks"].append("complete payload hashes after relocation")
            for name in ("lust", "lust-core"):
                require(digest(package / "bin" / name) == digest(baseline.with_name(name)), "native compiler differs from pristine build")
            report["checks"].append("packaged native executables identical to pristine release build")
            hidden_package = original.with_name(original.name + ".qualification-hidden")
            if hidden_package.exists():
                raise FileExistsError(hidden_package)
            original.rename(hidden_package)
            for path in (baseline.parents[2] / "library", baseline.parents[2] / "third_party/tcc"):
                hidden = path.with_name(path.name + ".qualification-hidden")
                if hidden.exists():
                    raise FileExistsError(hidden)
                path.rename(hidden)
                hidden_sources.append((path, hidden))
            report["compiler_identities"] = {}
            for name in ("rustc", "rustdoc", "cargo", "lust", "lustr-native"):
                version = execute(f"{name}-identity", [package / "bin" / name, "--version"])
                report["compiler_identities"][name] = version.stdout.decode().strip()
            reported_root = execute("relocated-sysroot", [package / "bin/rustc", "--print", "sysroot"])
            require(Path(reported_root.stdout.decode().strip()).resolve() == package, "compiler did not select relocated sysroot")
            report["checks"].append("Rust sysroot relocates without rustup")
            source = scratch / "source with spaces" / "supported.rs"
            source.parent.mkdir()
            source.write_text(fixture_source(42))
            cold = dict(environment, LUST_NO_DAEMON="1", LUST_NO_CACHE="1",
                        LUST_CACHE_DIR=str(scratch / "cold-cache"))
            built = scratch / "native-output"
            execute("native-build", [package / "bin/lust", "build", source, "-o", built], cold)
            result = execute("native-run", [built])
            require(result.stdout == b"42\n", "native runtime output differs")
            report["checks"].append("installed native compile and actual executable output")
            (scratch / "input.txt").write_text("packaged library works")
            library_source = scratch / "read-library.rs"
            library_source.write_text('fn main() { let text = std::fs::read_to_string("input.txt").unwrap(); println!("{}", text); }\n')
            library_binary = scratch / "read-library"
            execute("native-library-build", [package / "bin/lust", "build", library_source, "-o", library_binary], cold)
            result = execute("native-library-run", [library_binary])
            require(result.stdout == b"packaged library works\n", "packaged source library did not preserve filesystem behavior")
            report["checks"].append("native filesystem operation through relocated library assets")
            unsupported = repository / "crates/lustr/fixtures/subset/unsupported.rs"
            result = execute("native-unsupported", [package / "bin/lust", "check", unsupported], cold, None)
            require(result.returncode == 1 and b"error[L0405]" in result.stderr, "native unsupported case did not reject cleanly")
            invalid = scratch / "invalid.rs"
            invalid.write_text('fn main() { let value: u32 = "wrong"; println!("{}", value); }\n')
            result = execute("native-invalid", [package / "bin/lust", "check", invalid], cold, None)
            require(result.returncode == 1 and b"error[L0110]" in result.stderr, "native type error did not reject cleanly")
            report["checks"].append("native unsupported and invalid input remain failures")
            full = scratch / "full Rust workspace"
            shutil.copytree(repository / "crates/lustr/fixtures/full-rust", full, ignore=shutil.ignore_patterns("target"))
            compatibility = [package / "bin/cargo"]
            compatibility_environment = dict(environment, RUSTC=str(package / "bin/rustc"),
                                             RUSTDOC=str(package / "bin/rustdoc"))
            report["compiler_selection"] = {
                "cargo": str(package / "bin/cargo"),
                "rustc": compatibility_environment["RUSTC"],
                "rustdoc": compatibility_environment["RUSTDOC"],
                "mechanism": "Cargo RUSTC/RUSTDOC environment selection",
            }
            result = execute("compatibility-run", compatibility + ["run", "--offline", "--locked", "--quiet", "-p", "fixture-app",
                                                                     "--", "two words"], compatibility_environment, cwd=full)
            expected = b"derived=41 generated=7 async=42 raw=19 gat=13 const=15 ffi=21 body=42 probe=41 panic=1 drops=1\nargument=two words\n"
            require(result.stdout == expected, "compatibility workspace runtime output differs")
            result = execute("compatibility-tests", compatibility + ["test", "--offline", "--locked", "--workspace"],
                             compatibility_environment, cwd=full)
            require(b"features_keep_their_rust_semantics" in result.stdout and b"body_probe" in result.stdout, "expected unit test and doctest did not execute")
            invalid_full = scratch / "invalid Rust workspace"
            shutil.copytree(repository / "crates/lustr/fixtures/invalid-borrow", invalid_full, ignore=shutil.ignore_patterns("target"))
            result = execute("compatibility-rejection", compatibility + ["check", "--offline", "--locked"],
                             compatibility_environment, expected=None, cwd=invalid_full)
            require(result.returncode == 101 and b"E0502" in result.stderr, "compatibility borrow error did not reject cleanly")
            report["checks"].append("offline full-Rust compatibility workspace, proc macro, build script, panic/drop, tests, doctests and borrow rejection")
            native_cargo_runner = repository / "tools/qualify-native-cargo.py"
            spec = importlib.util.spec_from_file_location("qualify_native_cargo", native_cargo_runner)
            require(spec is not None and spec.loader is not None, "cannot load native Cargo qualification")
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            module.qualify(package / "bin/lustr-native", package / "bin/cargo", package / "bin/rustc",
                           repository / "tests/native-cargo", native_cargo_evidence)
            report["checks"].append("relocated in-process native Cargo binary compiler, real dep-info, external-input invalidation and explicit unsupported-artifact rejection")
            for original_source, hidden_source in reversed(hidden_sources):
                hidden_source.rename(original_source)
            hidden_sources.clear()
            report["checks"].append("relocated package works while original package and native runtime/library trees are unavailable")
            lanes = {
                "pristine": [baseline],
                "packaged_native": [package / "bin/lust"],
            }
            for mode in ("process-cold", "daemon-noop", "daemon-body-edit"):
                environments = {}
                source.write_text(fixture_source(42))
                for lane in lanes:
                    lane_environment = dict(environment, LUST_NO_CACHE="1",
                                            LUST_CACHE_DIR=str(scratch / f"{mode}-{lane}-cache"))
                    if mode == "process-cold":
                        lane_environment["LUST_NO_DAEMON"] = "1"
                    else:
                        socket = scratch / f"{mode}-{lane}.sock"
                        lane_environment.update(LUST_DAEMON_SOCKET=str(socket), LUST_DAEMON_IDLE="120")
                        core = baseline.with_name("lust-core") if lane == "pristine" else package / "bin/lust-core"
                        log = (scratch / f"{mode}-{lane}.log").open("wb")
                        log_streams.append(log)
                        daemon = subprocess.Popen([str(core), "daemon"], env=lane_environment,
                                                  stdout=log, stderr=log, cwd=scratch)
                        daemons.append(daemon)
                        deadline = time.monotonic() + 15
                        while not socket.exists():
                            if daemon.poll() is not None or time.monotonic() >= deadline:
                                raise AssertionError(f"{lane} daemon did not become ready")
                            time.sleep(0.01)
                    environments[lane] = lane_environment
                    execute(f"{mode}-{lane}-warmup", lanes[lane] + ["build", source, "-o", scratch / f"{lane}-bench"],
                            lane_environment)
                for sample in range(args.samples):
                    expected_answer = 43 + sample if mode == "daemon-body-edit" else 42
                    if mode == "daemon-body-edit":
                        source.write_text(fixture_source(expected_answer))
                    order = list(lanes)
                    order = order[sample % len(order):] + order[:sample % len(order)]
                    for lane in order:
                        binary = scratch / f"{lane}-bench"
                        execute(f"{mode}-{lane}-{sample}", lanes[lane] + ["build", source, "-o", binary], environments[lane])
                        record = report["commands"][-1]
                        report["timings"].append({
                            "mode": mode, "lane": lane, "sample": sample, "elapsed_ns": record["elapsed_ns"],
                            "source_sha256": digest(source), "command_index": len(report["commands"]) - 1,
                        })
                        result = execute(f"{mode}-{lane}-{sample}-runtime", [binary])
                        require(result.stdout == f"{expected_answer}\n".encode(), "timed native compilation produced wrong output")
                for daemon in daemons:
                    if daemon.poll() is None:
                        daemon.terminate()
                        daemon.wait(timeout=15)
                daemons.clear()
            summaries = []
            for mode in ("process-cold", "daemon-noop", "daemon-body-edit"):
                rows = [row for row in report["timings"] if row["mode"] == mode]
                reference = {row["sample"]: row["elapsed_ns"] for row in rows if row["lane"] == "pristine"}
                for lane in lanes:
                    timings = [row for row in rows if row["lane"] == lane]
                    summaries.append({
                        "mode": mode, "lane": lane, "samples": len(timings),
                        "median_ns": statistics.median(row["elapsed_ns"] for row in timings),
                        "median_paired_ratio_to_pristine": statistics.median(row["elapsed_ns"] / reference[row["sample"]] for row in timings),
                    })
            report["summaries"] = summaries
            report["checks"].append("raw paired native cold, daemon no-op and body-edit results with runtime output checks")
            report["success"] = True
        except BaseException as error:
            report["failure"] = f"{type(error).__name__}: {error}"
            raise
        finally:
            if native_cargo_evidence.exists():
                report["native_cargo_qualification"] = json.loads(native_cargo_evidence.read_text())
            for original_source, hidden_source in reversed(hidden_sources):
                hidden_source.rename(original_source)
            if hidden_package is not None and hidden_package.exists() and not original.exists():
                hidden_package.rename(original)
            for daemon in daemons:
                if daemon.poll() is None:
                    daemon.terminate()
                    try:
                        daemon.wait(timeout=15)
                    except subprocess.TimeoutExpired:
                        daemon.kill()
                        daemon.wait()
            for log in log_streams:
                log.close()
            report["daemon_logs"] = {
                path.name: path.read_text(errors="replace")
                for path in scratch.glob("*.log")
            }
            args.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"success": report["success"], "checks": report["checks"], "native_full_rust": False}))


if __name__ == "__main__":
    main()

