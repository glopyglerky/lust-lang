#!/usr/bin/env python3
# Derive edit differential (PLAN P3.1, extended for P3.2): "-Copy, -Clone
# through Option<A> and tuples". Odd seeds take the round 3 programs: all
# nine derivable traits, generic Adts, fields through Box/Vec/Option,
# wrappers of wrappers, consts and const fn bounds, WF bounds, supertraits,
# a user blanket impl, and `main` returning `Result<(), A>`. Each generated program defines one Adt A with a derive list
# and uses A only through compound types (tuples, Option, arrays, Vec,
# generic bounds, closures, a wrapper that derives over a field of A). A
# sequence of versions then drops and adds Copy and Clone (and sometimes
# Debug or PartialEq) on A's derive list. Seeds 2 mod 3 take the round 4
# programs (`versions4`): partial lists, first-ever derives, rejected lists
# and their reverts after unrelated edits or repeated rejects. Seeds 1 mod 6
# take the round 5 programs (`versions5`): templates (generic fns, closures,
# trait default bodies, generic impl methods, a generic const fn) whose
# bodies use one derived trait of A, which a version then drops. Every version runs on one warm
# daemon (check, then run), a cold lust (LUST_NO_DAEMON=1), and rustc 1.99.
#
# Classes per version: false accept (lust accepts, rustc rejects), false
# reject (lust rejects valid Rust; counted, not a failure: a lust gap in
# both modes), warm != cold, wrong output, hang (a lust timeout; never a
# reject). Exit 1 on any false accept, warm != cold, wrong output, or hang,
# and under LUST_CHECK_ITEMPATCH=1 on any patch oracle failure or a program
# the oracle never checked.
#
# Usage: python3 scripts/derive_sweep.py [--count N] [--seed S] [--step K] [--jobs J] [--keep DIR] [--resolver]
# --resolver adds a `const` item to every program, so the daemon takes the resolver path (with
# its body patch) instead of the single-file chunk path. The patch oracle (LUST_CHECK_ITEMPATCH=1)
# covers both paths.
# Env: LUST_BIN (default target/release/lust).
import argparse, os, random, re, shutil, signal, subprocess, sys, tempfile, time
from concurrent.futures import ProcessPoolExecutor
import oracle_log  # scripts/oracle_log.py

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LUST = os.environ.get("LUST_BIN", os.path.join(ROOT, "target/release/lust"))
TIMEOUT = 60

# Each shape is (fn items, main statements). `A` is the Adt type, `mk()`
# builds one, `show(&a)` prints a field of one without any derive.
SHAPES = {
    "tuple_copy": ("", "let t = (mk(), 1u8);\n    let u = t;\n    println!(\"{} {}\", show(&t.0), u.1);"),
    "option_copy": ("", "let o = Some(mk());\n    let p = o;\n    println!(\"{} {}\", o.is_some(), show(&p.unwrap()));"),
    "nested_copy": ("", "let n = Some((mk(), [mk(), mk()]));\n    let m = n;\n    println!(\"{} {}\", n.is_some(), show(&m.unwrap().1[1]));"),
    "array_repeat": ("", "let arr = [mk(); 3];\n    println!(\"{}\", show(&arr[2]));"),
    "generic_copy": (
        "fn twice<T: Copy>(t: T) -> (T, T) {\n    (t, t)\n}\n",
        "let w = twice(Some(mk()));\n    println!(\"{}\", show(&w.1.unwrap()));",
    ),
    "generic_copy_tuple": (
        "fn first<T: Copy, U: Copy>(p: (T, U)) -> T {\n    let q = p;\n    let _r = p;\n    q.0\n}\n",
        "println!(\"{}\", show(&first((mk(), 2i32))));",
    ),
    "option_clone": ("", "let o = Some(mk());\n    let c = o.clone();\n    println!(\"{}\", show(&c.unwrap()));"),
    "tuple_clone": ("", "let t = (mk(), String::from(\"s\"));\n    let c = t.clone();\n    println!(\"{} {}\", show(&c.0), t.1);"),
    "generic_clone": (
        "fn dup<T: Clone>(t: &T) -> T {\n    t.clone()\n}\n",
        "let v = vec![Some((mk(), 3u16))];\n    let d = dup(&v);\n    println!(\"{}\", show(&d[0].as_ref().unwrap().0));",
    ),
    "iter_copied": ("", "let v = vec![mk(), mk()];\n    let w: Vec<A> = v.iter().copied().collect();\n    println!(\"{}\", show(&w[1]));"),
    "iter_cloned": ("", "let v = vec![mk()];\n    let w: Vec<A> = v.iter().cloned().collect();\n    println!(\"{}\", show(&w[0]));"),
    "closure_move": ("", "let a = mk();\n    let f = move || show(&a);\n    println!(\"{} {}\", f(), show(&a));"),
    "option_unwrap_or": ("", "let o: Option<A> = None;\n    let x = o.unwrap_or(mk());\n    let y = o.unwrap_or(x);\n    println!(\"{} {}\", show(&x), show(&y));"),
    "ref_clone": ("", "let a = mk();\n    let r = &a;\n    let c = r.clone();\n    println!(\"{}\", show(&c));"),
    "vec_clone": ("", "let v = vec![(mk(), 'x')];\n    let w = v.clone();\n    println!(\"{} {}\", show(&w[0].0), v.len());"),
    "wrapper_derive": (
        "#[derive(Clone, Copy)]\nstruct H {\n    inner: A,\n    k: u8,\n}\n",
        "let h = H { inner: mk(), k: 4 };\n    let g = h;\n    println!(\"{} {}\", show(&h.inner), g.k);",
    ),
    "generic_wrapper": (
        "#[derive(Clone, Copy)]\nstruct G<T> {\n    t: T,\n}\nfn copy_g<T: Copy>(g: G<T>) -> (G<T>, G<T>) {\n    (g, g)\n}\n",
        "let g = G { t: Some(mk()) };\n    let (x, _y) = copy_g(g);\n    println!(\"{}\", show(&x.t.unwrap()));",
    ),
    "debug_option": ("", "let o = Some((mk(), 1u8));\n    println!(\"{:?}\", o);"),
    "eq_tuple": ("", "let t = (mk(), 1u8);\n    let s = (mk(), 1u8);\n    println!(\"{}\", t == s);"),
    "method_copy_self": (
        "impl A {\n    fn twice(self) -> (A, A) {\n        (self, self)\n    }\n}\n",
        "let (x, _y) = mk().twice();\n    println!(\"{}\", show(&x));",
    ),
    # Round 3 (P3.2): every derivable trait, fields through Box/Vec/Option,
    # wrappers of wrappers, consts and const fn bounds, WF, supertraits,
    # a user blanket impl, and Hash/Eq/Ord users.
    "hash_map": ("", "let mut m = std::collections::HashMap::new();\n    m.insert(mk(), 1u8);\n    println!(\"{}\", m.len());"),
    "btree_set": ("", "let mut s = std::collections::BTreeSet::new();\n    s.insert(mk());\n    s.insert(mk());\n    println!(\"{}\", s.len());"),
    "sort_vec": ("", "let mut v = vec![mk(), mk()];\n    v.sort();\n    println!(\"{}\", show(&v[1]));"),
    "cmp_lt": ("", "println!(\"{} {:?}\", mk() < mk(), mk().partial_cmp(&mk()));"),
    "ne_eq": ("", "println!(\"{} {}\", mk() != mk(), Some(mk()) == None);"),
    "box_vec_wrapper": (
        "#[derive(Clone, Debug, PartialEq)]\nstruct BW {\n    b: Box<A>,\n    v: Vec<A>,\n    o: Option<A>,\n}\n",
        "let w = BW { b: Box::new(mk()), v: vec![mk()], o: None };\n    let c = w.clone();\n    println!(\"{} {:?}\", c == w, c.o.is_none());",
    ),
    "wrapper_of_wrapper": (
        "#[derive(Clone, Copy)]\nstruct W1 {\n    a: A,\n}\n#[derive(Clone, Copy)]\nstruct W2 {\n    w: W1,\n    n: u8,\n}\n",
        "let w = W2 { w: W1 { a: mk() }, n: 2 };\n    let v = w;\n    println!(\"{} {}\", show(&w.w.a), v.n);",
    ),
    "hash_wrapper": (
        "#[derive(PartialEq, Eq, Hash)]\nstruct HK {\n    k: (A, u8),\n}\n",
        "let mut m = std::collections::HashSet::new();\n    m.insert(HK { k: (mk(), 1) });\n    println!(\"{}\", m.len());",
    ),
    "ord_wrapper": (
        "#[derive(PartialEq, Eq, PartialOrd, Ord)]\nstruct OK {\n    k: Option<A>,\n}\n",
        "println!(\"{:?}\", OK { k: Some(mk()) }.cmp(&OK { k: None }));",
    ),
    "default_wrapper": (
        "#[derive(Default)]\nstruct DW {\n    a: Option<A>,\n    v: Vec<A>,\n}\n",
        "let d = DW::default();\n    println!(\"{} {}\", d.a.is_none(), d.v.len());",
    ),
    "supertrait_impl": (
        "trait Dup: Clone {\n    fn dup2(&self) -> (Self, Self) {\n        (self.clone(), self.clone())\n    }\n}\nimpl Dup for A {}\n",
        "let (x, _) = mk().dup2();\n    println!(\"{}\", show(&x));",
    ),
    "wf_bound": (
        "struct NeedCopy<T: Copy>(T);\nfn wrap_nc(a: A) -> NeedCopy<A> {\n    NeedCopy(a)\n}\n",
        "let n = wrap_nc(mk());\n    println!(\"{}\", show(&n.0));",
    ),
    "blanket_debug": (
        "trait Show2 {\n    fn s2(&self) -> String;\n}\nimpl<T: std::fmt::Debug> Show2 for T {\n    fn s2(&self) -> String {\n        format!(\"<{:?}>\", self)\n    }\n}\n",
        "println!(\"{}\", mk().s2());",
    ),
    "generic_default": (
        "fn make_d<T: Default>() -> T {\n    T::default()\n}\n",
        "let d: A = make_d();\n    println!(\"{}\", show(&d));",
    ),
    "const_item": (
        "const CA: A = MK_CONST;\nconst ARR: [A; 2] = [CA; 2];\n",
        "let a = CA;\n    let b = ARR;\n    println!(\"{} {}\", show(&a), show(&b[1]));",
    ),
    "const_fn_bound": (
        "const fn ident_c<T: Copy>(t: T) -> T {\n    t\n}\nconst CB: A = ident_c(MK_CONST);\n",
        "println!(\"{}\", show(&CB));",
    ),
    "const_pattern": (
        "const CP: A = MK_CONST;\n",
        "let m = match mk() {\n        CP => 1,\n        _ => 2,\n    };\n    println!(\"{}\", m);",
    ),
    "assoc_bound": (
        "trait HasOut {\n    type Out: Clone;\n    fn out(&self) -> Self::Out;\n}\nimpl HasOut for u8 {\n    type Out = A;\n    fn out(&self) -> A {\n        mk()\n    }\n}\n",
        "let o = 3u8.out();\n    println!(\"{}\", show(&o));",
    ),
    "where_trivial": ("fn needs_where() -> u8\nwhere\n    A: Copy,\n{\n    1\n}\n", "println!(\"{}\", needs_where());"),
    "impl_on_vec": (
        "trait Tr2: Clone {\n    fn n2(&self) -> usize;\n}\nimpl Tr2 for Vec<A> {\n    fn n2(&self) -> usize {\n        self.len()\n    }\n}\n",
        "let v: Vec<A> = vec![mk()];\n    println!(\"{}\", v.n2());",
    ),
    "assoc_impl_bound": (
        "trait Pick {\n    type Out;\n}\nstruct Holder<T>(T);\nimpl<T: Clone> Pick for Holder<T> {\n    type Out = T;\n}\nfn pick_out(h: Holder<A>) -> <Holder<A> as Pick>::Out {\n    h.0\n}\n",
        "let o = pick_out(Holder(mk()));\n    println!(\"{}\", show(&o));",
    ),
    # Round 4: an associated-type impl bounded by Copy that A's reach
    # cannot involve, and one a generic fn's signature projects through
    # with an instance at RepA<A>.
    "assoc_iter_rep": (
        "struct Rep<T>(T);\nimpl<T: Copy> Iterator for Rep<T> {\n    type Item = T;\n    fn next(&mut self) -> Option<T> {\n        Some(self.0)\n    }\n}\n",
        "let mut r = Rep(4u8);\n    println!(\"{:?}\", r.next());",
    ),
    "assoc_iter_a": (
        "struct RepA<T>(T);\nimpl<T: Copy> Iterator for RepA<T> {\n    type Item = T;\n    fn next(&mut self) -> Option<T> {\n        Some(self.0)\n    }\n}\n"
        "fn first_of<I: Iterator>(mut i: I) -> Option<I::Item> {\n    i.next()\n}\n",
        "let f = first_of(RepA(mk()));\n    println!(\"{}\", f.map(|a| show(&a)).unwrap_or_default());",
    ),
    # R13 v2: A under user blanket impls bounded by a toggled trait.
    "blanket_overlap": (
        "trait Show3 {\n    fn s3(&self) -> u8 {\n        1\n    }\n}\nimpl<T: std::fmt::Debug> Show3 for T {}\nimpl Show3 for A {}\n",
        "println!(\"{}\", mk().s3());",
    ),
    "blanket_super": (
        "trait Base: std::fmt::Debug {}\nimpl<T: std::fmt::Debug> Base for T {}\ntrait Up {\n    fn up(&self) -> u8 {\n        2\n    }\n}\nimpl<T: Base> Up for T {}\n",
        "println!(\"{}\", mk().up());",
    ),
    "blanket_eq": (
        "trait Same {\n    fn same(&self, o: &Self) -> bool;\n}\nimpl<T: PartialEq> Same for T {\n    fn same(&self, o: &Self) -> bool {\n        self == o\n    }\n}\n",
        "println!(\"{}\", mk().same(&mk()));",
    ),
    "blanket_dyn": (
        "trait Show4 {\n    fn s4(&self) -> String;\n}\nimpl<T: std::fmt::Debug> Show4 for T {\n    fn s4(&self) -> String {\n        format!(\"{:?}\", self)\n    }\n}\n",
        "let b: Box<dyn Show4> = Box::new(mk());\n    println!(\"{}\", b.s4());",
    ),
}

ROUND1_SHAPES = sorted(list(SHAPES)[: list(SHAPES).index("hash_map")])
# The shapes rounds 3 and 4 sample (round 5 adds its own below).
BASE_SHAPES = sorted(SHAPES)

# Round 5 (rule zero, w-sig-front 14:41): a template whose body names A
# and uses one derived trait of A. Its check (borrowck included) must
# read that trait, so dropping it checks the template again. Bodies take
# `a: A` and give a String; per trait the use, per context the template.
T5_USES = {
    "Copy": "{ let b = a; let c = a; show(&b) + &show(&c) }",
    "Clone": "{ let b = a.clone(); show(&b) + &show(&a) }",
    "Debug": "format!(\"{:?}\", a)",
    "PartialEq": "format!(\"{}\", a == a)",
    "Eq": "{ need_eq(&a); show(&a) }",
    "PartialOrd": "format!(\"{}\", a < a)",
    "Ord": "format!(\"{:?}\", a.cmp(&a))",
    "Hash": "{ let mut h = std::collections::hash_map::DefaultHasher::new(); std::hash::Hash::hash(&a, &mut h); show(&a) }",
    "Default": "{ let _d = a; show(&A::default()) }",
}
T5_HELPERS = "fn need_eq<X: Eq>(_x: &X) {}\n"
# Contexts: (items with NAME and BODY, main expression). `gmk` gets its
# value from mk() alone; `gmkc` does too, inside a template with closures
# (no stored check: it rechecks when its body names the reach); `galias`
# names A only through a type alias (its literal too), with closures;
# `gtwice` and `gfnbound` (round 6) make the closure kind follow A's derives.
T5_CONTEXTS = {
    "gfn": ("fn NAME<T>(a: A, _t: T) -> String {\n    BODY\n}\n", "NAME(mk(), 0u8)"),
    "gclosure": ("fn NAME<T>(a: A, _t: T) -> String {\n    let f = |a: A| BODY;\n    f(a)\n}\n", "NAME(mk(), 'c')"),
    "gdefault": ("trait NAMET {\n    fn NAME(&self, a: A) -> String {\n        BODY\n    }\n}\nimpl NAMET for u8 {}\n", "1u8.NAME(mk())"),
    "gimpl": ("struct NAMES<T>(T);\nimpl<T> NAMES<T> {\n    fn NAME(&self, a: A) -> String {\n        BODY\n    }\n}\n", "NAMES(2u16).NAME(mk())"),
    "gmk": ("fn NAME<T>(_t: T) -> String {\n    let a = mk();\n    BODY\n}\n", "NAME(3i32)"),
    "galias": ("type NAMEA = A;\nfn NAME<T>(_t: T) -> String {\n    let g = || 1u8;\n    let a: NAMEA = MK_ALIAS;\n    format!(\"{}{}\", g(), BODY)\n}\n", "NAME(2u64)"),
    "gmkc": ("fn NAME<T>(_t: T) -> String {\n    let g = || 1u8;\n    let a = mk();\n    format!(\"{}{}\", g(), BODY)\n}\n", "NAME(true)"),
    # Round 6: the closure's kind follows A's derives (a move closure that
    # moves A out is FnOnce unless A is Copy): called twice, and passed to
    # an `Fn` bound.
    "gtwice": ("fn NAME<T>(a: A, _t: T) -> String {\n    let f = move || BODY;\n    format!(\"{}{}\", f(), f())\n}\n", "NAME(mk(), 4u8)"),
    "gfnbound": ("fn NAMEC<F: Fn() -> String>(f: F) -> String {\n    f()\n}\nfn NAME<T>(a: A, _t: T) -> String {\n    NAMEC(move || BODY)\n}\n", "NAME(mk(), 5u8)"),
}
for _tr, _use in T5_USES.items():
    for _cx, (_items, _call) in T5_CONTEXTS.items():
        _name = f"t5_{_cx}_{_tr.lower()}"
        _helper = T5_HELPERS.replace("need_eq", "need_eq_" + _name) if _tr == "Eq" else ""
        SHAPES[_name] = (_helper + _items.replace("NAME", _name).replace("BODY", _use.replace("need_eq", "need_eq_" + _name)),
                         f"println!(\"{{}}\", {_call.replace('NAME', _name)});")
# A generic const fn that copies A twice (Copy only).
SHAPES["t5_constfn_copy"] = ("const fn t5_constfn_copy<T: Copy>(a: A, t: T) -> (A, A, T) {\n    (a, a, t)\n}\n",
                             "let (x, y, _) = t5_constfn_copy(mk(), 1u8);\n    println!(\"{} {}\", show(&x), show(&y));")
T5_SHAPES = sorted(sh for sh in SHAPES if sh.startswith("t5_"))

# Round 7 (--proj): written projections that name A in signatures, fields
# and consts (`Self::Item` of an impl for a wrapper of A, std impls whose
# answer follows A's Clone: ToOwned, Cloned, Cow). The derive delta checks
# them again in place of a full collect.
P7_SHAPES = {
    "p7_self_item": (
        "struct P7It(Option<A>);\nimpl Iterator for P7It {\n    type Item = A;\n    fn next(&mut self) -> Option<Self::Item> {\n        self.0.take()\n    }\n}\n",
        "let mut it = P7It(Some(mk()));\n    println!(\"{}\", it.next().map(|a| show(&a)).unwrap_or_default());",
    ),
    "p7_index_out": (
        "struct P7V(Vec<A>);\nimpl std::ops::Index<usize> for P7V {\n    type Output = A;\n    fn index(&self, i: usize) -> &Self::Output {\n        &self.0[i]\n    }\n}\n",
        "let v = P7V(vec![mk()]);\n    println!(\"{}\", show(&v[0]));",
    ),
    "p7_vec_into": (
        "fn p7_ii(v: Vec<A>) -> <Vec<A> as IntoIterator>::IntoIter {\n    v.into_iter()\n}\n",
        "println!(\"{}\", p7_ii(vec![mk()]).count());",
    ),
    "p7_out_names_a": (
        "trait P7Tr {\n    type Out;\n}\nimpl P7Tr for u8 {\n    type Out = A;\n}\nfn p7_out(x: <u8 as P7Tr>::Out) -> String {\n    show(&x)\n}\n",
        "println!(\"{}\", p7_out(mk()));",
    ),
    "p7_toowned_param": (
        "fn p7_own(a: <A as ToOwned>::Owned) -> String {\n    show(&a)\n}\n",
        "println!(\"{}\", p7_own(mk()));",
    ),
    "p7_toowned_field": (
        "struct P7F {\n    o: <A as ToOwned>::Owned,\n}\n",
        "let f = P7F { o: mk() };\n    println!(\"{}\", show(&f.o));",
    ),
    "p7_cloned_item": (
        "fn p7_cl(a: <std::iter::Cloned<std::slice::Iter<'static, A>> as Iterator>::Item) -> String {\n    show(&a)\n}\n",
        "println!(\"{}\", p7_cl(mk()));",
    ),
    "p7_generic_impl_item": (
        "struct P7G<T>(Option<T>);\nimpl<T: Clone> Iterator for P7G<T> {\n    type Item = T;\n    fn next(&mut self) -> Option<Self::Item> {\n        self.0.clone()\n    }\n}\n"
        "fn p7_g(mut g: P7G<A>) -> Option<<P7G<A> as Iterator>::Item> {\n    g.next()\n}\n",
        "println!(\"{}\", p7_g(P7G(Some(mk()))).map(|a| show(&a)).unwrap_or_default());",
    ),
    "p7_cow": (
        "fn p7_cow(c: std::borrow::Cow<'_, A>) -> String {\n    show(&c)\n}\n",
        "println!(\"{}\", p7_cow(std::borrow::Cow::Owned(mk())));",
    ),
    "p7_const_ty": (
        "const P7C: Option<<A as ToOwned>::Owned> = None;\n",
        "println!(\"{}\", P7C.is_none());",
    ),
    # A body that calls a generic `impl Fn` fn stays pending in `lust
    # check` (its reads are kept): one that names A without a derive
    # bound, and one that needs A: Copy.
    "p7_pending": (
        "fn p7_mk<V: Copy>(v: V) -> impl Fn() -> V {\n    move || v\n}\nfn p7_pend(a: &A) -> String {\n    let f = p7_mk(show(a).len());\n    format!(\"{}\", f())\n}\n",
        "println!(\"{}\", p7_pend(&mk()));",
    ),
    "p7_pending_copy": (
        "fn p7_mkc<V: Copy>(v: V) -> impl Fn() -> V {\n    move || v\n}\nfn p7_pendc(a: A) -> String {\n    let f = p7_mkc(a);\n    show(&f()) + &show(&f())\n}\n",
        "println!(\"{}\", p7_pendc(mk()));",
    ),
}
SHAPES.update(P7_SHAPES)

# Shapes that need a trait an Adt form cannot derive: Default (enum forms).
NEEDS_DEFAULT = {"generic_default"} | {f"t5_{c}_default" for c in T5_CONTEXTS}
# Shapes a generic Adt form leaves out: lust rejects `#[derive(Copy)]` over a
# field of a generic Adt instance (`AG<u8>`) in both modes (a false reject).
NOT_GENERIC = {"wrapper_derive", "wrapper_of_wrapper"}

# Adt forms: (definition after the derive line, mk body, show body, alias
# line or "", derives Default). A generic form derives on `AG<T>` and the
# programs name the instance through `type A = AG<..>;`.
ADTS = [
    ("struct A {\n    x: u8,\n    y: i64,\n}\n", "A { x: 1, y: -2 }", "format!(\"{}\", a.x as i64 + a.y)", "", True),
    ("struct A(u32, char);\n", "A(7, 'q')", "format!(\"{}{}\", a.0, a.1)", "", True),
    ("enum A {\n    P,\n    Q(i8),\n}\n", "A::Q(3)", "match a {\n        A::P => String::from(\"p\"),\n        A::Q(n) => format!(\"q{}\", n),\n    }", "", False),
    ("struct A {\n    r: &'static str,\n    o: Option<u8>,\n}\n", "A { r: \"z\", o: Some(5) }", "format!(\"{}{:?}\", a.r, a.o)", "", True),
    ("struct AG<T> {\n    x: T,\n    y: i64,\n}\n", "AG { x: 1, y: -2 }", "format!(\"{}\", a.x as i64 + a.y)", "type A = AG<u8>;\n", True),
    ("enum AG<T> {\n    P,\n    Q(T),\n}\n", "AG::Q(3)", "match a {\n        AG::P => String::from(\"p\"),\n        AG::Q(n) => format!(\"q{}\", n),\n    }", "type A = AG<i8>;\n", False),
    ("struct AG<T>(Option<T>, u8);\n", "AG(Some(4), 9)", "format!(\"{:?}{}\", a.0, a.1)", "type A = AG<u16>;\n", True),
]

FULL = ["Clone", "Copy", "Debug", "PartialEq"]
# The round 3 list: every derivable trait (Default only on forms that take it).
FULL3 = ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Hash", "PartialOrd", "Ord"]


def alias_literal(mk, alias):
    """mk's literal through a type alias of A: a tuple struct takes the
    braced form (`Alias { 0: .., 1: .. }`; an alias is no constructor fn)."""
    m = re.match(r"^AG?\((.*)\)$", mk)
    if not m:
        return re.sub(r"^AG?(?=\W)", alias, mk)
    parts, depth, cur = [], 0, ""
    for ch in m.group(1):
        depth += ch in "([" and 1 or ch in ")]" and -1 or 0
        if ch == "," and depth == 0:
            parts.append(cur.strip())
            cur = ""
        else:
            cur += ch
    parts.append(cur.strip())
    return f"{alias} {{ " + ", ".join(f"{i}: {v}" for i, v in enumerate(parts)) + " }"


def program(adt, derives, shapes, resolver=False, main_result=False, unrel=None):
    defn, mk, show, alias, _ = adt
    out = ["const _ROUTE: u8 = 0;\n"] if resolver else []
    if derives:
        out.append(f"#[derive({', '.join(derives)})]\n")
    out.append(defn)
    out.append(alias)
    out.append(f"fn mk() -> A {{\n    {mk}\n}}\n")
    out.append(f"fn show(a: &A) -> String {{\n    {show}\n}}\n")
    for sh in shapes:
        # MK_ALIAS: mk's literal through the shape's alias of A.
        out.append(SHAPES[sh][0].replace("MK_CONST", mk).replace("MK_ALIAS", alias_literal(mk, sh + "A")))
    out.append("fn main() -> Result<(), A> {\n" if main_result else "fn main() {\n")
    if unrel is not None:
        out.append(f"    println!(\"u{unrel}\");\n")
    for sh in shapes:
        out.append("    {\n    " + SHAPES[sh][1] + "\n    }\n")
    if main_result:
        out.append("    if false {\n        return Err(mk());\n    }\n    Ok(())\n")
    out.append("}\n")
    return "".join(out)


def versions(rng):
    """A derive-list sequence that starts and ends full and drops and
    adds Copy and Clone in between."""
    seq = [FULL]
    steps = [
        ["Clone", "Debug", "PartialEq"],  # -Copy
        ["Debug", "PartialEq"],  # -Copy -Clone
        ["Copy", "Debug", "PartialEq"],  # -Clone (Copy needs Clone)
        ["Clone", "Copy", "PartialEq"],  # -Debug
        ["Clone", "Copy", "Debug"],  # -PartialEq
        ["Clone", "Copy"],
    ]
    for _ in range(rng.randint(2, 4)):
        seq.append(rng.choice(steps))
        if rng.random() < 0.5:
            seq.append(FULL)
    if seq[-1] != FULL:
        seq.append(FULL)
    return seq


def versions3(rng, full):
    """Round 3: from the full list, drop one to three traits (any), often
    go back to full, and sometimes add a trait back one at a time."""
    seq = [full]
    cur = list(full)
    for _ in range(rng.randint(3, 6)):
        r = rng.random()
        if r < 0.35:
            cur = list(full)
        elif r < 0.55 and len(cur) < len(full):
            missing = [t for t in full if t not in cur]
            cur = [t for t in full if t in cur or t == rng.choice(missing)]
        else:
            drop = rng.sample(cur, min(len(cur), rng.randint(1, 3)))
            cur = [t for t in cur if t not in drop]
        seq.append(cur)
    if seq[-1] != full:
        seq.append(full)
    return seq


# Traits whose derive needs another derived trait (rustc E0277 without it).
NEEDS = {"Copy": ["Clone"], "Eq": ["PartialEq"], "PartialOrd": ["PartialEq"], "Ord": ["PartialOrd", "Eq"]}


def versions4(rng, full):
    """Round 4 (refused deltas and undo): start from a partial list, so an
    added trait is one the program never derived; rejected lists (a trait
    without the one it needs), then their revert, sometimes after an
    unrelated edit (`unrel` moves) or a second reject. Pairs of (derive
    list, unrelated edit counter)."""
    order = lambda ts: [t for t in full if t in ts]
    cur = order(rng.sample(full, rng.randint(0, len(full) - 1)))
    unrel = 0
    seq = [(cur, unrel)]
    for _ in range(rng.randint(3, 6)):
        r = rng.random()
        missing = [t for t in full if t not in cur]
        if r < 0.3 and missing:
            # A trait in, with what it needs (often never derived before).
            t = rng.choice(missing)
            cur = order(set(cur) | {t} | set(NEEDS.get(t, [])))
            seq.append((cur, unrel))
        elif r < 0.45 and cur:
            cur = order(set(cur) - set(rng.sample(cur, rng.randint(1, min(2, len(cur))))))
            cur = order([t for t in cur if all(n in cur for n in NEEDS.get(t, []))])
            seq.append((cur, unrel))
        else:
            # A reject (a trait without what it needs), then maybe an
            # unrelated edit or a second reject, then the revert.
            t = rng.choice(list(NEEDS))
            bad = order((set(cur) | {t}) - {rng.choice(NEEDS[t])})
            seq.append((bad, unrel))
            k = rng.random()
            if k < 0.35:
                unrel += 1
                seq.append((bad, unrel))
            elif k < 0.6:
                t2 = rng.choice(list(NEEDS))
                seq.append((order((set(bad) | {t2}) - {rng.choice(NEEDS[t2])}), unrel))
            seq.append((cur, unrel))
    return seq


def versions5(rng, full, shapes):
    """Round 5: from the full list, drop a trait one of the template
    shapes uses (with the traits that need it, or alone: a derive error),
    then go back to full, sometimes after a second drop. Some programs
    start without the trait, so the add comes first."""
    order = lambda ts: [t for t in full if t in ts]
    used = [sh.rsplit("_", 1)[1] for sh in shapes if sh.startswith("t5_")]
    traits = [t for t in full if t.lower() in used]

    def drop(t):
        if rng.random() < 0.8:
            return order([u for u in full if u != t and t not in NEEDS.get(u, []) and not (u == "Ord" and t == "PartialEq")])
        return order([u for u in full if u != t])

    seq = [drop(rng.choice(traits))] if rng.random() < 0.25 else []
    seq.append(full)
    for _ in range(rng.randint(2, 4)):
        seq.append(drop(rng.choice(traits)))
        if rng.random() < 0.3:
            seq.append(drop(rng.choice(traits)))
        seq.append(full)
    return seq


def gen(seed, resolver=False):
    vs = gen_base(seed, resolver)
    if not os.environ.get("DSWEEP_PROJ"):
        return vs
    # --proj: one to three round 7 shapes in every version (their own
    # random stream, so the base program stays the seed's).
    prng = random.Random(seed * 7919 + 7)
    extra = sorted(prng.sample(sorted(P7_SHAPES), prng.randint(1, 3)))
    items = "".join(P7_SHAPES[sh][0] for sh in extra)
    stmts = "".join("    {\n    " + P7_SHAPES[sh][1] + "\n    }\n" for sh in extra)
    out = []
    for v in vs:
        head, body = v.split("fn main()", 1)
        brace = body.index("{\n") + 2
        out.append(head + items + "fn main()" + body[:brace] + stmts + body[brace:])
    return out


def gen_base(seed, resolver=False):
    rng = random.Random(seed)
    if seed % 6 == 1:
        adt = rng.choice(ADTS)
        full = FULL3 + (["Default"] if adt[4] else [])
        pool = [sh for sh in T5_SHAPES if adt[4] or sh not in NEEDS_DEFAULT]
        only = [c for c in os.environ.get("DSWEEP_CONTEXTS", "").split(",") if c]
        if only:
            pool = [sh for sh in pool if sh.split("_")[1] in only]
        shapes = rng.sample(pool, rng.randint(1, 3)) + rng.sample(BASE_SHAPES, rng.randint(0, 1))
        shapes = [sh for sh in sorted(set(shapes)) if (adt[4] or sh not in NEEDS_DEFAULT) and not (adt[3] and sh in NOT_GENERIC)]
        return [program(adt, d, shapes, resolver) for d in versions5(rng, full, shapes)]
    if seed % 3 == 2:
        adt = rng.choice(ADTS)
        full = FULL3 + (["Default"] if adt[4] else [])
        pool = [sh for sh in BASE_SHAPES if (adt[4] or sh not in NEEDS_DEFAULT) and not (adt[3] and sh in NOT_GENERIC)]
        shapes = rng.sample(pool, rng.randint(1, 4))
        if rng.random() < 0.4:
            shapes.append(rng.choice(["assoc_iter_rep", "assoc_iter_a"]))
        shapes = sorted(set(shapes))
        return [program(adt, d, shapes, resolver, False, u) for d, u in versions4(rng, full)]
    if seed % 2 == 0:
        # The round 1 programs: the first four Adt forms, the first shapes.
        adt = rng.choice(ADTS[:4])
        shapes = rng.sample(ROUND1_SHAPES, rng.randint(2, 5))
        return [program(adt, d, shapes, resolver) for d in versions(rng)]
    adt = rng.choice(ADTS)
    full = FULL3 + (["Default"] if adt[4] else [])
    pool = [sh for sh in BASE_SHAPES if (adt[4] or sh not in NEEDS_DEFAULT) and not (adt[3] and sh in NOT_GENERIC)]
    shapes = rng.sample(pool, rng.randint(2, 6))
    main_result = rng.random() < 0.2
    return [program(adt, d, shapes, resolver, main_result) for d in versions3(rng, full)]


def first_code(stderr):
    m = re.search(r"error\[(L\d+)\]", stderr)
    return m.group(1) if m else None


def run(cmd, cwd, env):
    try:
        r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=TIMEOUT)
        return r.returncode, r.stdout, r.stderr
    except subprocess.TimeoutExpired:
        return None, "", "timeout"


def case(args):
    seed, keep, resolver = args
    vs = gen(seed, resolver)
    work = tempfile.mkdtemp(prefix=f"lust-dsweep-{seed}-")
    sock = os.path.join(work, "d.sock")
    env = dict(os.environ, LUST_DAEMON_SOCKET=sock, LUST_NO_CACHE="1")
    env.pop("LUST_NO_DAEMON", None)
    cold_env = dict(env, LUST_NO_DAEMON="1")
    oracle_path = oracle_log.setup(env, work)
    log = open(os.path.join(work, "daemon.log"), "w")
    daemon = subprocess.Popen([LUST, "daemon"], cwd=work, env=env, stderr=log, start_new_session=True)
    main = os.path.join(work, "main.rs")
    res = {"versions": 0, "fa": [], "fr": 0, "wc": [], "wrong": [], "hang": [], "oracle": [], "oracle_ok": 0, "rustc_rejects": 0}
    try:
        t0 = time.time()
        while not os.path.exists(sock):
            if time.time() - t0 > 10 or daemon.poll() is not None:
                res["hang"].append(f"seed {seed}: daemon start failed")
                return seed, res
            time.sleep(0.005)
        mtime = int(time.time())
        for i, src in enumerate(vs):
            mtime += 2
            with open(main, "w") as f:
                f.write(src)
            os.utime(main, (mtime, mtime))
            tag = f"seed {seed} v{i + 1}"
            res["versions"] += 1
            wck = run([LUST, "check", main], work, env)
            cck = run([LUST, "check", main], work, cold_env)
            wr = run([LUST, "run", main, "-o", os.path.join(work, "w.bin")], work, env)
            cr = run([LUST, "run", main, "-o", os.path.join(work, "c.bin")], work, cold_env)
            rr = run(["rustc", "--edition", "2024", main, "-o", os.path.join(work, "r.bin")], work, env)
            rustc_ok = rr[0] == 0
            rout = run([os.path.join(work, "r.bin")], work, env) if rustc_ok else None
            res["rustc_rejects"] += 0 if rustc_ok else 1
            hung = [n for n, r in (("warm check", wck), ("cold check", cck), ("warm run", wr), ("cold run", cr)) if r[0] is None]
            if hung:
                res["hang"].append(f"{tag}: {', '.join(hung)} timed out")
                break
            if (wck[0] == 0) != (cck[0] == 0) or first_code(wck[2]) != first_code(cck[2]):
                res["wc"].append(f"{tag}: check warm rc={wck[0]} {wck[2][:200]!r} cold rc={cck[0]} {cck[2][:200]!r}")
            if (wr[0] == 0) != (cr[0] == 0) or (wr[0] == 0 and wr[1] != cr[1]) or first_code(wr[2]) != first_code(cr[2]):
                res["wc"].append(f"{tag}: run warm=({wr[0]},{wr[1][:80]!r},{wr[2][:160]!r}) cold=({cr[0]},{cr[1][:80]!r},{cr[2][:160]!r})")
            for name, r in (("warm check", wck), ("cold check", cck), ("warm run", wr), ("cold run", cr)):
                if r[0] == 0 and not rustc_ok:
                    res["fa"].append(f"{tag}: {name} accepts, rustc rejects: {rr[2][:200]!r}")
            if rustc_ok:
                for name, r in (("warm run", wr), ("cold run", cr)):
                    if r[0] == 0 and (r[0], r[1]) != (rout[0], rout[1]):
                        res["wrong"].append(f"{tag}: {name} out {r[1][:80]!r} rustc {rout[1][:80]!r}")
                if wck[0] != 0 and cck[0] != 0:
                    res["fr"] += 1
        bad = res["fa"] or res["wc"] or res["wrong"] or res["hang"]
        if bad and keep:
            d = os.path.join(keep, f"seed{seed}")
            os.makedirs(d, exist_ok=True)
            for i, src in enumerate(vs):
                with open(os.path.join(d, f"v{i + 1}.rs"), "w") as f:
                    f.write(src)
    finally:
        try:
            os.killpg(os.getpgid(daemon.pid), signal.SIGTERM)
        except Exception:
            daemon.terminate()
        log.close()
        ok, fails = oracle_log.collect(oracle_path)
        res["oracle_ok"] = ok
        res["oracle"] = oracle_log.problems(ok, fails, f"seed {seed}")
        shutil.rmtree(work, ignore_errors=True)
    return seed, res


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--count", type=int, default=100)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--step", type=int, default=1, help="seed stride (--seed 1 --step 6: round 5 programs only)")
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--keep", default="")
    ap.add_argument("--resolver", action="store_true", help="route every program to the resolver path")
    ap.add_argument("--proj", action="store_true", help="add round 7 projection shapes (P7_SHAPES) to every program")
    ap.add_argument("--contexts", default="", help="round 5 programs take template shapes of these contexts only (gtwice,gfnbound)")
    a = ap.parse_args()
    if a.contexts:
        os.environ["DSWEEP_CONTEXTS"] = a.contexts
    if a.proj:
        os.environ["DSWEEP_PROJ"] = "1"
    if not os.path.exists(LUST):
        print(f"{LUST} missing; run scripts/build.sh", file=sys.stderr)
        return 2
    seeds = [(a.seed + i * a.step, a.keep, a.resolver) for i in range(a.count)]
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
    print(f"derive-sweep: {a.count} programs, {tot['versions']} versions ({tot['rustc_rejects']} rustc rejects): "
          f"false_accepts={tot['fa']} warm_ne_cold={tot['wc']} wrong={tot['wrong']} hangs={tot['hang']} "
          f"false_rejects={tot['fr']}" + (f" oracle_checks={tot['oracle_ok']} oracle_failures={tot['oracle']}" if oracle_log.on() else ""))
    return 1 if tot["fa"] or tot["wc"] or tot["wrong"] or tot["hang"] or tot["oracle"] else 0


if __name__ == "__main__":
    sys.exit(main())
