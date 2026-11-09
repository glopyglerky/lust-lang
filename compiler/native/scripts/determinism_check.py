#!/usr/bin/env python3
# Determinism gate, one-shot half (the image half is front_check.py --chains): a build's output
# must not depend on thread timing. Each program of a fixed corpus builds as fresh one-shot
# processes (no image, no cache: LUST_NO_CACHE=1, fresh numbering), each lane three times:
#   release C (--release --target host, LUST_RELEASE_C=1): LUST_THREADS=1, N, and N again
#   release LLVM (with --llvm, LUST_RELEASE_BACKEND=llvm): the same three
#   dev C (plain build): N and N again (dev C splits its units by the thread count, so its runs
#   compare at one count)
# The thread count is read once per process (pool.rs), so each run is its own process. Each run
# writes its C units (--emit-c): the complete unit set (names and bytes), the binary bytes, the
# exit status, and stderr (all diagnostics, in order) must be equal across a lane's runs, for
# rejected builds too. Builds run many at a time, so threads also contend for cores: that varies
# the timing further.
#
# Why: mono and codegen make instances, Adt rows and types in a turn order that some outputs read
# (the first Adt instance registered under region-free args names the shared C type,
# `adt_c_keys`). Work moved onto the pool must keep every such order; front-check compares warm
# with cold builds, this compares cold with cold across thread counts.
#
# Corpus: front-check's program states (scripts/front_check.py `corpus`), the four fuzz crates
# (scripts/fuzz/fuzz.py: small, small2, small3, realish), and a fixed sample of tests/pass
# programs that use generics, closures, or dyn (every program whose name hashes into the
# sample; --sample N, default 48).
#
# --order-fuzz SEED adds the order audit: a release C build with LUST_ORDER_FUZZ=SEED (each
# materialize round takes its bodies in a shuffled order, so instances, Adt rows and types are
# made in another order) against the plain one. A difference there is an output that reads
# creation order: it is reported (`order:` lines, counted apart) and does not fail the run.
#
# LUST_BIN overrides the lust binary (default target/release/lust).
# Usage: determinism_check.py [--jobs J] [--threads N] [--sample N] [--llvm] [--order-fuzz SEED] [--keep DIR]
# Exit 0 = all equal, 1 = a difference, 2 = setup error.
import argparse, hashlib, os, re, shutil, subprocess, sys, tempfile, time
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "scripts"))
sys.path.insert(0, os.path.join(ROOT, "scripts", "fuzz"))
LUST = os.environ.get("LUST_BIN") or os.path.join(ROOT, "target/release/lust")
TIMEOUT = 300
GENERIC = re.compile(r"impl\s*<|fn\s+\w+\s*<|\bdyn\s|\|[^|]*\|\s*[{\w]|\bimpl\s+(Fn|Iterator)")


def corpus(sample):
    """(name, source) of every program, deduplicated by content."""
    import front_check
    import fuzz
    out, seen = [], set()

    def add(name, src):
        h = hashlib.sha256(src.encode()).hexdigest()
        if h not in seen:
            seen.add(h)
            out.append((name, src))

    for seq in front_check.corpus():
        name, srcs = seq[0], seq[1]
        for k, s in enumerate(srcs):
            src = s[1] if isinstance(s, tuple) else s
            # A step of several files (a dict: front_check's sources-late) is no one-file program.
            if isinstance(src, str):
                add(f"fc-{name}-{k}", src)
    for kind in ("small", "small2", "small3", "realish"):
        add(f"fuzz-{kind}", fuzz.crate_text(kind, 40))
    pass_dir = os.path.join(ROOT, "tests/pass")
    picks = []
    for f in sorted(os.listdir(pass_dir)):
        if not f.endswith(".rs"):
            continue
        src = open(os.path.join(pass_dir, f), errors="replace").read()
        if "fn main" not in src or not GENERIC.search(src):
            continue
        picks.append((hashlib.sha256(f.encode()).hexdigest(), f, src))
    for _, f, src in sorted(picks)[:sample]:
        add(f"pass-{f[:-3]}", src)
    return out


def build(d, args, env):
    """(status, stderr, units by name, binary bytes) of one build in `d`."""
    out_dir = d + ".units"
    shutil.rmtree(out_dir, ignore_errors=True)
    os.makedirs(out_dir)
    try:
        r = subprocess.run([LUST, "build", "main.rs", *args, "-o", d + ".bin", "--emit-c", os.path.join(out_dir, "t")],
                           cwd=os.path.dirname(d), env=env, capture_output=True, timeout=TIMEOUT)
    except subprocess.TimeoutExpired:
        return ("hang", "", {}, b"")
    err = r.stderr.decode(errors="replace").replace(d, "OUT")
    units = {f: open(os.path.join(out_dir, f), "rb").read() for f in sorted(os.listdir(out_dir))}
    binary = open(d + ".bin", "rb").read() if os.path.exists(d + ".bin") else b""
    return (r.returncode, err, units, binary)


def diff(a, b):
    """'' when two builds are equal, else what differs first."""
    (sa, ea, ua, ba), (sb, eb, ub, bb) = a, b
    if sa != sb:
        return f"status {sa} vs {sb}"
    if ea != eb:
        return "stderr differs"
    if ua.keys() != ub.keys():
        return f"unit names differ: {sorted(set(ua) ^ set(ub))[:4]}"
    for f in ua:
        if ua[f] != ub[f]:
            x, y = ua[f].split(b"\n"), ub[f].split(b"\n")
            n = next((i for i in range(min(len(x), len(y))) if x[i] != y[i]), min(len(x), len(y)))
            return f"{f} line {n + 1}: {x[n][:120] if n < len(x) else b''!r} vs {y[n][:120] if n < len(y) else b''!r}"
    if ba != bb:
        return "binary differs (units equal)"
    return ""


def check(item, args):
    name, src = item
    d = os.path.join(args.work, name)
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, "main.rs"), "w") as f:
        f.write(src)
    drop = ("LUST_DAEMON_SOCKET", "LUST_FRONT_STATE", "LUST_CACHE_DIR", "LUST_THREADS", "LUST_RELEASE_BACKEND", "LUST_RELEASE_C")
    base = {k: v for k, v in os.environ.items() if k not in drop}
    base.update(LUST_NO_CACHE="1", LUST_NO_DAEMON="1")
    lanes = [("dev", [], {}), ("release-c", ["--release", "--target", args.host], {"LUST_RELEASE_C": "1"})]
    if args.llvm:
        lanes.append(("release-llvm", ["--release", "--target", args.host], {"LUST_RELEASE_C": "1", "LUST_RELEASE_BACKEND": "llvm"}))
    problems = []
    for lane, flags, extra in lanes:
        n = str(args.threads)
        counts = ["1", n, n] if lane.startswith("release") else [n, n]
        runs = [build(os.path.join(d, f"{lane}-{k}"), flags, dict(base, LUST_THREADS=c, **extra)) for k, c in enumerate(counts)]
        for k in range(1, len(runs)):
            why = diff(runs[k - 1], runs[k])
            if why:
                problems.append(f"{name} {lane}: run {k} (threads {counts[k - 1]}) vs run {k + 1} (threads {counts[k]}): {why}")
        many = runs[-1]
        if args.order_fuzz is not None:
            fuzzed = build(os.path.join(d, f"{lane}-o"), flags, dict(base, LUST_THREADS=str(args.threads), LUST_ORDER_FUZZ=str(args.order_fuzz), **extra))
            why = diff(many, fuzzed)
            if why and lane.startswith("release"):
                problems.append(f"order: {name} {lane}: plain vs LUST_ORDER_FUZZ={args.order_fuzz}: {why}")
            elif why:
                # Dev C names expose creation numbering: a change here shows the shuffle ran.
                problems.append(f"shuffled: {name}")
    if problems and args.keep:
        shutil.copytree(d, os.path.join(args.keep, name), dirs_exist_ok=True)
    shutil.rmtree(d, ignore_errors=True)
    return problems


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--jobs", type=int, default=os.cpu_count() or 4)
    ap.add_argument("--threads", type=int, default=max(8, os.cpu_count() or 8))
    ap.add_argument("--sample", type=int, default=48)
    ap.add_argument("--llvm", action="store_true")
    ap.add_argument("--order-fuzz", type=int, default=None, metavar="SEED")
    ap.add_argument("--keep", help="copy each differing program's directory here")
    args = ap.parse_args()
    if not os.access(LUST, os.X_OK):
        print(f"error: {LUST} not built. Run scripts/build.sh first.")
        return 2
    v = subprocess.run(["rustc", "-vV"], capture_output=True, text=True).stdout
    args.host = next((l.split()[1] for l in v.splitlines() if l.startswith("host:")), "x86_64-unknown-linux-gnu")
    items = corpus(args.sample)
    args.work = tempfile.mkdtemp(prefix="determinism.", dir=os.environ.get("TMPDIR"))
    t0 = time.time()
    bad = order = shuffled = 0
    with ThreadPoolExecutor(max(1, args.jobs)) as pool:
        for problems in pool.map(lambda it: check(it, args), items):
            for p in problems:
                if p.startswith("shuffled: "):
                    shuffled += 1
                elif p.startswith("order: "):
                    order += 1
                    print(f"determinism: {p}"[:600])
                else:
                    bad += 1
                    print(f"determinism: DIFFERENT {p}"[:600])
    shutil.rmtree(args.work, ignore_errors=True)
    lanes = 3 if args.llvm else 2
    audit = (f", order audit: {order} programs read creation order in release units; the shuffle changed the dev C of {shuffled}"
             if args.order_fuzz is not None else "")
    print(f"determinism: {len(items)} programs x {lanes} lanes, {bad} different (threads 1, {args.threads}, {args.threads}{audit}, {time.time() - t0:.0f} s)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
