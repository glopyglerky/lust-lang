#!/usr/bin/env python3
"""Run concrete unsafe grammar and safety regressions through both compilers.

This diagnostic does not replace native-library qualification or process traces.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time


CASES = [
    ("unsafe_main", "bin", False, 'unsafe fn main() { println!("42"); }\n'),
    ("function_body_unsafe", "bin", False, 'fn main() unsafe { println!("42"); }\n'),
    ("if_body_unsafe", "bin", False, 'fn main() { if true unsafe { println!("42"); } }\n'),
    ("loop_body_unsafe", "bin", False, 'fn main() { loop unsafe { println!("42"); break; } }\n'),
    ("else_body_unsafe", "bin", False, 'fn main() { if false {} else unsafe { println!("42"); } }\n'),
    ("unsafe_ref_parameter", "bin", False,
     'unsafe fn operation(value: i32) -> i32 { value + 1 }\n'
     'fn invoke(ref function: unsafe fn(i32) -> i32) -> i32 { function(41) }\n'
     'fn main() { println!("{}", invoke(operation)); }\n'),
    ("allow_then_deny", "bin", False,
     '#[allow(unsafe_code)]\n#[deny(unsafe_code)]\n'
     'unsafe fn helper(value: i32) -> i32 { value + 1 }\n'
     'fn main() { println!("{}", unsafe { helper(41) }); }\n'),
    ("qualified_lint_override", "bin", False,
     '#[deny(unsafe_code)]\n#[allow(clippy::unsafe_code)]\n'
     'unsafe fn helper(value: i32) -> i32 { value + 1 }\n'
     'fn main() { println!("{}", unsafe { helper(41) }); }\n'),
    ("closure_return_context", "bin", False,
     'fn factory() -> unsafe fn(i32) -> i32 {\n'
     '    let make = || { return unsafe_helper; };\n'
     '    let safe: fn(i32) -> i32 = make();\n'
     '    safe\n}\n'
     'unsafe fn unsafe_helper(value: i32) -> i32 { value + 1 }\n'
     'fn main() { println!("{}", unsafe { factory()(41) }); }\n'),
    ("unsafe_type_length", "bin", False,
     '#[deny(unsafe_code)]\n'
     'fn main() { let values: [u8; unsafe { 1 }] = [0]; println!("{}", values[0] + 42); }\n'),
    ("unsafe_type_length_control", "bin", True,
     'fn main() { let values: [u8; unsafe { 1 }] = [0]; println!("{}", values[0] + 42); }\n'),
    ("closure_outside_unsafe", "bin", False,
     'unsafe fn helper(value: i32) -> i32 { value + 1 }\n'
     'fn main() { let call = || helper(41); unsafe { println!("{}", call()); } }\n'),
    ("closure_inside_unsafe", "bin", True,
     'unsafe fn helper(value: i32) -> i32 { value + 1 }\n'
     'fn main() { let call = unsafe { || helper(41) }; println!("{}", call()); }\n'),
    ("unsafe_call_control", "bin", True,
     'unsafe fn helper(value: i32) -> i32 { value + 1 }\n'
     'fn main() { println!("{}", unsafe { helper(41) }); }\n'),
    ("unsafe_main_library_control", "rlib", True, 'pub unsafe fn main() -> i32 { 42 }\n'),
    ("safe_main_library_control", "rlib", True, 'pub fn main() -> i32 { 42 }\n'),
    ("extension_prefix_middle_suffix", "bin", True,
     'fn prefix() -> i32 { 20 }\n'
     'unsafe fn helper(value: i32) -> i32 { value + 1 }\n'
     'fn suffix() -> i32 { 21 }\n'
     'fn main() { println!("{}", unsafe { helper(prefix() + suffix()) }); }\n'),
    ("extension_after_opaque_macro", "bin", True,
     'macro_rules! ignored { () => { unsafe extern "C" { fn never(); } }; }\n'
     'unsafe fn helper(value: i32) -> i32 { value + 1 }\n'
     'fn main() { println!("{}", unsafe { helper(41) }); }\n'),
    ("opaque_macro_between_extensions", "bin", True,
     'unsafe fn helper(value: i32) -> i32 { value + 1 }\n'
     'macro_rules! ignored { () => { unsafe extern "C" { fn never(); } }; }\n'
     'fn main() { println!("{}", unsafe { helper(41) }); }\n'),
    ("extension_with_string_pool", "bin", True,
     'fn main() { let text = "unsafe fn, unsafe {, unsafe impl";\n'
     '    println!("{}", unsafe { helper(41) }); }\n'
     'unsafe fn helper(value: i32) -> i32 { value + 1 }\n'),
    ("plain_opaque_macro_control", "bin", True,
     'macro_rules! ignored { () => { unsafe extern "C" { fn never(); } }; }\n'
     'fn main() { println!("42"); }\n'),
]


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--oracle", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    native, oracle, output = (path.resolve() for path in (args.native, args.oracle, args.output))
    if native == oracle or output.exists():
        raise ValueError("Use distinct native/oracle executables and a new evidence directory")
    output.mkdir(parents=True)
    shutil.copy2(Path(__file__), output / "runner-source.py")
    report = {
        "schema": "lustr.native-unsafe-regression-diagnostic.v1",
        "success": False, "qualified": False, "full_native_rust": False,
        "scope": "Concrete unsafe syntax, lint precedence, callable escape and closure scope regressions",
        "process_trace": False, "performance_claim": False,
        "tools": {name: {"path": str(path), "sha256": digest(path)}
                  for name, path in (("native", native), ("oracle", oracle))},
        "runner_sha256": digest(Path(__file__)),
        "commands": [], "cases": [], "failures": [],
        "accepted_invalid_cases": [], "unsupported_valid_cases": [],
    }
    sentinel = output / "blocked compilers"
    sentinel.mkdir()
    sentinel_log = output / "unexpected-compiler.log"
    for name in ("rustc", "rustdoc", "lust", "lust-core", "lustr", "cargo-lustr", "cargo-lust"):
        path = sentinel / name
        path.write_text('#!/bin/sh\nprintf "%s\\n" "$0" >> "$LUSTR_SENTINEL_LOG"\nexit 97\n')
        path.chmod(0o755)
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith(("CARGO_", "RUST", "LUST_", "LUSTR_", "LD_", "DYLD_"))
                   and key not in ("CARGO", "LUSTC", "LIBPATH", "SHLIB_PATH", "LDR_PRELOAD")}
    environment.update({"PATH": str(sentinel) + os.pathsep + "/usr/bin:/bin",
                        "LUSTR_SENTINEL_LOG": str(sentinel_log)})

    def execute(label: str, lane: str, argv: list[str | Path]) -> dict:
        command = [str(value) for value in argv]
        started = time.monotonic_ns()
        process = subprocess.Popen(command, cwd=output, env=environment, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, start_new_session=True)
        timed_out = False
        try:
            stdout, stderr = process.communicate(timeout=60)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            stdout, stderr = process.communicate()
        row = {"label": label, "lane": lane, "argv": command, "exit_code": process.returncode,
               "stdout": stdout.decode(errors="replace"), "stderr": stderr.decode(errors="replace"),
               "timed_out": timed_out, "elapsed_ns": time.monotonic_ns() - started}
        report["commands"].append(row)
        if timed_out:
            report["failures"].append(label + ": timed out")
        if sentinel_log.exists():
            report["failures"].append(label + ": unexpected compiler sentinel invoked")
        return row

    try:
        for lane, compiler in (("oracle", oracle), ("native", native)):
            identity = execute(lane + "-identity", lane, [compiler, "-vV"])
            report["tools"][lane]["version"] = identity["stdout"]
            if identity["exit_code"] != 0:
                raise ValueError(lane + " compiler identity failed")
        if not report["tools"]["native"]["version"].startswith("lustr-native "):
            raise ValueError("Candidate does not identify as the native Lustr compiler")
        for name, kind, valid, source in CASES:
            path = output / (name + ".rs")
            path.write_text(source)
            case = {"name": name, "crate_type": kind, "valid_rust": valid,
                    "source_sha256": digest(path), "lanes": {}}
            report["cases"].append(case)
            for lane, compiler in (("oracle", oracle), ("native", native)):
                artifact = output / (("lib" + name + "_" + lane + ".rlib") if kind == "rlib" else (name + "-" + lane + ".exe"))
                row = execute(name + "-" + lane, lane,
                              [compiler, "--crate-name", name, "--edition=2024", path,
                               "--crate-type", kind, "--emit=link", "-o", artifact])
                accepted = row["exit_code"] == 0
                case["lanes"][lane] = {"exit_code": row["exit_code"], "artifact_exists": artifact.exists()}
                if accepted != valid:
                    report["failures"].append(name + ": " + lane + " acceptance differs from the required Rust result")
                    if lane == "native":
                        report["unsupported_valid_cases" if valid else "accepted_invalid_cases"].append(name)
                if not valid and row["exit_code"] != 1:
                    report["failures"].append(name + ": " + lane + " must reject semantically with exit 1; a crash or infrastructure error is not rejection evidence")
                if accepted:
                    if not artifact.is_file():
                        report["failures"].append(name + ": " + lane + " successful compile produced no artifact")
                        continue
                    case["lanes"][lane]["artifact_sha256"] = digest(artifact)
                    if kind == "bin":
                        runtime = execute(name + "-" + lane + "-runtime", lane, [artifact])
                        if runtime["exit_code"] != 0 or runtime["stdout"] != "42\n":
                            report["failures"].append(name + ": " + lane + " runtime differs")
                elif artifact.exists() or not row["stderr"].strip():
                    report["failures"].append(name + ": " + lane + " rejection left an artifact or no diagnostic")
            if kind == "rlib":
                hidden = output / (name + ".sealed-source")
                path.rename(hidden)
                case["producer_source_hidden"] = not path.exists()
                case["retained_producer_source"] = hidden.name
                consumer = output / (name + "_consumer.rs")
                expression = "unsafe { component::main() }" if name == "unsafe_main_library_control" else "component::main()"
                consumer.write_text('fn main() { println!("{}", ' + expression + '); }\n')
                case["consumer_sha256"] = digest(consumer)
                for lane, compiler in (("oracle", oracle), ("native", native)):
                    artifact = output / ("lib" + name + "_" + lane + ".rlib")
                    if case["lanes"][lane]["exit_code"] != 0 or not artifact.is_file():
                        continue
                    executable = output / (name + "-" + lane + "-consumer.exe")
                    row = execute(name + "-" + lane + "-source-hidden-consumer", lane,
                                  [compiler, "--crate-name", name + "_consumer", "--edition=2024", consumer,
                                   "--crate-type", "bin", "--emit=link", "--extern", "component=" + str(artifact),
                                   "-o", executable])
                    if row["exit_code"] != 0 or not executable.is_file():
                        report["failures"].append(name + ": " + lane + " source-hidden consumer failed")
                    else:
                        runtime = execute(name + "-" + lane + "-source-hidden-runtime", lane, [executable])
                        if runtime["exit_code"] != 0 or runtime["stdout"] != "42\n":
                            report["failures"].append(name + ": " + lane + " source-hidden consumer runtime differs")
        for name, tool in report["tools"].items():
            if digest(Path(tool["path"])) != tool["sha256"]:
                report["failures"].append(name + ": compiler changed during the run")
        report["success"] = not report["failures"]
    finally:
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({key: report[key] for key in
                     ("success", "qualified", "accepted_invalid_cases", "unsupported_valid_cases", "failures")}, indent=2))
    raise SystemExit(0 if report["success"] else 1)


if __name__ == "__main__":
    main()
