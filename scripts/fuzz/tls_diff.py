#!/usr/bin/env python3
# Differential fuzz for thread_local! (W-TLS): random programs with lazy and
# const keys over Cell, RefCell, Vec, and a Drop type whose destructor
# touches other keys (so destruction order and AccessError show in the
# output), run with rustc and with lust (both with their default panic=unwind). Stdout and exit
# status must match. Rustc is the oracle; a program rustc rejects is
# skipped.
# Usage: python3 scripts/fuzz/tls_diff.py [first_seed] [count]   (exit 0 = no mismatch)
import os, random, subprocess, sys, tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LUST = os.path.join(ROOT, "target/release/lust")

HEADER = """use std::cell::{Cell, RefCell};
struct D(u32);
impl Drop for D {
    fn drop(&mut self) {
        CNT.set(CNT.get() + 1);
        match LOG.try_with(|l| l.borrow_mut().push(self.0)) {
            Ok(()) => println!("drop D({}) #{}", self.0, CNT.get()),
            Err(_) => println!("drop D({}) #{} log gone", self.0, CNT.get()),
        }
    }
}
impl Default for D {
    fn default() -> D {
        D(0)
    }
}
"""

# Per value kind: the type, the initializer, and the method calls to
# draw from (`{k}` is the key, `{x}` a random number).
KINDS = {
    "cell": ("Cell<u32>", "Cell::new({v})", [
        "{k}.set({x});", 'println!("get {{}}", {k}.get());', 'println!("take {{}}", {k}.take());',
        'println!("replace {{}}", {k}.replace({x}));', "{k}.with(|c| c.set(c.get() + {x}));",
        'println!("with get {{}}", {k}.with(Cell::get));',
    ]),
    "refcell_d": ("RefCell<D>", "RefCell::new(D({v}))", [
        "{k}.set(D({x}));", 'println!("take {{}}", {k}.take().0);', 'println!("replace {{}}", {k}.replace(D({x})).0);',
        '{k}.with_borrow(|d| println!("see {{}}", d.0));', "{k}.with_borrow_mut(|d| d.0 += {x});",
    ]),
    "refcell_vec": ("RefCell<Vec<u32>>", "RefCell::new(Vec::new())", [
        "{k}.with_borrow_mut(|v| v.push({x}));", '{k}.with_borrow(|v| println!("len {{}}", v.len()));',
        'println!("take {{:?}}", {k}.take());', "{k}.set(vec![{x}, {x}]);", 'println!("replace {{:?}}", {k}.replace(vec![{x}]));',
    ]),
    "plain_d": ("D", "D({v})", [
        '{k}.with(|d| println!("see {{}}", d.0));',
        'match {k}.try_with(|d| d.0) {{ Ok(v) => println!("ok {{}}", v), Err(_) => println!("err") }}',
    ]),
}


def program(seed):
    r = random.Random(seed)
    lines = [HEADER, "thread_local! {", "    static CNT: Cell<u32> = const { Cell::new(0) };",
             "    static LOG: RefCell<Vec<u32>> = RefCell::new(Vec::new());"]
    keys = []
    for i in range(r.randint(1, 4)):
        kind = r.choice(sorted(KINDS))
        ty, init, _ = KINDS[kind]
        init = init.format(v=r.randint(1, 50))
        if r.random() < 0.5:
            lines.append(f"    static K{i}: {ty} = const {{ {init} }};")
        else:
            lines.append(f'    static K{i}: {ty} = {{ println!("init K{i}"); {init} }};')
        keys.append((f"K{i}", kind))
    lines += ["}", "fn main() {"]
    for _ in range(r.randint(1, 10)):
        k, kind = r.choice(keys)
        lines.append("    " + r.choice(KINDS[kind][2]).format(k=k, x=r.randint(1, 99)))
    if r.random() < 0.5:
        lines.append('    LOG.with_borrow(|l| println!("log {:?}", l));')
    lines += ['    println!("end");', "}"]
    return "\n".join(lines) + "\n"


def run(cmd, timeout=120):
    try:
        p = subprocess.run(cmd, capture_output=True, timeout=timeout)
        return p.returncode, p.stdout
    except subprocess.TimeoutExpired:
        return None, b""


def main():
    first = int(sys.argv[1]) if len(sys.argv) > 1 else 1
    count = int(sys.argv[2]) if len(sys.argv) > 2 else 100
    work = tempfile.mkdtemp(prefix="lust-tls-diff-")
    env = dict(os.environ, LUST_NO_DAEMON="1")
    bad = skipped = 0
    for seed in range(first, first + count):
        src = os.path.join(work, f"p{seed}.rs")
        with open(src, "w") as f:
            f.write(program(seed))
        exe = os.path.join(work, f"p{seed}")
        if run(["rustc", "--edition", "2024", "-o", exe, src])[0] != 0:
            skipped += 1
            continue
        want = run([exe])
        p = subprocess.run([LUST, "run", src], capture_output=True, timeout=120, env=env)
        if (p.returncode, p.stdout) != want:
            bad += 1
            print(f"MISMATCH seed {seed}: lust exit {p.returncode}, rustc exit {want[0]} ({src})")
    print(f"tls_diff: seeds {first}..{first + count - 1}, {bad} mismatches, {skipped} skipped (rustc rejects)")
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
