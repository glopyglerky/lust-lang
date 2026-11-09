#!/usr/bin/env python3
# Definition checks (fx-defcheck, SOL-COVERAGE-4 section C): a written
# type or bound in a definition must be well-formed where it is
# written, with no use of the item, exactly where rustc checks it
# (aliases and multi-param defaults are lazy in rustc and must stay
# accepted). Each generated program pairs one owner kind (alias,
# trait const, supertrait, fn/struct bound, impl header, default
# parameter, const item, trait default body, generic fn body,
# derive, impl supertrait obligation) with a valid or invalid inner
# type or bound, used or unused.
#
# Every case runs rustc 1.99, cold `lust check`, and (when both
# accept) built binaries compared on stdout and exit code. A lust
# timeout is a hang, never a reject. Exit 1 on any false accept
# (rustc rejects, lust check or build accepts), wrong output, or
# hang. False rejects are listed for review (lust is a subset).
#
# Usage: python3 scripts/defcheck_sweep.py [--jobs J] [--family F]
#        [--keep DIR] [--daemon]   (needs target/release/lust built)
# --daemon also runs every case through a warm daemon (a valid
# prime, then the case as an edit) and requires the warm check's
# stderr to equal the cold one's byte for byte (class warm-diff).
import argparse, os, subprocess, sys, tempfile, time
from concurrent.futures import ProcessPoolExecutor

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LUST = os.path.join(ROOT, "target/release/lust")

NEED = "struct Need<T: Copy>(T);\n"
SUP = "trait Sup<T: Copy> {}\n"
MAIN0 = "fn main() {}\n"
MAINP = 'fn main() { println!("ok"); }\n'

# An argument state: (name, type text, value expr of that type).
GOOD = ("good", "u8", "1u8")
BAD = ("bad", "String", "String::new()")
ARGS = [GOOD, BAD]

CASES = []  # (family, name, source)


def case(family, name, src):
    CASES.append((family, name, src))


def build_cases():
    for an, aty, aexpr in ARGS:
        case("alias", f"alias_unused_{an}", NEED + f"type X = Need<{aty}>;\n" + MAINP)
        case("alias", f"alias_nested_unused_{an}", NEED + f"type X = Option<Need<{aty}>>;\n" + MAINP)
        case("alias", f"alias_used_sig_{an}", NEED + f"type X = Need<{aty}>;\nfn f(_x: X) {{}}\n" + MAINP)
        case("alias", f"alias_used_let_{an}", NEED + f"type X = Need<{aty}>;\nfn main() {{ let _x: Option<X> = None; println!(\"ok\"); }}\n")
        case("alias", f"alias_used_field_{an}", NEED + f"type X = Need<{aty}>;\nstruct S {{ f: X }}\n" + MAINP)
        case("alias", f"alias_used_ret_{an}", NEED + f"type X = Need<{aty}>;\nfn f() -> X {{ Need({aexpr}) }}\n" + MAINP)
        case("alias", f"alias_generic_used_{an}", NEED + f"type X<T> = Need<T>;\nfn main() {{ let _x: Option<X<{aty}>> = None; println!(\"ok\"); }}\n")
        case("alias", f"alias_default_{an}", f"type X<T: Copy = {aty}> = Option<T>;\n" + MAINP)
        case("alias", f"alias_default_used_{an}", f"type X<T: Copy = {aty}> = Option<T>;\nfn main() {{ let _x: X = None; println!(\"ok\"); }}\n")
        case("traitconst", f"trait_const_req_{an}", NEED + f"trait Tr {{ const C: Need<{aty}>; }}\n" + MAINP)
        case("traitconst", f"trait_const_nested_{an}", NEED + f"trait Tr {{ const C: Option<Need<{aty}>>; }}\n" + MAINP)
        case("traitconst", f"trait_const_default_{an}", NEED + f"trait Tr {{ const C: Need<{aty}> = Need({aexpr}); }}\n" + MAINP)
        case("bound", f"supertrait_{an}", SUP + f"trait Sub: Sup<{aty}> {{}}\n" + MAINP)
        case("bound", f"supertrait_where_{an}", SUP + f"trait Sub where Self: Sup<{aty}> {{}}\n" + MAINP)
        case("bound", f"fn_bound_{an}", SUP + f"fn f<T: Sup<{aty}>>(_x: T) {{}}\n" + MAINP)
        case("bound", f"fn_where_{an}", SUP + f"fn f<T>(_x: T) where T: Sup<{aty}> {{}}\n" + MAINP)
        case("bound", f"fn_impl_trait_{an}", SUP + f"fn f(_x: impl Sup<{aty}>) {{}}\n" + MAINP)
        case("bound", f"struct_bound_{an}", SUP + f"struct S<T: Sup<{aty}>>(T);\n" + MAINP)
        case("bound", f"enum_bound_{an}", SUP + f"enum E<T: Sup<{aty}>> {{ V(T) }}\n" + MAINP)
        case("bound", f"assoc_bound_{an}", SUP + f"trait Tr {{ type A2: Sup<{aty}>; }}\n" + MAINP)
        case("bound", f"dyn_bound_{an}", SUP + f"fn f(_x: &dyn Sup<{aty}>) {{}}\n" + MAINP)
        case("bound", f"alias_bound_{an}", SUP + f"type X<T: Sup<{aty}>> = Option<T>;\n" + MAINP)
        case("impl", f"impl_trait_arg_{an}", SUP + f"struct S;\nimpl Sup<{aty}> for S {{}}\n" + MAINP)
        case("impl", f"impl_self_ty_{an}", NEED + f"trait Tr2 {{}}\nimpl Tr2 for Need<{aty}> {{}}\n" + MAINP)
        case("impl", f"impl_assoc_ty_{an}", NEED + f"trait Tr {{ type A2; }}\nstruct S;\nimpl Tr for S {{ type A2 = Need<{aty}>; }}\n" + MAINP)
        case("impl", f"binding_ty_{an}", NEED + f"trait Tr {{ type A2; }}\nfn f<T: Tr<A2 = Need<{aty}>>>(_x: T) {{}}\n" + MAINP)
        case("direct", f"fn_sig_{an}", NEED + f"fn f(_x: Need<{aty}>) {{}}\n" + MAINP)
        case("direct", f"fn_ret_{an}", NEED + f"fn f() -> Need<{aty}> {{ Need({aexpr}) }}\n" + MAINP)
        case("direct", f"field_{an}", NEED + f"struct S {{ f: Need<{aty}> }}\n" + MAINP)
        case("direct", f"enum_field_{an}", NEED + f"enum E {{ V(Need<{aty}>) }}\n" + MAINP)
        case("direct", f"const_item_{an}", NEED + f"const C: Need<{aty}> = Need({aexpr});\n" + MAINP)
        case("direct", f"inherent_const_{an}", NEED + f"struct S;\nimpl S {{ const C: Need<{aty}> = Need({aexpr}); }}\n" + MAINP)
        case("default", f"default_single_unused_{an}", f"struct D<T: Copy = {aty}>(T);\n" + MAINP)
        case("default", f"default_single_used_{an}", f"struct D<T: Copy = {aty}>(T);\nfn main() {{ let _x: Option<D> = None; println!(\"ok\"); }}\n")
        case("default", f"trait_default_unused_{an}", f"trait Tr<T: Copy = {aty}> {{}}\n" + MAINP)
        case("default", f"trait_default_bound_{an}", f"trait Tr<T: Copy = {aty}> {{}}\nfn f<X: Tr>(_x: X) {{}}\n" + MAINP)
    # Generic-parameter forms: the bound comes from the item env or not.
    case("alias", "alias_generic_unused_free", NEED + "type X<T> = Need<T>;\n" + MAINP)
    case("alias", "alias_generic_unused_bound", NEED + "type X<T: Copy> = Need<T>;\n" + MAINP)
    case("traitconst", "trait_const_generic_free", NEED + "trait Tr<T> { const C: Need<T>; }\n" + MAINP)
    case("traitconst", "trait_const_generic_bound", NEED + "trait Tr<T: Copy> { const C: Need<T>; }\n" + MAINP)
    case("traitconst", "trait_const_generic_where", NEED + "trait Tr<T> where T: Copy { const C: Need<T>; }\n" + MAINP)
    case("traitconst", "trait_const_self_assoc_bound", NEED + "trait Tr { type A2: Copy; const C: Need<Self::A2>; }\n" + MAINP)
    case("traitconst", "trait_const_self_assoc_free", NEED + "trait Tr { type A2; const C: Need<Self::A2>; }\n" + MAINP)
    case("traitconst", "trait_const_unsized", "trait Tr { const C: [u8]; }\n" + MAINP)
    case("bound", "supertrait_param_bound", SUP + "trait Sub<T: Copy>: Sup<T> {}\n" + MAINP)
    case("bound", "supertrait_param_free", SUP + "trait Sub<T>: Sup<T> {}\n" + MAINP)
    case("bound", "fn_bound_env_proof", SUP + "fn f<T, X: Copy>(_x: T, _y: X) where T: Sup<X> {}\n" + MAINP)
    case("bound", "fn_bound_env_missing", SUP + "fn f<T, X>(_x: T, _y: X) where T: Sup<X> {}\n" + MAINP)
    case("bound", "fn_bound_super_chain", "trait Sub: Copy {}\ntrait SupS<T: Sub> {}\nfn f<T, X: Sub>(_x: T, _y: X) where T: SupS<X> {}\n" + MAINP)
    case("bound", "multi_param_trait_gg", "trait Sup2<T: Copy, U: Clone> {}\nfn f<X: Sup2<u8, String>>(_x: X) {}\n" + MAINP)
    case("bound", "multi_param_trait_bg", "trait Sup2<T: Copy, U: Clone> {}\nfn f<X: Sup2<String, String>>(_x: X) {}\n" + MAINP)
    case("bound", "multi_param_trait_gb", "struct NoClone;\ntrait Sup2<T: Copy, U: Clone> {}\nfn f<X: Sup2<u8, NoClone>>(_x: X) {}\n" + MAINP)
    case("bound", "multi_param_trait_bb", "struct NoClone;\ntrait Sup2<T: Copy, U: Clone> {}\nfn f<X: Sup2<String, NoClone>>(_x: X) {}\n" + MAINP)
    case("bound", "where_form_trait_param", "trait SupW<T> where T: Copy {}\nfn f<X: SupW<String>>(_x: X) {}\n" + MAINP)
    case("bound", "where_form_trait_param_ok", "trait SupW<T> where T: Copy {}\nfn f<X: SupW<u8>>(_x: X) {}\n" + MAINP)
    # Defaults that mention two parameters (C4): lazy unused, checked on use.
    pair = "trait Pair<B> {}\nstruct D<A: Pair<B> = u8, B = u16>(A, B);\n"
    pair_impl = "trait Pair<B> {}\nimpl Pair<u16> for u8 {}\nstruct D<A: Pair<B> = u8, B = u16>(A, B);\n"
    for tag, pre in (("noimpl", pair), ("impl", pair_impl)):
        case("default", f"default_pair_unused_{tag}", pre + MAINP)
        case("default", f"default_pair_sig_{tag}", pre + "fn f(_x: D) {}\n" + MAINP)
        case("default", f"default_pair_let_{tag}", pre + 'fn main() { let _x: Option<D> = None; println!("ok"); }\n')
        case("default", f"default_pair_ctor_{tag}", pre + 'fn main() { let _ = D(1u8, 2u16); println!("ok"); }\n')
        case("default", f"default_pair_field_{tag}", pre + "struct S { f: D }\n" + MAINP)
    case("default", "default_pair_one_default", "trait Pair<B> {}\nstruct D<A: Pair<B>, B = u16>(A, B);\nfn f(_x: D<u8>) {}\n" + MAINP)
    case("default", "default_dep_on_param", "struct D<T, U = Vec<T>>(U);\n" + MAINP)
    # A where-form Adt bound through an alias and a field.
    needw = "struct NeedW<T>(T) where T: Copy;\n"
    for an, aty, _ in ARGS:
        case("alias", f"alias_where_unused_{an}", needw + f"type X = NeedW<{aty}>;\n" + MAINP)
        case("alias", f"alias_where_used_{an}", needw + f"type X = NeedW<{aty}>;\nfn f(_x: X) {{}}\n" + MAINP)
        case("traitconst", f"trait_const_where_{an}", needw + f"trait Tr {{ const C: NeedW<{aty}>; }}\n" + MAINP)
    # C6: never-read const bodies, every owner.
    bad_const = "const BAD: u8 = if true { 1 } else { false };\n"
    ok_const = "const GOOD: u8 = if true { 1 } else { 2 };\n"
    case("constbody", "const_dead_branch", bad_const + MAINP)
    case("constbody", "const_dead_branch_ok", ok_const + MAINP)
    case("constbody", "const_inherent_dead", "struct S;\nimpl S { const BAD: u8 = if true { 1 } else { false }; }\n" + MAINP)
    case("constbody", "const_trait_default_dead", "trait Tr { const BAD: u8 = if true { 1 } else { false }; }\n" + MAINP)
    case("constbody", "const_trait_default_ok", "trait Tr { const GOOD: u8 = if true { 1 } else { 2 }; }\n" + MAINP)
    # C7: never-called trait default bodies.
    case("body", "trait_default_body_bad", "trait Tr { fn f(&self) { let _: String = 1u8.into(); } }\n" + MAINP)
    case("body", "trait_default_body_ok", "trait Tr { fn f(&self) -> u8 { 1u8 } }\n" + MAINP)
    case("body", "trait_default_body_move", "trait Tr<T> { fn f(&self, x: T) { let y = x; let _z = x; let _ = y; } }\n" + MAINP)
    # C8: never-called generic fns.
    case("body", "generic_fn_move_twice", "fn bad<T>(x: T) { let y = x; let z = x; let _ = (y, z); }\n" + MAINP)
    case("body", "generic_fn_copy_ok", "fn good<T: Copy>(x: T) -> (T, T) { (x, x) }\n" + MAINP)
    case("body", "generic_fn_type_err", "fn bad<T>(_x: T) -> u8 { false }\n" + MAINP)
    case("body", "generic_fn_closure_move", "fn bad<T>(x: T) { let f = move || { let _ = &x; }; let _z = x; f(); }\n" + MAINP)
    # C5: unused generic derives (regression witnesses).
    case("derive", "derive_missing_inner", "struct Inner<T>(T);\n#[derive(Debug)]\nstruct Outer<T> { inner: Inner<T> }\n" + MAINP)
    case("derive", "derive_inner_ok", "#[derive(Debug)]\nstruct Inner<T>(T);\n#[derive(Debug)]\nstruct Outer<T> { inner: Inner<T> }\n" + MAINP)
    case("derive", "derive_stronger_inner",
         "use std::fmt;\nstruct Inner<T>(T);\nimpl<T: fmt::Debug + Clone> fmt::Debug for Inner<T> { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, \"I\") } }\n#[derive(Debug)]\nstruct Outer<T> { inner: Inner<T> }\n" + MAINP)
    # C3 family: an impl owes its trait's supertraits at its own args.
    case("implsuper", "super_lifetime_static_only",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<&'static u8> for S {}\nimpl<'a> NeedT<&'a u8> for S where S: Tag<&'static u8> {}\n" + MAINP)
    case("implsuper", "super_lifetime_no_where",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<&'static u8> for S {}\nimpl<'a> NeedT<&'a u8> for S {}\n" + MAINP)
    case("implsuper", "super_lifetime_ok",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl<'a> Tag<&'a u8> for S {}\nimpl<'a> NeedT<&'a u8> for S {}\n" + MAINP)
    case("implsuper", "super_lifetime_where_exact",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl<'a> NeedT<&'a u8> for S where S: Tag<&'a u8> {}\n" + MAINP)
    case("implsuper", "super_type_arg_mismatch",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<u8> for S {}\nimpl NeedT<u16> for S where S: Tag<u8> {}\n" + MAINP)
    case("implsuper", "super_type_arg_ok",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<u16> for S {}\nimpl NeedT<u16> for S {}\n" + MAINP)
    case("implsuper", "super_generic_where_other",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl<X> NeedT<X> for S where S: Tag<u8> {}\n" + MAINP)
    case("implsuper", "super_generic_where_exact",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl<X> NeedT<X> for S where S: Tag<X> {}\n" + MAINP)
    case("implsuper", "super_nested_chain_bad",
         "trait Tag<A> {}\ntrait Mid<A>: Tag<A> {}\ntrait NeedT<A>: Mid<A> {}\nstruct S;\nimpl Tag<&'static u8> for S {}\nimpl<'a> Mid<&'a u8> for S where S: Tag<&'static u8> {}\nimpl<'a> NeedT<&'a u8> for S {}\n" + MAINP)
    case("implsuper", "super_two_supers_one_bad",
         "trait Tag<A> {}\ntrait Tag2<A> {}\ntrait NeedT<A>: Tag<A> + Tag2<A> {}\nstruct S;\nimpl<'a> Tag<&'a u8> for S {}\nimpl Tag2<&'static u8> for S {}\nimpl<'a> NeedT<&'a u8> for S where S: Tag2<&'static u8> {}\n" + MAINP)
    case("implsuper", "super_static_arg_ok",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<&'static u8> for S {}\nimpl NeedT<&'static u8> for S {}\n" + MAINP)
    case("implsuper", "super_ref_in_tuple_bad",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<(&'static u8, u8)> for S {}\nimpl<'a> NeedT<(&'a u8, u8)> for S where S: Tag<(&'static u8, u8)> {}\n" + MAINP)
    case("implsuper", "super_mut_ref_bad",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<&'static mut u8> for S {}\nimpl<'a> NeedT<&'a mut u8> for S where S: Tag<&'static mut u8> {}\n" + MAINP)
    case("implsuper", "super_slice_ref_bad",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<&'static [u8]> for S {}\nimpl<'a> NeedT<&'a [u8]> for S where S: Tag<&'static [u8]> {}\n" + MAINP)
    case("implsuper", "super_box_ref_bad",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<Box<&'static u8>> for S {}\nimpl<'a> NeedT<Box<&'a u8>> for S where S: Tag<Box<&'static u8>> {}\n" + MAINP)
    case("implsuper", "super_second_lifetime_bad",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl<'a> Tag<(&'a u8, &'static u8)> for S {}\nimpl<'a, 'b> NeedT<(&'a u8, &'b u8)> for S where S: Tag<(&'a u8, &'static u8)> {}\n" + MAINP)
    case("implsuper", "super_generic_ref_where_ok",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl<'a, X> NeedT<&'a X> for S where S: Tag<&'a X> {}\n" + MAINP)
    case("implsuper", "super_generic_ref_where_static",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl<'a, X> NeedT<&'a X> for S where S: Tag<&'static X> {}\n" + MAINP)
    # Matrix: argument shape x where-clause region relation. The
    # obligation always wants region 'a; the clause gives 'static,
    # 'a itself, 'b with a fact one way, or an Adt around it.
    shapes = [
        ("ref", "&{r} u8"), ("mutref", "&{r} mut u8"), ("tuple", "(&{r} u8, u16)"),
        ("slice", "&{r} [u8]"), ("box", "Box<&{r} u8>"), ("nest", "&{r} &{r} u8"),
        ("adtlt", "W<{r}>"),
    ]
    for sn, shp in shapes:
        decl = "struct W<'x>(&'x u8);\n" if sn == "adtlt" else ""
        head = "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\n" + decl
        a_static = shp.format(r="'static")
        a_a = shp.format(r="'a")
        a_b = shp.format(r="'b")
        case("implsuper", f"mx_{sn}_static_impl",
             head + f"impl Tag<{a_static}> for S {{}}\nimpl<'a> NeedT<{a_a}> for S where S: Tag<{a_static}> {{}}\n" + MAINP)
        case("implsuper", f"mx_{sn}_exact_where",
             head + f"impl<'a> NeedT<{a_a}> for S where S: Tag<{a_a}> {{}}\n" + MAINP)
        case("implsuper", f"mx_{sn}_generic_impl",
             head + f"impl<'a> Tag<{a_a}> for S {{}}\nimpl<'a> NeedT<{a_a}> for S {{}}\n" + MAINP)
        case("implsuper", f"mx_{sn}_b_fact_forward",
             head + f"impl Tag<{a_static}> for S {{}}\nimpl<'a, 'b> NeedT<{a_a}> for S where S: Tag<{a_b}>, 'a: 'b, 'b: 'a {{}}\n" + MAINP)
        case("implsuper", f"mx_{sn}_b_no_fact",
             head + f"impl<'b> Tag<{a_b}> for S {{}}\nimpl<'a, 'b> NeedT<{a_a}> for S where S: Tag<{a_b}> {{}}\n" + MAINP)
    # The proving clause arrives through a supertrait of the clause's trait.
    case("implsuper", "super_via_mid_static",
         "trait Tag<A> {}\ntrait Mid<A>: Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl Tag<&'static u8> for S {}\nimpl Mid<&'static u8> for S {}\nimpl<'a> NeedT<&'a u8> for S where S: Mid<&'static u8> {}\n" + MAINP)
    case("implsuper", "super_via_mid_exact",
         "trait Tag<A> {}\ntrait Mid<A>: Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct S;\nimpl<'a> NeedT<&'a u8> for S where S: Mid<&'a u8> {}\n" + MAINP)
    # A duplicated exact clause proves. (Two clauses at different
    # lifetimes are rustc E0283 ambiguity, a separate solver gap
    # recorded in the package report, so no case here.)
    case("implsuper", "super_two_clauses_exact",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct G<T>(T);\nimpl<'a, T> NeedT<&'a u8> for G<T> where G<T>: Tag<&'a u8>, G<T>: Tag<&'a u8> {}\n" + MAINP)
    # The impl's Self is generic; the clause names a concrete Self.
    case("implsuper", "super_generic_self",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct G<T>(T);\nimpl Tag<&'static u8> for G<u8> {}\nimpl<'a, T> NeedT<&'a u8> for G<T> where G<u8>: Tag<&'static u8> {}\n" + MAINP)
    case("implsuper", "super_generic_self_exact",
         "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct G<T>(T);\nimpl<'a, T> NeedT<&'a u8> for G<T> where G<T>: Tag<&'a u8> {}\n" + MAINP)
    # Round 2, family wc (tf-v07-bck's gen_wc matrix, p52): the
    # supertrait goal's shape (same, pair, ref, vec) x the Tk impl
    # (any T, T: 'static, only 'static shapes) x the impl's arg
    # region x its where clause (none, goal shape, element shape
    # at 'a or 'static, another trait, both). A clause proves the
    # goal only at its own shape and regions.
    wc_supers = {"same": "Tk<A>", "pair": "Tk<(A, A)>", "ref": "Tk<&'static A>", "vec": "Tk<Vec<A>>"}
    wc_impls = {
        "any": "impl<T> Tk<T> for S {}",
        "static": "impl<T: 'static> Tk<T> for S {}",
        "refstatic": "impl Tk<&'static u32> for S {}\nimpl Tk<(&'static u32, &'static u32)> for S {}\nimpl Tk<Vec<&'static u32>> for S {}\nimpl Tk<&'static &'static u32> for S {}",
    }
    wc_goals = {"same": "{x}", "pair": "({x}, {x})", "ref": "&'static {x}", "vec": "Vec<{x}>"}
    for sk, sup in wc_supers.items():
        for ik, timpl in wc_impls.items():
            for ak, arg in (("a", "&'a u32"), ("static", "&'static u32")):
                goal = wc_goals[sk].format(x=arg)
                wheres = {
                    "none": "",
                    "goal": f"where S: Tk<{goal}>",
                    "elem_a": "where S: Tk<&'a u32>",
                    "elem_static": "where S: Tk<&'static u32>",
                    "other_trait": "where S: Tk2",
                    "both": f"where S: Tk<&'a u32>, S: Tk<{goal}>",
                }
                for wk, w in wheres.items():
                    lt = "<'a> " if "'a" in arg or "'a" in w else ""
                    src = ("#![allow(unused)]\nstruct S;\ntrait Tk2 {}\nimpl Tk2 for S {}\ntrait Tk<A> {}\n"
                           f"trait Sup<A>: {sup} {{}}\n{timpl}\nimpl {lt}Sup<{arg}> for S {w} {{}}\n" + MAINP).replace("impl  ", "impl ").replace("impl <", "impl<")
                    case("wc", f"wc_{sk}_{ik}_{ak}_{wk}", src)
    # Round 2, family dupclause: clause count and lifetime mix
    # against one supertrait goal on a generic Self. One clause
    # proves it only at fact-equal regions; two clauses that read
    # regions differently are ambiguous (rustc E0283) when one
    # proves it; a 'static clause is not assembled against a
    # lifetime goal, so an impl can still prove that shape.
    dup_head = "trait Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct G<X>(X);\n"
    dup_impls = {"noimpl": "", "any": "impl<X, Y> Tag<Y> for G<X> {}\n", "staticbound": "impl<X, Y: 'static> Tag<Y> for G<X> {}\n"}
    for gname, goal_r in (("ga", "'a"), ("gs", "'static")):
        other_r = "'static" if goal_r == "'a" else "'a"
        exact = f"G<X>: Tag<&{goal_r} u8>"
        other = f"G<X>: Tag<&{other_r} u8>"
        bcl = "G<X>: Tag<&'b u8>"
        clause_sets = {
            "exact": exact,
            "other": other,
            "b": bcl,
            "b_eq": bcl + ", 'a: 'b, 'b: 'a",
            "b_ab": bcl + ", 'a: 'b",
            "b_ba": bcl + ", 'b: 'a",
            "exact_other": exact + ", " + other,
            "other_exact": other + ", " + exact,
            "exact_exact": exact + ", " + exact,
            "other_other": other + ", " + other,
            "exact_b": exact + ", " + bcl,
            "b_other": bcl + ", " + other,
            "exact_b_eq": exact + ", " + bcl + ", 'a: 'b, 'b: 'a",
            "exact_other_b": exact + ", " + other + ", " + bcl,
            "exact_exact_other": exact + ", " + exact + ", " + other,
        }
        for cname, clauses in clause_sets.items():
            impls = dup_impls.items() if cname in ("exact", "other", "b", "b_eq", "exact_other") else [("noimpl", ""), ("any", dup_impls["any"])]
            for iname, timpl in impls:
                src = dup_head + timpl + f"impl<'a, 'b, X> NeedT<&{goal_r} u8> for G<X> where {clauses} {{}}\n" + MAINP
                case("dupclause", f"dup_{gname}_{cname}_{iname}", src)
    # The proving clause can also arrive through a supertrait.
    case("dupclause", "dup_via_mid_static",
         "trait Tag<A> {}\ntrait Mid<A>: Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct G<X>(X);\nimpl<'a, X> NeedT<&'a u8> for G<X> where G<X>: Mid<&'a u8>, G<X>: Tag<&'static u8> {}\n" + MAINP)
    case("dupclause", "dup_via_mid_exact_only",
         "trait Tag<A> {}\ntrait Mid<A>: Tag<A> {}\ntrait NeedT<A>: Tag<A> {}\nstruct G<X>(X);\nimpl<'a, X> NeedT<&'a u8> for G<X> where G<X>: Mid<&'a u8> {}\n" + MAINP)
    # Wrapped argument types: validity follows the innermost type.
    wrapped = [("good_opt", "Option<u8>"), ("bad_opt", "Option<String>"), ("good_tup", "(u8, u8)"),
               ("bad_tup", "(String, u8)"), ("good_arr", "[u8; 2]"), ("bad_arr", "[String; 2]")]
    for wn, wt in wrapped:
        case("traitconst", f"trait_const_wrap_{wn}", NEED + f"trait Tr {{ const C: Need<{wt}>; }}\n" + MAINP)
        case("bound", f"supertrait_wrap_{wn}", SUP + f"trait Sub: Sup<{wt}> {{}}\n" + MAINP)
        case("bound", f"fn_bound_wrap_{wn}", SUP + f"fn f<T: Sup<{wt}>>(_x: T) {{}}\n" + MAINP)
        case("bound", f"struct_bound_wrap_{wn}", SUP + f"struct S<T: Sup<{wt}>>(T);\n" + MAINP)
        case("bound", f"assoc_bound_wrap_{wn}", SUP + f"trait Tr {{ type A2: Sup<{wt}>; }}\n" + MAINP)
        case("alias", f"alias_used_wrap_{wn}", NEED + f"type X = Need<{wt}>;\nfn f(_x: X) {{}}\n" + MAINP)
        case("impl", f"impl_trait_arg_wrap_{wn}", SUP + f"struct S;\nimpl Sup<{wt}> for S {{}}\n" + MAINP)
        case("direct", f"fn_sig_wrap_{wn}", NEED + f"fn f(_x: Need<{wt}>) {{}}\n" + MAINP)
        case("direct", f"field_wrap_{wn}", NEED + f"struct S {{ f: Need<{wt}> }}\n" + MAINP)
        case("alias", f"alias_unused_wrap_{wn}", NEED + f"type X = Need<{wt}>;\n" + MAINP)
        case("default", f"trait_default_wrap_{wn}", f"trait Tr<T: Copy = {wt}> {{}}\n" + MAINP)
        case("impl", f"binding_ty_wrap_{wn}", NEED + f"trait Tr {{ type A2; }}\nfn f<T: Tr<A2 = Need<{wt}>>>(_x: T) {{}}\n" + MAINP)
    # A user trait as the required bound, not only Copy.
    needm = "trait Mk {}\nimpl Mk for u8 {}\nstruct NeedM<T: Mk>(T);\n"
    supm = "trait Mk {}\nimpl Mk for u8 {}\ntrait SupM<T: Mk> {}\n"
    for an, aty, _ in ARGS:
        case("traitconst", f"trait_const_mk_{an}", needm + f"trait Tr {{ const C: NeedM<{aty}>; }}\n" + MAINP)
        case("bound", f"supertrait_mk_{an}", supm + f"trait Sub: SupM<{aty}> {{}}\n" + MAINP)
        case("bound", f"fn_bound_mk_{an}", supm + f"fn f<T: SupM<{aty}>>(_x: T) {{}}\n" + MAINP)
        case("alias", f"alias_used_mk_{an}", needm + f"type X = NeedM<{aty}>;\nfn f(_x: X) {{}}\n" + MAINP)
        case("bound", f"fn_bound_clone_{an}", "trait SupC<T: Clone> {}\n" + f"fn f<X: SupC<{aty}>>(_x: X) {{}}\n" + MAINP)
    # The same invalid type in a used position of a valid program shape:
    # partition into a struct used by main through a generic fn.
    for an, aty, aexpr in ARGS:
        case("direct", f"generic_ctor_{an}", NEED + f"fn make<T: Copy>(x: T) -> Need<T> {{ Need(x) }}\nfn main() {{ let _n: Need<{aty}> = make({aexpr}); println!(\"ok\"); }}\n")


_DENV = None


def init_daemon():
    """Pool worker init for --daemon: one private daemon per worker."""
    global _DENV
    import atexit, signal as _sig
    if not os.environ.get("SWEEP_DAEMON"):
        return
    d = tempfile.mkdtemp(prefix="defcheck-daemon-")
    env = dict(os.environ, LUST_DAEMON_SOCKET=os.path.join(d, "d.sock"), LUST_NO_CACHE="1", TMPDIR=d)
    env.pop("LUST_NO_DAEMON", None)
    log = open(os.path.join(d, "daemon.log"), "w")
    proc = subprocess.Popen([LUST, "daemon"], cwd=d, env=env, stderr=log, start_new_session=True)
    for _ in range(2000):
        if os.path.exists(env["LUST_DAEMON_SOCKET"]):
            break
        time.sleep(0.005)
    atexit.register(lambda: os.killpg(os.getpgid(proc.pid), _sig.SIGTERM))
    _DENV = env


def run(cmd, timeout=25, env=None):
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout, env=env)
        return r.returncode, r.stdout, r.stderr
    except subprocess.TimeoutExpired:
        return None, "", "timeout"


def one(job):
    family, name, src, keep = job
    d = keep or tempfile.mkdtemp(prefix="defcheck-")
    if keep:
        os.makedirs(d, exist_ok=True)
    path = os.path.join(d, name + ".rs")
    with open(path, "w") as f:
        f.write(src)
    env = dict(os.environ, LUST_NO_DAEMON="1", TMPDIR=d)
    rrc, _, rerr = run(["rustc", "--edition", "2024", "-A", "warnings", path, "-o", os.path.join(d, "rbin")], env=env)
    if rrc is None:
        return (family, name, "rustc-hang", rerr)
    crc, _, cerr = run([LUST, "check", path], env=env)
    if crc is None:
        return (family, name, "hang", "lust check timeout")
    if _DENV is not None:
        # Warm must equal cold byte for byte: prime the file's
        # session on a valid program, edit in this case's source
        # (mtime bumped), and check through the daemon.
        wenv = dict(_DENV, TMPDIR=d)
        mt = int(time.time()) + 100
        with open(path, "w") as f:
            f.write("fn main() {}\n")
        os.utime(path, (mt, mt))
        run([LUST, "check", path], env=wenv)
        with open(path, "w") as f:
            f.write(src)
        os.utime(path, (mt + 2, mt + 2))
        wrc, _, werr = run([LUST, "check", path], env=wenv)
        if wrc is None:
            return (family, name, "hang", "warm check timeout")
        if (wrc, werr) != (crc, cerr):
            return (family, name, "warm-diff", f"warm={werr[:140]!r} cold={cerr[:140]!r}")
    rustc_ok, lust_ok = rrc == 0, crc == 0
    if rustc_ok and lust_ok:
        brc, _, berr = run([LUST, "build", path, "-o", os.path.join(d, "lbin")], env=env)
        if brc is None:
            return (family, name, "hang", "lust build timeout")
        if brc != 0:
            return (family, name, "build-reject", berr[:200])
        orc, oout, _ = run([os.path.join(d, "rbin")], env=env)
        lrc, lout, _ = run([os.path.join(d, "lbin")], env=env)
        if orc is None or lrc is None:
            return (family, name, "hang", "run timeout")
        if (orc, oout) != (lrc, lout):
            return (family, name, "wrong-output", f"rustc=({orc},{oout!r}) lust=({lrc},{lout!r})")
        return (family, name, "ok", "")
    if rustc_ok and not lust_ok:
        return (family, name, "false-reject", cerr.strip().split("\n")[0][:160])
    if not rustc_ok and lust_ok:
        brc, _, _ = run([LUST, "build", path, "-o", os.path.join(d, "lbin")], env=env)
        if brc is None:
            return (family, name, "hang", "lust build timeout (rustc rejects)")
        return (family, name, "false-accept-check" if brc != 0 else "FALSE-ACCEPT", cerr[:120])
    return (family, name, "ok", "")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--jobs", type=int, default=8)
    ap.add_argument("--family", default="")
    ap.add_argument("--keep", default="")
    ap.add_argument("--daemon", action="store_true")
    a = ap.parse_args()
    if a.daemon:
        os.environ["SWEEP_DAEMON"] = "1"
    build_cases()
    jobs = [(f, n, s, os.path.join(a.keep, n) if a.keep else "") for (f, n, s) in CASES if not a.family or f == a.family]
    counts = {}
    bad = []
    with ProcessPoolExecutor(max_workers=a.jobs, initializer=init_daemon) as ex:
        for fam, name, cls, info in ex.map(one, jobs):
            counts[cls] = counts.get(cls, 0) + 1
            if cls != "ok":
                bad.append((fam, name, cls, info))
    for fam, name, cls, info in sorted(bad):
        print(f"  {cls:18s} {fam}/{name} {info}")
    # The summary prints last, so a gate lane can match it on the tail line.
    print(f"defcheck_sweep: {len(jobs)} cases: " + ", ".join(f"{k}={v}" for k, v in sorted(counts.items())))
    fatal = [b for b in bad if b[2] in ("FALSE-ACCEPT", "false-accept-check", "wrong-output", "hang", "build-reject", "warm-diff")]
    return 1 if fatal else 0


if __name__ == "__main__":
    sys.exit(main())
