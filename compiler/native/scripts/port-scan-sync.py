#!/usr/bin/env python3
"""Refresh tools/lust-port/features.json by testing lust against rustc.

Each probe in tools/lust-port/tests/features/ is one small program for
one feature id. This script builds it with rustc (edition 2024) and
with `lust check` plus `lust run`, then derives one status per
feature: supported (every probe matches rustc), partial (some probes
match), or unsupported (no probe matches). A lust build that runs with
wrong output is a false accept and fails the run.

Modes:
  (default)       run the probes and print the status table
  --write         also write status and milestone into features.json
                  (supported sets milestone 0; notes stay hand-written)
  --check         write nothing; exit 1 and list every feature whose
                  committed status (or supported milestone) differs
                  from the probe result on this tree. Statuses come
                  only from probes, so --check needs no ts-rust
                  checkout. scripts/port-sync-check.sh is the gate
                  form of this mode.
  --coverage      run lust-port on ts-rust, refresh docs/ts-rust-gaps.json,
                  and write docs/ts-rust-coverage.md for the core CLI scope

Probes are independent: --jobs N runs N at a time (default: CPU
count, max 8; 1 is serial). Each probe gets its own work dir (rustc
binary, lust cache, Cargo target, run working directory), so
parallel probes cannot see each other's files.

A timeout is never a status. A compile step that passes its time
limit (exit 124 from run()), is killed by a signal, or (for a lust
compile) fails with no L#### diagnostic gives verdict "timeout",
not a reject: under gate load a healthy lust compile can pass the
limit or die from the load, and reading that as a reject flips a
supported feature to unsupported (the ext.rustix gate failures; a
real lust reject always carries an L#### code). Such a probe runs
once more, serially after the pool, with the long RETRY_* limits.
If it still is not judged, --check fails that feature as a timeout
(not a status mismatch) and --write keeps its committed status and
warns. Each probe result records its wall time in "secs" (first
try plus any retry), and the run prints the slowest probes.

Probes with `// oracle: lust-only` have no single-file rustc oracle
(a proc macro). They never set supported by themselves: a lust reject
records unsupported, a lust accept is reported for a human call.

Probes with `// cargo-dep: <dependency line>` headers (one per line,
as written under [dependencies]) use a registry crate: the probe runs
as the bin of a temporary Cargo package with those dependencies. Cargo
builds and runs it against the real crate (offline, from the local
registry cache) as the oracle, and lust builds it from the same
manifest, which swaps in the lust adapter (docs/ARCH.md 3.6).

Env: LUST (lust binary), LUST_PORT (scanner binary), TS_RUST (ts-rust
checkout), LUST_TCC_HOME (passed through to lust). For --coverage,
LUST_DEV_FEATURES (exact comma list) or LUST_DEV_EXCLUDE_FEATURES
(comma list removed from each package default) override the project
lust dev config in the scanned tree (docs/dev-features.md); the
scanner otherwise reads that config itself. Per-step time
limits in seconds (floats allowed) can be set for tests:
PORT_SCAN_COMPILE_TIMEOUT, PORT_SCAN_RUN_TIMEOUT,
PORT_SCAN_CARGO_TIMEOUT, and the retry forms
PORT_SCAN_RETRY_COMPILE_TIMEOUT, PORT_SCAN_RETRY_RUN_TIMEOUT and
PORT_SCAN_RETRY_CARGO_TIMEOUT.
"""
import argparse
import atexit
import concurrent.futures
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBES = ROOT / "tools/lust-port/tests/features"
FEATURES_JSON = ROOT / "tools/lust-port/features.json"


def _timeout_env(name, default):
    """One per-step time limit in seconds, from $PORT_SCAN_<name> when set.

    A float such as 0.05 is allowed so a test can plant a tiny limit
    that even a fast probe step passes.
    """
    try:
        return max(0.01, float(os.environ.get("PORT_SCAN_" + name, default)))
    except ValueError:
        return float(default)


COMPILE_TIMEOUT = _timeout_env("COMPILE_TIMEOUT", 30)
RUN_TIMEOUT = _timeout_env("RUN_TIMEOUT", 10)
CARGO_TIMEOUT = _timeout_env("CARGO_TIMEOUT", 300)
# A timed-out probe runs once more, serially, with these long limits.
RETRY_COMPILE_TIMEOUT = _timeout_env("RETRY_COMPILE_TIMEOUT", 180)
RETRY_RUN_TIMEOUT = _timeout_env("RETRY_RUN_TIMEOUT", 60)
RETRY_CARGO_TIMEOUT = _timeout_env("RETRY_CARGO_TIMEOUT", 600)
NOT_CORE = {"ls", "lsp", "project", "api", "effect", "format", "fswatch", "ipc", "contentmapper"}
# Owner per feature for the coverage report. Unlisted features fall back
# to a root-cause owner in OWNER_BY_CAUSE.
OWNER = {
    "threads.thread_local": "W-TLS",
    "lifetimes.static_anon": "W-BORROW",
    "lifetimes.named": "W-BORROW",
    "lifetimes.hrtb": "W-BORROW",
    "closures.fn_ptr_lifetime": "W-BORROW",
    "closures.fn_ptr_lifetime_field": "W-BORROW",
    "lifetimes.hrtb_general": "W-BORROW",
    "threads.sync": "wave C (threads)",
    "threads.send_sync_bounds": "wave C (threads)",
    "panic.recovery": "wave C (panics)",
    "cinterop.system": "wave C (shims)",
    "cinterop.raw_ptr": "W-CINTEROP",
    "cinterop.extern": "W-CINTEROP",
    "cinterop.global_allocator": "W-CINTEROP",
    "ext.unknown_crate": "wave D",
    "ext.serde": "wave D",
    "ext.serde_json": "wave D",
    "std.net": "W-SHIM-P2",
    "std.net_addr": "W-STD-C",
    "std.error": "W-STD-C",
    "std.float_consts": "W-STD-C",
    "std.vec_into_iter": "W-STD-C",
    "std.hashmap": "W-STD-B",
    "std.hashmap_hasher": "W-STD-B",
    "std.cow": "W-STD-B",
    "core.std_basics": "W-SHIM-P2",
}
OWNER_BY_CAUSE = {
    "std-runtime": "W-STD-B",
    "external-crates": "W-DEPS",
    "traits-generics": "W-CHECK-G",
    "closures": "W-CHECK-G",
    "dyn-trait": "W-CHECK-G",
    "iterators": "W-CHECK-G",
    "attrs-cfg": "W-CHECK-G",
    "macros": "W-CHECK-G",
    "lifetimes": "W-BORROW",
    "unsafe": "wave C (shims)",
}


def run(cmd, timeout, env=None, cwd=None):
    """Run one command; return (exit_code, stdout, stderr). Exit 124 on timeout."""
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout, env=env, cwd=cwd)
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired:
        return 124, "", "timeout"


def parse_header(path):
    head = {}
    for line in path.read_text().splitlines():
        m = re.match(r"//\s*([a-z-]+):\s*(.+)", line)
        if m:
            head[m.group(1)] = m.group(2).strip()
        elif line.strip() and not line.startswith("//"):
            break
    return head


def first_code(stderr):
    m = re.search(r"L\d{4}", stderr)
    return m.group(0) if m else ""


def cargo_package(path, deps, tmp):
    """A temporary Cargo package whose bin `probe` is the probe file.
    Returns (manifest path, bin source path)."""
    pkg = tmp / (path.stem + "-pkg")
    (pkg / "src").mkdir(parents=True, exist_ok=True)
    (pkg / "src/main.rs").write_text(path.read_text())
    lines = ["[package]", 'name = "probe"', 'version = "0.0.0"', 'edition = "2024"', "", "[dependencies]", *deps, "", "[workspace]", ""]
    (pkg / "Cargo.toml").write_text("\n".join(lines))
    return pkg / "Cargo.toml", pkg / "src/main.rs"


def timed_out(res, step, limit):
    """Mark a probe result as timed out at one step. A timeout is not a verdict on lust."""
    res.update(verdict="timeout", detail=f"{step} timed out after {limit:g}s")
    return res


def unjudged(res, step, code, err):
    """Mark a compile step that died without judging the probe.

    Killed by a signal (code < 0, for example the OOM killer under
    load), or a lust compile that failed with no L#### diagnostic:
    a real lust reject always carries a code, so a failure without
    one is a load or tool failure, never a reject. The verdict is
    "timeout", so the probe gets the serial retry and, if it still
    is not judged, the timeout failure path.
    """
    if code < 0:
        detail = f"{step} killed by signal {-code}"
    else:
        first = err.strip().splitlines()[0][:120] if err.strip() else "no stderr"
        detail = f"{step} failed with no diagnostic (exit {code}): {first}"
    res.update(verdict="timeout", detail=detail)
    return res


def probe_one(path, lust, work, env, compile_timeout=COMPILE_TIMEOUT, run_timeout=RUN_TIMEOUT, cargo_timeout=CARGO_TIMEOUT):
    """Test one probe file in its own work dir. Returns a result dict.

    A compile step that exits 124 (run() hit its limit), is killed,
    or fails with no L#### diagnostic gives verdict "timeout",
    never a reject: the probe was not judged. Program runs keep
    their exit comparison, so a panicking probe still counts.
    """
    head = parse_header(path)
    res = {"file": path.name, "feature": head.get("feature", ""), "form": head.get("form", "")}
    if head.get("helper") == "true":
        res["verdict"] = "helper"
        return res
    deps = re.findall(r"^//\s*cargo-dep:\s*(.+?)\s*$", path.read_text(), re.M)
    if deps:
        return probe_package(path, deps, lust, work, env, res, compile_timeout, run_timeout, cargo_timeout)
    lust_only = head.get("oracle") == "lust-only"
    panic = head.get("rustc-panic", "abort")
    # A probe that needs unwinding builds lust with the same strategy.
    mode = ["-C", "panic=unwind"] if panic == "unwind" else []
    if not lust_only:
        rbin = work / (path.stem + ".rustc")
        code, _, err = run(
            ["rustc", "--edition", "2024", "-C", f"panic={panic}", "-o", str(rbin), str(path)],
            compile_timeout,
        )
        if code == 124:
            return timed_out(res, "rustc compile", compile_timeout)
        if code < 0:
            return unjudged(res, "rustc compile", code, err)
        if code != 0:
            res.update(verdict="rustc-reject", detail=err.strip().splitlines()[-1][:160] if err.strip() else "")
            return res
        # Same working directory as the lust run below: probes that touch relative paths
        # (core_std_basics_b reads Cargo.toml) must see the same files.
        rcode, rout, _ = run([str(rbin)], run_timeout, cwd=str(work))
        if rcode == 124:
            return timed_out(res, "rustc run", run_timeout)
        res["rustc_stdout"] = rout
        res["rustc_exit"] = rcode
    ccode, _, cerr = run([str(lust), "check", *mode, str(path)], compile_timeout, env)
    res["lust_code"] = first_code(cerr)
    if ccode == 124:
        return timed_out(res, "lust check", compile_timeout)
    if ccode != 0 and (ccode < 0 or not res["lust_code"]):
        return unjudged(res, "lust check", ccode, cerr)
    if ccode != 0:
        res["verdict"] = "reject"
        return res
    # `lust run` drops its binary in the working directory; keep it in the work dir.
    lcode, lout, lerr = run([str(lust), "run", *mode, str(path)], compile_timeout, env, cwd=str(work))
    res["lust_stdout"] = lout
    res["lust_exit"] = lcode
    if lcode == 124:
        return timed_out(res, "lust run", compile_timeout)
    if lust_only:
        res["verdict"] = "lust-only-accept" if lcode == 0 else "lust-only-runfail"
        return res
    same_exit = lcode == res["rustc_exit"] or (lcode != 0 and res["rustc_exit"] != 0)
    if lout == res["rustc_stdout"] and same_exit:
        res["verdict"] = "pass"
    elif lcode != 0 and not lout:
        res.update(verdict="runfail", detail=first_code(lerr) or "lust run failed after check passed")
    else:
        res.update(verdict="wrong", detail=f"lust stdout {lout!r} != rustc {res['rustc_stdout']!r}")
    return res


def probe_package(path, deps, lust, tmp, env, res, compile_timeout=COMPILE_TIMEOUT, run_timeout=RUN_TIMEOUT, cargo_timeout=CARGO_TIMEOUT):
    """A `cargo-dep` probe: Cargo with the real crate is the oracle.

    Unjudged compile steps are verdict "timeout", as in probe_one.
    The lust build of an adapter shim is the step that failed for
    ext.rustix under gate load.
    """
    manifest, src = cargo_package(path, deps, tmp)
    cenv = dict(env, CARGO_NET_OFFLINE="true")
    target = manifest.parent / "target"
    code, _, err = run(["cargo", "build", "-q", "--manifest-path", str(manifest), "--target-dir", str(target)], cargo_timeout, cenv)
    if code == 124:
        return timed_out(res, "cargo build", cargo_timeout)
    if code < 0:
        return unjudged(res, "cargo build", code, err)
    if code != 0:
        res.update(verdict="rustc-reject", detail=err.strip().splitlines()[-1][:160] if err.strip() else "")
        return res
    rcode, rout, _ = run([str(target / "debug/probe")], run_timeout)
    if rcode == 124:
        return timed_out(res, "cargo run", run_timeout)
    res["rustc_stdout"] = rout
    res["rustc_exit"] = rcode
    lbin = manifest.parent / "probe.lust"
    ccode, _, cerr = run([str(lust), "build", str(src), "-o", str(lbin), "--manifest-path", str(manifest), "--bin", "probe"], compile_timeout, cenv)
    res["lust_code"] = first_code(cerr)
    if ccode == 124:
        return timed_out(res, "lust build", compile_timeout)
    if ccode != 0 and (ccode < 0 or not res["lust_code"]):
        return unjudged(res, "lust build", ccode, cerr)
    if ccode != 0:
        res["verdict"] = "reject"
        return res
    lcode, lout, _ = run([str(lbin)], run_timeout)
    res["lust_stdout"] = lout
    res["lust_exit"] = lcode
    if lcode == 124:
        return timed_out(res, "lust run", run_timeout)
    same_exit = lcode == rcode or (lcode != 0 and rcode != 0)
    if lout == rout and same_exit:
        res["verdict"] = "pass"
    elif lcode != 0 and not lout:
        res.update(verdict="runfail", detail="lust build ran and failed")
    else:
        res.update(verdict="wrong", detail=f"lust stdout {lout!r} != cargo {rout!r}")
    return res


def feature_status(results):
    """Derive one status from the probe verdicts of one feature.

    A timed-out probe was not judged, so it blocks every status: the
    feature derives "timeout", which --write never writes and
    --check reports as a timeout, not as a status mismatch.
    """
    vs = [r["verdict"] for r in results]
    if any(v == "timeout" for v in vs):
        return "timeout"
    if all(v == "pass" for v in vs):
        return "supported"
    if any(v == "pass" for v in vs):
        return "partial"
    if all(v in ("reject", "runfail") for v in vs):
        return "unsupported"
    return "manual"  # lust-only accepts, or a wrong-output probe


# Verdicts that fail the whole run in every mode: lust ran with wrong
# output (a false accept), the rustc oracle itself rejected the probe,
# or the probe runner crashed on the probe.
BAD_VERDICTS = ("wrong", "rustc-reject", "error")


def run_probes(paths, lust, tmp, env, jobs):
    """Run every probe and return the results sorted by file name.

    Each probe runs in tmp/<stem> with its own lust cache, so probes
    can run in parallel (jobs > 1) without seeing each other's files,
    binaries, or Cargo target dirs. A probe that crashes the runner
    is a failed probe (verdict "error"), not a crashed run.

    A probe that times out runs once more after the pool, serially,
    in the same work dir (its Cargo target and lust cache carry
    over) with the long RETRY_* limits, and is marked "retried".
    Every result carries "secs": wall time of the first try plus
    any retry.
    """

    def one(p, compile_timeout=COMPILE_TIMEOUT, run_timeout=RUN_TIMEOUT, cargo_timeout=CARGO_TIMEOUT):
        work = tmp / p.stem
        work.mkdir(parents=True, exist_ok=True)
        penv = dict(env, LUST_CACHE_DIR=str(work / "cache"), LUST_DAEMON_SOCKET=str(work / "none.sock"))
        t0 = time.monotonic()
        try:
            res = probe_one(p, lust, work, penv, compile_timeout, run_timeout, cargo_timeout)
        except Exception as e:
            try:
                head = parse_header(p)
            except Exception:
                head = {}
            res = {
                "file": p.name,
                "feature": head.get("feature", ""),
                "form": head.get("form", ""),
                "verdict": "error",
                "detail": f"{type(e).__name__}: {e}"[:160],
            }
        res["secs"] = round(time.monotonic() - t0, 2)
        return res

    if jobs <= 1:
        results = [one(p) for p in paths]
    else:
        with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as ex:
            results = list(ex.map(one, paths))
    by_file = {p.name: p for p in paths}
    for i, r in enumerate(results):
        if r["verdict"] == "timeout":
            retry = one(by_file[r["file"]], RETRY_COMPILE_TIMEOUT, RETRY_RUN_TIMEOUT, RETRY_CARGO_TIMEOUT)
            retry["secs"] = round(r.get("secs", 0) + retry["secs"], 2)
            retry["retried"] = True
            results[i] = retry
            if retry["verdict"] != "timeout":
                print(f"slow: {retry['file']} timed out under load, judged alone: {retry['verdict']}")
    return sorted(results, key=lambda r: r["file"])


def check_features(features, derived, by_feature):
    """Compare the committed feature table with the probe results.

    Returns {"passed": [id], "failed": [(id, message)], "timeouts":
    [(id, message)], "unchecked": [(id, why)]}. A feature passes only
    when --write would leave its row alone: same status, and
    milestone 0 when supported. A feature with no probe, or a manual
    one (a lust-only accept needs a human call), cannot be derived
    from probes, so it is unchecked rather than silently passed. A
    probe whose feature id is not in the table fails: its status is
    not in features.json at all. A feature with a timed-out probe
    (after the serial retry) is a timeout failure, kept apart from
    "failed": its committed status was not judged, so there is no
    status mismatch to fix with --write.
    """
    passed, failed, timeouts, unchecked = [], [], [], []
    for f in features:
        fid = f["id"]
        rs = by_feature.get(fid)
        if rs is None:
            unchecked.append((fid, "no probe"))
            continue
        bad = [r for r in rs if r["verdict"] in BAD_VERDICTS]
        if bad:
            failed.append((fid, "probe " + ", ".join(f"{r['file']}:{r['verdict']}" for r in bad) + "; fix the probe or lust"))
            continue
        slow = [r for r in rs if r["verdict"] == "timeout"]
        if slow:
            timeouts.append(
                (
                    fid,
                    "probe "
                    + ", ".join(f"{r['file']}: {r.get('detail', 'timed out')}" for r in slow)
                    + f" (serial retry limit {RETRY_COMPILE_TIMEOUT:g}s); the committed status {f['status']!r} was not judged",
                )
            )
            continue
        d = derived.get(fid)
        if d == "manual":
            unchecked.append((fid, "manual"))
        elif f["status"] != d:
            failed.append((fid, f"committed status {f['status']!r} != probe status {d!r}"))
        elif d == "supported" and f.get("milestone") != 0:
            failed.append((fid, f"supported but committed milestone is {f.get('milestone')!r}; --write sets milestone 0"))
        else:
            passed.append(fid)
    known = {f["id"] for f in features}
    for fid in sorted(set(by_feature) - known):
        failed.append((fid, "probe feature id is not in features.json; add the feature row or fix the probe header"))
    failed.sort(key=lambda x: x[0])
    timeouts.sort(key=lambda x: x[0])
    return {"passed": passed, "failed": failed, "timeouts": timeouts, "unchecked": unchecked}


def print_check_fix(has_bad, has_unknown):
    """Tell the failing package exactly how to make the gate green."""
    print("port-sync check failed: tools/lust-port/features.json does not match the probe results on this tree.")
    print("A stale status from an old base can hide real coverage. Do not edit a status by hand.")
    print("Fix it in your worktree:")
    print("  1. Rebase on the current next:  git rebase next")
    print("  2. Rebuild lust:                export LUST_TCC_HOME=/home/theo/.t3/projects/lust-lang/third_party/tcc")
    print("                                  cargo build --release")
    print("  3. Rewrite the statuses:        python3 scripts/port-scan-sync.py --write")
    print("  4. Commit the result:           git add tools/lust-port/features.json")
    print("                                  git commit (a new commit, or amend yours)")
    print("  5. Run this check again:        scripts/port-sync-check.sh")
    if has_bad:
        print("A probe above also reports wrong output, a rustc reject, or a runner error.")
        print("--write cannot fix that. Fix the probe or lust first, then do steps 3 to 5.")
    if has_unknown:
        print("A probe above names a feature id that is not in features.json.")
        print("Add the feature row or fix the probe header first, then do the steps above.")


def print_timeout_fix():
    """An unjudged probe is not a stale status. Say what to do instead of --write."""
    print("port-sync check timed out: a probe above was not judged (it timed out, was killed, or")
    print(f"failed with no diagnostic) in the parallel pool and again in the serial retry")
    print(f"(compile limit {RETRY_COMPILE_TIMEOUT:g}s). Its feature status was not judged.")
    print("Do not edit features.json and do not run --write to fix this. Run the check again when")
    print("the box is less loaded. If the same probe is not judged on an idle box, it is a slow")
    print("probe or a slow lust compile: keep the 'slowest probes' line from this log and report it.")


def do_probes(args):
    lust = Path(os.environ.get("LUST", ROOT / "target/release/lust"))
    # Probes run lust with cwd in their work dir, so a relative path
    # (the gate uses LUST=target/release/lust) must become absolute
    # now, while cwd is still the repo root. A bare name stays as is
    # and resolves through PATH.
    if len(lust.parts) > 1 and not lust.is_absolute():
        lust = lust.resolve()
    if not lust.exists():
        sys.exit(f"lust binary not found: {lust} (build with cargo build --release)")
    table = json.loads(FEATURES_JSON.read_text())
    known = {f["id"] for f in table["features"]}
    env = dict(os.environ)
    env["LUST_NO_DAEMON"] = "1"
    tmp = Path(tempfile.mkdtemp(prefix="port-scan-sync-"))
    # Probe work dirs hold Cargo target dirs; do not leak them.
    atexit.register(shutil.rmtree, tmp, ignore_errors=True)
    jobs = getattr(args, "jobs", 0) or min(8, os.cpu_count() or 4)
    t0 = time.monotonic()
    results = run_probes(sorted(PROBES.glob("*.rs")), lust, tmp, env, jobs)
    print(f"probes: {len(results)} files in {time.monotonic() - t0:.1f}s (jobs={jobs})")
    slowest = sorted(results, key=lambda r: r.get("secs", 0), reverse=True)[:5]
    if slowest:
        print(
            "slowest probes: "
            + ", ".join(f"{r['file']}:{r['verdict']} {r.get('secs', 0):.1f}s" + (" (retried)" if r.get("retried") else "") for r in slowest)
        )
    by_feature = {}
    for r in results:
        if r["verdict"] != "helper":
            by_feature.setdefault(r["feature"], []).append(r)
    derived = {fid: feature_status(rs) for fid, rs in by_feature.items()}
    old = {f["id"]: f["status"] for f in table["features"]}

    print(f"{'feature':32} {'old':12} {'probe says':12} probes")
    for fid in sorted(by_feature):
        marks = ", ".join(f"{r['file']}:{r['verdict']}" + (f"({r['lust_code']})" if r.get("lust_code") else "") for r in by_feature[fid])
        print(f"{fid:32} {old.get(fid, 'NOT IN TABLE'):12} {derived[fid]:12} {marks}")
    unprobed = sorted(known - set(by_feature))
    unknown = sorted(set(by_feature) - known)
    if unprobed:
        print("no probe (status not checked):", ", ".join(unprobed))
    if unknown:
        print("probe id not in features.json:", ", ".join(unknown))

    bad = [r for r in results if r["verdict"] in BAD_VERDICTS]
    for r in bad:
        print(f"FAIL {r['file']}: {r['verdict']} {r.get('detail', '')}")
    timed = [r for r in results if r["verdict"] == "timeout"]
    for r in timed:
        print(f"TIMEOUT {r['file']}: {r.get('detail', '')} (after the serial retry; probe not judged)")
    if args.json:
        Path(args.json).write_text(json.dumps({"probes": results, "derived": derived}, indent=1) + "\n")
    if args.write:
        for f in table["features"]:
            d = derived.get(f["id"])
            if d == "timeout":
                # A timeout is not a status. Keep the committed row as it is.
                print(f"WARN {f['id']}: probe timed out; committed status {f['status']!r} left unchanged")
            elif d in ("supported", "partial", "unsupported"):
                f["status"] = d
                if d == "supported":
                    f["milestone"] = 0
        FEATURES_JSON.write_text(json.dumps(table, indent=1) + "\n")
        print(f"wrote {FEATURES_JSON}")
    if getattr(args, "check", False):
        chk = check_features(table["features"], derived, by_feature)
        for fid, msg in chk["failed"]:
            print(f"MISMATCH {fid}: {msg}")
        for fid, msg in chk["timeouts"]:
            print(f"TIMEOUT {fid}: {msg}")
        manual = [fid for fid, why in chk["unchecked"] if why == "manual"]
        if manual:
            print("manual (status not checked, needs a human call):", ", ".join(manual))
        # One machine line; scripts/port-sync-check.sh reads the counts from it.
        # A timeout failure counts as failed here, but keeps its own lines above.
        n_failed = len(chk["failed"]) + len(chk["timeouts"])
        print(f"port-scan-sync check: {len(chk['passed'])} passed, {n_failed} failed, {len(chk['unchecked'])} unchecked")
        if chk["timeouts"]:
            print_timeout_fix()
        if chk["failed"]:
            print_check_fix(has_bad=bool(bad), has_unknown=bool(unknown))
        if n_failed:
            sys.exit(2 if (bad or chk["timeouts"]) else 1)
    if bad or timed:
        sys.exit(2)
    return derived


def module_of(path):
    """Top module of a ts-rust file, the inventory coverage.py rule."""
    rest = path.replace("crates/ts_goport/src/", "")
    return rest.split("/")[0].replace(".rs", "")


def curve(files, feats):
    """Files and lines that compile after each milestone, report.rs style."""
    total_files = len(files)
    total_lines = sum(f["lines"] for f in files)
    rows = []
    for n in [0, 4, 5, 6, 7, 8]:
        ok = [
            f
            for f in files
            if not f["parse_error"]
            and all((feats.get(fid) or {}).get("milestone") is not None and feats[fid]["milestone"] <= n for fid in f["missing_features"])
        ]
        lines = sum(f["lines"] for f in ok)
        rows.append((n, len(ok), lines, 100.0 * len(ok) / max(total_files, 1), 100.0 * lines / max(total_lines, 1)))
    return rows


def ensure_scanner():
    """The lust-port scanner binary, never a stale one.

    The scanner embeds features.json and dependencies.json at build
    time (`include_str!` in tools/lust-port/src), so a binary built
    before the last edit of either silently scans with the old
    table. With no LUST_PORT override, rebuild it (cargo is a no-op
    when fresh); with an override, refuse a binary older than any
    input it embeds or compiles.
    """
    port_dir = ROOT / "tools/lust-port"
    build = ["cargo", "build", "--release", "--manifest-path", str(port_dir / "Cargo.toml")]
    custom = os.environ.get("LUST_PORT")
    if custom:
        scanner = Path(custom)
        if not scanner.exists():
            sys.exit(f"scanner not found: {scanner} (build with {' '.join(build)})")
        inputs = [FEATURES_JSON, port_dir / "dependencies.json", port_dir / "Cargo.toml", port_dir / "Cargo.lock"]
        inputs += [p for p in (port_dir / "src").rglob("*") if p.is_file()]
        stale = max(inputs, key=lambda p: p.stat().st_mtime)
        if scanner.stat().st_mtime < stale.stat().st_mtime:
            sys.exit(
                f"scanner is stale: {scanner} is older than {stale}, which it embeds or compiles; "
                f"rebuild with {' '.join(build)} or unset LUST_PORT to rebuild here"
            )
        return scanner
    scanner = port_dir / "target/release/lust-port"
    code, _, err = run(build, 1200)
    if code != 0:
        sys.exit(f"scanner build failed: {err[-500:]}")
    if not scanner.exists():
        sys.exit(f"scanner not found after build: {scanner}")
    return scanner


def scan_feature_args(env=None):
    """Scanner feature override flags from the environment.

    The scanner reads the project lust dev config from the scanned
    tree on its own. These env vars override it for one run, which is
    how a read-only tree (no `.lust.toml` yet) gets the dev selection:
    LUST_DEV_EXCLUDE_FEATURES=jemalloc for ts-rust.
    """
    env = os.environ if env is None else env
    exact = env.get("LUST_DEV_FEATURES", "").strip()
    exclude = env.get("LUST_DEV_EXCLUDE_FEATURES", "").strip()
    if exact and exclude:
        sys.exit("set only one of LUST_DEV_FEATURES and LUST_DEV_EXCLUDE_FEATURES")
    if exact:
        return ["--cargo-features", exact]
    if exclude:
        return ["--cargo-exclude-features", exclude]
    return []


def do_coverage(args):
    scanner = ensure_scanner()
    ts_rust = Path(os.environ.get("TS_RUST", "/home/theo/.t3/projects/ts-rust"))
    manifest = ts_rust / "Cargo.toml"
    code, out, err = run(
        [str(scanner), "scan", "--manifest-path", str(manifest), "--target", "x86_64-unknown-linux-gnu", "--json", "--summary"]
        + scan_feature_args(),
        600,
    )
    if code != 0:
        sys.exit(f"scan failed: {err[-500:]}")
    gaps_path = ROOT / "docs/ts-rust-gaps.json"
    gaps_path.write_text(out if out.endswith("\n") else out + "\n")
    rep = json.loads(out)
    table = json.loads(FEATURES_JSON.read_text())
    feats = {f["id"]: f for f in table["features"]}

    prod = [f for f in rep["files"] if f["cat"] == "prod" and f["scope"] == "active"]
    core = [f for f in prod if module_of(f["file"]) not in NOT_CORE]
    today = [f for f in core if not f["missing_features"] and not f["parse_error"]]
    today_lines = sum(f["lines"] for f in today)
    core_lines = sum(f["lines"] for f in core)

    blockers = []
    for fid, feat in feats.items():
        if feat["status"] == "supported":
            continue
        hit = [f for f in core if fid in f["missing_features"]]
        if not hit:
            continue
        sole = [f for f in hit if len(f["missing_features"]) == 1]
        blockers.append(
            {
                "id": fid,
                "title": feat["title"],
                "status": feat["status"],
                "milestone": feat["milestone"],
                "owner": OWNER.get(fid) or OWNER_BY_CAUSE.get(feat["root_cause"], "wave D"),
                "files": len(hit),
                "lines": sum(f["lines"] for f in hit),
                "sole_files": len(sole),
                "sole_lines": sum(f["lines"] for f in sole),
            }
        )
    blockers.sort(key=lambda b: (-b["lines"], b["id"]))
    top = blockers[:15]

    head = subprocess.run(["git", "-C", str(ts_rust), "rev-parse", "--short", "HEAD"], capture_output=True, text=True)
    ts_head = head.stdout.strip() or "unknown"
    lust_head = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()

    def curve_table(files):
        rows = ["| after milestone | files | files % | lines % |", "|---|---|---|---|"]
        for n, nfiles, _lines, fpct, lpct in curve(files, feats):
            label = "supported today" if n == 0 else f"milestone {n}"
            rows.append(f"| {label} | {nfiles} | {fpct:.1f} | {lpct:.1f} |")
        return rows

    md = [
        "# ts-rust coverage",
        "",
        "Generated by `python3 scripts/port-scan-sync.py --coverage`. Do not edit the numbers by hand.",
        "Feature statuses come from the probes in `tools/lust-port/tests/features/`",
        "(`python3 scripts/port-scan-sync.py --write`). A file counts as compiling",
        "when every construct the scanner sees in it maps to a supported feature.",
        "The scanner has no type data, so this is an upper bound.",
        "",
        f"lust commit: `{lust_head}` (the scanned tree, landed on `next`). ts-rust: `{ts_head}`.",
        "`docs/ts-rust-gaps.md` still holds the milestone-3 table scan and its inventory comparison; this file is the current burn-down.",
        f"scan: {rep['totals']['files']} files, {rep['totals']['lines']} lines, target x86_64-unknown-linux-gnu.",
        "Dev features (docs/dev-features.md): `cfg(feature = ...)` uses each package's effective dev"
        " selection. ts-rust dev builds run with `jemalloc` off:"
        + (
            " " + ", ".join(f"`{pkg}` = [{', '.join(fs) if fs else 'none'}]" for pkg, fs in sorted(rep.get("cargo_features", {}).items())) + "."
            if rep.get("cargo_features")
            else " not recorded by this scanner build."
        ),
        "",
        "## Core CLI scope today",
        "",
        "Core scope is the PLAN.md milestone 8 set, with the inventory rule:",
        "prod files active on this target whose top module is not ls, lsp, project,",
        "api, effect, format, fswatch, ipc, or contentmapper.",
        "The inventory counted 402 files and 388,041 lines on the same ts-rust commit.",
        "This scanner counts 406: it adds the 5 codegen tool files cargo reports as",
        "lib and bin targets, and drops `cmd/tsgo/isprocessalive_other.rs`, which is",
        "inactive on this target.",
        "",
        f"Core: {len(core)} files, {core_lines} lines. Compiles today: {len(today)} files, {today_lines} lines"
        f" ({100.0 * len(today) / max(len(core), 1):.1f}% of files, {100.0 * today_lines / max(core_lines, 1):.1f}% of lines).",
        "",
        "All active prod files: "
        f"{rep['coverage']['supported_today_files']} of {rep['coverage']['total_files']} files compile today"
        f" ({rep['coverage']['by_plan_milestone'][0]['lines_pct']}% of lines).",
        "",
        "## Top 15 blocking features, core scope",
        "",
        "`lines blocked` counts every core file that uses the feature. `sole` counts",
        "files where this feature is the only blocker: land it and those files compile.",
        "",
        "| # | feature | status | milestone | owner | files | lines blocked | sole files | sole lines |",
        "|---|---|---|---|---|---|---|---|---|",
    ]
    for i, b in enumerate(top, 1):
        ms = "none" if b["milestone"] is None else str(b["milestone"])
        md.append(
            f"| {i} | `{b['id']}` | {b['status']} | {ms} | {b['owner']} | {b['files']} | {b['lines']} | {b['sole_files']} | {b['sole_lines']} |"
        )
    md += ["", "## Core CLI curve", ""] + curve_table(core)
    md += ["", "## All active prod curve", ""] + curve_table(prod)
    md += [
        "",
        "## Reproduce",
        "",
        "```sh",
        "export LUST_TCC_HOME=/home/theo/.t3/projects/lust-lang/third_party/tcc",
        "cargo build --release",
        "cargo build --release --manifest-path tools/lust-port/Cargo.toml",
        "python3 scripts/port-scan-sync.py --write",
        "# The scanner reads the lust dev config from the ts-rust tree; on a tree without one,",
        "# pass the dev selection by env (jemalloc off for ts-rust dev builds):",
        "#   LUST_DEV_EXCLUDE_FEATURES=jemalloc TS_RUST=/path/to/ts-rust python3 scripts/port-scan-sync.py --coverage",
        "python3 scripts/port-scan-sync.py --coverage",
        "```",
        "",
    ]
    out_path = ROOT / "docs/ts-rust-coverage.md"
    out_path.write_text("\n".join(md))
    print(f"wrote {gaps_path}")
    print(f"wrote {out_path}")
    print(f"core: {len(core)} files, {core_lines} lines; today {len(today)} files, {today_lines} lines")
    for b in top[:5]:
        print(f"  blocker {b['id']}: {b['lines']} lines, sole {b['sole_lines']}")


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = ap.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true", help="write probe statuses into features.json")
    mode.add_argument("--check", action="store_true", help="fail when a committed status differs from the probe result")
    mode.add_argument("--coverage", action="store_true", help="scan ts-rust and write the coverage docs")
    ap.add_argument("--json", help="write raw probe results to this file")
    ap.add_argument("--jobs", type=int, default=0, help="probe workers (default: CPU count, max 8; 1 is serial)")
    args = ap.parse_args()
    if args.jobs < 0:
        ap.error("--jobs must be 0 (auto) or a positive worker count")
    if args.coverage:
        do_coverage(args)
    else:
        do_probes(args)


if __name__ == "__main__":
    main()
