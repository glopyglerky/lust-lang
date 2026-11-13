#!/usr/bin/env python3
"""The speed benchmark on a pinned ts-rust slice (bench/slices/<name>): lust vs cargo.

Entry point: `scripts/scale-gate.py --slice bench/slices/<name> [options]` (this file
holds the slice mode). `speed_slice.py --render docs/speed.md a.json [b.json ...]`
rewrites the results section of docs/speed.md from result JSONs. The promise it tracks: 100x faster WARM checks after a
one-function edit, on identical source. Cold check, build, and restart are
separate rows.

Both compilers work on ONE source tree, so every measured state is the same
source for both. Per slice it records:
  - cold check and cold build: lust one-shot (no daemon, empty cache) vs
    `cargo check` / `cargo build` after `cargo clean`
  - daemon first check (new session, empty cache), cached restart (new daemon,
    disk cache), no-op check (warm daemon, nothing changed) vs a no-op cargo check
  - body edits: 100+ edits that alternate across fns in different files; each
    inserts or changes one `let` in one fn body; check after every edit
  - signature, type, impl, error, and fix edits from the slice's edits.json,
    each applied and reverted in several cycles
  - warm build after a body edit (a subset of the body edits)
  - wall median, p95, worst per row, and the rustc/lust ratio for median
    and p95; lust phase times (--timings) per row; C bytes, units compiled,
    and link time (LUST_STATS, build rows); peak RSS (the lust daemon's
    VmHWM; for cargo, the largest process in its tree)
  - one record per lust request in the result JSON (`requests`): row, edit
    id, target fn, wall, every phase, and the `lust-visits:` counters, with
    the body-patch and kept-program (W-INCR3 `kept=`) hit or miss and its
    reason. Phase medians count a phase a request did not emit as zero
    (`lust_phases_median_ms`); `lust_phases_when_present_median_ms` keeps
    the old when-present view, with `lust_phases_present_n` counts.

rustc runs in two configs, each with its own target dir: `default` (stable dev
profile, incremental on) and `threads8` (RUSTC_BOOTSTRAP=1, -Z threads=8). Both
use debuginfo 0. The ratio uses the faster config's median per row.

Exactness, checked on every state: lust and cargo agree on pass or fail (exit
code). Every `--cold-every` states, and on every non-body edit, a cold one-shot
lust check runs on the same state; its stdout, stderr (timing lines removed),
and exit code must equal the warm daemon answer. A mismatch stops the run.

Dev features: when prune.json records `dev_features` (the effective
list prune.py used, docs/dev-features.md), both sides replay it: lust
gets `--features`, and the generated slice Cargo.toml declares those
features as its defaults, so cfg(feature = ...) means the same thing
in both compilers. Old slices without the field keep no features.

The slice's empty `main` is replaced by a call to bench/slices/driver.rs (it
calls kept ts-rust tspath fns and prints the results). After every build, the
lust binary and each rustc binary run; stdout, stderr, and exit code must match.
That gives the run row and the build+run rows (--no-driver turns it off).
"""
import json
import os
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LUST = ROOT / "target/release/lust"
ITEMS = ROOT / "bench/slices/items/target/release/slice-items"
EDIT_TAG = "_lust_bench_edit"
TIMING_LINE = re.compile(r"^  \S.*\s[\d.]+ ms$")
# The --timings header line names the request mode (full, bodies, ...), which
# differs between a warm and a cold answer by design.
TIMING_HEAD = re.compile(r"^lust (check|build) .*: ok \(")

RUSTC_CONFIGS = {
    "default": {},
    "threads8": {"RUSTC_BOOTSTRAP": "1", "RUSTFLAGS": "-Z threads=8"},
}


def pct(xs, p):
    xs = sorted(xs)
    if not xs:
        return None
    k = (len(xs) - 1) * p / 100
    lo = int(k)
    hi = min(lo + 1, len(xs) - 1)
    return xs[lo] + (xs[hi] - xs[lo]) * (k - lo)


def summary(xs):
    if not xs:
        return None
    return {"n": len(xs), "median": round(statistics.median(xs), 2), "p95": round(pct(xs, 95), 2),
            "worst": round(max(xs), 2)}


def parse_timings(stderr):
    """lust --timings stage lines -> {stage: ms}."""
    out = {}
    for line in stderr.splitlines():
        if TIMING_LINE.match(line):
            name, ms = line.strip().rsplit(None, 2)[0], line.strip().rsplit(None, 2)[1]
            try:
                out[name.strip()] = out.get(name.strip(), 0.0) + float(ms)
            except ValueError:
                pass
    return out


def parse_stats(stderr):
    out = {}
    for line in stderr.splitlines():
        if line.startswith("lust-stats:"):
            for kv in line.split(":", 1)[1].split():
                k, _, v = kv.partition("=")
                out[k] = int(v) if v.isdigit() else v
    return out


def parse_visits(stderr):
    """The last `lust-visits:` line (needs LUST_STATS) -> {counter: value}."""
    out = {}
    for line in stderr.splitlines():
        if line.startswith("lust-visits:"):
            out = {}
            for kv in line.split(":", 1)[1].split():
                k, _, v = kv.partition("=")
                out[k] = int(v) if v.isdigit() else v
    return out


def classify_request(visits, phases):
    """Body-patch and kept-program hit or miss for one request, with reasons.

    Patch: `path=patch` is a hit; `path=resolve`/`chunks` is a full-resolve
    miss and `why=` names the condition that stopped the patch. Kept
    program (W-INCR3): `kept=reuse` is a hit, any other value (`mode`,
    `fn_class`, `discovery`, `body_scan`, `gen_state`, `none`) is the miss
    reason, and a visits line with no `kept=` field at all means this lust
    build does not expose it yet (`not_exposed`, the hook for W-INCR3).
    """
    why = visits.get("why")
    why = None if why in (None, "-", "") else why
    path = visits.get("path")
    if path == "patch":
        patch = {"status": "hit", "reason": None}
    elif path in ("resolve", "chunks"):
        patch = {"status": "miss", "reason": why}
    elif path:
        patch = {"status": "unknown", "reason": f"path={path}"}
    elif "body patch" in phases:
        patch = {"status": "hit", "reason": None}
    elif any(k.startswith("resolve:") for k in phases):
        patch = {"status": "miss", "reason": None}
    else:
        patch = {"status": "unknown", "reason": None}
    if not visits:
        kept = {"status": "unknown", "reason": None}
    elif "kept" not in visits:
        kept = {"status": "not_exposed", "reason": None}
    elif visits["kept"] == "reuse":
        kept = {"status": "hit", "reason": None}
    elif visits["kept"] in ("-", ""):
        kept = {"status": "not_applicable", "reason": None}
    else:
        kept = {"status": "miss", "reason": visits["kept"]}
    return {"patch": patch, "kept": kept}


def count_patch_kept(requests):
    """Hit/miss counts and miss reasons over per-request records."""
    out = {"patch": {"hits": 0, "misses": 0, "unknown": 0, "miss_reasons": {}},
           "kept": {"hits": 0, "misses": 0, "unknown": 0, "not_exposed": 0,
                    "not_applicable": 0, "miss_reasons": {}}}
    for r in requests:
        for kind in ("patch", "kept"):
            c = r[kind]
            bucket = {"hit": "hits", "miss": "misses"}.get(c["status"], c["status"])
            if bucket in out[kind]:
                out[kind][bucket] += 1
            if c["status"] == "miss" and c["reason"]:
                out[kind]["miss_reasons"][c["reason"]] = out[kind]["miss_reasons"].get(c["reason"], 0) + 1
    return out


def phase_summaries(phases_list):
    """Per-phase stats over all requests of a row. A phase a request did not
    emit counts as zero in the overall values; `median_when_present` and
    `n_present` keep the fallback-only view visible and labeled."""
    names = []
    for ph in phases_list:
        for k in ph:
            if k not in names:
                names.append(k)
    out = {}
    for k in names:
        present = [ph[k] for ph in phases_list if k in ph]
        overall = summary([ph.get(k, 0.0) for ph in phases_list])
        out[k] = {"median_overall": overall["median"], "p95_overall": overall["p95"],
                  "worst_overall": overall["worst"],
                  "median_when_present": round(statistics.median(present), 2),
                  "n_present": len(present), "n": len(phases_list)}
    return out


def strip_noise(text):
    return "\n".join(l for l in text.splitlines()
                     if not TIMING_LINE.match(l) and not TIMING_HEAD.match(l)
                     and not l.startswith("lust-stats:") and not l.startswith("lust-visits:"))


class Bench:
    def __init__(self, slice_dir, opts):
        self.opts = opts
        self.slice_dir = Path(slice_dir).resolve()
        self.manifest = json.loads((self.slice_dir / "prune.json").read_text())
        self.edits_spec = json.loads((self.slice_dir / "edits.json").read_text())
        base = Path(os.environ.get("TMPDIR", "/tmp"))
        self.work = Path(tempfile.mkdtemp(prefix="lust-speed-", dir=base))
        self.crate = self.work / "crate"
        shutil.copytree(self.slice_dir / "src", self.crate / "src")
        root = self.manifest["spec"]["root"]
        self.root = self.crate / "src" / root
        # The shared driver gives the program output to compare (see driver.rs).
        driver = self.slice_dir.parent / "driver.rs"
        self.driver = driver.exists() and not opts["no_driver"]
        if self.driver:
            text = self.root.read_text()
            if text.count("fn main() {}") != 1:
                sys.exit(f"{self.root}: expected one `fn main() {{}}` to replace with the driver call")
            self.root.write_text(text.replace("fn main() {}", "mod bench_driver;\n\nfn main() {\n    bench_driver::run();\n}"))
            shutil.copy(driver, self.crate / "src" / "bench_driver.rs")
        self.runs_compared = 0
        self.dev_features = list(self.manifest.get("dev_features") or [])
        feats = ""
        if self.dev_features:
            names = ", ".join(f'"{f}"' for f in self.dev_features)
            feats = f"[features]\ndefault = [{names}]\n" + "".join(f"{f} = []\n" for f in self.dev_features) + "\n"
        (self.crate / "Cargo.toml").write_text(
            f'[package]\nname = "slice"\nversion = "0.0.0"\nedition = "2024"\n\n'
            f'[[bin]]\nname = "slice"\npath = "src/{root}"\n\n{feats}[profile.dev]\ndebug = 0\n\n[workspace]\n')
        # LUST_STATS on the daemon too: every warm answer then carries the
        # lust-visits: line the per-request records are built from.
        self.env = dict(os.environ, LUST_DAEMON_SOCKET=str(self.work / "s.sock"),
                        LUST_CACHE_DIR=str(self.work / "cache"), LUST_DAEMON_IDLE="3600s",
                        LUST_STATS="1")
        self.env.pop("LUST_NO_DAEMON", None)
        self.daemon = None
        self.rows = {}
        self.requests = []
        self.cold_n = 0
        self.states = 0
        self.mismatches = []
        self.rustc_cfgs = [] if opts["no_rustc"] else list(RUSTC_CONFIGS)

    # ---- process helpers

    def run(self, cmd, env=None, cwd=None):
        t = time.perf_counter()
        r = subprocess.run(cmd, cwd=cwd or self.crate, env=env or self.env, capture_output=True, text=True)
        return (time.perf_counter() - t) * 1000, r

    def lust(self, verb, extra_env=None, cold=False):
        env = dict(self.env, **(extra_env or {}))
        if cold:
            self.cold_n += 1
            env = dict(env, LUST_NO_DAEMON="1", LUST_CACHE_DIR=str(self.work / f"cold-{self.cold_n}"))
        cmd = [str(LUST), verb, "--timings", str(self.root)]
        if self.dev_features:
            cmd += ["--features", ",".join(self.dev_features)]
        if verb == "build":
            cmd += ["-o", str(self.work / "prog")]
        ms, r = self.run(cmd, env)
        if cold:
            shutil.rmtree(self.work / f"cold-{self.cold_n}", ignore_errors=True)
        return ms, r

    def cargo(self, cfg, verb):
        env = dict(os.environ, CARGO_TARGET_DIR=str(self.work / f"target-{cfg}"), CARGO_NET_OFFLINE="true",
                   **RUSTC_CONFIGS[cfg])
        env.pop("RUSTFLAGS", None) if cfg == "default" else None
        return self.run(["cargo", verb, "-q"], env)

    def cargo_rss_cold(self, cfg, verb):
        """Cold cargo run in a child Python so RUSAGE_CHILDREN covers only that tree."""
        code = ("import resource,subprocess,sys,os;"
                "r=subprocess.run(sys.argv[1:],capture_output=True);"
                "print(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss)")
        env = dict(os.environ, CARGO_TARGET_DIR=str(self.work / f"target-{cfg}-rss"), CARGO_NET_OFFLINE="true",
                   **RUSTC_CONFIGS[cfg])
        r = subprocess.run([sys.executable, "-c", code, "cargo", verb, "-q"], cwd=self.crate, env=env,
                           capture_output=True, text=True)
        shutil.rmtree(self.work / f"target-{cfg}-rss", ignore_errors=True)
        return int(r.stdout.strip() or 0)

    def start_daemon(self):
        self.daemon = subprocess.Popen([str(LUST), "daemon"], cwd=self.crate, env=self.env,
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        for _ in range(400):
            if (self.work / "s.sock").exists():
                return
            time.sleep(0.025)
        sys.exit("daemon did not start")

    def kill_daemon(self):
        if self.daemon:
            self.daemon.kill()
            self.daemon.wait()
            self.daemon = None
        for f in ("s.sock", "s.sock.lock"):
            (self.work / f).unlink(missing_ok=True)

    def daemon_hwm_kb(self):
        try:
            for line in open(f"/proc/{self.daemon.pid}/status"):
                if line.startswith("VmHWM:"):
                    return int(line.split()[1])
        except (OSError, AttributeError):
            return None

    # ---- recording

    def row(self, name):
        return self.rows.setdefault(name, {"lust": [], "phases": [], "stats": [], "requests": [],
                                            **{c: [] for c in self.rustc_cfgs}})

    def record_request(self, rec, row_name, verb, ms, r, edit_id=None, target=None, target_fn=None):
        """One per-request record: wall, every phase, visits counters, and the
        patch/kept classification. Appended to the row and to the run list."""
        phases = parse_timings(r.stderr)
        visits = parse_visits(r.stderr)
        req = {"seq": len(self.requests), "row": row_name, "verb": verb, "edit_id": edit_id,
               "target": target, "target_fn": target_fn, "wall_ms": round(ms, 2),
               "ok": r.returncode == 0, "phases_ms": phases, "visits": visits,
               "stats": parse_stats(r.stderr), **classify_request(visits, phases)}
        rec["requests"].append(req)
        self.requests.append(req)
        return req

    def target_label(self, target):
        p, ol, _fn = target
        return f"{p.relative_to(self.crate / 'src')}:{ol}"

    def measure_check(self, name, expect_ok=True, cold_compare=False, edit_id=None, target=None, target_fn=None):
        """Time a warm lust check and the cargo checks on the current state; check agreement."""
        self.states += 1
        ms, r = self.lust("check")
        rec = self.row(name)
        rec["lust"].append(ms)
        rec["phases"].append(parse_timings(r.stderr))
        self.record_request(rec, name, "check", ms, r, edit_id, target, target_fn)
        ok = r.returncode == 0
        if ok != expect_ok:
            self.fail(f"{name}: lust check exit {r.returncode}, expected {'ok' if expect_ok else 'error'}\n{r.stderr[-1500:]}")
        for cfg in self.rustc_cfgs:
            cms, cr = self.cargo(cfg, "check")
            rec[cfg].append(cms)
            if (cr.returncode == 0) != ok:
                self.fail(f"{name}: lust exit {r.returncode} but cargo check ({cfg}) exit {cr.returncode}\n{cr.stderr[-1500:]}")
        if cold_compare or (self.opts["cold_every"] and self.states % self.opts["cold_every"] == 0):
            _, c = self.lust("check", cold=True)
            if (c.returncode, c.stdout, strip_noise(c.stderr)) != (r.returncode, r.stdout, strip_noise(r.stderr)):
                self.fail(f"{name}: warm daemon answer differs from a cold one-shot check\n"
                          f"warm exit {r.returncode}: {strip_noise(r.stderr)[-800:]}\n"
                          f"cold exit {c.returncode}: {strip_noise(c.stderr)[-800:]}")
            rec.setdefault("cold_compares", 0)
            rec["cold_compares"] += 1
        return ms

    def measure_build(self, name, edit_id=None, target=None, target_fn=None):
        ms, r = self.lust("build", {"LUST_STATS": "1"})
        if r.returncode != 0:
            self.fail(f"{name}: lust build failed\n{r.stderr[-1500:]}")
        rec = self.row(name)
        rec["lust"].append(ms)
        rec["phases"].append(parse_timings(r.stderr))
        self.record_request(rec, name, "build", ms, r, edit_id, target, target_fn)
        st = parse_stats(r.stderr)
        st["link_ms"] = parse_timings(r.stderr).get("link", 0.0)
        rec["stats"].append(st)
        for cfg in self.rustc_cfgs:
            cms, cr = self.cargo(cfg, "build")
            rec[cfg].append(cms)
            if cr.returncode != 0:
                self.fail(f"{name}: cargo build ({cfg}) failed\n{cr.stderr[-1500:]}")
        self.run_programs(name, ms, {cfg: rec[cfg][-1] for cfg in self.rustc_cfgs})

    def run_programs(self, build_row, lust_build_ms, rustc_build_ms):
        """Run the lust and rustc binaries of the current state: outputs must match.
        Records the run row and a build+run row (this build plus its run)."""
        if not self.driver:
            return
        lms, lr = self.run([str(self.work / "prog")])
        run_rec = self.row("run (built program)")
        both = self.row(build_row.replace("build", "build+run", 1))
        run_rec["lust"].append(lms)
        both["lust"].append(lust_build_ms + lms)
        for cfg in self.rustc_cfgs:
            rms, rr = self.run([str(self.work / f"target-{cfg}/debug/slice")])
            if (lr.returncode, lr.stdout, lr.stderr) != (rr.returncode, rr.stdout, rr.stderr):
                self.fail(f"{build_row}: program output differs from rustc's ({cfg}): lust exit {lr.returncode}, "
                          f"rustc exit {rr.returncode}\nlust: {lr.stdout[-400:]}\nrustc: {rr.stdout[-400:]}")
            run_rec[cfg].append(rms)
            both[cfg].append(rustc_build_ms[cfg] + rms)
        self.runs_compared += 1

    def fail(self, msg):
        self.kill_daemon()
        print(f"FAIL {msg}", file=sys.stderr)
        print(f"work dir kept: {self.work}", file=sys.stderr)
        sys.exit(1)

    # ---- edits

    def body_targets(self):
        """Fns for body edits: one per file, largest files first, brace on its own line."""
        files = sorted((p for p in (self.crate / "src").rglob("*.rs") if p.name != "support.rs"),
                       key=lambda p: (-p.stat().st_size, str(p)))
        out = []
        for p in files:
            lines = p.read_text().split("\n")
            rows = subprocess.run([str(ITEMS), "--bodies", str(p)], capture_output=True, text=True).stdout.split("\n")
            for row in rows:
                if not row:
                    continue
                ol, oc, cl, cc, start = map(int, row.split())
                head = lines[start - 1]
                if cl - ol < 3 or lines[ol - 1][oc:].strip() or "const fn" in head:
                    continue
                m = re.search(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", " ".join(lines[start - 1:start + 2]))
                out.append((p, ol, m.group(1) if m else None))
                break
            if len(out) == self.opts["units"]:
                break
        return out

    def body_edit(self, target, value):
        """Insert or change the bench `let` on the line after one fn body's open brace."""
        p, ol, _fn = target
        lines = p.read_text().split("\n")
        indent = " " * (len(lines[ol - 1]) - len(lines[ol - 1].lstrip()) + 4)
        new = f"{indent}let {EDIT_TAG}: u32 = {value};"
        if EDIT_TAG in lines[ol]:
            lines[ol] = new
        else:
            lines.insert(ol, new)
        p.write_text("\n".join(lines))

    def text_edit(self, pairs):
        for rel, old, new in pairs:
            p = self.crate / "src" / rel
            t = p.read_text()
            if t.count(old) != 1:
                self.fail(f"edit anchor must occur once in {rel}: {old!r} (found {t.count(old)})")
            p.write_text(t.replace(old, new))

    # ---- the run

    def go(self):
        o = self.opts
        if not ITEMS.exists():
            subprocess.run(["cargo", "build", "--release", "-q", "--offline"], cwd=ITEMS.parent.parent.parent, check=True)
        lines = sum(1 for p in (self.crate / "src").rglob("*.rs") for _ in open(p))
        print(f"slice {self.slice_dir.name}: {lines} lines, ts-rust {self.manifest['tsrust_commit'][:9]}, work {self.work}", flush=True)
        rss = {}

        # Cold rows, interleaved per rep.
        for rep in range(o["reps"]):
            for verb in ("check", "build"):
                ms, r = self.lust(verb, {"LUST_STATS": "1"}, cold=True)
                if r.returncode != 0:
                    self.fail(f"cold {verb}: lust rejects the slice\n{r.stderr[-1500:]}")
                rec = self.row(f"cold {verb}")
                rec["lust"].append(ms)
                rec["phases"].append(parse_timings(r.stderr))
                self.record_request(rec, f"cold {verb}", verb, ms, r, edit_id=f"cold-{verb}-{rep + 1}")
                if verb == "build":
                    st = parse_stats(r.stderr)
                    st["link_ms"] = parse_timings(r.stderr).get("link", 0.0)
                    rec["stats"].append(st)
                for cfg in self.rustc_cfgs:
                    shutil.rmtree(self.work / f"target-{cfg}", ignore_errors=True)
                    cms, cr = self.cargo(cfg, verb)
                    if cr.returncode != 0:
                        self.fail(f"cold cargo {verb} ({cfg}) failed\n{cr.stderr[-1500:]}")
                    rec[cfg].append(cms)
                if verb == "build":
                    self.run_programs("cold build", ms, {cfg: rec[cfg][-1] for cfg in self.rustc_cfgs})
        # One-shot peak RSS (a child process tree each).
        rss["lust_cold_check_kb"] = self.cold_rss("check")
        for cfg in self.rustc_cfgs:
            rss[f"cargo_cold_check_kb_{cfg}"] = self.cargo_rss_cold(cfg, "check")
        # Prime both cargo configs for the warm rows (check and build artifacts).
        for cfg in self.rustc_cfgs:
            self.cargo(cfg, "check")
            self.cargo(cfg, "build")

        self.start_daemon()
        self.measure_check("daemon first check (new session, empty cache)", edit_id="daemon-first")
        for rep in range(o["reps"]):
            self.measure_check("no-op check (warm daemon)", edit_id=f"no-op-{rep + 1}")
        for rep in range(o["reps"]):
            self.kill_daemon()
            self.start_daemon()
            self.measure_check("cached restart check (new daemon, disk cache)", edit_id=f"restart-{rep + 1}")

        targets = self.body_targets()
        if len(targets) < 2:
            self.fail("fewer than 2 body edit targets")
        build_every = max(1, o["edits"] // o["build_samples"]) if o["build_samples"] else 0
        for i in range(o["edits"]):
            tgt = targets[i % len(targets)]
            self.body_edit(tgt, i + 1)
            self.measure_check("body edit check (one fn)", edit_id=f"body-{i + 1}",
                               target=self.target_label(tgt), target_fn=tgt[2])
            if build_every and i % build_every == 0:
                self.measure_build("body edit build (one fn)", edit_id=f"body-{i + 1}",
                                   target=self.target_label(tgt), target_fn=tgt[2])

        e = self.edits_spec
        for cyc in range(o["cycles"]):
            for kind in ("signature", "type", "impl"):
                spec = e[kind]
                pairs = [(x["file"], x["old"], x["new"]) for x in spec["edits"]]
                files = ",".join(sorted({f for f, _, _ in pairs}))
                self.text_edit(pairs)
                self.measure_check(f"{kind} edit check", cold_compare=True,
                                   edit_id=f"{kind}-{cyc + 1}-apply", target=files)
                self.text_edit([(f, n, old) for f, old, n in pairs])
                self.measure_check(f"{kind} edit check", cold_compare=True,
                                   edit_id=f"{kind}-{cyc + 1}-revert", target=files)
            self.body_edit(targets[0], '"not a u32"')
            self.measure_check("error edit check (both reject)", expect_ok=False, cold_compare=True,
                               edit_id=f"error-{cyc + 1}", target=self.target_label(targets[0]),
                               target_fn=targets[0][2])
            self.body_edit(targets[0], 7)
            self.measure_check("fix edit check", cold_compare=True, edit_id=f"fix-{cyc + 1}",
                               target=self.target_label(targets[0]), target_fn=targets[0][2])
        rss["lust_daemon_hwm_kb"] = self.daemon_hwm_kb()
        self.kill_daemon()
        return self.report(lines, rss, targets)

    def cold_rss(self, verb):
        code = ("import resource,subprocess,sys;subprocess.run(sys.argv[1:],capture_output=True);"
                "print(resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss)")
        env = dict(self.env, LUST_NO_DAEMON="1", LUST_CACHE_DIR=str(self.work / "rss-cache"))
        r = subprocess.run([sys.executable, "-c", code, str(LUST), verb, str(self.root)], env=env,
                           capture_output=True, text=True)
        shutil.rmtree(self.work / "rss-cache", ignore_errors=True)
        return int(r.stdout.strip() or 0)

    def report(self, lines, rss, targets):
        rows = []
        for name, rec in self.rows.items():
            lust = summary(rec["lust"])
            rus = {c: summary(rec[c]) for c in self.rustc_cfgs if rec[c]}
            best = min(rus.items(), key=lambda kv: kv[1]["median"]) if rus else None
            ph = phase_summaries(rec["phases"])
            stats = {}
            for st in rec["stats"]:
                for k in ("c_bytes", "units_compiled", "unit_cache_hits", "link_ms"):
                    if isinstance(st.get(k), (int, float)):
                        stats.setdefault(k, []).append(st[k])
            rows.append({
                "row": name, "lust": lust, "rustc": rus,
                "rustc_best": {"config": best[0], **best[1]} if best else None,
                "ratio": ratio(best[1]["median"], lust["median"]) if best and lust["median"] else None,
                "ratio_p95": ratio(best[1]["p95"], lust["p95"]) if best and lust["p95"] else None,
                # Overall medians: a phase a request did not emit counts as zero.
                "lust_phases_median_ms": {k: v["median_overall"] for k, v in ph.items()},
                "lust_phases_when_present_median_ms": {k: v["median_when_present"] for k, v in ph.items()},
                "lust_phases_present_n": {k: v["n_present"] for k, v in ph.items()},
                "lust_phases_ms": ph,
                "patch_kept": count_patch_kept(rec["requests"]),
                "build_stats_median": {k: statistics.median(v) for k, v in stats.items()},
                "cold_compares": rec.get("cold_compares", 0),
            })
        # A boat copy has no .git: the caller passes LUST_COMMIT.
        lust_commit = os.environ.get("LUST_COMMIT") or subprocess.run(
            ["git", "-C", str(ROOT), "describe", "--always", "--dirty"], capture_output=True, text=True).stdout.strip()
        rustc_v = subprocess.run(["rustc", "--version"], capture_output=True, text=True).stdout.strip()
        out = {
            "slice": self.slice_dir.name, "lines": lines, "code_lines": self.manifest.get("code_lines"),
            "tsrust_commit": self.manifest["tsrust_commit"], "lust_commit": lust_commit, "rustc": rustc_v,
            "host": {"nproc": os.cpu_count(), "load_at_end": os.getloadavg()[0]},
            "opts": self.opts, "states_checked": self.states, "program_runs_compared": self.runs_compared,
            "body_edit_targets": [self.target_label(t) for t in targets],
            "body_edit_target_fns": [t[2] for t in targets],
            "peak_rss": rss, "rows": rows, "requests": self.requests,
        }
        shutil.rmtree(self.work, ignore_errors=True)
        return out


def ratio(rustc_ms, lust_ms):
    """rustc / lust, 1 decimal at 1x and above, 2 below (0.04x must not print as 0)."""
    r = rustc_ms / lust_ms
    return round(r, 1) if r >= 1 else round(r, 2)


def patch_kept_line(pk):
    """One plain sentence of patch/kept counts, or '' for old result files."""
    if not pk:
        return ""

    def reasons(c):
        return " (" + ", ".join(f"{k} x{v}" for k, v in sorted(c["miss_reasons"].items())) + ")" \
            if c.get("miss_reasons") else ""

    p, k = pk["patch"], pk["kept"]
    line = f"body patch: {p['hits']} hit, {p['misses']} miss{reasons(p)}"
    if p["unknown"]:
        line += f", {p['unknown']} unknown"
    line += f". kept program: {k['hits']} hit, {k['misses']} miss{reasons(k)}"
    if k.get("not_exposed"):
        line += f", {k['not_exposed']} not exposed by this lust build (no kept= counter yet)"
    if k.get("not_applicable"):
        line += f", {k['not_applicable']} not applicable"
    if k.get("unknown"):
        line += f", {k['unknown']} unknown"
    return line + "."


def print_table(res):
    print(f"\n{res['slice']}: {res['lines']} lines ({res['code_lines']} code), ts-rust {res['tsrust_commit'][:9]}, "
          f"lust {res['lust_commit']}, {res['rustc']}, {res['host']['nproc']} cpus")
    print(f"{'row':<48} {'lust med':>9} {'p95':>8} {'worst':>8} {'n':>4} {'rustc med':>10} "
          f"{'rustc p95':>10} {'cfg':>9} {'ratio med':>10} {'ratio p95':>10}")
    for r in res["rows"]:
        l, b = r["lust"], r["rustc_best"]
        print(f"{r['row']:<48} {l['median']:>9.1f} {l['p95']:>8.1f} {l['worst']:>8.1f} {l['n']:>4} "
              f"{(b['median'] if b else float('nan')):>10.1f} {(b['p95'] if b else float('nan')):>10.1f} "
              f"{(b['config'] if b else '-'):>9} "
              f"{(str(r['ratio']) + 'x') if r['ratio'] else '-':>10} "
              f"{(str(r.get('ratio_p95')) + 'x') if r.get('ratio_p95') else '-':>10}")
        if r["row"].startswith("body edit check") and patch_kept_line(r.get("patch_kept")):
            print(f"  {patch_kept_line(r['patch_kept'])}")
    print(f"peak RSS: {res['peak_rss']}")
    print(f"states checked: {res['states_checked']} (lust and cargo agree on every one)")


def render_md(results):
    """The docs/speed.md tables for one or more result JSONs (one section per slice)."""
    out = []
    for res in results:
        rss = res["peak_rss"]
        out += [f"### `{res['slice']}`: {res['lines']:,} lines ({res['code_lines']:,} code)", "",
                f"ts-rust `{res['tsrust_commit'][:9]}`, lust `{res['lust_commit']}`, {res['rustc']}, "
                f"{res['host']['nproc']} vCPU. {res['states_checked']} source states; lust and cargo agree on "
                f"pass or fail on every one.", "",
                "| Row | lust median | p95 | worst | n | rustc median | rustc p95 (config) | Ratio median | Ratio p95 |",
                "|---|---:|---:|---:|---:|---:|---:|---:|---:|"]
        for r in res["rows"]:
            l, b = r["lust"], r["rustc_best"]
            rus = f"{b['median']:.0f} ms" if b else "-"
            rus95 = f"{b['p95']:.0f} ms ({b['config']})" if b else "-"
            x = f"{ratio(b['median'], l['median'])}x" if b and l["median"] else "-"
            x95 = r.get("ratio_p95")
            x95 = f"{x95}x" if x95 else (f"{ratio(b['p95'], l['p95'])}x" if b and l["p95"] else "-")
            if r["row"].startswith("body edit check"):
                x, x95 = f"**{x}**", f"**{x95}**"
            out.append(f"| {r['row']} | {l['median']:.0f} ms | {l['p95']:.0f} | {l['worst']:.0f} | {l['n']} | "
                       f"{rus} | {rus95} | {x} | {x95} |")
        build = next((r for r in res["rows"] if r["row"] == "cold build"), None)
        if build and build["build_stats_median"]:
            st = build["build_stats_median"]
            out += ["", f"Cold build: {st.get('c_bytes', 0) / 1e6:.1f} MB of C, {st.get('units_compiled', 0):.0f} units, "
                    f"link {st.get('link_ms', 0):.0f} ms."]
        out += [f"Peak RSS: lust cold check {rss.get('lust_cold_check_kb', 0) / 1024:.0f} MB, lust daemon "
                f"{(rss.get('lust_daemon_hwm_kb') or 0) / 1024:.0f} MB (VmHWM over the run), cargo cold check "
                + ", ".join(f"{rss[k] / 1024:.0f} MB ({k.rsplit('_', 1)[1]})" for k in rss if k.startswith("cargo_cold_check_kb_"))
                + " (largest process in the tree).", ""]
        body = next((r for r in res["rows"] if r["row"].startswith("body edit check")), None)
        if body:
            if patch_kept_line(body.get("patch_kept")):
                out += [f"Body edit check paths: {patch_kept_line(body['patch_kept'])}", ""]
            top = sorted(((k, v) for k, v in body["lust_phases_median_ms"].items() if k != "total"),
                         key=lambda kv: -kv[1])[:5]
            if "lust_phases_present_n" in body:
                out += ["lust phases, body edit check (median ms over all requests, absent = 0): "
                        + ", ".join(f"{k} {v:.0f}" for k, v in top) + ".", ""]
                n = body["lust"]["n"]
                partial = sorted(((k, body["lust_phases_when_present_median_ms"][k], c)
                                    for k, c in body["lust_phases_present_n"].items() if c < n),
                                  key=lambda t: -t[1])[:5]
                if partial:
                    out += ["lust phases when present only (fallback requests, median ms): "
                            + ", ".join(f"{k} {v:.0f} ({c}/{n} requests)" for k, v, c in partial) + ".", ""]
            else:  # old result file: its phase medians are when-present only
                out += ["lust phases, body edit check (median ms, when present; old result file): "
                        + ", ".join(f"{k} {v:.0f}" for k, v in top) + ".", ""]
    return "\n".join(out)


def write_md(path, results):
    text = Path(path).read_text()
    a, b = "<!-- speed-table:start -->", "<!-- speed-table:end -->"
    if a not in text or b not in text:
        sys.exit(f"{path} lacks the {a} / {b} markers")
    head, rest = text.split(a, 1)
    _, tail = rest.split(b, 1)
    Path(path).write_text(head + a + "\n" + render_md(results) + "\n" + b + tail)


def main(argv):
    if argv[:1] == ["--render"]:
        write_md(argv[1], [json.loads(Path(p).read_text()) for p in argv[2:]])
        return
    opts = {"edits": 120, "reps": 5, "cycles": 5, "units": 24, "build_samples": 12, "cold_every": 10,
            "no_rustc": False, "no_driver": False}
    slice_dir = None
    json_out = None
    args = list(argv)
    while args:
        a = args.pop(0)
        if a == "--slice":
            slice_dir = args.pop(0)
        elif a == "--json":
            json_out = args.pop(0)
        elif a == "--quick":
            opts.update(edits=20, reps=3, cycles=1, build_samples=2, cold_every=10)
        elif a == "--no-rustc":
            opts["no_rustc"] = True
        elif a == "--no-driver":
            opts["no_driver"] = True
        elif a.startswith("--") and a[2:].replace("-", "_") in opts:
            opts[a[2:].replace("-", "_")] = int(args.pop(0))
        else:
            sys.exit(f"unknown argument: {a}")
    if not slice_dir:
        sys.exit("usage: scale-gate.py --slice bench/slices/<name> [--quick] [--edits N] [--reps N] [--cycles N] "
                 "[--units N] [--build-samples N] [--cold-every N] [--no-rustc] [--json out.json]")
    if not LUST.exists():
        sys.exit(f"{LUST} not built; run cargo build --release first")
    res = Bench(slice_dir, opts).go()
    print_table(res)
    if json_out:
        Path(json_out).write_text(json.dumps(res, indent=1) + "\n")
        print(f"json: {json_out}")


if __name__ == "__main__":
    main(sys.argv[1:])
