#!/usr/bin/env python3
# Derived impls of a GENERIC Adt applied at a CONCRETE arg inside another
# derive (fx-derivebound). `#[derive(Debug)] struct Arena { t: Cow<Table> }`
# needs `Cow<Table>: Debug`, which the derive on `Cow<T>` gives only under its
# bound `T: Debug`; rustc checks that at Arena's definition, used or not.
# Each generated program has a leaf type `Leaf` whose impl of one trait X is
# derived, written, or missing; a generic `G<T>` (struct, tuple struct, or
# enum) that derives X (or not, or writes it with or without a bound) over a
# template field shape (T, Vec<T>, PhantomData<T>, T::Item, a nested derived
# enum, ...); and an outer `Out` (or `Out<U>`) that derives X over a field
# that names `G<A>` for a concrete A (Leaf, Vec<Leaf>, &'static Leaf, ...).
# main uses X on Out or not. Versions toggle the Leaf and G impls and back,
# so the warm derive delta is compared with a cold lust and rustc 1.99.
#
# Runs on scripts/derive_sweep.py's runner (warm daemon, cold lust, rustc;
# same classes and exit rule). Usage:
#   python3 scripts/derive_arg_sweep.py [--count N] [--seed S] [--jobs J] [--keep DIR]
import argparse, os, random, sys
from concurrent.futures import ProcessPoolExecutor

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import derive_sweep  # scripts/derive_sweep.py
import oracle_log  # scripts/oracle_log.py
from derive_def_sweep import SUPERS, TRAITS, closure

# Written impls on Leaf(u8): (trait path, body).
LEAF_WRITTEN = {
    "Debug": ("fmt::Debug", 'fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, "L{}", self.0)\n    }'),
    "Clone": ("Clone", "fn clone(&self) -> Self {\n        Leaf(self.0)\n    }"),
    "Copy": ("Copy", ""),
    "PartialEq": ("PartialEq", "fn eq(&self, o: &Self) -> bool {\n        self.0 == o.0\n    }"),
    "Eq": ("Eq", ""),
    "PartialOrd": ("PartialOrd", "fn partial_cmp(&self, o: &Self) -> Option<Ordering> {\n        self.0.partial_cmp(&o.0)\n    }"),
    "Ord": ("Ord", "fn cmp(&self, o: &Self) -> Ordering {\n        self.0.cmp(&o.0)\n    }"),
    "Hash": ("Hash", "fn hash<H: Hasher>(&self, h: &mut H) {\n        self.0.hash(h)\n    }"),
    "Default": ("Default", "fn default() -> Self {\n        Leaf(4)\n    }"),
}

# Written impls on G<T> through its `gn()` (no T needed).
G_WRITTEN = {
    "Debug": ("fmt::Debug", 'fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {\n        write!(f, "G{}", self.gn())\n    }'),
    "PartialEq": ("PartialEq", "fn eq(&self, o: &Self) -> bool {\n        self.gn() == o.gn()\n    }"),
    "Eq": ("Eq", ""),
    "PartialOrd": ("PartialOrd", "fn partial_cmp(&self, o: &Self) -> Option<Ordering> {\n        self.gn().partial_cmp(&o.gn())\n    }"),
    "Ord": ("Ord", "fn cmp(&self, o: &Self) -> Ordering {\n        self.gn().cmp(&o.gn())\n    }"),
    "Hash": ("Hash", "fn hash<H: Hasher>(&self, h: &mut H) {\n        self.gn().hash(h)\n    }"),
}

LEAF_STATES = ["derive", "absent", "written"]
G_STATES = ["derive", "absent", "w_none", "w_same"]

# G's template field over T: (type, expr from `t: T`, needs). "it": T: Iterator.
G_SHAPES = [
    ("T", "t", ""),
    ("Vec<T>", "vec![t]", ""),
    ("Option<Box<T>>", "Some(Box::new(t))", ""),
    ("PhantomData<T>", "{ drop(t); PhantomData }", ""),
    ("(T, u8)", "(t, 2)", ""),
    ("[T; 1]", "[t]", ""),
    ("Wrap<T>", "Wrap(t)", ""),
    ("Vec<Chunk<T>>", "vec![Chunk::Own(t), Chunk::Freed]", ""),
    ("Option<fn() -> T>", "{ drop(t); None }", ""),
    ("Option<T::Item>", "t.next()", "it"),
]

# The concrete arg A of G: (type, value expr). "L" marks Leaf.
ARGS = [
    ("Leaf", "Leaf(1)"),
    ("Leaf", "Leaf(1)"),
    ("u8", "1u8"),
    ("String", 'String::from("s")'),
    ("Vec<Leaf>", "vec![Leaf(1)]"),
    ("Option<Leaf>", "Some(Leaf(1))"),
    ("Box<Leaf>", "Box::new(Leaf(1))"),
    ("Rc<Leaf>", "Rc::new(Leaf(1))"),
    ("(Leaf, u8)", "(Leaf(1), 2)"),
    ("[Leaf; 1]", "[Leaf(1)]"),
    ("&'static Leaf", "&LEAF"),
    ("PhantomData<Leaf>", "PhantomData"),
    ("Wrap<Leaf>", "Wrap(Leaf(1))"),
]
# Iterator args for the T::Item shape.
ITER_ARGS = [("std::vec::IntoIter<Leaf>", "vec![Leaf(1)].into_iter()"), ("std::option::IntoIter<Leaf>", "Some(Leaf(1)).into_iter()")]

# Out's field over GA (= G<A>), with the value over `ga()`.
OUT_SHAPES = [
    ("GA", "ga()"),
    ("GA", "ga()"),
    ("Vec<GA>", "vec![ga()]"),
    ("Option<GA>", "Some(ga())"),
    ("Box<GA>", "Box::new(ga())"),
    ("Rc<GA>", "Rc::new(ga())"),
    ("&'static GA", "Box::leak(Box::new(ga()))"),
    ("(GA, u8)", "(ga(), 3)"),
    ("[GA; 1]", "[ga()]"),
    ("Wrap<GA>", "Wrap(ga())"),
    ("G<GA>", "mk(ga())"),
    ("Option<Box<GA>>", "Some(Box::new(ga()))"),
]

USES = {
    "Debug": 'println!("{:?}", o);',
    "Clone": 'let c = o.clone();\n    println!("{}", c.on());',
    "Copy": 'let c = o;\n    let d = o;\n    println!("{} {}", c.on(), d.on());',
    "PartialEq": 'println!("{}", o == o2);',
    "Eq": 'need_eq(&o);\n    println!("eq");',
    "PartialOrd": 'println!("{:?}", o.partial_cmp(&o2));',
    "Ord": 'println!("{:?}", o.cmp(&o2));',
    "Hash": 'let mut h = DefaultHasher::new();\n    o.hash(&mut h);\n    println!("{}", h.finish());',
    "Default": 'let d: OUTTY = Default::default();\n    println!("{}", d.on());',
}

HEADER = """use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::rc::Rc;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Wrap<X>(X);
fn need_eq<X: Eq>(_: &X) {}
"""


def derive_line(ts):
    return f"#[derive({', '.join(t for t in TRAITS if t in ts)})]\n" if ts else ""


def leaf(x, state):
    """`Leaf(u8)` with its impl of X (and X's supertraits) in `state`; the
    other traits stay derived, so only X decides."""
    dependents = [t for t in TRAITS if x in SUPERS.get(t, [])]
    derived = [t for t in TRAITS if t not in dependents and t != x]
    written = []
    if state == "derive":
        derived += [x] + dependents
    elif state == "written":
        written.append(x)
        if x == "Copy" and "Clone" not in derived:
            written.append("Clone")
    out = [derive_line(derived), "struct Leaf(u8);\n"]
    for t in written:
        path, body = LEAF_WRITTEN[t]
        out.append(f"impl {path} for Leaf {{\n    {body}\n}}\n" if body else f"impl {path} for Leaf {{}}\n")
    out.append("static LEAF: Leaf = Leaf(1);\n")
    return "".join(out)


def g_def(x, state, shape, form):
    """`G<T>` over `shape` in `form` (struct, tuple, enum); derives X's closure,
    or not, or writes X (`w_none`: no bound, `w_same`: `T: X`) and derives
    X's supertraits."""
    ty, expr, needs = shape
    tp = "T: Iterator" if needs == "it" else "T"
    derived, written = [], []
    if state == "derive":
        derived = closure(x)
    elif state.startswith("w_"):
        derived = SUPERS.get(x, [])
        written = [x]
    out = [derive_line(derived)]
    if form == "struct":
        out.append(f"struct G<{tp}> {{\n    f: {ty},\n    n: u8,\n}}\n")
        build, gn = f"G {{ f: {expr}, n: 1 }}", "self.n"
    elif form == "tuple":
        out.append(f"struct G<{tp}>({ty}, u8);\n")
        build, gn = f"G({expr}, 1)", "self.1"
    else:
        dflt = "    #[default]\n" if x == "Default" else ""
        out.append(f"enum G<{tp}> {{\n    A({ty}, u8),\n{dflt}    B,\n}}\n")
        build, gn = f"G::A({expr}, 1)", "match self {\n            G::A(_, n) => *n,\n            G::B => 9,\n        }"
    mut = "mut " if needs == "it" else ""
    out.append(f"impl<{tp}> G<T> {{\n    fn gn(&self) -> u8 {{\n        {gn}\n    }}\n}}\n")
    out.append(f"fn mk<{tp}>({mut}t: T) -> G<T> {{\n    {build}\n}}\n")
    for t in written:
        path, body = G_WRITTEN[t]
        bounds = (["Iterator"] if needs == "it" else []) + ([path] if state == "w_same" else [])
        bound = f"<T: {' + '.join(bounds)}>" if bounds else "<T>"
        out.append(f"impl{bound} {path} for G<T> {{\n    {body}\n}}\n" if body else f"impl{bound} {path} for G<T> {{}}\n")
    # The nested derived enum of the `Vec<Chunk<T>>` shape.
    cd = "    #[default]\n" if x == "Default" else ""
    out.append(derive_line(closure(x)) + f"enum Chunk<T> {{\n    Own(T),\n{cd}    Freed,\n}}\n")
    return "".join(out)


def gen_program(x, gshape, gform, arg, oshape, oform, generic, used, lstate, gstate):
    aty, aval = arg
    if gshape[2] == "it":
        # G<A>::f is A::Item, so G<G<A>> is not a type here.
        if oshape[0] == "G<GA>":
            oshape = ("GA", "ga()")
    gty = f"G<{aty}>"
    fty = oshape[0].replace("GA", gty)
    fval = oshape[1]
    out = [HEADER, leaf(x, lstate), g_def(x, gstate, gshape, gform)]
    out.append(f"fn ga() -> {gty} {{\n    mk({aval})\n}}\n")
    out.append(derive_line(closure(x)))
    gp, ga, ufield, uval = ("<U>", "<U>", ", U", ", 5u8") if generic else ("", "", "", "")
    outty = "Out<u8>" if generic else "Out"
    if oform == "struct":
        uf = "\n    u: U," if generic else ""
        out.append(f"struct Out{gp} {{\n    a: {fty},\n    n: u8,{uf}\n}}\n")
        mkv = f"Out {{ a: {fval}, n: 1{', u: 5u8' if generic else ''} }}"
        on = "self.n"
    elif oform == "tuple":
        out.append(f"struct Out{gp}({fty}, u8{ufield});\n")
        mkv = f"Out({fval}, 1{uval})"
        on = "self.1"
    else:
        dflt = "    #[default]\n" if x == "Default" else ""
        out.append(f"enum Out{gp} {{\n    A({fty}, u8{ufield}),\n{dflt}    B,\n}}\n")
        mkv = f"Out::A({fval}, 1{uval})"
        on = "match self {\n            Out::A(_, n, ..) => *n,\n            Out::B => 7,\n        }"
    out.append(f"impl{gp} Out{ga} {{\n    fn on(&self) -> u8 {{\n        {on}\n    }}\n}}\n")
    if used == "generic":
        xp = "fmt::Debug" if x == "Debug" else x
        out.append(f"fn need<X: {xp}>(_: &X) {{}}\n")
    out.append("fn main() {\n")
    out.append(f"    let o: {outty} = {mkv};\n    let o2: {outty} = {mkv};\n")
    out.append('    println!("{} {}", o.on(), o2.on());\n')
    if used == "generic":
        out.append("    need(&o);\n")
    elif used:
        out.append("    " + USES[x].replace("OUTTY", outty) + "\n")
    out.append("}\n")
    return "".join(out)


def gen(seed, resolver=False):
    rng = random.Random(seed * 6271 + 29)
    x = rng.choice(TRAITS)
    gshape = rng.choice(G_SHAPES)
    # Most Copy programs take a field shape that can be Copy.
    if x == "Copy" and rng.random() < 0.8:
        gshape = rng.choice([g for g in G_SHAPES if g[0] in ("T", "PhantomData<T>", "(T, u8)", "[T; 1]", "Wrap<T>", "Option<fn() -> T>")])
    gform = rng.choice(["struct", "struct", "tuple", "enum"])
    arg = rng.choice(ITER_ARGS if gshape[2] == "it" else ARGS)
    oshape = rng.choice(OUT_SHAPES)
    if x == "Copy" and rng.random() < 0.8:
        oshape = rng.choice([o for o in OUT_SHAPES if o[0] in ("GA", "&'static GA", "(GA, u8)", "[GA; 1]", "Wrap<GA>", "G<GA>")])
    oform = rng.choice(["struct", "struct", "tuple", "enum"])
    generic = rng.random() < 0.3
    used = rng.choice([False, False, "direct", "generic"])
    # Printing or comparing a fn pointer reads its address.
    if "fn()" in gshape[0] and x not in ("Clone", "Copy", "Default"):
        used = used and "generic"
    gstates = G_STATES if x in G_WRITTEN else ["derive", "derive", "absent"]
    l1, g1 = rng.choice(LEAF_STATES), rng.choice(gstates)
    steps = [(l1, g1)]
    for _ in range(rng.choice([2, 3])):
        lp, gp = steps[-1]
        if rng.random() < 0.6:
            steps.append((rng.choice([s for s in LEAF_STATES if s != lp]), gp))
        else:
            steps.append((lp, rng.choice([s for s in gstates if s != gp] or gstates)))
    steps.append((l1, g1))
    vs = [gen_program(x, gshape, gform, arg, oshape, oform, generic, used, l, g) for l, g in steps]
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
    print(f"derive-arg-sweep: {a.count} programs, {tot['versions']} versions ({tot['rustc_rejects']} rustc rejects): "
          f"false_accepts={tot['fa']} warm_ne_cold={tot['wc']} wrong={tot['wrong']} hangs={tot['hang']} "
          f"false_rejects={tot['fr']}" + (f" oracle_checks={tot['oracle_ok']} oracle_failures={tot['oracle']}" if oracle_log.on() else ""))
    return 1 if tot["fa"] or tot["wc"] or tot["wrong"] or tot["hang"] or tot["oracle"] else 0


if __name__ == "__main__":
    sys.exit(main())
