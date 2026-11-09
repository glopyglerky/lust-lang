#!/usr/bin/env python3
# Delimiter corruption differential (fx-attrdelim). rustc's lexer rejects any
# unbalanced or mismatched `()`, `[]`, `{}` anywhere in a file: in attributes
# (derive lists, cfg_attr, doc), macro calls, macro_rules bodies, cfg-off items,
# items and bodies. A delimiter inside a string, char literal or comment does
# not count. This sweep takes valid seed programs (the SEEDS below plus small
# tests/pass files), finds each delimiter outside literals and comments, and
# makes mutants: drop it, swap its kind, add a stray closer or opener after it.
# It also puts delimiters inside literals and comments, where rustc still
# accepts. Each mutant runs on rustc 1.99, a cold lust check, and a warm daemon
# check (the daemon first sees the seed, then the mutant as an edit).
#
# Classes: false accept (lust accepts, rustc rejects), new false reject (lust
# rejects a mutant rustc accepts, while lust accepts the seed), warm != cold,
# hang (a lust timeout; never a reject). Exit 1 on any of them.
#
# Usage: python3 scripts/fuzz/delim_sweep.py [--seed S] [--per-seed N] [--files K] [--jobs J] [--keep DIR] [--no-warm]
# Env: LUST_BIN (default target/release/lust).
import argparse, os, random, re, shutil, signal, subprocess, sys, tempfile, time
from concurrent.futures import ProcessPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LUST = os.environ.get("LUST_BIN", os.path.join(ROOT, "target/release/lust"))
TIMEOUT = 60
RUSTC = ["rustc", "--edition", "2024", "--crate-type", "bin", "--emit=metadata", "-A", "warnings"]

# Hand seeds: one of each place a token tree is read, plus literals and
# comments that hold delimiters which must not count.
SEEDS = [
    r'''#[derive(Debug, Clone, PartialEq)]
struct Qq {
    a: i64,
    b: (i64, i64),
}

#[cfg_attr(any(), derive(Copy))]
#[allow(dead_code)]
enum Shape {
    Dot,
    Pair(i64, i64),
}

fn main() {
    let q = Qq { a: 1, b: (2, 3) };
    let v = vec![q.a, q.b.0, q.b.1];
    println!("{:?} {}", q, v.len());
}
''',
    r'''/// Docs with ( and ] in them.
#[doc = "more ( docs } here"]
#[doc(alias = "pt")]
pub struct Pt(i64);

#[cfg(any())]
fn off() {
    let x = [1, 2, 3];
    let _ = (x, { 1 });
}

macro_rules! unused {
    ($a:expr) => { [$a, ($a)] };
    () => {{ 0 }};
}

macro_rules! twice {
    ($e:expr) => { ($e) * 2 };
}

fn main() {
    let p = Pt(4);
    assert_eq!(twice!(p.0), 8, "msg with ) in it");
    println!("{}", twice!(1));
}
''',
    r'''// A comment with ( [ { and } ] ) in it.
/* block ( comment /* nested ] */ still } */
const S: &str = "str ( with [ delims {";
const R: &str = r#"raw ) " ] } "#;
const B: &[u8] = b"bytes ( ]";
const C: char = '(';
const D: char = ']';
const E: char = '}';

#[inline]
fn pick<'a>(s: &'a str, i: usize) -> &'a str {
    &s[i..]
}

fn main() {
    let t = (S, R, B.len(), C, D, E);
    let m = match t.3 { '(' => 1, ')' => 2, _ => 3 };
    if m == 1 { println!("{} {:?} {}", pick(t.0, 1), t, format!("{{}}{}", m)); }
}
''',
    r'''#![allow(unused)]

mod inner {
    #[derive(Default, Debug)]
    pub struct W<T> {
        pub v: Vec<T>,
    }

    impl<T: Clone> W<T> {
        #[must_use]
        pub fn first(&self) -> Option<T> {
            self.v.first().cloned()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        assert!(matches!(Some(1), Some(_)));
    }
}

fn main() {
    let w = inner::W { v: vec![(1, [2, 3])] };
    let f = w.first();
    let arr: [[i32; 2]; 2] = [[1, 2], [3, 4]];
    println!("{:?} {}", f, arr[1][0]);
}
''',
]

OPENS, CLOSES = "([{", ")]}"


def scan(src):
    """Delimiters outside literals and comments, each as (offset, char, place),
    and the literal and comment spans as (lo, hi, kind). `place` is attr,
    macro, item (a top-level group) or body. A rough Rust scanner: enough for
    the seeds, which are valid Rust."""
    delims, spans, stack = [], [], []
    i, n = 0, len(src)
    while i < n:
        c = src[i]
        if src.startswith("//", i):
            j = src.find("\n", i)
            j = n if j < 0 else j
            spans.append((i, j, "line comment"))
            i = j
            continue
        if src.startswith("/*", i):
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif src.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            spans.append((i, j, "block comment"))
            i = j
            continue
        m = re.match(r'(?:b|c)?r(#*)"', src[i:])
        if m and not (i and (src[i - 1].isalnum() or src[i - 1] == "_")):
            end = src.find('"' + m.group(1), i + m.end())
            spans.append((i, end + 1 + len(m.group(1)), "raw string"))
            i = end + 1 + len(m.group(1))
            continue
        if c == '"' or (c in "bc" and src[i + 1 : i + 2] == '"' and not (i and (src[i - 1].isalnum() or src[i - 1] == "_"))):
            j = i + (1 if c == '"' else 2)
            while src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            spans.append((i, j + 1, "string"))
            i = j + 1
            continue
        if c == "'":
            m = re.match(r"'(\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.)|[^\\'\n])'", src[i:])
            if m:
                spans.append((i, i + m.end(), "char"))
                i += m.end()
                continue
        if c in OPENS:
            before = src[:i].rstrip()
            parent = stack[-1] if stack else None
            if before.endswith("#") or before.endswith("#!"):
                place = "attr"
            elif before.endswith("!") and not before.endswith("!=") and re.search(r"[A-Za-z_0-9]\s*!$", before):
                place = "macro"
            elif parent in ("attr", "macro"):
                place = parent
            else:
                place = "item" if not stack else "body"
            stack.append(place)
            delims.append((i, c, place))
        elif c in CLOSES:
            place = stack.pop() if stack else "item"
            delims.append((i, c, place))
        i += 1
    return delims, spans


def mutants(src, rng, k):
    """Up to k (op, place, text) mutants of src."""
    delims, spans = scan(src)
    out = []
    for off, ch, place in delims:
        others = (CLOSES if ch in CLOSES else OPENS).replace(ch, "")
        out.append(("drop", place, src[:off] + src[off + 1 :]))
        for o in others:
            out.append(("swap", place, src[:off] + o + src[off + 1 :]))
        out.append(("add closer", place, src[: off + 1] + rng.choice(CLOSES) + src[off + 1 :]))
        out.append(("add opener", place, src[: off + 1] + rng.choice(OPENS) + src[off + 1 :]))
    for lo, hi, kind in spans:
        d = rng.choice(OPENS + CLOSES)
        if kind == "char":
            out.append(("delim in literal", kind, src[:lo] + "'" + d + "'" + src[hi:]))
        elif kind == "line comment":
            out.append(("delim in comment", kind, src[:hi] + " " + d + src[hi:]))
        elif kind == "block comment":
            out.append(("delim in comment", kind, src[: hi - 2] + d + src[hi - 2 :]))
        else:
            q = src.index('"', lo) + 1
            out.append(("delim in literal", kind, src[:q] + d + src[q:]))
    # New comments and literals holding delimiters, at line ends and item starts.
    lines = [m.end() for m in re.finditer(r"\n", src)]
    for _ in range(6):
        at = rng.choice(lines)
        d = "".join(rng.choice(OPENS + CLOSES) for _ in range(rng.randint(1, 3)))
        what = rng.choice([f"// {d}\n", f"/* {d} */\n", f"/// {d}\nconst _: &str = \"{d}\";\n", f"const _: char = '{d[0]}';\n"])
        out.append(("delim in new " + ("comment" if what.startswith("/") and "const" not in what else "literal"), "line", src[:at] + what + src[at:]))
    rng.shuffle(out)
    return out[:k]


def run(cmd, cwd, env):
    try:
        r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=TIMEOUT)
        return r.returncode, r.stderr
    except subprocess.TimeoutExpired:
        return None, "timeout"


def case(args):
    idx, src, seed, k, warm = args
    rng = random.Random(seed * 1000003 + idx)
    work = tempfile.mkdtemp(prefix=f"lust-delim-{idx}-")
    sock = os.path.join(work, "d.sock")
    env = dict(os.environ, LUST_DAEMON_SOCKET=sock, LUST_NO_CACHE="1")
    env.pop("LUST_NO_DAEMON", None)
    cold_env = dict(env, LUST_NO_DAEMON="1")
    main = os.path.join(work, "main.rs")
    res = {"mutants": 0, "fa": [], "fr": [], "wc": [], "hang": [], "rustc_rejects": 0, "by": {}}
    daemon = None
    try:
        with open(main, "w") as f:
            f.write(src)
        base = run([LUST, "check", main], work, cold_env)
        if base[0] != 0:
            return idx, res  # lust rejects the seed: not a delimiter question
        if warm:
            daemon = subprocess.Popen([LUST, "daemon"], cwd=work, env=env, stderr=subprocess.DEVNULL, start_new_session=True)
            t0 = time.time()
            while not os.path.exists(sock):
                if time.time() - t0 > 10 or daemon.poll() is not None:
                    res["hang"].append(f"seed file {idx}: daemon start failed")
                    return idx, res
                time.sleep(0.005)
        mtime = int(time.time())
        for j, (op, place, text) in enumerate(mutants(src, rng, k)):
            tag = f"file {idx} m{j} [{op} @ {place}]"
            res["mutants"] += 1
            if warm:
                mtime += 2
                with open(main, "w") as f:
                    f.write(src)
                os.utime(main, (mtime, mtime))
                run([LUST, "check", main], work, env)
            mtime += 2
            with open(main, "w") as f:
                f.write(text)
            os.utime(main, (mtime, mtime))
            rr = run(RUSTC + ["-o", os.path.join(work, "r.rmeta"), main], work, env)
            cold = run([LUST, "check", main], work, cold_env)
            results = [("cold", cold)]
            if warm:
                results.append(("warm", run([LUST, "check", main], work, env)))
            rustc_ok = rr[0] == 0
            res["rustc_rejects"] += 0 if rustc_ok else 1
            key = f"{op} @ {place}"
            b = res["by"].setdefault(key, [0, 0])
            b[0] += 1
            bad = False
            for name, r in results:
                if r[0] is None:
                    res["hang"].append(f"{tag}: {name} lust timed out")
                    bad = True
                elif r[0] == 0 and not rustc_ok:
                    res["fa"].append(f"{tag}: {name} accepts, rustc: {rr[1].strip().splitlines()[0][:160]!r}")
                    bad = True
                elif r[0] != 0 and rustc_ok:
                    res["fr"].append(f"{tag}: {name} rejects: {r[1].strip().splitlines()[0][:160] if r[1].strip() else ''!r}")
                    bad = True
            if warm and None not in (results[0][1][0], results[1][1][0]) and (results[0][1][0] == 0) != (results[1][1][0] == 0):
                res["wc"].append(f"{tag}: warm rc={results[1][1][0]} cold rc={results[0][1][0]}")
                bad = True
            if bad:
                b[1] += 1
                if args_keep:
                    os.makedirs(args_keep, exist_ok=True)
                    with open(os.path.join(args_keep, f"f{idx}-m{j}.rs"), "w") as f:
                        f.write(text)
    finally:
        if daemon:
            try:
                os.killpg(os.getpgid(daemon.pid), signal.SIGTERM)
            except Exception:
                daemon.terminate()
        shutil.rmtree(work, ignore_errors=True)
    return idx, res


args_keep = None


def pass_files(rng, count):
    """Small tests/pass programs without `mod x;` files (each must stand alone)."""
    d = os.path.join(ROOT, "tests/pass")
    names = sorted(f for f in os.listdir(d) if f.endswith(".rs"))
    rng.shuffle(names)
    out = []
    for f in names:
        if len(out) >= count:
            break
        src = open(os.path.join(d, f)).read()
        if len(src) < 3000 and not re.search(r"^\s*(pub\s+)?mod\s+\w+\s*;", src, re.M) and "include" not in src:
            out.append(src)
    return out


def main():
    global args_keep
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--per-seed", type=int, default=60, help="mutants per seed program")
    ap.add_argument("--files", type=int, default=20, help="tests/pass programs to add to the hand seeds")
    ap.add_argument("--jobs", type=int, default=os.cpu_count())
    ap.add_argument("--keep", default=None)
    ap.add_argument("--no-warm", action="store_true")
    a = ap.parse_args()
    args_keep = a.keep
    rng = random.Random(a.seed)
    srcs = SEEDS + pass_files(rng, a.files)
    jobs = [(i, s, a.seed, a.per_seed, not a.no_warm) for i, s in enumerate(srcs)]
    tot = {"mutants": 0, "fa": [], "fr": [], "wc": [], "hang": [], "rustc_rejects": 0, "by": {}}
    with ProcessPoolExecutor(a.jobs, initializer=_init, initargs=(a.keep,)) as ex:
        for _, r in ex.map(case, jobs):
            for key in ("fa", "fr", "wc", "hang"):
                tot[key] += r[key]
            tot["mutants"] += r["mutants"]
            tot["rustc_rejects"] += r["rustc_rejects"]
            for key, (n, bad) in r["by"].items():
                t = tot["by"].setdefault(key, [0, 0])
                t[0] += n
                t[1] += bad
    for key in ("fa", "fr", "wc", "hang"):
        for line in tot[key]:
            print(f"FAIL[{key}] {line}")
    for key in sorted(tot["by"]):
        n, bad = tot["by"][key]
        print(f"  {key:32} {n:5} mutants, {bad} bad")
    print(
        f"delim sweep: {len(srcs)} seeds, {tot['mutants']} mutants ({tot['rustc_rejects']} rustc rejects): "
        f"{len(tot['fa'])} false accepts, {len(tot['fr'])} new false rejects, {len(tot['wc'])} warm!=cold, {len(tot['hang'])} hangs"
    )
    sys.exit(1 if tot["fa"] or tot["fr"] or tot["wc"] or tot["hang"] else 0)


def _init(keep):
    global args_keep
    args_keep = keep


if __name__ == "__main__":
    main()
