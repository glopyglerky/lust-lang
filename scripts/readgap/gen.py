#!/usr/bin/env python3
"""Read-gap histories: warm-vs-cold differential runs aimed at the reads type-level code makes.

Each history is a daemon scenario (tests/daemon layout): v1 a valid program, v2 one item-level
edit whose effect reaches an unchanged body only through type-level code (a path inside a
generic argument, an array length, an associated type, impl selection, an auto trait, a layout,
an attribute, a macro, a glob, a Drop or Copy change, const evaluation), v3 back to v1. The
runner (scripts/test-daemon-scenarios.py) checks and runs each version warm and cold and with
rustc: warm must equal cold on accept or reject and on run stdout, so a stale reuse shows up
as a false accept or as wrong output.

Usage: gen.py --seed N --count K --out DIR   (writes DIR/rg-<family>-<k>/v1.rs, v2.rs, v3.rs)
       gen.py --list
A seed picks, per history, the family, the values in it, and the form that holds the body
(a plain fn, a generic fn's instance, a closure, a method), so one family covers several
paths through the reuse code (plain bodies, instance bodies, closure bodies). About 40% of
the histories split the program: the body in body.rs, the edited items in main.rs."""
import argparse
import os
import random
import re


def wrap(form, ret, body, rng):
    """`body` (statements and a tail of type `ret`) as fn `f`, called from main with its
    value printed with `{:?}`."""
    if form == "plain":
        return f"fn f() -> {ret} {{\n{body}\n}}\n", "f()"
    if form == "generic":
        return f"fn f<Z: Copy>(_z: Z) -> {ret} {{\n{body}\n}}\n", "f(1u8)"
    if form == "closure":
        return f"fn f() -> {ret} {{\n    let c = || {{\n{body}\n    }};\n    c()\n}}\n", "f()"
    return f"struct Holder;\n\nimpl Holder {{\n    fn f(&self) -> {ret} {{\n{body}\n    }}\n}}\n", "Holder.f()"


def program(items, form, ret, body, rng, extra_main=""):
    fdef, call = wrap(form, ret, body, rng)
    return f"{items}\n{fdef}\nfn main() {{\n{extra_main}    println!(\"{{:?}}\", {call});\n}}\n"


# Each family: rng -> (items_v1, items_v2, ret, body). The body text is the same in both
# versions; only the items differ.
FAMILIES = {}


def family(cls):
    def reg(fn):
        FAMILIES[fn.__name__] = (cls, fn)
        return fn
    return reg


# ---- paths inside generic arguments and defaults ----
@family("generic-args")
def alias_in_arg(rng):
    a, b = rng.choice([("u8", "u16"), ("u16", "u32"), ("i8", "i64")])
    wrapper = rng.choice(["Option<m::A>", "Vec<m::A>", "(m::A, u8)", "[m::A; 1]"])
    init = {"Option<m::A>": "Some(120)", "Vec<m::A>": "vec![120]", "(m::A, u8)": "(120, 0)", "[m::A; 1]": "[120]"}[wrapper]
    get = {"Option<m::A>": "x.unwrap()", "Vec<m::A>": "x[0]", "(m::A, u8)": "x.0", "[m::A; 1]": "x[0]"}[wrapper]
    items = "mod m {{\n    pub type A = {};\n}}\n"
    body = f"    let x: {wrapper} = {init};\n    let y = {get};\n    format!(\"{{}} {{}}\", y.wrapping_mul(3), std::mem::size_of_val(&y))"
    return items.format(a), items.format(b), "String", body


@family("generic-args")
def param_default(rng):
    a, b = rng.choice([("u8", "u16"), ("u16", "u64")])
    items = "struct W<T = {}> {{\n    v: T,\n}}\n"
    body = "    let w: W = W { v: 200 };\n    format!(\"{} {}\", w.v.wrapping_add(100), std::mem::size_of::<W>())"
    return items.format(a), items.format(b), "String", body


@family("generic-args")
def turbofish_alias(rng):
    a, b = rng.choice([("u8", "u32"), ("i16", "i64")])
    items = "type A = {};\n"
    body = "    std::mem::size_of::<Option<A>>()"
    return items.format(a), items.format(b), "usize", body


# ---- array and const-generic lengths ----
@family("lengths")
def const_len(rng):
    n1, n2 = rng.choice([(1, 2), (3, 5), (4, 1)])
    items = "const N: usize = {};\n"
    body = "    let a: [u8; N] = std::array::from_fn(|i| i as u8 + 1);\n    a.iter().map(|&x| x as usize).sum::<usize>() * 10 + a.len()"
    return items.format(n1), items.format(n2), "usize", body


@family("lengths")
def const_generic_arg(rng):
    n1, n2 = rng.choice([(2, 3), (4, 7)])
    items = "const K: usize = {};\n\nstruct B<const L: usize> {{\n    a: [u8; L],\n}}\n"
    body = "    let b: B<K> = B { a: [9; K] };\n    b.a.len() * 100 + std::mem::size_of::<B<K>>()"
    return items.format(n1), items.format(n2), "usize", body


@family("lengths")
def const_fn_len(rng):
    n1, n2 = rng.choice([(2, 3), (1, 4)])
    items = "const fn n() -> usize {{\n    {}\n}}\n"
    body = "    let a = [7u8; n()];\n    a.len() + std::mem::size_of::<[u16; n()]>()"
    return items.format(n1), items.format(n2), "usize", body


@family("lengths")
def assoc_const_len(rng):
    n1, n2 = rng.choice([(2, 5), (3, 1)])
    items = "struct S;\n\nimpl S {{\n    const LEN: usize = {};\n}}\n"
    body = "    let a: [u8; S::LEN] = [1; S::LEN];\n    a.len()"
    return items.format(n1), items.format(n2), "usize", body


# ---- associated types and projections ----
@family("projections")
def assoc_ty(rng):
    a, b = rng.choice([("u8", "u16"), ("u16", "u8")])
    items = "trait Tr {{\n    type Out;\n}}\n\nstruct S;\n\nimpl Tr for S {{\n    type Out = {};\n}}\n"
    body = "    let x: <S as Tr>::Out = 200;\n    format!(\"{} {}\", x.wrapping_add(100), std::mem::size_of_val(&x))"
    return items.format(a), items.format(b), "String", body


@family("projections")
def iterator_item(rng):
    a, b = rng.choice([("u8", "u32"), ("u16", "u64")])
    items = "struct C(u32);\n\nimpl Iterator for C {{\n    type Item = {t};\n    fn next(&mut self) -> Option<{t}> {{\n        if self.0 == 0 {{\n            None\n        }} else {{\n            self.0 -= 1;\n            Some(200)\n        }}\n    }}\n}}\n"
    body = "    let v: Vec<_> = C(2).collect();\n    format!(\"{} {}\", v[0].wrapping_add(100), std::mem::size_of_val(&v[0]))"
    return items.format(t=a), items.format(t=b), "String", body


# ---- impl selection ----
@family("impls")
def inherent_shadows_trait(rng):
    v = rng.choice([7, 9])
    base = "trait Tr {{\n    fn get(&self) -> u32 {{\n        1\n    }}\n}}\n\nstruct S;\n\nimpl Tr for S {{}}\n{extra}"
    extra = f"\nimpl S {{\n    fn get(&self) -> u32 {{\n        {v}\n    }}\n}}\n"
    body = "    S.get()"
    return base.format(extra=""), base.format(extra=extra), "u32", body


@family("impls")
def blanket_vs_specific(rng):
    base = "use std::fmt::Display;\n\ntrait Show {{\n    fn show(&self) -> String;\n}}\n\nimpl<T: Display> Show for T {{\n    fn show(&self) -> String {{\n        format!(\"<{{}}>\", self)\n    }}\n}}\n\nstruct P;\n{extra}"
    extra = "\nimpl Display for P {\n    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n        write!(f, \"p\")\n    }\n}\n"
    body = "    format!(\"{} {}\", 5.show(), P.show())"
    return base.format(extra=extra), base.format(extra=""), "String", body


@family("impls")
def from_impl_added(rng):
    base = "struct A(u8);\nstruct B(u32);\n\nimpl From<A> for B {{\n    fn from(a: A) -> B {{\n        B(a.0 as u32 * {k})\n    }}\n}}\n"
    k1, k2 = rng.choice([(2, 3), (10, 1)])
    body = "    let b: B = A(5).into();\n    b.0"
    return base.format(k=k1), base.format(k=k2), "u32", body


@family("impls")
def default_method_override(rng):
    base = "trait Tr {{\n    fn v(&self) -> u32 {{\n        1\n    }}\n}}\n\nstruct S;\n\nimpl Tr for S {{{over}}}\n"
    over = "\n    fn v(&self) -> u32 {\n        42\n    }\n"
    body = "    let d: &dyn Tr = &S;\n    d.v() + S.v()"
    return base.format(over=""), base.format(over=over), "u32", body


@family("impls")
def trait_in_scope_glob(rng):
    base = "mod m {{\n    pub trait Tr {{\n        fn go(&self) -> u32;\n    }}\n    impl Tr for u8 {{\n        fn go(&self) -> u32 {{\n            {k}\n        }}\n    }}\n}}\n\nuse m::*;\n"
    k1, k2 = rng.choice([(3, 4), (8, 1)])
    body = "    5u8.go()"
    return base.format(k=k1), base.format(k=k2), "u32", body


# ---- auto traits ----
@family("auto")
def send_through_field(rng):
    base = "struct Inner {{\n    v: {t},\n}}\n\nstruct Outer {{\n    i: Inner,\n}}\n\nfn need_send<T: Send>(_t: &T) -> u8 {{\n    1\n}}\n"
    t1, t2 = rng.choice([("u8", "std::rc::Rc<u8>"), ("Vec<u8>", "std::cell::Cell<*const u8>")])
    body = "    let o: Option<Outer> = None;\n    need_send(&o)"
    return base.format(t=t1), base.format(t=t2), "u8", body


@family("auto")
def sync_static(rng):
    base = "struct S {{\n    v: {t},\n}}\n\nstatic G: Option<S> = None;\n"
    t1, t2 = ("u8", "std::cell::Cell<u8>")
    body = "    G.is_none()"
    return base.format(t=t1), base.format(t=t2), "bool", body


# ---- trait objects and closure layouts ----
@family("layouts")
def dyn_size(rng):
    base = "trait Tr {{\n    fn n(&self) -> usize;\n}}\n\nstruct S {{\n    a: [u8; {k}],\n}}\n\nimpl Tr for S {{\n    fn n(&self) -> usize {{\n        self.a.len()\n    }}\n}}\n\nfn make() -> Box<dyn Tr> {{\n    Box::new(S {{ a: [0; {k}] }})\n}}\n"
    k1, k2 = rng.choice([(3, 9), (16, 2)])
    body = "    let b = make();\n    b.n() * 100 + std::mem::size_of_val(&*b)"
    return base.format(k=k1), base.format(k=k2), "usize", body


@family("layouts")
def closure_capture_size(rng):
    base = "#[derive(Clone, Copy)]\nstruct P {{\n    a: {t},\n}}\n\nfn p() -> P {{\n    P {{ a: 1 }}\n}}\n"
    t1, t2 = rng.choice([("u8", "u64"), ("u32", "u16")])
    body = "    let x = p();\n    let c = move || x.a as u64 + 1;\n    std::mem::size_of_val(&c) as u64 * 10 + c()"
    return base.format(t=t1), base.format(t=t2), "u64", body


@family("layouts")
def struct_field_order(rng):
    base = "#[derive(Debug, Default)]\nstruct S {{\n{fields}}}\n"
    f1 = "    a: u8,\n    b: u32,\n"
    f2 = "    a: u8,\n    c: u16,\n    b: u32,\n"
    body = "    let s = S::default();\n    format!(\"{:?} {}\", s, std::mem::size_of::<S>())"
    return base.format(fields=f1), base.format(fields=f2), "String", body


# ---- cfg and attributes ----
@family("attrs")
def repr_discriminant(rng):
    base = "#[derive(Clone, Copy)]\n#[repr({r})]\nenum E {{\n    A = {a},\n    B,\n}}\n"
    v1 = base.format(r="u8", a=1)
    v2 = base.format(r=rng.choice(["u16", "u8"]), a=rng.choice([5, 200]))
    body = "    let e = E::B;\n    format!(\"{} {}\", e as u32, std::mem::size_of::<E>())"
    return v1, v2, "String", body


@family("attrs")
def cfg_item_switch(rng):
    base = "#[cfg({c1})]\nfn k() -> u32 {{\n    1\n}}\n\n#[cfg({c2})]\nfn k() -> u32 {{\n    2\n}}\n"
    v1 = base.format(c1="all()", c2="any()")
    v2 = base.format(c1="any()", c2="all()")
    body = "    k() * 10"
    return v1, v2, "u32", body


@family("attrs")
def derive_partial_eq(rng):
    base = "{d}\nstruct S {{\n    a: u8,\n}}\n{manual}"
    v1 = base.format(d="#[derive(PartialEq)]", manual="")
    v2 = base.format(d="", manual="\nimpl PartialEq for S {\n    fn eq(&self, _o: &S) -> bool {\n        false\n    }\n}\n")
    body = "    S { a: 1 } == S { a: 1 }"
    return v1, v2, "bool", body


# ---- macros ----
@family("macros")
def hygiene_binder(rng):
    base = "macro_rules! pick {{\n    ($x:ident) => {{{{\n        let {b}: u32 = 1;\n        $x\n    }}}};\n}}\n"
    body = "    let x: u32 = 5;\n    pick!(x) + x"
    return base.format(b="$x"), base.format(b="x"), "u32", body


@family("macros")
def nested_macro(rng):
    base = "macro_rules! inner {{\n    () => {{\n        {k}\n    }};\n}}\n\nmacro_rules! outer {{\n    () => {{\n        inner!() + 1\n    }};\n}}\n"
    k1, k2 = rng.choice([(3, 4), (10, 20)])
    body = "    outer!()"
    return base.format(k=k1), base.format(k=k2), "u32", body


@family("macros")
def macro_type_position(rng):
    base = "macro_rules! t {{\n    () => {{\n        {t}\n    }};\n}}\n"
    t1, t2 = rng.choice([("u8", "u16"), ("i32", "i8")])
    body = "    let x: t!() = 100;\n    format!(\"{} {}\", x.wrapping_mul(3), std::mem::size_of_val(&x))"
    return base.format(t=t1), base.format(t=t2), "String", body


# ---- use-globs and shadowing ----
@family("globs")
def glob_shadow(rng):
    base = "mod a {{\n    pub const V: u32 = 1;\n}}\n\nmod b {{\n    {item}\n}}\n\nuse a::*;\nuse b::*;\n"
    v1 = base.format(item="pub const W: u32 = 2;")
    v2 = base.format(item="pub const V: u32 = 3;")
    body = "    V"
    return v1, v2, "u32", body


@family("globs")
def local_shadows_glob(rng):
    base = "mod a {{\n    pub fn k() -> u32 {{\n        1\n    }}\n}}\n\nuse a::*;\n{local}"
    local = "\nfn k() -> u32 {\n    9\n}\n"
    body = "    k()"
    return base.format(local=""), base.format(local=local), "u32", body


@family("globs")
def glob_type_alias(rng):
    base = "mod a {{\n    pub type T = {t};\n}}\n\nuse a::*;\n"
    t1, t2 = rng.choice([("u8", "u64"), ("i16", "i8")])
    body = "    let v: T = 100;\n    format!(\"{} {}\", v.wrapping_mul(3), std::mem::size_of::<T>())"
    return base.format(t=t1), base.format(t=t2), "String", body


# ---- Drop and Copy ----
@family("dropcopy")
def drop_added(rng):
    base = "struct D(u8);\n{imp}\nfn hold() -> u8 {{\n    let d = D(3);\n    d.0\n}}\n"
    imp = "\nimpl Drop for D {\n    fn drop(&mut self) {\n        println!(\"drop {}\", self.0);\n    }\n}\n"
    body = "    let _a = D(1);\n    hold()"
    return base.format(imp=""), base.format(imp=imp), "u8", body


@family("dropcopy")
def copy_removed(rng):
    base = "#[derive(Clone{c})]\nstruct P {{\n    v: u8,\n}}\n"
    body = "    let p = P { v: 2 };\n    let q = p;\n    p.v + q.v"
    return base.format(c=", Copy"), base.format(c=""), "u8", body


@family("dropcopy")
def drop_order_field(rng):
    base = "struct N(u8);\n\nimpl Drop for N {{\n    fn drop(&mut self) {{\n        println!(\"n{{}}\", self.0);\n    }}\n}}\n\nstruct Pair {{\n{fields}}}\n"
    f1 = "    a: N,\n    b: N,\n"
    f2 = "    b: N,\n    a: N,\n"
    body = "    let p = Pair { a: N(1), b: N(2) };\n    let _ = &p;\n    7u8"
    return base.format(fields=f1), base.format(fields=f2), "u8", body


# ---- const evaluation through fns ----
@family("consteval")
def const_through_fn(rng):
    base = "const fn g(x: u32) -> u32 {{\n    x * {k}\n}}\n\nconst C: u32 = g(3);\n"
    k1, k2 = rng.choice([(2, 5), (7, 1)])
    body = "    C + 1"
    return base.format(k=k1), base.format(k=k2), "u32", body


@family("consteval")
def static_init_fn(rng):
    base = "const fn g() -> [u8; 2] {{\n    [{a}, 1]\n}}\n\nstatic T: [u8; 2] = g();\n"
    a1, a2 = rng.choice([(4, 9), (0, 3)])
    body = "    T[0] as u32 * 10 + T[1] as u32"
    return base.format(a=a1), base.format(a=a2), "u32", body


@family("consteval")
def enum_disc_const(rng):
    base = "const BASE: isize = {b};\n\n#[derive(Clone, Copy)]\nenum E {{\n    A = BASE,\n    B,\n}}\n"
    b1, b2 = rng.choice([(1, 10), (5, 0)])
    body = "    E::B as isize"
    return base.format(b=b1), base.format(b=b2), "isize", body


# ---- more type-level reads ----
@family("projections")
def rpit_hidden(rng):
    base = "fn mk() -> impl std::fmt::Display {{\n    {v}\n}}\n"
    v1, v2 = rng.choice([("250u8", "300u16"), ("7i8", "-7i64")])
    body = "    format!(\"{} {}\", mk(), std::mem::size_of_val(&mk()))"
    return base.format(v=v1), base.format(v=v2), "String", body


@family("impls")
def closure_param_sig(rng):
    base = "fn apply<F: Fn({t}) -> {t}>(f: F) -> u64 {{\n    f(250) as u64\n}}\n"
    t1, t2 = rng.choice([("u8", "u16"), ("u16", "u32")])
    body = "    apply(|x| x.wrapping_add(10))"
    return base.format(t=t1), base.format(t=t2), "u64", body


@family("impls")
def deref_target(rng):
    base = "use std::ops::Deref;\n\nstruct A;\nstruct B;\n\nimpl A {{\n    fn who(&self) -> u32 {{\n        1\n    }}\n}}\n\nimpl B {{\n    fn who(&self) -> u32 {{\n        2\n    }}\n}}\n\nstatic GA: A = A;\nstatic GB: B = B;\n\nstruct W;\n\nimpl Deref for W {{\n    type Target = {t};\n    fn deref(&self) -> &{t} {{\n        &{g}\n    }}\n}}\n"
    v1 = base.format(t="A", g="GA")
    v2 = base.format(t="B", g="GB")
    body = "    W.who()"
    return v1, v2, "u32", body


@family("impls")
def op_impl(rng):
    base = "#[derive(Clone, Copy)]\nstruct P(u32);\n\nimpl std::ops::Add for P {{\n    type Output = {o};\n    fn add(self, q: P) -> {o} {{\n        {e}\n    }}\n}}\n"
    v1 = base.format(o="u32", e="self.0 + q.0")
    v2 = base.format(o="u64", e="(self.0 as u64) << 33 | q.0 as u64")
    body = "    let r = P(1) + P(2);\n    format!(\"{} {}\", r, std::mem::size_of_val(&r))"
    return v1, v2, "String", body


@family("impls")
def index_output(rng):
    base = "struct T;\n\nstatic A8: [u8; 2] = [250, 1];\nstatic A16: [u16; 2] = [250, 1];\n\nimpl std::ops::Index<usize> for T {{\n    type Output = {o};\n    fn index(&self, i: usize) -> &{o} {{\n        &{g}[i]\n    }}\n}}\n"
    v1 = base.format(o="u8", g="A8")
    v2 = base.format(o="u16", g="A16")
    body = "    T[0].wrapping_add(10) as u64"
    return v1, v2, "u64", body


@family("lengths")
def const_generic_default(rng):
    n1, n2 = rng.choice([(2, 5), (3, 1)])
    base = "struct B<const N: usize = {n}> {{\n    a: [u8; N],\n}}\n"
    body = "    let b: B = B { a: [1; _] };\n    b.a.len()"
    return base.format(n=n1), base.format(n=n2), "usize", body


@family("generic-args")
def default_in_field(rng):
    a, b = rng.choice([("u8", "u64"), ("u16", "u8")])
    base = "#[derive(Default)]\nstruct W<T = {t}> {{\n    v: T,\n}}\n\n#[derive(Default)]\nstruct H {{\n    w: W,\n}}\n"
    body = "    let h = H::default();\n    format!(\"{} {}\", h.w.v, std::mem::size_of::<H>())"
    return base.format(t=a), base.format(t=b), "String", body


@family("projections")
def assoc_const_generic(rng):
    k1, k2 = rng.choice([(3, 8), (100, 1)])
    base = "trait Tr {{\n    const K: u32;\n}}\n\nstruct S;\n\nimpl Tr for S {{\n    const K: u32 = {k};\n}}\n\nfn get<T: Tr>() -> u32 {{\n    T::K * 2\n}}\n"
    body = "    get::<S>() + <S as Tr>::K"
    return base.format(k=k1), base.format(k=k2), "u32", body


@family("macros")
def dollar_crate(rng):
    base = "mod m {{\n    pub fn k() -> u32 {{\n        {v}\n    }}\n}}\n\nmacro_rules! call {{\n    () => {{\n        $crate::m::k()\n    }};\n}}\n"
    v1, v2 = rng.choice([(1, 2), (40, 3)])
    body = "    call!() * 10"
    return base.format(v=v1), base.format(v=v2), "u32", body


@family("macros")
def macro_items(rng):
    base = "macro_rules! mk {{\n    () => {{\n        fn k() -> {t} {{\n            200\n        }}\n    }};\n}}\n\nmk!();\n"
    t1, t2 = rng.choice([("u8", "u16"), ("u32", "u8")])
    body = "    let v = k();\n    format!(\"{} {}\", v.wrapping_add(100), std::mem::size_of_val(&v))"
    return base.format(t=t1), base.format(t=t2), "String", body


@family("globs")
def use_rename(rng):
    base = "mod a {{\n    pub fn k() -> u8 {{\n        200\n    }}\n}}\n\nmod b {{\n    pub fn k() -> u16 {{\n        200\n    }}\n}}\n\nuse {m}::k as j;\n"
    body = "    let v = j();\n    format!(\"{} {}\", v.wrapping_add(100), std::mem::size_of_val(&v))"
    return base.format(m="a"), base.format(m="b"), "String", body


@family("auto")
def send_bound_generic(rng):
    base = "struct S {{\n    v: {t},\n}}\n\nfn spawnish<T: Send + 'static>(t: T) -> usize {{\n    std::mem::size_of_val(&t)\n}}\n"
    t1, t2 = ("u32", "std::rc::Rc<u32>")
    body = "    spawnish(S { v: Default::default() })"
    return base.format(t=t1), base.format(t=t2), "usize", body


# ---- reads only codegen makes: vtable slots, discriminants, field offsets ----
@family("codegen")
def vtable_order(rng):
    base = "trait Tr {{\n{extra}    fn a(&self) -> u32 {{\n        1\n    }}\n    fn b(&self) -> u32 {{\n        2\n    }}\n}}\n\nstruct S;\n\nimpl Tr for S {{}}\n\nfn obj() -> Box<dyn Tr> {{\n    Box::new(S)\n}}\n"
    extra = "    fn z(&self) -> u32 {\n        99\n    }\n"
    body = "    let d = obj();\n    d.b() * 10 + d.a()"
    return base.format(extra=""), base.format(extra=extra), "u32", body


@family("codegen")
def variant_insert(rng):
    base = "#[derive(Clone, Copy)]\npub enum E {{\n    A,\n{extra}    B,\n    C,\n}}\n\npub fn mk(i: u8) -> E {{\n    match i {{\n        0 => E::A,\n        1 => E::B,\n        _ => E::C,\n    }}\n}}\n"
    extra = "    X,\n"
    body = "    let e = mk(1);\n    match e {\n        E::B => 7u32,\n        E::C => 8,\n        _ => 9,\n    }"
    return base.format(extra=""), base.format(extra=extra), "u32", body


@family("codegen")
def field_reorder_ref(rng):
    base = "pub struct S {{\n{extra}    pub a: u8,\n    pub b: u32,\n}}\n\npub fn mk() -> S {{\n    S {{ {init}a: 3, b: 77 }}\n}}\n\nfn read(s: &S) -> u32 {{\n    s.b + s.a as u32\n}}\n"
    body = "    let s = mk();\n    read(&s)"
    return base.format(extra="", init=""), base.format(extra="    pub z: u64,\n", init="z: 5, "), "u32", body


@family("codegen")
def payload_size_ref(rng):
    base = "pub enum P {{\n    N,\n    V({t}),\n}}\n\npub fn mk() -> P {{\n    P::V(200)\n}}\n\nfn get(p: &P) -> u64 {{\n    match p {{\n        P::V(x) => *x as u64 + 1,\n        P::N => 0,\n    }}\n}}\n"
    t1, t2 = rng.choice([("u8", "u64"), ("u16", "u8")])
    body = "    get(&mk())"
    return base.format(t=t1), base.format(t=t2), "u64", body


@family("codegen")
def generic_layout_instance(rng):
    base = "#[derive(Clone, Copy, Default)]\npub struct L {{\n    pub a: {t},\n    pub b: u8,\n}}\n\nfn second<T: Copy>(v: &[T]) -> T {{\n    v[1]\n}}\n\npub fn mk() -> Vec<L> {{\n    vec![L {{ a: 1, b: 2 }}, L {{ a: 3, b: 4 }}]\n}}\n"
    t1, t2 = rng.choice([("u8", "u64"), ("u32", "u16")])
    body = "    let v = mk();\n    let l = second(&v);\n    l.a as u64 * 10 + l.b as u64"
    return base.format(t=t1), base.format(t=t2), "u64", body


# ---- header-only edits (the daemon's header delta keeps its program for these) ----
@family("headers")
def callee_ret_type(rng):
    a, b = rng.choice([("u8", "u16"), ("u16", "u32"), ("i8", "i64")])
    base = "fn g() -> {t} {{\n    100\n}}\n"
    body = "    let v = g();\n    format!(\"{} {}\", v.wrapping_mul(3), std::mem::size_of_val(&v))"
    return base.format(t=a), base.format(t=b), "String", body


@family("headers")
def callee_param_type(rng):
    a, b = rng.choice([("u8", "u16"), ("u16", "u64")])
    base = "fn g(x: {t}) -> u64 {{\n    x as u64\n}}\n"
    body = "    let a = 250;\n    let r = g(a);\n    r + a.wrapping_add(10) as u64"
    return base.format(t=a), base.format(t=b), "u64", body


@family("headers")
def trait_method_sig(rng):
    a, b = rng.choice([("u8", "u16"), ("u32", "u8")])
    base = "trait Tr {{\n    fn m(&self) -> {t};\n}}\n\nstruct S;\n\nimpl Tr for S {{\n    fn m(&self) -> {t} {{\n        200\n    }}\n}}\n"
    body = "    let v = S.m();\n    format!(\"{} {}\", v.wrapping_add(100), std::mem::size_of_val(&v))"
    return base.format(t=a), base.format(t=b), "String", body


@family("headers")
def const_type(rng):
    a, b = rng.choice([("u8", "u16"), ("u16", "u8")])
    base = "const K: {t} = 200;\n"
    body = "    let v = K;\n    format!(\"{} {}\", v.wrapping_add(100), std::mem::size_of_val(&v))"
    return base.format(t=a), base.format(t=b), "String", body


@family("headers")
def static_type(rng):
    a, b = rng.choice([("u8", "u16"), ("u32", "u8")])
    base = "static G: {t} = 200;\n"
    body = "    let v = G;\n    format!(\"{} {}\", v.wrapping_add(100), std::mem::size_of_val(&v))"
    return base.format(t=a), base.format(t=b), "String", body


@family("headers")
def generic_callee_ret(rng):
    a, b = rng.choice([("u8", "u16"), ("u16", "u32")])
    base = "fn g<T: Copy>(_t: T) -> {t} {{\n    100\n}}\n"
    body = "    let v = g(1u8);\n    format!(\"{} {}\", v.wrapping_mul(3), std::mem::size_of_val(&v))"
    return base.format(t=a), base.format(t=b), "String", body


# ---- traits that enter or leave scope (Sol's read-gap review r06: a std trait's import) ----
def _scope_pair(rng, with_import, user):
    if user:
        head = "mod t {\n    pub trait Add2 {\n        fn add(self, o: super::S) -> u8;\n    }\n    impl Add2 for super::S {\n        fn add(self, _o: super::S) -> u8 {\n            7\n        }\n    }\n}\n\n"
        use = "use t::Add2;\n"
        imp = ""
    else:
        head = ""
        use = "use std::ops::Add;\n"
        imp = "impl std::ops::Add for S {\n    type Output = u8;\n    fn add(self, _o: S) -> u8 {\n        7\n    }\n}\n\n"
    mine = "trait Mine {\n    fn add(&self, _o: S) -> u8 {\n        9\n    }\n}\n\nimpl Mine for S {}\n"
    return (use if with_import else "") + "\n#[derive(Clone, Copy)]\nstruct S;\n\n" + head + imp + mine


@family("scope")
def std_trait_import_removed(rng):
    body = "    S.add(S)"
    return _scope_pair(rng, True, False), _scope_pair(rng, False, False), "u8", body


@family("scope")
def std_trait_import_added(rng):
    body = "    S.add(S)"
    return _scope_pair(rng, False, False), _scope_pair(rng, True, False), "u8", body


@family("scope")
def user_trait_import_removed(rng):
    body = "    S.add(S)"
    return _scope_pair(rng, True, True), _scope_pair(rng, False, True), "u8", body


@family("scope")
def std_glob_removed(rng):
    v1 = "use std::ops::*;\n\nstruct S;\n\nimpl Add for S {\n    type Output = u8;\n    fn add(self, _o: S) -> u8 {\n        7\n    }\n}\n"
    v2 = "struct S;\n\nimpl std::ops::Add for S {\n    type Output = u8;\n    fn add(self, _o: S) -> u8 {\n        7\n    }\n}\n"
    body = "    S.add(S)"
    return v1, v2, "u8", body


FORMS = ["plain", "generic", "closure", "method"]


def split(items, form, ret, body, rng, header):
    """The program as two files: main.rs (the items, `mod body;` after them so their macros
    are in scope there) and body.rs (fn `f`, which sees the parent's private items through
    `use super::*`). An edit then changes main.rs only, and body.rs takes the per-file path."""
    fdef, call = wrap(form, ret, body, rng)
    fdef = re.sub(r"\bfn f\b", "pub fn f", fdef).replace("struct Holder;", "pub struct Holder;")
    main = f"{header}{items}\nmod body;\n\nfn main() {{\n    println!(\"{{:?}}\", body::{call});\n}}\n"
    return {"main.rs": main, "body.rs": f"#[allow(unused_imports)]\nuse super::*;\n\n{fdef}"}


def publish(items):
    """`items` as a library crate's source: every top-level item, field and inherent impl item
    pub, so a dependent crate can name them (tuple fields too)."""
    out, ctx = [], []
    for line in items.splitlines():
        stripped = line.lstrip()
        indent = len(line) - len(stripped)
        if indent == 0 and re.match(r"(struct|enum|fn|trait|type|const|static|mod)\b", stripped):
            m = re.match(r"struct (\w+)(<[^>]*>)?\((.*)\);$", stripped)
            if m:
                fields = ", ".join("pub " + f.strip() for f in m.group(3).split(","))
                line = f"pub struct {m.group(1)}{m.group(2) or ''}({fields});"
            else:
                line = "pub " + line
        if indent == 0 and stripped.endswith("{"):
            kind = "struct" if re.match(r"(pub )?struct\b", stripped) else ("inherent" if re.match(r"impl(<[^>]*>)? \w+(<[^>]*>)? \{$", stripped) else "other")
            ctx = [kind]
        elif indent == 0 and stripped.startswith("}"):
            ctx = []
        elif indent == 4 and ctx == ["struct"] and re.match(r"\w+:", stripped):
            line = "    pub " + stripped
        elif indent == 4 and ctx == ["inherent"] and re.match(r"(fn|const)\b", stripped):
            line = "    pub " + stripped
        out.append(line)
    return "\n".join(out) + "\n"


def dep_history(items1, items2, form, ret, body, rng, header):
    """The program as a Cargo package whose items live in a path dependency `dep` (an edit
    there under an unchanged dependent crate). Versions are overlays (tests/daemon cargo
    layout): v1 the whole package, v2 and v3 only dep/src/lib.rs."""
    fdef, call = wrap(form, ret, body, rng)
    main = f"{header}#[allow(unused_imports)]\nuse dep::*;\n\n{fdef}\nfn main() {{\n    println!(\"{{:?}}\", {call});\n}}\n"
    manifest = '[package]\nname = "scen"\nversion = "0.1.0"\nedition = "2024"\n\n[workspace]\n\n[dependencies]\ndep = { path = "dep", version = "0.1" }\n'
    dep_manifest = '[package]\nname = "dep"\nversion = "0.1.0"\nedition = "2024"\n'
    v1 = {"Cargo.toml": manifest, "src/main.rs": main, "dep/Cargo.toml": dep_manifest, "dep/src/lib.rs": publish(items1)}
    return [v1, {"dep/src/lib.rs": publish(items2)}, {"dep/src/lib.rs": publish(items1)}]


def history(name, rng):
    cls, fn = FAMILIES[name]
    items1, items2, ret, body = fn(rng)
    form = rng.choice(FORMS)
    if form == "closure" and ("format!" in body or "let " in body):
        body = "\n".join("    " + l for l in body.splitlines())
    header = f"// read-gap history: family {name} ({cls}), form {form}\n"
    pick = rng.random()
    if pick < 0.2 and not any(k in items1 + items2 for k in ("use ", "macro_rules", "static ")) and form != "method":
        return form + "-dep", dep_history(items1, items2, form, ret, body, rng, header)
    if pick < 0.5:
        v1 = split(items1, form, ret, body, rng, header)
        v2 = split(items2, form, ret, body, rng, header)
        return form + "-split", [v1, v2, v1]
    v1 = program(items1, form, ret, body, rng)
    v2 = program(items2, form, ret, body, rng)
    return form, [header + v1, header + v2, header + v1]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--count", type=int, default=len(FAMILIES))
    ap.add_argument("--out")
    ap.add_argument("--list", action="store_true")
    a = ap.parse_args()
    if a.list:
        for n, (cls, _) in FAMILIES.items():
            print(cls, n)
        return
    rng = random.Random(a.seed)
    names = sorted(FAMILIES)
    os.makedirs(a.out, exist_ok=True)
    for k in range(a.count):
        name = names[k % len(names)] if k < len(names) else rng.choice(names)
        form, versions = history(name, rng)
        d = os.path.join(a.out, f"rg-{name}-{form}-{a.seed}-{k}")
        os.makedirs(d, exist_ok=True)
        if form.endswith("-dep"):
            with open(os.path.join(d, "cargo"), "w") as f:
                f.write("src/main.rs --bin scen\n")
        for i, src in enumerate(versions, 1):
            if isinstance(src, dict):
                vd = os.path.join(d, f"v{i}")
                os.makedirs(vd, exist_ok=True)
                for fname, text in src.items():
                    os.makedirs(os.path.dirname(os.path.join(vd, fname)), exist_ok=True)
                    with open(os.path.join(vd, fname), "w") as f:
                        f.write(text)
            else:
                with open(os.path.join(d, f"v{i}.rs"), "w") as f:
                    f.write(src)
    print(f"wrote {a.count} histories to {a.out}")


if __name__ == "__main__":
    main()
