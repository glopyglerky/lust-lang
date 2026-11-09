#!/usr/bin/env python3
# Lifetime name x position differential (fx-ltdecl). rustc rejects a lifetime
# name no generic list, `for<..>` binder, or impl header declares (E0261), and
# a nested item that names an outer item's lifetime (E0401). `'static` is
# valid everywhere a lifetime is; `'_` only in some positions (E0637); a loop
# label never declares a lifetime. Each template below is one position where
# a lifetime is written, with `{L}` for the probed name and `'a` declared by
# the enclosing scope where the position has one. Each template runs with
# five names: `'a` (declared), `'x` (undeclared), `'static`, `'_`, and `'lb`
# (a label of the same name is in scope). Each program runs on rustc 1.99 and
# a cold `lust check`.
#
# Classes: false accept (lust accepts, rustc rejects), false reject (lust
# rejects, rustc accepts), hang (a lust timeout; never a reject). With
# --baseline BIN, a false reject that BIN also has is old, not new. Exit 1 on
# a false accept, a new false reject, or a hang.
#
# Usage: python3 scripts/fuzz/lifetime_sweep.py [--jobs J] [--keep DIR] [--baseline BIN] [--only NAME]
# Env: LUST_BIN (default target/release/lust).
import argparse, os, shutil, subprocess, sys, tempfile
from concurrent.futures import ThreadPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
LUST = os.environ.get("LUST_BIN", os.path.join(ROOT, "target/release/lust"))
TIMEOUT = 60
RUSTC = ["rustc", "--edition", "2024", "--crate-type", "bin", "--emit=metadata", "-A", "warnings"]
NAMES = ["'a", "'x", "'static", "'_", "'lb"]

# (name, program). `{L}` is the probed lifetime. Each program is valid Rust
# with `{L}` = `'a` unless the position never takes a declared name.
TEMPLATES = [
    # Statics and consts: only 'static (or elided) is in scope.
    ("static_ref", "static S: &{L} u8 = &1; fn main() { let _ = S; }"),
    ("static_opt", "static S: Option<&{L} u8> = None; fn main() { let _ = S; }"),
    ("static_vec", "static S: Vec<&{L} str> = Vec::new(); fn main() { let _ = S.len(); }"),
    ("static_mutex", "static S: std::sync::Mutex<Vec<&{L} [u8]>> = std::sync::Mutex::new(Vec::new()); fn main() { let _ = S.lock(); }"),
    ("static_tuple", "static S: (u8, &{L} str) = (1, \"\"); fn main() { let _ = S; }"),
    ("static_array", "static S: [&{L} str; 1] = [\"\"]; fn main() { let _ = S; }"),
    ("static_fnptr", "fn g(x: &u8) -> u8 { *x } static S: fn(&{L} u8) -> u8 = g; fn main() { let _ = S; }"),
    ("static_dyn", "static S: &(dyn Sync + {L}) = &1u8; fn main() { let _ = S; }"),
    ("static_cow", "use std::borrow::Cow; static S: Option<Cow<{L}, str>> = None; fn main() { let _ = &S; }"),
    ("static_mut", "static mut S: Option<&{L} u8> = None; fn main() {}"),
    ("const_ref", "const C: &{L} str = \"\"; fn main() { let _ = C; }"),
    ("const_opt", "const C: Option<&{L} u8> = None; fn main() { let _ = C; }"),
    ("const_slice", "const C: &[&{L} str] = &[]; fn main() { let _ = C; }"),
    ("const_result", "const C: Result<&{L} str, u8> = Ok(\"\"); fn main() { let _ = C; }"),
    ("static_in_fn", "fn f<'a>(x: &'a u8) -> u8 { static S: Option<&{L} u8> = None; let _ = S; *x } fn main() { f(&1); }"),
    ("const_in_fn", "fn f<'a>(x: &'a u8) -> u8 { const C: Option<&{L} u8> = None; let _ = C; *x } fn main() { f(&1); }"),
    ("const_in_impl_fn", "struct W<'a>(&'a u8); impl<'a> W<'a> { fn m(&self) -> u8 { const C: Option<&{L} u8> = None; let _ = C; *self.0 } } fn main() { W(&1).m(); }"),
    ("assoc_const_impl", "struct W<'a>(&'a u8); impl<'a> W<'a> { const C: Option<&{L} u8> = None; } fn main() { let _ = W::C; }"),
    ("assoc_const_trait", "trait T<'a> { const C: Option<&{L} u8>; } fn main() {}"),
    ("static_hrtb_fnptr", "static S: Option<for<'b> fn(&'b u8, &{L} u8)> = None; fn main() { let _ = S; }"),
    ("static_dyn_hrtb", "static S: Option<&(dyn for<'b> Fn(&'b u8) -> &{L} u8 + Sync)> = None; fn main() { let _ = S; }"),
    ("static_block_const", "static S: u8 = { const C: Option<&{L} u8> = None; let _ = C; 1 }; fn main() { let _ = S; }"),
    ("const_in_closure", "fn f<'a>(x: &'a u8) -> u8 { let c = || { const C: Option<&{L} u8> = None; C }; let _ = c(); *x } fn main() { f(&1); }"),
    ("static_in_trait_default", "trait T<'a> { fn m(&self) -> u8 { static S: Option<&{L} u8> = None; let _ = S; 1 } } fn main() {}"),
    ("trait_impl_const", "trait T { const C: Option<&'static u8>; } impl T for u8 { const C: Option<&{L} u8> = None; } fn main() { let _ = <u8 as T>::C; }"),
    ("trait_impl_const_params", "trait T { const C: Option<&'static u8>; } struct W<'a>(&'a u8); impl<'a> T for W<'a> { const C: Option<&{L} u8> = None; } fn main() { let _ = <W<'static> as T>::C; }"),
    # Fn signatures.
    ("fn_arg", "fn f<'a>(x: &{L} u8) -> u8 { *x } fn main() { f(&1); }"),
    ("fn_ret", "fn f<'a>(x: &'a u8) -> Option<&{L} u8> { let _ = x; None } fn main() { f(&1); }"),
    ("fn_ret_nested", "fn f<'a>(x: &'a u8) -> Vec<Option<&{L} u8>> { let _ = x; Vec::new() } fn main() { f(&1); }"),
    ("fn_noparams_arg", "fn f(x: &{L} u8) -> u8 { *x } fn main() { f(&1); }"),
    ("fn_cow_arg", "use std::borrow::Cow; fn f<'a>(x: Cow<{L}, str>) -> usize { x.len() } fn main() { f(Cow::Borrowed(\"\")); }"),
    ("fn_ref_cell", "use std::cell::{Ref, RefCell}; fn f<'a>(x: Ref<{L}, u8>) -> u8 { *x } fn main() { let c = RefCell::new(1); f(c.borrow()); }"),
    ("fn_guard", "use std::sync::{Mutex, MutexGuard}; fn f<'a>(x: MutexGuard<{L}, u8>) -> u8 { *x } fn main() { let m = Mutex::new(1); f(m.lock().unwrap()); }"),
    ("fn_fnptr_arg", "fn f<'a>(g: fn(&{L} u8) -> u8) -> u8 { g(&1) } fn h(x: &u8) -> u8 { *x } fn main() { f(h); }"),
    ("fn_dyn_arg", "fn f<'a>(g: &dyn Fn(&{L} u8) -> u8) -> u8 { g(&1) } fn main() { f(&|x: &u8| *x); }"),
    ("fn_dyn_bound", "fn f<'a>(g: Box<dyn Fn() -> u8 + {L}>) -> u8 { g() } fn main() { f(Box::new(|| 1)); }"),
    ("fn_impl_arg", "fn f<'a>(g: impl Fn(&{L} u8) -> u8) -> u8 { g(&1) } fn main() { f(|x: &u8| *x); }"),
    ("fn_impl_ret_bound", "fn f<'a>(x: &'a u8) -> impl Sized + {L} { let _ = x; 1u8 } fn main() { f(&1); }"),
    ("fn_impl_ret_use", "fn f<'a>(x: &'a u8) -> impl Sized + use<{L}> { let _ = x; 1u8 } fn main() { f(&1); }"),
    ("fn_param_bound", "fn f<'a, 'b: {L}>(x: &'a u8, y: &'b u8) -> u8 { *x + *y } fn main() { f(&1, &2); }"),
    ("fn_type_bound", "fn f<'a, T: {L}>(x: &'a u8, t: T) -> u8 { let _ = t; *x } fn main() { f(&1, 2u8); }"),
    ("fn_where_bound", "fn f<'a, T>(x: &'a u8, t: T) -> u8 where T: {L} { let _ = t; *x } fn main() { f(&1, 2u8); }"),
    ("fn_where_ref", "fn f<'a, T>(x: &'a u8, t: T) -> u8 where &{L} T: Copy { let _ = t; *x } fn main() { f(&1, 2u8); }"),
    ("fn_hrtb_ret", "fn f<'a>(g: &dyn for<'b> Fn(&'b u8) -> &{L} u8) -> u8 { *g(&1) } fn main() { f(&|_: &u8| &1); }"),
    ("fn_hrtb_where", "fn f<'a, F>(g: F) -> u8 where for<'b> F: Fn(&'b u8, &{L} u8) -> u8 { g(&1, &2) } fn main() { f(|x: &u8, y: &u8| *x + *y); }"),
    ("fn_hrtb_bound", "fn f<'a, F: for<'b> Fn(&'b u8) -> &{L} u8>(g: F) -> u8 { *g(&1) } fn main() { f(|_: &u8| &1); }"),
    ("fn_phantom", "use std::marker::PhantomData; fn f<'a>(p: PhantomData<&{L} ()>) { let _ = p; } fn main() { f(PhantomData); }"),
    ("fn_tuple_slice", "fn f<'a>(x: (&{L} [u8], u8)) -> u8 { x.1 } fn main() { f((&[1], 2)); }"),
    ("fn_apit_bound", "fn f<'a>(x: &'a u8, t: impl Sized + {L}) -> u8 { let _ = t; *x } fn main() { f(&1, 2u8); }"),
    ("fn_where_lt", "fn f<'a, 'b>(x: &'a u8, y: &'b u8) -> u8 where 'b: {L} { *x + *y } fn main() { f(&1, &2); }"),
    ("fn_assoc_binding", "fn f<'a>(x: &'a u8, i: impl Iterator<Item = &{L} u8>) -> u8 { let _ = i; *x } fn main() { f(&1, [&2u8].into_iter()); }"),
    # Bodies: annotations, turbofish, casts, closures, labels.
    ("body_let", "fn f<'a>(x: &'a u8) -> u8 { let y: &{L} u8 = &3; *x + *y } fn main() { f(&1); }"),
    ("body_let_opt", "fn f<'a>(x: &'a u8) -> u8 { let y: Option<&{L} u8> = None; let _ = y; *x } fn main() { f(&1); }"),
    ("body_turbofish", "fn f<'a>(x: &'a u8) -> u8 { let v = Vec::<&{L} u8>::new(); let _ = v; *x } fn main() { f(&1); }"),
    ("body_cast", "fn f<'a>(x: &'a u8) -> u8 { let y = &3u8 as &{L} u8; *x + *y } fn main() { f(&1); }"),
    ("body_closure_arg", "fn f<'a>(x: &'a u8) -> u8 { let c = |y: &{L} u8| *y; c(&2) + *x } fn main() { f(&1); }"),
    ("body_closure_ret", "fn f<'a>(x: &'a u8) -> u8 { let c = || -> Option<&{L} u8> { None }; let _ = c(); *x } fn main() { f(&1); }"),
    ("body_box_dyn", "fn f<'a>(x: &'a u8) -> u8 { let b: Box<dyn Fn() -> u8 + {L}> = Box::new(|| 1); b() + *x } fn main() { f(&1); }"),
    ("body_noparams_let", "fn f(x: &u8) -> u8 { let y: Option<&{L} u8> = None; let _ = y; *x } fn main() { f(&1); }"),
    ("body_main_let", "fn main() { let y: Option<&{L} u8> = None; let _ = y; }"),
    ("body_in_label", "fn main() { 'lb: loop { let y: Option<&{L} u8> = None; let _ = y; break 'lb; } }"),
    ("label_then_use", "fn f<'a>(x: &'a u8) -> u8 { 'lb: loop { break 'lb; } let y: Option<&{L} u8> = None; let _ = y; *x } fn main() { f(&1); }"),
    ("nested_fn", "fn f<'a>(x: &'a u8) -> u8 { fn g(y: &{L} u8) -> u8 { *y } g(x) } fn main() { f(&1); }"),
    ("nested_struct", "fn f<'a>(x: &'a u8) -> u8 { struct In<'c>(&'c u8, Option<&{L} u8>); In(x, None).0.clone() } fn main() { f(&1); }"),
    ("closure_in_static_fn", "fn main() { let c = |y: &u8| { let z: Option<&{L} u8> = None; let _ = z; *y }; c(&1); }"),
    # Structs, enums, unions of fields.
    ("struct_field", "struct S<'a> { x: &'a u8, y: Option<&{L} u8> } fn main() { let s = S { x: &1, y: None }; let _ = (s.x, s.y); }"),
    ("struct_noparams", "struct S { y: Option<&{L} u8> } fn main() { let s = S { y: None }; let _ = s.y; }"),
    ("tuple_struct", "struct S<'a>(&'a u8, Option<&{L} u8>); fn main() { let s = S(&1, None); let _ = (s.0, s.1); }"),
    ("struct_box_dyn", "struct S<'a> { x: &'a u8, f: Box<dyn Fn() -> u8 + {L}> } fn main() { let s = S { x: &1, f: Box::new(|| 1) }; let _ = (s.x, (s.f)()); }"),
    ("struct_where", "struct S<'a, T> where T: {L} { x: &'a u8, t: T } fn main() { let s = S { x: &1, t: 2u8 }; let _ = (s.x, s.t); }"),
    ("struct_param_bound", "struct S<'a, 'b: {L}> { x: &'a u8, y: &'b u8 } fn main() { let s = S { x: &1, y: &2 }; let _ = (s.x, s.y); }"),
    ("enum_variant", "enum E<'a> { A(&'a u8), B(Option<&{L} u8>) } fn main() { let e = E::B(None); if let E::A(x) | E::B(Some(x)) = e { let _ = x; } }"),
    ("enum_noparams", "enum E { A(Option<&{L} u8>) } fn main() { let E::A(x) = E::A(None); let _ = x; }"),
    ("enum_struct_variant", "enum E<'a> { A { x: &'a u8, y: Option<&{L} u8> } } fn main() { let E::A { x, y } = E::A { x: &1, y: None }; let _ = (x, y); }"),
    # Traits and impls.
    ("trait_method_arg", "trait T<'a> { fn m(&self, x: &{L} u8) -> u8; } fn main() {}"),
    ("trait_method_default", "trait T<'a> { fn m(&self, x: &{L} u8) -> u8 { *x } } impl<'a> T<'a> for u8 {} fn main() { 1u8.m(&2); }"),
    ("trait_noparams", "trait T { fn m(&self) -> Option<&{L} u8>; } fn main() {}"),
    ("trait_assoc_bound", "trait T<'a> { type Out: Sized + {L}; } fn main() {}"),
    ("trait_super", "trait T<'a>: Sized + {L} {} fn main() {}"),
    ("trait_where", "trait T<'a> where Self: {L} {} fn main() {}"),
    ("impl_header_self", "trait T { fn m(&self) -> u8; } impl<'a> T for &{L} u8 { fn m(&self) -> u8 { **self } } fn main() { (&1u8).m(); }"),
    ("impl_header_noparams", "trait T { fn m(&self) -> u8; } impl T for &{L} u8 { fn m(&self) -> u8 { **self } } fn main() { (&1u8).m(); }"),
    ("impl_header_trait", "trait T<'a> { fn m(&self) -> u8; } impl<'a> T<{L}> for u8 { fn m(&self) -> u8 { *self } } fn main() { 1u8.m(); }"),
    ("impl_header_adt", "struct W<'a>(&'a u8); impl<'a> W<{L}> { fn m(&self) -> u8 { *self.0 } } fn main() { W(&1).m(); }"),
    ("impl_header_inband", "struct W<'a>(&'a u8); impl W<{L}> { fn m(&self) -> u8 { *self.0 } } fn main() { W(&1).m(); }"),
    ("impl_where", "struct W<'a, T>(&'a u8, T); impl<'a, T> W<'a, T> where T: {L} { fn m(&self) -> u8 { *self.0 } } fn main() { W(&1, 2u8).m(); }"),
    ("impl_method_arg", "struct W<'a>(&'a u8); impl<'a> W<'a> { fn m(&self, y: &{L} u8) -> u8 { *self.0 + *y } } fn main() { W(&1).m(&2); }"),
    ("impl_method_ret", "struct W<'a>(&'a u8); impl<'a> W<'a> { fn m(&self) -> Option<&{L} u8> { None } } fn main() { W(&1).m(); }"),
    ("impl_method_body", "struct W<'a>(&'a u8); impl<'a> W<'a> { fn m(&self) -> u8 { let y: Option<&{L} u8> = None; let _ = y; *self.0 } } fn main() { W(&1).m(); }"),
    ("impl_assoc_type", "trait T { type Out; fn m(&self) -> u8; } struct W<'a>(&'a u8); impl<'a> T for W<'a> { type Out = Option<&{L} u8>; fn m(&self) -> u8 { *self.0 } } fn main() { W(&1).m(); }"),
    ("impl_nested_fn", "struct W<'a>(&'a u8); impl<'a> W<'a> { fn m(&self) -> u8 { fn g(y: &{L} u8) -> u8 { *y } g(self.0) } } fn main() { W(&1).m(); }"),
    ("trait_impl_method", "trait T<'a> { fn m(&self, x: &'a u8) -> u8; } impl<'a> T<'a> for u8 { fn m(&self, x: &{L} u8) -> u8 { *x } } fn main() { 1u8.m(&2); }"),
    ("impl_assoc_type_fnptr", "trait T { type Out; fn m(&self) -> u8; } struct W<'a>(&'a u8); impl<'a> T for W<'a> { type Out = fn(&{L} u8); fn m(&self) -> u8 { *self.0 } } fn main() { W(&1).m(); }"),
    ("impl_assoc_type_dyn", "trait T { type Out; fn m(&self) -> u8; } struct W<'a>(&'a u8); impl<'a> T for W<'a> { type Out = Box<dyn Fn() -> u8 + {L}>; fn m(&self) -> u8 { *self.0 } } fn main() { W(&1).m(); }"),
    ("struct_where_lt", "struct S<'a, 'b> where 'b: {L} { x: &'a u8, y: &'b u8 } fn main() { let s = S { x: &1, y: &2 }; let _ = (s.x, s.y); }"),
    # Type aliases.
    ("alias_ref", "type A = &{L} u8; fn main() { let x: A = &1; let _ = x; }"),
    ("alias_params", "type A<'a> = Option<&{L} u8>; fn main() { let x: A = None; let _ = x; }"),
    ("alias_fnptr", "type A = fn(&{L} u8) -> u8; fn g(x: &u8) -> u8 { *x } fn main() { let x: A = g; let _ = x; }"),
    ("alias_dyn", "type A = Box<dyn Fn() -> u8 + {L}>; fn main() { let x: A = Box::new(|| 1); let _ = x(); }"),
    ("alias_cow", "use std::borrow::Cow; type A = Option<Cow<{L}, str>>; fn main() { let x: A = None; let _ = x; }"),
    ("alias_params_cow", "type A<'a> = std::borrow::Cow<{L}, str>; fn main() { let x: A = std::borrow::Cow::Borrowed(\"\"); let _ = x; }"),
    ("alias_dyn_item", "type A = Box<dyn Iterator<Item = &{L} u8>>; fn main() { let x: A = Box::new(std::iter::empty()); let _ = x; }"),
    ("alias_hrtb", "type A = for<'b> fn(&'b u8, &{L} u8); fn g(_: &u8, _: &u8) {} fn main() { let x: A = g; let _ = x; }"),
]


def program(src, name):
    return src.replace("{L}", name)


def run(cmd, cwd, env):
    try:
        r = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=TIMEOUT)
        return r.returncode, r.stderr
    except subprocess.TimeoutExpired:
        return None, "timeout"


def case(args):
    tname, name, src, baseline = args
    work = tempfile.mkdtemp(prefix="lust-lt-")
    try:
        main = os.path.join(work, "main.rs")
        with open(main, "w") as f:
            f.write(src + "\n")
        env = dict(os.environ, LUST_NO_DAEMON="1", LUST_NO_CACHE="1")
        rc, rerr = run(RUSTC + ["-o", os.path.join(work, "out"), main], work, env)
        lc, lerr = run([LUST, "check", main], work, env)
        bc = run([baseline, "check", main], work, env)[0] if baseline else None
        return tname, name, src, rc == 0, lc, lerr, bc, rerr
    finally:
        shutil.rmtree(work, ignore_errors=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--jobs", type=int, default=os.cpu_count())
    ap.add_argument("--keep", help="write each finding's program here")
    ap.add_argument("--baseline", help="old lust binary; its false rejects are not new")
    ap.add_argument("--only", help="run only templates whose name contains this")
    a = ap.parse_args()
    if a.baseline:
        # A baseline that cannot check would make every false reject "old".
        work = tempfile.mkdtemp(prefix="lust-lt-")
        with open(os.path.join(work, "main.rs"), "w") as f:
            f.write("fn main() {}\n")
        bc = run([a.baseline, "check", os.path.join(work, "main.rs")], work, dict(os.environ, LUST_NO_DAEMON="1"))[0]
        shutil.rmtree(work, ignore_errors=True)
        if bc != 0:
            sys.exit(f"lifetime sweep: baseline {a.baseline} cannot check `fn main() {{}}`")
    cases = [(t, n, program(s, n), a.baseline) for t, s in TEMPLATES if not a.only or a.only in t for n in NAMES]
    fa, fr, old_fr, hang, agree = [], [], [], [], 0
    with ThreadPoolExecutor(a.jobs) as ex:
        for tname, name, src, rok, lc, lerr, bc, rerr in ex.map(case, cases):
            tag = f"{tname} {name}"
            if lc is None:
                hang.append((tag, src, lerr))
            elif rok != (lc == 0):
                if lc == 0:
                    fa.append((tag, src, rerr.strip().splitlines()[0] if rerr.strip() else ""))
                elif bc not in (None, 0):
                    old_fr.append((tag, src, lerr.strip().splitlines()[0] if lerr.strip() else ""))
                else:
                    fr.append((tag, src, lerr.strip().splitlines()[0] if lerr.strip() else ""))
            else:
                agree += 1
    for label, xs in (("FALSE ACCEPT", fa), ("NEW FALSE REJECT", fr), ("old false reject", old_fr), ("HANG", hang)):
        for tag, src, msg in xs:
            print(f"{label}: {tag}: {msg}")
    if a.keep:
        os.makedirs(a.keep, exist_ok=True)
        for label, xs in (("fa", fa), ("fr", fr), ("oldfr", old_fr), ("hang", hang)):
            for tag, src, _ in xs:
                fn = f"{label}-{tag.replace(' ', '-').replace(chr(39), '')}.rs"
                with open(os.path.join(a.keep, fn), "w") as f:
                    f.write(src + "\n")
    print(f"lifetime sweep: {len(cases)} programs, {agree} agree, {len(fa)} false accepts, "
          f"{len(fr)} new false rejects, {len(old_fr)} old false rejects, {len(hang)} hangs")
    sys.exit(1 if fa or fr or hang else 0)


if __name__ == "__main__":
    main()
