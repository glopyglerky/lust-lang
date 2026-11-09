// Signature nodes for the body patch (W-PATCH-MISS2). A fn's expression
// and pattern ranges (`FnDef::exprs`, `FnDef::pats`) start at its first
// parameter, so parameter patterns (`_: &str`, `(a, b): (u8, u8)`) and
// expressions in the signature's types (`[u8; 4]`) sit at the start of the
// ranges, before the body's nodes. The checker numbers a fn's nodes by
// offset from the range starts. The patch appends the new body at the end
// of the arenas, so these nodes move with it: `copy` copies them ahead of
// the new body (same order, so the same offsets as a cold parse) and
// `relink` points the parameters and type nodes at the copies. The old
// nodes stay in the arenas, unreachable.
//
// `copy` proves first that it can do this: every node in the segment is a
// form with no type, block, or string index (literal, path without
// generic arguments, unary or binary expression; pattern of any shape over
// those), its child nodes sit in the segment, and every node is reached
// exactly once: from a parameter (its pattern), from a type node of the
// signature (parameter, `self`, return, and where clause types), or from
// another node of the segment. Anything else (an expression in a bound's
// generic arguments, say) refuses, and the request resolves in full.
use crate::ast::{Ast, BindingKind, Expr, ExprId, FnDef, GenericArg, GenericArgs, Pat, PatId, PathSeg, TraitBound, TyExpr, TyId, WherePred};
use crate::diag::Span;

/// What `relink` needs: the old segment ranges and the type nodes that
/// point into the expression segment.
pub(super) struct SigNodes {
    exprs: (u32, u32),
    pats: (u32, u32),
    tys: Vec<TyId>,
}

impl SigNodes {
    /// The copy of an old pattern id in the copied Ast (`copy`'s `into`).
    pub(super) fn pat(&self, p: PatId) -> PatId {
        PatId(p.0 - self.pats.0)
    }
}

/// Copies the signature segment of `def` (the nodes of its ranges before
/// the first one inside `body`) into `into`, an empty Ast, from index 0.
/// None when the segment is empty; Err when a node is not provably copied.
pub(super) fn copy(ast: &Ast, def: &FnDef, body: Span, into: &mut Ast) -> Result<Option<SigNodes>, &'static str> {
    // A node a macro expanded counts by its call site: its own span can
    // name the macro's definition, outside the body that called it.
    let inside = |s: Span| {
        let s = if s.ctxt == crate::diag::CtxtId::ROOT { s } else { crate::diag::call_site_span(s) };
        s.file == body.file && body.lo <= s.lo && s.hi <= body.hi
    };
    let first_inside = |(first, end): (u32, u32), spans: &[Span]| (first..end).find(|&i| spans.get(i as usize).is_some_and(|&s| inside(s)));
    let e_end = first_inside(def.exprs, &ast.expr_spans).unwrap_or(def.exprs.1);
    let p_end = first_inside(def.pats, &ast.pat_spans).unwrap_or(def.pats.1);
    let sig = SigNodes { exprs: (def.exprs.0, e_end), pats: (def.pats.0, p_end), tys: Vec::new() };
    if sig.exprs.0 == sig.exprs.1 && sig.pats.0 == sig.pats.1 {
        return Ok(None);
    }
    // Every node after the segment is the body's.
    let all_inside = |(lo, hi): (u32, u32), spans: &[Span]| spans.get(lo as usize..hi as usize).is_some_and(|s| s.iter().all(|&s| inside(s)));
    if !all_inside((e_end, def.exprs.1), &ast.expr_spans) || !all_inside((p_end, def.pats.1), &ast.pat_spans) {
        return Err("signature nodes");
    }
    let n_e = (sig.exprs.1 - sig.exprs.0) as usize;
    let n_p = (sig.pats.1 - sig.pats.0) as usize;
    let (mut e_refs, mut p_refs) = (vec![0u32; n_e], vec![0u32; n_p]);
    let in_e = |e: ExprId| (sig.exprs.0..sig.exprs.1).contains(&e.0);
    let in_p = |p: PatId| (sig.pats.0..sig.pats.1).contains(&p.0);
    let e_ref = |e: &mut ExprId, refs: &mut Vec<u32>| -> Result<(), &'static str> {
        if !in_e(*e) {
            return Err("signature nodes");
        }
        refs[(e.0 - sig.exprs.0) as usize] += 1;
        e.0 -= sig.exprs.0;
        Ok(())
    };
    for i in sig.exprs.0..sig.exprs.1 {
        let mut x = ast.exprs[i as usize].clone();
        match &mut x {
            Expr::Int(..) | Expr::Bool(_) | Expr::Char(_) | Expr::Unit | Expr::Underscore => {}
            Expr::Path(segs) if plain_path(segs) => {}
            Expr::Unary { expr, .. } => e_ref(expr, &mut e_refs)?,
            Expr::Binary { lhs, rhs, .. } => {
                e_ref(lhs, &mut e_refs)?;
                e_ref(rhs, &mut e_refs)?;
            }
            _ => return Err("signature nodes"),
        }
        into.exprs.push(x);
        into.expr_spans.push(ast.expr_spans[i as usize]);
    }
    for i in sig.pats.0..sig.pats.1 {
        let mut x = ast.pats[i as usize].clone();
        let mut kids: Vec<&mut PatId> = Vec::new();
        match &mut x {
            Pat::Wild | Pat::Binding { .. } | Pat::Rest => {}
            Pat::Path(segs) if plain_path(segs) => {}
            Pat::Tuple(ps) | Pat::Or(ps) => kids.extend(ps.iter_mut()),
            Pat::TupleStruct { path, elems } if plain_path(path) => kids.extend(elems.iter_mut()),
            Pat::Struct { path, fields, .. } if plain_path(path) => kids.extend(fields.iter_mut().map(|f| &mut f.1)),
            Pat::Slice { prefix, middle, suffix } => kids.extend(prefix.iter_mut().chain(middle.iter_mut()).chain(suffix.iter_mut())),
            Pat::Ref { inner, .. } | Pat::At { inner, .. } | Pat::Paren(inner) => kids.push(inner),
            Pat::Lit(e) => e_ref(e, &mut e_refs)?,
            Pat::Range { lo, hi, .. } => {
                for e in lo.iter_mut().chain(hi.iter_mut()) {
                    e_ref(e, &mut e_refs)?;
                }
            }
            _ => return Err("signature nodes"),
        }
        for k in kids {
            if !in_p(*k) {
                return Err("signature nodes");
            }
            p_refs[(k.0 - sig.pats.0) as usize] += 1;
            k.0 -= sig.pats.0;
        }
        into.pats.push(x);
        into.pat_spans.push(ast.pat_spans[i as usize]);
    }
    for p in &def.params {
        if let Some(pat) = p.pat {
            if !in_p(pat) {
                return Err("signature nodes");
            }
            p_refs[(pat.0 - sig.pats.0) as usize] += 1;
        }
    }
    // The signature's type nodes, and their expression fields.
    let mut roots: Vec<TyId> = def.params.iter().map(|p| p.ty).collect();
    roots.extend(def.self_param.as_ref().and_then(|s| s.ty));
    roots.extend(def.ret);
    for w in &def.where_clause.preds {
        if let WherePred::Type { ty, bounds, .. } = w {
            roots.push(*ty);
            // A bound's type arguments are type nodes; its const arguments
            // cannot point at a copy (`relink` edits type nodes only).
            let mut b = TyExpr::Dyn { binder: Vec::new(), bounds: bounds.clone(), lifetime: None };
            roots.extend(ty_children(&b));
            let mut bad = false;
            ty_exprs(&mut b, &mut |e| bad |= in_e(*e));
            if bad {
                return Err("signature nodes");
            }
        }
    }
    let mut tys = Vec::new();
    let mut stack = roots;
    while let Some(t) = stack.pop() {
        let mut node = ast.ty(t).clone();
        stack.extend(ty_children(&node));
        let mut hit = false;
        let mut bad = false;
        ty_exprs(&mut node, &mut |e| {
            if in_e(*e) {
                e_refs[(e.0 - sig.exprs.0) as usize] += 1;
                hit = true;
            } else {
                bad = true;
            }
        });
        if bad {
            return Err("signature nodes");
        }
        if hit {
            tys.push(t);
        }
    }
    if e_refs.iter().chain(&p_refs).any(|&n| n != 1) {
        return Err("signature nodes");
    }
    Ok(Some(SigNodes { tys, ..sig }))
}

/// Points the fn's parameters and signature type nodes at the copies,
/// which `apply` appended at `base` (expression and pattern arena lengths
/// before the append).
pub(super) fn relink(ast: &mut Ast, item: usize, sub: Option<usize>, sig: &SigNodes, base: (u32, u32)) {
    let (e0, p0) = (sig.exprs.0, sig.pats.0);
    for t in &sig.tys {
        ty_exprs(&mut ast.tys[t.0 as usize], &mut |e| e.0 = e.0 - e0 + base.0);
    }
    for p in &mut super::patch::fn_def_at_mut(ast, item, sub).params {
        if let Some(pat) = &mut p.pat {
            pat.0 = pat.0 - p0 + base.1;
        }
    }
}

fn plain_path(segs: &[PathSeg]) -> bool {
    segs.iter().all(|s| matches!(s.args, GenericArgs::None))
}

/// The type nodes one type node holds, through generic arguments too.
pub(crate) fn ty_children(t: &TyExpr) -> Vec<TyId> {
    fn segs(segs: &[PathSeg], out: &mut Vec<TyId>) {
        for s in segs {
            match &s.args {
                GenericArgs::None => {}
                GenericArgs::Angle(args) => {
                    for a in args {
                        match a {
                            GenericArg::Type(t) | GenericArg::Binding { kind: BindingKind::EqTy(t), .. } => out.push(*t),
                            GenericArg::Binding { kind: BindingKind::Constraint(bs), .. } => {
                                for b in bs {
                                    if let TraitBound::Trait { trait_ref, .. } = b {
                                        out.extend(trait_ref.path.iter().flat_map(|s| arg_tys(&s.args)));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                GenericArgs::Parenthesized { inputs, output } => out.extend(inputs.iter().chain(output)),
            }
        }
    }
    let mut out = Vec::new();
    match t {
        TyExpr::Path(p) => segs(p, &mut out),
        TyExpr::QPath { self_ty, trait_path, segs: p } => {
            out.push(*self_ty);
            segs(trait_path, &mut out);
            segs(p, &mut out);
        }
        TyExpr::Ref { inner, .. } | TyExpr::Slice(inner) | TyExpr::Ptr { inner, .. } | TyExpr::Paren(inner) | TyExpr::Array { inner, .. } => {
            out.push(*inner)
        }
        TyExpr::Tuple(ts) => out.extend(ts),
        TyExpr::FnPtr { inputs, ret, .. } => out.extend(inputs.iter().map(|p| p.ty).chain(*ret)),
        TyExpr::Dyn { bounds, .. } | TyExpr::ImplTrait { bounds, .. } => {
            for b in bounds {
                if let TraitBound::Trait { trait_ref, .. } = b {
                    segs(&trait_ref.path, &mut out);
                }
            }
        }
        TyExpr::Unit | TyExpr::Never | TyExpr::Infer | TyExpr::MacroCall(_) => {}
    }
    out
}

/// The type arguments of one segment's angle arguments (a bound inside
/// an associated constraint).
fn arg_tys(args: &GenericArgs) -> Vec<TyId> {
    match args {
        GenericArgs::Angle(args) => args
            .iter()
            .filter_map(|a| match a {
                GenericArg::Type(t) | GenericArg::Binding { kind: BindingKind::EqTy(t), .. } => Some(*t),
                _ => None,
            })
            .collect(),
        GenericArgs::Parenthesized { inputs, output } => inputs.iter().chain(output).copied().collect(),
        GenericArgs::None => Vec::new(),
    }
}

/// Calls `f` on each expression field of one type node: an array length
/// and the const arguments of its paths and bounds.
fn ty_exprs(t: &mut TyExpr, f: &mut dyn FnMut(&mut ExprId)) {
    fn segs(segs: &mut [PathSeg], f: &mut dyn FnMut(&mut ExprId)) {
        for s in segs {
            if let GenericArgs::Angle(args) = &mut s.args {
                for a in args {
                    match a {
                        GenericArg::Const(e) | GenericArg::Binding { kind: BindingKind::EqConst(e), .. } => f(e),
                        GenericArg::Binding { kind: BindingKind::Constraint(bs), .. } => bounds(bs, f),
                        _ => {}
                    }
                }
            }
        }
    }
    fn bounds(bs: &mut [TraitBound], f: &mut dyn FnMut(&mut ExprId)) {
        for b in bs {
            if let TraitBound::Trait { trait_ref, .. } = b {
                segs(&mut trait_ref.path, f);
            }
        }
    }
    match t {
        TyExpr::Array { len, .. } => f(len),
        TyExpr::Path(p) => segs(p, f),
        TyExpr::QPath { trait_path, segs: p, .. } => {
            segs(trait_path, f);
            segs(p, f);
        }
        TyExpr::Dyn { bounds: bs, .. } | TyExpr::ImplTrait { bounds: bs, .. } => bounds(bs, f),
        _ => {}
    }
}
