#!/usr/bin/env python3
"""Qualify real, independently compiled scalar native libraries through Cargo.

The Rust compiler is an explicit oracle only. The native lane always uses the
given Lustr executable. Functional exec tracing is not performance evidence.
"""
from __future__ import annotations

import argparse
import ast
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import signal
import struct
import subprocess
import time


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def read_ar(path: Path) -> list[tuple[str, bytes]]:
    data = path.read_bytes()
    require(data.startswith(b"!<arch>\n"), f"not an ar library: {path}")
    at, result, names = 8, [], set()
    while at < len(data):
        header = data[at:at + 60]
        require(len(header) == 60 and header[58:] == b"`\n", "invalid ar member header")
        name = header[:16].decode("ascii").strip()
        size = int(header[48:58].decode("ascii").strip())
        at += 60
        require(0 <= size <= len(data) - at, "truncated ar member")
        body = data[at:at + size]
        at += size + size % 2
        if name in ("/", "//", "/SYM64/"):
            continue
        name = name.removesuffix("/")
        require(name not in names and "/" not in name, "duplicate or nonlocal ar member")
        names.add(name)
        result.append((name, body))
    require(at == len(data), "trailing or missing ar padding")
    return result


def write_ar(path: Path, members: list[tuple[str, bytes]]) -> None:
    with path.open("xb") as stream:
        stream.write(b"!<arch>\n")
        for name, body in members:
            require(len(name) <= 15 and "/" not in name, "unsupported mutation member name")
            header = f"{name + '/':<16}{0:<12}{0:<6}{0:<6}{'100644':<8}{len(body):<10}`\n"
            require(len(header) == 60, "invalid mutation header size")
            stream.write(header.encode() + body + (b"\n" if len(body) % 2 else b""))


def inspect_metadata(data: bytes, objects: list[tuple[str, bytes]]) -> dict:
    """Inspect the documented v1 format independently of the compiler reader."""
    require(data.startswith(b"LUSTRMD\x01") and len(data) > 16, "missing versioned native metadata")
    at, end = 8, len(data) - 8

    def checksum(value: bytes) -> int:
        result = 0xcbf29ce484222325
        for byte in value:
            result = ((result ^ byte) * 0x100000001b3) & 0xffffffffffffffff
        return result

    require(checksum(data[:end]) == int.from_bytes(data[end:], "little"), "native metadata checksum differs")

    def take(size: int) -> bytes:
        nonlocal at
        require(size >= 0 and at + size <= end, "truncated native metadata")
        value = data[at:at + size]
        at += size
        return value

    def number(size: int = 4) -> int:
        return int.from_bytes(take(size), "little")

    def string() -> str:
        return take(number()).decode("utf-8")

    result = {name: string() for name in ("crate_name", "disambiguator", "target", "compiler", "runtime")}
    result["profile_flags"] = number(1)
    types = ["unit", "bool", "char", "i8", "i16", "i32", "i64", "i128", "isize",
             "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64"]

    def abi_type() -> str:
        value = number(1)
        require(value < len(types), "unknown native metadata ABI type")
        return types[value]

    exports = []
    for _ in range(number()):
        path = [string() for _ in range(number())]
        symbol, safety = string(), number(1)
        require(safety in (0, 1), "invalid native safety bit")
        params = [abi_type() for _ in range(number())]
        exports.append({"path": path, "symbol": symbol, "unsafe": bool(safety), "params": params, "result": abi_type()})
    result["exports"] = exports
    result["locations"] = [{"symbol": string(), "file": string(), "line": number()} for _ in range(number())]
    recorded_objects = [{"name": string(), "bytes": number(), "checksum": number(8)} for _ in range(number())]
    require(recorded_objects == [{"name": name, "bytes": len(body), "checksum": checksum(body)} for name, body in objects],
            "native metadata does not bind exact object members")
    require(at == end, "unexpected native metadata payload or source bodies")
    result["objects"] = recorded_objects
    return result


def compiler_invocations(stderr: str, compiler: Path) -> list[dict]:
    result = []
    for line in stderr.splitlines():
        line = line.strip()
        if not line.startswith("Running `") or not line.endswith("`"):
            continue
        parts = shlex.split(line[len("Running `"):-1])
        environment = {}
        while parts and re.match(r"^[A-Za-z_][A-Za-z_0-9]*=", parts[0]):
            key, value = parts.pop(0).split("=", 1)
            environment[key] = value
        if parts and Path(parts[0]).resolve() == compiler:
            result.append({"argv": parts, "environment": environment})
    return result


def option(argv: list[str], name: str) -> str | None:
    for index, arg in enumerate(argv):
        if arg == name:
            return argv[index + 1]
        if arg.startswith(name + "="):
            return arg[len(name) + 1:]
    return None


def qualifier(native: Path, cargo: Path, oracle: Path, fixture: Path, output: Path,
              diagnostic_without_trace: bool = False, scope: str = "required") -> dict:
    native, cargo, oracle, fixture, output = (path.resolve() for path in (native, cargo, oracle, fixture, output))
    require(native != oracle, "native compiler and oracle must be distinct executables")
    require(scope in ("required", "safe-primitives-supplement"), "unknown qualification scope")
    require_unsafe = scope == "required"
    require(not output.exists(), "retain previous evidence and choose a new output directory")
    require(shutil.which("nm") is not None, "native object inspection requires nm")
    require(diagnostic_without_trace or shutil.which("strace") is not None,
            "functional qualification requires strace; use explicit diagnostic mode only for local checks")
    output.mkdir(parents=True)
    shutil.copy2(Path(__file__).resolve(), output / "runner-source.py")
    shutil.copytree(fixture, output / "initial-fixture")
    report = {
        "schema": "lustr.native-library-qualification.v1", "success": False, "qualified": False,
        "no_trace_qualification": diagnostic_without_trace,
        "qualification_scope": scope, "full_required_suite_passed": False,
        "scope": "Two independently compiled native libraries with typed scalar free-function exports, private helpers, unsafe-call checks and distinct crate disambiguators, consumed through real Cargo --extern arguments.",
        "full_native_rust": False, "complete_library_metadata": False, "runtime_rustc_fallback": False,
        "unimplemented_followups": ["public structs/enums and layout", "references/ownership and cross-crate drops",
                                   "generic/type/trait metadata and downstream monomorphization",
                                   "transitive dependencies and re-exports", "macros and procedural macros",
                                   "metadata-only cargo check and pipelining", "profile/target permutations"],
        "timing_scope": "Elapsed times are diagnostic only; no performance claim.",
        "process_evidence": "Cargo argv and compiler sentinels only; exec tracing unavailable and qualification disabled" if diagnostic_without_trace else "strace execve and execveat for functional compilation commands",
        "runner_sha256": digest(Path(__file__).resolve()),
        "retained_runner": "runner-source.py", "retained_initial_fixture": "initial-fixture",
        "fixture_sha256": {path.relative_to(fixture).as_posix(): digest(path)
                           for path in sorted(fixture.rglob("*")) if path.is_file()},
        "tools": {name: {"path": str(path), "sha256": digest(path)}
                  for name, path in (("native", native), ("cargo", cargo), ("oracle", oracle))},
        "commands": [], "checks": [], "artifacts": {}, "blocked": [],
    }
    if not require_unsafe:
        report["scope"] = "Supplemental safe primitive library diagnostics. The separate required unsafe producer/caller suite remains a required capability gate and is not replaced by this result."
        report["unimplemented_followups"].insert(0, "unsafe producer definitions and unsafe consumer calls; full required suite remains red")
    projects = {}
    environments = {}
    hidden = []
    sentinel = output / "compiler sentinels"
    sentinel.mkdir()
    for name in ("rustc", "rustdoc", "lust", "lust-core", "lustr", "cargo-lustr", "cargo-lust"):
        path = sentinel / name
        path.write_text('#!/bin/sh\nprintf "%s\\n" "$0" >> "$LUSTR_SENTINEL_LOG"\nexit 97\n')
        path.chmod(0o755)
    for lane, compiler in (("oracle", oracle), ("native", native)):
        project = output / (lane + " project with spaces")
        shutil.copytree(fixture, project)
        projects[lane] = project
        env = {key: value for key, value in os.environ.items()
               if not key.startswith(("CARGO_", "RUST", "LUST_", "LUSTR_", "LD_", "DYLD_"))
               and key not in ("CARGO", "LUSTC", "LIBPATH", "SHLIB_PATH", "LDR_PRELOAD")}
        env.update({"PATH": str(sentinel) + os.pathsep + "/usr/bin:/bin",
                    "CARGO_HOME": str(output / (lane + " cargo home")),
                    "RUSTUP_HOME": str(output / (lane + " no rustup")), "CARGO_NET_OFFLINE": "true",
                    "RUSTC": str(compiler), "RUSTDOC": str(sentinel / "rustdoc"),
                    "RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": "",
                    "LUSTR_SENTINEL_LOG": str(output / (lane + " unexpected compiler"))})
        environments[lane] = env

    def execute(label: str, lane: str, argv: list[str | Path], cwd: Path | None = None,
                additions: dict | None = None, trace: bool = True) -> dict:
        trace = trace and not diagnostic_without_trace
        command = [str(arg) for arg in argv]
        environment = dict(environments[lane], **(additions or {}))
        trace_path = output / f"{len(report['commands']):03d}-{label}.exec.log"
        actual = ["/usr/bin/strace", "-f", "-qq", "-s", "16384", "-e", "trace=execve,execveat", "-o", str(trace_path), "--", *command] if trace else command
        started = time.monotonic_ns()
        process = subprocess.Popen(actual, cwd=cwd or projects[lane], env=environment,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        timed_out = False
        try:
            stdout, stderr = process.communicate(timeout=180)
        except subprocess.TimeoutExpired:
            timed_out = True
            os.killpg(process.pid, signal.SIGKILL)
            stdout, stderr = process.communicate()
        row = {"label": label, "lane": lane, "argv": command, "cwd": str(cwd or projects[lane]),
               "exit_code": process.returncode, "elapsed_ns": time.monotonic_ns() - started,
               "timed_out": timed_out, "stdout": stdout.decode(errors="replace"),
               "stderr": stderr.decode(errors="replace"), "trace": trace_path.name if trace else None}
        report["commands"].append(row)
        require(not timed_out, label + " timed out")
        if trace:
            if "ptrace" in row["stderr"].lower() and "operation not permitted" in row["stderr"].lower():
                report["infrastructure_failure"] = "The execution environment denied strace ptrace; compiler process qualification did not run."
                raise RuntimeError(report["infrastructure_failure"])
            observed = []
            unfinished = {}
            for line in trace_path.read_text().splitlines():
                match = re.search(r'execve\(("(?:[^"\\]|\\.)*")', line)
                at_match = re.search(r'execveat\([^,]*,\s*("(?:[^"\\]|\\.)*")', line)
                match = match or at_match
                pid = line.split(maxsplit=1)[0]
                if match and re.search(r"= 0$", line):
                    path = ast.literal_eval(match.group(1))
                    require(Path(path).is_absolute(), label + ": unresolved relative or fd-only successful exec")
                    observed.append(path)
                elif match and "<unfinished ...>" in line:
                    unfinished[pid] = ast.literal_eval(match.group(1))
                elif "<... execve resumed>" in line or "<... execveat resumed>" in line:
                    path = unfinished.pop(pid, None)
                    require(path is not None, label + ": incomplete exec trace")
                    if re.search(r"= 0$", line):
                        require(Path(path).is_absolute(), label + ": unresolved relative or fd-only successful exec")
                        observed.append(path)
            require(not unfinished, label + ": unterminated exec trace")
            require(observed, label + ": no executed processes recorded")
            require(str(Path(command[0]).resolve()) in [str(Path(path).resolve()) for path in observed],
                    label + ": command itself absent from the exec trace")
            row["executed_processes"] = observed
            if lane == "native":
                bad = [path for path in observed if Path(path).resolve() != native
                       and (Path(path).name in ("rustc", "rustdoc", "lust", "lust-core", "lustr", "cargo-lustr", "cargo-lust")
                            or Path(path).resolve() == oracle)]
                require(not bad, label + ": unexpected compiler process " + repr(bad))
        require(not Path(environment["LUSTR_SENTINEL_LOG"]).exists(), label + ": compiler fallback sentinel was invoked")
        (output / "commands.json").write_text(json.dumps(report["commands"], indent=2) + "\n")
        return row

    def build(label: str, lane: str, expected: int = 0) -> tuple[dict, list[dict], list[dict]]:
        compiler = native if lane == "native" else oracle
        row = execute(label, lane, [cargo, "build", "--offline", "--locked", "-vv", "--message-format=json",
                                    "--target-dir", output / (lane + " target"), "-p", "lustr_library_contract"])
        events = [json.loads(line) for line in row["stdout"].splitlines() if line.strip()]
        artifacts = [event for event in events if event.get("reason") == "compiler-artifact"]
        invocations = compiler_invocations(row["stderr"], compiler)
        report.setdefault("cargo_builds", []).append({"label": label, "lane": lane, "artifacts": artifacts,
                                                       "compiler_invocations": invocations})
        require(row["exit_code"] == expected, label + f": Cargo exited {row['exit_code']}, expected {expected}: " + row["stderr"])
        return row, artifacts, invocations

    def run_app(label: str, lane: str, expected: str, executable: Path | None = None,
                expected_exit: int = 0, stderr_fragments: tuple[str, ...] = ()) -> None:
        binary = executable or output / (lane + " target") / "debug/lustr_library_contract"
        row = execute(label, lane, [binary])
        require(row["exit_code"] == expected_exit and row["stdout"] == expected, label + ": wrong executable result")
        require(all(fragment in row["stderr"] for fragment in stderr_fragments), label + ": missing expected runtime diagnostic or producer location")

    def replay(label: str, lane: str, invocation: dict, source: Path, expect_exit: int,
               extern_replacement: Path | None = None, run: bool = False,
               expected_stdout: str = "left=31 right=109 local=21\n", runtime_exit: int = 0,
               runtime_stderr: tuple[str, ...] = ()) -> dict:
        out = output / (label + " output")
        out.mkdir()
        argv = invocation["argv"].copy()
        source_args = [i for i, value in enumerate(argv) if value.endswith(".rs")]
        require(len(source_args) == 1, "consumer command does not have exactly one source root")
        argv[source_args[0]] = str(source)
        require("--out-dir" in argv, "consumer Cargo command has no output directory")
        argv[argv.index("--out-dir") + 1] = str(out)
        if extern_replacement:
            pairs = [i for i, value in enumerate(argv) if value == "--extern" and argv[i + 1].startswith("left=")]
            require(len(pairs) == 1, "consumer did not receive the left --extern artifact")
            argv[pairs[0] + 1] = "left=" + str(extern_replacement)
        row = execute(label, lane, argv, projects[lane] / "app", invocation["environment"])
        require(row["exit_code"] == expect_exit if expect_exit >= 0 else row["exit_code"] in (1, 2),
                label + ": unexpected compiler exit " + str(row["exit_code"]))
        binaries = [path for path in out.iterdir() if path.is_file() and os.access(path, os.X_OK)]
        if expect_exit == 0:
            require(len(binaries) == 1, label + ": expected one new executable")
            if run:
                run_app(label + "-runtime", lane, expected_stdout, binaries[0], runtime_exit, runtime_stderr)
        else:
            require(not binaries and row["stderr"].strip(), label + ": invalid input produced an executable or no diagnostic")
        return row

    try:
        for lane, compiler in (("oracle", oracle), ("native", native)):
            identity = execute(lane + "-identity", lane, [compiler, "-vV"])
            require(identity["exit_code"] == 0, "compiler identity failed")
            report["tools"][lane]["version"] = identity["stdout"]
        require(report["tools"]["native"]["version"].startswith("lustr-native "), "candidate is not the native compiler")
        native_identity = re.match(r"lustr-native ([^ ]+) \(Lust [^,]+, build ([0-9a-f]+)\)", report["tools"]["native"]["version"])
        require(native_identity is not None, "native compiler version does not expose its build identity")
        expected_compiler = "lustr-native-abi1:" + native_identity.group(1) + ":" + native_identity.group(2)
        expected_runtime = "lust-runtime-tinycc:" + native_identity.group(2)
        builds = {}
        for lane in ("oracle", "native"):
            _, artifacts, invocations = build(lane + "-initial", lane)
            libraries = [artifact for artifact in artifacts if artifact["target"]["kind"] == ["lib"]]
            require(len(libraries) == 2, lane + ": Cargo did not build two real library targets")
            require({call["environment"].get("CARGO_PKG_VERSION") for call in invocations
                     if option(call["argv"], "--crate-name") == "shared_symbol"} == {"0.1.0", "0.2.0"},
                    lane + ": missing separate compiler invocations for the same-named crate versions")
            consumers = [call for call in invocations if option(call["argv"], "--crate-name") == "lustr_library_contract"]
            require(len(consumers) == 1, lane + ": missing consumer compiler invocation")
            externs = [value for value in consumers[0]["argv"] if value.startswith(("left=", "right="))]
            require(len(externs) == 2 and len({value.split("=", 1)[1] for value in externs}) == 2,
                    lane + ": Cargo did not supply distinct library artifacts through --extern")
            builds[lane] = {"consumer": consumers[0], "externs": dict(value.split("=", 1) for value in externs)}
            run_app(lane + "-initial-runtime", lane, "left=31 right=109 local=21\n")
        report["checks"].append("real Cargo separately compiles and links two same-named library crate versions through distinct --extern artifacts")

        strong_symbols = []
        for alias, archive_name in builds["native"]["externs"].items():
            archive = Path(archive_name)
            members = read_ar(archive)
            metadata = [body for name, body in members if name == "lustr.meta"]
            objects = [(name, body) for name, body in members if re.fullmatch(r"unit[0-9]+\.o", name)]
            require(len(metadata) == 1 and metadata[0] and objects and len(members) == len(objects) + 1,
                    alias + ": library needs real native semantic metadata and object members, without source bodies")
            decoded = inspect_metadata(metadata[0], objects)
            report["artifacts"][alias] = {"path": str(archive), "sha256": digest(archive), "metadata": decoded}
            require(decoded["crate_name"] == "shared_symbol" and decoded["target"] == "x86_64-unknown-linux-gnu"
                    and decoded["compiler"] == expected_compiler and decoded["runtime"] == expected_runtime
                    and decoded["profile_flags"] == 6, alias + ": metadata lost producer/compiler/target/profile identity")
            expected = {"combine": (["i32", "i32"], "i32", False)}
            if alias == "left":
                expected.update({"unit_argument": (["unit", "i32"], "i32", False),
                                 "unit_result": ([], "unit", False)})
                if require_unsafe:
                    expected["unchecked_add"] = (["i32"], "i32", True)
                expected.update({name + "_roundtrip": ([kind], kind, False) for name, kind in (
                    ("bool", "bool"), ("char", "char"), ("f32", "f32"), ("f64", "f64"),
                    ("signed", "i128"), ("unsigned", "u128"))})
            actual = {"::".join(export["path"]): (export["params"], export["result"], export["unsafe"]) for export in decoded["exports"]}
            require(actual == expected and len(actual) == len(decoded["exports"]), alias + ": public metadata differs from actual typed exports or exposes private helpers")
            folder = output / (alias + " inspected objects")
            folder.mkdir()
            symbols = set()
            for name, body in objects:
                require(len(body) >= 64 and body[:6] == b"\x7fELF\x02\x01"
                        and struct.unpack_from("<HH", body, 16) == (1, 62), alias + ": object is not ELF64 x86_64 relocatable code")
                path = folder / name
                path.write_bytes(body)
                result = execute(alias + "-symbols-" + name, "native", ["/usr/bin/nm", "--defined-only", "--extern-only", path], trace=False)
                require(result["exit_code"] == 0, "object symbol inspection failed")
                symbols.update(re.findall(r"^[0-9a-fA-F]+ T (f_[0-9a-f]{16})$", result["stdout"], re.MULTILINE))
            require(symbols, alias + ": no concrete exported native function symbol")
            require({export["symbol"] for export in decoded["exports"]}.issubset(symbols), alias + ": metadata names an export without concrete object code")
            strong_symbols.append(symbols)
            report["artifacts"][alias] = {"path": str(archive), "sha256": digest(archive),
                                          "members": [{"name": name, "bytes": len(body), "sha256": hashlib.sha256(body).hexdigest()} for name, body in members],
                                          "metadata": decoded,
                                          "strong_function_symbols": sorted(symbols)}
        require(not strong_symbols[0] & strong_symbols[1], "same-named crate versions have colliding strong native symbols")
        require(report["artifacts"]["left"]["metadata"]["disambiguator"] != report["artifacts"]["right"]["metadata"]["disambiguator"],
                "same-named crate versions lost their distinct metadata identities")
        report["checks"].append("native archives contain versioned semantic metadata and ELF relocatable code with disjoint strong exported symbols")

        for lane in ("oracle", "native"):
            for dependency in ("left", "right"):
                source = projects[lane] / dependency / "src"
                destination = output / (lane + " sealed " + dependency + " source")
                source.rename(destination)
                hidden.append((source, destination))
            consumer = builds[lane]["consumer"]
            replay(lane + "-source-hidden", lane, consumer, projects[lane] / "app/src/main.rs", 0, run=True)
            positives = [("primitive_abi", "primitive ABI passed\n"), ("conditional_consumer", "conditional=31\n"),
                         ("function_imports", "imports=31,109\n")]
            if require_unsafe:
                positives.append(("unsafe_allowed", "unsafe=42\n"))
            for case, expected_stdout in positives:
                replay(lane + "-" + case, lane, consumer, projects[lane] / "cases" / (case + ".rs"),
                       0, run=True, expected_stdout=expected_stdout)
            negatives = ["private", "type_mismatch", "arity_mismatch", "result_mismatch"]
            if require_unsafe:
                negatives.append("unsafe_forbidden")
            for case in negatives:
                replay(lane + "-reject-" + case, lane, consumer, projects[lane] / "cases" / (case + ".rs"), 1)
            replay(lane + "-imported-overflow", lane, consumer, projects[lane] / "cases/imported_overflow.rs",
                   0, run=True, expected_stdout="", runtime_exit=101,
                   runtime_stderr=("attempt to add with overflow", "left/src/lib.rs:2"))
        report["checks"].append("consumer builds from actual dependency artifacts while producer source is absent; public signatures, privacy, scalar ABI boundaries and unit arguments remain checked")
        if require_unsafe:
            report["checks"].append("unsafe producer contract is preserved, safe caller rejected and unsafe caller executes correctly")

        for case in ("public_generic", "public_type", "public_const", "public_macro", "public_must_use", "public_deprecated", "public_no_mangle"):
            for lane, compiler in (("oracle", oracle), ("native", native)):
                artifact = output / (lane + "-unsupported-" + case + ".rlib")
                result = execute(lane + "-export-" + case, lane,
                                 [compiler, "--crate-name", "unsupported", "--edition=2024",
                                  projects[lane] / "cases" / (case + ".rs"), "--crate-type", "rlib", "--emit=link", "-o", artifact])
                if lane == "oracle":
                    require(result["exit_code"] == 0 and artifact.is_file(), case + ": fixture is not a valid Rust library")
                else:
                    require(result["exit_code"] in (1, 2) and result["stderr"].strip() and not artifact.exists(),
                            case + ": unrepresented public contract was silently omitted or produced an artifact")
        report["checks"].append("valid public generic, type, constant, macro and unrepresented attribute contracts are explicit native gaps; none is silently omitted from a successful artifact")

        left = Path(builds["native"]["externs"]["left"])
        original = read_ar(left)
        mutations = output / "native invalid artifacts"
        mutations.mkdir()
        roundtrip = mutations / "roundtrip.rlib"
        write_ar(roundtrip, original)
        replay("native-archive-roundtrip", "native", builds["native"]["consumer"], projects["native"] / "app/src/main.rs", 0, roundtrip, run=True)
        cases = {
            "missing-metadata": [(name, body) for name, body in original if name != "lustr.meta"],
            "missing-object": [(name, body) for name, body in original if name == "lustr.meta"],
            "corrupt-metadata": [(name, b"invalid native metadata" if name == "lustr.meta" else body) for name, body in original],
            "truncated-object": [(name, body[:8] if name.endswith(".o") else body) for name, body in original],
        }
        for name, members in cases.items():
            archive = mutations / (name + ".rlib")
            write_ar(archive, members)
            replay("native-reject-" + name, "native", builds["native"]["consumer"], projects["native"] / "app/src/main.rs", -1, archive)
        replay("native-reject-foreign-rust-library", "native", builds["native"]["consumer"],
               projects["native"] / "app/src/main.rs", -1, Path(builds["oracle"]["externs"]["left"]))
        report["checks"].append("missing/corrupt metadata or native objects and foreign Rust archives fail without a new executable; valid rebuilt ar control executes")

        for source, destination in reversed(hidden):
            destination.rename(source)
        hidden.clear()
        for lane in ("oracle", "native"):
            _, artifacts, _ = build(lane + "-no-change", lane)
            require(len(artifacts) == 3 and all(artifact["fresh"] for artifact in artifacts), "unchanged Cargo units were not fresh")
            left_source = projects[lane] / "left/src/lib.rs"
            left_source.write_text(left_source.read_text().replace("value + 5", "value + 6"))
            _, artifacts, _ = build(lane + "-private-body-edit", lane)
            require(any(not artifact["fresh"] for artifact in artifacts), "private body edit did not rebuild")
            run_app(lane + "-private-body-edit-runtime", lane, "left=33 right=109 local=21\n")
            left_source.write_text(left_source.read_text().replace("first: i32, second: i32", "first: i32").replace("* 2 + second", "* 2 + 1"))
            build(lane + "-signature-invalidates-consumer", lane, 101)
            main = projects[lane] / "app/src/main.rs"
            main.write_text(main.read_text().replace("left::combine(10, 1)", "left::combine(10)"))
            build(lane + "-signature-recovery", lane)
            run_app(lane + "-signature-recovery-runtime", lane, "left=33 right=109 local=21\n")
        report["checks"].append("Cargo no-op, private-body edit, signature-to-error transition and valid recovery preserve dependency invalidation and runtime behavior")
        for name, tool in report["tools"].items():
            require(digest(Path(tool["path"])) == tool["sha256"], name + ": executable changed during qualification")
        report["success"] = True
        report["full_required_suite_passed"] = require_unsafe
        report["qualified"] = not diagnostic_without_trace and require_unsafe
    except BaseException as error:
        report["failure"] = f"{type(error).__name__}: {error}"
        report["blocked"] = ["Unreached library gates remain unqualified; failure is not a native completion result."]
        raise
    finally:
        for source, destination in reversed(hidden):
            if destination.exists() and not source.exists():
                destination.rename(source)
        (output / "commands.json").write_text(json.dumps(report["commands"], indent=2) + "\n")
        (output / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--cargo", type=Path, required=True)
    parser.add_argument("--oracle", type=Path, required=True)
    parser.add_argument("--fixture", type=Path, default=Path(__file__).resolve().parents[1] / "tests/native-library")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--diagnostic-without-trace", action="store_true",
                        help="run local correctness probes without ptrace; never sets qualified=true")
    args = parser.parse_args()
    qualifier(args.native, args.cargo, args.oracle, args.fixture, args.output, args.diagnostic_without_trace)


if __name__ == "__main__":
    main()
