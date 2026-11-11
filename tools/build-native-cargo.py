#!/usr/bin/env python3
"""Build the native Cargo compiler and retain its observed build provenance."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys


HOST = "x86_64-unknown-linux-gnu"
SCHEMA = "lustr.native-cargo-build.v2"
# Upstream explicitly reserves its gate profile for correctness work. Measure
# and package the release recipe used by Lust's perfcheck and shipped binaries.
SETTINGS = {"opt_level": 3, "codegen_units": 1, "lto": "thin", "debug": "line-tables-only",
            "incremental": False}
PROFILE_ENV = {"CARGO_PROFILE_RELEASE_OPT_LEVEL": "3", "CARGO_PROFILE_RELEASE_CODEGEN_UNITS": "1",
               "CARGO_PROFILE_RELEASE_LTO": "thin", "CARGO_PROFILE_RELEASE_DEBUG": "line-tables-only",
               "CARGO_PROFILE_RELEASE_INCREMENTAL": "false", "CARGO_INCREMENTAL": "0"}


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def run(argv: list[str | Path], cwd: Path | None = None, env: dict | None = None) -> str:
    return subprocess.check_output([str(v) for v in argv], cwd=cwd, env=env, text=True).strip()


def source_identity(repository: Path) -> dict:
    if run(["git", "status", "--porcelain=v1", "--untracked-files=all"], repository):
        raise ValueError("commit all source and helper changes before recording native build provenance")
    return {"commit": run(["git", "rev-parse", "HEAD"], repository),
            "tree": run(["git", "rev-parse", "HEAD^{tree}"], repository),
            "native_tree": run(["git", "rev-parse", "HEAD:compiler/native"], repository),
            "helper_sha256": digest(repository / "tools/build-native-cargo.py")}


def runtime_digest(root: Path) -> str:
    h = hashlib.sha256()
    for directory in (root / "bin", root / "lib"):
        if not directory.is_dir():
            raise ValueError(f"missing TinyCC build input: {directory}")
        for path in sorted(directory.rglob("*")):
            if path.is_file():
                h.update(str(path.relative_to(root)).encode() + b"\0")
                h.update(bytes.fromhex(digest(path)))
    return h.hexdigest()


def option(argv: list[str], name: str) -> str | None:
    values = []
    for at, arg in enumerate(argv):
        if arg == name:
            if at + 1 == len(argv):
                raise ValueError(f"missing value for {name}")
            values.append(argv[at + 1])
        elif arg.startswith(name + "="):
            values.append(arg[len(name) + 1:])
    if len(values) > 1:
        raise ValueError(f"ambiguous repeated compiler option {name}")
    return values[0] if values else None


def validate_invocation(argv: list[str], target: Path, bootstrap: str) -> dict:
    if not argv or argv[0] != bootstrap or option(argv, "--crate-name") != "lustr_native":
        raise ValueError("receipt does not contain the direct bootstrap invocation for lustr_native")
    for name, value in (("--crate-type", "bin"), ("--edition", "2024"), ("--target", HOST),
                        ("--out-dir", str(target / HOST / "release/deps"))):
        if option(argv, name) != value:
            raise ValueError(f"unexpected native compiler build option {name}")
    if option(argv, "--emit") != "dep-info,link" or "src/main.rs" not in argv:
        raise ValueError("native build did not compile and link the expected entry point")
    if any(arg in argv for arg in ("-g", "-O", "--test")):
        raise ValueError("unexpected shorthand profile or test-harness option")
    codegen = {}
    at = 1
    while at < len(argv):
        arg = argv[at]
        if arg in ("-C", "--codegen"):
            at += 1
            if at == len(argv):
                raise ValueError("missing codegen option")
            arg = argv[at]
        elif arg.startswith("-C"):
            arg = arg[2:]
        elif arg.startswith("--codegen="):
            arg = arg[len("--codegen="):]
        else:
            at += 1
            continue
        name, equal, value = arg.partition("=")
        if not equal or name in codegen:
            raise ValueError(f"ambiguous codegen option {arg}")
        codegen[name] = value
        at += 1
    if (codegen.get("opt-level") != "3" or codegen.get("codegen-units") != "1"
            or codegen.get("lto") != "thin" or codegen.get("debuginfo") != "line-tables-only"):
        raise ValueError("actual native release build must use opt3, one codegen unit, ThinLTO and line tables")
    if "incremental" in codegen:
        raise ValueError("timed release compiler must not use an incremental compiler build")
    return {"opt_level": int(codegen["opt-level"]), "codegen_units": int(codegen["codegen-units"]),
            "lto": codegen["lto"], "debug": codegen["debuginfo"], "incremental": False}


def running_argv(line: str) -> list[str] | None:
    text = line.strip()
    if not text.startswith("Running `") or not text.endswith("`"):
        return None
    argv = shlex.split(text[len("Running `"):-1])
    # Cargo -vv prefixes commands with environment assignments. Do not retain
    # those values or invoke a wrapper to observe compiler execution.
    while argv and re.fullmatch(r"[A-Za-z_][A-Za-z_0-9]*=.*", argv[0], re.DOTALL):
        argv.pop(0)
    return argv


def safe_line(line: str, argv: list[str] | None, secrets: list[str]) -> str:
    if argv is not None:
        line = "Running `" + shlex.join(argv) + "`\n"
    elif "process didn't exit successfully:" in line and "`" in line:
        line = line[:line.index("`")] + "[command omitted]" + line[line.rfind("`") + 1:]
    elif "cargo:rustc-env=" in line:
        line = line.split("cargo:rustc-env=", 1)[0] + "cargo:rustc-env=[value omitted]\n"
    else:
        try:
            event = json.loads(line)
            if event.get("reason") == "build-script-executed":
                event["env"] = [[name, "[value omitted]"] for name, _ in event.get("env", [])]
                line = json.dumps(event) + "\n"
        except (json.JSONDecodeError, AttributeError):
            pass
    for secret in secrets:
        line = line.replace(secret, "[secret omitted]")
    return line


def verify_receipt(path: Path, repository: Path, binary: Path, tcc: Path) -> dict:
    record = json.loads(path.read_text())
    if record.get("schema") != SCHEMA:
        raise ValueError("missing native Cargo build receipt schema")
    build = record["build"]
    actual = validate_invocation(build["rustc_argv"], Path(build["target_dir"]), record["bootstrap"]["path"])
    if build.get("profile") != "release" or build.get("settings") != actual:
        raise ValueError("native Cargo receipt settings differ from the observed compiler invocation")
    if record["binary"]["sha256"] != digest(binary):
        raise ValueError("native Cargo executable differs from the recorded build")
    if record["binary"]["verbose_version"] != run([binary, "--version", "--verbose"]):
        raise ValueError("native Cargo executable identity differs from the recorded build")
    if record["source"] != source_identity(repository):
        raise ValueError("native Cargo build receipt belongs to different source or helper code")
    if record["tinycc"]["sha256"] != runtime_digest(tcc):
        raise ValueError("native Cargo build receipt belongs to different TinyCC inputs")
    log_name = record["log"]["file"]
    if not log_name or Path(log_name).name != log_name:
        raise ValueError("native build log must be adjacent to its receipt")
    log = path.parent / log_name
    if digest(log) != record["log"]["sha256"]:
        raise ValueError("native Cargo build log differs from the receipt")
    invocations = [argv for line in log.read_text().splitlines()
                   if (argv := running_argv(line)) and option(argv, "--crate-name") == "lustr_native"]
    if invocations != [build["rustc_argv"]]:
        raise ValueError("native Cargo receipt is not backed by one matching logged compiler invocation")
    return record


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--cargo", type=Path, required=True, help="actual Cargo binary, not a rustup proxy")
    parser.add_argument("--rustc", type=Path, required=True, help="actual bootstrap rustc binary")
    parser.add_argument("--tcc-root", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True, help="new directory outside the source checkout")
    parser.add_argument("--receipt", type=Path, required=True, help="new JSON receipt outside the checkout")
    args = parser.parse_args()
    repository, cargo, rustc, tcc, target, receipt = (value.resolve() for value in
        (args.repository, args.cargo, args.rustc, args.tcc_root, args.target_dir, args.receipt))
    log = receipt.with_suffix(".cargo.log")
    if receipt.suffix != ".json":
        parser.error("--receipt must name a new .json file")
    if any(path.is_relative_to(repository) for path in (target, receipt)):
        parser.error("native build target and receipt must be outside the source checkout")
    if any(path.exists() for path in (target, receipt, log)):
        parser.error("native build target, receipt and log must be new paths")
    source = source_identity(repository)
    if source["helper_sha256"] != digest(Path(__file__)):
        raise ValueError("invoke the committed helper from the selected source checkout")
    env = {key: value for key, value in os.environ.items()
           if not key.startswith(("RUST", "LUST_", "CARGO_", "LD_", "DYLD_")) or key == "CARGO_HOME"}
    env.update(PROFILE_ENV)
    env.update({"RUSTC": str(rustc), "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "",
                "RUSTFLAGS": "", "CARGO_ENCODED_RUSTFLAGS": "", "CARGO_TERM_COLOR": "never",
                "LUST_TCC_HOME": str(tcc)})
    bootstrap = {"path": str(rustc), "sha256": digest(rustc), "verbose_version": run([rustc, "-vV"], env=env)}
    if f"host: {HOST}" not in bootstrap["verbose_version"].splitlines():
        raise ValueError("native package bootstrap must run on Linux x86_64")
    cargo_identity = {"path": str(cargo), "sha256": digest(cargo), "verbose_version": run([cargo, "-Vv"], env=env)}
    tcc_hash = runtime_digest(tcc)
    argv = [str(cargo), "build", "--manifest-path", str(repository / "crates/lustr-native/Cargo.toml"),
            "--profile", "release", "--target", HOST, "--target-dir", str(target), "--locked", "--offline",
            "-j1", "-vv", "--message-format=json"]
    target.mkdir(parents=True, exist_ok=False)
    receipt.parent.mkdir(parents=True, exist_ok=True)
    secret_values = [value for key, value in os.environ.items() if len(value) >= 8
                     and re.search(r"TOKEN|PASSWORD|SECRET|CREDENTIAL|PRIVATE_KEY|COOKIE", key, re.I)]
    invocations, artifacts = [], []
    with log.open("x") as stream:
        process = subprocess.Popen(argv, cwd=repository, env=env, text=True,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        assert process.stdout is not None
        for line in process.stdout:
            command = running_argv(line)
            if command and option(command, "--crate-name") == "lustr_native":
                invocations.append(command)
            try:
                event = json.loads(line)
                if (event.get("reason") == "compiler-artifact" and event.get("target", {}).get("name") == "lustr-native"
                        and event.get("executable")):
                    artifacts.append(event)
            except json.JSONDecodeError:
                pass
            stream.write(safe_line(line, command, secret_values))
            stream.flush()
            if line.lstrip().startswith(("Compiling ", "Finished ", "error:")):
                print(safe_line(line, command, secret_values), end="", flush=True)
        if process.wait() != 0:
            raise ValueError(f"native Cargo build failed; retained log: {log}")
    if len(invocations) != 1 or len(artifacts) != 1 or artifacts[0].get("fresh") is not False:
        raise ValueError("expected one fresh native compiler invocation and linked Cargo artifact")
    settings = validate_invocation(invocations[0], target, str(rustc))
    binary = target / HOST / "release/lustr-native"
    if artifacts[0]["executable"] != str(binary) or not os.access(binary, os.X_OK):
        raise ValueError("Cargo did not produce the expected native compiler executable")
    version = run([binary, "--version", "--verbose"], env=env)
    if not version.startswith("lustr-native ") or "native-full-rust: false" not in version.splitlines():
        raise ValueError("native executable did not report the expected experimental compiler identity")
    if source_identity(repository) != source or runtime_digest(tcc) != tcc_hash or digest(rustc) != bootstrap["sha256"]:
        raise ValueError("native build inputs changed during compilation")
    record = {"schema": SCHEMA, "source": source, "bootstrap": bootstrap, "cargo": cargo_identity,
              "tinycc": {"path": str(tcc), "sha256": tcc_hash},
              "build": {"profile": "release", "settings": settings, "target_dir": str(target),
                        "cargo_argv": argv, "rustc_argv": invocations[0], "profile_environment": PROFILE_ENV},
              "binary": {"path": str(binary), "sha256": digest(binary), "verbose_version": version},
              "log": {"file": log.name, "sha256": digest(log)}}
    with receipt.open("x") as stream:
        json.dump(record, stream, indent=2)
        stream.write("\n")
    try:
        verify_receipt(receipt, repository, binary, tcc)
    except Exception:
        receipt.unlink()
        raise
    print(json.dumps({"binary": str(binary), "receipt": str(receipt), "sha256": record["binary"]["sha256"]}))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"native build provenance: {error}", file=sys.stderr)
        raise SystemExit(1)
