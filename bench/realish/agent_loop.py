# The "real-ish work" benchmark: an agent's edit loop on a 31k-line crate (40 warehouse modules).
#
# For each edit: apply it, check, build, and run the program, with both compilers:
#   rustc 1.99, in its fastest incremental setup on this machine:
#       -Z threads=8 (parallel front end), -C incremental, -C debuginfo=0
#   lust, through a warm `lust daemon`.
# Both must agree on pass/fail, and on program output when they pass.
#
# Usage: python3 bench/realish/agent_loop.py [modules]
import os
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LUST = os.path.join(ROOT, "target/release/lust")
N = int(sys.argv[1]) if len(sys.argv) > 1 else 40
RUSTC = ["rustc", "--edition", "2024", "-C", "panic=abort", "-A", "warnings", "-Z", "threads=8"]
ENV = dict(os.environ, RUSTC_BOOTSTRAP="1")

# (module, description, old text, new text). Names get the module's `_m<k>` suffix via {m}.
EDITS = [
    (3, "tweak the bulk discount", "if qty >= 10 { subtotal / 20 } else { 0 }", "if qty >= 8 { subtotal / 25 } else { 0 }"),
    (11, "restock more often", "if round % 10 == 9 {", "if round % 8 == 7 {"),
    (19, "add a currency suffix", 'format!("{}${}.{}{}", sign, c / 100, pad, frac)', 'format!("{}${}.{}{} USD", sign, c / 100, pad, frac)'),
    (27, "lower the max quantity", "if line.qty <= 0 || line.qty > 50 {", "if line.qty <= 0 || line.qty > 40 {"),
    (5, "allow cancelling paid orders", "OrderState_m{m}::Paid => to == OrderState_m{m}::Shipped,",
     "OrderState_m{m}::Paid => to == OrderState_m{m}::Shipped || to == OrderState_m{m}::Cancelled,"),
    (33, "MISTAKE: wrong integer type", "let mut units = 0;", "let mut units: u32 = 0;"),
    (33, "fix the type error", "let mut units: u32 = 0;", "let mut units = 0;"),
    (14, "restock a bit more", "level.on_hand += level.reorder_qty;", "level.on_hand += level.reorder_qty + 2;"),
    (8, "rewrite a heap check", "if item.cost > dist[item.node] {", "if item.cost >= dist[item.node] + 1 {"),
    (21, "NEW FUNCTION: add a helper (signature change)", "fn validate_m{m}(order: &Order_m{m}) -> Option<OrderError_m{m}> {",
     "fn qty_ok_m{m}(q: i64) -> bool {\n    q > 0 && q <= 50\n}\n\nfn validate_m{m}(order: &Order_m{m}) -> Option<OrderError_m{m}> {"),
    (38, "MISTAKE: borrow conflict", "    out.push_str(s);\n    out\n}", "    let first = &out;\n    out.push_str(s);\n    let _ = first.len();\n    out\n}"),
    (38, "fix the borrow conflict", "    let first = &out;\n    out.push_str(s);\n    let _ = first.len();\n    out\n}", "    out.push_str(s);\n    out\n}"),
]


def run(cmd, cwd, env=None):
    start = time.perf_counter()
    r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True)
    return (time.perf_counter() - start) * 1000, r


def apply(path, module, old, new):
    src = open(path).read()
    marker = f"// ==================== module {module} ===================="
    start = src.index(marker)
    end = src.find("// ==================== module ", start + len(marker))
    end = len(src) if end < 0 else end
    section = src[start:end]
    old, new = old.replace("{m}", str(module)), new.replace("{m}", str(module))
    assert old in section, f"edit not found in module {module}: {old!r}"
    open(path, "w").write(src[:start] + section.replace(old, new, 1) + src[end:])


def main():
    work = tempfile.mkdtemp(prefix="lust-agent-loop-")
    crate = os.path.join(work, "crate.rs")
    with open(crate, "w") as f:
        subprocess.run([sys.executable, os.path.join(ROOT, "bench/realish/make.py"), str(N)], stdout=f, check=True)
    lines = sum(1 for _ in open(crate))
    daemon = subprocess.Popen([LUST, "daemon"], cwd=work, stderr=subprocess.DEVNULL)
    time.sleep(0.3)
    try:
        rc_check = RUSTC + ["--emit=metadata", "-C", f"incremental={work}/inc_check", "-o", f"{work}/x.rmeta", crate]
        rc_build = RUSTC + ["-C", "debuginfo=0", "-C", f"incremental={work}/inc_build", "-o", f"{work}/prog_rustc", crate]
        lu_check = [LUST, "check", crate]
        lu_build = [LUST, "build", crate, "-o", f"{work}/prog_lust"]

        print(f"crate: {N} modules, {lines} lines\n")
        rows = []
        runs = []
        cold = [run(rc_check, work, ENV)[0], run(rc_build, work, ENV)[0], run(lu_check, work)[0], run(lu_build, work)[0]]
        print(f"cold (fills caches): rustc check {cold[0]:.0f} ms, build {cold[1]:.0f} ms | lust check {cold[2]:.0f} ms, build {cold[3]:.0f} ms\n")
        print(f"{'#':>2}  {'edit':<46} {'rustc check':>11} {'lust check':>10} {'rustc build':>11} {'lust build':>10}  result")
        for i, (module, desc, old, new) in enumerate(EDITS, 1):
            apply(crate, module, old, new)
            rct, rcr = run(rc_check, work, ENV)
            lct, lcr = run(lu_check, work)
            ok_r, ok_l = rcr.returncode == 0, lcr.returncode == 0
            assert ok_r == ok_l, f"edit {i}: rustc ok={ok_r}, lust ok={ok_l}\n{lcr.stderr}"
            rbt = lbt = 0.0
            if ok_r:
                rbt, _ = run(rc_build, work, ENV)
                lbt, lbr = run(lu_build, work)
                assert lbr.returncode == 0, lbr.stderr
                rrt, ro = run([f"{work}/prog_rustc"], work)
                lrt, lo = run([f"{work}/prog_lust"], work)
                assert ro.stdout == lo.stdout, f"edit {i}: program output differs"
                runs.append((rrt, lrt))
                result = "same output"
            else:
                first = next((l for l in lcr.stderr.splitlines() if l.startswith("error[")), "?")
                result = "both reject; lust: " + first
            rows.append((rct, lct, rbt, lbt))
            build = f"{rbt:>9.0f}ms {lbt:>8.1f}ms" if ok_r else f"{'-':>11} {'-':>10}"
            print(f"{i:>2}  {desc:<46} {rct:>9.0f}ms {lct:>8.1f}ms {build}  {result}")

        tr = sum(r[0] + r[2] for r in rows)
        tl = sum(r[1] + r[3] for r in rows)
        cr, cl = sum(r[0] for r in rows), sum(r[1] for r in rows)
        br, bl = sum(r[2] for r in rows), sum(r[3] for r in rows)
        print(f"\ntotal check: rustc {cr / 1000:.2f} s, lust {cl / 1000:.3f} s  ({cr / cl:.0f}x)")
        print(f"total build: rustc {br / 1000:.2f} s, lust {bl / 1000:.3f} s  ({br / bl:.0f}x)")
        print(f"edit loop (check + build, {len(EDITS)} edits): rustc {tr / 1000:.2f} s, lust {tl / 1000:.3f} s  ({tr / tl:.0f}x)")
        rr, lr = sum(r[0] for r in runs), sum(r[1] for r in runs)
        print(f"running the program after each build ({len(runs)} runs): rustc build {rr:.0f} ms, lust build {lr:.0f} ms")
    finally:
        daemon.terminate()
        shutil.rmtree(work, ignore_errors=True)


main()
