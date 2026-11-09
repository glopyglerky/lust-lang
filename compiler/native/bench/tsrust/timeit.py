#!/usr/bin/env python3
"""Time a command N times. Appends one JSON line per run to a results file.

This is /tmp/lust-ts/baseline/timeit.py with the results path as an argument
instead of a hard-coded path.

Usage: timeit.py <results.jsonl> <label> <runs> <logfile> -- <cmd...>
Env for the child comes from the calling shell. Load is read before each run.
"""
import json
import os
import resource
import subprocess
import sys
import time

out_path, label, runs, logfile = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
assert sys.argv[5] == "--"
cmd = sys.argv[6:]
walls = []
for i in range(runs):
    try:
        load = open("/proc/loadavg").read().split()[:3]
    except OSError:
        load = ["?", "?", "?"]
    before = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    t0 = time.perf_counter()
    with open(logfile, "ab") as log:
        log.write(f"\n===== {label} run {i + 1}: {' '.join(cmd)} =====\n".encode())
        log.flush()
        p = subprocess.run(cmd, stdout=log, stderr=subprocess.STDOUT)
    wall = time.perf_counter() - t0
    after = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
    rec = {
        "label": label, "run": i + 1, "wall_s": round(wall, 3),
        "exit": p.returncode, "load": load,
        "child_maxrss_kib": max(before, after),
        "cmd": " ".join(cmd),
        "rustflags": os.environ.get("RUSTFLAGS", ""),
        "rustup_toolchain": os.environ.get("RUSTUP_TOOLCHAIN", "(default)"),
        "target_dir": os.environ.get("CARGO_TARGET_DIR", "(default)"),
        "t": time.strftime("%Y-%m-%d %H:%M:%S"),
    }
    with open(out_path, "a") as f:
        f.write(json.dumps(rec) + "\n")
    walls.append(wall)
    print(f"{label} run {i + 1}: {wall:.2f}s exit={p.returncode} load={load[0]}")
walls_sorted = sorted(walls)
med = walls_sorted[len(walls) // 2]
print(f"{label} MEDIAN {med:.2f}s  runs={[round(w, 2) for w in walls]}")
