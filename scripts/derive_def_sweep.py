#!/usr/bin/env python3
# Derived impls of GENERIC Adts, checked at the definition (fx-derivegen).
# rustc checks every derived impl body where the type is defined, under the
# derive's bounds (`T: Trait` per type param, `T::Item: Trait` per path that
# starts at a type param, the Adt's own bounds), whether or not a use
# instantiates the impl. Each generated program defines an inner type `In<T>`
# whose impl of one trait X is derived, written (with several bounds), or
# missing, and an outer `Out<..>` that derives X (plus its supertraits) over
# a field shape that names `In<..>` or a type param (Vec/Option/Box/Rc of it,
# tuples, arrays, references, PhantomData, Cell, projections, fn pointers,
# dyn Fn). main uses the derived impl or not. Versions then toggle the inner
# impl (derive bit, written impl added or removed, its bound changed) and
# back, so the warm derive and header deltas are compared with a cold lust
# and rustc 1.99 on every step.
#
# Runs on scripts/derive_sweep.py's runner (warm daemon, cold lust, rustc;
# same classes and exit rule). Usage:
#   python3 scripts/derive_def_sweep.py [--count N] [--seed S] [--jobs J] [--keep DIR]
import argparse, os, random, re, sys
from concurrent.futures import ProcessPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import derive_sweep  # scripts/derive_sweep.py
import oracle_log  # scripts/oracle_log.py

TRAITS = ["Debug", "Clone", "Copy", "PartialEq", "Eq", "PartialOrd", "Ord", "Hash", "Default"]
SUPERS = {"Copy": ["Clone"], "Eq": ["PartialEq"], "PartialOrd": ["PartialEq"], "Ord": ["PartialEq", "Eq", "PartialOrd"]}

# Written impls of each trait for `In<T>` (fields `v: u8, _p: Option<fn() -> T>`;
# not PhantomData, whose Debug and Hash lust lacks):
# (header trait path, body).
WRITTEN = {
    "Debug": ("fmt::Debug", 'fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, "In{}", self.v)\n    }'),
    "Clone": ("Clone", "fn clone(&self) -> Self {\n        In { v: self.v, _p: None }\n    }"),
    "Copy": ("Copy", ""),
    "PartialEq": ("PartialEq", "fn eq(&self, o: &Self) -> bool {\n        self.v == o.v\n    }"),
    "Eq": ("Eq", ""),
    "PartialOrd": ("PartialOrd", "fn partial_cmp(&self, o: &Self) -> Option<Ordering> {\n        self.v.partial_cmp(&o.v)\n    }"),
    "Ord": ("Ord", "fn cmp(&self, o: &Self) -> Ordering {\n        self.v.cmp(&o.v)\n    }"),
    "Hash": ("Hash", "fn hash<H: Hasher>(&self, h: &mut H) {\n        self.v.hash(h)\n    }"),
    "Default": ("Default", "fn default() -> Self {\n        In { v: 0, _p: None }\n    }"),
}

# Inner impl states of X: derived (with or without X), or written with a bound.
STATES = ["derive", "noderive", "w_none", "w_same", "w_more", "w_u8", "absent"]

# Field shapes: (type, value expr, needs) with `I` = the inner type at T, `t` a
# T value, `iv` an inner value. needs: "a" lifetime param, "n" const param,
# "it" T: Iterator (T = Range<u8> in main), "u" a second type param U.
SHAPES = [
    ("I", "iv()", ""),
    ("Vec<I>", "vec![iv()]", ""),
    ("Option<I>", "Some(iv())", ""),
    ("Box<I>", "Box::new(iv())", ""),
    ("Rc<I>", "Rc::new(iv())", ""),
    ("(I, u8)", "(iv(), 1)", ""),
    ("[I; 2]", "[iv(), iv()]", ""),
    ("&'a I", "&INNER", "a"),
    ("PhantomData<I>", "PhantomData", ""),
    ("Wrap<I>", "Wrap(iv())", ""),
    ("Option<Wrap<Vec<I>>>", "Some(Wrap(vec![iv()]))", ""),
    ("T", "tv()", ""),
    ("Vec<T>", "vec![tv()]", ""),
    ("Option<Box<T>>", "Some(Box::new(tv()))", ""),
    ("(T, u8)", "(tv(), 2)", ""),
    ("[T; N]", "[tv(); N]", "n"),
    ("&'a T", "&TV", "a"),
    ("&'a mut T", "&mut TVM", "a"),
    ("fn(T) -> T", "idf", ""),
    ("Box<dyn Fn(T)>", "Box::new(|_| {})", ""),
    ("Cell<T>", "Cell::new(tv())", ""),
    ("PhantomData<T>", "PhantomData", ""),
    ("(T, U)", "(tv(), uv())", "u"),
    ("Vec<(U, T)>", "vec![(uv(), tv())]", "u"),
    ("(Box<str>, Rc<N0>, T)", '("s".into(), Rc::new(N0(1)), tv())', ""),
    ("Box<[I]>", "vec![iv()].into_boxed_slice()", ""),
    ("Box<[T]>", "vec![tv()].into_boxed_slice()", ""),
    ("Option<Rc<T>>", "Some(Rc::new(tv()))", ""),
    ("T::Item", "1u8", "it"),
    ("Vec<T::Item>", "vec![1u8]", "it"),
    ("Option<(T::Item, T)>", "None", "it"),
    ("<T as Iterator>::Item", "1u8", "it"),
]

# One use of the derived impl of each trait on `o` and `o2` (two Out values).
USES = {
    "Debug": 'println!("{:?}", o);',
    "Clone": 'let c = o.clone();\n    println!("{}", c.n);',
    "Copy": 'let c = o;\n    let d = o;\n    println!("{} {}", c.n, d.n);',
    "PartialEq": 'println!("{}", o == o2);',
    "Eq": "need_eq(&o);\n    println!(\"eq\");",
    "PartialOrd": 'println!("{:?}", o.partial_cmp(&o2));',
    "Ord": 'println!("{:?}", o.cmp(&o2));',
    "Hash": 'let mut h = DefaultHasher::new();\n    o.hash(&mut h);\n    println!("{}", h.finish());',
    "Default": 'let d: OUTTY = Default::default();\n    println!("{}", d.n);',
}

HEADER = """use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::rc::Rc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Wrap<X>(X);
fn need_eq<X: Eq>(_: &X) {}
fn idf<X>(x: X) -> X {
    x
}
"""


def closure(x):
    return SUPERS.get(x, []) + [x]


def inner(x, state):
    """`In<T>` with its impl of X (and X's supertraits) in `state`; every
    other derivable trait stays derived, so only X decides."""
    sup = SUPERS.get(x, [])
    # Traits whose derive needs X (dropping X drops them too).
    dependents = [t for t in TRAITS if x in SUPERS.get(t, [])]
    derived = [t for t in TRAITS if t not in dependents and t != x]
    written = []
    if state == "derive":
        derived.append(x)
        derived += dependents
    elif state.startswith("w_"):
        # X and its supertraits written with no bound, X with the state's bound.
        derived = [t for t in derived if t not in sup]
        for s in sup:
            written.append((s, "impl<T>", "In<T>"))
        hdr = {"w_none": ("impl<T>", "In<T>"), "w_same": (f"impl<T: {x}>", "In<T>"), "w_more": (f"impl<T: {x} + Default>", "In<T>"), "w_u8": ("impl", "In<u8>")}[state]
        written.append((x, hdr[0], hdr[1]))
    out = [f"#[derive({', '.join(t for t in TRAITS if t in derived)})]\n" if derived else ""]
    out.append("struct In<T> {\n    v: u8,\n    _p: Option<fn() -> T>,\n}\n")
    for t, imp, self_ty in written:
        path, body = WRITTEN[t]
        out.append(f"{imp} {path} for {self_ty} {{\n    {body}\n}}\n" if body else f"{imp} {path} for {self_ty} {{}}\n")
    return "".join(out)


def gen_program(rng, x, shape, state, used, is_enum, drop_super, extra_bound, tconc):
    ty, val, needs = shape
    params = []
    if "a" in needs:
        params.append("'a")
    params.append("T: Iterator" if "it" in needs else ("T: Clone" if extra_bound else "T"))
    if "u" in needs:
        params.append("U")
    if "n" in needs:
        params.append("const N: usize")
    ity = "In<T>"
    fty = re.sub(r"\bI\b", ity, ty)
    derives = [t for t in closure(x) if t not in drop_super]
    out = [HEADER, inner(x, state)]
    out.append(f"#[derive({', '.join(derives)})]\n")
    gp = ", ".join(params)
    if is_enum:
        dflt = "    #[default]\n" if x == "Default" and rng.random() < 0.9 else ""
        out.append(f"enum Out<{gp}> {{\n    A({fty}, u8),\n{dflt}    B,\n}}\n")
    else:
        out.append(f"struct Out<{gp}> {{\n    a: {fty},\n    n: u8,\n}}\n")
    if is_enum:
        out.append(f"impl<{gp}> Out<{', '.join(p.split(':')[0].replace('const ', '') for p in params)}> {{\n    fn n(&self) -> u8 {{\n        match self {{\n            Out::A(_, n) => *n,\n            Out::B => 7,\n        }}\n    }}\n}}\n")
    t = "std::ops::Range<u8>" if "it" in needs else tconc
    tvals = {"u8": "3u8", "String": 'String::from("s")', "N0": "N0(0)", "std::ops::Range<u8>": "0u8..3"}
    out.append("struct N0(u8);\n")
    out.append(f"fn iv() -> In<{t}> {{\n    In {{ v: 5, _p: None }}\n}}\n")
    out.append(f"fn tv() -> {t} {{\n    {tvals[t]}\n}}\n")
    out.append("fn uv() -> u16 {\n    9\n}\n")
    args = (["'_"] if "a" in needs else []) + [t] + (["u16"] if "u" in needs else []) + (["2"] if "n" in needs else [])
    outty = f"Out<{', '.join(args)}>"
    v = val.replace("TV", "tvv").replace("INNER", "innv").replace("&mut tvvM", "&mut tvm")
    if "N]" in v:
        v = "[tv(), tv()]"
    out.append("fn main() {\n")
    if "a" in needs:
        out.append("    let innv = iv();\n    let tvv = tv();\n    let mut tvm = tv();\n    let mut tvm2 = tv();\n")
    def mk(var, mut):
        fv = v.replace("tvm", "tvm2") if mut else v
        if is_enum:
            return f"    let {var}: {outty} = Out::A({fv}, 1);\n"
        return f"    let {var}: {outty} = Out {{ a: {fv}, n: 1 }};\n"
    out.append(mk("o", False))
    out.append(mk("o2", True))
    nexpr = "o.n()" if is_enum else "o.n"
    out.append(f"    println!(\"{{}}\", {nexpr});\n")
    if used == "generic":
        # The derived impl's record: `Out<T, ..>: X` under `T: X` only.
        tp = [q.split(":")[0].replace("const ", "") for q in params]
        xp = "fmt::Debug" if x == "Debug" else x
        wh = ", ".join(f"{q}: {xp}" for q in tp if q in ("T", "U"))
        gargs = ", ".join("'_" if q == "'a" else q for q in tp)
        out.insert(out.index("fn main() {\n"), f"fn need<X: {xp}>(_: &X) {{}}\nfn via<{gp}>(o: &Out<{gargs}>) where {wh} {{\n    need(o);\n}}\n")
        out.append("    via(&o);\n")
    elif used:
        use = USES[x].replace("OUTTY", outty)
        if is_enum:
            use = use.replace("c.n", "c.n()").replace("d.n", "d.n()")
        out.append("    " + use + "\n")
    out.append("}\n")
    return "".join(out)


def gen(seed, resolver=False):
    rng = random.Random(seed * 7919 + 13)
    x = rng.choice(TRAITS)
    shape = rng.choice(SHAPES)
    used = rng.choice([False, False, "direct", "generic"])
    # A fn pointer's Debug, Hash, and compares read its address.
    if shape[0].startswith("fn(") and x not in ("Clone", "Copy"):
        used = used and "generic"
    is_enum = rng.random() < 0.3
    drop_super = [s for s in SUPERS.get(x, []) if rng.random() < 0.08]
    extra_bound = rng.random() < 0.15
    tconc = rng.choice(["u8", "u8", "u8", "String", "N0"])
    s1 = rng.choice(STATES)
    s2 = rng.choice([s for s in STATES if s != s1])
    states = [s1, s2, s1] + ([rng.choice(STATES)] if rng.random() < 0.5 else [])
    vs = [gen_program(random.Random(seed), x, shape, st, used, is_enum, drop_super, extra_bound, tconc) for st in states]
    if resolver:
        vs = ["const _ROUTE: u8 = 0;\n" + v for v in vs]
    return vs


def case(args):
    derive_sweep.gen = gen
    return derive_sweep.case(args)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=100)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--keep", default="")
    ap.add_argument("--resolver", action="store_true")
    ap.add_argument("--print", type=int, default=0, help="print the versions of seed N and stop")
    a = ap.parse_args()
    if a.print:
        for i, v in enumerate(gen(a.print, a.resolver)):
            print(f"// ---- v{i + 1}\n{v}")
        return 0
    seeds = [(a.seed + i, a.keep, a.resolver) for i in range(a.count)]
    with ProcessPoolExecutor(max_workers=a.jobs) as pool:
        results = list(pool.map(case, seeds))
    tot = {"versions": 0, "fa": 0, "fr": 0, "wc": 0, "wrong": 0, "hang": 0, "oracle": 0, "oracle_ok": 0, "rustc_rejects": 0}
    for seed, r in sorted(results):
        for k in ("fa", "wc", "wrong", "hang", "oracle"):
            for line in r[k]:
                print(f"{k.upper()} {line}")
            tot[k] += len(r[k])
        for k in ("versions", "fr", "rustc_rejects", "oracle_ok"):
            tot[k] += r[k]
    print(f"derive-def-sweep: {a.count} programs, {tot['versions']} versions ({tot['rustc_rejects']} rustc rejects): "
          f"false_accepts={tot['fa']} warm_ne_cold={tot['wc']} wrong={tot['wrong']} hangs={tot['hang']} "
          f"false_rejects={tot['fr']}" + (f" oracle_checks={tot['oracle_ok']} oracle_failures={tot['oracle']}" if oracle_log.on() else ""))
    return 1 if tot["fa"] or tot["wc"] or tot["wrong"] or tot["hang"] or tot["oracle"] else 0


if __name__ == "__main__":
    sys.exit(main())
