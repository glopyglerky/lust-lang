// rustc's recursion limit on deep but finite types (E0275). lust answers
// the derive family and the auto traits on std and structural types without
// nested obligations, so its obligation queue never sees how deep rustc's
// evaluation goes there: `Option` nested 128 deep asks `u8: Clone` at
// depth 128, which overflows. `struct_depth` gives that depth from the
// type, by the std impls rustc walks; the obligation queue adds it
// (check/mod.rs `select_obligation_at`), and `check_uses` applies it where
// lust has no obligation at all. A build also asks `Copy` and layout of
// every local's concrete type (`check_locals`, `query_overflows`).
// Boundaries were calibrated against rustc 1.99 by binary search (check
// and build mode); where a shape was not measured the rules err early.
use crate::check::collect::Program;
use crate::def::DefId;
use crate::intern::FxMap;
use crate::ty::{BuiltinTy, Ty, TyIntern, TyKind};

/// Which impl family a trait walks: `Copy` and `Clone` stop at a shared
/// reference (builtin, no nested goal); the auto traits walk fields; the
/// rest of the derive family forwards through every container.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// `Copy` alone, as rustc asks it of every local in a build: `Box`,
    /// `Vec`, and the other non-`Copy` std types fail at once.
    Copy,
    CopyClone,
    Auto,
    Derive,
}

/// The family of a trait lust answers structurally, or None.
pub fn family(p: &Program, tr: DefId) -> Option<Family> {
    let l = &p.gx.lang;
    if tr == l.copy || tr == l.clone {
        Some(Family::CopyClone)
    } else if tr == l.send || tr == l.sync {
        Some(Family::Auto)
    } else if [l.debug, l.partial_eq, l.eq, l.partial_ord, l.ord, l.hash, l.default].contains(&tr) {
        Some(Family::Derive)
    } else {
        None
    }
}

/// The deepest nesting level below a root goal `t: Trait` (of family `fam`)
/// that rustc's evaluation reaches through std's structural impls: each
/// `Option`, `Result`, tuple, array, slice, and derived Adt arg is one level
/// (a derive bounds its params), a reference one level unless `Copy` or
/// `Clone` stop there, and for the auto traits `Box` two levels and `Vec`
/// three (their std fields). Leaves are 0. `memo` makes the walk linear in
/// the distinct parts of the type (a doubling type is a small DAG).
pub fn struct_depth(p: &Program, tys: &impl TyIntern, fam: Family, t: Ty, memo: &mut FxMap<(Family, Ty), usize>) -> usize {
    if let Some(&d) = memo.get(&(fam, t)) {
        return d;
    }
    // A recursive type meets itself again: rustc's cycle, no deeper level.
    memo.insert((fam, t), 0);
    let mut down = |e: Ty, step: usize| step + struct_depth(p, tys, fam, e, memo);
    let d = match tys.kind(t) {
        TyKind::Tuple(ts) => tys.tys_of(ts).to_vec().into_iter().map(|e| down(e, 1)).max().unwrap_or(0),
        TyKind::Array(e, _) | TyKind::Slice(e) => down(e, 1),
        TyKind::Ref(_, m, e) if !matches!(fam, Family::Copy | Family::CopyClone) || m.is_mut() => down(e, 1),
        TyKind::Adt(..) => {
            if let Some(e) = tys.option_elem(t) {
                down(e, 1)
            } else if let Some((a, b)) = tys.result_args(t) {
                down(a, 1).max(down(b, 1))
            } else {
                let args: Vec<Ty> = tys.adt_parts(t).map_or(Vec::new(), |(_, a)| tys.args(a).iter().filter_map(|g| g.as_ty()).collect());
                let b = tys.builtin_of(t);
                match b {
                    // The auto traits walk std's fields; calibrated against rustc
                    // 1.99 (`Option` around each, `Send`). rustc caches a field
                    // type with no param (`RawVecInner`), so a nested `Vec` costs
                    // only its 3 levels to `T`; counting the 5 of the fixed part
                    // again is early, never late.
                    Some(BuiltinTy::Boxed) if fam == Family::Auto => args.first().map_or(2, |&e| down(e, 2).max(2)),
                    Some(BuiltinTy::Vec) if fam == Family::Auto => args.first().map_or(5, |&e| down(e, 3).max(5)),
                    Some(BuiltinTy::String) if fam == Family::Auto => 6,
                    Some(BuiltinTy::Rc | BuiltinTy::Weak) if fam == Family::Auto => 0,
                    Some(BuiltinTy::Hash(crate::ty::hash::HashTy::HashMap | crate::ty::hash::HashTy::HashSet)) if fam == Family::Auto => {
                        // `K` and `V` (the hasher `S` is reached shallower).
                        4 + args.into_iter().take(2).map(|e| down(e, 0)).max().unwrap_or(0)
                    }
                    Some(BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc)) if fam == Family::Auto => args.first().map_or(2, |&e| down(e, 2)),
                    Some(_) if fam == Family::Auto => args.into_iter().map(|e| down(e, 1)).max().unwrap_or(1),
                    Some(BuiltinTy::Boxed | BuiltinTy::Vec) if fam != Family::Copy => args.first().map_or(0, |&e| down(e, 1)),
                    // `HashMap<K, V, S>` bounds `K` and `V` by the trait.
                    Some(BuiltinTy::Hash(_)) if fam != Family::Copy => args.into_iter().map(|e| down(e, 1)).max().unwrap_or(0),
                    Some(_) => 0,
                    None => match tys.local_adt(t) {
                        Some(a) if fam == Family::Auto => {
                            let fields: Vec<Ty> = p.adt(a).variants.iter().flat_map(|v| v.fields.iter().map(|f| f.ty)).collect();
                            fields.into_iter().chain(args).map(|e| down(e, 1)).max().unwrap_or(0)
                        }
                        // A derive bounds each type param by the trait.
                        Some(_) => args.into_iter().map(|e| down(e, 1)).max().unwrap_or(0),
                        None => 0,
                    },
                }
            }
        }
        _ => 0,
    };
    memo.insert((fam, t), d);
    d
}

/// rustc's recursion limit at the places in one body where lust has no
/// obligation to count it: the `Copy` query on each parameter and on a tuple
/// or array local used by value (borrowck), the trait uses lust answers
/// structurally (`clone` and the comparison and formatting methods,
/// comparison operators, format arguments) by `struct_depth`, autoderef
/// (E0055), and builtin adapter chains (`chain_overflow`). Returns the first
/// overflow (rustc E0275 or E0055) as L0611.
pub fn check_uses(p: &Program, fid: crate::def::LocalDefId, ck: &crate::check::Checked, limit: usize) -> Option<crate::diag::Diag> {
    use crate::ast::{BinOp, Expr};
    let def = p.fn_info(fid).def(&p.ast);
    let mut memo: FxMap<(Family, Ty), usize> = FxMap::default();
    // rustc's borrowck asks `Copy` of each parameter, with no fast path (a
    // tuple of scalars nested 128 deep overflows); a generic parameter's
    // type only when an Adt is outermost (a generic tuple waits for mono).
    for &l in &ck.param_locals {
        let info = &ck.locals[l as usize];
        let t = info.ty;
        if tys_generic(&ck.tys, t) && !matches!(ck.tys.kind(t), TyKind::Adt(..)) {
            continue;
        }
        if struct_depth(p, &ck.tys, Family::Copy, t, &mut memo) >= limit {
            return Some(overflow_diag(info.span));
        }
    }
    // rustc's struct-tail walk (the last field of nested structs) on each
    // local's type stops at the recursion limit.
    for info in &ck.locals {
        if !tys_generic(&ck.tys, info.ty) && struct_tail_steps(p, &ck.tys, info.ty, &[], limit + 1) > limit {
            return Some(crate::diag::Diag::new(
                "L0611",
                info.span,
                "reached the recursion limit finding the struct tail of this type",
                "Nest the type less deeply, or raise `#![recursion_limit]`.",
            ));
        }
    }
    for i in def.exprs.0..def.exprs.1 {
        let e = crate::ast::ExprId(i);
        let uses: Vec<(Family, crate::ast::ExprId, bool)> = match p.ast.expr(e) {
            Expr::MethodCall { recv, name, .. } => match p.interner.get(*name) {
                "clone" | "clone_from" => vec![(Family::CopyClone, *recv, true)],
                "eq" | "ne" | "cmp" | "partial_cmp" | "lt" | "le" | "gt" | "ge" | "hash" | "fmt" | "max" | "min" => {
                    vec![(Family::Derive, *recv, true)]
                }
                _ => Vec::new(),
            },
            Expr::Binary { op: BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge, lhs, .. } => {
                vec![(Family::Derive, *lhs, false)]
            }
            Expr::Macro { args, .. } => args.iter().map(|&a| (Family::Derive, a, false)).collect(),
            Expr::Path(_) if matches!(ck.expr_res.get((i - ck.expr_base) as usize), Some(crate::check::Res::Local(_))) => {
                let t = ck.ty(e);
                if matches!(ck.tys.kind(t), TyKind::Tuple(_) | TyKind::Array(..)) { vec![(Family::Copy, e, false)] } else { Vec::new() }
            }
            _ => Vec::new(),
        };
        if let Some(d) = chain_overflow(p, ck, e, limit) {
            return Some(d);
        }
        for (fam, at, peel) in uses {
            let mut t = ck.ty(at);
            while peel
                && let TyKind::Ref(_, m, inner) = ck.tys.kind(t)
                && !m.is_mut()
            {
                t = inner;
            }
            if struct_depth(p, &ck.tys, fam, t, &mut memo) >= limit {
                return Some(overflow_diag(p.ast.expr_spans[i as usize]));
            }
        }
    }
    None
}

/// The adapter layers of a builtin iterator type: `Map<Rev<slice::Iter>>`
/// is 2 (a source iterator is 0; `Zip` and `Chain` take their deeper side).
fn iter_layers(tys: &impl TyIntern, t: Ty, memo: &mut FxMap<Ty, usize>) -> usize {
    if let Some(&k) = memo.get(&t) {
        return k;
    }
    let k = match crate::ty::iter::iter_parts(tys, t) {
        Some((_, args)) => {
            let inner: Vec<usize> =
                args.into_iter().filter(|&a| crate::ty::iter::iter_parts(tys, a).is_some()).map(|a| iter_layers(tys, a, memo)).collect();
            inner.into_iter().max().map_or(0, |k| k + 1)
        }
        None => 0,
    };
    memo.insert(t, k);
    k
}

/// The deref steps rustc's method probe and field lookup take from `t`:
/// references, `Box`, `Rc`, and `Arc`, each one step, up to `cap`.
fn deref_steps(tys: &impl TyIntern, mut t: Ty, cap: usize) -> usize {
    let mut n = 0;
    while n <= cap {
        t = match tys.kind(t) {
            TyKind::Ref(_, _, e) => e,
            TyKind::Adt(..)
                if matches!(tys.builtin_of(t), Some(BuiltinTy::Boxed | BuiltinTy::Rc | BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc))) =>
            {
                match tys.adt_parts(t).and_then(|(_, a)| tys.args(a).first().and_then(|g| g.as_ty())) {
                    Some(e) => e,
                    None => break,
                }
            }
            _ => break,
        };
        n += 1;
    }
    n
}

/// rustc's limits at one expression that lust has no obligation for:
/// autoderef past the limit (E0055) on a method receiver or field base,
/// and the `Iterator` obligations of a long builtin adapter chain (E0275).
/// The chain rule is calibrated on single-kind chains against rustc 1.99
/// (`check` and build) and errs early where a shape was not measured: a
/// method call on a chain of `limit` adapter layers overflows; `count`,
/// `fold`, `any`, `collect`, and a same-kind adapter on a `Rev`, `Take`,
/// `Skip`, `StepBy`, or `Peekable` chain one layer later; a `for` loop and
/// `Box::new(it)` (it unsizes) one layer earlier (a `for` over `Peekable`
/// not); a chain with `Copied` or `Cloned` one layer earlier. A build
/// (mono instantiates every adapter) has its own measured table, and a
/// chain's layout (`layout_depth`) bounds the adapter calls there.
fn chain_overflow(p: &Program, ck: &crate::check::Checked, e: crate::ast::ExprId, limit: usize) -> Option<crate::diag::Diag> {
    use crate::ast::Expr;
    use crate::ty::iter::IterKind as K;
    let span = p.ast.expr_spans[e.0 as usize];
    let build = p.gx.walk_instances;
    // (the chain, layers it may have past `limit`: negative is earlier)
    let (base, slack): (crate::ast::ExprId, isize) = match p.ast.expr(e) {
        Expr::MethodCall { recv, name, .. } => {
            if deref_steps(&ck.tys, ck.ty(*recv), limit) > limit {
                return Some(crate::diag::Diag::new(
                    "L0611",
                    span,
                    "reached the recursion limit while auto-dereferencing the receiver",
                    "Dereference the value explicitly, or nest it less deeply.",
                ));
            }
            let top = crate::ty::iter::iter_parts(&ck.tys, peel(ck, ck.ty(*recv))).map(|(k, _)| k);
            let name = p.interner.get(*name);
            let slack = if build {
                // Build boundaries (mono adds its own levels): the measured
                // consumers, and two layers early for any other method.
                let filtering = matches!(top, Some(K::Filter | K::Peekable | K::FilterMap | K::MapWhile | K::TakeWhile | K::SkipWhile));
                match name {
                    "count" | "sum" => -(filtering as isize),
                    "fold" | "any" | "all" | "last" | "max" | "min" => -1,
                    "next" | "nth" | "by_ref" => -1 - (top == Some(K::Peekable)) as isize,
                    // An adapter call builds the next layer: layout decides.
                    "map" | "filter" | "rev" | "take" | "skip" | "step_by" | "peekable" | "inspect" | "fuse" | "enumerate" | "zip"
                    | "chain" | "copied" | "cloned" | "filter_map" | "take_while" | "skip_while" => 0,
                    _ => -2,
                }
            } else {
                let late = matches!(top, Some(K::Rev | K::Take | K::Skip | K::StepBy | K::Peekable))
                    && match name {
                        "count" | "fold" | "any" | "collect" => true,
                        "rev" => top == Some(K::Rev),
                        "take" => top == Some(K::Take),
                        "skip" => top == Some(K::Skip),
                        "step_by" => top == Some(K::StepBy),
                        "peekable" => top == Some(K::Peekable),
                        _ => false,
                    };
                late as isize
            };
            (*recv, slack)
        }
        Expr::Field { base, .. } => {
            if deref_steps(&ck.tys, ck.ty(*base), limit) > limit {
                return Some(crate::diag::Diag::new(
                    "L0611",
                    span,
                    "reached the recursion limit while auto-dereferencing the field base",
                    "Dereference the value explicitly, or nest it less deeply.",
                ));
            }
            return None;
        }
        Expr::For { iter, .. } => {
            let peek = matches!(crate::ty::iter::iter_parts(&ck.tys, peel(ck, ck.ty(*iter))), Some((K::Peekable, _)));
            (
                *iter,
                if build {
                    -1 - peek as isize
                } else if peek {
                    0
                } else {
                    -1
                },
            )
        }
        Expr::Call { callee, args } => match args.iter().copied().max_by_key(|&a| iter_layers(&ck.tys, ck.ty(a), &mut FxMap::default())) {
            Some(a) => {
                let boxed = matches!(p.ast.expr(*callee), Expr::Path(segs) if segs.len() == 2
                    && p.interner.get(segs[0].name) == "Box" && p.interner.get(segs[1].name) == "new");
                (
                    a,
                    if build {
                        -2
                    } else if boxed {
                        -1
                    } else {
                        0
                    },
                )
            }
            None => return None,
        },
        _ => return None,
    };
    let t = peel(ck, ck.ty(base));
    let mut memo: FxMap<Ty, usize> = FxMap::default();
    let layers = iter_layers(&ck.tys, t, &mut memo) as isize;
    if layers == 0 {
        return None;
    }
    let copies = has_copied(&ck.tys, t, &mut FxMap::default());
    // The one-layer-later shapes were measured on single-kind chains; a
    // mixed chain (`rev` over `filter` needs `DoubleEndedIterator` through
    // each `Filter`) overflows up to one layer earlier.
    let mixed = !single_kind(&ck.tys, t);
    let slack = if mixed { slack.min(0) - 1 } else { slack } - copies as isize;
    if layers >= limit as isize + slack {
        return Some(crate::diag::Diag::new(
            "L0611",
            span,
            "overflow evaluating the `Iterator` requirement of an adapter chain deeper than the recursion limit",
            "Split the chain, or collect part of it first.",
        ));
    }
    None
}

/// `t` without its references.
fn peel(ck: &crate::check::Checked, mut t: Ty) -> Ty {
    while let TyKind::Ref(_, _, inner) = ck.tys.kind(t) {
        t = inner;
    }
    t
}

/// True when an adapter chain has a `Copied` or `Cloned` layer (its
/// `Item = &T` bound costs rustc one more level).
fn has_copied(tys: &impl TyIntern, t: Ty, memo: &mut FxMap<Ty, bool>) -> bool {
    use crate::ty::iter::IterKind as K;
    if let Some(&b) = memo.get(&t) {
        return b;
    }
    let b = match crate::ty::iter::iter_parts(tys, t) {
        Some((k, args)) => {
            matches!(k, K::Copied | K::Cloned)
                || args.into_iter().any(|a| crate::ty::iter::iter_parts(tys, a).is_some() && has_copied(tys, a, memo))
        }
        None => false,
    };
    memo.insert(t, b);
    b
}

/// The nesting of rustc's layout queries for a value of `t`: a pointer or a
/// scalar is 1, an inline container one more than its deepest part. rustc's
/// query stack overflows when this passes `limit + 1` ("queries overflow
/// the depth limit"; a tuple nested 129 deep).
fn layout_depth(p: &Program, tys: &impl TyIntern, t: Ty, memo: &mut FxMap<Ty, usize>) -> usize {
    layout_depth_in(p, tys, t, &[], false, memo)
}

/// `layout_depth` of `t` with `Param(i)` standing for `params[i]` (the
/// fields of a generic Adt with no instance yet name its params); `sizeof`
/// gives the depths of a `size_of` query (calibrated per std type).
fn layout_depth_in(p: &Program, tys: &impl TyIntern, t: Ty, params: &[Ty], sizeof: bool, memo: &mut FxMap<Ty, usize>) -> usize {
    let open = !params.is_empty() && tys.flags(t).contains_param();
    if let TyKind::Param(i) = tys.kind(t)
        && let Some(&a) = params.get(i.0 as usize)
    {
        return layout_depth_in(p, tys, a, &[], sizeof, memo);
    }
    if !open && let Some(&d) = memo.get(&t) {
        return d;
    }
    if !open {
        memo.insert(t, 1);
    }
    let mut max_of = |ts: Vec<Ty>, memo: &mut FxMap<Ty, usize>| {
        1 + ts.into_iter().map(|e| layout_depth_in(p, tys, e, params, sizeof, memo)).max().unwrap_or(0)
    };
    let d = match tys.kind(t) {
        TyKind::Tuple(ts) => max_of(tys.tys_of(ts).to_vec(), memo),
        TyKind::Array(e, _) => max_of(vec![e], memo),
        TyKind::Closure(_, a) => max_of(crate::check::collect::closure_fields(tys, a), memo),
        TyKind::Adt(..) => {
            let args: Vec<Ty> = tys.adt_parts(t).map_or(Vec::new(), |(_, a)| tys.args(a).iter().filter_map(|g| g.as_ty()).collect());
            match tys.builtin_of(t) {
                Some(BuiltinTy::Option | BuiltinTy::Result) => max_of(args, memo),
                // std's fields, calibrated against rustc 1.99 (`Option` around
                // each): `Cell` holds an `UnsafeCell`, `RefCell` also a borrow
                // flag; the pointer types and collections do not hold `T` inline.
                Some(BuiltinTy::Cell) => 1 + max_of(args, memo),
                Some(BuiltinTy::RefCell) => (1 + max_of(args, memo)).max(4),
                Some(BuiltinTy::Sync(crate::ty::sync::SyncTy::Mutex | crate::ty::sync::SyncTy::RwLock)) => (1 + max_of(args, memo)).max(7),
                Some(BuiltinTy::String) => 8,
                Some(BuiltinTy::Vec) => 7,
                Some(BuiltinTy::Hash(_)) => 7 - 3 * sizeof as usize,
                Some(BuiltinTy::Boxed) => 5 - 2 * sizeof as usize,
                Some(BuiltinTy::Rc | BuiltinTy::Weak | BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc)) => 4,
                // Adapters hold their inner iterator inline, `Peekable`
                // next to an `Option<Option<Item>>`; a slice iterator is
                // two levels.
                Some(BuiltinTy::Iter(k)) => {
                    use crate::ty::iter::IterKind as K;
                    let inner: Vec<Ty> = args.iter().copied().filter(|&a| crate::ty::iter::iter_parts(tys, a).is_some()).collect();
                    if inner.is_empty() {
                        2
                    } else if k == K::Peekable {
                        max_of(inner, memo).max(4)
                    } else {
                        max_of(inner, memo)
                    }
                }
                Some(_) => 1,
                None => match tys.local_adt(t) {
                    Some(a) => {
                        let fields: Vec<Ty> = p.adt(a).variants.iter().flat_map(|v| v.fields.iter().map(|f| f.ty)).collect();
                        1 + fields.into_iter().map(|f| layout_depth_in(p, tys, f, &args, sizeof, memo)).max().unwrap_or(0)
                    }
                    None => 1,
                },
            }
        }
        _ => 1,
    };
    if !open {
        memo.insert(t, d);
    }
    d
}

/// rustc's build-time limits on the concrete types of one body's locals
/// (parameters, bindings, temporaries): the `Copy` query mono and drop
/// elaboration ask of each (an `Option` nested 128 deep overflows), and
/// layout (a tuple nested 129 deep). `cargo check` asks neither, so this
/// runs only when lust builds. Types with a generic parameter are skipped:
/// rustc asks these of instances only (a deep instance-only type is a known
/// gap).
pub fn check_locals(p: &Program, types: &[(Ty, crate::diag::Span)], tys: &impl TyIntern, limit: usize) -> Option<crate::diag::Diag> {
    let mut seen: FxMap<Ty, ()> = FxMap::default();
    let mut copy_memo: FxMap<(Family, Ty), usize> = FxMap::default();
    let mut layout_memo: FxMap<Ty, usize> = FxMap::default();
    for &(t, span) in types {
        if seen.insert(t, ()).is_some() || tys.flags(t).contains_param() || tys.flags(t).contains_infer() {
            continue;
        }
        // rustc's fast path: a tuple or array of scalars is `Copy` with no
        // trait query.
        if !trivially_copy(tys, t) && struct_depth(p, tys, Family::Copy, t, &mut copy_memo) >= limit {
            return Some(crate::diag::Diag::new(
                "L0611",
                span,
                "overflow evaluating whether this type is `Copy`: it nests deeper than the recursion limit",
                "Nest the type less deeply, or raise `#![recursion_limit]`.",
            ));
        }
        if layout_depth(p, tys, t, &mut layout_memo) > limit + 1 {
            return Some(crate::diag::Diag::new(
                "L0611",
                span,
                "queries overflow the depth limit computing this type's layout",
                "Nest the type less deeply, or raise `#![recursion_limit]`.",
            ));
        }
    }
    None
}

/// rustc's `is_trivially_pure_clone_copy`: scalars, shared references, raw
/// and fn pointers, and tuples and arrays of those.
fn trivially_copy(tys: &impl TyIntern, t: Ty) -> bool {
    match tys.kind(t) {
        TyKind::Int(_) | TyKind::Float(_) | TyKind::Bool | TyKind::Char | TyKind::Never | TyKind::Unit => true,
        TyKind::RawPtr(..) | TyKind::FnPtr(_) | TyKind::FnDef(..) => true,
        TyKind::Ref(_, m, _) => !m.is_mut(),
        TyKind::Tuple(ts) => tys.tys_of(ts).iter().all(|&e| trivially_copy(tys, e)),
        TyKind::Array(e, _) => trivially_copy(tys, e),
        _ => false,
    }
}

/// True when every adapter layer of a chain is the same kind.
fn single_kind(tys: &impl TyIntern, t: Ty) -> bool {
    let Some((k, _)) = crate::ty::iter::iter_parts(tys, t) else { return true };
    let mut cur = t;
    while let Some((k2, args)) = crate::ty::iter::iter_parts(tys, cur) {
        let inner: Vec<Ty> = args.into_iter().filter(|&a| crate::ty::iter::iter_parts(tys, a).is_some()).collect();
        if inner.is_empty() {
            return true;
        }
        if k2 != k || inner.len() > 1 {
            return false;
        }
        cur = inner[0];
    }
    true
}

fn tys_generic(tys: &impl TyIntern, t: Ty) -> bool {
    tys.flags(t).contains_param()
}

fn overflow_diag(span: crate::diag::Span) -> crate::diag::Diag {
    crate::diag::Diag::new(
        "L0611",
        span,
        "overflow evaluating a trait requirement: the type nests deeper than the recursion limit",
        "Nest the type less deeply, or raise `#![recursion_limit]`.",
    )
}

/// True when a build's `size_of`/`align_of` (layout, on a const-eval query
/// stack one level deeper than a local's) or `needs_drop` (the `Copy`
/// query, with rustc's fast path) of the concrete type `t` passes the
/// recursion limit. `cargo check` asks neither.
pub fn query_overflows(p: &Program, tys: &impl TyIntern, t: Ty, needs_drop: bool, limit: usize) -> bool {
    if !p.gx.walk_instances || tys.flags(t).contains_param() || tys.flags(t).contains_infer() {
        return false;
    }
    if needs_drop {
        !trivially_copy(tys, t) && struct_depth(p, tys, Family::Copy, t, &mut FxMap::default()) >= limit
    } else {
        // `Box` and `HashMap` are shallower on this stack (an `Option`
        // nested 125 deep around a `Box` overflows here and as a local).
        layout_depth_in(p, tys, t, &[], true, &mut FxMap::default()) >= limit
    }
}

/// The steps of rustc's struct-tail walk from `t` (each struct's last
/// field; `HashMap` is two std structs deep), up to `cap`. `params` stands
/// for a generic struct's params when it has no instance yet.
fn struct_tail_steps(p: &Program, tys: &impl TyIntern, t: Ty, params: &[Ty], cap: usize) -> usize {
    if cap == 0 {
        return 0;
    }
    if let TyKind::Param(i) = tys.kind(t)
        && let Some(&a) = params.get(i.0 as usize)
    {
        return struct_tail_steps(p, tys, a, &[], cap);
    }
    if matches!(tys.builtin_of(t), Some(BuiltinTy::Hash(crate::ty::hash::HashTy::HashMap | crate::ty::hash::HashTy::HashSet))) {
        return 2;
    }
    let Some(a) = tys.local_adt(t) else { return 0 };
    let def = p.adt(a);
    if def.is_enum {
        return 0;
    }
    let Some(last) = def.variants.first().and_then(|v| v.fields.last()) else { return 0 };
    let args: Vec<Ty> = tys.adt_parts(t).map_or(Vec::new(), |(_, a)| tys.args(a).iter().filter_map(|g| g.as_ty()).collect());
    1 + struct_tail_steps(p, tys, last.ty, &args, cap - 1)
}
