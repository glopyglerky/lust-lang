// Generic parameter defaults, with rustc's rules. Structs, enums, traits,
// and type aliases may give a type or const parameter a default; fns,
// impls, and methods may not. Defaults are trailing, name only the
// params declared before them (E0128), and outside a trait never `Self`
// (E0735). A type path that omits trailing arguments gets the defaults:
// each one resolves over the params before it (as `Param(i)`) and is then
// substituted with the arguments chosen so far, so `S<T, U = T>` written
// `S<u8>` is `S<u8, u8>`. An expression path with no arguments written
// infers every parameter instead (rustc's `infer_args`), so it never
// comes here. A default that names a lifetime is not supported yet.
use crate::ast::{self, Ast, Expr, GenericArgs, TyExpr};
use crate::check::generics::{ConstScalar, ResolveCx, Scope, check_const_expr_ty, names_lifetime, resolve_const_arg, resolve_scoped};
use crate::diag::{Diag, Span, not_yet};
use crate::intern::{Interner, Known, Sym};
use crate::ty::*;

/// How deep default resolution may nest (a default naming a type whose
/// own defaults apply, and so on) before it counts as a cycle.
const MAX_DEPTH: u32 = 16;

/// Which items may write defaults (rustc: structs, enums, unions, type
/// aliases, and traits).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    /// A struct, enum, or type alias: `Self` is not in scope.
    Type,
    /// A trait: `Self` is the trait's Self parameter.
    Trait,
    /// A fn, impl, or method: no defaults.
    Other,
}

/// The type and const params of written generics, in order (lifetimes
/// take no defaults and no argument slots here).
fn value_params(g: &ast::Generics) -> Vec<&ast::GenericParam> {
    g.params.iter().filter(|p| !matches!(p, ast::GenericParam::Lifetime { .. })).collect()
}

/// The written generics of the struct, enum, or type alias with this
/// flat name (the merged Ast holds every module's items).
pub fn type_generics<'a>(ast: &'a Ast, cx: &ResolveCx, name: Sym) -> Option<&'a ast::Generics> {
    match &ast.items.get(*cx.item_index.get(&name)? as usize)?.kind {
        ast::ItemKind::Struct(s) if s.name == name => Some(&s.generics),
        ast::ItemKind::Enum(e) if e.name == name => Some(&e.generics),
        ast::ItemKind::Type(a) if a.name == name => Some(&a.generics),
        _ => None,
    }
}

/// The written generics of the trait with this flat name.
pub fn trait_generics<'a>(ast: &'a Ast, cx: &ResolveCx, name: Sym) -> Option<&'a ast::Generics> {
    match &ast.items.get(*cx.item_index.get(&name)? as usize)?.kind {
        ast::ItemKind::Trait(t) if t.name == name => Some(&t.generics),
        _ => None,
    }
}

/// The `GenTables::item_index` of an Ast: struct, enum, alias, and trait
/// items by flat name, the first of a repeated name.
pub fn item_index(ast: &Ast) -> crate::intern::FxMap<Sym, u32> {
    let mut out = crate::intern::FxMap::default();
    for (i, it) in ast.items.iter().enumerate() {
        let name = match &it.kind {
            ast::ItemKind::Struct(s) => s.name,
            ast::ItemKind::Enum(e) => e.name,
            ast::ItemKind::Type(a) => a.name,
            ast::ItemKind::Trait(t) => t.name,
            _ => continue,
        };
        out.entry(name).or_insert(i as u32);
    }
    out
}

/// The number of leading type and const params that have no default:
/// a path must write at least these.
pub fn required(g: &ast::Generics) -> usize {
    value_params(g)
        .iter()
        .take_while(|p| match p {
            ast::GenericParam::Type { default, .. } => default.is_none(),
            ast::GenericParam::Const { default, .. } => default.is_none(),
            ast::GenericParam::Lifetime { .. } => true,
        })
        .count()
}

/// Completes `args` (the written type and const args, lifetimes left
/// out) with the defaults of the omitted trailing params of `g`. For a
/// trait, `self_ty` is the trait ref's Self type and `args` start after
/// it. A param with no default ends the fill; the caller reports the
/// arity.
#[allow(clippy::too_many_arguments)]
pub fn fill(
    ast: &Ast,
    interner: &Interner,
    known: &Known,
    cx: &ResolveCx,
    tys: &mut impl TyIntern,
    outer: &Scope,
    g: &ast::Generics,
    self_ty: Option<Ty>,
    args: &mut Vec<GenericArg>,
    span: Span,
) -> Result<(), Diag> {
    let params = value_params(g);
    if args.len() >= params.len() {
        return Ok(());
    }
    if outer.default_depth >= MAX_DEPTH {
        return Err(Diag::new(
            "L0601",
            span,
            "cycle detected when resolving generic parameter defaults",
            "Break the cycle: a default must not name its own type, directly or through other defaults.",
        ));
    }
    while args.len() < params.len() {
        let i = args.len();
        let Some(arg) = resolve_default(ast, interner, known, cx, tys, outer.default_depth + 1, &params, i, self_ty.is_some(), span)?
        else {
            return Ok(());
        };
        let full: Vec<GenericArg> = self_ty.map(GenericArg::Ty).into_iter().chain(args.iter().copied()).collect();
        args.push(subst_arg(tys, arg, &full));
    }
    Ok(())
}

/// The default of value param `i`, over the params before it (`Param(j)`,
/// shifted by one past `Self` for a trait). None when it has no default.
#[allow(clippy::too_many_arguments)]
fn resolve_default(
    ast: &Ast,
    interner: &Interner,
    known: &Known,
    cx: &ResolveCx,
    tys: &mut impl TyIntern,
    depth: u32,
    params: &[&ast::GenericParam],
    i: usize,
    is_trait: bool,
    span: Span,
) -> Result<Option<GenericArg>, Diag> {
    let shift = is_trait as u32;
    let mut sc = Scope { default_depth: depth, ..Scope::default() };
    if is_trait {
        sc.self_ty = Some(tys.intern(TyKind::Param(ParamId(0))));
    }
    for (j, p) in params[..i].iter().enumerate() {
        let idx = shift + j as u32;
        match p {
            ast::GenericParam::Type { name, .. } => sc.type_params.push((*name, idx)),
            ast::GenericParam::Const { name, ty, .. } => {
                let t = resolve_scoped(ast, interner, known, cx, tys, *ty, &Scope::default()).unwrap_or(ERROR);
                sc.const_params.push((*name, idx, t))
            }
            ast::GenericParam::Lifetime { .. } => {}
        }
    }
    match params[i] {
        ast::GenericParam::Type { default: Some(t), .. } => {
            if names_lifetime(ast, interner, *t) {
                return Err(not_yet("L0614", ast.ty_span(*t), "generic parameter defaults that name a lifetime"));
            }
            Ok(Some(GenericArg::Ty(resolve_scoped(ast, interner, known, cx, tys, *t, &sc)?)))
        }
        ast::GenericParam::Const { default: Some(e), ty, .. } => {
            let c = resolve_const_arg(ast, interner, known, tys, &sc, *e, span)?;
            let t = resolve_scoped(ast, interner, known, cx, tys, *ty, &Scope::default()).unwrap_or(ERROR);
            if let Some(want) = ConstScalar::of(tys, t) {
                check_const_expr_ty(ast, interner, known, cx, tys, &sc, *e, c, want, span, "this const default")?;
            }
            Ok(Some(GenericArg::Const(c)))
        }
        _ => Ok(None),
    }
}

/// One default under the arguments chosen before it.
fn subst_arg(tys: &mut impl TyIntern, g: GenericArg, args: &[GenericArg]) -> GenericArg {
    match g {
        GenericArg::Ty(t) => GenericArg::Ty(subst(tys, t, args)),
        GenericArg::Const(c) => match tys.const_kind(c) {
            ConstKind::Param(i) => match args.get(i as usize) {
                Some(GenericArg::Const(k)) => GenericArg::Const(*k),
                _ => g,
            },
            _ => g,
        },
        other => other,
    }
}

/// The resolved defaults of an item's params in declared order (None for
/// a param without one), each over the params before it. Errors go to
/// `errs`; the driver folds the result into the item's `Sig` fingerprint
/// and the finish-side WF check proves the non-dependent ones.
#[allow(clippy::too_many_arguments)]
pub fn resolve_all(
    ast: &Ast,
    interner: &Interner,
    known: &Known,
    cx: &ResolveCx,
    tys: &mut impl TyIntern,
    g: &ast::Generics,
    is_trait: bool,
    errs: &mut Vec<Diag>,
) -> Vec<Option<GenericArg>> {
    let params = value_params(g);
    (0..params.len())
        .map(|i| {
            let span = param_span(params[i]);
            resolve_default(ast, interner, known, cx, tys, 1, &params, i, is_trait, span).unwrap_or_else(|d| {
                errs.push(d);
                None
            })
        })
        .collect()
}

fn param_span(p: &ast::GenericParam) -> Span {
    match p {
        ast::GenericParam::Type { span, .. } | ast::GenericParam::Const { span, .. } => *span,
        ast::GenericParam::Lifetime { lt, .. } => lt.span,
    }
}

/// The written-form rules on defaults (see the file header), checked once
/// per item: where defaults may appear, trailing order, no forward
/// reference, and no `Self` outside a trait.
pub fn check_written(ast: &Ast, known: &Known, g: &ast::Generics, owner: Owner, errs: &mut Vec<Diag>) {
    let params = value_params(g);
    let has_default = |p: &ast::GenericParam| match p {
        ast::GenericParam::Type { default, .. } => default.is_some(),
        ast::GenericParam::Const { default, .. } => default.is_some(),
        ast::GenericParam::Lifetime { .. } => false,
    };
    for (i, p) in params.iter().enumerate() {
        if !has_default(p) {
            continue;
        }
        let span = param_span(p);
        if owner == Owner::Other {
            errs.push(Diag::new(
                "L0110",
                span,
                "defaults for generic parameters are not allowed here",
                "Remove the default; only structs, enums, traits, and type aliases take them.",
            ));
            continue;
        }
        if params[i + 1..].iter().any(|q| !has_default(q)) {
            errs.push(Diag::new(
                "L0110",
                span,
                "generic parameters with a default must be trailing",
                "Move the parameters with defaults after the ones without.",
            ));
        }
        // The param itself and every later one are not declared yet.
        let later: Vec<Sym> = params[i..]
            .iter()
            .filter_map(|q| match q {
                ast::GenericParam::Type { name, .. } | ast::GenericParam::Const { name, .. } => Some(*name),
                ast::GenericParam::Lifetime { .. } => None,
            })
            .collect();
        let mut names = Names { ast, find: &later, self_type: known.self_type, found: false, self_found: false };
        match p {
            ast::GenericParam::Type { default: Some(t), .. } => names.ty(*t),
            ast::GenericParam::Const { default: Some(e), .. } => names.expr(*e),
            _ => {}
        }
        if names.found {
            errs.push(Diag::new(
                "L0110",
                span,
                "generic parameter defaults cannot reference parameters before they are declared",
                "Reorder the parameters, or name only earlier ones in the default.",
            ));
        }
        if names.self_found && owner == Owner::Type {
            errs.push(Diag::new(
                "L0110",
                span,
                "generic parameters cannot use `Self` in their defaults",
                "Write the type out instead of `Self`.",
            ));
        }
    }
}

/// True when a default of value param `from` or later names `Self`.
pub fn defaults_name_self(ast: &Ast, known: &Known, g: &ast::Generics, from: usize) -> bool {
    value_params(g).iter().skip(from).any(|p| {
        let mut names = Names { ast, find: &[], self_type: known.self_type, found: false, self_found: false };
        match p {
            ast::GenericParam::Type { default: Some(t), .. } => names.ty(*t),
            ast::GenericParam::Const { default: Some(e), .. } => names.expr(*e),
            _ => {}
        }
        names.self_found
    })
}

/// A walk over a written default for the names it uses.
struct Names<'a> {
    ast: &'a Ast,
    find: &'a [Sym],
    self_type: Sym,
    found: bool,
    self_found: bool,
}

impl Names<'_> {
    fn seg_name(&mut self, segs: &[ast::PathSeg]) {
        if let Some(first) = segs.first() {
            if first.name == self.self_type {
                self.self_found = true;
            }
            if self.find.contains(&first.name) {
                self.found = true;
            }
        }
        for s in segs {
            self.args(&s.args);
        }
    }

    fn args(&mut self, args: &GenericArgs) {
        match args {
            GenericArgs::Angle(a) => {
                for g in a {
                    match g {
                        ast::GenericArg::Type(t) => self.ty(*t),
                        ast::GenericArg::Const(e) => self.expr(*e),
                        ast::GenericArg::Binding { kind, .. } => match kind {
                            ast::BindingKind::EqTy(t) => self.ty(*t),
                            ast::BindingKind::EqConst(e) => self.expr(*e),
                            ast::BindingKind::Constraint(bs) => self.bounds(bs),
                        },
                        ast::GenericArg::Lifetime(_) => {}
                    }
                }
            }
            GenericArgs::Parenthesized { inputs, output } => {
                for &t in inputs {
                    self.ty(t);
                }
                if let Some(t) = output {
                    self.ty(*t);
                }
            }
            GenericArgs::None => {}
        }
    }

    fn bounds(&mut self, bs: &[ast::TraitBound]) {
        for b in bs {
            if let ast::TraitBound::Trait { trait_ref, .. } = b {
                self.seg_name(&trait_ref.path);
            }
        }
    }

    fn expr(&mut self, e: ast::ExprId) {
        if let Expr::Path(segs) = self.ast.expr(e) {
            self.seg_name(segs);
        }
    }

    fn ty(&mut self, t: ast::TyId) {
        match self.ast.ty(t) {
            TyExpr::Path(segs) => self.seg_name(segs),
            TyExpr::Ref { inner, .. } | TyExpr::Slice(inner) | TyExpr::Ptr { inner, .. } | TyExpr::Paren(inner) => self.ty(*inner),
            TyExpr::Tuple(ts) => {
                for &e in ts {
                    self.ty(e);
                }
            }
            TyExpr::Array { inner, len } => {
                self.ty(*inner);
                self.expr(*len);
            }
            TyExpr::FnPtr { inputs, ret, .. } => {
                for p in inputs {
                    self.ty(p.ty);
                }
                if let Some(r) = ret {
                    self.ty(*r);
                }
            }
            TyExpr::Dyn { bounds, .. } | TyExpr::ImplTrait { bounds, .. } => self.bounds(bounds),
            TyExpr::QPath { self_ty, trait_path, segs } => {
                self.ty(*self_ty);
                self.seg_name(trait_path);
                for s in segs {
                    self.args(&s.args);
                }
            }
            TyExpr::Unit | TyExpr::Never | TyExpr::Infer | TyExpr::MacroCall(_) => {}
        }
    }
}
