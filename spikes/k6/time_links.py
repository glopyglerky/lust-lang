#!/usr/bin/env python3
"""K6: time the final link for each path on the ~200-export shim.

Usage: time_links.py <out_dir> <tcc_home> [rounds=5]
Interleaves variants round by round and prints the median wall ms.
"""
import pathlib
import statistics
import subprocess
import sys
import time

out = pathlib.Path(sys.argv[1]).resolve()
tcc_home = pathlib.Path(sys.argv[2])
rounds = int(sys.argv[3]) if len(sys.argv) > 3 else 5
tcc = str(tcc_home / "bin" / "tcc")
libtcc_path = str(tcc_home / "lib" / "tcc")
driver = str(out / "libtcc_driver")
gcc_dir = subprocess.check_output(["cc", "-print-file-name=libgcc_s.so"], text=True).strip()
gcc_dir = str(pathlib.Path(gcc_dir).parent)
big = out / "big"
main_big = str(big / "main_big.c")
main_big_o = str(big / "main_big.o")
staticlib = str(big / "libshim_big.a")
rpath = f"-Wl,-rpath={big}"


def variants():
    return {
        "tcc_cli_static": [tcc, main_big, staticlib, f"-L{gcc_dir}", "-lpthread", "-ldl", "-lm", "-lgcc_s"],
        "libtcc_static": [driver, libtcc_path, "exe", None, main_big_o, staticlib,
                          f"-l:{gcc_dir}:gcc_s", "-l:/usr/lib/x86_64-linux-gnu:pthread"],
        "tcc_cli_cdylib": [tcc, main_big, f"-L{big}", "-lshim_big", rpath, "-lpthread", "-ldl"],
        "libtcc_cdylib": [driver, libtcc_path, "exe", None, main_big_o, f"-l:{big}:shim_big", f"-o:{rpath}"],
        "cc_static": ["cc", main_big_o, staticlib, "-lpthread", "-ldl", "-lm"],
        "cc_cdylib": ["cc", main_big_o, f"-L{big}", "-lshim_big", rpath, "-lpthread", "-ldl"],
    }


def run_once(name, cmd, idx):
    exe = big / f"big_{name}_{idx}"
    cmd = [str(exe) if part is None else part for part in cmd]
    if name.startswith("tcc_cli") or name.startswith("cc_"):
        cmd = cmd + ["-o", str(exe)]
    t0 = time.perf_counter()
    r = subprocess.run(cmd, capture_output=True, text=True)
    dt = (time.perf_counter() - t0) * 1000
    if r.returncode != 0:
        print(f"LINKFAIL {name} round={idx}: {r.stderr.strip()[:300]}")
        return None, exe
    return dt, exe


times = {k: [] for k in variants()}
exes = {}
for idx in range(rounds):
    for name, cmd in variants().items():
        dt, exe = run_once(name, cmd, idx)
        exes[name] = exe
        if dt is not None:
            times[name].append(dt)

for name in variants():
    exe = exes[name]
    ok = "norun"
    if pathlib.Path(exe).exists():
        r = subprocess.run([str(exe)], capture_output=True, text=True)
        line = [ln for ln in r.stdout.splitlines() if ln.startswith("RESULT")]
        ok = f"{line[0] if line else 'no RESULT'} exit={r.returncode}"
        size = pathlib.Path(exe).stat().st_size
    else:
        size = -1
    med = statistics.median(times[name]) if times[name] else float("nan")
    print(f"LINKTIME {name} median_ms={med:.1f} runs={len(times[name])}/{rounds} exe_bytes={size} run=[{ok}]")
