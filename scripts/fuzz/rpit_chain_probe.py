#!/usr/bin/env python3
"""Edit-pair probes for the RPIT chain (sigedit: Sol's finding 1).

An item that an unchanged `-> impl Trait` fn reads changes (a fn's return or
param type, a struct field, a type alias); the opaque's hidden type moves,
and a caller that needs an auto trait (or the trait itself) from the opaque
must see it. Each probe primes a warm daemon session with version A, then
checks version B warm, cold, and with rustc 1.99; half the probes go B to A
instead. Shapes vary: the opaque's definer (free fn, method, generic fn,
nested `Option<impl ..>`, a 2 or 3 deep chain), what changes, the caller
(direct, through a let, in a generic fn, in a closure, `thread::spawn`).

Classes: false accept (warm accepts, rustc rejects), warm != cold (verdict
or first error code), hang (a timeout is never a reject).

Usage: rpit_chain_probe.py --count 400 --seed 1 [--lust target/release/lust] [--out DIR]
"""
import argparse, os, random, re, shutil, signal, subprocess, sys, tempfile, time

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

CHANGES = [
    # (A item text, B item text): what f returns, through the edited item.
    ("fn f() -> u32 { 7 }", "fn f() -> std::rc::Rc<u32> { std::rc::Rc::new(7) }"),
    ("fn f() -> u32 { 7 }", "fn f() -> std::cell::Cell<u32> { std::cell::Cell::new(7) }"),
    ("fn f() -> u32 { 7 }", "fn f() -> *const u8 { std::ptr::null() }"),
    ("fn f() -> u32 { 7 }", "fn f() -> u64 { 7 }"),
    ("fn f() -> u32 { 7 }", "fn f() -> String { String::new() }"),
    ("fn f() -> u32 { 7 }", "fn f() -> Vec<std::rc::Rc<u8>> { Vec::new() }"),
    ("struct W(u32);\nfn f() -> W { W(7) }", "struct W(std::rc::Rc<u32>);\nfn f() -> W { W(std::rc::Rc::new(7)) }"),
    ("struct W { v: u32 }\nfn f() -> W { W { v: 7 } }", "struct W { v: std::cell::Cell<u32> }\nfn f() -> W { W { v: std::cell::Cell::new(7) } }"),
    ("type T = u32;\nfn f() -> T { 7 }", "type T = std::rc::Rc<u32>;\nfn f() -> T { std::rc::Rc::new(7) }"),
    ("fn f() -> u32 { 7 }", "fn f() -> u32 where u32: Copy { 7 }"),
]
TRAITS = ["std::fmt::Debug", "Sized", "std::any::Any", "Clone"]
AUTOS = ["Send", "Sync", "Send + Sync"]


def definer(rng, trait):
    """(items defining the opaque user, the expression a caller writes)."""
    form = rng.choice(["free", "method", "generic", "option", "chain2", "chain3"])
    if form == "free":
        return f"fn g() -> impl {trait} {{ f() }}\n", "g()"
    if form == "method":
        return f"struct S;\nimpl S {{\n    fn g(&self) -> impl {trait} {{ f() }}\n}}\n", "S.g()"
    if form == "generic":
        return f"fn g<X: Copy>(_x: X) -> impl {trait} {{ f() }}\n", "g(1u8)"
    if form == "option":
        return f"fn g() -> Option<impl {trait}> {{ Some(f()) }}\n", "g()"
    out = f"fn g1() -> impl {trait} {{ f() }}\n"
    depth = 2 if form == "chain2" else 3
    for k in range(2, depth + 1):
        out += f"fn g{k}() -> impl {trait} {{ g{k - 1}() }}\n"
    return out, f"g{depth}()"


def caller(rng, expr, auto):
    shape = rng.choice(["direct", "let", "generic", "closure", "spawn", "unused_fn"])
    need = f"fn need<T: {auto}>(t: T) -> T {{ t }}\n"
    if shape == "direct":
        return need + f"fn main() {{ let _ = need({expr}); }}\n"
    if shape == "let":
        return need + f"fn main() {{ let x = {expr}; let _y = need(x); }}\n"
    if shape == "generic":
        return need + f"fn c<Z>(z: Z) -> Z {{ let _ = need({expr}); z }}\nfn main() {{ let _ = c(1); }}\n"
    if shape == "closure":
        return need + f"fn main() {{ let k = || need({expr}); let _ = k(); }}\n"
    if shape == "spawn" and "Sync" not in auto:
        return f"fn main() {{ let x = {expr}; let _ = std::thread::spawn(move || {{ let _y = x; }}).join(); }}\n"
    return need + f"#[allow(dead_code)]\nfn unused() {{ let _ = need({expr}); }}\nfn main() {{}}\n"


def probe(rng):
    a_item, b_item = rng.choice(CHANGES)
    trait = rng.choice(TRAITS)
    defs, expr = definer(rng, trait)
    tail = defs + caller(rng, expr, rng.choice(AUTOS))
    a, b = a_item + "\n" + tail, b_item + "\n" + tail
    return (a, b) if rng.random() < 0.5 else (b, a)


def run(cmd, cwd, env, timeout):
    try:
        r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)
        return r.returncode, r.stderr
    except subprocess.TimeoutExpired:
        return "TIMEOUT", ""


def code(err):
    m = re.search(r"error\[(L\d+)\]", err)
    return m.group(1) if m else None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=400)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--lust", default=os.path.join(ROOT, "target/release/lust"))
    ap.add_argument("--out", default=None)
    ap.add_argument("--timeout", type=float, default=60)
    a = ap.parse_args()
    rng = random.Random(a.seed)
    work = tempfile.mkdtemp(prefix="lust-rpitprobe-")
    out = a.out or os.path.join(work, "bad")
    sock = os.path.join(work, "d.sock")
    env = dict(os.environ, LUST_DAEMON_SOCKET=sock, LUST_NO_CACHE="1")
    env.pop("LUST_NO_DAEMON", None)
    cold = dict(env, LUST_NO_DAEMON="1", LUST_DAEMON_SOCKET=os.path.join(work, "none"))
    log = open(os.path.join(work, "daemon.log"), "w")

    def start():
        if os.path.exists(sock):
            os.unlink(sock)
        d = subprocess.Popen([a.lust, "daemon"], cwd=work, env=env, stderr=log, start_new_session=True)
        t0 = time.time()
        while not os.path.exists(sock):
            if time.time() - t0 > 10:
                raise RuntimeError("daemon start failed")
            time.sleep(0.01)
        return d

    daemon = start()
    counts = {"probes": 0, "false_accept": 0, "warm_ne_cold": 0, "hang": 0, "rustc_rejects": 0,
              "lust_false_reject": 0}
    try:
        for i in range(a.count):
            va, vb = probe(rng)
            d = os.path.join(work, f"p{i}")
            os.makedirs(d)
            main_rs = os.path.join(d, "main.rs")
            with open(main_rs, "w") as f:
                f.write(va)
            run([a.lust, "check", main_rs], d, env, a.timeout)
            with open(main_rs, "w") as f:
                f.write(vb)
            w, werr = run([a.lust, "check", main_rs], d, env, a.timeout)
            c, cerr = run([a.lust, "check", main_rs], d, cold, a.timeout)
            # --emit=obj, not metadata: metadata skips the MIR deny lints and post-mono errors.
            r, _ = run(["rustc", "--edition", "2024", "--crate-type", "bin", "-C", "panic=abort",
                        "-C", "overflow-checks=on", "--emit=obj", "-o", os.path.join(d, "x.o"), main_rs], d, env, a.timeout)
            counts["probes"] += 1
            bad = []
            if "TIMEOUT" in (w, c):
                counts["hang"] += 1
                bad.append("hang")
                daemon.kill()
                daemon = start()
            else:
                if (w == 0) != (c == 0) or code(werr) != code(cerr):
                    counts["warm_ne_cold"] += 1
                    bad.append(f"warm {w} {code(werr)} != cold {c} {code(cerr)}")
                if r != "TIMEOUT" and r != 0:
                    counts["rustc_rejects"] += 1
                    if w == 0:
                        counts["false_accept"] += 1
                        bad.append("false accept")
                if r == 0 and c != 0:
                    counts["lust_false_reject"] += 1
            if bad:
                os.makedirs(out, exist_ok=True)
                for tag, text in (("a", va), ("b", vb)):
                    with open(os.path.join(out, f"p{i}-{tag}.rs"), "w") as f:
                        f.write(text)
                print(f"BAD p{i}: {'; '.join(bad)}", flush=True)
            shutil.rmtree(d, ignore_errors=True)
    finally:
        try:
            os.killpg(os.getpgid(daemon.pid), signal.SIGTERM)
        except Exception:
            pass
    print("rpit_chain_probe: " + " ".join(f"{k}={v}" for k, v in counts.items()))
    bad = counts["false_accept"] + counts["warm_ne_cold"] + counts["hang"]
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
