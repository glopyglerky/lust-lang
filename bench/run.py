#!/usr/bin/env python3
"""The lust benchmark suite: one labeled table a reader can trust.

Rows:
  - small file (2 lines): check, build, and run, both compilers
  - fresh cold check/build on bench/real, the 31k warehouse crate, the 48k
    generated crate (lust in a fresh process with no daemon; rustc gets the
    best median of {default, -Z threads=8}, debuginfo=0 for both sides)
  - cached cold check/build on the same corpora (new process, no edit since
    a priming compile: rustc incremental cache vs lust disk cache)
  - warm edit loop: bench/realish/agent_loop.py totals (lust daemon vs
    rustc incremental, 12 edits on the 31k crate)
  - 8-core row: fresh cold build of the 48k crate under `taskset -c 0-7`
    (skipped with a note where taskset does not exist, e.g. macOS)
  - run time of the built bench/real program (context: tcc does not optimize)

Method, same rules as bench/compare.sh: matched flags (edition 2024,
panic=abort, overflow checks on, debuginfo=0 unless a row says otherwise),
full-process wall time, interleaved rounds in rotating order, median and
p90. Every corpus is also built once per compiler and the program outputs
must match byte for byte, or the run fails.

Usage: python3 bench/run.py [--reps N] [--quick] [--json PATH] [--md PATH]
       --quick: 1 rep, small + bench/real corpora (edit loop still full size;
       combine with --skip-edit-loop for a smoke run).
"""
import argparse
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LUST = os.path.join(ROOT, "target/release/lust")
RUSTC_BASE = ["rustc", "--edition", "2024", "-C", "panic=abort", "-C", "overflow-checks=on", "-A", "warnings"]
BOOTSTRAP = dict(os.environ, RUSTC_BOOTSTRAP="1")


def timed(cmd, cwd, env=None):
    start = time.perf_counter()
    r = subprocess.run(cmd, cwd=cwd, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    return time.perf_counter() - start, r


def stats(samples):
    s = sorted(samples)
    med = statistics.median(s)
    p90 = s[min(len(s) - 1, int(len(s) * 0.9 + 0.999) - 1)] if s else 0.0
    return med, p90


def interleave(jobs, reps, cwd):
    """jobs: list of (tag, cmd, env). Runs reps rounds, rotating start order.
    Returns {tag: (median, p90)}."""
    samples = {tag: [] for tag, _, _ in jobs}
    for rep in range(reps):
        for i in range(len(jobs)):
            tag, cmd, env = jobs[(i + rep) % len(jobs)]
            t, r = timed(cmd, cwd, env)
            assert r.returncode == 0, f"{tag} failed: {' '.join(cmd)}\n{r.stderr[:2000]}"
            samples[tag].append(t)
    return {tag: stats(v) for tag, v in samples.items()}


def rustc_cmd(config, kind, src, out, inc_dir=None):
    cmd = list(RUSTC_BASE) + ["-C", "debuginfo=0"]
    env = None
    if "threads" in config:
        cmd += ["-Z", "threads=8"]
        env = BOOTSTRAP
    if inc_dir:
        cmd += ["-C", f"incremental={inc_dir}"]
    if kind == "check":
        cmd += ["--emit=metadata", "-o", out + ".rmeta", src]
    else:
        cmd += ["-o", out, src]
    return cmd, env


def lust_cmd(kind, src, out):
    if kind == "check":
        return [LUST, "check", src]
    return [LUST, "build", src, "-o", out]


def cold_env(work):
    # A truly cold lust compile: socket that never exists, daemon autostart
    # off, whole-request disk cache off.
    return dict(os.environ, LUST_DAEMON_SOCKET=os.path.join(work, "no-daemon.sock"),
                LUST_NO_DAEMON="1", LUST_NO_CACHE="1")


def disk_cache_env(cache_dir):
    # Cold process, daemon off, but the on-disk request cache is on and kept
    # in cache_dir: the lust counterpart of a primed rustc incremental cache.
    return dict(os.environ, LUST_NO_DAEMON="1", LUST_CACHE_DIR=cache_dir)


class Suite:
    def __init__(self, reps):
        self.reps = reps
        self.rows = []
        self.work = tempfile.mkdtemp(prefix="lust-bench-run-")

    def row(self, label, corpus, lines, op, rustc_config, rustc_stats, lust_stats, notes=""):
        rmed, rp90 = rustc_stats
        lmed, lp90 = lust_stats
        self.rows.append({
            "label": label, "corpus": corpus, "lines": lines, "op": op,
            "rustc_config": rustc_config,
            "rustc_median_s": round(rmed, 4), "rustc_p90_s": round(rp90, 4),
            "lust_median_s": round(lmed, 4), "lust_p90_s": round(lp90, 4),
            "speedup": round(rmed / lmed, 1) if lmed > 0 else None,
            "notes": notes,
        })
        print(f"  {label}: rustc {rmed:.3f}s ({rustc_config}), lust {lmed:.3f}s", flush=True)

    def corpus_file(self, name, generator):
        path = os.path.join(self.work, name + ".rs")
        with open(path, "w") as f:
            generator(f)
        with open(path) as f:
            lines = sum(1 for _ in f)
        return path, lines

    def check_same_output(self, name, src):
        out_r = os.path.join(self.work, name + "-rustc")
        out_l = os.path.join(self.work, name + "-lust")
        cmd, _ = rustc_cmd("default", "build", src, out_r)
        r = subprocess.run(cmd, cwd=self.work, capture_output=True, text=True)
        assert r.returncode == 0, f"{name}: rustc build failed\n{r.stderr[:2000]}"
        r = subprocess.run(lust_cmd("build", src, out_l), cwd=self.work, env=cold_env(self.work), capture_output=True, text=True)
        assert r.returncode == 0, f"{name}: lust build failed\n{r.stderr[:2000]}"
        a = subprocess.run([out_r], capture_output=True)
        b = subprocess.run([out_l], capture_output=True)
        assert a.stdout == b.stdout and a.returncode == b.returncode, f"{name}: program output differs"
        return out_r, out_l

    def fresh_cold(self, label, corpus, src, lines, op, prefix=None):
        """Fresh cold check/build. rustc: best of default/threads8, no incremental."""
        jobs = []
        for cfg in ("default", "threads8"):
            out = os.path.join(self.work, f"{corpus}-{op}-{cfg}")
            cmd, env = rustc_cmd(cfg, op, src, out)
            if prefix:
                cmd = prefix + cmd
            jobs.append((f"rustc:{cfg}", cmd, env))
        lcmd = lust_cmd(op, src, os.path.join(self.work, f"{corpus}-{op}-lust"))
        if prefix:
            lcmd = prefix + lcmd
        jobs.append(("lust", lcmd, cold_env(self.work)))
        got = interleave(jobs, self.reps, self.work)
        best = min(("default", "threads8"), key=lambda c: got[f"rustc:{c}"][0])
        self.row(label, corpus, lines, op, best, got[f"rustc:{best}"], got["lust"])

    def cached_cold(self, label, corpus, src, lines, op):
        """Cached cold: new process, no edit since a priming compile.
        rustc uses its primed incremental cache; lust uses its primed
        whole-request disk cache (daemon off in both cases)."""
        inc = os.path.join(self.work, f"inc-{corpus}-{op}")
        out_r = os.path.join(self.work, f"{corpus}-{op}-inc")
        cmd, env = rustc_cmd("threads8", op, src, out_r, inc_dir=inc)
        r = subprocess.run(cmd, cwd=self.work, env=env, capture_output=True, text=True)
        assert r.returncode == 0, f"{corpus}: rustc prime failed\n{r.stderr[:2000]}"
        lcmd = lust_cmd(op, src, os.path.join(self.work, f"{corpus}-{op}-lust2"))
        lenv = disk_cache_env(os.path.join(self.work, f"lustcache-{corpus}-{op}"))
        r = subprocess.run(lcmd, cwd=self.work, env=lenv, capture_output=True, text=True)
        assert r.returncode == 0, f"{corpus}: lust prime failed\n{r.stderr[:2000]}"
        jobs = [("rustc:inc", cmd, env), ("lust", lcmd, lenv)]
        got = interleave(jobs, self.reps, self.work)
        self.row(label, corpus, lines, op, "threads8+inc", got["rustc:inc"], got["lust"],
                 notes="caches primed, no edit; new process, daemon off")


def gen_file(args):
    def go(f):
        subprocess.run([sys.executable] + args, stdout=f, check=True)
    return go


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--reps", type=int, default=3)
    ap.add_argument("--quick", action="store_true")
    ap.add_argument("--json", default=None, help="write JSON results here")
    ap.add_argument("--md", default=None, help="also write the markdown table here")
    ap.add_argument("--skip-edit-loop", action="store_true")
    a = ap.parse_args()
    reps = 1 if a.quick else a.reps
    suite = Suite(reps)
    print(f"work dir {suite.work}, reps {reps}")

    # --- small file: 2 lines, check/build/run ---
    print("small file")
    small, small_lines = suite.corpus_file("small", lambda f: f.write('// 2-line smoke program.\nfn main() { println!("hi"); }\n'))
    prog_r, prog_l = suite.check_same_output("small", small)
    suite.fresh_cold("small file (2 lines), fresh cold", "small", small, small_lines, "check")
    suite.fresh_cold("small file (2 lines), fresh cold", "small", small, small_lines, "build")
    jobs = [("rustc:run", [prog_r], None), ("lust", [prog_l], None)]
    got = interleave(jobs, max(reps, 5), suite.work)
    suite.row("small file (2 lines), run built program", "small", small_lines, "run", "debug binary", got["rustc:run"], got["lust"])

    # --- corpora ---
    corpora = []
    real_src = os.path.join(ROOT, "bench/real/main.rs")
    real_dst = os.path.join(suite.work, "real.rs")
    shutil.copyfile(real_src, real_dst)
    with open(real_dst) as f:
        corpora.append(("bench/real (logscan, non-cloned)", "real", real_dst, sum(1 for _ in f)))
    if not a.quick:
        p31, l31 = suite.corpus_file("crate31k", gen_file([os.path.join(ROOT, "bench/realish/make.py"), "40"]))
        corpora.append(("31k crate (40 warehouse modules)", "crate31k", p31, l31))
        p48, l48 = suite.corpus_file("plain48k", gen_file([os.path.join(ROOT, "bench/rustc-baseline/gen.py"), "plain", "3200", "0"]))
        corpora.append(("48k crate (generated plain)", "plain48k", p48, l48))

    for label, name, src, lines in corpora:
        print(label)
        suite.check_same_output(name, src)
        suite.fresh_cold(f"{label}, fresh cold", name, src, lines, "check")
        suite.fresh_cold(f"{label}, fresh cold", name, src, lines, "build")
        suite.cached_cold(f"{label}, cached cold", name, src, lines, "check")
        suite.cached_cold(f"{label}, cached cold", name, src, lines, "build")

    # Run time of the built bench/real program.
    print("bench/real run time")
    jobs = [("rustc:run", [os.path.join(suite.work, "real-rustc")], None),
            ("lust", [os.path.join(suite.work, "real-lust")], None)]
    got = interleave(jobs, max(reps, 3), suite.work)
    suite.row("bench/real, run built program", "real", corpora[0][3], "run", "debug binary",
              got["rustc:run"], got["lust"], notes="tcc output is unoptimized; context only")

    # --- 8-core row on the 48k crate ---
    if not a.quick:
        print("8-core row")
        _, _, p48, l48 = corpora[2]
        if shutil.which("taskset"):
            suite.fresh_cold("48k crate, fresh cold, 8 cores (taskset)", "plain48k", p48, l48, "build", prefix=["taskset", "-c", "0-7"])
        else:
            suite.rows.append({"label": "48k crate, fresh cold, 8 cores (taskset)", "corpus": "plain48k",
                               "lines": l48, "op": "build", "rustc_config": None, "rustc_median_s": None,
                               "rustc_p90_s": None, "lust_median_s": None, "lust_p90_s": None,
                               "speedup": None, "notes": "taskset not available on this platform"})
            print("  taskset missing, row skipped")

    # --- warm edit loop ---
    if not a.skip_edit_loop:
        print("warm edit loop")
        modules = "40"  # agent_loop edits modules up to 38; it cannot shrink.
        r = subprocess.run([sys.executable, os.path.join(ROOT, "bench/realish/agent_loop.py"), modules],
                           capture_output=True, text=True, cwd=ROOT)
        assert r.returncode == 0, f"agent_loop failed\n{r.stdout[-2000:]}\n{r.stderr[-2000:]}"
        out = r.stdout
        crate_lines = int(re.search(r"crate: \d+ modules, (\d+) lines", out).group(1))
        for op, pat in (("check", r"total check: rustc ([\d.]+) s, lust ([\d.]+) s"),
                        ("build", r"total build: rustc ([\d.]+) s, lust ([\d.]+) s"),
                        ("check+build", r"edit loop \(check \+ build, \d+ edits\): rustc ([\d.]+) s, lust ([\d.]+) s")):
            m = re.search(pat, out)
            assert m, f"could not parse agent_loop totals for {op}"
            rs, ls = float(m.group(1)), float(m.group(2))
            suite.rows.append({
                "label": f"warm edit loop, 12 edits total ({modules} modules)", "corpus": "crate31k",
                "lines": crate_lines, "op": op, "rustc_config": "threads8+inc",
                "rustc_median_s": rs, "rustc_p90_s": rs, "lust_median_s": ls, "lust_p90_s": ls,
                "speedup": round(rs / ls, 1), "notes": "single pass of 12 edits, sums (agent_loop.py)",
            })
            print(f"  warm {op}: rustc {rs:.2f}s, lust {ls:.3f}s")

    md = render_md(suite.rows, reps)
    print("\n" + md)
    if a.md:
        with open(a.md, "w") as f:
            f.write(md + "\n")
    if a.json:
        meta = {
            "tool": "bench/run.py", "reps": reps, "cores": os.cpu_count(),
            "rustc": subprocess.run(["rustc", "--version"], capture_output=True, text=True).stdout.strip(),
            "git_rev": subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, capture_output=True, text=True).stdout.strip(),
            "platform": sys.platform,
        }
        with open(a.json, "w") as f:
            json.dump({"meta": meta, "rows": suite.rows}, f, indent=2)
        print(f"json written to {a.json}")
    shutil.rmtree(suite.work, ignore_errors=True)


def fmt_s(v):
    if v is None:
        return "-"
    return f"{v * 1000:.0f} ms" if v < 1 else f"{v:.2f} s"


def render_md(rows, reps):
    out = [f"Benchmark suite (bench/run.py, reps={reps}, median of interleaved rounds; p90 in JSON).",
           "Matched settings: edition 2024, panic=abort, overflow checks on, debuginfo=0.",
           "",
           "| Row | Lines | Op | rustc best | rustc | lust | Speedup |",
           "|---|---:|---|---|---:|---:|---:|"]
    for r in rows:
        cfg = r["rustc_config"] or "-"
        sp = f"{r['speedup']}x" if r["speedup"] else "-"
        out.append(f"| {r['label']} | {r['lines']} | {r['op']} | {cfg} | {fmt_s(r['rustc_median_s'])} | {fmt_s(r['lust_median_s'])} | {sp} |")
    notes = [r for r in rows if r["notes"]]
    if notes:
        out.append("")
        for r in notes:
            out.append(f"- {r['label']} ({r['op']}): {r['notes']}")
    return "\n".join(out)


if __name__ == "__main__":
    main()
