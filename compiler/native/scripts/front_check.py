#!/usr/bin/env python3
# Release builds through a front image vs one-shot release builds, on a small corpus of
# layout-sensitive edits (rel-front, after b24cbd1a6's c_layout phase cache made WRONG CODE in
# image builds only: a cache hit skipped a body's Layout read, so the body kept old field
# offsets when a type it reads through a pointer grew). Each sequence is one program edited
# step by step in its own directory; every step builds twice:
#   image     the release driver (release C) with a front image kept across the steps
#             (LUST_FRONT_STATE, LUST_FRONT_SAVE=background, as scripts/release-x64.sh FRONT=1
#             does), one codegen thread (so bodies share one thread's caches), the disk cache on;
#   one-shot  the same with no image and no cache (LUST_NO_CACHE=1), 4 threads.
# Both write their units (--emit-c); the unit files must be equal byte for byte (compared by
# content), and both binaries must give the same exit status and stdout. Every sequence runs
# with release C and with the LLVM backend (LUST_RELEASE_BACKEND=llvm; LUST_CC=clang where no
# zig is on PATH and cc is not clang). The corpus: a type held by value inside a struct that
# fns read through a pointer grows (directly, as an enum payload, behind Option, as an array,
# as a generic arg), statics and thread_local keys are added, and tests/daemon/dt01's six
# versions. Every step must give the same status and stderr on both sides, a rejected build
# too (the same diagnostics in the same order). The one-shot side is a fresh session: its own
# numbering and no caches, so it shares no state with the image side. Four recovery sequences
# (the kept-program reviews, 2026-10-19) put rejected builds between accepted ones, and mix
# `lust check` steps into the image's builds (a mode switch). At the end the run prints the
# image side's front-end paths (path/why/kept of each request, from LUST_VISITS_LOG): a green
# run where every step took the full path proves little about a fast path.
# Two sequences cover the LLVM units' inputs that zig's compile cache cannot see (b41791434):
# an edit that changes a unit's helpers' C but not its IR (zig keyed the object on the IR and
# served the last one at that path: the old string printed), and a merge that removes items,
# built with a profile (PGO unit names, per-unit profiles). The PGO one trains at its first
# step (an instrumented one-shot build, run once) and needs zig plus a clang and llvm-profdata
# of zig's LLVM major (LUST_PGO_CC, LUST_PROFDATA, or clang-<N> and llvm-profdata-<N> on PATH);
# without them it is skipped and says so.
# lang-hasher-edit: a written `Hasher` impl gains and loses a method while generic bodies over a
# struct that holds a map with that hasher are kept (fx-coldmono's LangImpls finding, 2026-10-19).
# Three dt01 sequences take the shapes of CI jobs (rel-cache's merge 12 replay, 2026-10-18): image
# builds that emit no C before the compared build, a jump over versions no build saw, and builds
# that each start again from an older state (a CI disk whose commits did not land).
# The image side runs B1's oracle (LUST_CHECK_UNIT_FP=1); dt01-b1-skip runs B1's skipping
# (LUST_B1=1) in builds that emit no C. It also runs the generic reuse oracle
# (LUST_CHECK_GENREUSE=1, src/driver/genreuse_oracle.rs): every body and stored entry the image's
# session keeps must have clean reads, and every kept body made again without reuse the same C.
# sources-late: each image build starts before its step's files are written, with
# LUST_SOURCES_READY (a CI checkout beside the restore); the files come 0.3 s later, then the
# ready file, so a build that read any file early answers for the step before.
#
# env-in-image: two image builds with an unread marker var in the env and raw image layers
# (LUST_FRONT_ZSTD=0); no file of the state or cache directory may hold the marker (a front image
# save writes every live heap block), and each must hold the value of a var `env!` read (the scan
# sees heap strings).
#
# LUST_BIN overrides the lust binary (default target/release/lust).
# Usage: front_check.py [--jobs N] [--keep DIR] [--only NAME]
# Exit 0 = all same, 1 = a difference or a build failure on one side, 2 = setup error.
import argparse, hashlib, os, re, shutil, subprocess, sys, tempfile, time
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LUST = os.environ.get("LUST_BIN") or os.path.join(ROOT, "target/release/lust")
BUILD_TIMEOUT = 180
RUN_TIMEOUT = 20


def nested(fields_t, init_t, use_t):
    """Two fns read `y` through `&S`, `S { t: T, y: u64 }`; `T`'s fields are the edit."""
    return f'''struct T {{
{fields_t}
}}

struct S {{
    t: T,
    y: u64,
}}

#[inline(never)]
fn get_a(p: &S) -> u64 {{
    p.y
}}

#[inline(never)]
fn get_b(p: &S) -> u64 {{
    p.y + 1
}}

#[inline(never)]
fn both(p: &S) -> u64 {{
    get_a(p) * 100 + get_b(p)
}}

fn main() {{
    let s = S {{ t: {init_t}, y: 42 }};
    println!("{{}} {{}} {{}} {{}}", get_a(&s), get_b(&s), both(&s), {use_t});
}}
'''


def enum_payload(variants, arms, extra_main):
    return f'''#[derive(Clone, Copy)]
enum E {{
{variants}
}}

struct S {{
    e: E,
    y: u64,
}}

#[inline(never)]
fn read_y(p: &S) -> u64 {{
    p.y
}}

#[inline(never)]
fn read_y2(p: &S) -> u64 {{
    p.y * 2
}}

#[inline(never)]
fn read_e(p: &S) -> u64 {{
    match p.e {{
{arms}
    }}
}}

fn main() {{
    let s = S {{ e: E::B(7, 3), y: 42 }};
    let t = S {{ e: E::A(5), y: 9 }};
    println!("{{}} {{}} {{}} {{}}", read_y(&s), read_y2(&t), read_e(&s), read_e(&t));
{extra_main}
}}
'''


def generic(fields_inner, init_inner, fields_w, init_w_extra, use_inner):
    return f'''struct Inner {{
{fields_inner}
}}

struct W<T> {{
    t: T,
{fields_w}
    n: u64,
}}

#[inline(never)]
fn get<T>(w: &W<T>) -> u64 {{
    w.n
}}

#[inline(never)]
fn get_plus(w: &W<Inner>) -> u64 {{
    w.n + 1
}}

fn main() {{
    let w = W {{ t: {init_inner},{init_w_extra} n: 40 }};
    let v = W {{ t: 7u8,{init_w_extra} n: 3 }};
    println!("{{}} {{}} {{}} {{}}", get(&w), get_plus(&w), get(&v), {use_inner});
}}
'''


def option_wrap(fields_t, init_t):
    return f'''struct T {{
{fields_t}
}}

struct S {{
    o: Option<T>,
    y: u64,
}}

#[inline(never)]
fn get_a(p: &S) -> u64 {{
    p.y
}}

#[inline(never)]
fn get_b(p: &S) -> u64 {{
    p.y + 1
}}

fn main() {{
    let s = S {{ o: Some({init_t}), y: 42 }};
    let n = S {{ o: None, y: 5 }};
    println!("{{}} {{}} {{}} {{}}", get_a(&s), get_b(&s), get_b(&n), s.o.is_some());
}}
'''


def array_len(n):
    return f'''struct T {{
    buf: [u8; {n}],
}}

struct S {{
    t: T,
    y: u64,
}}

#[inline(never)]
fn get_a(p: &S) -> u64 {{
    p.y
}}

#[inline(never)]
fn get_b(p: &S) -> u64 {{
    p.y + p.t.buf[0] as u64
}}

fn main() {{
    let s = S {{ t: T {{ buf: [1; {n}] }}, y: 42 }};
    println!("{{}} {{}} {{}}", get_a(&s), get_b(&s), s.t.buf.len());
}}
'''


def statics(first_keys, uses):
    return f'''use std::cell::Cell;

thread_local! {{
{first_keys}
    static B: Cell<u64> = Cell::new(20);
}}

static NAMES: [&str; 3] = ["x", "y", "z"];

fn bump_b() -> u64 {{
    B.with(|b| {{
        b.set(b.get() + 1);
        b.get()
    }})
}}

fn main() {{
    println!("{{}} {{}} {{}}", bump_b(), bump_b(), NAMES[2]);
{uses}
}}
'''


def helper_edit(label, step):
    """Only `main`'s string and constant change: the LLVM unit's IR stays, its helpers' C moves."""
    return f'''struct Acc {{
    total: u64,
    hits: u32,
}}

impl Acc {{
    #[inline(never)]
    fn add(&mut self, x: u64) {{
        self.total += x;
        self.hits += 1;
    }}

    #[inline(never)]
    fn report(&self) -> String {{
        format!("total={{}} hits={{}}", self.total, self.hits)
    }}
}}

#[inline(never)]
fn run(n: u64) -> Acc {{
    let mut a = Acc {{ total: 0, hits: 0 }};
    for i in 0..n {{
        if i % 3 == 0 {{
            a.add(i * 7);
        }}
    }}
    a
}}

fn main() {{
    let a = run(10);
    println!("{{}} [{label}] {{}}", a.report(), {step});
}}
'''


def hot_flip(runs):
    """Release C copies by profile: only main changes, so `walk` runs or not; a profile of each
    version makes `walk` (and its std callees, and `label` with its literals: a copy with
    kconst's slots) hot or cold while its unit's code stays."""
    return f'''fn label(x: u64) -> &'static str {{
    if x % 3 == 0 {{ "fizz" }} else {{ "buzz" }}
}}

fn walk(v: &[u64]) -> u64 {{
    let mut s = 0;
    for i in 0..v.len() {{
        s += v[i] * 3 + (v[i] >> 1) + label(v[i]).len() as u64;
    }}
    s + v.iter().copied().max().unwrap_or(0)
}}

fn fill_a(n: u64) -> u64 {{
    (0..n).map(|i| i * i).sum()
}}

fn fill_b(n: u64) -> u64 {{
    let mut s = String::new();
    for i in 0..n {{
        s.push_str(&i.to_string());
    }}
    s.len() as u64
}}

fn fill_c(n: u64) -> u64 {{
    let mut v: Vec<u64> = (0..n).collect();
    v.sort_by(|a, b| b.cmp(a));
    v[0]
}}

fn main() {{
    let v: Vec<u64> = (0..100).collect();
    let runs = {runs};
    if runs > 0 {{
        println!("{{}}", walk(&v));
    }}
    println!("{{}}", fill_a(10) + fill_b(10) + fill_c(10));
}}
'''


def shapes(with_tag):
    """Methods in several impl blocks of one type, called across fns; a merge removes one."""
    tag_impl = '''impl Shape {
    #[inline(never)]
    fn tag(&self) -> &'static str {
        if self.w > self.h { "wide" } else { "tall" }
    }
}

''' if with_tag else ""
    tag_use = '''    println!("{}", v[0].tag());
''' if with_tag else ""
    return f'''struct Shape {{
    w: u32,
    h: u32,
}}

impl Shape {{
    #[inline(never)]
    fn area(&self) -> u32 {{
        self.w * self.h
    }}
}}

{tag_impl}impl Shape {{
    #[inline(never)]
    fn perim(&self) -> u32 {{
        2 * (self.w + self.h)
    }}
}}

impl Shape {{
    fn scaled(&self, k: u32) -> Shape {{
        Shape {{ w: self.w * k, h: self.h * k }}
    }}
}}

#[inline(never)]
fn total(v: &[Shape]) -> u64 {{
    let mut t = 0u64;
    for s in v {{
        t += s.area() as u64 + s.perim() as u64;
    }}
    t
}}

#[inline(never)]
fn describe(s: &Shape) -> String {{
    format!("{{}}x{{}} p={{}} a={{}}", s.w, s.h, s.perim(), s.area())
}}

fn main() {{
    let mut v = Vec::new();
    for i in 1..200u32 {{
        v.push(Shape {{ w: i % 7 + 1, h: i % 5 + 2 }}.scaled(2));
    }}
    let mut sum = 0u64;
    for _ in 0..50 {{
        sum += total(&v);
    }}
    println!("{{}} {{}}", sum, describe(&v[2]));
{tag_use}}}
'''


def lifetime_variants(swap):
    """Two instances of `M` that differ only in lifetimes share one C type,
    named by the variant registered first (`register_adt_instance`): one
    body names the `'a` variants as local types, another calls a method
    on the `'static` ones (which makes them as its instance's owner). A
    build that makes a request's instance before an earlier body's turn
    (c847cb8b6, reverted) registers them in another order than an image
    build: the C name moved (tsgo replay states 9-11). `swap` puts the
    two bodies the other way round."""
    first = """#[inline(never)]
fn first<'a>(r: Rc<dyn Tr + 'a>, s: &'a str) -> u32 {
    let m: M<Rc<dyn Tr + 'a>> = M { t: r, n: 5 };
    let w: M<&'a str> = M { t: s, n: 1 };
    m.n + m.t.v() + w.n + w.t.len() as u32
}
"""
    second = """#[inline(never)]
fn second(r: Rc<dyn Tr>, s: &'static str) -> u32 {
    let m = M { t: r, n: 7 };
    let w = M { t: s, n: 2 };
    m.get() + w.get()
}
"""
    a, b = (second, first) if swap else (first, second)
    return LIFETIME_HEAD + a + "\n" + b + """
fn main() {
    let k: Rc<dyn Tr> = Rc::new(K(3));
    println!("{} {}", first(k.clone(), "abc"), second(k, "de"));
}
"""


def recovery(state):
    """A small program with a trait, a generic, a closure and a struct read through a
    pointer, in one of five states for the recovery sequences: `ok1`, `ok2` (other
    bodies, one more generic instance), `body_err` and `body_err2` (two different body
    errors), and `type_err` (a field names a type that does not exist). A rejected
    build must not leave state that changes a later answer."""
    field = "    w: Missing," if state == "type_err" else "    w: u16,"
    scale = {
        "body_err": '    let bad: u8 = "x";\n    p.v as u64 * 3 + bad as u64',
        "ok2": "    p.v as u64 * 5 + p.w as u64",
    }.get(state, "    p.v as u64 * 3 + p.w as u64")
    extra = {
        "body_err2": "    let n = p.no_such_method();\n    n + wrap(1u8).0 as u64",
        "ok2": "    wrap(1u8).0 as u64 + wrap(2u64).0",
    }.get(state, "    wrap(1u8).0 as u64")
    return f'''struct P {{
    v: u32,
{field}
}}

struct Wrap<T>(T);

fn wrap<T>(t: T) -> Wrap<T> {{
    Wrap(t)
}}

trait Area {{
    fn area(&self) -> u64;
}}

impl Area for P {{
    fn area(&self) -> u64 {{
        self.v as u64 * 2
    }}
}}

#[inline(never)]
fn scale(p: &P) -> u64 {{
{scale}
}}

#[inline(never)]
fn extra(p: &P) -> u64 {{
{extra}
}}

fn main() {{
    let p = P {{ v: 7, w: 9 }};
    let add = |x: u64| x + p.area();
    println!("{{}} {{}}", add(scale(&p)), extra(&p));
}}
'''


def const_fn_closure(val, body_const, generic=False):
    """A const that calls a const fn, read by a closure body (or a generic instance's body):
    the const fn's body edit moves the const's value with no change to the reader's source.
    `body_const` adds a const item in another fn's body, so the request takes a full resolve
    (two edit windows) and not the body patch. The fuzzer's small2 seed 11 (2026-10-19): the
    image build kept the closure's old value."""
    k9 = "    const K9: i64 = 5;\n" if body_const else ""
    reader = ("fn use_seed<T: Into<i64>>(n: T) -> i64 {\n    n.into() + SEED\n}\n" if generic else
              "fn use_seed(n: i64) -> i64 {\n    let add_seed = |x: i64| x + SEED;\n    add_seed(n)\n}\n")
    return f'''const fn seed_val() -> i64 {{
    {val}
}}

const SEED: i64 = seed_val();

fn label(n: i64) -> i64 {{
{k9}    n
}}

{reader}
fn main() {{
    println!("{{}} {{}}", label(1), use_seed(10i64));
}}
'''


def b1_case(case, v):
    """Sol's review cases for a type store kept across collects (B1-PLAN.md), version `v`:
    each edit keeps old interned types around while their meaning changes, so a build from
    the kept store must still equal a fresh one-shot. A helper fn's body moves in every
    version too, so each step is a full collect, not a body patch."""
    tail = f"\nfn helper() -> u32 {{\n    {40 + v}\n}}\n"
    if case == "rename":
        name = ["Old", "New", "Old"][v]
        return f"""struct {name} {{
    n: u32,
}}

fn first(v: &[{name}]) -> Option<&{name}> {{
    v.first()
}}

fn main() {{
    let v = vec![{name} {{ n: 7 }}];
    println!("{{}} {{}}", first(&v).map(|x| x.n).unwrap_or(0), helper());
}}
""" + tail
    if case == "params":
        sig = ["fn pick<T, U>(x: T, _y: U) -> T", "fn pick<U, T>(x: T, _y: U) -> T", "fn pick<T, U>(x: T, _y: U) -> T"][v]
        return f"""{sig} {{
    x
}}

fn main() {{
    println!("{{}} {{}}", pick(5u8, "s"), helper());
}}
""" + tail
    if case == "lifetimes":
        sig = ["fn longer<'a, 'b>(x: &'a str, _y: &'b str) -> &'a str", "fn longer<'b, 'a>(x: &'a str, _y: &'b str) -> &'a str",
               "fn longer<'a, 'b>(_x: &'a str, _y: &'b str) -> &'static str"][v]
        body = ["x", "x", '"fixed"'][v]
        return f"""{sig} {{
    {body}
}}

fn main() {{
    let a = String::from("abc");
    println!("{{}} {{}}", longer(&a, "z"), helper());
}}
""" + tail
    if case == "closure":
        ret = ["1u8", "1u16", "1u8"][v]
        return f"""fn run() -> u64 {{
    let f = || {ret};
    f() as u64 + 2
}}

fn main() {{
    println!("{{}} {{}}", run(), helper());
}}
""" + tail
    if case == "hidden":
        ty = ["u8", "u16", "u8"][v]
        return f"""fn make() -> {ty} {{
    200
}}

fn hidden() -> impl std::fmt::Debug {{
    make()
}}

fn main() {{
    println!("{{:?}} {{}} {{}}", hidden(), std::mem::size_of_val(&hidden()), helper());
}}
""" + tail
    if case == "array":
        n = [3, 5, 3][v]
        return f"""const N: usize = {n};

fn total(a: [u8; N]) -> usize {{
    a.len() + a[0] as usize
}}

fn main() {{
    println!("{{}} {{}}", total([1; N]), helper());
}}
""" + tail
    if case == "layout":
        ty = ["u8", "u64", "u8"][v]
        return f"""struct S {{
    x: {ty},
    y: u32,
}}

fn read(s: &S) -> u64 {{
    s.x as u64 + s.y as u64
}}

fn main() {{
    let s = S {{ x: 3, y: 4 }};
    println!("{{}} {{}} {{}}", read(&s), std::mem::size_of::<S>(), helper());
}}
""" + tail
    if case == "instance":
        use = ["let g = G { t: 1u8 }; println!(\"{} {}\", g.t, helper());", "println!(\"{}\", helper());",
               "let g = G { t: 2u8 }; println!(\"{} {}\", g.t, helper());"][v]
        return f"""struct G<T> {{
    t: T,
}}

fn main() {{
    {use}
}}
""" + tail
    raise ValueError(case)


def agg_param(k):
    """Callees in another module take a 24-byte struct (release C: by pointer), a 16-byte pair
    (by value; by pointer under LUST_RELEASE_AGG_PARAM=all) and a 9-byte Option the unwrap fast
    path reads, through direct calls, a fn pointer and a dyn method; each step edits only the
    callees' bodies (`k`)."""
    return f'''mod m {{
    #[derive(Debug, Clone, Copy)]
    pub struct Wide {{
        pub a: u64,
        pub b: u64,
        pub c: u8,
    }}

    #[derive(Debug, Clone, Copy)]
    pub struct Pair {{
        pub a: u64,
        pub b: u64,
    }}

    #[inline(never)]
    pub fn wide(x: Wide) -> u64 {{
        x.a * {k} + x.b + x.c as u64
    }}

    #[inline(never)]
    pub fn pair(x: Pair) -> u64 {{
        x.a + x.b * {k}
    }}

    #[inline(never)]
    pub fn first(o: Option<[u8; 8]>) -> u8 {{
        o.unwrap()[0] + {k}
    }}

    pub trait Area {{
        fn area(&self, x: Wide) -> u64;
    }}

    pub struct Sq;

    impl Area for Sq {{
        fn area(&self, x: Wide) -> u64 {{
            x.a * x.b + {k}
        }}
    }}
}}

fn main() {{
    let w = m::Wide {{ a: 2, b: 3, c: 4 }};
    let f: fn(m::Wide) -> u64 = m::wide;
    let s: &dyn m::Area = &m::Sq;
    println!("{{}} {{}} {{}} {{}} {{}}", m::wide(w), f(w), m::pair(m::Pair {{ a: 5, b: 6 }}), m::first(std::hint::black_box(Some([9u8; 8]))), s.area(w));
}}
'''


def agg_layout(grow):
    """A struct grows from 16 to 24 bytes and an array struct from 8 to 9 (and back), so the
    callees that take them by value change their C parameter passing, while the callers' text
    (`mid`, `main`) stays the same."""
    extra, init, add, n = (",\n    pub c: u8", ", c: 1", " + self.c as u64", 9) if grow else ("", "", "", 8)
    return f'''mod m {{
    #[derive(Debug, Clone, Copy)]
    pub struct S {{
        pub a: u64,
        pub b: u64{extra}
    }}

    impl S {{
        pub fn new(x: u64) -> S {{
            S {{ a: x, b: x + 1{init} }}
        }}

        pub fn total(self) -> u64 {{
            self.a + self.b{add}
        }}
    }}

    #[derive(Debug, Clone, Copy)]
    pub struct T {{
        pub a: [u8; {n}],
    }}

    impl T {{
        pub fn new(x: u8) -> T {{
            T {{ a: [x; {n}] }}
        }}
    }}

    #[inline(never)]
    pub fn use_s(s: S) -> u64 {{
        s.total()
    }}

    #[inline(never)]
    pub fn use_t(t: T) -> u64 {{
        t.a.iter().map(|&b| b as u64).sum()
    }}

    #[inline(never)]
    pub fn pass(s: S, t: T) -> (S, T) {{
        (s, t)
    }}
}}

fn mid(x: u64) -> u64 {{
    let s = m::S::new(x);
    let f: fn(m::S) -> u64 = m::use_s;
    let (s2, t2) = m::pass(s, m::T::new(x as u8));
    m::use_s(s) + f(s2) + m::use_t(t2)
}}

fn main() {{
    println!("{{}} {{}}", mid(3), mid(7));
}}
'''


def later_patch(v):
    """A program on the resolver path (std imports) whose first image build is a full resolve and
    whose later builds patch: a body edit (v1), a header edit (v2, the item patch), a const fn
    body edit (v3), then a new item again (v4, a full resolve). A one-shot image build leaves the
    patch base's span probe and const reach to the first patch that needs them (resolved_setup);
    each later answer must still equal a fresh one-shot's."""
    body = ["a + b", "a * 2 + b", "a * 2 + b", "a * 2 + b", "a * 2 + b"][v]
    sig = ["fn mix(a: u64, b: u64) -> u64", "fn mix(a: u64, b: u64) -> u64", "fn mix(a: u64, b: u32) -> u64",
           "fn mix(a: u64, b: u32) -> u64", "fn mix(a: u64, b: u32) -> u64"][v]
    b = "b" if v < 2 else "b as u64"
    body = body.replace("b", b) if v >= 2 else body
    seed = ["3", "3", "3", "7", "7"][v]
    extra = "" if v < 4 else "\nfn added() -> u64 {\n    11\n}\n"
    call = "mix(10, 4)" if v < 4 else "mix(10, 4) + added()"
    return f"""use std::collections::HashMap;

const fn seed() -> u64 {{
    {seed}
}}

const SEED: u64 = seed();

{sig} {{
    {body} + SEED
}}

fn table() -> HashMap<u64, u64> {{
    let mut m = HashMap::new();
    m.insert(1, {call});
    m
}}
{extra}
fn main() {{
    let t = table();
    println!("{{}} {{}}", t[&1], SEED);
}}
"""


def auto_shape(v):
    """A generic Adt's auto trait answer (2026-10-20: a closure template reads the Adt's auto
    shape, not its deep Structure). `Service<P>` holds `Holder`, which holds `Inner`; a closure
    template needs `Arc<Service<P>>: Send`, so `Service<P>: Sync`. v1 adds a `Cell` inside
    `Inner` (Service is no longer Sync: the template must reject), v2 restores it, v3 adds a
    `Cell` inside the argument `Mid` (the caller must reject), v4 restores it."""
    inner = "    n: u32,\n    c: std::cell::Cell<u32>," if v == 1 else "    n: u32,"
    inner_init = "Inner { n: 1, c: std::cell::Cell::new(0) }" if v == 1 else "Inner { n: 1 }"
    mid = "    m: u32,\n    c: std::cell::Cell<u32>," if v == 3 else "    m: u32,"
    mid_init = "Mid { m: 3, c: std::cell::Cell::new(0) }" if v == 3 else "Mid { m: 3 }"
    return f"""use std::sync::Arc;

struct Inner {{
{inner}
}}

struct Holder {{
    inner: Inner,
}}

struct Mid {{
{mid}
}}

struct Service<P = Mid> {{
    p: P,
    holder: Holder,
    k: u32,
}}

fn run<F: FnOnce() -> u32 + Send + 'static>(f: F) -> u32 {{
    std::thread::spawn(f).join().unwrap()
}}

fn handler<P: Send + Sync + 'static>(s: Arc<Service<P>>) -> u32 {{
    let t = s.clone();
    run(move || t.k + t.holder.inner.n + {v})
}}

fn main() {{
    let s = Arc::new(Service {{ p: {mid_init}, holder: Holder {{ inner: {inner_init} }}, k: 2 }});
    println!("{{}} {{}}", handler(s.clone()), s.p.m);
}}
"""


def glue_shape(v):
    """A generic Adt's derive glue answer (2026-10-20: a template reads the Adt's glue shape, not
    its deep Structure). `#[derive(Clone)] Slot<T>` holds a `Holder` and a `Cell<Flag>`; a generic
    fn clones a `Slot<T>` inside a closure. v1: `Holder` loses `Clone` (rejected), v2 restores it,
    v3: `Flag` loses `Copy` (a `Cell<Flag>` no longer clones: rejected), v4 restores it."""
    holder = "#[derive(Clone)]\n" if v != 1 else ""
    flag = "#[derive(Clone, Copy)]" if v != 3 else "#[derive(Clone)]"
    return f"""use std::cell::Cell;

{holder}struct Holder {{
    n: u32,
}}

{flag}
struct Flag {{
    on: bool,
}}

#[derive(Clone)]
struct Slot<T> {{
    t: T,
    holder: Holder,
    flag: Cell<Flag>,
}}

fn twice<T: Clone>(s: &Slot<T>) -> (Slot<T>, u32) {{
    let make = || s.clone();
    let a = make();
    let n = a.holder.n + {v};
    (a, n)
}}

fn main() {{
    let s = Slot {{ t: 7u64, holder: Holder {{ n: 2 }}, flag: Cell::new(Flag {{ on: true }}) }};
    let (c, n) = twice(&s);
    println!("{{}} {{}} {{}}", c.t, n, c.flag.get().on);
}}
"""



def hole_lang(v, generic=False):
    """Sol blocker 1 in fn-body form: `k::<_>()` infers T = i32 from i32's one PartialEq impl;
    v1 adds `impl PartialEq<G> for i32` (the call is ambiguous now) with f unchanged."""
    imp = "impl PartialEq<G> for i32 {\n    fn eq(&self, _: &G) -> bool {\n        true\n    }\n}\n" if v == 1 else ""
    f = "fn f<U>() -> u8 {\n    k::<_>()\n}\n" if generic else "fn f() -> u8 {\n    k::<_>()\n}\n"
    call = "f::<u8>()" if generic else "f()"
    return f"""struct G;

fn k<T>() -> u8
where
    i32: PartialEq<T>,
{{
    7
}}

{f}
{imp}
fn main() {{
    let _g = G;
    println!("{{}}", {call});
}}
"""


def hole_module(v, kind):
    """Sol blocker 2 in fn-body form: m's fn needs an import (a trait for a method, or a const
    binding); v1 inserts `mod n` with that import before m and removes it from m, so m's old
    module id may name n now. A cold check rejects f; v2 restores v0."""
    use = {"method": "use crate::defs::T;", "binding": "use crate::defs::K;", "generic": "use crate::defs::T;"}[kind]
    body = {"method": "crate::S.k()", "binding": "K", "generic": "crate::S.k()"}[kind]
    sig = "pub fn f<U>() -> u8" if kind == "generic" else "pub fn f() -> u8"
    call = "m::f::<u8>()" if kind == "generic" else "m::f()"
    n = f"mod n {{\n    #[allow(unused_imports)]\n    {use}\n}}\n\n" if v == 1 else ""
    m_use = "" if v == 1 else f"    {use}\n"
    return f"""mod defs {{
    pub trait T {{
        fn k(&self) -> u8;
    }}
    pub const K: u8 = 3;
}}

pub struct S;

impl defs::T for S {{
    fn k(&self) -> u8 {{
        1
    }}
}}

{n}mod m {{
{m_use}    {sig} {{
        {body}
    }}
}}

fn main() {{
    println!("{{}}", {call});
}}
"""


def hole_limit(limit):
    """Sol blocker 4 in fn-body form: a local of 128 nested Options under a recursion limit."""
    return f"""#![recursion_limit = "{limit}"]

type O2<T> = Option<Option<T>>;
type O4<T> = O2<O2<T>>;
type O8<T> = O4<O4<T>>;
type O16<T> = O8<O8<T>>;
type O32<T> = O16<O16<T>>;
type O64<T> = O32<O32<T>>;
type O128<T> = O64<O64<T>>;

fn f() -> u8 {{
    let _x: O128<u8> = None;
    5
}}

fn main() {{
    println!("{{}}", f());
}}
"""


def agg_ret(k):
    """`main` calls fns in another module that return 12-byte `Option<(u32, u32)>` (release C:
    through the out pointer) and a 16-byte pair of u64 (by value, but through the out pointer
    under LUST_RELEASE_AGG_RET=all); each step edits only the callees' bodies (`k`), so a cached
    `main` keeps its calls unless the ABI mode is part of the cache key."""
    return f'''mod m {{
    #[derive(Debug, Clone, Copy)]
    pub struct W {{
        pub a: u64,
        pub b: u64,
    }}

    #[inline(never)]
    pub fn pair(x: u32) -> Option<(u32, u32)> {{
        if x == 0 {{ None }} else {{ Some((x + {k}, x * {k})) }}
    }}

    #[inline(never)]
    pub fn wide(x: W) -> W {{
        W {{ a: x.b + {k}, b: x.a * {k} }}
    }}
}}

fn main() {{
    let f: fn(u32) -> Option<(u32, u32)> = m::pair;
    let w = m::wide(m::W {{ a: 2, b: 3 }});
    println!("{{:?}} {{:?}} {{}} {{}}", m::pair(5), f(0), w.a, w.b);
}}
'''

def hole2(case, form, v):
    """Sol's const-body review round 2, each history in a fn body or a const body: lint (a
    crate-level allow removed), hygiene (a macro's binder loses its `$`), alias (a type alias
    inside a type argument changes), length (a named const array length changes), drop (a
    recursive field type behind a Box makes drop checking overflow). v1 must be rejected."""
    head, item = "", ""
    if case == "lint":
        head = "#![allow(overflowing_literals)]\n" if v == 0 else ""
        stmt = "let _: u8 = 256;"
    elif case == "hygiene":
        binder = "$x" if v == 0 else "x"
        item = f"macro_rules! m {{\n    ($x:ident) => {{{{\n        let {binder}: u8 = 1;\n        $x\n    }}}};\n}}\n\n"
        stmt = "let _: u8 = m!(x);"
    elif case == "alias":
        item = f"mod m {{\n    pub type A = {'u8' if v == 0 else 'u16'};\n}}\n\n"
        stmt = "let _: Option<m::A> = Some(1u8);"
    elif case == "length":
        item = f"const N: usize = {1 if v == 0 else 2};\n\n"
        stmt = "let _: [u8; N] = [0; 1];"
    else:
        inner = "T" if v == 0 else "(T, T)"
        item = f"struct S {{\n    g: Grow<u8>,\n}}\n\nstruct Grow<T> {{\n    value: T,\n    next: Option<Box<Grow<{inner}>>>,\n}}\n\n"
        stmt = "let _x: Option<Box<S>> = None;"
    if form == "fn":
        body = f"fn f() -> u8 {{\n    if false {{\n        {stmt}\n    }}\n    7\n}}\n\nfn main() {{\n    println!(\"{{}}\", f());\n}}\n"
    else:
        body = f"const C: u8 = {{\n    if false {{\n        {stmt}\n    }}\n    7\n}};\n\nfn main() {{\n    println!(\"{{}}\", C);\n}}\n"
    return head + item + body


LIFETIME_HEAD = """use std::rc::Rc;

trait Tr {
    fn v(&self) -> u32;
}

struct K(u32);

impl Tr for K {
    fn v(&self) -> u32 {
        self.0
    }
}

struct M<T> {
    t: T,
    n: u32,
}

impl<T> M<T> {
    #[inline(never)]
    fn get(&self) -> u32 {
        self.n
    }
}

"""


def lang_hasher(write_u32):
    """A written `Hasher` impl gains and loses a method while generic bodies (`Option`'s
    methods over a struct that holds a map with that hasher) are kept: the hasher's impl
    reaches them through the struct's layout fingerprint (fx-coldmono 10:39, the `LangImpls`
    reads of tsgo's hashers). The map's order follows the hasher."""
    extra = """
    fn write_u32(&mut self, n: u32) {
        self.0 = (n % 7) as u64;
    }""" if write_u32 else ""
    return f"""use std::collections::HashMap;
use std::hash::{{BuildHasherDefault, Hasher}};

#[derive(Default)]
struct KeyHasher(u64);

impl Hasher for KeyHasher {{
    fn finish(&self) -> u64 {{
        self.0
    }}
    fn write(&mut self, b: &[u8]) {{
        for &x in b {{
            self.0 = self.0.wrapping_mul(31).wrapping_add(x as u64);
        }}
    }}{extra}
}}

struct Table {{
    m: HashMap<u32, u32, BuildHasherDefault<KeyHasher>>,
    n: u32,
}}

#[inline(never)]
fn keys(t: &Table) -> Vec<u32> {{
    t.m.keys().copied().collect()
}}

fn main() {{
    let mut t = Table {{ m: HashMap::default(), n: 2 }};
    for i in 0..12u32 {{
        t.m.insert(i * 5, i);
    }}
    let o = Some(t);
    let n = o.as_ref().map(|t| t.m.len() as u32 + t.n).unwrap_or(0);
    println!("{{}} {{}} {{:?}}", o.is_some(), n, o.as_ref().map(keys));
}}
"""


def late_sources(v):
    """Step `v` of sources-late: a crate of two files (main.rs, m.rs) whose answer changes at
    every step (a body edit, a new fn, a removed one)."""
    m = ["pub fn v() -> u64 { 1 }\n", "pub fn v() -> u64 { 2 }\n", "pub fn v() -> u64 { 2 }\npub fn w() -> u64 { 40 }\n",
         "pub fn v() -> u64 { 3 }\n"][v]
    call = "m::v() + m::w()" if v == 2 else "m::v()"
    return {"main.rs": f"mod m;\n\nfn main() {{\n    println!(\"{{}}\", {call});\n}}\n", "m.rs": m}


def corpus():
    """(name, [sources]) of every sequence."""
    seqs = [
        ("nested-grow", [
            nested("    a: u64,", "T { a: 1 }", "s.t.a"),
            nested("    a: u64,\n    b: u64,", "T { a: 1, b: 2 }", "s.t.a + s.t.b"),
            nested("    a: u64,\n    b: u64,\n    c: u32,", "T { a: 1, b: 2, c: 3 }", "s.t.a + s.t.b + s.t.c as u64"),
            nested("    a: u64,", "T { a: 1 }", "s.t.a"),
        ]),
        ("enum-payload", [
            enum_payload("    A(u8),\n    B(u32, u16),", "        E::A(x) => x as u64,\n        E::B(a, b) => a as u64 + b as u64,", ""),
            enum_payload("    A(u8),\n    B(u32, u16),\n    C(u64, u64),",
                         "        E::A(x) => x as u64,\n        E::B(a, b) => a as u64 + b as u64,\n        E::C(a, b) => a + b,",
                         "    let c = S { e: E::C(1, 2), y: 3 };\n    println!(\"{}\", read_e(&c) + read_y(&c));"),
            enum_payload("    C(u64, u64),\n    A(u8),\n    B(u32, u16),",
                         "        E::A(x) => x as u64,\n        E::B(a, b) => a as u64 + b as u64,\n        E::C(a, b) => a + b,",
                         "    let c = S { e: E::C(1, 2), y: 3 };\n    println!(\"{}\", read_e(&c) + read_y(&c));"),
        ]),
        ("generic-arg", [
            generic("    a: u32,", "Inner { a: 1 }", "", "", "w.t.a"),
            generic("    a: u32,\n    b: u64,\n    c: u64,", "Inner { a: 1, b: 2, c: 3 }", "", "", "w.t.a as u64 + w.t.b + w.t.c"),
            generic("    a: u32,\n    b: u64,\n    c: u64,", "Inner { a: 1, b: 2, c: 3 }", "    m: [u64; 2],", " m: [5, 6],", "w.t.a as u64 + w.m[1]"),
        ]),
        ("option-wrap", [
            option_wrap("    a: u64,", "T { a: 1 }"),
            option_wrap("    a: u64,\n    b: [u64; 3],", "T { a: 1, b: [2, 3, 4] }"),
        ]),
        ("array-len", [array_len(4), array_len(20), array_len(1)]),
        ("statics", [
            statics("", ""),
            statics("    static A: Cell<u64> = Cell::new(10);",
                    "    A.with(|a| a.set(a.get() + 5));\n    println!(\"{}\", A.with(|a| a.get()));"),
            statics("    static A: Cell<u64> = Cell::new(10);\n    static AA: Cell<u32> = Cell::new(1);",
                    "    A.with(|a| a.set(a.get() + 5));\n    AA.with(|a| a.set(a.get() * 3));\n    println!(\"{} {}\", A.with(|a| a.get()), AA.with(|a| a.get()));"),
        ]),
    ]
    seqs.append(("lang-hasher-edit", [lang_hasher(False), lang_hasher(True), lang_hasher(False)]))
    seqs.append(("lifetime-variant-order", [lifetime_variants(False), lifetime_variants(True), lifetime_variants(False)]))
    seqs.append(("llvm-helper-only-edit", [helper_edit("v1", 1), helper_edit("v2-label", 1), helper_edit("v2-label", 3)]))
    seqs.append(("remove-impl-pgo", [shapes(True), shapes(False), shapes(True)], {"pgo": True}))
    seqs.append(("hot-flip-pgo-copies", [hot_flip(1), hot_flip(0), hot_flip(1)], {"pgo": True, "copies": True, "retrain": True}))
    dt = os.path.join(ROOT, "tests/daemon/dt01_deep_type_edits")
    dt01 = [open(os.path.join(dt, f"v{k}.rs")).read() for k in range(1, 7)]
    seqs.append(("dt01", dt01))
    # A CI job's shapes on the image (check_seq): five builds without emitted C before the compared
    # one, a jump over three versions, and four builds that each start from the second's state.
    seqs.append(("dt01-quiet", dt01, {"quiet": True}))
    seqs.append(("dt01-jump", dt01, {"steps": [0, 1, 5]}))
    seqs.append(("dt01-stale", dt01, {"stale_after": 1}))
    # B1's skipping itself: builds that emit no C skip the units their reads call clean; their
    # programs' output must match the one-shot builds'.
    seqs.append(("dt01-b1-skip", dt01, {"quiet": True, "b1": "skip"}))
    # Recovery (the kept-program reviews): a rejected build, then the state after it. A
    # ("check", src) step runs `lust check` on both sides, on the same image (a mode switch).
    # A const fn edit across skipped image states: the image builds the first and the last
    # state only (`steps`), as a CI disk that missed the states between.
    c = const_fn_closure
    seqs.append(("const-fn-closure", [c(3, False), c(7, False), c(10, True)], {"steps": [0, 2]}))
    seqs.append(("const-fn-closure-each", [c(3, False), c(10, True), c(4, False), c(4, True)]))
    seqs.append(("const-fn-generic", [c(3, False, True), c(7, False, True), c(10, True, True)], {"steps": [0, 2]}))
    # rel-front's discovery case (2026-10-19): a stored discovery entry asks for
    # `make::<[u8; N]>`, and a const fn edit moves N; the entry must replay its reads.
    disc = os.path.join(ROOT, "tests/daemon/cv02_const_fn_value_in_discovery")
    dv = [open(os.path.join(disc, f"v{k}.rs")).read() for k in (1, 2, 3)]
    seqs.append(("const-fn-discovery", dv))
    seqs.append(("const-fn-discovery-jump", dv, {"steps": [0, 2]}))
    # The release C ABI mode (LUST_RELEASE_AGG_RET) switched between steps, with a callee-body
    # edit at each step (`env`: one environment per step, both sides): every unit's calls and
    # definitions must follow the mode of their build. Once with B1's oracle, once skipping.
    ar = [agg_ret(3), agg_ret(4), agg_ret(5)]
    modes = [{}, {"LUST_RELEASE_AGG_RET": "all"}, {}]
    seqs.append(("agg-ret-mode", ar, {"env": modes}))
    seqs.append(("agg-ret-mode-b1-skip", ar, {"env": modes, "quiet": True, "b1": "skip"}))
    # Sol's review cases for a kept type store (B1-PLAN.md), each through three versions.
    for case in ("rename", "params", "lifetimes", "closure", "hidden", "array", "layout", "instance"):
        seqs.append((f"b1-{case}", [b1_case(case, v) for v in range(3)]))
    # The release C ABI parameter mode (LUST_RELEASE_AGG_PARAM) default -> all -> default -> 0 -> default, a
    # callee-body edit at each step; and layout edits across 8/9 and 16/17 bytes under
    # unchanged callers (each mode).
    ap = [agg_param(k) for k in (3, 4, 5, 6, 7)]
    pmodes = [{}, {"LUST_RELEASE_AGG_PARAM": "all"}, {}, {"LUST_RELEASE_AGG_PARAM": "0"}, {}]
    seqs.append(("agg-param-mode", ap, {"env": pmodes}))
    seqs.append(("agg-param-mode-b1-skip", ap, {"env": pmodes, "quiet": True, "b1": "skip"}))
    lay = [agg_layout(False), agg_layout(True), agg_layout(False)]
    seqs.append(("agg-param-layout", lay))
    seqs.append(("agg-param-layout-all", lay, {"env": [{"LUST_RELEASE_AGG_PARAM": "all"}] * 3}))
    # Patches after a full resolve in an image chain (lazy patch base prep, 2026-10-20).
    seqs.append(("later-patch", [later_patch(v) for v in range(5)]))
    seqs.append(("sources-late", [late_sources(v) for v in range(4)], {"late": True}))
    seqs.append(("auto-shape", [auto_shape(v) for v in range(5)]))
    seqs.append(("glue-shape", [glue_shape(v) for v in range(5)]))
    # Stale reuse in fn-body and generic reuse (Sol's const-body review blockers in fn-body
    # form, 2026-10-20; all but the binding and check-then-build ones were false accepts).
    seqs.append(("hole-lang", [hole_lang(v) for v in (0, 1, 0)]))
    seqs.append(("hole-lang-generic", [hole_lang(v, True) for v in (0, 1, 0)]))
    for kind in ("method", "binding", "generic"):
        seqs.append((f"hole-module-{kind}", [hole_module(v, kind) for v in (0, 1, 0)]))
    seqs.append(("hole-limit", [hole_limit(256), hole_limit(128), hole_limit(256)]))
    seqs.append(("hole-check-then-build", [("check", hole_limit(128)), hole_limit(128)]))
    # Sol's const-body review round 2 histories, in fn and const bodies (2026-10-20).
    for case in ("lint", "hygiene", "alias", "length", "drop"):
        for form in ("fn", "const"):
            seqs.append((f"hole2-{case}-{form}", [hole2(case, form, v) for v in (0, 1, 0)]))
    r = recovery
    seqs.append(("recover-body", [r("ok1"), r("body_err"), r("ok2")]))
    seqs.append(("recover-two-errors", [r("ok1"), r("body_err"), r("body_err2"), r("ok1")]))
    seqs.append(("recover-type", [r("ok1"), r("type_err"), r("ok2"), r("type_err"), r("ok1")]))
    seqs.append(("recover-mixed", [("check", r("ok1")), r("ok1"), ("check", r("body_err")), r("body_err"),
                                   ("check", r("ok2")), r("type_err"), ("check", r("ok1")), r("ok2")]))
    return seqs


def units(d):
    """The unit files `--emit-c` wrote into `d`, by content hash."""
    out = {}
    for f in sorted(os.listdir(d)):
        b = open(os.path.join(d, f), "rb").read()
        out.setdefault(hashlib.sha256(b).hexdigest(), []).append((f, b))
    return out


def unit_diff(dw, dc):
    """'' when the two unit sets are equal by content, else a short account."""
    w, c = units(dw), units(dc)
    only_w = [x for h, fs in w.items() for x in fs[len(c.get(h, [])):]]
    only_c = [x for h, fs in c.items() for x in fs[len(w.get(h, [])):]]
    if not only_w and not only_c:
        return ""
    msg = f"units differ: image-only {[f for f, _ in only_w][:4]}, one-shot-only {[f for f, _ in only_c][:4]}"
    by_name = {f: b for f, b in only_c}
    for f, b in only_w[:2]:
        if f in by_name:
            a, o = b.split(b"\n"), by_name[f].split(b"\n")
            n = next((i for i in range(min(len(a), len(o))) if a[i] != o[i]), min(len(a), len(o)))
            msg += f"; {f} line {n + 1}: image {a[n][:160] if n < len(a) else b''!r} vs one-shot {o[n][:160] if n < len(o) else b''!r}"
    return msg


def exact_units(da, db):
    """'' when two unit directories hold the same files, names and bytes, else the first difference."""
    a, b = sorted(os.listdir(da)), sorted(os.listdir(db))
    if a != b:
        return f"unit names differ: {sorted(set(a) ^ set(b))[:4]}"
    for f in a:
        if open(os.path.join(da, f), "rb").read() != open(os.path.join(db, f), "rb").read():
            return f"unit {f} differs"
    return ""


def run_bin(path, cwd):
    try:
        r = subprocess.run([path], cwd=cwd, capture_output=True, timeout=RUN_TIMEOUT)
        return (r.returncode, r.stdout)
    except subprocess.TimeoutExpired:
        return ("timeout", b"")


def build(cmd, env, cwd, late=None):
    """(exit status or 'hang', stderr). The image's own `lust: front state` lines are dropped:
    the rest is the answer, which must equal the one-shot's. `late` runs 0.3 s after the build
    starts (sources-late: it writes the step's files, then the ready file)."""
    p = subprocess.Popen(cmd, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if late:
        time.sleep(0.3)
        late()
    try:
        _, err = p.communicate(timeout=BUILD_TIMEOUT)
    except subprocess.TimeoutExpired:
        p.kill()
        p.communicate()
        return ("hang", "")
    err = err.decode(errors="replace")
    return (p.returncode, "".join(l for l in err.splitlines(keepends=True) if not l.startswith("lust: front state")))


def put(d, src):
    """Writes one step's sources into `d`: main.rs, or each file of a dict."""
    for name, text in (src if isinstance(src, dict) else {"main.rs": src}).items():
        with open(os.path.join(d, name), "w") as f:
            f.write(text)


def head(err):
    """The end of a stderr text, on one line, for a message."""
    return err[-400:].replace("\n", " ")


def first_diff(a, b):
    """The first differing line of two texts, for a message."""
    x, y = a.splitlines(), b.splitlines()
    n = next((i for i in range(min(len(x), len(y))) if x[i] != y[i]), min(len(x), len(y)))
    return f"line {n + 1}: image {x[n][:200] if n < len(x) else ''!r} vs one-shot {y[n][:200] if n < len(y) else ''!r}"


VISIT = re.compile(r"\b(path|why|kept|pool)=(\S+)")


def last_path(log, seen):
    """The image request's front-end path from the visits log (`lust-visits:`, LUST_VISITS_LOG):
    'path/why/kept' of the first line after `seen` bytes, or '-' when the request logged none."""
    if not os.path.exists(log):
        return "-", seen
    with open(log) as f:
        f.seek(seen)
        new = f.read()
    lines = [l for l in new.splitlines() if l.startswith("lust-visits:")]
    if not lines:
        return "-", seen + len(new)
    v = dict(VISIT.findall(lines[-1]))
    pool = "/pool" if v.get("pool", "-") != "-" else ""
    return f"{v.get('path', '-')}/{v.get('why', '-')}/{v.get('kept', '-')}{pool}", seen + len(new)


def env_check(args, work):
    """env-in-image (see the header): the problems found, as lines."""
    d = os.path.join(work, "env-in-image")
    os.makedirs(d, exist_ok=True)
    secret, read = "lust-secret-" + os.urandom(8).hex(), "lust-read-" + os.urandom(8).hex()
    drop = ("LUST_DAEMON_SOCKET", "LUST_CACHE_DIR", "LUST_NO_CACHE", "LUST_FRONT_STATE", "LUST_RELEASE_BACKEND", "LUST_FRONT_ZSTD")
    env = {k: v for k, v in os.environ.items() if k not in drop}
    env.update(LUST_RELEASE_C="1", LUST_NO_DAEMON="1", LUST_FRONT_STATE=os.path.join(d, "front"), LUST_FRONT_SAVE="background",
               LUST_CACHE_DIR=os.path.join(d, "cache"), LUST_FRONT_ZSTD="0", FRONT_CHECK_SECRET=secret, FRONT_CHECK_READ=read)
    problems = []
    for k, tail in enumerate(["", " + 1"]):
        put(d, 'fn main() {\n    println!("{}", env!("FRONT_CHECK_READ").len()' + tail + ');\n}\n')
        rc, err = build([LUST, "build", "main.rs", "--release", "--target", args.host, "-o", "app.bin"], env, d)
        if rc != 0:
            return [f"env-in-image: build {k + 1} failed ({rc}): {head(err)}"]
        lock = os.path.join(d, "front", "lock")
        if os.path.exists(lock):
            subprocess.run(["flock", lock, "true"], timeout=BUILD_TIMEOUT)
        found = {secret: [], read: []}
        for top in ("front", "cache"):
            for root, _, files in os.walk(os.path.join(d, top)):
                for f in files:
                    with open(os.path.join(root, f), "rb") as fh:
                        data = fh.read()
                    for m in found:
                        if m.encode() in data:
                            found[m].append(os.path.relpath(os.path.join(root, f), d))
        if found[secret]:
            problems.append(f"env-in-image: after build {k + 1} an env var no expansion reads is in {', '.join(found[secret])}")
        if not found[read]:
            problems.append(f"env-in-image: after build {k + 1} no file holds the value env! read: the scan cannot see the image")
    return problems


def check_seq(name, srcs, backend, args, work, opts):
    """One sequence on one backend: (name, backend, problems, steps, paths); steps is 0 when
    skipped; paths holds the image side's front-end path of each request (`last_path`)."""
    d = os.path.join(work, f"{name}-{backend}")
    os.makedirs(d, exist_ok=True)
    drop = ("LUST_DAEMON_SOCKET", "LUST_CACHE_DIR", "LUST_NO_CACHE", "LUST_THREADS", "LUST_FRONT_STATE", "LUST_RELEASE_BACKEND",
            "LUST_PGO_USE", "LUST_PGO_GEN", "LUST_PGO_CC", "LUST_PROFDATA")
    base = {k: v for k, v in os.environ.items() if k not in drop}
    base.update(LUST_RELEASE_C="1", LUST_NO_DAEMON="1")
    if backend == "llvm":
        base["LUST_RELEASE_BACKEND"] = "llvm"
        if args.llvm_cc:
            base["LUST_CC"] = args.llvm_cc
    rel = ["--release", "--target", args.host]
    prof = os.path.join(d, "train.profdata")

    def train(src):
        """The profile both sides use (`prof`): an instrumented one-shot build of `src`, run
        once. None, or what failed."""
        with open(os.path.join(d, "main.rs"), "w") as f:
            f.write(src)
        raw = os.path.join(d, "raw")
        shutil.rmtree(raw, ignore_errors=True)
        gen = {k: v for k, v in base.items() if not k.startswith(("LUST_PGO_USE", "LUST_PGO_COPIES"))}
        gen.update(LUST_NO_CACHE="1", LUST_THREADS="4", LUST_PGO_GEN="1", LUST_PGO_CC=args.pgo["cc"])
        rg, eg = build([LUST, "build", "main.rs", *rel, "-o", "gen.bin"], gen, d)
        if rg != 0:
            return f"{name} {backend}: the instrumented build failed ({rg}): {head(eg)[:300]}"
        subprocess.run([os.path.join(d, "gen.bin")], cwd=d, env=dict(os.environ, LLVM_PROFILE_FILE=os.path.join(raw, "%p.profraw")),
                       capture_output=True, timeout=RUN_TIMEOUT)
        m = subprocess.run([args.pgo["profdata"], "merge", "-o", prof, raw], capture_output=True, text=True)
        return f"{name} {backend}: llvm-profdata merge failed: {m.stderr[:300]}" if m.returncode != 0 else None

    if opts.get("pgo"):
        if not args.pgo:
            return name, backend, [], 0, []
        err = train(srcs[0])
        if err:
            return name, backend, [err], len(srcs), []
        base.update(LUST_PGO_USE=prof, LUST_PROFDATA=args.pgo["profdata"])
        # `copies`: release C copies chosen by the same profile (LUST_PGO_COPIES; every fn it saw
        # run is hot), in 1 KB units so most calls cross units.
        if opts.get("copies"):
            base.update(LUST_PGO_COPIES=prof, LUST_PGO_COPY_MIN="1", LUST_RELEASE_UNIT_KB="1")
    visits = os.path.join(d, "visits.log")
    img = dict(base, LUST_FRONT_STATE=os.path.join(d, "front"), LUST_FRONT_SAVE="background", LUST_CACHE_DIR=os.path.join(d, "cache"), LUST_THREADS="1",
               LUST_VISITS_LOG=visits)
    # B1 (release C fn units a warm build does not assemble): the image side runs its oracle
    # (LUST_CHECK_UNIT_FP=1: every unit assembles, and one its recorded reads call clean must
    # have the same text), or with `b1` skip the skipping itself (LUST_B1=1).
    img.update({"LUST_B1": "1"} if opts.get("b1") == "skip" else {"LUST_CHECK_UNIT_FP": "1"})
    img["LUST_CHECK_GENREUSE"] = "1"
    # A collect that starts from a kept type store must equal one from a fresh store
    # (LUST_CHECK_B1, B1-PLAN.md).
    img["LUST_CHECK_B1"] = "1"
    one = dict(base, LUST_NO_CACHE="1", LUST_THREADS="4")
    # --chains: a second image chain of the same history at 16 threads, with its own image and
    # caches (the determinism gate): its builds must equal the first chain's unit for unit (names
    # and bytes) and the one-shot reference like the first chain does.
    visits2 = os.path.join(d, "visits2.log")
    img2 = dict(img, LUST_FRONT_STATE=os.path.join(d, "front2"), LUST_CACHE_DIR=os.path.join(d, "cache2"), LUST_THREADS="16",
                LUST_VISITS_LOG=visits2) if args.chains else None
    problems, paths, seen, seen2 = [], [], 0, 0
    locks = [os.path.join(d, "front", "lock"), os.path.join(d, "front2", "lock")]

    def saved():
        """Waits for the last background image saves."""
        for lock in locks:
            if os.path.exists(lock):
                subprocess.run(["flock", lock, "true"], timeout=BUILD_TIMEOUT)

    # CI job shapes (opts): `steps`, the steps the image side builds (a jump skips the rest);
    # `quiet`, emitted C only at the last step (a CI job's builds emit none); `stale_after`, a
    # step whose state (image and cache) every later step starts from again (a CI disk whose
    # commits did not land: merges 10, 11 and 12 each built from merge 9's state).
    order = opts.get("steps", list(range(len(srcs))))
    snap = None
    for step in order:
        where = f"{name} {backend} step {step + 1}"
        # `retrain`: a profile of the second step replaces the first at the same path, so the
        # image side must see the new hot set (B1 keys units on it, not on the path).
        if opts.get("retrain") and step == 1:
            err = train(srcs[step][1] if isinstance(srcs[step], tuple) else srcs[step])
            if err:
                problems.append(err)
                break
        # A ("check", src) step checks instead of building, on the same image.
        cmd, src = srcs[step] if isinstance(srcs[step], tuple) else ("build", srcs[step])
        # `late`: the image build's files arrive after it starts (`build`), as a CI checkout's.
        ready = os.path.join(d, "ready")
        late = None
        if opts.get("late") and cmd == "build":
            if os.path.exists(ready):
                os.remove(ready)
            late = lambda: (put(d, src), open(ready, "w").close())
        else:
            put(d, src)
        if snap:
            saved()
            for sub in ("front", "cache", "front2", "cache2"):
                shutil.rmtree(os.path.join(d, sub), ignore_errors=True)
                if os.path.exists(os.path.join(snap, sub)):
                    shutil.copytree(os.path.join(snap, sub), os.path.join(d, sub), symlinks=True)
        if cmd == "check":
            rw, ew = build([LUST, "check", "main.rs"], img, d)
            rc, ec = build([LUST, "check", "main.rs"], one, d)
            path, seen = last_path(visits, seen)
            paths.append(path)
            if rw != rc:
                problems.append(f"{where} (check): status image {rw} vs one-shot {rc}: {head(ew if rw else ec)[:300]}")
            elif ew != ec:
                problems.append(f"{where} (check): stderr differs, {first_diff(ew, ec)}")
            if img2:
                r2, e2 = build([LUST, "check", "main.rs"], img2, d)
                path2, seen2 = last_path(visits2, seen2)
                paths.append("chain2 " + path2)
                if (r2, e2) != (rc, ec):
                    problems.append(f"{where} (check): chain 2 status {r2} vs one-shot {rc}, stderr {first_diff(e2, ec)}")
            if problems:
                break
            continue
        emit = not opts.get("quiet") or step == order[-1]
        for side in ("ew", "ec", "ew2"):
            shutil.rmtree(os.path.join(d, side), ignore_errors=True)
            os.makedirs(os.path.join(d, side))
        ce = lambda side: ["--emit-c", os.path.join(d, side, "t")] if emit else []
        # `env`: this step's extra environment, on every side (both image chains and one-shot).
        extra = opts["env"][step] if opts.get("env") else {}
        knob = {"LUST_SOURCES_READY": ready} if late else {}
        rw, ew = build([LUST, "build", "main.rs", *rel, "-o", "image.bin", *ce("ew")], dict(img, **extra, **knob), d, late)
        rc, ec = build([LUST, "build", "main.rs", *rel, "-o", "one.bin", *ce("ec")], dict(one, **extra), d)
        path, seen = last_path(visits, seen)
        paths.append(path)
        if rw != rc:
            problems.append(f"{where}: build status image {rw} vs one-shot {rc}: {head(ew if rw else ec)[:300]}")
            break
        # Rejected and accepted builds alike: the same diagnostics, in the same order (the
        # progress line names each side's own output file).
        ew, ec = ew.replace("image.bin", "OUT"), ec.replace("one.bin", "OUT")
        if ew != ec:
            problems.append(f"{where}: stderr differs, {first_diff(ew, ec)}")
            break
        if img2:
            r2, e2 = build([LUST, "build", "main.rs", *rel, "-o", "image2.bin", *ce("ew2")], dict(img2, **extra), d)
            path2, seen2 = last_path(visits2, seen2)
            paths.append("chain2 " + path2)
            e2 = e2.replace("image2.bin", "OUT")
            if (r2, e2) != (rc, ec):
                problems.append(f"{where}: chain 2 status {r2} vs one-shot {rc}, stderr {first_diff(e2, ec)}")
                break
        if opts.get("stale_after") == step:
            saved()
            snap = os.path.join(d, "snap")
            for sub in ("front", "cache", "front2", "cache2"):
                if os.path.exists(os.path.join(d, sub)):
                    shutil.copytree(os.path.join(d, sub), os.path.join(snap, sub), symlinks=True)
        if rw != 0:
            continue
        diff = unit_diff(os.path.join(d, "ew"), os.path.join(d, "ec")) if emit else ""
        if diff:
            problems.append(f"{where}: {diff}")
        if img2 and emit:
            # Two chains of one history: the same unit files, names and bytes.
            same = exact_units(os.path.join(d, "ew"), os.path.join(d, "ew2"))
            if same:
                problems.append(f"{where}: chain 1 vs chain 2: {same}")
        ow, oc = run_bin(os.path.join(d, "image.bin"), d), run_bin(os.path.join(d, "one.bin"), d)
        if ow != oc:
            problems.append(f"{where}: output image {ow!r} vs one-shot {oc!r}"[:600])
        if img2 and run_bin(os.path.join(d, "image2.bin"), d) != oc:
            problems.append(f"{where}: output of chain 2 differs from the one-shot")
        if problems:
            break
    # The last background image save ends before the directory goes.
    saved()
    if problems and args.keep:
        shutil.copytree(d, os.path.join(args.keep, f"{name}-{backend}"), dirs_exist_ok=True)
    return name, backend, problems, len(srcs), paths


def is_clang(cmd):
    try:
        return "clang" in subprocess.run([cmd, "--version"], capture_output=True, text=True).stdout
    except OSError:
        return False


def pgo_tools():
    """{cc, profdata} for the PGO sequences: zig's LLVM major in both, and a profile runtime that
    links; None when any is missing."""
    if not shutil.which("zig"):
        return None
    v = subprocess.run(["zig", "cc", "--version"], capture_output=True, text=True).stdout
    m = re.search(r"clang version (\d+)", v)
    if not m:
        return None
    major = m.group(1)
    cc = os.environ.get("LUST_PGO_CC") or shutil.which(f"clang-{major}")
    pd = os.environ.get("LUST_PROFDATA") or shutil.which(f"llvm-profdata-{major}")
    if not cc or not pd:
        return None
    with tempfile.TemporaryDirectory() as t:
        src = os.path.join(t, "p.c")
        with open(src, "w") as f:
            f.write("int main(void) { return 0; }\n")
        if subprocess.run([cc, "-fprofile-generate", src, "-o", os.path.join(t, "p")], capture_output=True).returncode != 0:
            return None
    return {"cc": cc, "profdata": pd, "major": major}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--jobs", type=int, default=min(14, os.cpu_count() or 4))
    ap.add_argument("--keep", help="copy each failing sequence's directory here")
    ap.add_argument("--only", help="run only the sequences whose name has this text")
    ap.add_argument("--chains", action="store_true", help="a second image chain at 16 threads (the determinism gate)")
    args = ap.parse_args()
    if not os.access(LUST, os.X_OK):
        print(f"error: {LUST} not built. Run scripts/build.sh first.")
        return 2
    v = subprocess.run(["rustc", "-vV"], capture_output=True, text=True).stdout
    args.host = next((l.split()[1] for l in v.splitlines() if l.startswith("host:")), "x86_64-unknown-linux-gnu")
    # The LLVM units need a clang: zig cc is one, else cc when it is clang, else clang.
    args.llvm_cc = None
    if not shutil.which("zig") and not os.environ.get("LUST_CC") and not is_clang("cc"):
        if not shutil.which("clang"):
            print("error: the LLVM backend needs clang (or zig) on PATH")
            return 2
        args.llvm_cc = "clang"
    args.pgo = None if args.llvm_cc else pgo_tools()
    seqs = [s if len(s) == 3 else (*s, {}) for s in corpus() if not args.only or args.only in s[0]]
    if any(o.get("pgo") for _, _, o in seqs) and not args.pgo:
        print("front-check: PGO sequences skipped: they need zig, and clang and llvm-profdata of zig's LLVM major with a profile runtime")
    work = tempfile.mkdtemp(prefix="front-check.", dir=os.environ.get("TMPDIR"))
    jobs = [(n, srcs, b, o) for n, srcs, o in seqs for b in ("c", "llvm")]
    t0 = time.time()
    bad = steps = 0
    cover = {}
    with ThreadPoolExecutor(max(1, args.jobs)) as pool:
        for name, backend, problems, n, paths in pool.map(lambda j: check_seq(j[0], j[1], j[2], args, work, j[3]), jobs):
            steps += n
            for p in paths:
                cover[p] = cover.get(p, 0) + 1
            for p in problems:
                bad += 1
                print(f"front-check: DIFFERENT {p}")
    if not args.only or args.only in "env-in-image":
        steps += 2
        for p in env_check(args, work):
            bad += 1
            print(f"front-check: DIFFERENT {p}")
    # Path coverage (path/why/kept of each image request): a green run whose every
    # interesting step refused to the full collect proves little about a fast path.
    print("front-check: image paths: " + ", ".join(f"{k} {v}" for k, v in sorted(cover.items(), key=lambda kv: -kv[1])))
    shutil.rmtree(work, ignore_errors=True)
    print(f"front-check: {steps} steps, {bad} different ({len(seqs)} sequences, release C and LLVM, image vs one-shot, {time.time() - t0:.0f} s)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
