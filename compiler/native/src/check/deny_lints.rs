// rustc's deny-by-default lints that read a checked body: each one is
// an error under rustc unless an `#[allow]`, `#[warn]`, or `#[expect]`
// of the lint (or its group) covers the code. lust reads the levels on
// the statement and on the enclosing fn; a level written anywhere else
// (impl, module, crate) is not read, so the code stays rejected there
// (a false reject, never a false accept).
//
// - `undropped_manually_drops` (L0543): `drop(m)` with `m` a
//   `ManuallyDrop<T>` does nothing.
// - `let_underscore_lock` (L0544): `let _ = m.lock();` drops the guard
//   at once.
// - `enum_intrinsics_non_enums` (L0546): `mem::discriminant` of a type
//   that is not an enum (and has no type parameters).

use super::Checked;
use crate::ast::{self, Attr, Expr, Pat, PatId, Stmt};
use crate::check::collect::Program;
use crate::check::{Builtin, Res};
use crate::def::LocalDefId;
use crate::diag::Diag;
use crate::ty::sync::SyncTy;
use crate::ty::{BuiltinTy, Ty, TyIntern, TyKind};

/// Runs the body lints on one plain or template body.
pub fn check_body(p: &Program, fid: LocalDefId, ck: &Checked) -> Option<Diag> {
    let def = p.fn_def(fid);
    let fn_attrs = &def.attrs;
    // Every block of the body (its `let` statements and level scopes).
    let mut blocks: Vec<ast::BlockId> = def.body.into_iter().collect();
    for i in def.exprs.0..def.exprs.1 {
        match &p.ast.exprs[i as usize] {
            Expr::Block(b) | Expr::UnsafeBlock(b) | Expr::ConstBlock(b) => blocks.push(*b),
            Expr::If { then, .. } => blocks.push(*then),
            Expr::While { body, .. } | Expr::Loop { body, .. } | Expr::For { body, .. } | Expr::LabeledBlock { body, .. } | Expr::AsyncBlock { body, .. } => {
                blocks.push(*body)
            }
            _ => {}
        }
    }
    for s in blocks.clone().iter().flat_map(|&b| p.ast.block(b).stmts.iter()) {
        if let Stmt::Let { els: Some(e), .. } = s {
            blocks.push(*e);
        }
    }
    // The spans a statement, tail, arm, or block level covers.
    let scoped = |names: &[&str]| -> Vec<crate::diag::Span> {
        let mut out = Vec::new();
        for &b in &blocks {
            let blk = p.ast.block(b);
            if leveled(p, &blk.inner_attrs, names) {
                out.push(blk.span);
            }
            if let Some(t) = blk.tail
                && leveled(p, &blk.tail_attrs, names)
            {
                out.push(p.ast.expr_spans[t.0 as usize]);
            }
            for s in &blk.stmts {
                match s {
                    Stmt::Let { attrs, span, .. } if leveled(p, attrs, names) => out.push(*span),
                    Stmt::Expr { expr, attrs, .. } if leveled(p, attrs, names) => out.push(p.ast.expr_spans[expr.0 as usize]),
                    _ => {}
                }
            }
        }
        for i in def.exprs.0..def.exprs.1 {
            if let Expr::Match { arms, .. } = &p.ast.exprs[i as usize] {
                for a in arms.iter().filter(|a| leveled(p, &a.attrs, names)) {
                    out.push(p.ast.pat_spans[a.pat.0 as usize].to(p.ast.expr_spans[a.body.0 as usize]));
                }
            }
        }
        out
    };
    // Whether a lint at `inner` sits in a span that levels one of `names`
    // (`scoped`, walked only when a lint would fire: most bodies have none).
    let within = |inner: crate::diag::Span, names: &[&str]| scoped(names).iter().any(|o| o.file == inner.file && o.lo <= inner.lo && inner.hi <= o.hi);
    let md_names = ["undropped_manually_drops"];
    for i in def.exprs.0..def.exprs.1 {
        let e = &p.ast.exprs[i as usize];
        if let Expr::Call { callee, args } = e
            && let [arg] = args.as_slice()
            && (matches!(ck.res(*callee), Res::Builtin(Builtin::MemDrop)) || matches!(ck.res(ast::ExprId(i)), Res::Builtin(Builtin::MemDrop)))
            && let Some(d) = undropped_manually_drops(p, ck, def.exprs, ast::ExprId(i), *arg)
            && !leveled(p, fn_attrs, &md_names)
            && !within(p.ast.expr_spans[i as usize], &md_names)
        {
            return Some(d);
        }
    }
    let ei_names = ["enum_intrinsics_non_enums"];
    if !leveled(p, fn_attrs, &ei_names) {
        for i in def.exprs.0..def.exprs.1 {
            if let Expr::Call { callee, args } = &p.ast.exprs[i as usize]
                && let [arg] = args.as_slice()
                && let Some(d) = discriminant_non_enum(p, ck, *callee, *arg, ast::ExprId(i))
                && !within(d.span, &ei_names)
            {
                return Some(d);
            }
        }
    }
    let lock_names = ["let_underscore_lock", "let_underscore"];
    if leveled(p, fn_attrs, &lock_names) {
        return None;
    }
    for &b in &blocks {
        for s in &p.ast.block(b).stmts {
            if let Stmt::Let { pat, .. } = s
                && let Some(d) = let_underscore_lock(p, ck, *pat)
                && !within(d.span, &lock_names)
            {
                return Some(d);
            }
        }
    }
    // `_ = m.lock();` desugars to `let _ = ..` (rustc's AssignDesugar).
    for i in def.exprs.0..def.exprs.1 {
        if let Expr::Assign { lhs, rhs } = &p.ast.exprs[i as usize]
            && matches!(p.ast.exprs[lhs.0 as usize], Expr::Underscore)
            && is_lock(p, ck, ck.ty(*rhs))
            && !within(p.ast.expr_spans[i as usize], &lock_names)
        {
            return Some(lock_diag(p.ast.expr_spans[lhs.0 as usize]));
        }
    }
    None
}

/// True when `attrs` raise none of `names` to an error: an `allow`,
/// `warn`, or `expect` names one. (`deny` and `forbid` of a lint lust
/// does not otherwise check are rejected at parse, L0531.)
pub(crate) fn leveled(p: &Program, attrs: &[Attr], names: &[&str]) -> bool {
    fn has(trees: &[crate::lexer::TokenTree], p: &Program, names: &[&str]) -> bool {
        trees.iter().any(|t| match t {
            crate::lexer::TokenTree::Token(tok) => matches!(tok.tok, crate::lexer::Tok::Ident(s) if names.contains(&p.interner.get(s))),
            crate::lexer::TokenTree::Delim(_, inner, _) => has(inner, p, names),
        })
    }
    attrs.iter().any(|a| {
        a.path.len() == 1
            && matches!(p.interner.get(a.path[0].name), "allow" | "warn" | "expect")
            && matches!(&a.input, ast::AttrInput::List(_, trees) if has(trees, p, names))
    })
}

/// rustc's `undropped_manually_drops`: a direct `drop` call whose
/// argument is a `ManuallyDrop<T>`. A `Copy` argument takes the
/// `dropping_copy_types` warning instead, unless the call is a match
/// arm's whole body and the argument may have side effects (rustc's
/// `is_single_call_in_arm`).
fn undropped_manually_drops(p: &Program, ck: &Checked, range: (u32, u32), call: ast::ExprId, arg: ast::ExprId) -> Option<Diag> {
    let t = ck.ty(arg);
    let a = ck.tys.local_adt(t).or_else(|| ck.tys.adt_parts(t).and_then(|(d, _)| p.defs.local_adt(d)))?;
    if !p.is_manually_drop_adt(a) {
        return None;
    }
    if crate::check::collect::is_copy(p, &ck.tys, t) && !single_call_in_arm(p, range, call, arg) {
        return None;
    }
    Some(Diag::new(
        "L0543",
        p.ast.expr_spans[call.0 as usize],
        "calls to `std::mem::drop` with `std::mem::ManuallyDrop` instead of the inner value does nothing",
        "Drop the inner value: `drop(ManuallyDrop::into_inner(m))` (rustc `undropped_manually_drops`).",
    ))
}

/// rustc's `enum_intrinsics_non_enums` for `mem::discriminant` (std
/// source, library/core/src/mem.rs): the pointee type is neither an enum
/// nor open in a type parameter.
fn discriminant_non_enum(p: &Program, ck: &Checked, callee: ast::ExprId, arg: ast::ExprId, call: ast::ExprId) -> Option<Diag> {
    let Res::Fn(f) = ck.res(callee) else { return None };
    let info = p.fn_info(f);
    // Std module fns carry their module's hidden name (src/stdmods.rs).
    if p.name(info.name) != "__lust_core_mem$discriminant" || p.shipped_crate_at(info.def(&p.ast).name_span) != Some(crate::ty::lang::ShippedCrate::Core) {
        return None;
    }
    let TyKind::Ref(_, _, t) = ck.tys.kind(ck.ty(arg)) else { return None };
    let enum_like = ck.tys.option_elem(t).is_some()
        || ck.tys.result_args(t).is_some()
        || ck.tys.local_adt(t).or_else(|| ck.tys.adt_parts(t).and_then(|(d, _)| p.defs.local_adt(d))).is_some_and(|a| p.adt(a).is_enum);
    if enum_like || ck.tys.flags(t).contains_param() || matches!(ck.tys.kind(t), TyKind::Error | TyKind::Infer(_)) {
        return None;
    }
    Some(Diag::new(
        "L0546",
        p.ast.expr_spans[call.0 as usize],
        "the return value of `mem::discriminant` is unspecified when called with a non-enum type",
        "Pass a reference to an enum value (rustc `enum_intrinsics_non_enums`).",
    ))
}

/// rustc's `is_single_call_in_arm`: the argument may have side effects
/// and the call is exactly some match arm's body.
fn single_call_in_arm(p: &Program, range: (u32, u32), call: ast::ExprId, arg: ast::ExprId) -> bool {
    side_effects(p, arg)
        && p.ast.exprs[range.0 as usize..range.1 as usize]
            .iter()
            .any(|e| matches!(e, Expr::Match { arms, .. } if arms.iter().any(|a| a.body == call)))
}

/// rustc's `Expr::can_have_side_effects`.
fn side_effects(p: &Program, e: ast::ExprId) -> bool {
    match &p.ast.exprs[e.0 as usize] {
        Expr::Path(_)
        | Expr::QPath { .. }
        | Expr::SelfValue
        | Expr::Int(..)
        | Expr::Bool(_)
        | Expr::Str(_)
        | Expr::Char(_)
        | Expr::Float(_)
        | Expr::ByteStr(_)
        | Expr::Unit
        | Expr::OffsetOf { .. } => false,
        Expr::Unary { expr, .. } | Expr::Ref { expr, .. } | Expr::Cast { expr, .. } | Expr::Deref(expr) => side_effects(p, *expr),
        Expr::Field { base, .. } | Expr::TupleField { base, .. } => side_effects(p, *base),
        Expr::Index { base, index } => side_effects(p, *base) || side_effects(p, *index),
        Expr::StructLit { fields, update, .. } => fields.iter().any(|f| side_effects(p, f.2)) || update.is_some_and(|b| side_effects(p, b)),
        Expr::Tuple(es) | Expr::Array(es) => es.iter().any(|&x| side_effects(p, x)),
        _ => true,
    }
}

/// rustc's `let_underscore_lock`: a `_` anywhere in a `let` pattern
/// whose type needs drop and is a lock guard, or a `Result` whose `Ok`
/// type is one.
fn let_underscore_lock(p: &Program, ck: &Checked, pat: PatId) -> Option<Diag> {
    let mut found = None;
    walk_wild(p, pat, &mut |w| {
        if found.is_none() && is_lock(p, ck, ck.pat_ty(w)) {
            found = Some(lock_diag(p.ast.pat_spans[w.0 as usize]));
        }
    });
    found
}

fn lock_diag(span: crate::diag::Span) -> Diag {
    Diag::new(
        "L0544",
        span,
        "non-binding let on a synchronization lock",
        "Bind the guard to a name (`let _guard = ..`), or call `drop(..)` on it (rustc `let_underscore_lock`).",
    )
}

/// True for a `MutexGuard`, `RwLockReadGuard`, or `RwLockWriteGuard`,
/// alone or as a `Result`'s `Ok` type.
fn is_lock(_p: &Program, ck: &Checked, t: Ty) -> bool {
    let t = ck.tys.result_args(t).map_or(t, |(ok, _)| ok);
    matches!(
        ck.tys.builtin_of(t),
        Some(BuiltinTy::Sync(SyncTy::MutexGuard | SyncTy::RwLockReadGuard | SyncTy::RwLockWriteGuard))
    ) && !matches!(ck.tys.kind(t), TyKind::Error)
}

/// Calls `f` on every `_` sub-pattern of `pat`.
fn walk_wild(p: &Program, pat: PatId, f: &mut dyn FnMut(PatId)) {
    match p.ast.pat(pat) {
        Pat::Wild => f(pat),
        Pat::TupleStruct { elems, .. } | Pat::Tuple(elems) | Pat::Or(elems) => {
            for &e in elems {
                walk_wild(p, e, f);
            }
        }
        Pat::Struct { fields, .. } => {
            for &(_, e) in fields {
                walk_wild(p, e, f);
            }
        }
        Pat::Slice { prefix, middle, suffix } => {
            for &e in prefix.iter().chain(middle.iter()).chain(suffix) {
                walk_wild(p, e, f);
            }
        }
        Pat::Ref { inner, .. } | Pat::At { inner, .. } | Pat::Paren(inner) => walk_wild(p, *inner, f),
        _ => {}
    }
}
