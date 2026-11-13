// Type-relative paths through a type alias of a builtin or std type
// (`type V4 = Vec<u8>; V4::new()`, `FxHashMap::default()`), and the
// written Self type of every type-relative builtin path. rustc reads
// `Alias::item` as `<Target>::item`: the builtin items match by their
// canonical std path (`std::vec::Vec::new`), and the Self type the path
// writes (the alias's type, a turbofish on the type segment, or `<T>::`)
// must agree with the builtin's signature, as one check for all of them.
use super::*;

impl<'p> Ctx<'p> {
    /// The type an alias on a value path's type segment names: a plain
    /// alias's type (`type V4 = Vec<u8>`), or a generic alias's body
    /// under the segment's type args (fresh variables when none are
    /// written, the defaults of the missing ones otherwise). The bool is
    /// true for a generic alias, whose written args are its own. None
    /// when the segment names no alias. With `report` off the arg-count
    /// errors are dropped (a second look at a path already reported).
    pub(super) fn alias_seg_ty(&mut self, seg: &ast::PathSeg, span: Span, report: bool) -> Option<(Ty, bool)> {
        let p = self.p;
        if p.aliases.is_empty() && p.gx.generic_aliases.is_empty() {
            return None;
        }
        let bound = self.binding(Namespace::Type, seg.name);
        let flat = match bound {
            BindingRes::Def(def) => p.resolved.as_ref().and_then(|t| t.def_info.get(&def)).map_or(seg.name, |i| i.flat),
            _ => seg.name,
        };
        if let Some(&t) = p.aliases.get(&flat) {
            return Some((t, false));
        }
        if !matches!(bound, BindingRes::Def(_)) {
            return None;
        }
        let la = p.gx.generic_aliases.get(&flat)?.clone();
        let mark = self.errs.len();
        let t = self.generic_alias_body(&la, seg, span);
        if !report {
            self.errs.truncate(mark);
        }
        t.map(|t| (t, true))
    }

    /// A generic alias's body under one path segment's type args (see
    /// `alias_seg_ty`). Reports a wrong arg count (L0107).
    fn generic_alias_body(&mut self, la: &generics::GenericAlias, seg: &ast::PathSeg, span: Span) -> Option<Ty> {
        let p = self.p;
        let written = seg.type_args();
        // An alias has no const params, so a const arg is one too many.
        let consts = matches!(&seg.args, ast::GenericArgs::Angle(a) if a.iter().any(|g| matches!(g, ast::GenericArg::Const(_))));
        if written.len() > la.type_params.len() || consts {
            self.err("L0107", span, format!("this alias takes {} type argument(s)", la.type_params.len()), "Write one type per parameter.");
            return None;
        }
        let mut body = generics::Scope {
            alias_lifetimes: la.lifetimes.iter().map(|&n| (n, Region::Erased)).collect(),
            alias_depth: 1,
            ..Default::default()
        };
        let cx = p.gx.resolve_cx(&p.type_names, &p.aliases);
        for (i, &(name, default)) in la.type_params.iter().enumerate() {
            // rustc infers every param when no args are written, and
            // takes the missing ones' defaults otherwise (E0107 when one
            // has none).
            let t = match (written.get(i), default) {
                (Some(&w), _) => self.resolve_written(w),
                _ if written.is_empty() => self.new_var(false),
                (None, Some(d)) => generics::resolve_scoped(&p.ast, &p.interner, &p.known, &cx, &mut self.tys, d, &body).ok()?,
                (None, None) => {
                    self.err(
                        "L0107",
                        span,
                        format!("this alias takes {} type argument(s)", la.type_params.len()),
                        "Write one type per parameter.",
                    );
                    return None;
                }
            };
            body.param_tys.push((name, t));
        }
        generics::resolve_scoped(&p.ast, &p.interner, &p.known, &cx, &mut self.tys, la.ty, &body).ok()
    }

    /// The builtin type a two-segment value path's alias names
    /// (`V4::new` with `type V4 = Vec<u8>`): `alias_path_target` when it
    /// is a builtin Adt with a canonical std path (or the error type).
    pub(super) fn builtin_alias_ty(&mut self, segs: &[ast::PathSeg], span: Span, report: bool) -> Option<Ty> {
        self.alias_path_target(segs, span, report).filter(|&t| t == ERROR || builtin_std_path(&self.tys, t).is_some())
    }

    /// The type a two-segment value path's alias names when it is not a
    /// user Adt (those keep their own path, `alias_adt_seg`): a builtin
    /// Adt, a primitive, an array, a slice, a tuple. `Alias::item` is
    /// then `<Type>::item`. With `report` on, a wrong arg count on the
    /// alias, or args on a plain alias, is an error here (rustc E0107),
    /// and the result is the error type.
    pub(super) fn alias_path_target(&mut self, segs: &[ast::PathSeg], span: Span, report: bool) -> Option<Ty> {
        if segs.len() != 2 {
            return None;
        }
        let seg = &segs[0];
        let mark = self.errs.len();
        let found = self.alias_seg_ty(seg, span, report);
        let t = match found {
            Some((t, _)) if self.tys.adt_parts(t).is_none_or(|(d, _)| self.p.adt_by_def(d).is_none()) => t,
            // An alias whose args do not resolve (reported).
            None if report && self.errs.len() > mark => return Some(ERROR),
            _ => return None,
        };
        if report && matches!(found, Some((_, false))) && seg.has_args() {
            self.err("L0107", span, "this type alias takes no generic arguments", "Remove the `<..>` part: the alias fixes the type.");
            return Some(ERROR);
        }
        // The type must be well-formed where the path uses it, as a
        // written type (rustc E0277 for `type V = Vec<str>; V::new()`).
        if report {
            self.wf_check_written(t, span);
        }
        Some(t)
    }

    /// `[u8, MAX]` for `U::MAX` through `type U = u8` (a primitive or
    /// `str`): the primitive's own path, which the primitive items and
    /// trait calls match by name. None for other paths.
    pub(super) fn primitive_alias_segs(&mut self, segs: &[ast::PathSeg], span: Span) -> Option<Vec<ast::PathSeg>> {
        let k = &*self.p.known;
        if segs.len() != 2 || k.int_ty(segs[0].name).is_some() {
            return None;
        }
        // Report nothing yet: a builtin Adt alias reports at its own use.
        let mark = self.errs.len();
        let t = self.alias_path_target(segs, span, false);
        self.errs.truncate(mark);
        let name = match self.tys.kind(t?) {
            TyKind::Int(i) => self.p.interner.intern(i.name()),
            TyKind::Float(f) => self.p.interner.intern(f.name()),
            TyKind::Bool => k.bool,
            TyKind::Char => k.char,
            TyKind::Str => k.str,
            _ => return None,
        };
        if segs[0].has_args() {
            self.err("L0107", span, "this type alias takes no generic arguments", "Remove the `<..>` part: the alias fixes the type.");
            return None;
        }
        Some(vec![ast::PathSeg::plain(name), segs[1].clone()])
    }

    /// `OP::Some(x)`, `OP::None`, `RR::Ok(x)`, and `RR::Err(e)` through
    /// an alias of `Option` or `Result`: (true for Option, the variant
    /// flag, true for Some and Ok). The variant's args come from the
    /// alias (`builtin_ctor_args`). Args on the variant are rustc's E0109.
    pub(super) fn alias_ctor_pattern(&mut self, pat: PatId, span: Span) -> Option<(bool, bool)> {
        let p = self.p;
        let (path, elems) = match p.ast.pat(pat) {
            Pat::TupleStruct { path, elems } => (path, Some(elems.len())),
            Pat::Path(path) => (path, None),
            _ => return None,
        };
        let t = self.builtin_alias_ty(path, span, true)?;
        let (k, item) = (&p.known, path.get(1)?.name);
        let found = match (self.tys.builtin_of(t)?, elems) {
            (BuiltinTy::Option, Some(1)) if item == k.Some => (true, true),
            (BuiltinTy::Option, None) if item == k.None => (true, false),
            (BuiltinTy::Result, Some(1)) if item == k.Ok => (false, true),
            (BuiltinTy::Result, Some(1)) if item == k.Err => (false, false),
            _ => return None,
        };
        if path[1].has_args() {
            self.err("L0107", span, "type arguments are not allowed on this variant", "Write the type's arguments on the alias.");
            return None;
        }
        Some(found)
    }

    /// True when a builtin call's callee is a type-relative path with
    /// args on the item (`Vec::new::<u8>`, `<Vec<u8>>::new::<u8>`): no
    /// builtin type's associated fn has params of its own (rustc E0107).
    /// Module fns (`std::mem::swap::<u8>`) take theirs.
    pub(super) fn type_item_has_args(&self, callee: ExprId) -> bool {
        match self.p.ast.expr(callee) {
            Expr::QPath { segs, .. } => segs.last().is_some_and(writes_args),
            Expr::Path(segs) if segs.len() >= 2 && writes_args(&segs[segs.len() - 1]) => {
                let prefix: Vec<&str> = segs[..segs.len() - 1].iter().map(|s| self.name(s.name)).collect();
                !crate::ty::lang::std_is_module_path(&prefix)
            }
            _ => false,
        }
    }

    /// `Cache::default()` through an alias, `String::default()`, or a
    /// user trait's fn (`V4::mk()`): a type-relative call to a name the
    /// builtin type has no item for, through the trait that declares it.
    /// `assoc_call` rejects a user trait's item where std has an item of
    /// the name lust does not model (L0643). None when no trait applies.
    pub(super) fn std_trait_assoc_call(
        &mut self,
        e: ExprId,
        callee: ExprId,
        base: Ty,
        name: Sym,
        args: &[ExprId],
        span: Span,
    ) -> Option<Ty> {
        // `M::clone(&m)` through `type M = HashMap<..>`: the Clone call
        // with a coercion site for `&M` (check/ufcs.rs).
        if name == self.p.known.clone
            && builtin_std_path(&self.tys, base).is_some()
            && let Some(t) = self.clone_call_at(e, base, args, span)
        {
            return Some(t);
        }
        self.assoc_call(e, callee, base, None, name, args, span)
    }

    /// The Self type of a builtin type path with no item of the called
    /// name (`String::default`, `Vec::<u8>::default`): the written Self,
    /// or the type itself when it takes no args. With no written args
    /// on a generic builtin (`let b: Box<L> = Box::default()`), the
    /// expected type names the Self when it is that same builtin.
    /// None otherwise (the args stay unknown here).
    pub(super) fn builtin_path_base(&mut self, segs: &[ast::PathSeg], span: Span, expected: Option<Ty>) -> Option<Ty> {
        let n = segs.len();
        if n < 2 {
            return None;
        }
        if segs[n - 2].has_args() {
            return self.written_builtin_self(segs, span).filter(|&t| t != ERROR);
        }
        let p = self.p;
        let cx = p.gx.resolve_cx(&p.type_names, &p.aliases);
        let scope = self.body_scope.clone();
        if let Ok(t) = generics::resolve_path_ty(&p.ast, &p.interner, &p.known, &cx, &mut self.tys, &scope, &segs[..n - 1], span) {
            return builtin_std_path(&self.tys, t).is_some().then_some(t);
        }
        // The written type needs args it does not write: take them
        // from the expected type when it applies the same builtin
        // the path names (never when a user type of the name is the
        // binding here, which the trait probe above already tried).
        // Else fresh variables, as rustc infers a path's omitted args
        // (`Vec::from_json(v)` for a user trait implemented for `Vec<T>`).
        if let Some(want) = expected.map(|t| self.resolve(t))
            && self.builtin_names_path(want, segs)
        {
            return Some(want);
        }
        let name = self.name(segs[n - 2].name).to_string();
        let fresh = self.fresh_builtin_self(&name)?;
        self.builtin_names_path(fresh, segs).then_some(fresh)
    }

    /// True when builtin Adt type `want` is the type that the path
    /// `segs` (type segments, then the item) names: its std path or,
    /// for a bare name, its name with no user type of that name bound.
    fn builtin_names_path(&mut self, want: Ty, segs: &[ast::PathSeg]) -> bool {
        let n = segs.len();
        let Some((def, _)) = self.tys.adt_parts(want) else { return false };
        let Some(named) = builtin_std_path(&self.tys, want) else { return false };
        let ty_name = segs[n - 2].name;
        if n == 2 && matches!(self.binding(Namespace::Type, ty_name), BindingRes::Def(d) if d != def) {
            return false;
        }
        let path: Vec<&str> = segs[..n - 1].iter().map(|s| self.name(s.name)).collect();
        if n == 2 { named.last() == path.last() } else { named == path || std_reexport(&path) == Some(named.as_slice()) }
    }

    /// A generic builtin type by its name with a fresh variable for each
    /// type arg (`Vec` gives `Vec<?T>`), for `builtin_path_base`; it
    /// checks the path and binding after.
    fn fresh_builtin_self(&mut self, name: &str) -> Option<Ty> {
        let v = self.new_var(false);
        Some(match name {
            "Vec" => self.tys.mk_vec(v),
            "Option" => self.tys.mk_option(v),
            "Box" => self.tys.mk_boxed(v),
            "Rc" => self.tys.mk_rc(v),
            "RefCell" => self.tys.mk_builtin1(|d| d.refcell, v),
            "Cell" => self.tys.mk_builtin1(|d| d.cell, v),
            "Arc" => crate::ty::sync::sync_ty(&mut self.tys, crate::ty::sync::SyncTy::Arc, &[v]),
            "HashMap" => {
                let (w, s) = (self.new_var(false), self.new_var(false));
                crate::ty::hash::mk_hash(&mut self.tys, crate::ty::hash::HashTy::HashMap, &[v, w, s])
            }
            "HashSet" => {
                let s = self.new_var(false);
                crate::ty::hash::mk_hash(&mut self.tys, crate::ty::hash::HashTy::HashSet, &[v, s])
            }
            "Result" => {
                let e = self.new_var(false);
                self.tys.mk_result(v, e)
            }
            _ => return None,
        })
    }

    /// The `n` type args of a written builtin Self type (`Option<u8>`
    /// for `<Option<u8>>::Some`, an alias's type for `OP::Some`); error
    /// types when it has another count.
    pub(super) fn builtin_self_args(&self, t: Ty, n: usize) -> Vec<Ty> {
        let args: Vec<Ty> = match self.tys.adt_parts(self.resolve(t)) {
            Some((_, a)) => self.tys.args(a).iter().filter_map(|g| g.as_ty()).collect(),
            None => Vec::new(),
        };
        if args.len() == n { args } else { vec![ERROR; n] }
    }

    /// `[std, vec, Vec, item]` for a builtin Adt type `t`: the canonical
    /// std path `resolve_assoc` matches builtin items by. The item keeps
    /// its written args. None when `t` has no such path.
    pub(super) fn canonical_item_segs(&mut self, t: Ty, item: &ast::PathSeg) -> Option<Vec<ast::PathSeg>> {
        let path = builtin_std_path(&self.tys, t)?;
        let mut segs: Vec<ast::PathSeg> = path.iter().map(|s| ast::PathSeg::plain(self.p.interner.intern(s))).collect();
        segs.push(item.clone());
        Some(segs)
    }

    /// A builtin type's own associated item (`<Vec<u8>>::new`,
    /// `V4::with_capacity`), by its canonical path. None, with nothing
    /// reported, when the builtin has no item of that name: the caller
    /// then probes the traits, as rustc does after inherent items.
    pub(super) fn builtin_type_item(&mut self, t: Ty, item: &ast::PathSeg, span: Span) -> Option<Res> {
        let segs = self.canonical_item_segs(t, item)?;
        let mark = self.errs.len();
        let r = self.resolve_assoc(&segs, span);
        if r.is_none() {
            self.errs.truncate(mark);
        }
        r
    }

    /// The Self type a type-relative builtin path writes: an alias's
    /// type (`type V4 = Vec<u8>; V4::new()`), or the type segment with
    /// its turbofish (`Rc::<u8>::strong_count`, `std::boxed::Box::<u8>::new`).
    /// None when the path writes no args for the type (they are
    /// inferred). The builtin's signature must agree with it
    /// (`unify_builtin_self`).
    pub(super) fn written_builtin_self(&mut self, segs: &[ast::PathSeg], span: Span) -> Option<Ty> {
        if segs.len() < 2 {
            return None;
        }
        if let Some(t) = self.builtin_alias_ty(segs, span, false) {
            return Some(t);
        }
        let n = segs.len();
        if !segs[n - 2].has_args() {
            return None;
        }
        // Each written arg as a written type (binding reads, bounds, its
        // own errors), then the whole type for the arg count.
        let mark = self.errs.len();
        for a in segs[n - 2].type_args() {
            self.resolve_written(a);
        }
        if self.errs.len() > mark {
            return Some(ERROR);
        }
        let p = self.p;
        let cx = p.gx.resolve_cx(&p.type_names, &p.aliases);
        let scope = self.body_scope.clone();
        match generics::resolve_path_ty(&p.ast, &p.interner, &p.known, &cx, &mut self.tys, &scope, &segs[..n - 1], span) {
            Ok(t) => Some(self.instantiate_infers(t)),
            Err(d) => {
                self.errs.push(d);
                Some(ERROR)
            }
        }
    }

    /// Unifies the Self type of a builtin call's signature with the
    /// written one (`written_builtin_self`), before the args are checked:
    /// the first of the return type and the first param (behind
    /// references) that applies the same builtin, else a bare variable
    /// return (`Vec::clone`). A Self that is not a generic Adt has
    /// nothing to agree on. False after reporting a mismatch.
    pub(super) fn unify_builtin_self(&mut self, want: Ty, params: &[Ty], ret: Ty, item: Sym, span: Span) -> bool {
        let want = self.resolve(want);
        let Some((def, args)) = self.tys.adt_parts(want) else { return true };
        if self.tys.args(args).is_empty() {
            return true;
        }
        let peel = |this: &Self, mut t: Ty| {
            t = this.resolve(t);
            while let TyKind::Ref(_, _, inner) = this.tys.kind(t) {
                t = this.resolve(inner);
            }
            t
        };
        let slot = [Some(ret), params.first().copied()]
            .into_iter()
            .flatten()
            .map(|t| peel(self, t))
            .find(|&t| self.tys.adt_parts(t).is_some_and(|(d, _)| d == def))
            .or_else(|| matches!(self.tys.kind(self.resolve(ret)), TyKind::Infer(_)).then_some(ret));
        let Some(slot) = slot else {
            let tn = self.tn(want);
            self.err("L0104", span, format!("no associated function `{}` for `{tn}`", self.name(item)), "Check the type's arguments.");
            return false;
        };
        if self.unify(slot, want) {
            return true;
        }
        self.mismatch(span, want, slot);
        false
    }
}

/// The canonical std path of a builtin Adt type's head (`Vec<u8>` is
/// `std::vec::Vec`). None for other types and for the builtins with no
/// associated items to name (iterators, fmt structs, guards).
pub(super) fn builtin_std_path(tys: &impl TyIntern, t: Ty) -> Option<Vec<&'static str>> {
    let mut path = vec!["std"];
    match tys.builtin_of(t)? {
        BuiltinTy::String => path.extend(["string", "String"]),
        BuiltinTy::Vec => path.extend(["vec", "Vec"]),
        BuiltinTy::Option => path.extend(["option", "Option"]),
        BuiltinTy::Result => path.extend(["result", "Result"]),
        BuiltinTy::Boxed => path.extend(["boxed", "Box"]),
        BuiltinTy::Rc => path.extend(["rc", "Rc"]),
        BuiltinTy::Weak => path.extend(["rc", "Weak"]),
        BuiltinTy::RefCell => path.extend(["cell", "RefCell"]),
        BuiltinTy::Cell => path.extend(["cell", "Cell"]),
        BuiltinTy::ExitCode => path.extend(["process", "ExitCode"]),
        BuiltinTy::Sync(k) => {
            path.extend(k.module());
            path.push(k.name());
        }
        BuiltinTy::Hash(k) if !matches!(k, crate::ty::hash::HashTy::Entry | crate::ty::hash::HashTy::OccupiedEntry | crate::ty::hash::HashTy::VacantEntry) && !k.is_iter_form() => {
            let (module, name) = k.path();
            path = module.to_vec();
            path.push(name);
        }
        _ => return None,
    }
    Some(path)
}

/// The canonical path (`builtin_std_path`'s) of a std re-export path of a
/// builtin type: `std::collections::HashMap` is
/// `std::collections::hash_map::HashMap`.
fn std_reexport(path: &[&str]) -> Option<&'static [&'static str]> {
    match path {
        ["std", "collections", "HashMap"] => Some(&["std", "collections", "hash_map", "HashMap"]),
        ["std", "collections", "HashSet"] => Some(&["std", "collections", "hash_set", "HashSet"]),
        _ => None,
    }
}

/// True when a segment writes generic args (`::<>` writes none).
fn writes_args(seg: &ast::PathSeg) -> bool {
    match &seg.args {
        ast::GenericArgs::None => false,
        ast::GenericArgs::Angle(a) => !a.is_empty(),
        ast::GenericArgs::Parenthesized { .. } => true,
    }
}
