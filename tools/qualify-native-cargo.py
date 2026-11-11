#!/usr/bin/env python3
"""Qualify the in-process native Cargo compiler against an actual Rust compiler.

This is a bounded binary-crate contract. It never counts rustc output as native
output, and never retries a rejected native compilation through another engine.
"""
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


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def environment(scratch: Path, compiler: Path, sentinel: Path) -> dict[str, str]:
    result = dict(os.environ)
    for name in list(result):
        if name.startswith(("CARGO_", "RUST", "LUST_", "LUSTR_", "LD_", "DYLD_")) or name in {
            "CARGO", "LUSTC", "LIBPATH", "SHLIB_PATH", "LDR_PRELOAD"
        }:
            result.pop(name)
    result.update({
        "PATH": str(sentinel) + os.pathsep + "/usr/bin:/bin",
        "CARGO_HOME": str(scratch / "cargo-home"),
        "RUSTUP_HOME": str(scratch / "no-rustup"),
        "CARGO_NET_OFFLINE": "true",
        "RUSTC": str(compiler),
        "RUSTDOC": str(sentinel / "rustdoc"),
        "RUSTC_WRAPPER": "",
        "RUSTC_WORKSPACE_WRAPPER": "",
        "LUSTR_COMPILER_SENTINEL": str(scratch / "unexpected-compiler"),
    })
    return result


def module_source(answer: int, result_type: str = "u32") -> str:
    return (
        f"pub fn answer() -> {result_type} {{ {answer} }}\n"
        "pub fn path() -> &'static str { module_path!() }\n"
    )


def expected_output(answer: int = 42, included: str = "first", external: str = "unset",
                    feature: bool = False) -> bytes:
    return (
        f"answer={answer} included={included} env={external} cache_env=unset "
        "module=lustr_native_contract nested=lustr_native_contract::value "
        f"file=src/main.rs debug=true feature={str(feature).lower()}\n"
    ).encode()


def qualify(native: Path, cargo: Path, oracle: Path, fixture: Path, output: Path,
            samples: int = 6) -> dict:
    require(samples >= 4 and samples % 2 == 0, "use an even sample count of at least four")
    require(not output.exists(), "evidence output already exists")
    output.parent.mkdir(parents=True, exist_ok=True)
    native, cargo, oracle, fixture = (path.resolve() for path in (native, cargo, oracle, fixture))
    report = {
        "schema": "lustr.native-cargo-qualification.v1",
        "runner_sha256": digest(Path(__file__).resolve()),
        "platform": platform.platform(),
        "native_full_rust": False,
        "full_native_speed_qualified": False,
        "runtime_coordinator": False,
        "scope": "Dependency-free native binary crate, Cargo dep-info and supported development profile. No native rlib, test harness, or full Rust claim.",
        "timing_scope": "Paired Cargo builds of the same fixture and profile; fresh targets, unchanged inputs, body and signature edits, and invalid inputs. No daemon latency equivalence is claimed.",
        "tools": {name: {"path": str(path), "sha256": digest(path)}
                  for name, path in (("native", native), ("cargo", cargo), ("oracle", oracle))},
        "fixture_sha256": {path.relative_to(fixture).as_posix(): digest(path)
                           for path in sorted(fixture.rglob("*")) if path.is_file()},
        "commands": [], "checks": [], "timings": [], "success": False,
    }
    with tempfile.TemporaryDirectory(prefix="lustr-native-cargo-") as temporary:
        scratch = Path(temporary)
        sentinel = scratch / "compiler sentinel"
        sentinel.mkdir()
        for name in ("rustc", "rustdoc", "lust", "lust-core", "lustr", "cargo-lustr", "cargo-lust"):
            path = sentinel / name
            path.write_text('#!/bin/sh\nprintf "%s\\n" "$0" >> "$LUSTR_COMPILER_SENTINEL"\nexit 97\n')
            path.chmod(0o755)
        projects = {name: scratch / (name + " project with spaces") for name in ("native", "oracle")}
        environments = {name: environment(scratch / name, compiler, sentinel)
                        for name, compiler in (("native", native), ("oracle", oracle))}
        for name, project in projects.items():
            shutil.copytree(fixture, project)
            (scratch / name).mkdir()

        def execute(label: str, argv: list[str | Path], env: dict[str, str], cwd: Path,
                    expected: int | None = 0) -> subprocess.CompletedProcess:
            command = [str(value) for value in argv]
            start = time.monotonic_ns()
            process = subprocess.Popen(command, env=env, cwd=cwd, stdout=subprocess.PIPE,
                                       stderr=subprocess.PIPE, start_new_session=True)
            timed_out = False
            try:
                stdout, stderr = process.communicate(timeout=180)
            except subprocess.TimeoutExpired:
                timed_out = True
                os.killpg(process.pid, signal.SIGKILL)
                stdout, stderr = process.communicate()
            elapsed = time.monotonic_ns() - start
            result = subprocess.CompletedProcess(command, process.returncode, stdout, stderr)
            report["commands"].append({"label": label, "argv": command, "cwd": str(cwd),
                                       "exit_code": process.returncode, "elapsed_ns": elapsed,
                                       "stdout": stdout.decode(errors="replace"),
                                       "stderr": stderr.decode(errors="replace"), "timed_out": timed_out})
            require(not timed_out, label + " timed out")
            require(expected is None or process.returncode == expected,
                    f"{label}: expected {expected}, got {process.returncode}: {stderr.decode(errors='replace')}")
            return result

        def build(label: str, lane: str, target: Path, wanted: bytes | None,
                  flags: tuple[str, ...] = (), fresh: bool | None = None) -> tuple[int, int | None]:
            result = execute(label, [cargo, "build", "--offline", "--locked", "--message-format=json",
                                     "--target-dir", target, *flags], environments[lane], projects[lane],
                             0 if wanted is not None else 101)
            command_index = len(report["commands"]) - 1
            artifact = target / "debug/lustr_native_contract"
            if wanted is None:
                require(not artifact.exists(), label + ": invalid source produced an executable")
                return command_index, None
            events = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
            artifacts = [event for event in events if event.get("reason") == "compiler-artifact"
                         and event.get("target", {}).get("name") == "lustr_native_contract"]
            require(len(artifacts) == 1, label + ": expected one Cargo binary artifact")
            require(Path(artifacts[0]["executable"]).resolve() == artifact.resolve(),
                    label + ": Cargo did not report the real executable")
            if fresh is not None:
                require(artifacts[0]["fresh"] is fresh, label + ": incorrect Cargo freshness")
            run = execute(label + "-runtime", [artifact], environments[lane], projects[lane])
            runtime_index = len(report["commands"]) - 1
            require(run.stdout == wanted, f"{label}: wrong output {run.stdout!r}, expected {wanted!r}")
            return command_index, runtime_index

        try:
            for name, compiler in (("native", native), ("oracle", oracle)):
                result = execute(name + "-identity", [compiler, "-vV"], environments[name], projects[name])
                report["tools"][name]["version"] = result.stdout.decode().strip()
            require("lustr-native" in report["tools"]["native"]["version"], "native compiler identity is missing")
            targets = {name: scratch / (name + " integration target") for name in projects}
            for lane in projects:
                build(lane + "-initial", lane, targets[lane], expected_output(), fresh=False)
                build(lane + "-fresh", lane, targets[lane], expected_output(), fresh=True)
                (projects[lane] / "src/value.rs").write_text(module_source(43))
                build(lane + "-module-edit", lane, targets[lane], expected_output(43), fresh=False)
                (projects[lane] / "data.txt").write_text("second\n")
                build(lane + "-include-edit", lane, targets[lane], expected_output(43, "second"), fresh=False)
                environments[lane]["LUSTR_NATIVE_INPUT"] = "first"
                build(lane + "-env-set", lane, targets[lane], expected_output(43, "second", "first"), fresh=False)
                environments[lane]["LUSTR_NATIVE_INPUT"] = "second"
                build(lane + "-env-change", lane, targets[lane], expected_output(43, "second", "second"), fresh=False)
                environments[lane].pop("LUSTR_NATIVE_INPUT")
                build(lane + "-env-unset", lane, targets[lane], expected_output(43, "second"), fresh=False)
                build(lane + "-feature-on", lane, targets[lane], expected_output(43, "second", feature=True),
                      ("--features", "special"), fresh=False)
                build(lane + "-feature-off", lane, targets[lane], expected_output(43, "second"), fresh=False)
            report["checks"].extend([
                "Cargo selects the in-process native compiler directly",
                "real executables match the Rust oracle, including crate name, nested module_path and file",
                "unchanged Cargo build is fresh; module, included-file, environment set/change/unset and feature edits invalidate correctly",
            ])

            # These invocations must fail, without publishing pretend library or metadata artifacts.
            reject_root = scratch / "unsupported requests"
            reject_root.mkdir()
            source = reject_root / "source.rs"
            source.write_text("pub fn answer() -> u32 { 42 }\n")
            for label, flags in (
                ("rlib", ["--crate-type", "rlib", "--emit=link"]),
                ("metadata", ["--crate-type", "bin", "--emit=metadata"]),
                ("optimized-profile", ["--crate-type", "bin", "--emit=link", "-C", "opt-level=3"]),
                ("foreign-target", ["--crate-type", "bin", "--emit=link", "--target", "aarch64-unknown-linux-gnu"]),
                ("malformed-check-cfg", ["--crate-type", "bin", "--emit=link", "--check-cfg", "cfg(feature, values("]),
                ("malformed-version", ["--version=garbage"]),
            ):
                artifact = reject_root / label
                result = execute("native-reject-" + label,
                                 [native, "--crate-name", "source", "--edition=2024", source, "-o", artifact, *flags],
                                 environments["native"], reject_root, None)
                require(result.returncode == 2, label + ": expected explicit unsupported-invocation exit 2")
                require(not artifact.exists(), label + ": unsupported invocation produced an artifact")
            report["checks"].append("unsupported rlib, metadata, optimization and foreign targets, plus malformed options, fail without fake artifacts")

            report["invalid_sources"] = {}
            for case in ("invalid-type", "invalid-move", "invalid-alias"):
                invalid = fixture.parent / "native-qualification" / (case + ".rs")
                text = invalid.read_text()
                report["invalid_sources"][case] = {"sha256": digest(invalid), "source": text}
                for lane, project in projects.items():
                    (project / "src/main.rs").write_text(text)
                    build(case + "-" + lane, lane, scratch / (lane + " " + case), None)
            report["checks"].append("type, move and aliasing errors remain compilation failures in both compilers")

            original_main = (fixture / "src/main.rs").read_text()
            for mode in ("fresh-target", "no-change", "body-edit", "signature-edit", "invalid-edit"):
                mode_targets = {lane: scratch / (lane + " " + mode) for lane in projects}
                for lane, project in projects.items():
                    (project / "src/main.rs").write_text(original_main)
                    (project / "src/value.rs").write_text(module_source(42))
                    (project / "data.txt").write_text("first\n")
                    if mode in ("no-change", "body-edit", "signature-edit"):
                        build(mode + "-" + lane + "-warmup", lane, mode_targets[lane], expected_output())
                for sample in range(samples):
                    order = ("native", "oracle") if sample % 2 == 0 else ("oracle", "native")
                    answer = 43 + sample if mode in ("body-edit", "signature-edit") else 42
                    for lane in order:
                        project = projects[lane]
                        if mode in ("body-edit", "signature-edit"):
                            kind = "u64" if mode == "signature-edit" and sample % 2 == 0 else "u32"
                            (project / "src/value.rs").write_text(module_source(answer, kind))
                        if mode == "invalid-edit":
                            (project / "src/main.rs").write_text(
                                f'fn main() {{ let value: u32 = "invalid-{sample}"; println!("{{}}", value); }}\n')
                        target = mode_targets[lane]
                        if mode in ("fresh-target", "invalid-edit"):
                            target = target / str(sample)
                        wanted = None if mode == "invalid-edit" else expected_output(answer)
                        command_index, runtime_index = build(
                            mode + "-" + lane + "-" + str(sample), lane, target, wanted,
                            fresh=True if mode == "no-change" else False if wanted is not None else None)
                        record = report["commands"][command_index]
                        report["timings"].append({
                            "mode": mode, "lane": lane, "sample": sample,
                            "elapsed_ns": record["elapsed_ns"], "command_index": command_index,
                            "runtime_command_index": runtime_index,
                            "compile_plus_run_ns": record["elapsed_ns"] + report["commands"][runtime_index]["elapsed_ns"]
                            if runtime_index is not None else None,
                            "source_sha256": {name: digest(project / name)
                                              for name in ("src/main.rs", "src/value.rs", "data.txt", "Cargo.toml")},
                        })
            report["checks"].append("balanced paired Cargo timings retain correct outputs, signature/body rebuilds and invalid-program rejection")
            for lane in projects:
                require(not (scratch / lane / "unexpected-compiler").exists(),
                        lane + ": unexpected compiler fallback or wrapper invocation")
            report["checks"].append("no compiler executable fallback; compiler-name sentinels remained unused")
            report["summaries"] = []
            for mode in sorted({row["mode"] for row in report["timings"]}):
                rows = [row for row in report["timings"] if row["mode"] == mode]
                oracle_times = {row["sample"]: row["elapsed_ns"] for row in rows if row["lane"] == "oracle"}
                for lane in projects:
                    selected = [row for row in rows if row["lane"] == lane]
                    report["summaries"].append({
                        "mode": mode, "lane": lane, "samples": len(selected),
                        "median_ns": statistics.median(row["elapsed_ns"] for row in selected),
                        "median_paired_ratio_to_oracle": statistics.median(
                            row["elapsed_ns"] / oracle_times[row["sample"]] for row in selected),
                    })
            report["success"] = True
        except BaseException as error:
            report["failure"] = f"{type(error).__name__}: {error}"
            raise
        finally:
            output.write_text(json.dumps(report, indent=2) + "\n")
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--cargo", type=Path, required=True)
    parser.add_argument("--oracle-rustc", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, default=Path(__file__).resolve().parents[1] / "tests/native-cargo")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=6)
    args = parser.parse_args()
    result = qualify(args.native, args.cargo, args.oracle_rustc, args.fixture, args.output, args.samples)
    print(json.dumps({"success": result["success"], "checks": result["checks"],
                      "native_full_rust": False, "summaries": result["summaries"]}))


if __name__ == "__main__":
    main()
