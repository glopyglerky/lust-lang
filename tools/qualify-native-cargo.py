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


def clear_source_cache(project: Path, scratch: Path) -> dict:
    """Remove only a fixture project's owned, source-adjacent native cache."""
    require(project.resolve().is_relative_to(scratch.resolve()), "cache reset requires an owned temporary project")
    cache = project / "src/.lust-cache"
    require(not cache.is_symlink() and cache.resolve().is_relative_to(project.resolve()),
            "native source cache must stay within its owned temporary project")
    existed = cache.exists()
    if existed:
        require(cache.is_dir(), "native source cache is not a directory")
        shutil.rmtree(cache)
    require(not cache.exists(), "native source cache remains after reset")
    return {"path": str(cache), "existed_before": existed, "empty_before_compile": True}


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
        "timing_scope": "Paired Cargo builds of the same fixture and profile; fresh targets and native unit caches, unchanged inputs, body and signature edits, and cold invalid inputs. No daemon latency equivalence is claimed.",
        "cache_policy": {
            "fresh-target": "New Cargo target and removal of only the owned source-adjacent .lust-cache before every sample; reset time excluded.",
            "cold-invalid": "New Cargo target and empty native unit cache before each invalid input; no preceding valid build, so this measures cold rejection.",
            "no-change": "Existing Cargo target and native unit cache; Cargo must report its artifact fresh without invoking the compiler.",
            "body-edit": "Existing Cargo target and native unit cache after a valid warmup; each changed body must rebuild.",
            "signature-edit": "Existing Cargo target and native unit cache after a valid warmup; each changed signature must rebuild.",
            "system_state": "OS page caches are not flushed; fresh refers to compiler artifact caches, not a cold host.",
        },
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
                  flags: tuple[str, ...] = (), fresh: bool | None = None,
                  project: Path | None = None,
                  env: dict[str, str] | None = None) -> tuple[int, int | None]:
            work_project = projects[lane] if project is None else project
            work_env = environments[lane] if env is None else env
            result = execute(label, [cargo, "build", "--offline", "--locked", "--message-format=json",
                                     "--target-dir", target, *flags], work_env, work_project,
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
            run = execute(label + "-runtime", [artifact], work_env, work_project)
            runtime_index = len(report["commands"]) - 1
            require(run.stdout == wanted, f"{label}: wrong output {run.stdout!r}, expected {wanted!r}")
            return command_index, runtime_index

        try:
            cargo_identity = execute("cargo-identity", [cargo, "-Vv"], environments["native"], projects["native"])
            report["tools"]["cargo"]["version"] = cargo_identity.stdout.decode().strip()
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
                # Cargo retains distinct feature/profile variants. Returning to
                # an already built variant should reuse its valid artifact.
                build(lane + "-feature-off", lane, targets[lane], expected_output(43, "second"), fresh=True)
                environments[lane]["CARGO_INCREMENTAL"] = "1"
                build(lane + "-incremental-enabled", lane, targets[lane], expected_output(43, "second"), fresh=False)
                build(lane + "-incremental-fresh", lane, targets[lane], expected_output(43, "second"), fresh=True)
                environments[lane].pop("CARGO_INCREMENTAL")
                build(lane + "-incremental-disabled", lane, targets[lane], expected_output(43, "second"), fresh=True)
            report["checks"].extend([
                "Cargo selects the in-process native compiler directly",
                "real executables match the Rust oracle, including crate name, nested module_path and file",
                "unchanged Cargo build is fresh; module, included-file and environment set/change/unset edits invalidate; a new feature variant builds and the previous variant remains reusable",
                "Cargo incremental selection stays fresh without exposing the compiler's internal cache directory to application macros",
            ])

            manifest_text = (fixture / "Cargo.toml").read_text()
            default_manifest, separator, profile_tail = manifest_text.partition("\n[profile.dev]\n")
            require(bool(separator) and not any(line.lstrip().startswith("[") for line in profile_tail.splitlines()),
                    "default-profile fixture requires the known final profile.dev table")
            default_manifest += "\n"
            report["default_development_profile"] = {
                "manifest": default_manifest,
                "environment_policy": {
                    "scenario": "ordinary development outside CI",
                    "CI": "unset", "TF_BUILD": "unset", "CARGO_INCREMENTAL": "unset",
                    "reason": "Cargo disables default incremental compilation when CI or TF_BUILD is present. This scenario removes those markers for both compiler lanes; explicit incremental selection is qualified separately.",
                },
                "lanes": {},
            }
            for lane in projects:
                # Cargo's is_ci() checks presence, so even CI=false would select
                # its CI defaults. Change only this non-CI development scenario.
                default_env = dict(environments[lane])
                removed_markers = [name for name in ("CI", "TF_BUILD") if name in default_env]
                for name in ("CI", "TF_BUILD"):
                    default_env.pop(name, None)
                require("CARGO_INCREMENTAL" not in default_env,
                        "default development scenario must not force incremental selection")
                default_project = scratch / (lane + " default development profile")
                shutil.copytree(fixture, default_project)
                (default_project / "Cargo.toml").write_text(default_manifest)
                default_target = scratch / (lane + " default development target")
                build_index, runtime_index = build(lane + "-default-dev-build", lane, default_target,
                                                   expected_output(), fresh=False, project=default_project,
                                                   env=default_env)
                events = [json.loads(line) for line in report["commands"][build_index]["stdout"].splitlines()
                          if line.strip()]
                profile = next(event["profile"] for event in events if event.get("reason") == "compiler-artifact")
                require(profile["opt_level"] == "0" and profile["debuginfo"] == 2
                        and profile["debug_assertions"] and profile["overflow_checks"],
                        lane + ": Cargo did not use its ordinary development profile")
                fresh_index, _ = build(lane + "-default-dev-fresh", lane, default_target,
                                       expected_output(), fresh=True, project=default_project, env=default_env)
                incremental = default_target / "debug/incremental"
                require(incremental.is_dir() and any(incremental.iterdir()),
                        lane + ": default development build did not create its incremental cache")
                report["default_development_profile"]["lanes"][lane] = {
                    "build_command_index": build_index, "runtime_command_index": runtime_index,
                    "fresh_command_index": fresh_index, "cargo_profile": profile,
                    "incremental_directory": str(incremental),
                    "removed_inherited_ci_markers": removed_markers,
                }
            report["checks"].append("ordinary non-CI Cargo development defaults compile and run correctly with Cargo's debuginfo=2 request and incremental caching, then remain fresh")

            profile_root = scratch / "native profile contracts"
            profile_root.mkdir()
            profile_source = profile_root / "profile.rs"
            profile_text = (
                "fn next(x: u8) -> u8 { x + 1 }\n"
                'fn main() { println!("debug={} abort={} next={}", cfg!(debug_assertions), '
                'cfg!(panic = "abort"), next(255)); }\n'
            )
            profile_source.write_text(profile_text)
            report["profile_contract"] = {
                "source": profile_text, "source_sha256": digest(profile_source), "cases": [],
            }
            for label, flags, expected_exit, expected_stdout in (
                ("debug-off-default-overflow", ["-C", "debug-assertions=no"],
                 0, b"debug=false abort=false next=0\n"),
                ("abort-overflow-off", ["-C", "panic=abort", "-C", "overflow-checks=no"],
                 0, b"debug=true abort=true next=0\n"),
                ("unwind-overflow-on", ["-C", "panic=unwind", "-C", "overflow-checks=yes"],
                 101, b""),
                ("abort-overflow-on", ["-C", "panic=abort", "-C", "overflow-checks=yes"],
                 -signal.SIGABRT, b""),
            ):
                observations = {}
                for lane, compiler in (("native", native), ("oracle", oracle)):
                    artifact = profile_root / (label + "-" + lane)
                    execute("profile-" + label + "-" + lane + "-build",
                            [compiler, "--crate-name", "profile", "--edition=2024", profile_source,
                             "--emit=link", "-o", artifact, *flags], environments[lane], profile_root)
                    build_index = len(report["commands"]) - 1
                    runtime = execute("profile-" + label + "-" + lane + "-runtime", [artifact],
                                      environments[lane], profile_root, expected_exit)
                    require(runtime.stdout == expected_stdout, label + "-" + lane + ": wrong profile output")
                    if expected_exit != 0:
                        require(b"attempt to add with overflow" in runtime.stderr,
                                label + "-" + lane + ": runtime failed for a reason other than checked overflow")
                    observations[lane] = {
                        "build_command_index": build_index,
                        "runtime_command_index": len(report["commands"]) - 1,
                        "exit_code": runtime.returncode, "stdout": runtime.stdout.decode(),
                    }
                require(observations["native"]["exit_code"] == observations["oracle"]["exit_code"]
                        and observations["native"]["stdout"] == observations["oracle"]["stdout"],
                        label + ": native profile behavior differs from the Rust oracle")
                report["profile_contract"]["cases"].append({
                    "name": label, "flags": flags, "expected_exit": expected_exit,
                    "expected_stdout": expected_stdout.decode(), "observations": observations,
                })
            report["checks"].append("native debug assertions, overflow defaults/overrides and panic unwind/abort match the Rust oracle, including exit 101 and SIGABRT")

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
            for mode in ("fresh-target", "no-change", "body-edit", "signature-edit", "cold-invalid"):
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
                        if mode == "cold-invalid":
                            (project / "src/main.rs").write_text(
                                f'fn main() {{ let value: u32 = "invalid-{sample}"; println!("{{}}", value); }}\n')
                        target = mode_targets[lane]
                        cache_reset = None
                        if mode in ("fresh-target", "cold-invalid"):
                            target = target / str(sample)
                            require(not target.exists(), "cold timing target must not already exist")
                            cache_reset = clear_source_cache(project, scratch)
                        wanted = None if mode == "cold-invalid" else expected_output(answer)
                        command_index, runtime_index = build(
                            mode + "-" + lane + "-" + str(sample), lane, target, wanted,
                            fresh=True if mode == "no-change" else False if wanted is not None else None)
                        record = report["commands"][command_index]
                        report["timings"].append({
                            "mode": mode, "lane": lane, "sample": sample,
                            "elapsed_ns": record["elapsed_ns"], "command_index": command_index,
                            "runtime_command_index": runtime_index,
                            "source_cache_reset": cache_reset,
                            "compile_plus_run_ns": record["elapsed_ns"] + report["commands"][runtime_index]["elapsed_ns"]
                            if runtime_index is not None else None,
                            "source_sha256": {name: digest(project / name)
                                              for name in ("src/main.rs", "src/value.rs", "data.txt", "Cargo.toml")},
                        })
            report["checks"].append("balanced paired Cargo timings retain correct outputs, signature/body rebuilds, empty-cache fresh compilation and cold invalid-program rejection")
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
