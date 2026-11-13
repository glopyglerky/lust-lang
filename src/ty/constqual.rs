//! rustc's const qualifs of a const item's value (rustc_const_eval
//! check_consts/qualifs.rs): `NeedsDrop` and `HasMutInterior`, by value,
//! not only by type. `&CONST` is promoted to `'static` when neither
//! holds (lower `promotable`), so `const A: Option<String> = None` gives
//! a promoted `&A` and `const B: Option<String> = mk()` does not.
//!
//! rustc computes them with a dataflow pass over the const's MIR; this
//! walks the initializer's AST the same way: an aggregate (tuple, array,
//! struct literal, variant) qualifies by its parts, plus `NeedsDrop` for
//! an Adt with a `Drop` impl; a call, a method call, or any form not
//! handled here qualifies by its type; `if` and `match` join every arm,
//! dead ones too (rustc's MIR keeps `if false` branches); a `let` local
//! carries its initializer's qualifs, and a plain assignment replaces
//! them. The const's type gates the result.
use super::consteval::ConstCx;
use super::Ty;
use crate::ast::{self, Expr, ExprId, Pat, PatId, Stmt};
use crate::intern::Sym;

/// The two qualifs rustc's promotion reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Qualif {
    pub drop: bool,
    pub cell: bool,
}

impl Qualif {
    pub const NONE: Qualif = Qualif { drop: false, cell: false };
    /// Every qualif: the answer for a form only the type can decide.
    pub const ALL: Qualif = Qualif { drop: true, cell: true };

    pub fn any(self) -> bool {
        self.drop || self.cell
    }

    pub fn or(self, o: Qualif) -> Qualif {
        Qualif { drop: self.drop || o.drop, cell: self.cell || o.cell }
    }

    pub fn and(self, o: Qualif) -> Qualif {
        Qualif { drop: self.drop && o.drop, cell: self.cell && o.cell }
    }
}

/// The qualifs of a const item of type `ty` whose initializer is `e`.
/// Call it with the evaluator's context for that item (its scope and
/// `Self`).
pub fn of_const<'a, C: ConstCx<'a>>(cx: &mut C, e: ExprId, ty: Ty) -> Qualif {
    let by_ty = Qualif { drop: cx.needs_drop(ty), cell: cx.interior_mut(ty) };
    if !by_ty.any() {
        return Qualif::NONE;
    }
    walk(cx, &mut Vec::new(), e, 0).and(by_ty)
}

type Env = Vec<(Sym, Qualif)>;

fn walk<'a, C: ConstCx<'a>>(cx: &mut C, env: &mut Env, e: ExprId, depth: u32) -> Qualif {
    if depth > 64 {
        return Qualif::ALL;
    }
    let d = depth + 1;
    let all = |cx: &mut C, env: &mut Env, es: &[ExprId]| es.iter().fold(Qualif::NONE, |q, &x| q.or(walk(cx, env, x, d)));
    match cx.ast().expr(e).clone() {
        // Scalars: an operator on an Adt is a trait call, not const.
        Expr::Int(..) | Expr::Float(_) | Expr::Bool(_) | Expr::Char(_) | Expr::Str(_) | Expr::ByteStr(_) | Expr::Unit => Qualif::NONE,
        Expr::Binary { .. } | Expr::Cast { .. } | Expr::Unary { op: ast::UnOp::Neg | ast::UnOp::Not, .. } => Qualif::NONE,
        Expr::Range { lo, hi, .. } => all(cx, env, &lo.into_iter().chain(hi).collect::<Vec<_>>()),
        Expr::Deref(expr) | Expr::Ref { expr, .. } => walk(cx, env, expr, d),
        Expr::Tuple(es) | Expr::Array(es) => all(cx, env, &es),
        Expr::ArrayRepeat { value, .. } => walk(cx, env, value, d),
        Expr::Field { base, .. } | Expr::TupleField { base, .. } => walk(cx, env, base, d),
        Expr::StructLit { path, fields, update } => match cx.struct_target(&path) {
            Some((adt, _, _)) => {
                let fs: Vec<ExprId> = fields.iter().map(|f| f.2).chain(update).collect();
                inherent(cx, adt).or(all(cx, env, &fs))
            }
            None => Qualif::ALL,
        },
        Expr::Call { callee, args } => match cx.call_target(callee) {
            super::consteval::CallTarget::VariantCtor { adt, .. } => inherent(cx, adt).or(all(cx, env, &args)),
            _ => Qualif::ALL,
        },
        Expr::Path(segs) => {
            let at = cx.ast().expr_span(e);
            path(cx, env, &segs, at)
        }
        Expr::Block(b) | Expr::UnsafeBlock(b) => block(cx, env, b, d),
        Expr::ConstBlock(b) => block(cx, &mut Vec::new(), b, d),
        Expr::If { cond, then, els } => {
            let saved = env.len();
            if let Expr::Let { pat, scrut } = cx.ast().expr(cond).clone() {
                let q = walk(cx, env, scrut, d);
                bind(cx, env, pat, q);
            }
            let t = block(cx, env, then, d);
            env.truncate(saved);
            t.or(els.map_or(Qualif::NONE, |x| walk(cx, env, x, d)))
        }
        Expr::Match { scrut, arms } => {
            let q = walk(cx, env, scrut, d);
            arms.iter().fold(Qualif::NONE, |acc, arm| {
                let saved = env.len();
                bind(cx, env, arm.pat, q);
                let a = walk(cx, env, arm.body, d);
                env.truncate(saved);
                acc.or(a)
            })
        }
        _ => Qualif::ALL,
    }
}

/// `NeedsDrop` for an Adt with a written `Drop` impl (rustc's
/// `in_adt_inherently`). `Option` and `Result` have none.
fn inherent<'a, C: ConstCx<'a>>(cx: &mut C, adt: crate::ty::AdtId) -> Qualif {
    Qualif { drop: adt.0 < u32::MAX - 1 && cx.adt_has_drop(adt), cell: false }
}

/// A path: a local of the initializer, a const item (its own qualifs),
/// or a unit variant. Anything else qualifies by its type.
fn path<'a, C: ConstCx<'a>>(cx: &mut C, env: &mut Env, segs: &[ast::PathSeg], at: crate::diag::Span) -> Qualif {
    if let [seg] = segs {
        if let Some(&(_, q)) = env.iter().rev().find(|(n, _)| *n == seg.name) {
            return q;
        }
        if let Some(q) = cx.const_item_qualif(seg.name, at) {
            return q;
        }
    }
    match cx.unit_variant(segs, None) {
        Some((adt, _, _)) => inherent(cx, adt),
        None => Qualif::ALL,
    }
}

fn block<'a, C: ConstCx<'a>>(cx: &mut C, env: &mut Env, b: ast::BlockId, d: u32) -> Qualif {
    let saved = env.len();
    let blk = cx.ast().block(b).clone();
    for s in &blk.stmts {
        match s {
            Stmt::Let { pat, init, els: None, .. } => {
                let q = init.map_or(Qualif::NONE, |i| walk(cx, env, i, d));
                bind(cx, env, *pat, q);
            }
            Stmt::Item(_) => {}
            Stmt::Expr { expr, .. } => match cx.ast().expr(*expr).clone() {
                // A whole-local assignment replaces its qualifs, as
                // rustc's transfer function does.
                Expr::Assign { lhs, rhs } if matches!(cx.ast().expr(lhs), Expr::Path(p) if p.len() == 1) => {
                    let q = walk(cx, env, rhs, d);
                    let Expr::Path(p) = cx.ast().expr(lhs) else { unreachable!() };
                    let name = p[0].name;
                    match env.iter_mut().rev().find(|(n, _)| *n == name) {
                        Some(slot) => slot.1 = q,
                        None => taint(env),
                    }
                }
                // Any other statement may change a local (an assignment
                // in a branch, a `&mut` borrow): every local qualifies.
                _ => taint(env),
            },
            _ => taint(env),
        }
    }
    let q = blk.tail.map_or(Qualif::NONE, |t| walk(cx, env, t, d));
    env.truncate(saved);
    q
}

fn taint(env: &mut Env) {
    env.iter_mut().for_each(|(_, q)| *q = Qualif::ALL);
}

/// Binds the names of `pat` to `q`: a binding takes the qualifs of the
/// value it is a part of.
fn bind<'a, C: ConstCx<'a>>(cx: &mut C, env: &mut Env, pat: PatId, q: Qualif) {
    let mut names = Vec::new();
    pat_names(cx.ast(), pat, &mut names);
    env.extend(names.into_iter().map(|n| (n, q)));
}

fn pat_names(ast: &ast::Ast, p: PatId, out: &mut Vec<Sym>) {
    match ast.pat(p) {
        Pat::Binding { name, .. } => out.push(*name),
        Pat::At { name, inner, .. } => {
            out.push(*name);
            pat_names(ast, *inner, out);
        }
        Pat::Ref { inner, .. } => pat_names(ast, *inner, out),
        Pat::Slice { prefix, middle, suffix } => prefix.iter().chain(middle).chain(suffix).for_each(|&x| pat_names(ast, x, out)),
        Pat::TupleStruct { elems, .. } | Pat::Tuple(elems) => elems.iter().for_each(|&x| pat_names(ast, x, out)),
        Pat::Struct { fields, .. } => fields.iter().for_each(|&(_, x)| pat_names(ast, x, out)),
        Pat::Or(ps) => {
            if let Some(&x) = ps.first() {
                pat_names(ast, x, out);
            }
        }
        _ => {}
    }
}
