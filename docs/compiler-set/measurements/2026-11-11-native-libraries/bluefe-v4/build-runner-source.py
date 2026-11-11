#!/usr/bin/env python3
"""Attempt unchanged BlueFe with Lustr as its sole Rust compiler."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def config_text(compiler: Path, cargo: Path, work: Path) -> str:
    values = [compiler, cargo, work]
    if any(not p.is_absolute() or any(c in str(p) for c in '\n\r"\\') for p in values):
        raise ValueError("build paths must be absolute and representable by upstream bootstrap's TOML reader")
    # Stage zero keeps the selected Lustr binary responsible for compiling the
    # compiler sources. A later bootstrap stage would select the stock compiler
    # produced from those sources and would violate this qualification.
    return f'''change-id = "ignore"
[build]
build = "x86_64-unknown-linux-gnu"
host = ["x86_64-unknown-linux-gnu"]
target = ["x86_64-unknown-linux-gnu"]
rustc = "{compiler}"
cargo = "{cargo}"
rustdoc = "{work / 'blocked-bin/rustdoc'}"
build-dir = "{work / 'build'}"
build-stage = 0
jobs = 2
docs = false
submodules = false
locked-deps = true
[llvm]
download-ci-llvm = true
[rust]
download-rustc = false
deny-warnings = false
'''


def environment(compiler: Path, work: Path) -> dict[str, str]:
    env = dict(os.environ)
    for key in list(env):
        if key.startswith(("RUST", "CARGO_BUILD_", "CARGO_TARGET_", "CARGO_ENCODED_", "LUST_", "LUSTR_")):
            env.pop(key)
    env.update({
        "RUSTC": str(compiler),
        "RUSTC_WRAPPER": "",
        "RUSTC_WORKSPACE_WRAPPER": "",
        "RUSTDOC": str(work / "blocked-bin/rustdoc"),
        "PATH": str(work / "blocked-bin") + os.pathsep + "/usr/bin:/bin",
        "BLUEFE_UNEXPECTED_COMPILER": str(work / "unexpected-compiler.log"),
        "PYTHONDONTWRITEBYTECODE": "1",
    })
    return env


def successful_execs(trace: str) -> list[str]:
    # strace's argv/env output is retained separately. Failed PATH probes are
    # not compiler executions; only successful execve transitions count here.
    result, pending = [], {}
    for line in trace.splitlines():
        pid = re.match(r"(?:\[pid\s+)?(\d+)\]?\s+", line)
        key = pid.group(1) if pid else "root"
        if ("execve(" in line or "execveat(" in line) and line.endswith("<unfinished ...>"):
            pending[key] = line.removesuffix("<unfinished ...>")
            continue
        resumed = re.search(r"<\.\.\. execve(?:at)? resumed>", line)
        if resumed:
            if key not in pending:
                raise ValueError("resumed exec without its opening record cannot be attributed")
            line = pending.pop(key) + line[resumed.end():]
        match = re.search(r'execve\(("(?:[^"\\]|\\.)*"),.*\)\s*=\s*0\s*$', line)
        if match:
            path = json.loads(match.group(1))
            if not Path(path).is_absolute():
                raise ValueError("successful execve without an absolute path cannot be attributed to a compiler")
            result.append(path)
        at = re.search(r'execveat\([^,]+,\s*("(?:[^"\\]|\\.)*"),.*\)\s*=\s*0\s*$', line)
        if at:
            path = json.loads(at.group(1))
            if not Path(path).is_absolute():
                raise ValueError("successful execveat without an absolute path cannot be attributed to a compiler")
            result.append(path)
    if pending:
        raise ValueError("unterminated exec records leave compiler execution unobserved")
    return result


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--source", type=Path, required=True)
    p.add_argument("--lustr", type=Path, required=True)
    p.add_argument("--cargo", type=Path, required=True)
    p.add_argument("--guard", type=Path, required=True)
    p.add_argument("--work", type=Path, required=True)
    p.add_argument("--diagnostic-without-trace", action="store_true",
                   help="execute for diagnosis where ptrace is unavailable; never reports qualification success")
    args = p.parse_args()
    source, compiler, cargo, guard, work = (v.resolve() for v in (args.source, args.lustr, args.cargo, args.guard, args.work))
    if work.exists() or work.is_relative_to(source) or source.is_relative_to(work):
        p.error("--work must be a new directory outside the source checkout")
    for path in (compiler, cargo, guard):
        if not path.is_file() or not os.access(path, os.X_OK):
            p.error(f"missing executable: {path}")
    tracer = shutil.which("strace")
    if tracer is None and not args.diagnostic_without_trace:
        p.error("strace is required to retain actual compiler process evidence")
    work.mkdir(parents=True)
    report = {
        "schema": "bluefe.lustr-only-build.v1", "success": False,
        "source": {"repository": "glopyglerky/blueFe", "commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=source, text=True).strip()},
        "compiler": {"path": str(compiler), "sha256": digest(compiler)},
        "cargo": {"path": str(cargo), "sha256": digest(cargo)},
        "stage": 0, "stock_rustc_fallback": False, "complete_native_lustr": False,
        "process_trace_required": not args.diagnostic_without_trace,
        "diagnostic_only": args.diagnostic_without_trace,
    }
    try:
        subprocess.run([guard], cwd=source, check=True)
        version = subprocess.check_output([compiler, "--version", "--verbose"], text=True, timeout=30)
        report["compiler"]["version"] = version
        if not version.startswith("lustr-native ") or "backend: unchanged Lust C/TinyCC" not in version:
            raise ValueError("the selected executable is not the native Lustr compiler")
        blocked = work / "blocked-bin"
        blocked.mkdir()
        for name in ("rustc", "rustdoc", "rustup", "cargo-lustr", "cargo-lust"):
            path = blocked / name
            path.write_text('#!/bin/sh\nprintf "%s\\n" "$0" >> "$BLUEFE_UNEXPECTED_COMPILER"\nexit 97\n')
            path.chmod(0o755)
        config = work / "bootstrap.toml"
        config.write_text(config_text(compiler, cargo, work))
        report["configuration_sha256"] = digest(config)
        command = [sys.executable, str(source / "x.py"), "--config", str(config),
                   "--build-dir", str(work / "build"), "-vv", "build", "--stage", "0", "compiler/rustc"]
        report["command"] = command
        start = time.monotonic()
        with (work / "build.log").open("wb") as log:
            executed = command if args.diagnostic_without_trace else [tracer, "-f", "-s", "65536", "-e", "trace=execve,execveat",
                                                                       "-o", str(work / "execve.log"), *command]
            result = subprocess.run(executed,
                                    cwd=source, env=environment(compiler, work), stdout=log, stderr=subprocess.STDOUT)
        report["elapsed_seconds"] = time.monotonic() - start
        report["returncode"] = result.returncode
        execs = [] if args.diagnostic_without_trace else successful_execs((work / "execve.log").read_text())
        report["executed_programs"] = execs
        native_calls = [path for path in execs if Path(path).resolve() == compiler]
        report["native_compiler_executions"] = len(native_calls)
        # Upstream's bootstrap forwarding shim is permitted only in its exact
        # build directory. It must still execute the selected native compiler.
        shim = work / "build/bootstrap/debug/rustc"
        unexpected = [path for path in execs if Path(path).name in ("rustc", "rustc.exe")
                      and Path(path).resolve() not in (compiler, shim)]
        report["unexpected_compilers"] = unexpected
        if unexpected or (work / "unexpected-compiler.log").exists():
            raise ValueError("a compiler other than the selected native Lustr was invoked")
        if result.returncode != 0:
            raise ValueError("unchanged BlueFe build failed; inspect build.log and process evidence to distinguish native gaps from build-environment failures")
        if args.diagnostic_without_trace:
            report["diagnostic_build_success"] = True
            raise ValueError("diagnostic build completed without process tracing; compiler-exclusivity qualification remains open")
        if not native_calls:
            raise ValueError("the build did not execute the selected native compiler")
        outputs = sorted((work / "build").glob("*/stage0*/**/rustc"))
        outputs = [path for path in outputs if path.is_file() and path != shim]
        if not outputs:
            raise ValueError("bootstrap returned success without a compiler artifact")
        report["outputs"] = [{"path": str(path), "sha256": digest(path)} for path in outputs]
        report["success"] = True
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        report["error"] = str(error)
    finally:
        guard_run = subprocess.run([guard], cwd=source, capture_output=True, text=True)
        report["source_guard_after"] = {"returncode": guard_run.returncode,
                                        "stdout": guard_run.stdout, "stderr": guard_run.stderr}
        if guard_run.returncode != 0:
            report["success"] = False
        report["compiler_unchanged"] = digest(compiler) == report["compiler"]["sha256"]
        if not report["compiler_unchanged"]:
            report["success"] = False
        (work / "qualification.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"success": report["success"], "evidence": str(work / "qualification.json"),
                      "error": report.get("error")}, indent=2))
    return 0 if report["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
