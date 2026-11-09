#!/usr/bin/env python3
"""Which body edits miss the warm body patch, and why, on a slice.

Usage: bench/slices/patch_miss.py <slice dir> [--sample N] [--seed S] [--cold-every K] [--json out.json] [--lust BIN] [--only-rpit]

For a sample of fn bodies across the slice (every fn whose open brace ends
its line, const fns included), a warm daemon checks after one edit that
inserts a `let` after the brace, then after its undo. Each check's
`lust-visits:` line (LUST_VISITS_LOG) gives the path (patch, resolve),
why= (the first failed patch condition), and kept= (collected program
reuse). Prints counts per reason, wall median/p95/worst for hits and
misses, and a few example fns per reason. Every K-th edit also compares
the warm answer with a cold one-shot check (stdout, stderr, exit code).
A check that fails (every edit keeps the slice valid) counts as
path=fail, and the run goes on. --lust measures another lust binary.
--only-rpit samples only fns whose signature returns `impl Trait`.
"""
import json
import random
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent.parent / "scripts"))
import speed_slice as s  # noqa: E402


def all_bodies(b):
    out = []
    for p in sorted((b.crate / "src").rglob("*.rs")):
        if p.name in ("support.rs", "bench_driver.rs"):
            continue
        lines = p.read_text().split("\n")
        rows = subprocess.run([str(s.ITEMS), "--bodies", str(p)], capture_output=True, text=True).stdout.split("\n")
        for row in rows:
            if not row:
                continue
            ol, oc, cl, cc, start = map(int, row.split())
            if cl - ol < 2 or lines[ol - 1][oc:].strip():
                continue
            sig = " ".join(lines[start - 1:ol])
            out.append((p, ol, "const fn" in lines[start - 1], "-> impl " in sig))
    return out


def main():
    args = sys.argv[1:]
    opt = {"--sample": 300, "--seed": 1, "--cold-every": 0, "--json": None, "--lust": None, "--only-rpit": False}
    slice_dir = None
    while args:
        a = args.pop(0)
        if a == "--only-rpit":
            opt[a] = True
        elif a in opt:
            opt[a] = args.pop(0) if a in ("--json", "--lust") else int(args.pop(0))
        else:
            slice_dir = a
    if not s.ITEMS.exists():
        subprocess.run(["cargo", "build", "--release", "-q", "--offline"], cwd=s.ITEMS.parent.parent.parent, check=True)
    if opt["--lust"]:
        s.LUST = Path(opt["--lust"]).resolve()
    b = s.Bench(slice_dir, {"no_driver": False, "no_rustc": True})
    log = b.work / "visits.log"
    b.env["LUST_VISITS_LOG"] = str(log)
    b.start_daemon()
    for _ in range(2):
        _, r = b.lust("check")
        if r.returncode != 0:
            b.fail(f"slice does not check\n{r.stderr[-1500:]}")
    bodies = [x[:3] for x in all_bodies(b) if x[3] or not opt["--only-rpit"]]
    random.Random(opt["--seed"]).shuffle(bodies)
    bodies = bodies[: opt["--sample"]]
    rows = []
    n = 0
    for k, (p, ol, is_const) in enumerate(bodies):
        orig = p.read_text()
        for step in ("edit", "undo"):
            if step == "edit":
                b.body_edit((p, ol, None), k + 1)
            else:
                p.write_text(orig)
            mark = log.stat().st_size if log.exists() else 0
            ms, r = b.lust("check")
            line = log.read_text()[mark:].strip().splitlines()
            v = visits(line[0] if line else "")
            if r.returncode != 0:
                print(f"check fails after {step} at {p}:{ol}: {r.stderr[:300]}", flush=True)
                v["path"] = "fail"
            n += 1
            if opt["--cold-every"] and n % opt["--cold-every"] == 0:
                _, c = b.lust("check", cold=True)
                if (c.returncode, c.stdout, s.strip_noise(c.stderr)) != (r.returncode, r.stdout, s.strip_noise(r.stderr)):
                    b.fail(f"{p}:{ol} {step}: warm differs from cold")
            rows.append({"file": str(p.relative_to(b.crate / "src")), "line": ol, "const": is_const, "step": step, "ms": round(ms, 2),
                         "path": v.get("path", "?"), "why": v.get("why", ""), "kept": v.get("kept", "")})
    b.kill_daemon()
    report(rows)
    if opt["--json"]:
        Path(opt["--json"]).write_text(json.dumps(rows, indent=1))


def visits(line):
    out = {}
    for kv in line.split()[1:]:
        k, _, val = kv.partition("=")
        out[k] = val
    return out


def report(rows):
    ms = [r["ms"] for r in rows]
    if not ms:
        print("0 checks")
        return
    print(f"{len(rows)} checks: median {s.pct(ms, 50):.1f} ms, p95 {s.pct(ms, 95):.1f}, worst {max(ms):.1f}")
    groups = {}
    for r in rows:
        key = f"path={r['path']} why={r['why'] or '-'} kept={r['kept'] or '-'}"
        groups.setdefault(key, []).append(r)
    for key, rs in sorted(groups.items(), key=lambda kv: -len(kv[1])):
        t = [r["ms"] for r in rs]
        ex = ", ".join(f"{r['file']}:{r['line']}{' const' if r['const'] else ''}" for r in rs[:3])
        print(f"{len(rs):5d}  {key:60s} med {s.pct(t, 50):7.1f}  p95 {s.pct(t, 95):7.1f}  worst {max(t):7.1f}  e.g. {ex}")


if __name__ == "__main__":
    main()
