//! Per-instance evaluation of generic associated consts: an
//! initializer that names its impl's parameters (`N`, `T::V`,
//! `Self::X`) has one value per instance, computed here at a
//! concrete use site through the same bounded const evaluator
//! collect uses (`ty::consteval`). The `consts` entry of such a
//! const holds a zero placeholder; only this path reads the
//! real value.

use super::collect::{Program, VariantShape};
use super::generics::{self, Scope};
use crate::ast::{self, Ast};
use crate::def::DefId;
use crate::diag::{Diag, Span};
use crate::intern::Sym;
use crate::ty::consteval::{CallTarget, ConstCx, ConstValue, TypedValue};
use crate::ty::*;

/// One generic associated const instance to evaluate: the impl
/// (or trait) generics its initializer is over, the concrete
/// arguments for them, and its Self type.
pub struct AssocEnv {
    pub generics: Generics,
    pub args: Vec<GenericArg>,
    pub self_ty: Ty,
    /// The trait being implemented, when this is a trait assoc
    /// const: inner `T::C` reads prefer that trait's const.
    pub trait_def: Option<DefId>,
}

/// Evaluates the generic associated const `idx` (an index into
/// `Program::consts` with a `gx.assoc_const_tmpls` entry) at the
/// concrete arguments of `env`.
pub fn eval_assoc_const<'a, T: TyIntern>(p: &'a Program, tys: &'a mut T, idx: u32, env: AssocEnv) -> Result<TypedValue, Diag> {
    eval_inner(p, tys, idx, env, 0)
}

fn eval_inner<'a, T: TyIntern>(p: &'a Program, tys: &'a mut T, idx: u32, env: AssocEnv, depth: u32) -> Result<TypedValue, Diag> {
    let (expr, generics) = p
        .gx
        .assoc_const_tmpls
        .get(&idx)
        .cloned()
        .ok_or_else(|| Diag::new("L0630", Span::default(), "this associated const is outside the bounded const evaluator", "Use a literal value."))?;
    debug_assert_eq!(generics.params.len(), env.generics.params.len());
    let ty = subst(tys, p.consts[idx as usize].ty, &env.args);
    let mut cx = InstCx { p, ast: &p.ast, tys, env, depth };
    let (v, vt) = crate::ty::consteval::eval(&mut cx, expr, Some(ty))?;
    // The declared type after substitution is the value's type.
    Ok((v, if vt == ERROR { ty } else { vt }))
}

/// The `ConstCx` of one per-instance evaluation: names resolve
/// against the program's evaluated consts, this instance's
/// arguments, and (recursively) other associated consts.
struct InstCx<'a, T: TyIntern> {
    p: &'a Program,
    ast: &'a Ast,
    tys: &'a mut T,
    env: AssocEnv,
    depth: u32,
}

impl<T: TyIntern> InstCx<'_, T> {
    /// The resolution scope of the initializer: its parameters
    /// bound to this instance's concrete arguments.
    fn scope(&self) -> Scope {
        let mut scope = Scope::default();
        for (i, param) in self.env.generics.params.iter().enumerate() {
            match param.kind {
                ParamKind::Type => {
                    scope.type_params.push((param.name, i as u32));
                    if let Some(t) = self.env.args.get(i).and_then(|g| g.as_ty()) {
                        scope.param_tys.push((param.name, t));
                    }
                }
                ParamKind::Const(ty) => scope.const_params.push((param.name, i as u32, ty)),
                ParamKind::Lifetime => {}
            }
        }
        scope.self_ty = Some(self.env.self_ty);
        scope
    }

    /// The concrete type a path head names as an associated
    /// const's base: `Self`, a type parameter, or a written Adt
    /// application (`S<3>::C`, `Vec<u8>::C` through its Adt).
    fn base_ty(&mut self, seg: &ast::PathSeg) -> Option<Ty> {
        let p = self.p;
        if seg.name == p.known.self_type {
            return Some(self.env.self_ty);
        }
        for (i, param) in self.env.generics.params.iter().enumerate() {
            if param.name == seg.name {
                if !matches!(param.kind, ParamKind::Type) {
                    return None;
                }
                return self.env.args.get(i).and_then(|g| g.as_ty());
            }
        }
        let adt = *p.type_names.get(&seg.name)?;
        let scope = self.scope();
        let cx = p.gx.resolve_cx(&p.type_names, &p.aliases);
        let mut args: Vec<GenericArg> = Vec::new();
        if let ast::GenericArgs::Angle(written) = &seg.args {
            for w in written {
                match w {
                    ast::GenericArg::Type(t) => {
                        let t = generics::resolve_scoped(&p.ast, &p.interner, &p.known, &cx, self.tys, *t, &scope).ok()?;
                        args.push(GenericArg::Ty(subst(self.tys, t, &self.env.args)));
                    }
                    ast::GenericArg::Const(e) => {
                        let c = generics::resolve_const_arg(&p.ast, &p.interner, &p.known, self.tys, &scope, *e, Span::default()).ok()?;
                        args.push(GenericArg::Const(c));
                    }
                    ast::GenericArg::Lifetime(_) => args.push(GenericArg::Lifetime(Region::Erased)),
                    ast::GenericArg::Binding { .. } => return None,
                }
            }
        }
        // Const arguments bound by this instance still name the
        // impl's params after `resolve_const_arg`: substitute.
        let args: Vec<GenericArg> = args
            .into_iter()
            .map(|g| match g {
                GenericArg::Const(c) => match self.tys.const_kind(c) {
                    ConstKind::Param(i) => self.env.args.get(i as usize).copied().unwrap_or(g),
                    _ => g,
                },
                other => other,
            })
            .collect();
        Some(self.tys.mk_adt(p.def_of_adt(adt), &args))
    }

    /// The value of associated const `idx` under `env`: its
    /// evaluated value, or a fresh evaluation when generic.
    fn const_value(&mut self, idx: u32, env: AssocEnv) -> Option<TypedValue> {
        if self.depth >= 32 {
            return None;
        }
        if self.p.gx.assoc_const_tmpls.contains_key(&idx) {
            self.depth += 1;
            let out = eval_inner(self.p, &mut *self.tys, idx, env, self.depth).ok();
            self.depth -= 1;
            out
        } else {
            let c = &self.p.consts[idx as usize];
            Some((c.value.clone(), subst(self.tys, c.ty, &env.args)))
        }
    }

    /// The impl arguments bound by matching an impl Self pattern
    /// (over `generics`) against a concrete base type.
    fn match_args(&mut self, pat: Ty, base: Ty, generics: &Generics) -> Option<Vec<GenericArg>> {
        let (mut map, mut cmap) = (Vec::new(), Vec::new());
        if !super::collect::self_pat_matches(self.tys, pat, base, &mut map, &mut cmap) {
            return None;
        }
        let mut args: Vec<GenericArg> = Vec::new();
        for (i, param) in generics.params.iter().enumerate() {
            args.push(match param.kind {
                ParamKind::Type => GenericArg::Ty(map.iter().find(|&&(j, _)| j == i as u32).map(|&(_, t)| t)?),
                ParamKind::Const(_) => GenericArg::Const(cmap.iter().find(|&&(j, _)| j == i as u32).map(|&(_, c)| c)?),
                ParamKind::Lifetime => GenericArg::Lifetime(Region::Erased),
            });
        }
        Some(args)
    }

    /// An associated const read on a concrete base type: an
    /// inherent impl's, else a trait impl's (the env's trait
    /// first), else a trait default.
    fn assoc_on(&mut self, base: Ty, name: Sym) -> Option<TypedValue> {
        let p = self.p;
        if let Some(adt) = self.tys.local_adt(base) {
            for &idx in p.inherent_assoc_consts(adt, name) {
                let Some((pat, gens, _)) = p.gx.inherent_const_self.get(&idx).cloned() else { continue };
                if let Some(args) = self.match_args(pat, base, &gens) {
                    return self.const_value(idx, AssocEnv { generics: gens, args, self_ty: base, trait_def: None });
                }
            }
        }
        let mut traits: Vec<DefId> = Vec::new();
        for &assoc in p.gx.trait_assoc_consts.keys() {
            let Some(t) = p.gx.index.trait_of_assoc(assoc) else { continue };
            let Some(rec) = p.gx.index.trait_record(t) else { continue };
            if rec.assoc_items.iter().any(|&(d, n, _)| d == assoc && n == name) && !traits.contains(&t) {
                traits.push(t);
            }
        }
        // The trait being implemented answers first (`T::V` in
        // its own impl); other traits still answer (`Self::V0`
        // from a second trait's default).
        if let Some(want) = self.env.trait_def {
            traits.sort_by_key(|&t| t != want);
            if !traits.contains(&want) {
                traits.insert(0, want);
            }
        }
        for trait_def in traits {
            let Some(assoc) = p.gx.index.trait_record(trait_def).and_then(|r| r.assoc_by_name(name)) else { continue };
            for &id in p.gx.index.impls_of_trait(trait_def) {
                let rec = p.gx.index.get(id).clone();
                let Some(&idx) = p.impl_assoc_consts.get(&(rec.def, assoc)) else { continue };
                let Some(pat) = self.tys.args(rec.trait_ref.args).first().and_then(|g| g.as_ty()) else { continue };
                if let Some(args) = self.match_args(pat, base, &rec.generics) {
                    return self.const_value(idx, AssocEnv { generics: rec.generics.clone(), args, self_ty: base, trait_def: Some(trait_def) });
                }
            }
            if let Some(&idx) = p.trait_assoc_const_values.get(&assoc) {
                let gens = p.gx.index.trait_record(trait_def)?.generics.clone();
                let mut args: Vec<GenericArg> = vec![GenericArg::Ty(base)];
                for param in gens.params.iter().skip(1) {
                    args.push(match param.kind {
                        ParamKind::Const(_) => return None,
                        ParamKind::Lifetime => GenericArg::Lifetime(Region::Erased),
                        ParamKind::Type => return None,
                    });
                }
                return self.const_value(idx, AssocEnv { generics: gens, args, self_ty: base, trait_def: Some(trait_def) });
            }
        }
        None
    }

    /// The (Adt, variant) a path denotes, as collect's does.
    fn variant_of(&self, segs: &[ast::PathSeg]) -> Option<(AdtId, u32)> {
        let p = self.p;
        let head = |n: Sym| {
            if n == p.known.self_type {
                self.tys.local_adt(self.env.self_ty).and_then(|a| Some(p.adt(a).name)).unwrap_or(n)
            } else {
                n
            }
        };
        if segs.len() >= 4 {
            let path: Vec<&str> = segs.iter().map(|s| p.interner.get(s.name)).collect();
            return generics::std_variant_of(p, &path);
        }
        if segs.len() == 1 {
            let &a = p.type_names.get(&head(segs[0].name))?;
            if p.adt(a).is_enum {
                return None;
            }
            return Some((a, 0));
        }
        if segs.len() == 2 {
            let &a = p.type_names.get(&head(segs[0].name))?;
            let v = p.adt(a).variants.iter().position(|v| v.name == segs[1].name)?;
            return Some((a, v as u32));
        }
        None
    }
}

/// A const parameter value of this instance as a `ConstValue`.
fn const_kind_value(kind: ConstKind) -> Option<ConstValue> {
    match kind {
        ConstKind::Int(v) => Some(ConstValue::Int(v as i128)),
        ConstKind::Bool(b) => Some(ConstValue::Bool(b)),
        ConstKind::Char(c) => Some(ConstValue::Char(c)),
        _ => None,
    }
}

impl<'a, T: TyIntern> ConstCx<'a> for InstCx<'a, T> {
    type Tys = T;

    fn tys(&mut self) -> &mut T {
        self.tys
    }
    fn tys_ref(&self) -> &T {
        self.tys
    }
    fn ast(&self) -> &'a Ast {
        self.ast
    }
    fn sym_text(&self, s: Sym) -> &str {
        self.p.interner.get(s)
    }
    fn patterns_resolved(&self) -> bool {
        self.p.resolved.is_some()
    }
    fn const_item(&mut self, name: Sym, _at: Span) -> Option<TypedValue> {
        // A const parameter of this instance (`const V: usize = N`).
        for (i, param) in self.env.generics.params.iter().enumerate() {
            if param.name == name
                && let ParamKind::Const(ty) = param.kind
                && let Some(GenericArg::Const(c)) = self.env.args.get(i)
            {
                let v = const_kind_value(self.tys.const_kind(*c))?;
                return Some((v, subst(self.tys, ty, &self.env.args)));
            }
        }
        let &i = self.p.const_names.get(&name)?;
        let c = &self.p.consts[i as usize];
        Some((c.value.clone(), c.ty))
    }
    fn assoc_const_item(&mut self, segs: &[ast::PathSeg]) -> Option<TypedValue> {
        let [.., ty_seg, item_seg] = segs else { return None };
        let (ty_seg, name) = (ty_seg.clone(), item_seg.name);
        let base = self.base_ty(&ty_seg)?;
        self.assoc_on(base, name)
    }
    fn call_target(&mut self, callee: ast::ExprId) -> CallTarget {
        let p = self.p;
        let ast::Expr::Path(segs) = p.ast.expr(callee) else { return CallTarget::Unknown };
        let last = segs[segs.len() - 1].name;
        let ltext = p.interner.get(last);
        if ltext == "size_of" || ltext == "align_of" {
            let ok_path = segs.len() == 1 || (segs.len() == 3 && matches!(p.interner.get(segs[0].name), "std" | "core") && p.interner.get(segs[1].name) == "mem");
            if ok_path && segs[segs.len() - 1].type_args().len() == 1 {
                let arg = segs[segs.len() - 1].type_args()[0];
                let t = self.resolve_ty(arg);
                return if ltext == "size_of" { CallTarget::SizeOf(t) } else { CallTarget::AlignOf(t) };
            }
            return CallTarget::Unknown;
        }
        if segs.len() == 1 && last == p.known.Some {
            return CallTarget::VariantCtor { adt: AdtId(u32::MAX), variant: 1, ty: ERROR };
        }
        if segs.len() == 1 && last == p.known.Ok {
            return CallTarget::VariantCtor { adt: AdtId(u32::MAX - 1), variant: 0, ty: ERROR };
        }
        if segs.len() == 1 && last == p.known.Err {
            return CallTarget::VariantCtor { adt: AdtId(u32::MAX - 1), variant: 1, ty: ERROR };
        }
        let fid = if segs.len() == 1 {
            p.fn_names.get(&last).copied()
        } else if segs.len() == 2 {
            let head = if segs[0].name == p.known.self_type {
                self.tys.local_adt(self.env.self_ty).map(|a| p.adt(a).name).unwrap_or(segs[0].name)
            } else {
                segs[0].name
            };
            p.type_names.get(&head).and_then(|&a| p.methods.get(&(a, last)).and_then(|fs| fs.first()).copied())
        } else {
            None
        };
        if let Some(f) = fid {
            let target = super::collect::const_fn_target(p, f);
            if let CallTarget::ConstFn(b) = &target
                && b.generic != crate::ty::consteval::ConstGeneric::No
                && segs.iter().any(|s| !s.type_args().is_empty())
            {
                return CallTarget::Unknown;
            }
            return target;
        }
        if let Some((adt, variant)) = self.variant_of(segs)
            && p.adt(adt).variants[variant as usize].shape == VariantShape::Tuple
        {
            return CallTarget::VariantCtor { adt, variant, ty: self.tys.adt_ty(adt) };
        }
        CallTarget::Unknown
    }
    fn unit_variant(&mut self, segs: &[ast::PathSeg], expected: Option<Ty>) -> Option<(AdtId, u32, Ty)> {
        if segs.len() == 1 && segs[0].name == self.p.known.None {
            let elem = expected.and_then(|t| self.tys.option_elem(t)).unwrap_or(ERROR);
            return Some((AdtId(u32::MAX), 0, self.tys.mk_option(elem)));
        }
        let (adt, variant) = self.variant_of(segs)?;
        if self.p.adt(adt).variants[variant as usize].shape != VariantShape::Unit {
            return None;
        }
        Some((adt, variant, self.tys.adt_ty(adt)))
    }
    fn static_item(&mut self, name: Sym, _at: Span) -> Option<(u32, Ty)> {
        self.p.static_names.get(&name).map(|&i| (i, self.p.statics[i as usize].ty))
    }
    fn static_value(&mut self, idx: u32) -> Option<&crate::ty::consteval::ConstValue> {
        self.p.statics.get(idx as usize).map(|s| &s.value)
    }
    fn struct_target(&mut self, path: &[ast::PathSeg]) -> Option<(AdtId, u32, Ty)> {
        let (adt, variant) = self.variant_of(path)?;
        Some((adt, variant, self.tys.adt_ty(adt)))
    }
    fn variant_fields(&self, adt: AdtId, variant: u32) -> Vec<(Sym, Ty)> {
        if adt.0 >= u32::MAX - 1 {
            return Vec::new();
        }
        self.p.adt(adt).variants[variant as usize].fields.iter().map(|f| (f.name, f.ty)).collect()
    }
    fn variant_count(&self, adt: AdtId) -> u32 {
        if adt.0 >= u32::MAX - 1 {
            return 0;
        }
        self.p.adt(adt).variants.len() as u32
    }
    fn pattern_variant(&mut self, segs: &[ast::PathSeg]) -> Option<(AdtId, u32)> {
        self.variant_of(segs)
    }
    fn resolve_ty(&mut self, t: ast::TyId) -> Ty {
        let p = self.p;
        let cx = p.gx.resolve_cx(&p.type_names, &p.aliases);
        let scope = self.scope();
        match generics::resolve_scoped(&p.ast, &p.interner, &p.known, &cx, self.tys, t, &scope) {
            Ok(ty) => subst(self.tys, ty, &self.env.args),
            Err(_) => ERROR,
        }
    }
    fn layout(&mut self, t: Ty) -> Option<crate::ty::layout::Layout> {
        let p: &Program = self.p;
        crate::ty::layout::layout_of_ty(self.tys, &super::collect::UserLayout(p), t).ok()
    }
    fn ty_text(&self, t: Ty) -> String {
        super::collect::ty_name(self.p, &*self.tys, t)
    }
    fn adt_repr(&self, adt: AdtId) -> crate::ty::layout::Repr {
        self.p.adt(adt).repr
    }
    fn discriminant(&self, adt: AdtId, variant: u32) -> i128 {
        self.p.adt(adt).variants[variant as usize].disc
    }
    fn is_enum(&self, adt: AdtId) -> bool {
        adt.0 < u32::MAX - 1 && self.p.adt(adt).is_enum
    }
}
