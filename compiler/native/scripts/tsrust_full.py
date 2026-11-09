#!/usr/bin/env python3
"""Whole-workspace census of the ts-rust fork (third_party/ts-rust) under lust.

scripts/tsrust-full.sh calls this after it has the fork tree. Steps:
  cargo   build and test the fork with Cargo (the fork must stay green under rustc)
  check   `lust check` of every lib, bin and cdylib target of the workspace
  build   `lust build` of every bin target whose check passed (lust builds bins only)
  parity  (--parity) scripts/tsrust_parity.py: lust bins vs Cargo bins on the fork's inputs
  report  errors grouped by root cause (code + normalized message), with counts
           and the first span, then the last line
           "tsrust-full: <ok>/<targets> check, <ok>/<bins> build, <k> root causes"

lust stops a crate at its first failing stage (expansion, resolve, check), so a
target shows only the errors of that stage. Fix the top rows and run it again.
Dev features come from the fork's .lust.toml (docs/dev-features.md).

Usage: tsrust_full.py --fork DIR --lust PATH --out DIR [--skip-cargo] [--skip-build]
                      [--only NAME,...] [-j N] [--timeout SECS] [--parity]
"""
import argparse
import json
import os
import re
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

TARGET_KINDS = ("lib", "bin", "cdylib")
# Fork tests that read the Go checkout's testdata (TS_GO_REPO), which is not here.
NEEDS_GO_CHECKOUT = ["format::api_test::test_format_format_checker_ts"]


def targets(fork):
    """Every lib/bin/cdylib target of the workspace, from `cargo metadata`.

    Tests and examples are left out: lust has no flag to select them. Cargo
    builds and runs them in the cargo step."""
    r = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--offline"],
        cwd=fork,
        capture_output=True,
        text=True,
    )
    if r.returncode != 0:
        sys.exit(f"tsrust-full: cargo metadata failed:\n{r.stderr[-2000:]}")
    out = []
    for p in json.loads(r.stdout)["packages"]:
        for t in p["targets"]:
            kind = next((k for k in t["kind"] if k in TARGET_KINDS), None)
            if kind is None:
                continue
            name = p["name"] if kind != "bin" else f"{p['name']}:{t['name']}"
            out.append({"name": name, "kind": kind, "bin": t["name"], "src": t["src_path"], "manifest": p["manifest_path"]})
    return out


# The fork's library crates in dependency order, for the deep and build censuses (goport_core
# only on forks that have it, v0.7.0 and later). The codegen tools are leaf crates of their own.
CENSUS_LIBS = ["goport_util", "goport_lsproto", "goport_core", "ts_goport"]
TOOL_LIBS = ["ts_ast_codegen", "ts_diagnostics_codegen"]


def census_libs(fork):
    """CENSUS_LIBS the fork has, comma-joined. Exits when the fork has a lib outside CENSUS_LIBS
    and TOOL_LIBS, so a new crate is never left out of a census without a word."""
    names = {t["name"] for t in targets(fork) if t["kind"] == "lib"}
    new = sorted(names - set(CENSUS_LIBS) - set(TOOL_LIBS))
    if new:
        sys.exit(f"tsrust-full: fork libs not in CENSUS_LIBS (add them in dependency order): {', '.join(new)}")
    return ",".join(n for n in CENSUS_LIBS if n in names)


def run_cargo(fork, out, log):
    """cargo test --release --workspace, then cargo build --release --bins (same profile, so the
    second step reuses the first). Returns (ok, summary line)."""
    # One test thread: the fork's tests share process state (its scripts/goport/goport-tests.sh
    # sets RUST_TEST_THREADS=1; with more threads about 67 ts_goport lib tests fail).
    env = dict(os.environ, CARGO_TARGET_DIR=str(out / "cargo-target"), RUST_TEST_THREADS="1")
    skips = [a for t in NEEDS_GO_CHECKOUT for a in ("--skip", t)]
    steps = [
        ["cargo", "test", "--release", "--workspace", "--offline", "--no-fail-fast", "--"] + skips,
        ["cargo", "build", "--release", "--workspace", "--bins", "--offline"],
    ]
    passed = failed = 0
    with open(log, "w") as f:
        for cmd in steps:
            f.write(f"$ {' '.join(cmd)}\n")
            f.flush()
            t0 = time.time()
            r = subprocess.run(cmd, cwd=fork, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            f.write(r.stdout + f"\n# rc={r.returncode} {time.time() - t0:.0f}s\n")
            for m in re.finditer(r"test result: \w+\. (\d+) passed; (\d+) failed", r.stdout):
                passed += int(m.group(1))
                failed += int(m.group(2))
            if r.returncode != 0:
                return False, f"cargo: FAIL ({' '.join(cmd[:2])} rc={r.returncode}; {passed} passed, {failed} failed; log {log})"
    return True, f"cargo: ok ({passed} tests passed, {failed} failed)"


def lust_cmd(lust, verb, t, out):
    cmd = [lust, verb, t["src"], "--manifest-path", t["manifest"], "--json", "--max-errors", "1000", "--offline"]
    cmd += ["--lib"] if t["kind"] != "bin" else ["--bin", t["bin"]]
    if verb == "build":
        cmd += ["-o", str(out / "bin" / t["name"].replace(":", "-"))]
    return cmd


def cargo_feature_args(t, fork):
    """Cargo flags that select the features a lust dev check of target `t` uses: the package
    `default` list minus the fork's .lust.toml `dev-exclude-features` (or its exact
    `dev-features`), as src/protocol.rs reads them. Used where cargo must check the same
    cfg as lust (tsrust_deep.py --rustc, scripts/tsedit.py)."""
    import tomllib

    pkg = tomllib.loads(Path(t["manifest"]).read_text())
    table = pkg.get("features", {})
    cfg_path = Path(fork) / ".lust.toml"
    cfg = tomllib.loads(cfg_path.read_text()) if cfg_path.exists() else {}
    if "dev-features" in cfg:
        feats = cfg["dev-features"]
    else:
        feats = [f for f in table.get("default", []) if f not in cfg.get("dev-exclude-features", [])]
    args = ["--no-default-features"]
    return args + ["--features", ",".join(feats)] if feats else args


def cargo_check_cmd(t, fork):
    """`cargo check` of one target with lust's dev features (short messages, offline)."""
    pkg = t["name"].split(":")[0]
    sel = ["--lib"] if t["kind"] != "bin" else ["--bin", t["bin"]]
    return ["cargo", "check", "-p", pkg, *sel, "--offline", "--message-format", "short", *cargo_feature_args(t, fork)]


def run_lust(lust, verb, t, out, timeout):
    """One lust check or build. Returns a result dict: status ok | error | hang | crash, and the
    diagnostics. A timeout is a hang, never a reject (BRIEF rule)."""
    env = dict(os.environ, LUST_NO_DAEMON="1", LUST_CACHE_DIR=str(out / "lust-cache"), CARGO_TARGET_DIR=str(out / "lust-cargo-target"))
    cmd = lust_cmd(lust, verb, t, out)
    t0 = time.time()
    try:
        r = subprocess.run(cmd, cwd=out, env=env, capture_output=True, text=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        return {"target": t["name"], "verb": verb, "status": "hang", "secs": timeout, "diags": [], "stderr": ""}
    diags = []
    for line in r.stdout.splitlines():
        try:
            d = json.loads(line)
        except ValueError:
            continue
        if d.get("kind") == "diagnostic" and d.get("severity") == "error":
            diags.append(d)
    if r.returncode == 0:
        status = "ok"
    elif diags:
        status = "error"
    else:
        status = "crash"  # nonzero exit with no diagnostic: a panic, a tcc failure, a driver error
    return {
        "target": t["name"],
        "verb": verb,
        "status": status,
        "rc": r.returncode,
        "secs": round(time.time() - t0, 1),
        "diags": diags,
        "stderr": r.stderr[-4000:],
    }


def norm_code_text(m):
    """One `...` span of a message: module paths and generic arguments dropped, so
    `Result<Vec<(String, Entry)>, x::GenerateError>` and `Result<Vec<_>, _>` are one key."""
    t = re.sub(r"<.*>", "<..>", m.group(1))
    return "`" + re.sub(r"\b[A-Za-z_]\w*::", "", t) + "`"


def norm_message(msg, fork):
    """Root cause key text: fork paths made relative, code spans reduced, numbers masked."""
    msg = msg.replace(str(fork) + "/", "")
    msg = re.sub(r"`([^`]*)`", norm_code_text, msg)
    return re.sub(r"\d+", "N", msg)


def rel(path, fork):
    try:
        return os.path.relpath(os.path.realpath(path), os.path.realpath(fork))
    except ValueError:
        return path


def crash_reason(res):
    """One line for a target that failed with no diagnostic: the last meaningful stderr line."""
    lines = [ln.strip() for ln in res["stderr"].splitlines() if ln.strip()]
    for ln in reversed(lines):
        if "panicked" in ln or "error" in ln.lower():
            return ln[:200]
    return lines[-1][:200] if lines else f"exit {res.get('rc')}, no output"


def group(results, fork):
    """Root causes over all targets: one site (file, span, code) counts once even when several
    targets reach it (ts_goport's bins all reach its lib)."""
    causes = {}
    for res in results:
        if res["status"] in ("hang", "crash"):
            key = (res["status"].upper(), norm_message(crash_reason(res), fork))
            c = causes.setdefault(key, {"sites": set(), "targets": set(), "first": res["target"], "verbs": set()})
            c["sites"].add(res["target"])
            c["targets"].add(res["target"])
            c["verbs"].add(res["verb"])
            continue
        for d in res["diags"]:
            key = (d.get("code", "?"), norm_message(d.get("message", ""), fork))
            sp = d.get("span") or {}
            site = f"{rel(d.get('file', '?'), fork)}:{sp.get('line', 0)}:{sp.get('col', 0)}"
            c = causes.setdefault(key, {"sites": set(), "targets": set(), "first": site, "verbs": set()})
            c["sites"].add(site)
            c["targets"].add(res["target"])
            c["verbs"].add(res["verb"])
    rows = [
        {"code": k[0], "message": k[1], "count": len(v["sites"]), "targets": sorted(v["targets"]), "first": v["first"], "verbs": sorted(v["verbs"])}
        for k, v in causes.items()
    ]
    rows.sort(key=lambda r: (-len(r["targets"]), -r["count"], r["code"], r["message"]))
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--fork", required=True, type=Path)
    ap.add_argument("--lust", required=True)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--skip-cargo", action="store_true")
    ap.add_argument("--skip-build", action="store_true")
    ap.add_argument("--only", default="", help="comma list of target names (package or package:bin)")
    ap.add_argument("-j", type=int, default=4)
    # A lust build of a ts_goport bin peaks at about 10.6 GB (fx-buildbins 16:44); 3 at once on a
    # 32 GB boat box get killed by the OOM killer.
    ap.add_argument("--build-j", type=int, default=2)
    ap.add_argument("--timeout", type=int, default=1800)
    ap.add_argument("--parity", action="store_true", help="after the builds, run tsrust_parity.py (needs the cargo step)")
    a = ap.parse_args()
    fork = a.fork.resolve()
    out = a.out.resolve()
    (out / "bin").mkdir(parents=True, exist_ok=True)
    if not (fork / ".lust.toml").exists():
        print("tsrust-full: warning: the fork has no .lust.toml (dev features); jemalloc stays on", flush=True)

    cargo_line = "cargo: skipped"
    if not a.skip_cargo:
        ok, cargo_line = run_cargo(fork, out, out / "cargo.log")
    print(f"tsrust-full: {cargo_line}", flush=True)

    ts = targets(fork)
    if a.only:
        want = set(a.only.split(","))
        ts = [t for t in ts if t["name"] in want]
    with ThreadPoolExecutor(a.j) as pool:
        checks = list(pool.map(lambda t: run_lust(a.lust, "check", t, out, a.timeout), ts))
    for r in checks:
        print(f"tsrust-full: check {r['target']}: {r['status']} ({len(r['diags'])} errors, {r['secs']}s)", flush=True)
    builds = []
    if not a.skip_build:
        # lust builds bin targets only (a lib has no main and lust has no rlib).
        ok_ts = [t for t, r in zip(ts, checks) if r["status"] == "ok" and t["kind"] == "bin"]
        with ThreadPoolExecutor(a.build_j) as pool:
            builds = list(pool.map(lambda t: run_lust(a.lust, "build", t, out, a.timeout), ok_ts))
        for r in builds:
            print(f"tsrust-full: build {r['target']}: {r['status']} ({len(r['diags'])} errors, {r['secs']}s)", flush=True)

    rows = group(checks + builds, fork)
    (out / "results.json").write_text(json.dumps({"cargo": cargo_line, "check": checks, "build": builds, "causes": rows}, indent=1))
    print("\nroot causes (code, normalized message, sites, targets, first span):")
    for i, r in enumerate(rows, 1):
        verb = "" if r["verbs"] == ["check"] else f" [{'/'.join(r['verbs'])}]"
        print(f"{i:3}. {r['code']} {r['message']}{verb}\n     {r['count']} sites, {len(r['targets'])} targets, first {r['first']}")
    if a.parity:
        here = Path(__file__).resolve().parent
        subprocess.run([sys.executable, str(here / "tsrust_parity.py"), "--fork", str(fork), "--cargo-bins", str(out / "cargo-target/release"),
                        "--lust-bins", str(out / "bin"), "--out", str(out)])
    n = len(ts)
    n_bins = sum(t["kind"] == "bin" for t in ts)
    c_ok = sum(r["status"] == "ok" for r in checks)
    b_ok = sum(r["status"] == "ok" for r in builds)
    print(f"tsrust-full: {c_ok}/{n} check, {b_ok}/{n_bins} build, {len(rows)} root causes")


if __name__ == "__main__":
    main()
