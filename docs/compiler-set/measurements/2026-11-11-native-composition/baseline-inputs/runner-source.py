#!/usr/bin/env python3
"""Check native generic helpers combined with unsafe contracts and real imports.

Every candidate artifact comes from Lustr. Rustc is a separate oracle. These
functional checks neither qualify full Rust coverage nor claim a speed result.
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


UNSAFE_ADD = "unsafe fn add(value: i32) -> i32 { value + 1 }\n"
CASES = [
    ("identity", True,
     "fn identity<T>(value: T) -> T { value }\n" + UNSAFE_ADD +
     'fn main() { println!("{}", unsafe { add(identity(41)) }); }\n'),
    ("where_copy", True,
     "fn duplicate<T>(value: T) -> (T, T) where T: Copy { (value, value) }\n" + UNSAFE_ADD +
     'fn main() { let pair = duplicate(41); println!("{}", unsafe { add(pair.1) }); }\n'),
    ("const_parameter", True,
     "fn first<const N: usize>(values: [i32; N]) -> i32 { values[0] }\n" + UNSAFE_ADD +
     'fn main() { println!("{}", unsafe { add(first([41, 9, 2])) }); }\n'),
    ("lifetime_parameter", True,
     "fn identity<'a, T>(value: &'a T) -> &'a T { value }\n" + UNSAFE_ADD +
     'fn main() { let value = 41; println!("{}", unsafe { add(*identity(&value)) }); }\n'),
    ("unsafe_generic_function", True,
     "unsafe fn identity<T>(value: T) -> T { value }\n"
     'fn main() { println!("{}", unsafe { identity(42) }); }\n'),
    ("multiple_instances", True,
     "fn identity<T>(value: T) -> T { value }\n" + UNSAFE_ADD +
     'fn main() { let a = identity(20i32); let b = identity(21u32); '
     'println!("{}", unsafe { add(a + b as i32) }); }\n'),
    ("unused_safe_template", True,
     UNSAFE_ADD + "fn unused<T>(value: T) -> i32 { unsafe { add(41) } }\n"
     'fn main() { println!("42"); }\n'),
    ("generic_unsafe_pointer_parameter", True,
     UNSAFE_ADD + "fn invoke<T>(value: T, call: unsafe fn(i32) -> i32) -> i32 { unsafe { call(41) } }\n"
     'fn main() { println!("{}", invoke(true, add)); }\n'),
    ("unused_unsafe_call", False,
     UNSAFE_ADD + "fn unused<T>(value: T) -> i32 { add(41) }\n"
     'fn main() { println!("42"); }\n'),
    ("instantiated_unsafe_call", False,
     UNSAFE_ADD + "fn invoke<T>(value: T) -> i32 { add(41) }\n"
     'fn main() { println!("{}", invoke(true)); }\n'),
    ("unsafe_generic_call_without_block", False,
     "unsafe fn identity<T>(value: T) -> T { value }\n"
     'fn main() { println!("{}", identity(42)); }\n'),
    ("generic_pointer_safety_erasure", False,
     "unsafe fn identity<T>(value: T) -> T { value }\n"
     'fn main() { let safe: fn(i32) -> i32 = identity::<i32>; println!("{}", safe(42)); }\n'),
    ("unsafe_value_through_generic", False,
     "fn identity<T>(value: T) -> T { value }\n" + UNSAFE_ADD +
     'fn main() { let safe: fn(i32) -> i32 = identity(add as unsafe fn(i32) -> i32); '
     'println!("{}", safe(41)); }\n'),
    ("unused_pointer_call", False,
     UNSAFE_ADD + "fn unused<T>(value: T, call: unsafe fn(i32) -> i32) -> i32 { call(41) }\n"
     'fn main() { println!("42"); }\n'),
    ("unused_template_move", False,
     "fn duplicate<T>(value: T) -> (T, T) { (value, value) }\n" + UNSAFE_ADD +
     'fn main() { println!("{}", unsafe { add(41) }); }\n'),
    ("unused_template_type", False,
     "fn wrong<T>(value: T) -> T { 42 }\n" + UNSAFE_ADD +
     'fn main() { println!("{}", unsafe { add(41) }); }\n'),
    ("unused_template_borrow", False,
     "fn escaped<'a, T>(value: T) -> &'a T { &value }\n" + UNSAFE_ADD +
     'fn main() { println!("{}", unsafe { add(41) }); }\n'),
    ("unused_template_lint", False,
     UNSAFE_ADD + "#[deny(unsafe_code)]\nfn unused<T>(value: T) -> i32 { unsafe { add(41) } }\n"
     'fn main() { println!("42"); }\n'),
]

PRODUCER = (
    "fn identity<T>(value: T) -> T { value }\n"
    "pub unsafe fn add(value: i32) -> i32 { identity(value) + 1 }\n"
    "pub fn answer() -> i32 { identity(42) }\n"
)
CONSUMERS = [
    ("imported_unsafe_with_generic", True,
     "fn identity<T>(value: T) -> T { value }\n"
     'fn main() { println!("{}", unsafe { component::add(identity(41)) }); }\n'),
    ("imported_safe_with_generic", True,
     "fn identity<T>(value: T) -> T { value }\n"
     'fn main() { println!("{}", identity(component::answer())); }\n'),
    ("imported_unsafe_unused_template", False,
     "fn unused<T>(value: T) -> i32 { component::add(41) }\n"
     'fn main() { println!("42"); }\n'),
    ("imported_unsafe_instantiated_template", False,
     "fn invoke<T>(value: T) -> i32 { component::add(41) }\n"
     'fn main() { println!("{}", invoke(true)); }\n'),
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
        "schema": "lustr.native-mixed-generics-diagnostic.v1", "success": False,
        "qualified": False, "full_native_rust": False, "performance_claim": False,
        "scope": "Local generic helpers with native unsafe contracts and scalar artifact imports",
        "tools": {name: {"path": str(path), "sha256": digest(path)}
                  for name, path in (("native", native), ("oracle", oracle))},
        "runner_sha256": digest(Path(__file__)), "commands": [], "cases": [], "failures": [],
    }
    sentinel = output / "blocked-compilers"
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
                        "LUSTR_SENTINEL_LOG": str(sentinel_log), "LUST_NO_DAEMON": "1", "LUST_NO_CACHE": "1"})

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

    def check_case(name: str, valid: bool, source: str, libraries: dict | None = None) -> None:
        path = output / (name + ".rs")
        path.write_text(source)
        case = {"name": name, "valid_rust": valid, "source_sha256": digest(path), "lanes": {}}
        report["cases"].append(case)
        for lane, compiler in (("oracle", oracle), ("native", native)):
            if libraries is not None and lane not in libraries:
                report["failures"].append(name + ": missing " + lane + " producer artifact")
                continue
            artifact = output / (name + "-" + lane + ".exe")
            command = [compiler, "--crate-name", name, "--edition=2024", path,
                       "--crate-type", "bin", "--emit=link", "-o", artifact]
            if libraries is not None:
                command += ["--extern", "component=" + str(libraries[lane])]
            row = execute(name + "-" + lane, lane, command)
            accepted = row["exit_code"] == 0
            case["lanes"][lane] = {"exit_code": row["exit_code"], "artifact_exists": artifact.exists()}
            if accepted != valid:
                report["failures"].append(name + ": " + lane + " acceptance differs from the required Rust result")
            if accepted:
                if not artifact.is_file():
                    report["failures"].append(name + ": " + lane + " produced no artifact")
                    continue
                case["lanes"][lane]["artifact_sha256"] = digest(artifact)
                runtime = execute(name + "-" + lane + "-runtime", lane, [artifact])
                if runtime["exit_code"] != 0 or runtime["stdout"] != "42\n":
                    report["failures"].append(name + ": " + lane + " runtime differs")
            elif artifact.exists() or not row["stderr"].strip():
                report["failures"].append(name + ": " + lane + " rejection left an artifact or no diagnostic")

    try:
        for lane, compiler in (("oracle", oracle), ("native", native)):
            version = execute(lane + "-identity", lane, [compiler, "-vV"])
            report["tools"][lane]["version"] = version["stdout"]
            if version["exit_code"] != 0 or not version["stdout"].startswith("lustr-native " if lane == "native" else "rustc "):
                raise ValueError(lane + " compiler identity failed")
        for name, valid, source in CASES:
            check_case(name, valid, source)
        producer = output / "producer.rs"
        producer.write_text(PRODUCER)
        report["producer_source_sha256"] = digest(producer)
        libraries = {}
        for lane, compiler in (("oracle", oracle), ("native", native)):
            artifact = output / ("libcomponent-" + lane + ".rlib")
            row = execute("producer-" + lane, lane,
                          [compiler, "--crate-name", "component", "--edition=2024", producer,
                           "--crate-type", "rlib", "--emit=link", "-o", artifact])
            if row["exit_code"] == 0 and artifact.is_file():
                libraries[lane] = artifact
                report.setdefault("producer_artifacts", {})[lane] = {"sha256": digest(artifact)}
            else:
                report["failures"].append("producer: " + lane + " rejected private generic helper")
        producer.rename(output / "producer.sealed-source")
        report["producer_source_hidden"] = not producer.exists()
        for name, valid, source in CONSUMERS:
            check_case(name, valid, source, libraries)
        for name, tool in report["tools"].items():
            if digest(Path(tool["path"])) != tool["sha256"]:
                report["failures"].append(name + ": compiler changed during the run")
        report["success"] = not report["failures"]
    except Exception as error:
        report["failures"].append(type(error).__name__ + ": " + str(error))
    finally:
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"success": report["success"], "cases": len(report["cases"]),
                      "commands": len(report["commands"]), "failures": report["failures"]}, indent=2))
    raise SystemExit(0 if report["success"] else 1)


if __name__ == "__main__":
    main()
