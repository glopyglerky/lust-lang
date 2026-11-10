#!/usr/bin/env python3
"""Focused, evidence-retaining diagnostic. Does not replace the upstream CI gate."""
import concurrent.futures
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time

PINNED_TREE = "0ab1b09815802894e6078de0af59273cbdf46cb0"
FLAGS = ["-O1", "-w", "-Werror=incompatible-pointer-types",
         "-fno-sanitize-recover=undefined", "-fsanitize=address,undefined"]
CLEAR = ["PANIC", "RELEASE", "EH", "LL", "CC_DRIVER", "CC_TARGET",
         "DIFFTEST_SHARD", "DIFFTEST_JOBS", "LUST_UNWIND_TABLES",
         "LUST_RELEASE_C", "LUST_RELEASE_BACKEND", "LUST_STRICT"]

def save(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


class WatchdogExpired(RuntimeError):
    pass

def live_session_pids(session_id):
    """Linux runner only. GNU timeout changes process groups, not the session."""
    result = []
    for entry in Path("/proc").iterdir():
        if not entry.name.isdigit():
            continue
        pid = int(entry.name)
        try:
            state = (entry / "stat").read_text().rsplit(")", 1)[1].split()[0]
            if state != "Z" and os.getsid(pid) == session_id:
                result.append(pid)
        except (FileNotFoundError, ProcessLookupError):
            continue
    return result

def stop_session(session_id):
    # Each launch has its own session. Re-scan to catch a fork concurrent with
    # the first scan. Never launch another measurement after a watchdog fires.
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        pids = live_session_pids(session_id)
        if not pids:
            return True
        for pid in pids:
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
        time.sleep(0.05)
    return not live_session_pids(session_id)

class Recorder:
    def __init__(self, output):
        self.output = output
        output.mkdir(parents=True, exist_ok=False)
        self.env = os.environ.copy()
        for key in CLEAR:
            self.env.pop(key, None)
        self.env.update(RUN_TIMEOUT="10", CASE_TIMEOUT="30", SAN_CC_LIMIT="20",
                        DIFFTEST_RUSTC_CACHE="off", LUST_NO_DAEMON="1", LC_ALL="C")

    def run(self, label, args, cwd, limit=None, env=None, watchdog=600):
        prefix = self.output / label
        command = list(map(str, args))
        if limit is not None:
            command = ["timeout", str(limit)] + command
        command = ["/usr/bin/time", "-v", "-o", str(prefix) + ".time"] + command
        selected_env = dict(self.env)
        selected_env.update(env or {})
        record = {"label": label, "command": command, "cwd": str(cwd),
                  "timeout_seconds": limit, "outer_watchdog_seconds": watchdog,
                  "environment_overrides": env or {},
                  "started_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
                  "load_before": os.getloadavg(), "status": "started"}
        save(Path(str(prefix) + ".json"), record)
        started = time.monotonic()
        with open(str(prefix) + ".stdout", "wb") as out, open(str(prefix) + ".stderr", "wb") as err:
            child = subprocess.Popen(command, cwd=cwd, env=selected_env, stdout=out,
                                     stderr=err, start_new_session=True)
            cleanup_verified = None
            try:
                status = child.wait(timeout=watchdog)
                watchdog_fired = False
            except subprocess.TimeoutExpired:
                cleanup_verified = stop_session(child.pid)
                status = child.wait(timeout=5)
                watchdog_fired = True
        record.update(status="finished", returncode=status,
                      wall_seconds=time.monotonic() - started,
                      outer_watchdog_fired=watchdog_fired,
                      cleanup_verified=cleanup_verified,
                      session_id=child.pid, load_after=os.getloadavg())
        timing = Path(str(prefix) + ".time")
        time_text = timing.read_text() if timing.is_file() else ""
        for field, pattern in [
            ("user_seconds", r"User time \(seconds\): ([0-9.]+)"),
            ("system_seconds", r"System time \(seconds\): ([0-9.]+)"),
            ("max_rss_kib", r"Maximum resident set size \(kbytes\): ([0-9]+)"),
        ]:
            match = re.search(pattern, time_text)
            record[field] = float(match.group(1)) if match else None
        save(Path(str(prefix) + ".json"), record)
        print(json.dumps(record, sort_keys=True), flush=True)
        if watchdog_fired:
            raise WatchdogExpired(label + ": outer watchdog fired; investigation aborted, cleanup verified=" + str(cleanup_verified))
        return record

    def stdout(self, row):
        return (self.output / (row["label"] + ".stdout")).read_bytes()

    def stderr(self, row):
        return (self.output / (row["label"] + ".stderr")).read_bytes()

def has_sanitizer_error(stderr):
    text = stderr.decode("utf-8", errors="replace")
    lines = [line for line in text.splitlines()
             if "WARNING: AddressSanitizer failed to allocate" not in line]
    return any(re.search(r"[0-9]: runtime error|Sanitizer", line) for line in lines)

def self_test():
    with tempfile.TemporaryDirectory(prefix="lustr-diag-selftest-") as tmp:
        rec = Recorder(Path(tmp) / "records")
        ok = rec.run("ok", [sys.executable, "-c", "print('expected')"], tmp, 10)
        bad = rec.run("bad", [sys.executable, "-c", "raise SystemExit(7)"], tmp, 10)
        slow = rec.run("slow", [sys.executable, "-c", "import time; time.sleep(10)"], tmp, 0.1)
        assert ok["returncode"] == 0 and rec.stdout(ok) == b"expected\n"
        assert bad["returncode"] == 7
        assert slow["returncode"] == 124 and not slow["outer_watchdog_fired"]
        assert has_sanitizer_error(b"file.c:1:2: runtime error: overflow")
        assert has_sanitizer_error(b"ERROR: AddressSanitizer: bad access")
        assert not has_sanitizer_error(b"WARNING: AddressSanitizer failed to allocate")
        pidfile = Path(tmp) / "nested.pid"
        code = "import os,time; open(%r,'w').write(str(os.getpid())); time.sleep(30)" % str(pidfile)
        try:
            rec.run("watchdog-nested", ["timeout", "30", sys.executable, "-c", code],
                    tmp, watchdog=0.5)
        except WatchdogExpired:
            row = json.loads((rec.output / "watchdog-nested.json").read_text())
            assert row["outer_watchdog_fired"] and row["cleanup_verified"]
            assert pidfile.is_file(), "nested child did not start"
            assert not live_session_pids(row["session_id"])
        else:
            raise AssertionError("outer watchdog did not abort")
    print("diagnostic self-tests passed", flush=True)

def main(output):
    repo = Path(__file__).resolve().parents[1]
    native = repo / "compiler/native"
    fixture = native / "tests/pass/unicode_props_hash.rs"
    lust = native / "target/release/lust"
    rec = Recorder(output.resolve())
    out = rec.output
    work = out / "reproduced-build"
    work.mkdir()
    base_env = {"LUST_CACHE_DIR": str(work / "cache"),
                "LUST_DAEMON_SOCKET": str(work / "no-daemon.sock")}
    rec.env.update(base_env)
    versions = {}
    for name, command in [
        ("commit", ["git", "rev-parse", "HEAD"]),
        ("upstream-tree", ["git", "rev-parse", "HEAD:compiler/native"]),
        ("rustc", ["rustc", "-vV"]), ("cargo", ["cargo", "-V"]),
        ("gcc", ["gcc", "-v"]), ("timeout", ["timeout", "--version"]),
        ("kernel", ["uname", "-a"]), ("cpu", ["lscpu"]), ("nproc", ["nproc"]),
        ("memory", ["free", "-b"]),
    ]:
        row = rec.run("metadata-" + name, command, repo, 30)
        versions[name] = rec.stdout(row).decode("utf-8", errors="replace").strip()
        if row["returncode"] != 0:
            raise RuntimeError("metadata command failed: " + name)
    if versions["upstream-tree"] != PINNED_TREE:
        raise RuntimeError("protected upstream tree differs")
    nproc = int(versions["nproc"])
    if not 1 <= nproc <= 16:
        raise RuntimeError("unexpected CPU count; refuse uncontrolled parallelism")
    limits = {}
    for filename in ["/sys/fs/cgroup/cpu.max", "/sys/fs/cgroup/memory.max",
                     "/sys/fs/cgroup/cpuset.cpus.effective", "/proc/meminfo"]:
        path = Path(filename)
        if path.is_file():
            limits[filename] = path.read_text()
    selected = {}
    for pattern in ["unicode_props_hash", "unicode_"]:
        selected[pattern] = sorted(str(p.relative_to(native))
                                  for folder in ["pass", "fail", "reject"]
                                  for p in (native / "tests" / folder).glob("*.rs")
                                  if pattern in p.stem)
    if selected["unicode_props_hash"] != ["tests/pass/unicode_props_hash.rs"]:
        raise RuntimeError("focused filter is no longer unique")
    save(out / "metadata.json", {
        "versions": versions, "cpu_affinity": sorted(os.sched_getaffinity(0)),
        "limits": limits, "selected_cases": selected,
        "fixture_sha256": digest(fixture), "lust_sha256": digest(lust),
        "effective_environment": {key: rec.env.get(key) for key in
                                  CLEAR + ["RUN_TIMEOUT", "CASE_TIMEOUT", "SAN_CC_LIMIT",
                                           "DIFFTEST_RUSTC_CACHE", "LUST_NO_DAEMON"]},
        "scope": "Linux-only diagnostic; not full CI or a qualification of full Rust",
    })
    shutil.copy2(fixture, out / "fixture.rs")
    authoritative = []
    authoritative.append(rec.run(
        "upstream-isolated-10s", ["bash", "scripts/difftest.sh", "unicode_props_hash"],
        native, env={"SANITIZE": "1", "DIFFTEST_JOBS": "1"}, watchdog=900))
    authoritative.append(rec.run(
        "upstream-unicode-pool-10s", ["bash", "scripts/difftest.sh", "unicode_"],
        native, env={"SANITIZE": "1"}, watchdog=900))
    save(out / "upstream-results.json", authoritative)
    for row in authoritative:
        print("UPSTREAM OUTPUT " + row["label"] + "\n"
              + rec.stdout(row).decode("utf-8", errors="replace")
              + rec.stderr(row).decode("utf-8", errors="replace"), flush=True)

    # An independent reproduction of sanitize_case, not artifacts from the runs above.
    # The protected upstream script deletes its own temporary output.
    def required(label, args, limit=None):
        row = rec.run(label, args, work, limit)
        if row["returncode"] != 0:
            raise RuntimeError(label + " failed; see retained command and stderr")
        return row

    required("oracle-compile", ["rustc", "--edition", "2024", "-A", "warnings",
                               "-o", "rustc.bin", fixture], 30)
    oracle = rec.run("oracle-run-10s", ["./rustc.bin"], work, 10)
    def lust_compile(label, args):
        row = rec.run(label, args, work, 30)
        if row["returncode"] in (124, 137):
            row = rec.run(label + "-isolated-compile-retry", args, work, 180)
        if row["returncode"] != 0:
            raise RuntimeError(label + " failed; no runtime conclusion")
    name = "unicode_props_hash"
    lust_compile("lust-compile", [lust, "build", fixture, "-o", name + ".lust"])
    plain = rec.run("lust-run-10s", ["./" + name + ".lust"], work, 10)
    oracle_ok = (oracle["returncode"] == 0 and plain["returncode"] == 0
                 and rec.stdout(oracle) == rec.stdout(plain)
                 and rec.stderr(oracle) == rec.stderr(plain) == b"")
    save(out / "oracle-comparison.json", {"equal_successful_results": oracle_ok,
                                        "oracle": oracle, "plain": plain})
    if not oracle_ok:
        raise RuntimeError("plain execution does not match oracle; no sanitizer timing conclusion")
    lust_compile("emit-c", [lust, "build", fixture, "-o", name + ".tmp",
                             "--emit-c", name])
    units = sorted(work.glob(name + ".*.c"))
    if not units:
        raise RuntimeError("no emitted C units")
    objects = []
    builds = []
    for index, unit in enumerate(units):
        unit_hash = digest(unit)
        obj = work / (unit_hash[:40] + ".o")
        row = rec.run("gcc-o1-unit-" + str(index),
                      ["gcc"] + FLAGS + ["-c", "-o", obj, unit], work, 20)
        fallback = row["returncode"] == 124
        if fallback:
            # Upstream O0 fallback has no per-unit timeout. The enclosing diagnostic
            # watchdog is only an incomplete-investigation bound, never a pass.
            row = rec.run("gcc-o0-unit-" + str(index),
                          ["gcc"] + FLAGS + ["-O0", "-c", "-o", obj, unit], work)
        builds.append({"source": unit.name, "sha256": unit_hash,
                       "o0_fallback": fallback, "result": row})
        save(out / "compilation.json", builds)
        if row["returncode"] != 0:
            raise RuntimeError("generated C compilation failed")
        objects.append(obj)
    binary = work / (name + ".san")
    required("gcc-link", ["gcc"] + FLAGS + ["-o", binary] + objects + ["-lm"])
    save(out / "binary.json", {"sha256": digest(binary), "units": builds,
                              "flags": FLAGS, "nproc": nproc})
    san_env = {"ASAN_OPTIONS": "detect_leaks=1:allocator_may_return_null=1"}
    expected = rec.stdout(oracle)
    def execute(label, limit=10, barrier=None):
        if barrier:
            barrier.wait(timeout=30)
        row = rec.run(label, [binary], work, limit, env=san_env, watchdog=limit + 60)
        row["matches_oracle"] = row["returncode"] == 0 and rec.stdout(row) == expected
        row["sanitizer_error"] = has_sanitizer_error(rec.stderr(row))
        row["checks_pass"] = row["matches_oracle"] and not row["sanitizer_error"]
        save(out / (label + ".comparison.json"), row)
        return row
    repeated = []
    # Alternate the order. Parallel copies test execution contention, not the
    # full suite's mixture of compilation, object-cache locking and execution.
    for pair in range(3):
        order = ["isolated", "concurrent"] if pair % 2 == 0 else ["concurrent", "isolated"]
        for mode in order:
            if mode == "isolated":
                repeated.append(execute("isolated-%d-10s" % pair))
            else:
                barrier = threading.Barrier(nproc)
                with concurrent.futures.ThreadPoolExecutor(max_workers=nproc) as pool:
                    futures = [pool.submit(execute, "concurrent-%d-%d-10s" % (pair, i),
                                           10, barrier) for i in range(nproc)]
                    repeated.extend(f.result() for f in futures)
            save(out / "runtime-10s-results.json", repeated)
    # This file is never overwritten. A longer run cannot erase a 10s result.
    original = {"upstream_failed": any(r["returncode"] != 0 for r in authoritative),
                "reproduced_10s_failed": any(not r["checks_pass"] for r in repeated),
                "upstream": authoritative, "runtime": repeated}
    save(out / "qualification-10s-before-exploration.json", original)
    extended = None
    if any(r["returncode"] == 124 for r in repeated) or original["upstream_failed"]:
        extended = execute("exploratory-isolated-60s", 60)
    summary = {
        "original_10s_upstream_failed": original["upstream_failed"],
        "reproduced_10s_failed": original["reproduced_10s_failed"],
        "exploratory_60s": extended,
        "nproc": nproc,
        "o0_fallback_units": [r["source"] for r in builds if r["o0_fallback"]],
        "binary_sha256": digest(binary),
        "fixture_sha256": digest(fixture),
        "runtime_10s": [{key: row[key] for key in [
            "label", "returncode", "wall_seconds", "user_seconds", "system_seconds",
            "max_rss_kib", "matches_oracle", "sanitizer_error", "checks_pass"]}
                       for row in repeated],
        "limitations": [
            "No full-suite scheduling reproduction and no full CI pass claim.",
            "unicode_ subset success cannot rule out full-suite contention.",
            "Retained C and binary are an independent build, not the upstream run's deleted files.",
            "Concurrent copies test runtime contention, not mixed compilation and cache locking.",
            "60s success is diagnostic only and does not change any 10s failure.",
        ],
    }
    save(out / "summary.json", summary)
    print("DIAGNOSTIC SUMMARY " + json.dumps(summary, sort_keys=True), flush=True)
    # Preserve both the authoritative failures and failures of the reproduction.
    return int(original["upstream_failed"] or original["reproduced_10s_failed"])

if __name__ == "__main__":
    if sys.argv[1:] == ["--self-test"]:
        self_test()
    elif len(sys.argv) == 2:
        raise SystemExit(main(Path(sys.argv[1])))
    else:
        raise SystemExit("usage: python3 tools/diagnose-unicode-sanitizer.py OUTPUT | --self-test")
