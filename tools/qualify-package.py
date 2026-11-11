#!/usr/bin/env python3
"""Exercise the installed package after relocation and retain raw timing records."""
from __future__ import annotations

import argparse
import hashlib
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


def clean_environment(package: Path, scratch: Path) -> dict[str, str]:
    environment = dict(os.environ)
    for name in list(environment):
        if name.startswith(("CARGO_", "RUST", "LUST_", "LUSTR_")) or name in {"CARGO", "LUSTC"}:
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
        "native_full_rust": False, "full_native_speed_qualified": False,
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
            report["source_revision"] = manifest["source_revision"]
            assert manifest["schema"] == "lustr.toolchain-package" and manifest["schema_version"] == 1
            assert manifest["native_lust"] is True and manifest["native_full_rust"] is False
            for entry in manifest["payload"]:
                path = package / entry["path"]
                if "symlink" in entry:
                    assert path.is_symlink() and os.readlink(path) == entry["symlink"]
                    assert path.resolve().is_relative_to(package)
                else:
                    assert digest(path) == entry["sha256"], f"payload mismatch: {entry['path']}"
            report["checks"].append("complete payload hashes after relocation")
            for name in ("lust", "lust-core"):
                assert digest(package / "bin" / name) == digest(baseline.with_name(name))
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
            for name in ("rustc", "cargo", "lust"):
                version = execute(f"{name}-identity", [package / "bin" / name, "--version"])
                report["compiler_identities"][name] = version.stdout.decode().strip()
            reported_root = execute("relocated-sysroot", [package / "bin/rustc", "--print", "sysroot"])
            assert Path(reported_root.stdout.decode().strip()).resolve() == package
            report["checks"].append("Rust sysroot relocates without rustup")
            source = scratch / "source with spaces" / "supported.rs"
            source.parent.mkdir()
            source.write_text(fixture_source(42))
            cold = dict(environment, LUST_NO_DAEMON="1", LUST_NO_CACHE="1",
                        LUST_CACHE_DIR=str(scratch / "cold-cache"))
            built = scratch / "native-output"
            execute("native-build", [package / "bin/lustr", "--engine", "lust", "build", source, "-o", built], cold)
            result = execute("native-run", [built])
            assert result.stdout == b"42\n"
            report["checks"].append("installed native compile and actual executable output")
            unsupported = repository / "crates/lustr/fixtures/subset/unsupported.rs"
            result = execute("native-unsupported", [package / "bin/lustr", "--engine", "lust", "check", unsupported], cold, None)
            assert result.returncode != 0 and b"L0405" in result.stderr
            invalid = scratch / "invalid.rs"
            invalid.write_text('fn main() { let value: u32 = "wrong"; println!("{}", value); }\n')
            result = execute("native-invalid", [package / "bin/lustr", "--engine", "lust", "check", invalid], cold, None)
            assert result.returncode != 0
            report["checks"].append("native unsupported and invalid input remain failures")
            full = scratch / "full Rust workspace"
            shutil.copytree(repository / "crates/lustr/fixtures/full-rust", full, ignore=shutil.ignore_patterns("target"))
            compatibility = [package / "bin/lustr", "--engine", "rustc"]
            result = execute("compatibility-run", compatibility + ["run", "--offline", "--locked", "--quiet", "-p", "fixture-app",
                                                                     "--", "two words"], cwd=full)
            expected = b"derived=41 generated=7 async=42 raw=19 gat=13 const=15 ffi=21 body=42 probe=41 panic=1 drops=1\nargument=two words\n"
            assert result.stdout == expected
            result = execute("compatibility-tests", compatibility + ["test", "--offline", "--locked", "--workspace"], cwd=full)
            assert b"features_keep_their_rust_semantics" in result.stdout and b"body_probe" in result.stdout
            invalid_full = scratch / "invalid Rust workspace"
            shutil.copytree(repository / "crates/lustr/fixtures/invalid-borrow", invalid_full, ignore=shutil.ignore_patterns("target"))
            result = execute("compatibility-rejection", compatibility + ["check", "--offline", "--locked"], expected=None, cwd=invalid_full)
            assert result.returncode != 0 and b"E0502" in result.stderr
            report["checks"].append("offline full-Rust compatibility workspace, proc macro, build script, panic/drop, tests, doctests and borrow rejection")
            for original_source, hidden_source in reversed(hidden_sources):
                hidden_source.rename(original_source)
            hidden_sources.clear()
            report["checks"].append("relocated package works while original package and native runtime/library trees are unavailable")
            lanes = {
                "pristine": [baseline],
                "packaged_native": [package / "bin/lust"],
                "packaged_lustr": [package / "bin/lustr", "--engine", "lust"],
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
                    order = order[sample % 3:] + order[:sample % 3]
                    for lane in order:
                        binary = scratch / f"{lane}-bench"
                        execute(f"{mode}-{lane}-{sample}", lanes[lane] + ["build", source, "-o", binary], environments[lane])
                        record = report["commands"][-1]
                        report["timings"].append({
                            "mode": mode, "lane": lane, "sample": sample, "elapsed_ns": record["elapsed_ns"],
                            "source_sha256": digest(source), "command_index": len(report["commands"]) - 1,
                        })
                        result = execute(f"{mode}-{lane}-{sample}-runtime", [binary])
                        assert result.stdout == f"{expected_answer}\n".encode()
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

