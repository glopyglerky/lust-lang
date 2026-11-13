#!/usr/bin/env python3
"""Native daemon base-library lifetime regression.

Usage: python3 scripts/test-base-lifetime.py [path/to/lust]
Default compiler: LUST_BIN, else target/release/lust. Requires rustc and
otool on Darwin or readelf on Linux. Unsupported hosts fail setup.

Checks actual warm builds against cold native/Rust output, native loader
metadata, immutable replacement, missing-base healing, and kept binaries
after daemon shutdown. Both semantic oracles run throughout the warm
sequence. Commands, relevant metadata, and oracle/daemon logs go to stdout.
"""

import argparse
import hashlib
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time

import oracle_log


ROOT = Path(__file__).resolve().parent.parent
DARWIN = sys.platform == "darwin"
SUFFIX = ".dylib" if DARWIN else ".so"


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def run(args, work, env, show=True):
    args = [str(arg) for arg in args]
    print("+ " + shlex.join(args), flush=True)
    result = subprocess.run(args, cwd=work, env=env, capture_output=True, text=True,
                            errors="replace", timeout=60)
    if result.stderr:
        print(result.stderr, end="" if result.stderr.endswith("\n") else "\n", flush=True)
    if result.stdout and (show or result.returncode):
        print(result.stdout, end="" if result.stdout.endswith("\n") else "\n", flush=True)
    require(result.returncode == 0, f"command exited {result.returncode}: {shlex.join(args)}")
    return result


def macho_fields(path, work, env):
    text = run(["otool", "-l", path], work, env, show=False).stdout
    fields = {}
    for block in re.split(r"(?m)^Load command \d+\s*$", text)[1:]:
        command = re.search(r"(?m)^\s*cmd (LC_\w+)\s*$", block)
        if not command or command[1] not in ("LC_LOAD_DYLIB", "LC_ID_DYLIB", "LC_RPATH"):
            continue
        value = re.search(r"(?m)^\s*(?:name|path) (.+) \(offset \d+\)\s*$", block)
        require(value is not None, f"unreadable Mach-O load command in {path}: {block}")
        fields.setdefault(command[1], []).append(value[1])
        print(f"metadata {path.name}: {command[1]}={value[1]}", flush=True)
    return fields


def base_dependency(exe, base_dir, owner, work, env):
    if DARWIN:
        fields = macho_fields(exe, work, env)
        loads = fields.get("LC_LOAD_DYLIB", [])
        deps = [name for name in loads if "libbase-" in name]
        require(str(base_dir) in fields.get("LC_RPATH", []),
                f"{exe.name}: no matching LC_RPATH for {base_dir}: {fields}")
        prefix = "@rpath/"
    else:
        text = run(["readelf", "-d", exe], work, env).stdout
        deps = [name for name in re.findall(r"\(NEEDED\).*?\[([^]]+)\]", text) if "libbase-" in name]
        rpaths = re.findall(r"\((?:RUNPATH|RPATH)\).*?\[([^]]+)\]", text)
        require(any(str(base_dir) in path.split(":") for path in rpaths),
                f"{exe.name}: no matching runtime search path for {base_dir}")
        prefix = ""
    require(len(deps) == 1, f"{exe.name}: expected one daemon base dependency, got {deps}")
    pattern = re.escape(prefix) + rf"libbase-{owner}-[0-9a-f]{{32}}" + re.escape(SUFFIX)
    require(re.fullmatch(pattern, deps[0]) is not None,
            f"{exe.name}: dependency is not an immutable base owned by daemon {owner}: {deps[0]}")
    base = base_dir / deps[0].removeprefix(prefix)
    require(base.is_file(), f"{exe.name}: linked base does not exist: {base}")
    if DARWIN:
        fields = macho_fields(base, work, env)
        require(fields.get("LC_ID_DYLIB") == [deps[0]],
                f"{base.name}: install name does not equal the executable's load name: {fields}")
    print(f"base {exe.name}: {base.name}", flush=True)
    return base


def output(exe, expected, work, env):
    got = run([exe], work, env).stdout
    require(got == expected, f"{exe.name}: expected {expected!r}, got {got!r}")


def case(lust, work):
    source = work / "main.rs"
    socket = work / "d.sock"
    # Exercise the existing rpath option's quoting as well as its native
    # load-command emission. The socket itself stays in a short path.
    cache = work / 'cache space # café "quoted"'
    base_dir = cache / "bases"
    tmp = work / "tmp"
    tmp.mkdir()
    oracle_path = work / "oracle.log"
    log_path = work / "daemon.log"
    env = dict(os.environ, LUST_DAEMON_SOCKET=str(socket), LUST_CACHE_DIR=str(cache),
               LUST_NO_CACHE="1", LUST_DAEMON_IDLE="0", LUST_STATS="1", TMPDIR=str(tmp),
               LUST_CHECK_ITEMPATCH="1", LUST_CHECK_RESOLVE="1",
               LUST_ITEMPATCH_LOG=str(oracle_path), LUST_RESOLVE_LOG=str(oracle_path))
    for key in ("LUST_NO_DAEMON", "LUST_FRONT_STATE", "LUST_FRONT_STATE_VERBOSE"):
        env.pop(key, None)
    problems = []
    daemon = None
    with log_path.open("w") as log:
        try:
            print(f"+ {shlex.quote(str(lust))} daemon", flush=True)
            daemon = subprocess.Popen([str(lust), "daemon"], cwd=work, env=env,
                                      stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            deadline = time.monotonic() + 10
            while not socket.is_socket():
                require(daemon.poll() is None, "daemon exited before binding its socket")
                require(time.monotonic() < deadline, "daemon did not bind its socket")
                time.sleep(0.01)
            bases = []
            old_hash = None
            for version, value in enumerate((41, 42)):
                # A static-data edit forces a new base, while keeping the
                # crate, function identities, and daemon session fixed.
                source.write_text(
                    "mod values {\n"
                    f"    pub static ANSWER: i32 = {value};\n"
                    "    pub fn read() -> i32 { ANSWER }\n"
                    "}\nfn main() { println!(\"{}\", values::read()); }\n", encoding="utf-8")
                stamp = time.time() + version + 1
                os.utime(source, (stamp, stamp))
                expected = f"{value}\n"
                exe = work / f"warm-{version}"
                run([lust, "check", source], work, env)
                run([lust, "build", source, "-o", exe], work, env)
                require(daemon.poll() is None, "the requested warm daemon exited during a build")
                base = base_dependency(exe, base_dir, daemon.pid, work, env)
                bases.append(base)
                output(exe, expected, work, env)
                # Oracle counts below must come from the warm daemon.
                cold_env = dict(env, LUST_NO_DAEMON="1", LUST_CHECK_ITEMPATCH="0", LUST_CHECK_RESOLVE="0")
                cold_env.pop("LUST_ITEMPATCH_LOG", None)
                cold_env.pop("LUST_RESOLVE_LOG", None)
                cold = work / f"cold-{version}"
                run([lust, "build", source, "-o", cold], work, cold_env)
                output(cold, expected, work, cold_env)
                reference = work / f"rust-{version}"
                run(["rustc", "--edition", "2024", "-C", "overflow-checks=on", source, "-o", reference], work, env)
                output(reference, expected, work, env)
                if version == 0:
                    old_hash = hashlib.sha256(base.read_bytes()).digest()
                else:
                    require(bases[0] != base, "the static edit did not replace the base")
                    require(hashlib.sha256(bases[0].read_bytes()).digest() == old_hash,
                            "replacement changed the older immutable base")
                    output(work / "warm-0", "41\n", work, env)

            # Delete only this test's current library. The unchanged warm
            # build must heal from retained objects under the same name.
            bases[1].unlink()
            healed = work / "healed"
            built = run([lust, "build", source, "-o", healed], work, env)
            stats = re.findall(r"(?m)^lust-stats: (.+)$", built.stderr)
            require(stats, "healing build did not report warm compilation counters")
            counters = dict(re.findall(r"(\w+)=(\d+)", stats[-1]))
            require(counters.get("table_rebuilt") == "0" and counters.get("dirty_units") == "0"
                    and int(counters.get("units_compiled", "999")) <= 1,
                    f"healing rebuilt the base objects instead of retaining them: {stats[-1]}")
            require(base_dependency(healed, base_dir, daemon.pid, work, env) == bases[1],
                    "healing changed the base's immutable name")
            output(work / "warm-1", "42\n", work, env)
            output(healed, "42\n", work, env)

            print("+ terminate daemon and wait for cleanup", flush=True)
            daemon.terminate()
            daemon.wait(timeout=15)
            require(not socket.exists(), "daemon did not remove its socket on shutdown")
            require(not (tmp / f"lust-daemon-{daemon.pid}").exists(), "daemon did not remove its temporary object directory")
            require(hashlib.sha256(bases[0].read_bytes()).digest() == old_hash,
                    "shutdown changed or removed the older immutable base")
            for name, expected in (("warm-0", "41\n"), ("warm-1", "42\n"), ("healed", "42\n")):
                output(work / name, expected, work, env)
        except (OSError, RuntimeError, subprocess.SubprocessError) as exc:
            problems.append(str(exc))
        finally:
            if daemon is not None and daemon.poll() is None:
                daemon.terminate()
                try:
                    daemon.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    daemon.kill()
                    daemon.wait(timeout=10)
                    problems.append("daemon required SIGKILL during cleanup")

    text = log_path.read_text(errors="replace")
    print("daemon log:\n" + text, flush=True)
    problems.extend(line for line in text.splitlines() if line.startswith("guard:"))
    starts = len(re.findall(r"(?m)^guard-check: start ", text))
    dones = len(re.findall(r"(?m)^guard-check: done ", text))
    if dones < starts:
        problems.append(f"{starts - dones} background guard checks did not finish")
    oracle_text = oracle_path.read_text(errors="replace") if oracle_path.exists() else ""
    print("semantic oracle log:\n" + oracle_text, flush=True)
    _, failures = oracle_log.collect(str(oracle_path))
    problems.extend(failures)
    for prefix in ("itempatch oracle: ok", "resolve oracle: ok"):
        count = sum(line.startswith(prefix) for line in oracle_text.splitlines())
        print(f"oracle checks {prefix}: {count}", flush=True)
        if count == 0:
            problems.append(f"no {prefix} checks recorded")
    for problem in problems:
        print(f"FAIL base lifetime: {problem}", file=sys.stderr)
    if problems:
        return 1
    print(f"PASS base lifetime ({sys.platform}): native metadata, replacement, healing, shutdown", flush=True)
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("lust", nargs="?", default=os.environ.get("LUST_BIN", str(ROOT / "target/release/lust")))
    args = parser.parse_args()
    lust = Path(args.lust).resolve()
    tool = "otool" if DARWIN else "readelf"
    if sys.platform not in ("darwin", "linux") or not os.access(lust, os.X_OK) or any(shutil.which(t) is None for t in (tool, "rustc")):
        print(f"setup error: requires Linux/Darwin, an executable compiler ({lust}), {tool}, and rustc", file=sys.stderr)
        return 2
    # /tmp keeps the Unix socket below both native sockaddr_un limits.
    with tempfile.TemporaryDirectory(prefix="lust-base-", dir="/tmp") as directory:
        return case(lust, Path(directory).resolve())


if __name__ == "__main__":
    sys.exit(main())
