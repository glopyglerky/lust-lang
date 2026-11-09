// `use<..>` precise capturing on a return-position `impl Trait` (edition
// 2024). Without a list the opaque type captures every generic param in
// scope; with one it captures only the listed lifetimes (type and const
// params must all be listed, as rustc requires). Collect validates the
// list (`check_list`); the region signature records the captured
// universals (`captured_regions`, `RegionSig::opaque_caps`): the defining
// body's returned value may hold only those (rustc E0700), and a call
// captures only the arguments that name them (borrowck `call`).
use super::generics::{Scope, scope_region};
use crate::ast::{self, Ast, TyExpr, UseArg, UseCaps};
use crate::diag::{Diag, Span};
use crate::intern::Interner;
use crate::ty::{Generics, ParamKind, Region};

/// rustc's rules for one return-position `use<..>` list over the fn's
/// generics (the impl's, the fn's own, and its `impl Trait` argument
/// params): lifetimes first, no param twice, no `Self` alias, only
/// params in scope, and every type and const param listed. Lifetime
/// names resolve in `captured_regions`.
pub(super) fn check_list(interner: &Interner, generics: &Generics, caps: &UseCaps, errs: &mut Vec<Diag>) {
    let mut seen: Vec<crate::intern::Sym> = Vec::new();
    let mut past_lifetimes = false;
    for a in &caps.args {
        let (name, span) = match a {
            UseArg::Lifetime(lt) => (lt.name, lt.span),
            UseArg::Param(n, s) => (*n, *s),
        };
        let text = interner.get(name);
        if seen.contains(&name) {
            errs.push(Diag::new("L0110", span, format!("cannot capture parameter `{text}` twice"), "Remove the second one."));
            return;
        }
        seen.push(name);
        match a {
            UseArg::Lifetime(lt) if past_lifetimes => {
                let text = interner.get(lt.name);
                errs.push(Diag::new(
                    "L0110",
                    span,
                    format!("lifetime parameter `'{text}` must be listed before non-lifetime parameters"),
                    "List the lifetimes first in `use<..>`.",
                ));
                return;
            }
            UseArg::Lifetime(_) => {}
            UseArg::Param(n, _) => {
                past_lifetimes = true;
                if text == "Self" {
                    errs.push(Diag::new(
                        "L0110",
                        span,
                        "`Self` can't be captured in `use<...>` precise captures list, since it is an alias",
                        "List the impl's own type params instead.",
                    ));
                    return;
                }
                if !generics.params.iter().any(|p| p.name == *n && !matches!(p.kind, ParamKind::Lifetime)) {
                    errs.push(Diag::new(
                        "L0101",
                        span,
                        format!("cannot find type or const parameter `{text}` in this scope"),
                        "List only the generic params of this fn and its impl.",
                    ));
                    return;
                }
            }
        }
    }
    for p in &generics.params {
        let what = match p.kind {
            ParamKind::Lifetime => continue,
            ParamKind::Type => "type",
            ParamKind::Const(_) => "const",
        };
        if !seen.contains(&p.name) {
            errs.push(Diag::new(
                "L0110",
                caps.span,
                format!("`impl Trait` must mention all {what} parameters in scope in `use<...>`"),
                "Add the param to the `use<..>` list (an `impl Trait` argument cannot be named: give it a name).",
            ));
            return;
        }
    }
}

/// The universals (`Region::Param` indices) a return-position `impl
/// Trait` with a `use<..>` list captures, resolved in the output scope
/// `rs` (so `'_` is the output elision lifetime, L0202 when there is
/// none, as rustc E0106). None without a list (it captures all).
pub(super) fn captured_regions(interner: &Interner, rs: &Scope, ast: &Ast, ret: Option<ast::TyId>, errs: &mut Vec<Diag>) -> Option<Vec<u32>> {
    let TyExpr::ImplTrait { captures: Some(caps), .. } = ast.ty(ret?) else { return None };
    let mut out = Vec::new();
    for a in &caps.args {
        let UseArg::Lifetime(lt) = a else { continue };
        match scope_region(interner, rs, Some(*lt), lt.span) {
            Ok(Region::Param(i)) => out.push(i),
            Ok(_) => {}
            Err(d) => errs.push(d),
        }
    }
    Some(out)
}

/// The span of a `use<..>` list anywhere in an argument type (rustc
/// rejects precise capturing outside a return-position `impl Trait`).
pub(super) fn in_arg_type(ast: &Ast, t: ast::TyId) -> Option<Span> {
    let in_args = |args: &ast::GenericArgs| match args {
        ast::GenericArgs::Angle(args) => args.iter().find_map(|a| match a {
            ast::GenericArg::Type(t) | ast::GenericArg::Binding { kind: ast::BindingKind::EqTy(t), .. } => in_arg_type(ast, *t),
            _ => None,
        }),
        ast::GenericArgs::Parenthesized { inputs, output } => inputs.iter().chain(output).find_map(|&t| in_arg_type(ast, t)),
        _ => None,
    };
    match ast.ty(t) {
        TyExpr::ImplTrait { captures: Some(c), .. } => Some(c.span),
        TyExpr::ImplTrait { bounds, .. } | TyExpr::Dyn { bounds, .. } => bounds.iter().find_map(|b| match b {
            ast::TraitBound::Trait { trait_ref, .. } => trait_ref.path.iter().find_map(|s| in_args(&s.args)),
            _ => None,
        }),
        TyExpr::Ref { inner, .. } | TyExpr::Paren(inner) | TyExpr::Slice(inner) | TyExpr::Ptr { inner, .. } | TyExpr::Array { inner, .. } => {
            in_arg_type(ast, *inner)
        }
        TyExpr::Tuple(elems) => elems.iter().find_map(|&e| in_arg_type(ast, e)),
        TyExpr::Path(segs) => segs.iter().find_map(|s| in_args(&s.args)),
        TyExpr::FnPtr { inputs, ret, .. } => inputs.iter().map(|p| p.ty).chain(*ret).find_map(|t| in_arg_type(ast, t)),
        _ => None,
    }
}
