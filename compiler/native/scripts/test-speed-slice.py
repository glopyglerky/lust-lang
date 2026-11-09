#!/usr/bin/env python3
# Unit tests for the honest-tail aggregation in scripts/speed_slice.py:
# absent phases count as zero, per-request patch/kept classification,
# and median+p95 ratios in the rendered table.
# Usage: python3 scripts/test-speed-slice.py   (exit 0 = pass)
import os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import speed_slice as ss

failures = []


def check(name, cond, detail=""):
    print(f"{'ok' if cond else 'FAIL'} {name}" + (f": {detail}" if detail and not cond else ""))
    if not cond:
        failures.append(name)


# A fallback-only phase present in 1 of 5 requests: overall median is 0,
# the when-present median keeps the fallback cost, and n_present says so.
phases = [{"collect": 10.0}, {"collect": 20.0}, {"collect": 30.0, "resolve: expand+parse": 276.0},
          {"collect": 40.0}, {"collect": 50.0}]
s = ss.phase_summaries(phases)
check("absent phase counts as zero", s["resolve: expand+parse"]["median_overall"] == 0.0, str(s))
check("when-present keeps fallback cost", s["resolve: expand+parse"]["median_when_present"] == 276.0)
check("when-present count", s["resolve: expand+parse"]["n_present"] == 1 and s["resolve: expand+parse"]["n"] == 5)
check("always-present phase unchanged", s["collect"]["median_overall"] == 30.0 and s["collect"]["n_present"] == 5)

# lust-visits parsing and patch/kept classification (W-INCR3 kept= names).
v = ss.parse_visits("lust-stats: mode=bodies\nlust-visits: path=patch why=- files=1 nodes=4 spans=9 "
                    "scanned=1 slot_fns=100 collect=500 discovery=2 checked=1 kept=reuse prog_spans=300\n")
check("parse visits", v.get("path") == "patch" and v.get("checked") == 1 and v.get("kept") == "reuse", str(v))
c = ss.classify_request(v, {"body patch": 3.0})
check("patch hit + kept hit", c["patch"]["status"] == "hit" and c["kept"]["status"] == "hit", str(c))

v2 = ss.parse_visits("lust-visits: path=resolve why=body_holds_item files=9 nodes=1 spans=0 scanned=0 "
                     "slot_fns=0 collect=500 discovery=2 checked=1 kept=discovery prog_spans=0\n")
c2 = ss.classify_request(v2, {"resolve: expand+parse": 276.0})
check("patch miss with reason", c2["patch"] == {"status": "miss", "reason": "body_holds_item"}, str(c2))
check("kept miss with reason", c2["kept"] == {"status": "miss", "reason": "discovery"}, str(c2))

# Current daemon (no kept= yet): the hook reports not_exposed, not a miss.
v3 = ss.parse_visits("lust-visits: path=patch why=- files=1 nodes=4 spans=9 scanned=1 slot_fns=100 "
                     "collect=500 discovery=2 checked=1\n")
c3 = ss.classify_request(v3, {})
check("kept not exposed hook", c3["kept"]["status"] == "not_exposed", str(c3))
check("no visits is unknown", ss.classify_request({}, {})["patch"]["status"] == "unknown")

# Row counting over per-request records.
reqs = [{"patch": {"status": "hit", "reason": None}, "kept": {"status": "hit", "reason": None}},
        {"patch": {"status": "miss", "reason": "body_holds_item"}, "kept": {"status": "miss", "reason": "fn_class"}},
        {"patch": {"status": "miss", "reason": "body_holds_item"}, "kept": {"status": "not_exposed", "reason": None}}]
cnt = ss.count_patch_kept(reqs)
check("patch counts", cnt["patch"] == {"hits": 1, "misses": 2, "unknown": 0,
                                        "miss_reasons": {"body_holds_item": 2}}, str(cnt))
check("kept counts", cnt["kept"]["hits"] == 1 and cnt["kept"]["misses"] == 1
      and cnt["kept"]["not_exposed"] == 1 and cnt["kept"]["miss_reasons"] == {"fn_class": 1}, str(cnt))

# Rendered docs show median and p95 ratios, and the patch counts line.
res = {"slice": "demo", "lines": 100, "code_lines": 50, "tsrust_commit": "a" * 40, "lust_commit": "deadbeef",
       "rustc": "rustc 1.99", "host": {"nproc": 16}, "states_checked": 3,
       "peak_rss": {"lust_cold_check_kb": 1024, "lust_daemon_hwm_kb": 2048},
       "rows": [{"row": "body edit check (one fn)",
                 "lust": {"n": 5, "median": 10.0, "p95": 100.0, "worst": 200.0},
                 "rustc_best": {"config": "default", "n": 5, "median": 1000.0, "p95": 2000.0, "worst": 3000.0},
                 "ratio": 100.0, "ratio_p95": 20.0, "build_stats_median": {},
                 "lust_phases_median_ms": {"total": 9.0, "collect": 5.0, "body patch": 3.0},
                 "lust_phases_when_present_median_ms": {"resolve: expand+parse": 276.0},
                 "lust_phases_present_n": {"resolve: expand+parse": 1},
                 "patch_kept": cnt}]}
md = ss.render_md([res])
check("md median+p95 ratio", "100" in md and "20" in md, md)
check("md patch line", "body patch: 1 hit" in md and "body_holds_item" in md, md)
check("md kept line", "kept program" in md, md)
check("md phase labels", "absent = 0" in md and "when present" in md, md)

print("FAILURES:", failures) if failures else print("all ok")
sys.exit(1 if failures else 0)
