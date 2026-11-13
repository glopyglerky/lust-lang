// `Extend` on Vec and String in body checking (W-STD-B). The argument
// goes through `IntoIterator` (`ty::iter::as_iter_ty`, or `ByNext` for an
// iterator lust calls through its `next`); the item kinds
// are the impls std declares: `Vec<T>: Extend<T>` and `Extend<&T>` for
// `T: Copy`; `String: Extend<char | &char | &str | String | Box<str> |
// Cow<str>>`. The iterator expansion (lower/iter.rs) writes Rust's
// exact reserve and push sequence. Maps and sets: check/hashmap.rs.
use super::*;

/// An `extend` call whose element was still open while its items were
/// references, so std's `Extend<T>` and `Extend<&T>` (a map's `(&K, &V)`)
/// both fitted (rustc E0282 unless a later use fixes the element).
/// `finish_deferred_extends` decides it at body end.
pub(super) struct DeferredExtend {
    /// The receiver's element type (a map's `(K, V)`).
    pub(super) elem: Ty,
    /// The argument's item type; for a source-written collection, the
    /// fresh element its probe bound instead (`source_extend_base`).
    pub(super) item: Ty,
    pub(super) map: bool,
    /// lust has the reference-item form for this receiver (Vec and the
    /// hash collections); source-written collections have only `Extend<T>`.
    pub(super) by_ref: bool,
    pub(super) span: Span,
}

/// The `Extend` impls of a collection that fit one call (`extend_pick`).
pub(super) enum ExtendPick {
    Owned,
    /// Only the reference form, with the referents.
    ByRef(Vec<Ty>),
    Both,
    Neither,
}

impl<'p> Ctx<'p> {
    /// L0610 for an `extend` whose item type is still open (rustc E0283):
    /// std has `Extend<T>` and `Extend<&T>` (and `(K, V)`, `(&K, &V)` for
    /// maps), and an open item picks neither. A map item counts as open
    /// when both tuple parts are.
    pub(super) fn extend_item_open(&mut self, item: Ty, arg: ExprId) -> bool {
        // A fn item or closure output (`.map(Enum::Variant)`) is known
        // once the queue runs.
        let d = self.deep(item);
        if self.open_extend_item(d) {
            self.run_obligations(false);
        }
        let item = self.deep(item);
        let open = self.open_extend_item(item);
        if open {
            self.err(
                "L0610",
                self.span(arg),
                "type annotations needed: the item type of this `extend` must be known here",
                "Annotate the element type, such as `Vec::<i32>::new()`.",
            );
        }
        open
    }

    /// True when an `Extend` item type picks none of std's impls: a type
    /// variable, or a tuple of them (`extend_item_open`, `std_extend_item_open`).
    pub(super) fn open_extend_item(&self, item: Ty) -> bool {
        match self.kind(item) {
            TyKind::Tuple(ts) => {
                let ts = self.tys.tys_of(ts);
                !ts.is_empty() && ts.iter().all(|&t| self.unbound_general(t))
            }
            _ => self.unbound_general(item),
        }
    }

    /// `extend` on a source-written collection (std's `VecDeque`,
    /// `BTreeSet`, `BTreeMap`; the indexmap adapter's `IndexMap`,
    /// `IndexSet`) before the method probe: the call's type, or None for
    /// the probe. rustc E0283: an open item type picks none of
    /// `Extend<T>` and `Extend<&T>`. rustc E0282: with an open element
    /// and `&` items both impls fit until a later use fixes the element,
    /// so the call is typed here and `finish_deferred_extends` decides;
    /// the impl is selected at body end (`defer_trait_method`). Source
    /// writes only `Extend<T>` (method selection cannot pick between two
    /// `Extend` impls yet), so the reference form stays a reject.
    pub(super) fn source_extend(&mut self, e: ExprId, base: Ty, derefs: u8, args: &[ExprId]) -> Option<Ty> {
        let map = self.source_extend_coll(base)?;
        let [arg] = args else { return None };
        let trial = method::ArgTrial::mark(self);
        let at = self.expr(*arg, None);
        let at = self.deep(at);
        let item = self.extend_arg_item(at);
        if let Some(i) = item {
            let d = self.deep(i);
            if self.open_extend_item(d) {
                self.run_obligations(false);
            }
        }
        let item = item.map(|i| self.deep(i));
        let open = item.is_some_and(|i| self.open_extend_item(i));
        let targs: Vec<GenericArg> = self.tys.adt_parts(base).map(|(_, a)| self.tys.args(a).to_vec()).unwrap_or_default();
        let elem = self.extend_elem(&targs, map);
        let both = !open && item.is_some_and(|i| matches!(self.extend_pick(elem, i, map), ExtendPick::Both));
        trial.rollback(self);
        if open {
            self.expr(*arg, None);
            self.err(
                "L0610",
                self.span(*arg),
                "type annotations needed: the item type of this `extend` must be known here",
                "Annotate the element type, such as `Vec::<i32>::new()`.",
            );
            return Some(ERROR);
        }
        if !both {
            return None;
        }
        let at = self.expr(*arg, None);
        let at = self.deep(at);
        let item = self.extend_arg_item(at).unwrap_or(ERROR);
        let span = self.span(*arg);
        self.deferred_extend.push(DeferredExtend { elem, item, map, by_ref: false, span });
        let extend = self.p.gx.lang.extend;
        let tr_args = self.tys.intern_args(&[GenericArg::Ty(base), GenericArg::Ty(elem)]);
        let tr = TraitRef { def: extend, args: tr_args, bindings: EMPTY_BINDINGS };
        let name = self.p.interner.intern("extend");
        // The one written impl's `extend` until body end selects it:
        // closure capture analysis reads the call's receiver mode first.
        let snap = self.snapshot_vars();
        let obl = self.obl_len();
        let sel = self.select_obligation(tr);
        self.rollback_vars(snap);
        self.obl_truncate(obl);
        if let Selection::Impl { id, .. } = sel {
            let p = self.p;
            let f = p.gx.index.get(id).items.iter().find_map(|x| p.local_def(x.hash).filter(|&l| p.fn_info(l).name == name));
            if let Some(f) = f {
                self.set_res(e, Res::Method { f, derefs, autoref: Autoref::RefMut });
            }
        }
        method::defer_trait_method(self, e, tr, name, vec![GenericArg::Ty(at)], derefs, Autoref::RefMut);
        self.record_pick(e, extend, derefs, ReceiverForm::RefMut);
        Some(UNIT)
    }

    /// The item type of an `extend` argument: a builtin iterator's, else
    /// its `IntoIterator::Item` (source-written iterators such as
    /// `btree_map::Iter`); None when that does not normalize.
    fn extend_arg_item(&mut self, at: Ty) -> Option<Ty> {
        if let Some(i) = crate::ty::iter::as_iter_ty(&mut self.tys, at).and_then(|it| crate::ty::iter::item_ty(&mut self.tys, it)) {
            return Some(i);
        }
        let args = self.tys.intern_args(&[GenericArg::Ty(at)]);
        let proj = self.tys.intern(TyKind::Projection(AssocId(self.p.gx.lang.assoc_into_item), args));
        let item = self.normalize_here(proj);
        let item = self.deep(item);
        (!self.tys.flags(item).contains_projection()).then_some(item)
    }

    /// Whether `base` is a source-written collection with both `Extend`
    /// forms in std (see `source_extend`), and if so whether it is a map.
    fn source_extend_coll(&self, base: Ty) -> Option<bool> {
        let (def, _) = self.tys.adt_parts(base)?;
        let a = self.tys.local_adt_of(def)?;
        let adt = self.p.adt(a);
        let path = self.p.display_path(adt.name);
        let name = path.rsplit("::").next()?;
        let map = if self.p.is_std_adt(a) {
            match name {
                "VecDeque" | "BTreeSet" => false,
                "BTreeMap" => true,
                _ => return None,
            }
        } else if self.p.adapter_at(adt.span).is_some_and(|c| c.name == "indexmap") {
            match name {
                "IndexSet" => false,
                "IndexMap" => true,
                _ => return None,
            }
        } else {
            return None;
        };
        Some(map)
    }

    /// The element type a collection's `Extend<T>` takes: the first type
    /// argument, or a map's `(K, V)`.
    fn extend_elem(&mut self, targs: &[GenericArg], map: bool) -> Ty {
        let ts: Vec<Ty> = targs.iter().filter_map(|g| g.as_ty()).take(if map { 2 } else { 1 }).collect();
        match ts[..] {
            [k, v] if map => {
                let ts = self.tys.intern_tys(&[k, v]);
                self.tys.intern(TyKind::Tuple(ts))
            }
            [t, ..] => t,
            _ => ERROR,
        }
    }

    /// Which of std's `Extend` impls fit a collection with element `elem`
    /// (a map's `(K, V)`) and items `item`: `Extend<T>`, and the
    /// reference form `Extend<&T>` (`Extend<(&K, &V)>`) whose parts must be
    /// Copy. Trial unification only; nothing stays bound.
    pub(super) fn extend_pick(&mut self, elem: Ty, item: Ty, map: bool) -> ExtendPick {
        let owned = self.unifies(elem, item);
        let parts = self.ref_item_parts(item, map).filter(|ps| {
            ps.iter().all(|&p| {
                let p = self.deep(p);
                self.tys.flags(p).contains_infer() || is_copy(self.p, &self.tys, p)
            })
        });
        let by_ref = parts.filter(|ps| {
            let x = self.owned_item(ps, map);
            self.unifies(elem, x)
        });
        match (owned, by_ref) {
            (true, Some(_)) => ExtendPick::Both,
            (true, None) => ExtendPick::Owned,
            (false, Some(ps)) => ExtendPick::ByRef(ps),
            (false, None) => ExtendPick::Neither,
        }
    }

    /// The referents of a reference item: `&X` gives `[X]`, a map's
    /// `(&K, &V)` gives `[K, V]`.
    fn ref_item_parts(&mut self, item: Ty, map: bool) -> Option<Vec<Ty>> {
        let item = self.deep(item);
        let refd = |cx: &Self, t: Ty| match cx.kind(t) {
            TyKind::Ref(_, Mut::Shared, x) => Some(x),
            _ => None,
        };
        if !map {
            return refd(self, item).map(|x| vec![x]);
        }
        let TyKind::Tuple(ts) = self.kind(item) else { return None };
        match self.tys.tys_of(ts) {
            &[k, v] => Some(vec![refd(self, k)?, refd(self, v)?]),
            _ => None,
        }
    }

    /// The item a reference item's parts make: `X`, or a map's `(K, V)`.
    fn owned_item(&mut self, parts: &[Ty], map: bool) -> Ty {
        if !map {
            return parts[0];
        }
        let ts = self.tys.intern_tys(parts);
        self.tys.intern(TyKind::Tuple(ts))
    }

    /// True when `a` and `b` unify; the bindings are rolled back.
    fn unifies(&mut self, a: Ty, b: Ty) -> bool {
        let snap = self.snapshot_vars();
        let obl = self.obl_len();
        let ok = self.unify(a, b);
        self.rollback_vars(snap);
        self.obl_truncate(obl);
        ok
    }

    /// Body end for the `extend` calls whose choice of `Extend` impl
    /// waited on a later use (`DeferredExtend`): an element still open
    /// to both impls is rustc's E0282; otherwise the one impl that fits
    /// binds it, as rustc selects once the element is known.
    pub(super) fn finish_deferred_extends(&mut self) {
        for d in std::mem::take(&mut self.deferred_extend) {
            let (elem, item) = (self.deep(d.elem), self.deep(d.item));
            if self.has_error_kind(elem) || self.has_error_kind(item) {
                continue;
            }
            match self.extend_pick(elem, item, d.map) {
                // An instance check sees concrete types where its template
                // saw a parameter that is not Copy: the template picked
                // `Extend<T>` (an open choice there was its own error).
                ExtendPick::Both if self.is_instance_check() => {
                    self.unify(elem, item);
                }
                ExtendPick::Both => self.err(
                    "L0610",
                    d.span,
                    "type annotations needed: the element type of this `extend` must be known (both `Extend<T>` and `Extend<&T>` fit)",
                    "Annotate the element type, such as `VecDeque::<u8>::new()`.",
                ),
                ExtendPick::Owned => {
                    self.unify(elem, item);
                }
                ExtendPick::ByRef(parts) if d.by_ref => {
                    let x = self.owned_item(&parts, d.map);
                    self.unify(elem, x);
                    for p in parts {
                        self.deferred_copy.push((p, d.span));
                    }
                }
                ExtendPick::ByRef(_) => {
                    let tn = self.tn(item);
                    self.errs.push(unsupported(d.span, &format!("`extend` with items of type `{tn}` into a collection of their referents")));
                }
                ExtendPick::Neither => self.mismatch(d.span, elem, item),
            }
        }
    }

    /// `extend` on a Vec or String receiver, or None for other receivers.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn seq_extend(
        &mut self,
        e: ExprId,
        recv: ExprId,
        base: Ty,
        derefs: u8,
        name_span: Span,
        args: &[ExprId],
    ) -> Option<Ty> {
        let vec_el = self.tys.vec_elem(base);
        if vec_el.is_none() && !self.tys.is_string_ty(base) {
            return None;
        }
        let [a] = args else {
            self.err("L0105", name_span, "`extend` takes 1 argument", "Pass the iterator.");
            for &a in args {
                self.expr(a, None);
            }
            return Some(ERROR);
        };
        let at = self.expr(*a, None);
        let at = self.deep(at);
        let Some(it) = crate::ty::iter::as_iter_ty(&mut self.tys, at).or_else(|| self.wrapper_iter_arg(at)).or_else(|| self.into_iterator_arg(*a, at)) else {
            let tn = self.tn(at);
            self.err("L0640", self.span(*a), format!("`extend` from `{tn}` is not implemented yet"), "Pass an iterator, a Vec, or a slice.");
            return Some(ERROR);
        };
        if it == ERROR {
            return Some(ERROR);
        }
        let item = crate::ty::iter::item_ty(&mut self.tys, it).unwrap_or(ERROR);
        let item = self.deep(item);
        if self.extend_item_open(item, *a) {
            return Some(ERROR);
        }
        if let Some(el) = vec_el
            && matches!(self.extend_pick(el, item, false), ExtendPick::Both)
        {
            self.deferred_extend.push(DeferredExtend { elem: el, item, map: false, by_ref: true, span: self.span(*a) });
            self.check_packed_borrow(recv, name_span);
            self.set_res(e, Res::BuiltinMethod { b: Builtin::VecExtend, derefs, autoref: Autoref::RefMut });
            return Some(UNIT);
        }
        let b = match vec_el {
            Some(el) => {
                // `Extend<T>`, else `Extend<&T>` for `T: Copy`.
                let snap = self.snapshot_vars();
                let obl = self.obl_len();
                if self.unify(el, item) {
                    Builtin::VecExtend
                } else {
                    self.rollback_vars(snap);
                    self.obl_truncate(obl);
                    // Only shared items: std has no `Extend<&mut T>`.
                    match self.kind(item) {
                        TyKind::Ref(_, Mut::Shared, x) if self.unify(el, x) => {
                            self.deferred_copy.push((x, self.span(*a)));
                            Builtin::VecExtend
                        }
                        _ => {
                            self.mismatch(self.span(*a), el, item);
                            return Some(ERROR);
                        }
                    }
                }
            }
            None => {
                let k = self.kind(item);
                let ok = item == CHAR
                    || item == STRING
                    || matches!(k, TyKind::Ref(_, Mut::Shared, x) if self.resolve(x) == CHAR || self.resolve(x) == STR)
                    || self.tys.smart_elem(item) == Some(STR)
                    || crate::check::collect::cow_arg(&self.tys, item) == Some(STR);
                if !ok {
                    let tn = self.tn(item);
                    self.err("L0612", self.span(*a), format!("a `String` cannot be extended with items of type `{tn}`"), "Extend with chars or strs.");
                    return Some(ERROR);
                }
                Builtin::StringExtend
            }
        };
        self.check_packed_borrow(recv, name_span);
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::RefMut });
        Some(UNIT)
    }

    /// `x.extend(iter)` where `x`'s type is a type parameter with one
    /// `Extend<A>` bound (ts-rust frontend/core_ls_ext.rs copy_map_into): the
    /// argument must be `IntoIterator<Item = A>`. Only a template check
    /// meets a parameter receiver; its instances check the concrete
    /// `extend` (Vec, String, maps and sets, or a written impl).
    pub(super) fn param_extend(&mut self, e: ExprId, base: Ty, derefs: u8, name_span: Span, args: &[ExprId]) -> Option<Ty> {
        if !matches!(self.kind(base), TyKind::Param(_)) {
            return None;
        }
        let lang = self.p.gx.lang;
        let items: Vec<Ty> = self
            .env()
            .iter()
            .filter_map(|q| match q {
                Predicate::Trait(b) if b.count == 0 && b.value.def == lang.extend && self.tys.args(b.value.args).first().and_then(|g| g.as_ty()) == Some(base) => {
                    self.tys.args(b.value.args).get(1).and_then(|g| g.as_ty())
                }
                _ => None,
            })
            .collect();
        let [a_ty] = items.as_slice() else { return None };
        let a_ty = *a_ty;
        let [a] = args else {
            self.err("L0105", name_span, "`extend` takes 1 argument", "Pass the iterator.");
            for &a in args {
                self.expr(a, None);
            }
            return Some(ERROR);
        };
        let at = self.expr(*a, None);
        let span = self.span(*a);
        let targs = self.tys.intern_args(&[GenericArg::Ty(at)]);
        self.obl.push(obligations::Obligation::Trait { trait_ref: TraitRef { def: lang.into_iterator, args: targs, bindings: EMPTY_BINDINGS }, span, depth: 0 });
        self.obl.push(obligations::Obligation::AssocEq { eq: AssocEqData { assoc: lang.assoc_into_item, trait_args: targs, ty: a_ty }, span });
        self.set_res(e, Res::BuiltinMethod { b: Builtin::ExtendParam, derefs, autoref: Autoref::RefMut });
        Some(UNIT)
    }
}
