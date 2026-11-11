#!/usr/bin/env python3
"""Exercise real native request-cache hits and invalidation, without timing claims.

A private TinyCC helper records invocations and can be disabled through a test
file. That fault proves a warm native cache hit does not run the backend again.
Only this functional test package is instrumented; production timings must use
the unmodified compiler and runtime.
"""
from __future__ import annotations

import argparse
import errno
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", type=Path, required=True)
    parser.add_argument("--tcc-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    root = args.output.resolve()
    root.mkdir(parents=True, exist_ok=False)
    native = args.native.resolve()
    report = {"schema": "lustr.native-request-cache-functional.v1", "success": False,
              "qualified": False, "performance_claim": False,
              "scope": "Actual native compiler with a private instrumented TinyCC helper; functional only.",
              "native_sha256": sha(native), "runner_sha256": sha(Path(__file__)),
              "commands": [], "checks": [], "failures": []}
    package = root / "package"
    (package / "bin").mkdir(parents=True)
    compiler = package / "bin/lustr-native"
    shutil.copy2(native, compiler)
    tcc = package / "lib/lust/tcc"
    shutil.copytree(args.tcc_root.resolve(), tcc)
    helper = tcc / "bin/tcc"
    helper.rename(helper.with_name("tcc-real"))
    helper.write_text("""#!/usr/bin/python3
import os,pathlib,sys
here=pathlib.Path(__file__).resolve()
root=here.parents[4]
with (root/'backend-invocations').open('ab',buffering=0) as log:
    log.write((str(os.getpid())+'\\n').encode())
if (root/'backend-disabled').exists():
    sys.stderr.write('test backend disabled\\n')
    sys.exit(77)
os.execv(str(here.with_name('tcc-real')),[str(here.with_name('tcc-real'))]+sys.argv[1:])
""")
    helper.chmod(0o755)
    disabled = package / "backend-disabled"
    invoked = package / "backend-invocations"
    base_env = {key: value for key, value in os.environ.items() if not key.startswith(("LUST_", "LUSTR_"))}
    logs = root / "logs"
    logs.mkdir()

    def check(value, message):
        if not value:
            raise ValueError(message)
        report["checks"].append(message)

    def execute(label, command, cwd, env, umask=-1):
        start = time.monotonic_ns()
        result = subprocess.run([str(arg) for arg in command], cwd=cwd, env=env,
                                stdin=subprocess.DEVNULL, capture_output=True, timeout=60, umask=umask)
        index = len(report["commands"])
        out, err = logs / f"{index:04}.stdout", logs / f"{index:04}.stderr"
        out.write_bytes(result.stdout)
        err.write_bytes(result.stderr)
        report["commands"].append({"label": label, "argv": [str(arg) for arg in command],
                                   "cwd": str(cwd), "exit_code": result.returncode,
                                   "elapsed_ns": time.monotonic_ns() - start,
                                   "stdout": str(out.relative_to(root)), "stderr": str(err.relative_to(root)),
                                   "stdout_sha256": sha(out), "stderr_sha256": sha(err)})
        return result

    def project(name, text):
        directory = root / name
        directory.mkdir()
        (directory / "main.rs").write_text(text)
        (directory / "tmp").mkdir()
        env = dict(base_env, LUST_NO_DAEMON="1", LUST_CACHE_DIR=str(directory / "cache"), TMPDIR=str(directory / "tmp"))
        return directory, env

    def build(directory, env, *extra, umask=-1):
        return execute(directory.name + "-compile", [compiler, "--crate-name", "cache_probe", "--crate-type", "bin",
                       "--edition=2024", "--emit=link,dep-info", "--out-dir", directory,
                       "--error-format=json", "--json=artifacts,unused-externs-silent",
                       "--force-warn=unused_crate_dependencies", *extra, directory / "main.rs"], directory, env, umask)

    def markers(directory):
        return sorted((directory / "cache").glob("req-native-*"))

    def run(directory, env, expected):
        result = execute(directory.name + "-run", [directory / "cache_probe"], directory, env)
        check(result.returncode == 0 and result.stdout == expected.encode(), directory.name + " runtime output matches")

    def remove_output(directory):
        (directory / "cache_probe").unlink(missing_ok=True)
        (directory / "cache_probe.d").unlink(missing_ok=True)

    scalar = 'unsafe fn add(value: i32) -> i32 { value + 1 }\nfn main() { println!("{}", unsafe { add(41) }); }\n'
    try:
        directory, env = project("root-only-unsafe", scalar)
        first = build(directory, env)
        check(first.returncode == 0, "initial native compile succeeds")
        check(invoked.is_file() and invoked.stat().st_size > 0, "initial native compile actually invokes the test backend")
        check(len(markers(directory)) == 1, "complete root-only success creates one cache admission record")
        before = invoked.read_bytes()
        dep_info = (directory / "cache_probe.d").read_bytes()
        run(directory, env, "42\n")
        remove_output(directory)
        disabled.touch()
        warm = build(directory, env)
        check(warm.returncode == 0 and invoked.read_bytes() == before, "warm hit succeeds with the backend disabled and invokes no helper")
        check((directory / "cache_probe.d").read_bytes() == dep_info, "warm hit regenerates complete identical Cargo dep-info")
        notifications = [json.loads(line) for line in warm.stderr.splitlines()]
        check(any(row.get("$message_type") == "artifact" and row.get("artifact") == str(directory / "cache_probe") for row in notifications),
              "warm hit reports the requested final artifact path")
        check(any(row.get("$message_type") == "unused_extern" and row.get("unused_extern_names") == [] for row in notifications),
              "warm hit preserves unused-extern reporting")
        run(directory, env, "42\n")
        disabled.unlink()

        # A corrupted executable must never be published or treated as success.
        cached = list((directory / "cache").glob("req-*.exe"))
        check(len(cached) == 1, "one actual upstream cached executable exists")
        cached[0].write_bytes(b"corrupt executable")
        remove_output(directory)
        disabled.touch()
        corrupt = build(directory, env)
        check(corrupt.returncode != 0 and not (directory / "cache_probe").exists(), "corrupt cache forces recompilation and never publishes damaged output")
        disabled.unlink()
        check(build(directory, env).returncode == 0, "native recompilation repairs a corrupt executable cache")
        run(directory, env, "42\n")
        # An incomplete cached response likewise cannot become a successful hit.
        cached[0].with_suffix("").write_bytes(b"incomplete response")
        remove_output(directory)
        check(build(directory, env).returncode == 0, "malformed cached response is recomputed")
        run(directory, env, "42\n")
        cached[0].chmod(0o644)
        remove_output(directory)
        disabled.touch()
        changed_mode = build(directory, env)
        check(changed_mode.returncode != 0 and not (directory / "cache_probe").exists(),
              "non-executable cached permissions cannot produce a successful artifact")
        disabled.unlink()
        check(build(directory, env).returncode == 0, "recompilation repairs corrupted executable permissions")
        run(directory, env, "42\n")
        marker = markers(directory)[0]
        marker.write_bytes(b"incomplete admission")
        remove_output(directory)
        disabled.touch()
        bad_marker = build(directory, env)
        check(bad_marker.returncode != 0 and not (directory / "cache_probe").exists(),
              "corrupted admission record cannot authorize a cache hit")
        disabled.unlink()
        check(build(directory, env).returncode == 0, "complete recompilation repairs admission metadata")
        run(directory, env, "42\n")
        remove_output(directory)
        (directory / "cache_probe.d").mkdir()
        disabled.touch()
        before = invoked.read_bytes()
        dep_failure = build(directory, env)
        check(dep_failure.returncode != 0 and not (directory / "cache_probe").exists() and invoked.read_bytes() == before,
              "dep-info failure after a warm hit is a compiler failure and removes the unqualified executable")
        disabled.unlink()
        (directory / "cache_probe.d").rmdir()
        check(build(directory, env).returncode == 0, "warm cache remains usable after correcting the dep-info destination")
        run(directory, env, "42\n")

        (directory / "main.rs").write_text(scalar.replace("add(41)", "add(90)"))
        check(build(directory, env).returncode == 0, "changed source invalidates the successful whole request")
        run(directory, env, "91\n")
        prior_markers = {path: path.read_bytes() for path in markers(directory)}
        (directory / "main.rs").write_text(scalar.replace("unsafe { add(41) }", "add(41)"))
        remove_output(directory)
        rejected = build(directory, env)
        check(rejected.returncode == 1 and not (directory / "cache_probe").exists(), "invalid unsafe edit rejects instead of replaying cached success")
        check(prior_markers == {path: path.read_bytes() for path in markers(directory)}, "rejected source does not create a success admission record")

        directory, env = project("compiler-replacement", scalar)
        check(build(directory, env).returncode == 0, "compiler-identity seed compile succeeds")
        replacement = compiler.with_name("replacement")
        previous = compiler.stat()
        shutil.copy2(compiler, replacement)
        replacement.replace(compiler)
        check(compiler.stat().st_mtime_ns == previous.st_mtime_ns and compiler.stat().st_ino != previous.st_ino,
              "replacement preserves native compiler timestamp while changing inode")
        remove_output(directory)
        disabled.touch()
        replaced = build(directory, env)
        check(replaced.returncode != 0 and not (directory / "cache_probe").exists(),
              "same-path timestamp-preserving compiler replacement invalidates the cached executable")
        disabled.unlink()
        check(build(directory, env).returncode == 0, "replacement compiler builds a new successful cache entry")
        run(directory, env, "42\n")
        previous = compiler.stat()
        # The Linux exit path can keep the executable's write exclusion briefly
        # after waitpid reaps its leader. Wait only for that specific condition.
        deadline = time.monotonic() + 2
        while True:
            try:
                stream = compiler.open("r+b")
                break
            except OSError as error:
                if error.errno != errno.ETXTBSY or time.monotonic() >= deadline:
                    raise
                time.sleep(0.005)
        with stream:
            byte = stream.read(1)
            stream.seek(0)
            stream.write(byte)
        os.utime(compiler, ns=(previous.st_atime_ns, previous.st_mtime_ns))
        check(compiler.stat().st_ino == previous.st_ino and compiler.stat().st_mtime_ns == previous.st_mtime_ns
              and compiler.stat().st_ctime_ns != previous.st_ctime_ns,
              "in-place same-size rewrite preserves inode and mtime but changes ctime")
        remove_output(directory)
        disabled.touch()
        overwritten = build(directory, env)
        check(overwritten.returncode != 0 and not (directory / "cache_probe").exists(),
              "in-place compiler rewrite with restored timestamp invalidates cache by ctime")
        disabled.unlink()
        check(build(directory, env).returncode == 0, "rewritten compiler recomputes successfully")
        run(directory, env, "42\n")

        # The upstream cache identity must still cover the actual runtime tree.
        header = tcc / "lib/tcc/include/stddef.h"
        original_header = header.read_bytes()
        header.write_bytes(original_header + b"\n/* request-cache identity test */\n")
        remove_output(directory)
        disabled.touch()
        changed_runtime = build(directory, env)
        check(changed_runtime.returncode != 0 and not (directory / "cache_probe").exists(),
              "changed TinyCC runtime input invalidates native whole-request cache")
        disabled.unlink()
        header.write_bytes(original_header)

        directory, env = project("caller-umask", scalar)
        check(build(directory, env, umask=0o022).returncode == 0, "initial umask compile succeeds")
        check((directory / "cache_probe").stat().st_mode & 0o7777 == 0o755, "initial executable follows umask 022")
        remove_output(directory)
        disabled.touch()
        changed_mask = build(directory, env, umask=0o077)
        check(changed_mask.returncode != 0 and not (directory / "cache_probe").exists(),
              "tightened caller umask invalidates the old cache admission")
        disabled.unlink()
        check(build(directory, env, umask=0o077).returncode == 0, "tightened umask recompiles successfully")
        check((directory / "cache_probe").stat().st_mode & 0o7777 == 0o700, "recompiled executable follows umask 077")
        remove_output(directory)
        before = invoked.read_bytes()
        disabled.touch()
        check(build(directory, env, umask=0o077).returncode == 0 and invoked.read_bytes() == before,
              "same-umask warm hit needs no backend compilation")
        check((directory / "cache_probe").stat().st_mode & 0o7777 == 0o700, "warm hit preserves the caller's executable permissions")
        disabled.unlink()
        run(directory, env, "42\n")

        for name, source in (
            ("spaced-env", 'fn main() { println!("{}", env /* comment */ ! ("NATIVE_CACHE_VALUE")); }\n'),
            ("spaced-option-env", 'fn main() { println!("{}", option_env /* comment */ ! ("NATIVE_CACHE_VALUE").unwrap()); }\n'),
        ):
            directory, env = project(name, source)
            for value in ("first", "second"):
                env["NATIVE_CACHE_VALUE"] = value
                check(build(directory, env).returncode == 0, name + " compile succeeds")
                run(directory, env, value + "\n")
            check(not markers(directory), name + " actual expansion dependency prohibits whole-request admission")

        directory, env = project("spaced-include", 'fn main() { println!("{}", include_str /* comment */ ! ("value.txt")); }\n')
        for value in ("first", "second"):
            (directory / "value.txt").write_text(value)
            check(build(directory, env).returncode == 0, "included-file compile succeeds")
            run(directory, env, value + "\n")
        check(not markers(directory), "actual included-file dependency prohibits whole-request admission")

        directory, env = project("module-edit", 'mod child;\nfn main() { println!("{}", child::answer()); }\n')
        for value in (42, 91):
            (directory / "child.rs").write_text(f"pub fn answer() -> i32 {{ {value} }}\n")
            check(build(directory, env).returncode == 0, "module compile succeeds")
            run(directory, env, str(value) + "\n")
        check(not markers(directory), "actual module source map prohibits whole-request admission")

        directory, env = project("profile", 'fn main() { println!("{}", cfg!(debug_assertions)); }\n')
        for setting, expected in (("yes", "true\n"), ("no", "false\n"), ("yes", "true\n")):
            check(build(directory, env, "-C", "debug-assertions=" + setting).returncode == 0, "profile invocation succeeds")
            run(directory, env, expected)

        # Cargo already supplied the effective source compilation request. A
        # source-adjacent manifest must not cause cache lookup/store to run
        # another Cargo build or independently rediscover build-script inputs.
        directory, env = project("parent-manifest", scalar)
        (directory / "Cargo.toml").write_text('[package]\nname="parent_manifest"\nversion="0.1.0"\nedition="2024"\n'
                                             'build="build.rs"\n[[bin]]\nname="cache_probe"\npath="main.rs"\n[workspace]\n')
        (directory / "build.rs").write_text('compile_error!("Cargo owns build scripts");\n')
        sentinels = directory / "sentinels"
        sentinels.mkdir()
        cargo_log = directory / "unexpected-cargo"
        cargo = sentinels / "cargo"
        cargo.write_text('#!/usr/bin/python3\nfrom pathlib import Path\n'
                         + f'with Path({str(cargo_log)!r}).open("a") as stream: stream.write("cargo called\\n")\n'
                         + 'raise SystemExit(97)\n')
        cargo.chmod(0o755)
        env["PATH"] = str(sentinels) + os.pathsep + env.get("PATH", "")
        check(build(directory, env).returncode == 0, "source compilation under a parent manifest succeeds")
        check(not cargo_log.exists(), "cold cache admission never invokes Cargo or its build script")
        run(directory, env, "42\n")
        remove_output(directory)
        before = invoked.read_bytes()
        disabled.touch()
        check(build(directory, env).returncode == 0 and invoked.read_bytes() == before,
              "parent-manifest warm hit reuses the completed native request")
        check(not cargo_log.exists(), "warm cache lookup never invokes Cargo or its build script")
        run(directory, env, "42\n")
        (directory / "Cargo.toml").write_text('[package]\nname="different_parent"\nversion="9.0.0"\nedition="2024"\n[workspace]\n')
        (directory / "build.rs").write_text('compile_error!("changed outside the effective compiler request");\n')
        (directory / ".cargo").mkdir()
        (directory / ".cargo/config.toml").write_text('[build]\nrustflags=["--cfg=unrelated_parent_setting"]\n')
        remove_output(directory)
        check(build(directory, env).returncode == 0 and invoked.read_bytes() == before,
              "parent manifest/config edits do not alter an identical effective native request")
        check(not cargo_log.exists(), "changed parent inputs never reenter Cargo through the cache")
        disabled.unlink()
        run(directory, env, "42\n")

        directory, env = project("cache-disabled", scalar)
        env["LUST_NO_CACHE"] = "1"
        check(build(directory, env).returncode == 0, "cache-disabled native compile succeeds")
        check(not markers(directory), "cache-disabled compile creates no admission record")
        run(directory, env, "42\n")
        report["success"] = True
    except Exception as error:
        report["failures"].append(type(error).__name__ + ": " + str(error))
    finally:
        disabled.unlink(missing_ok=True)
        (root / "report.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"success": report["success"], "commands": len(report["commands"]), "checks": len(report["checks"]),
                      "failures": report["failures"], "report": str(root / "report.json")}))
    return 0 if report["success"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
