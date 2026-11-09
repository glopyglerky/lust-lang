// Iterator methods in body checking (W-STD-A phase 3, ty/iter.rs).
//
// Sources (`iter`, `iter_mut`, `into_iter` on Vec, slices, and
// arrays; integer ranges), adapters, and consumers type here as
// builtin methods. A closure argument checks against a fresh type
// variable that carries the pending `FnMut` obligation of the Rust
// signature, so its parameter types come from the obligation as for a
// generic call (closure.rs `expected_closure_sig`). Lowering emits one
// builtin call per method; `lower/iter.rs` expands them after borrowck.
//
// Rule zero: a method, receiver form, or argument shape outside this
// list rejects (L0640 for the declared Iterator surface lust lacks).
use super::*;
use crate::ty::iter::{self as it, IterKind};

/// Iterator methods Rust declares that lust does not implement yet:
/// a call rejects with L0640 naming the method, never L0104.
const DECLARED_ONLY: &[&str] = &[
    "eq",
    "ne",
    "next_chunk",
    "advance_by",
    "flat_map",
    "flatten",
    "scan",
    "fuse",
    "partition_in_place",
    "is_partitioned",
    "try_for_each",
    "reduce",
    "try_reduce",
    "try_find",
    "rposition",
    "rev_position",
    "copied_",
    "cycle",
    "array_chunks",
    "sum_",
    "cmp",
    "cmp_by",
    "partial_cmp",
    "partial_cmp_by",
    "eq_by",
    "lt",
    "le",
    "gt",
    "ge",
    "is_sorted",
    "is_sorted_by",
    "is_sorted_by_key",
    "intersperse",
    "intersperse_with",
    "map_windows",
    "collect_into",
    "rfold",
    "rfind",
    "nth_back",
    "max_by",
    "min_by",
];

/// The answer of a `from_iter` target probe (`from_iter_ty_target`):
/// a type this call builds, an error already reported, or no claim.
enum FromIterProbe {
    Found(Ty),
    Bail,
    Miss,
}

/// How a concrete `from_iter` target builds (`from_iter_class`).
#[derive(PartialEq)]
enum FromIterClass {
    Collect,
    BTree,
    /// `Result<BTreeMap<..>, E>` and kin (F3): core's generic
    /// try-collect impl over the BTree target.
    TryBTree,
    Unsupported,
}

impl<'p> Ctx<'p> {
    /// The callable an iterator adapter's callable arg names, under its
    /// references (`&mut f`, `&&c`), deeply resolved.
    fn peel_callable_refs(&mut self, t: Ty) -> Ty {
        let mut t = self.deep(t);
        while let TyKind::Ref(_, _, inner) = self.kind(t) {
            t = self.deep(inner);
        }
        t
    }

    /// True when an adapter's callable of type `t` is a `dyn` value (behind
    /// references and boxes) that the expansion cannot call as `FnMut`: a
    /// `dyn FnOnce` (lower/iter.rs `call_fn` calls `dyn Fn` and `dyn FnMut`).
    fn dyn_once_callable(&mut self, t: Ty) -> bool {
        let mut t = self.peel_callable_refs(t);
        while let Some(inner) = self.tys.boxed_elem(t) {
            t = self.peel_callable_refs(inner);
        }
        if !matches!(self.kind(t), TyKind::Dyn(..)) {
            return false;
        }
        let lang = self.p.gx.lang;
        crate::check::object::principal_of(&self.tys, t).is_none_or(|pr| pr.def != lang.fn_ && pr.def != lang.fn_mut)
    }

    /// `iter`, `iter_mut`, and `into_iter` on Vec, slices, and arrays,
    /// and every iterator method on a builtin iterator or `Chars`
    /// receiver. Returns None when the call is none of these, so the
    /// caller's other dispatch runs.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn iter_method(
        &mut self,
        e: ExprId,
        recv: ExprId,
        rt: Ty,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
        expected: Option<Ty>,
    ) -> Option<Ty> {
        let seq_elem = match self.kind(base) {
            TyKind::Slice(el) | TyKind::Array(el, _) => Some(el),
            _ => self.tys.vec_elem(base),
        };
        if let Some(el) = seq_elem {
            // Every method call on a Vec passes here: compare the
            // borrowed name before anything allocates.
            let n = match self.name(name) {
                "iter" => "iter",
                "iter_mut" => "iter_mut",
                "into_iter" => "into_iter",
                "sort" => "sort",
                "sort_unstable" => "sort_unstable",
                "sort_by" => "sort_by",
                "sort_unstable_by" => "sort_unstable_by",
                "sort_by_key" => "sort_by_key",
                "sort_unstable_by_key" => "sort_unstable_by_key",
                "binary_search" => "binary_search",
                "binary_search_by" => "binary_search_by",
                "binary_search_by_key" => "binary_search_by_key",
                "windows" => "windows",
                "chunks" => "chunks",
                "chunks_mut" => "chunks_mut",
                "chunks_exact" => "chunks_exact",
                "chunks_exact_mut" => "chunks_exact_mut",
                "utf8_chunks" => "utf8_chunks",
                "split" => "split",
                "split_mut" => "split_mut",
                "rsplit" => "rsplit",
                "rsplit_mut" => "rsplit_mut",
                "splitn" => "splitn",
                "splitn_mut" => "splitn_mut",
                "rsplitn" => "rsplitn",
                "rsplitn_mut" => "rsplitn_mut",
                "split_inclusive" => "split_inclusive",
                "split_inclusive_mut" => "split_inclusive_mut",
                "chunk_by" => "chunk_by",
                "chunk_by_mut" => "chunk_by_mut",
                _ => return None,
            };
            if n == "chunk_by" || n == "chunk_by_mut" {
                // core's `chunk_by` (R13): the slice by reference and
                // `F: FnMut(&T, &T) -> bool`, both references
                // higher-ranked.
                let mutable = n == "chunk_by_mut";
                let elem_ref = self.tys.mk_ref(false, el);
                let pred = self.fn_mut_param_ref(&[elem_ref, elem_ref], BOOL, name_span);
                let k = if mutable { IterKind::ChunkByMut } else { IterKind::ChunkBy };
                let ret = it::mk_iter(&mut self.tys, k, &[el, pred]);
                let autoref = if mutable { Autoref::RefMut } else { Autoref::Ref };
                self.set_res(e, Res::BuiltinMethod { b: Builtin::IterChunkBy, derefs, autoref });
                self.check_args(&[pred], args, name_span);
                return Some(ret);
            }
            if let Some((mode, mutable)) = it::SliceSplitMode::of_method(n) {
                // The slice splits (core slice/iter.rs): the slice by
                // reference (mutable for the `_mut` forms) and
                // `F: FnMut(&T) -> bool`, after the count for the N forms.
                let elem_ref = self.tys.mk_ref(false, el);
                let pred = self.fn_mut_param_ref(&[elem_ref], BOOL, name_span);
                let ret = it::mk_slice_split(&mut self.tys, el, pred, mode, mutable);
                let autoref = if mutable { Autoref::RefMut } else { Autoref::Ref };
                self.set_res(e, Res::BuiltinMethod { b: Builtin::IterSliceSplit, derefs, autoref });
                let counted = matches!(mode, it::SliceSplitMode::SplitN | it::SliceSplitMode::RSplitN);
                let params: Vec<Ty> = if counted { vec![USIZE, pred] } else { vec![pred] };
                self.check_args(&params, args, name_span);
                return Some(ret);
            }
            if n == "utf8_chunks" {
                // `[u8]::utf8_chunks` (tf-slice): bytes only.
                if el != int(IntTy::U8) {
                    return None;
                }
                let ret = it::mk_iter(&mut self.tys, IterKind::Utf8Chunks, &[el]);
                self.set_res(e, Res::BuiltinMethod { b: Builtin::IterUtf8Chunks, derefs, autoref: Autoref::Ref });
                self.check_args(&[], args, name_span);
                return Some(ret);
            }
            if let Some(kind) = match n {
                "chunks" => Some(IterKind::Chunks),
                "chunks_mut" => Some(IterKind::ChunksMut),
                "chunks_exact" => Some(IterKind::ChunksExact),
                "chunks_exact_mut" => Some(IterKind::ChunksExactMut),
                _ => None,
            } {
                // The chunk family (tf-slice): the slice by
                // reference (mutable for the `_mut` forms).
                let mutable = matches!(kind, IterKind::ChunksMut | IterKind::ChunksExactMut);
                let ret = it::mk_iter(&mut self.tys, kind, &[el]);
                let autoref = if mutable { Autoref::RefMut } else { Autoref::Ref };
                self.set_res(e, Res::BuiltinMethod { b: Builtin::IterChunks, derefs, autoref });
                self.check_args(&[USIZE], args, name_span);
                return Some(ret);
            }
            if n == "windows" {
                // `slice::windows(size)` (W-STD-B): the slice by reference.
                let ret = it::mk_iter(&mut self.tys, IterKind::Windows, &[el]);
                self.set_res(e, Res::BuiltinMethod { b: Builtin::IterWindows, derefs, autoref: Autoref::Ref });
                self.check_args(&[USIZE], args, name_span);
                return Some(ret);
            }
            if n.starts_with("sort") || n.starts_with("binary_search") {
                return Some(self.seq_sort(e, recv, el, derefs, n, name_span, args));
            }
            return self.iter_source(e, rt, base, el, derefs, n, name_span, args);
        }
        // `iter`, `iter_mut`, and `into_iter` on an `Option` or
        // `Result` receiver (fx-option-rest, ty/iter.rs): core's
        // six one-item iterators. Other names fall through.
        if self.tys.option_elem(base).is_some() || self.tys.result_args(base).is_some() {
            let is_opt = self.tys.option_elem(base).is_some();
            let el = if is_opt { self.tys.option_elem(base).unwrap_or(ERROR) } else { self.tys.result_args(base).map_or(ERROR, |(t, _)| t) };
            let (kind_iter, kind_iter_mut) = if is_opt {
                (IterKind::OptionIter, IterKind::OptionIterMut)
            } else {
                (IterKind::ResultIter, IterKind::ResultIterMut)
            };
            let (autoref, d, ret) = match self.name(name) {
                "iter" => (Autoref::Ref, derefs, it::mk_iter(&mut self.tys, kind_iter, &[el])),
                "iter_mut" => (Autoref::RefMut, derefs, it::mk_iter(&mut self.tys, kind_iter_mut, &[el])),
                "into_iter" => {
                    // Rust's probe, as in `iter_source`: the first
                    // autoderef step with an `IntoIterator` impl.
                    let mut t = self.resolve(rt);
                    let mut found = None;
                    for j in 0..=derefs {
                        if let Some(it) = it::opt_res_iter(&mut self.tys, t) {
                            found = Some((j, it));
                            break;
                        }
                        match self.kind(t) {
                            TyKind::Ref(_, _, inner) => t = self.resolve(inner),
                            _ => break,
                        }
                    }
                    // No step in the reference chain fits (the enum
                    // sits behind a smart pointer): not yet.
                    let Some((j, ret)) = found else {
                        self.err("L0640", name_span, "`into_iter` on this receiver is not implemented yet", "Iterate with `.iter()` instead.");
                        return Some(self.iter_bail(args));
                    };
                    (Autoref::None, j, ret)
                }
                _ => return None,
            };
            if !turbofish.is_empty() {
                self.errs.push(not_yet("L0721", name_span, "turbofish on this iterator method"));
                return Some(self.iter_bail(args));
            }
            self.set_res(e, Res::BuiltinMethod { b: Builtin::IterOptRes, derefs: d, autoref });
            self.check_args(&[], args, name_span);
            return Some(ret);
        }
        if !it::is_iter(&self.tys, base) && !self.param_iter(base) {
            // A std source collection iterator struct (its one field
            // holds a builtin iterator): its `Iterator` methods type
            // as the inner type's, and lowering unwraps field 0
            // (collect.rs `collection_iter_inner`). Other method
            // names fall through to normal resolution.
            if let Some(inner) = self.p.collection_iter_inner(&mut self.tys, base) {
                let n = self.name(name);
                if self.iter_method_known(n)
                    || matches!(n, "next" | "nth" | "any" | "all" | "find" | "find_map" | "position")
                    || DECLARED_ONLY.contains(&n)
                {
                    let n = n.to_string();
                    return Some(self.iter_call(e, recv, inner, base, derefs, &n, name_span, args, turbofish, expected));
                }
            }
            // A user iterator type by value: wrapped once as `ByNext`,
            // which every adapter and consumer takes (W-STD-B).
            let n = self.name(name).to_string();
            // The `&mut self` consumers and `size_hint` borrow the
            // iterator, which stays (and drops) where Rust has it: the
            // call takes it by reference, also through `&I` or `&mut I`,
            // and the expansion wraps the reference (lower/iter.rs
            // `user_iter`). The rest move it into the wrapper.
            let borrowing = matches!(n.as_str(), "nth" | "any" | "all" | "find" | "find_map" | "try_fold" | "position" | "size_hint" | "len");
            // `by_ref(&mut self) -> &mut Self`, and a by-value method on
            // a `&mut I` receiver (core's `impl Iterator for &mut I`): the
            // wrapper holds the reference and steps `*r` in place.
            let by_ref = n == "by_ref";
            let rt = self.out.expr_ty[self.ei(recv)];
            let rt = self.deep(rt);
            let by_mut = derefs == 1 && !borrowing && matches!(self.kind(rt), TyKind::Ref(_, Mut::Mut, inner) if inner == base);
            if (derefs > 0 && !borrowing && !by_ref && !by_mut) || n == "next" || !(ITER_METHODS.contains(&n.as_str()) || by_ref) {
                return None;
            }
            let item = self.by_next_item(base, name_span)?;
            if item == ERROR {
                for &a in args {
                    self.expr(a, None);
                }
                return Some(ERROR);
            }
            if by_ref {
                if !turbofish.is_empty() {
                    self.errs.push(not_yet("L0721", name_span, "turbofish on this iterator method"));
                    return Some(self.iter_bail(args));
                }
                self.set_res(e, Res::BuiltinMethod { b: Builtin::IterByRef, derefs, autoref: Autoref::RefMut });
                self.check_args(&[], args, name_span);
                return Some(self.tys.mk_ref(true, base));
            }
            let held = if by_mut { rt } else { base };
            let wrapped = it::mk_iter(&mut self.tys, IterKind::ByNext, &[held, item]);
            if borrowing {
                return Some(self.iter_call(e, recv, wrapped, wrapped, derefs, &n, name_span, args, turbofish, expected));
            }
            let i = self.ei(recv);
            self.out.coerce[i] = Coerce::ByNext(wrapped);
            return Some(self.iter_call(e, recv, wrapped, wrapped, 0, &n, name_span, args, turbofish, expected));
        }
        let n = self.name(name).to_string();
        // `Chars` keeps its own `next` and `count` builtins (`count`
        // through `&mut Chars` counts in place, below).
        if self.tys.is_chars_ty(base) && (n == "next" || (n == "count" && derefs == 0)) {
            return None;
        }
        // A range's derived `Hash::hash` is no iterator method: the
        // hasher hook takes it (check/hashmap.rs).
        if n == "hash" && it::iter_parts(&self.tys, base).is_some_and(|(k, _)| k.is_range()) {
            return None;
        }
        // A name of no iterator trait and no iterator view method (a user
        // trait's `tag`, from `impl<T: Clone> Tag for T`): normal resolution.
        if !STD_ITER_TRAIT_METHODS.contains(&n.as_str()) && !ITER_VIEW_METHODS.contains(&n.as_str()) {
            return None;
        }
        Some(self.iter_call(e, recv, base, base, derefs, &n, name_span, args, turbofish, expected))
    }

    /// `Vec::drain` and `String::drain`: an inherent method whose one
    /// argument is any `R: RangeBounds<usize>`. The argument form is
    /// recorded for lowering (check/mod.rs `DrainRange`).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn drain_method(
        &mut self,
        e: ExprId,
        recv: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
    ) -> Option<Ty> {
        use Builtin as B;
        if self.name(name) != "drain" {
            return None;
        }
        let elem = self.tys.vec_elem(base);
        let is_string = self.tys.is_string_ty(base);
        if elem.is_none() && !is_string {
            return None;
        }
        if !turbofish.is_empty() {
            self.errs.push(not_yet("L0721", name_span, "turbofish on `drain`"));
            return Some(self.iter_bail(args));
        }
        if args.len() != 1 {
            self.err("L0105", name_span, "`drain` takes 1 argument", "Pass the range to remove, such as `1..3`.");
            return Some(self.iter_bail(args));
        }
        let arg = args[0];
        let Some(form) = self.drain_range_form(e, arg, name_span) else { return Some(ERROR) };
        self.check_packed_borrow(recv, name_span);
        self.out.drain_ranges.push((e, form));
        let (b, ret) = match elem {
            Some(el) => (B::VecDrain, it::mk_iter(&mut self.tys, IterKind::VecDrain, &[el])),
            None => (B::StringDrain, it::mk_iter(&mut self.tys, IterKind::StringDrain, &[])),
        };
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::RefMut });
        Some(ret)
    }

    /// The form of one `drain` argument, after checking it (see
    /// `drain_method`). None when an error already reported.
    pub(super) fn drain_range_form(&mut self, e: ExprId, arg: ExprId, span: Span) -> Option<DrainRange> {
        use DrainRange as DR;
        let p = self.p;
        if let Expr::Range { lo, hi, .. } = p.ast.expr(arg) {
            for b in [lo, hi].into_iter().flatten() {
                self.expect_expr(*b, USIZE);
            }
            self.set(arg, UNIT);
            return Some(DR::Syntactic);
        }
        let t = self.expr(arg, None);
        let t = self.deep(t);
        if let Some(form) = self.drain_form_of(t, span) {
            return Some(DR::Value(form));
        }
        // A generic parameter (or projection) whose env proves the
        // bound, or a user type's written `RangeBounds` impl.
        let tr = self.range_bounds_ref(t);
        match self.select_obligation_at(tr, span, 0) {
            Selection::Impl { id, args: impl_args, .. } => {
                let rec = p.gx.index.get(id).clone();
                if rec.synthetic != crate::traits::index::Synthetic::None {
                    // A synthetic impl this code did not name above
                    // (the `&R` blanket over a written impl, say).
                    return self.drain_reject(t, span);
                }
                // Impl validation for a lang trait does not check
                // that every method is written (generics.rs needs a
                // template), so check the two methods this call
                // runs: present, and returning `Bound<&usize>`.
                let written = |this: &mut Self, site: ExprId, name: &str| -> Option<LocalDefId> {
                    let (f, _, ret) = this.method_instance_for_impl(site, id, tr, impl_args.clone(), name, span)?;
                    let in_impl = rec.items.iter().any(|&d| this.p.local_def(d.hash) == Some(f));
                    let ok = in_impl
                        && generics::bound_elem(&this.tys, this.resolve(ret)).is_some_and(|r| {
                            matches!(this.kind(this.resolve(r)), TyKind::Ref(_, _, i) if this.unify(i, USIZE))
                        });
                    ok.then_some(f)
                };
                let (Some(start), Some(end)) = (written(self, e, "start_bound"), written(self, arg, "end_bound")) else {
                    return self.drain_reject(t, span);
                };
                Some(DR::Trait { start, end })
            }
            Selection::ParamEvidence(_) => Some(DR::Param),
            _ => self.drain_reject(t, span),
        }
    }

    /// The trait ref `t: RangeBounds<usize>`.
    fn range_bounds_ref(&mut self, t: Ty) -> TraitRef {
        let args = self.tys.intern_args(&[GenericArg::Ty(t), GenericArg::Ty(USIZE)]);
        TraitRef { def: self.p.gx.lang.range_bounds, args, bindings: EMPTY_BINDINGS }
    }

    /// Reports a drain argument lust cannot read as bounds.
    fn drain_reject(&mut self, t: Ty, span: Span) -> Option<DrainRange> {
        if !matches!(self.kind(self.resolve(t)), TyKind::Error) {
            let tn = self.tn(t);
            self.err(
                "L0612",
                span,
                format!("`{tn}` does not implement `RangeBounds<usize>` as `drain` needs"),
                "Pass a range such as `1..3`, `1..`, `..`, or a `(Bound, Bound)` pair.",
            );
        }
        None
    }

    /// The builtin range form of a value type, when it is one of the
    /// six range kinds over `usize` (the index type unifies with
    /// `usize`), a `(Bound<usize>, Bound<usize>)` pair, or a shared
    /// reference to one of those.
    pub(super) fn drain_form_of(&mut self, t: Ty, span: Span) -> Option<crate::check::DrainForm> {
        use crate::check::DrainForm as F;
        use IterKind as K;
        let t = self.resolve(t);
        if let Some((k, a)) = it::iter_parts(&self.tys, t) {
            let form = match k {
                K::Range => F::Range,
                K::RangeInclusive => F::RangeInclusive,
                K::RangeFrom => F::RangeFrom,
                K::RangeTo => F::RangeTo,
                K::RangeToInclusive => F::RangeToInclusive,
                K::RangeFull => return Some(F::RangeFull),
                _ => return None,
            };
            return self.unify(a[0], USIZE).then_some(form);
        }
        if let TyKind::Tuple(ts) = self.kind(t) {
            let es: Vec<Ty> = self.tys.tys_of(ts).to_vec();
            if es.len() == 2
                && let (Some(a), Some(b)) = (generics::bound_elem(&self.tys, es[0]), generics::bound_elem(&self.tys, es[1]))
            {
                // The two pair impls in core: `(Bound<T>, Bound<T>)`
                // and `(Bound<&T>, Bound<&T>)`. When both payloads are
                // unbound plain variables (two `Unbounded`s with no
                // annotation), the impl is ambiguous, as rustc finds.
                let (ra, rb) = (self.resolve(a), self.resolve(b));
                let plain = |k: TyKind| matches!(k, TyKind::Infer(v) if !v.is_int() && !v.is_float());
                if plain(self.kind(ra)) && plain(self.kind(rb)) {
                    self.err(
                        "L0610",
                        span,
                        "type annotations needed: the bound type must be known here",
                        "Annotate the bounds, such as `Bound::Unbounded::<usize>`.",
                    );
                    return Some(F::Bounds);
                }
                let rus = self.tys.mk_ref(false, USIZE);
                let snap = self.snapshot_vars();
                if self.unify(a, rus) && self.unify(b, rus) {
                    return Some(F::BoundsRef);
                }
                self.rollback_vars(snap);
                if self.unify(a, USIZE) && self.unify(b, USIZE) {
                    return Some(F::Bounds);
                }
            }
        }
        None
    }

    /// The inherent methods of `core::ops::Bound<T>`: `as_ref`,
    /// `map` and `cloned` (`copied` is unstable in 1.99).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn bound_adt_method(
        &mut self,
        e: ExprId,
        recv: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
    ) -> Option<Ty> {
        use Builtin as B;
        let elem = generics::bound_elem(&self.tys, self.resolve(base))?;
        let n = self.name(name);
        let (b, autoref, params, ret): (Builtin, Autoref, Vec<Ty>, Ty) = match n {
            "as_ref" => {
                let r = self.tys.mk_ref(false, elem);
                (B::BoundAsRef, Autoref::Ref, vec![], generics::bound_ty(&mut self.tys, r))
            }
            "map" => {
                let out = self.new_var(false);
                let f = self.fn_mut_param(&[elem], out, name_span);
                (B::BoundMap, Autoref::None, vec![f], generics::bound_ty(&mut self.tys, out))
            }
            "cloned" => {
                let TyKind::Ref(_, Mut::Shared, inner) = self.kind(self.resolve(elem)) else { return None };
                self.deferred_clone.push((inner, name_span));
                (B::BoundCloned, Autoref::None, vec![], generics::bound_ty(&mut self.tys, inner))
            }
            _ => return None,
        };
        if autoref != Autoref::None {
            self.check_packed_borrow(recv, name_span);
        }
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref });
        self.check_args(&params, args, name_span);
        Some(ret)
    }

    /// `WaitTimeoutResult::timed_out`: its flag (TRIAGE T5).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn wait_timeout_result_method(
        &mut self,
        e: ExprId,
        recv: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
    ) -> Option<Ty> {
        let _ = recv;
        if self.tys.adt_parts(self.resolve(base)).map(|(d, _)| d) != Some(generics::wait_timeout_result_def()) {
            return None;
        }
        if self.name(name) != "timed_out" {
            return None;
        }
        self.set_res(e, Res::BuiltinMethod { b: Builtin::WaitTimedOut, derefs, autoref: Autoref::Ref });
        self.check_args(&[], args, name_span);
        Some(BOOL)
    }

    /// The inherent methods of `core::mem::MaybeUninit<T>` lust
    /// takes (TRIAGE T5): `write`, `as_ptr`, and `as_mut_ptr` (the
    /// safe surface the fork uses as a buffer). The unsafe
    /// readers stay out: lust has no unsafe.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn maybe_uninit_method(
        &mut self,
        e: ExprId,
        recv: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
    ) -> Option<Ty> {
        let _ = recv;
        let inner = generics::maybe_uninit_elem(&self.tys, self.resolve(base))?;
        let (b, autoref, params, ret): (Builtin, Autoref, Vec<Ty>, Ty) = match self.name(name) {
            "write" => (Builtin::MaybeUninitWrite, Autoref::RefMut, vec![inner], self.tys.mk_ref(true, inner)),
            "as_ptr" => (Builtin::MaybeUninitAsPtr, Autoref::Ref, vec![], self.tys.intern(TyKind::RawPtr(Mut::Shared, inner))),
            "as_mut_ptr" => (Builtin::MaybeUninitAsMutPtr, Autoref::RefMut, vec![], self.tys.intern(TyKind::RawPtr(Mut::Mut, inner))),
            _ => return None,
        };
        if autoref != Autoref::None {
            self.check_packed_borrow(recv, name_span);
        }
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref });
        self.check_args(&params, args, name_span);
        Some(ret)
    }

    /// The inherent methods of `core::num::NonZero<T>` lust takes:
    /// `get` (the wrapped integer). `new` is an associated fn
    /// (check's `resolve_assoc`); the rest of std's arithmetic
    /// surface on NonZero is not modeled yet.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn nonzero_adt_method(
        &mut self,
        e: ExprId,
        recv: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
    ) -> Option<Ty> {
        let _ = recv;
        let inner = generics::nonzero_elem(&self.tys, self.resolve(base))?;
        if self.name(name) != "get" {
            return None;
        }
        self.set_res(e, Res::BuiltinMethod { b: Builtin::NonZeroGet, derefs, autoref: Autoref::None });
        self.check_args(&[], args, name_span);
        Some(inner)
    }

    /// `RangeBounds::start_bound` and `end_bound` on a builtin range
    /// kind, a `(Bound, Bound)` pair, or a shared reference to one
    /// (for the reference, `derefs` reaches the value). user impls
    /// and generic receivers resolve through the trait machinery
    /// before this hook; instances of a template call land here
    /// through the builtin dispatch.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn bound_method(
        &mut self,
        e: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
        expected: Option<Ty>,
    ) -> Option<Ty> {
        use Builtin as B;
        let start = match self.name(name) {
            "start_bound" => true,
            "end_bound" => false,
            _ => return None,
        };
        // The receiver must at least be a builtin range kind or pair
        // (a user type of the same method names is not this hook),
        // and the trait must be in scope, as for any trait method.
        if !self.is_range_value_ty(base) || !super::method::trait_in_scope_pub(self, self.p.gx.lang.range_bounds) {
            return None;
        }
        // The index type: the range's element or the pair's Bound
        // payload; for `RangeFull` (any `T`), the replayed trait
        // ref's input or the expected type names it.
        let idx = self.range_index_ty(base).or_else(|| {
            let tr = self.replayed_trait_ref(e, self.p.gx.lang.range_bounds)?;
            self.tys.args(tr.args).get(1).and_then(|g| g.as_ty())
        }).or_else(|| {
            let want = expected?;
            let r = generics::bound_elem(&self.tys, self.resolve(want))?;
            match self.kind(self.resolve(r)) {
                TyKind::Ref(_, _, i) => Some(i),
                _ => None,
            }
        })?;
        if !turbofish.is_empty() {
            self.errs.push(not_yet("L0721", name_span, "turbofish on this method"));
            return Some(self.iter_bail(args));
        }
        self.check_args(&[], args, name_span);
        let r = self.tys.mk_ref(false, idx);
        let ret = generics::bound_ty(&mut self.tys, r);
        let b = if start { B::RangeStartBound } else { B::RangeEndBound };
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::Ref });
        Some(ret)
    }

    /// A public field of a builtin range kind: `start` and `end`
    /// on `Range`, `start` on `RangeFrom`, `end` on `RangeTo` and
    /// `RangeToInclusive`, as (field index, type).
    /// `RangeInclusive`'s fields are private in std.
    pub(super) fn range_field(&mut self, t: Ty, name: Sym) -> Option<(u32, Ty)> {
        use IterKind as K;
        let (k, a) = it::iter_parts(&self.tys, self.resolve(t))?;
        let idx = a.first().copied()?;
        match (k, self.name(name)) {
            (K::Range, "start") | (K::RangeFrom, "start") => Some((0, idx)),
            (K::Range, "end") => Some((1, idx)),
            (K::RangeTo, "end") | (K::RangeToInclusive, "end") => Some((0, idx)),
            _ => None,
        }
    }

    /// `contains` and `is_empty` on the builtin range kinds and
    /// `(Bound, Bound)` pairs, and `RangeInclusive::start`/`end`.
    /// `is_empty` is inherent on `Range` and `RangeInclusive` only
    /// (the trait's is unstable in 1.99); `contains` on `RangeFull`
    /// and pairs needs the trait in scope, the kinds' own are
    /// inherent.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn range_query_method(
        &mut self,
        e: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
    ) -> Option<Ty> {
        use Builtin as B;
        use IterKind as K;
        let n = self.name(name).to_string();
        let n = n.as_str();
        if !matches!(n, "contains" | "is_empty" | "start" | "end") {
            return None;
        }
        if !self.is_range_value_ty(base) {
            return None;
        }
        if !turbofish.is_empty() {
            self.errs.push(not_yet("L0721", name_span, "turbofish on this method"));
            return Some(self.iter_bail(args));
        }
        let kind = it::iter_parts(&self.tys, self.resolve(base)).map(|(k, _)| k);
        let (b, params, ret): (Builtin, Vec<Ty>, Ty) = match n {
            "start" | "end" => {
                if kind != Some(K::RangeInclusive) {
                    return None;
                }
                let idx = self.range_index_ty(base)?;
                let r = self.tys.mk_ref(false, idx);
                (if n == "start" { B::RangeInclStart } else { B::RangeInclEnd }, vec![], r)
            }
            "is_empty" => {
                if !matches!(kind, Some(K::Range | K::RangeInclusive)) {
                    return None;
                }
                (B::RangeIsEmpty, vec![], BOOL)
            }
            _ => {
                // contains: `RangeFull` and pairs answer through the
                // trait, so the trait must be in scope for those.
                let inherent = matches!(kind, Some(K::Range | K::RangeInclusive | K::RangeFrom | K::RangeTo | K::RangeToInclusive));
                if !inherent && !super::method::trait_in_scope_pub(self, self.p.gx.lang.range_bounds) {
                    return None;
                }
                let idx = match self.range_index_ty(base) {
                    Some(i) => i,
                    None => {
                        // RangeFull: the argument names the index type.
                        let Some(&a) = args.first() else {
                            self.err("L0105", name_span, "`contains` takes 1 argument", "Pass the item to look for.");
                            return Some(self.iter_bail(args));
                        };
                        let at = self.expr(a, None);
                        match self.kind(self.resolve(at)) {
                            TyKind::Ref(_, _, i) => i,
                            _ => return Some(ERROR),
                        }
                    }
                };
                // The expansion compares with scalar order.
                match self.kind(self.resolve(idx)) {
                    TyKind::Int(_) | TyKind::Char | TyKind::Param(_) | TyKind::Projection(..) | TyKind::Error => {}
                    k if k.is_int_infer() => {}
                    _ => {
                        self.errs.push(not_yet("L0640", name_span, "`contains` over this index type"));
                        return Some(self.iter_bail(args));
                    }
                }
                let r = self.tys.mk_ref(false, idx);
                // The argument of the RangeFull form checked above.
                if self.range_index_ty(base).is_none() {
                    self.set_res(e, Res::BuiltinMethod { b: B::RangeContains, derefs, autoref: Autoref::Ref });
                    return Some(BOOL);
                }
                (B::RangeContains, vec![r], BOOL)
            }
        };
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::Ref });
        self.check_args(&params, args, name_span);
        Some(ret)
    }

    /// True for a builtin range kind or a `(Bound, Bound)` pair
    /// (behind shared references, which the receiver autoderef
    /// already peeled for `base`, but check anyway).
    fn is_range_value_ty(&mut self, t: Ty) -> bool {
        let t = self.resolve(t);
        if let TyKind::Ref(_, m, inner) = self.kind(t)
            && !m.is_mut()
        {
            return self.is_range_value_ty(inner);
        }
        if let Some((k, _)) = it::iter_parts(&self.tys, t) {
            return k.is_range();
        }
        if let TyKind::Tuple(ts) = self.kind(t) {
            let es: Vec<Ty> = self.tys.tys_of(ts).to_vec();
            return es.len() == 2 && es.iter().all(|&e| generics::bound_elem(&self.tys, e).is_some());
        }
        false
    }

    /// The index type of a builtin `RangeBounds` value type (a range
    /// kind's element, or a `(Bound<T>, Bound<T>)` pair's `T`).
    fn range_index_ty(&mut self, t: Ty) -> Option<Ty> {
        use IterKind as K;
        let t = self.resolve(t);
        if let TyKind::Ref(_, m, inner) = self.kind(t)
            && !m.is_mut()
        {
            return self.range_index_ty(inner);
        }
        if let Some((k, a)) = it::iter_parts(&self.tys, t)
            && k.is_range()
        {
            // `RangeFull` alone has no index on the value (it works
            // for every `T`): it has none to name here.
            return a.first().copied();
        }
        if let TyKind::Tuple(ts) = self.kind(t) {
            let es: Vec<Ty> = self.tys.tys_of(ts).to_vec();
            if es.len() == 2
                && let (Some(a), Some(b)) = (generics::bound_elem(&self.tys, es[0]), generics::bound_elem(&self.tys, es[1]))
                && self.unify(a, b)
            {
                // The `(Bound<&T>, Bound<&T>)` impl answers for `T`.
                return Some(match self.kind(self.resolve(a)) {
                    TyKind::Ref(_, _, i) => i,
                    _ => a,
                });
            }
        }
        None
    }

    /// The sort and binary-search family on a Vec, slice, or array
    /// (runtime/sort.h ports Rust 1.99's algorithms). Comparators are
    /// closures; `binary_search` and its `_by_key` key need an `Ord`
    /// type lust compares (collect.rs `has_total_ord`), and the sorts
    /// also need a `PartialOrd::lt` lust can call (`has_partial_cmp`):
    /// std's sort compares with `lt`, not `Ord::cmp`.
    #[allow(clippy::too_many_arguments)]
    fn seq_sort(&mut self, e: ExprId, recv: ExprId, el: Ty, derefs: u8, n: &str, span: Span, args: &[ExprId]) -> Ty {
        use Builtin as B;
        let ord = generics::ordering_ty(&mut self.tys);
        let r_el = self.tys.mk_ref(false, el);
        let res = self.tys.mk_result(USIZE, USIZE);
        let (b, autoref, params, ret) = match n {
            "sort" | "sort_unstable" => {
                self.deferred_sort_ord.push((el, span));
                (if n == "sort" { B::SliceSort } else { B::SliceSortUnstable }, Autoref::RefMut, vec![], UNIT)
            }
            "sort_by" | "sort_unstable_by" => {
                // `F: FnMut(&T, &T) -> Ordering`: both references are
                // higher-ranked, so an element reference cannot escape.
                let f = self.fn_mut_param_ref(&[r_el, r_el], ord, span);
                (if n == "sort_by" { B::SliceSortBy } else { B::SliceSortUnstableBy }, Autoref::RefMut, vec![f], UNIT)
            }
            "sort_by_key" | "sort_unstable_by_key" => {
                // Rust's bound is `F: FnMut(&T) -> K` with a
                // higher-ranked `&T`, so the key cannot borrow from
                // the element (as in `min_by_key`/`max_by_key` below).
                // `binary_search_by_key` is the opposite: its `&'a T`
                // is tied to the slice, so it keeps `fn_mut_param`.
                let key = self.new_var(false);
                let f = self.fn_mut_param_ref(&[r_el], key, span);
                self.deferred_sort_ord.push((key, span));
                (if n == "sort_by_key" { B::SliceSortByKey } else { B::SliceSortUnstableByKey }, Autoref::RefMut, vec![f], UNIT)
            }
            "binary_search" => {
                self.deferred_ord.push((el, span));
                (B::SliceBinarySearch, Autoref::Ref, vec![r_el], res)
            }
            "binary_search_by" => {
                let f = self.fn_mut_param(&[r_el], ord, span);
                (B::SliceBinarySearchBy, Autoref::Ref, vec![f], res)
            }
            _ => {
                let key = self.new_var(false);
                let rk = self.tys.mk_ref(false, key);
                let f = self.fn_mut_param(&[r_el], key, span);
                self.deferred_ord.push((key, span));
                (B::SliceBinarySearchByKey, Autoref::Ref, vec![rk, f], res)
            }
        };
        if autoref != Autoref::None {
            self.check_packed_borrow(recv, span);
        }
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref });
        self.check_args(&params, args, span);
        if !matches!(b, B::SliceSort | B::SliceSortUnstable | B::SliceBinarySearch) && self.adapter_callable_gap(args, &params, false) {
            return ERROR;
        }
        ret
    }

    /// A source method on a Vec, slice, or array receiver.
    #[allow(clippy::too_many_arguments)]
    fn iter_source(&mut self, e: ExprId, rt: Ty, base: Ty, el: Ty, derefs: u8, n: &str, span: Span, args: &[ExprId]) -> Option<Ty> {
        use Builtin as B;
        let (b, autoref, derefs, ret) = match n {
            "iter" => (B::IterIter, Autoref::Ref, derefs, it::mk_iter(&mut self.tys, IterKind::SliceIter, &[el])),
            "iter_mut" => (B::IterIterMut, Autoref::RefMut, derefs, it::mk_iter(&mut self.tys, IterKind::SliceIterMut, &[el])),
            "into_iter" => {
                // Rust's probe: the first autoderef step whose type has
                // an `IntoIterator` impl, by value: `&X` or `&mut X`
                // (a slice iterator), or `Vec<T>` itself.
                let mut t = self.resolve(rt);
                let mut found = None;
                for j in 0..=derefs {
                    match self.kind(t) {
                        TyKind::Ref(_, m, inner) => {
                            let inner = self.resolve(inner);
                            let seq = matches!(self.kind(inner), TyKind::Slice(_) | TyKind::Array(..)) || self.tys.vec_elem(inner).is_some();
                            if seq {
                                let (b, k) = if m.is_mut() { (B::IterIterMut, IterKind::SliceIterMut) } else { (B::IterIter, IterKind::SliceIter) };
                                found = Some((b, j, it::mk_iter(&mut self.tys, k, &[el])));
                                break;
                            }
                            t = inner;
                        }
                        _ if self.tys.vec_elem(t).is_some() => {
                            found = Some((B::IterVecIntoIter, j, it::mk_iter(&mut self.tys, IterKind::VecIntoIter, &[el])));
                            break;
                        }
                        // An array by value (`array::IntoIter`,
                        // fx-array-iter): every element type.
                        TyKind::Array(..) if j == 0 => {
                            found = Some((B::IterArrayIntoIter, 0, it::mk_iter(&mut self.tys, IterKind::ArrayIntoIter, &[el, t])));
                            break;
                        }
                        _ => break,
                    }
                }
                let Some((b, j, ret)) = found else {
                    // An array by value (`array::IntoIter`) or a smart
                    // pointer step: valid Rust lust does not have yet.
                    self.err("L0640", span, "`into_iter` on this receiver is not implemented yet", "Iterate with `.iter()`, or over a `Vec`.");
                    for &a in args {
                        self.expr(a, None);
                    }
                    return Some(ERROR);
                };
                (b, Autoref::None, j, ret)
            }
            _ => return None,
        };
        let _ = base;
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref });
        self.check_args(&[], args, span);
        Some(ret)
    }

    /// One method on a builtin iterator receiver. `actual_base` is
    /// the receiver's own type when it differs from the builtin
    /// iterator `base` it wraps (a collection iterator struct): the
    /// std collection `collect` target defers to a trait method
    /// whose iterator argument keeps that type.
    #[allow(clippy::too_many_arguments)]
    fn iter_call(
        &mut self,
        e: ExprId,
        recv: ExprId,
        base: Ty,
        actual_base: Ty,
        derefs: u8,
        n: &str,
        span: Span,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
        expected: Option<Ty>,
    ) -> Ty {
        use Builtin as B;
        use IterKind as K;
        let item = self.iter_item(base);
        let by_value = !matches!(
            n,
            "next" | "next_back" | "nth" | "any" | "all" | "find" | "find_map" | "try_fold" | "position" | "rposition" | "peek" | "next_if" | "next_if_eq" | "size_hint" | "len"
                | "as_slice" | "as_mut_slice" | "as_str" | "remainder" | "by_ref"
        );
        // A by-value method through `&mut I` uses core's `impl Iterator
        // for &mut I`: the reference moves into a `ByNext` wrapper whose
        // `next` steps `*r` in place (lower/iter.rs). Other forms through
        // a reference (`&I`, `&&mut I`) stay out.
        // A std collection iterator struct behind `&mut` (`&mut
        // vec_deque::IntoIter<T>`, typed as its inner iterator `base`)
        // steps through the struct's own `next` the same way. BTree
        // `range` iterators stay out: lowering would hint them as their
        // exact inner iterator, where std gives `(0, None)`.
        let wrapper = actual_base != base
            && self.p.collection_iter_inner(&mut self.tys, actual_base) == Some(base)
            && !self.p.is_btree_range_iter(&mut self.tys, actual_base);
        if by_value && derefs == 1 && (actual_base == base || wrapper) && self.iter_method_known(n) {
            let rt = self.out.expr_ty[self.ei(recv)];
            let rt = self.deep(rt);
            if matches!(self.kind(rt), TyKind::Ref(_, Mut::Mut, inner) if inner == actual_base) {
                let wrapped = it::mk_iter(&mut self.tys, IterKind::ByNext, &[rt, item]);
                let i = self.ei(recv);
                self.out.coerce[i] = Coerce::ByNext(wrapped);
                return self.iter_call(e, recv, wrapped, wrapped, 0, n, span, args, turbofish, expected);
            }
        }
        if by_value && derefs > 0 && self.iter_method_known(n) {
            self.err(
                "L0640",
                span,
                format!("`{n}` on an iterator behind a reference is not implemented yet"),
                "Call it on the iterator value itself.",
            );
            return self.iter_bail(args);
        }
        if !turbofish.is_empty() && !matches!(n, "collect" | "sum" | "product") {
            self.errs.push(not_yet("L0721", span, "turbofish on this iterator method"));
            return self.iter_bail(args);
        }
        let mk = |this: &mut Self, k: IterKind, a: &[Ty]| it::mk_iter(&mut this.tys, k, a);
        let r = |this: &mut Self, t: Ty| this.tys.mk_ref(false, t);
        let (b, params, ret): (Builtin, Vec<Ty>, Ty) = match n {
            "next" => {
                let o = self.tys.mk_option(item);
                (B::IterNext, vec![], o)
            }
            // `size_hint` (W-STD-B): the hint the expansions compute.
            "size_hint" => {
                let ou = self.tys.mk_option(USIZE);
                let ts = self.tys.intern_tys(&[USIZE, ou]);
                let t = self.tys.intern(TyKind::Tuple(ts));
                (B::IterSizeHint, vec![], t)
            }
            // DoubleEndedIterator::next_back (W-STD-B).
            "next_back" => {
                if !self.double_ended(base, span) {
                    let tn = self.tn(base);
                    self.err("L0612", span, format!("`{tn}` is not a DoubleEndedIterator here"), "Iterate forward with `next`.");
                    return self.iter_bail(args);
                }
                let o = self.tys.mk_option(item);
                (B::IterNextBack, vec![], o)
            }
            "len" => {
                if !self.exact_size(base, span) {
                    return self.iter_unknown(n, span, base, args);
                }
                (B::IterLen, vec![], USIZE)
            }
            "as_slice" | "as_mut_slice" => {
                let m = n == "as_mut_slice";
                match it::iter_parts(&self.tys, base) {
                    Some((K::VecDrain, a)) if !m => {
                        let sl = self.tys.intern(TyKind::Slice(a[0]));
                        let r = self.tys.mk_ref(false, sl);
                        (B::VecDrainAsSlice, vec![], r)
                    }
                    // `array::IntoIter` (fx-array-iter): the slice of
                    // the elements not yet yielded, as core gives it.
                    Some((K::ArrayIntoIter, a)) => {
                        let sl = self.tys.intern(TyKind::Slice(a[0]));
                        let r = self.tys.mk_ref(m, sl);
                        (B::ArrayIterAsSlice, vec![], r)
                    }
                    _ => return self.iter_unknown(n, span, base, args),
                }
            }
            "as_str" => {
                let Some((K::StringDrain, _)) = it::iter_parts(&self.tys, base) else { return self.iter_unknown(n, span, base, args) };
                (B::StrDrainAsStr, vec![], REF_STR)
            }
            "remainder" | "into_remainder" => {
                // `ChunksExact::remainder` and
                // `ChunksExactMut::into_remainder` (tf-slice): the
                // stored remainder slice.
                let (el, m) = match (n, it::iter_parts(&self.tys, base)) {
                    ("remainder", Some((K::ChunksExact, a))) => (a[0], false),
                    ("into_remainder", Some((K::ChunksExactMut, a))) => (a[0], true),
                    _ => return self.iter_unknown(n, span, base, args),
                };
                let sl = self.tys.intern(TyKind::Slice(el));
                let r = self.tys.mk_ref(m, sl);
                (B::IterRemainder, vec![], r)
            }
            "map" => {
                let out = self.new_var(false);
                let f = self.fn_mut_param(&[item], out, span);
                let t = mk(self, K::Map, &[base, f, out]);
                (B::IterMap, vec![f], t)
            }
            "filter" => {
                let ri = r(self, item);
                let p = self.fn_mut_param_ref(&[ri], BOOL, span);
                let t = mk(self, K::Filter, &[base, p]);
                (B::IterFilter, vec![p], t)
            }
            "filter_map" => {
                let out = self.new_var(false);
                let o = self.tys.mk_option(out);
                let f = self.fn_mut_param(&[item], o, span);
                let t = mk(self, K::FilterMap, &[base, f, out]);
                (B::IterFilterMap, vec![f], t)
            }
            "map_while" => {
                let out = self.new_var(false);
                let o = self.tys.mk_option(out);
                let f = self.fn_mut_param(&[item], o, span);
                let t = mk(self, K::MapWhile, &[base, f, out]);
                (B::IterMapWhile, vec![f], t)
            }
            "enumerate" => (B::IterEnumerate, vec![], mk(self, K::Enumerate, &[base])),
            // `flatten` and `flat_map` (W-STD-B): the inner items' (or the
            // closure output's) IntoIter is the second iterator.
            "flatten" => {
                if let Some((g, out)) = self.written_into_iter_fn(item) {
                    let Some(u) = self.flat_inner(out, span) else { return self.iter_bail(args) };
                    let m = mk(self, K::Map, &[base, g, out]);
                    (B::IterFlattenInto, vec![], mk(self, K::Flatten, &[m, u]))
                } else {
                    let Some(u) = self.flat_inner(item, span) else { return self.iter_bail(args) };
                    (B::IterFlatten, vec![], mk(self, K::Flatten, &[base, u]))
                }
            }
            // `flat_map(f)` is `Flatten<Map<I, F>>`, what std's FlatMap runs.
            "flat_map" => {
                let out = self.new_var(false);
                let f = self.fn_mut_param(&[item], out, span);
                self.set_res(e, Res::BuiltinMethod { b: B::IterFlatMap, derefs, autoref: Autoref::None });
                self.check_args(&[f], args, span);
                self.run_obligations(false);
                // The expansion is `Flatten<Map<I, F>>`, whose map calls a
                // closure, fn item or fn pointer (lower/iter.rs `call_fn`).
                let fd = self.peel_callable_refs(f);
                if !matches!(self.kind(fd), TyKind::Closure(..) | TyKind::FnDef(..) | TyKind::FnPtr(_) | TyKind::Error) {
                    self.err("L0640", span, "`flat_map` with a fn item or fn pointer is not implemented yet", "Pass a closure: `|x| f(x)`.");
                    return ERROR;
                }
                let out = self.deep(out);
                let Some(u) = self.flat_inner(out, span) else { return ERROR };
                let m = mk(self, K::Map, &[base, f, out]);
                return mk(self, K::Flatten, &[m, u]);
            }
            "rev" => {
                if !self.double_ended(base, span) {
                    let tn = self.tn(base);
                    self.err("L0612", span, format!("`{tn}` is not a double-ended iterator"), "Reverse a slice, Vec, or range iterator instead.");
                    return self.iter_bail(args);
                }
                (B::IterRev, vec![], mk(self, K::Rev, &[base]))
            }
            "take" => (B::IterTake, vec![USIZE], mk(self, K::Take, &[base])),
            "skip" => (B::IterSkip, vec![USIZE], mk(self, K::Skip, &[base])),
            "step_by" => (B::IterStepBy, vec![USIZE], mk(self, K::StepBy, &[base])),
            "chain" | "zip" => {
                let Some(&a) = args.first() else {
                    self.err("L0105", span, format!("`{n}` takes 1 argument"), "Pass the other iterator.");
                    return self.iter_bail(args);
                };
                let at = self.expr(a, None);
                let at = self.deep(at);
                let Some(other) = self.into_iter_ty(at) else {
                    let tn = self.tn(at);
                    self.err(
                        "L0612",
                        self.span(a),
                        format!("`{tn}` is not an iterator lust can {n} with yet"),
                        "Pass an iterator, a `&Vec`, a slice, or a `Vec`.",
                    );
                    return self.iter_bail(&args[1..]);
                };
                if args.len() != 1 {
                    self.err("L0105", span, format!("`{n}` takes 1 argument"), "Pass the other iterator.");
                }
                let oi = self.iter_item(other);
                if n == "chain" && !self.unify(oi, item) {
                    self.mismatch(self.span(a), item, oi);
                }
                self.set_res(e, Res::BuiltinMethod { b: if n == "chain" { B::IterChain } else { B::IterZip }, derefs, autoref: Autoref::None });
                return mk(self, if n == "chain" { K::Chain } else { K::Zip }, &[base, other]);
            }
            // core's `eq`/`ne`: `I: IntoIterator, Self::Item:
            // PartialEq<I::Item>`. lust compares items of one type
            // (the eq glue), known when the call is checked.
            "eq" | "ne" => {
                let [a] = args else {
                    self.err("L0105", span, format!("`{n}` takes 1 argument"), "Pass the other iterator.");
                    return self.iter_bail(args);
                };
                let at = self.expr(*a, None);
                let at = self.deep(at);
                let Some(other) = self.into_iter_ty(at) else {
                    let tn = self.tn(at);
                    self.err("L0612", self.span(*a), format!("`{tn}` is not an iterator lust can compare with yet"), "Pass an iterator, a `&Vec`, a slice, or a `Vec`.");
                    return ERROR;
                };
                if it::iter_parts(&self.tys, other).is_some_and(|(k, _)| k == IterKind::ByNext) && !it::is_iter(&self.tys, at) {
                    let i = self.ei(*a);
                    self.out.coerce[i] = Coerce::ByNext(other);
                }
                let oi = self.iter_item(other);
                let (x, y) = (self.deep(item), self.deep(oi));
                if self.unbound_general(x) || self.unbound_general(y) {
                    self.err("L0610", span, format!("type annotations needed: the item types of this `{n}` must be known here"), "Annotate the element types.");
                    return ERROR;
                }
                if !self.unify(x, y) {
                    let (xn, yn) = (self.tn(x), self.tn(y));
                    self.err("L0640", span, format!("`{n}` between `{xn}` and `{yn}` items is not implemented yet"), "Map the items to one type first.");
                    return ERROR;
                }
                self.deferred_eq.push((x, span));
                self.set_res(e, Res::BuiltinMethod { b: if n == "eq" { B::IterEq } else { B::IterNe }, derefs, autoref: Autoref::None });
                return BOOL;
            }
            "copied" | "cloned" => {
                let inner = match self.kind(self.resolve(item)) {
                    TyKind::Ref(_, _, t) => t,
                    _ => {
                        let tn = self.tn(item);
                        self.err("L0612", span, format!("`{n}` needs an iterator over references, not over `{tn}`"), "Drop the call.");
                        return self.iter_bail(args);
                    }
                };
                if n == "copied" {
                    self.deferred_copy.push((inner, span));
                    (B::IterCopied, vec![], mk(self, K::Copied, &[base, inner]))
                } else {
                    self.deferred_clone.push((inner, span));
                    (B::IterCloned, vec![], mk(self, K::Cloned, &[base, inner]))
                }
            }
            "take_while" | "skip_while" => {
                let ri = r(self, item);
                let p = self.fn_mut_param_ref(&[ri], BOOL, span);
                if n == "take_while" {
                    (B::IterTakeWhile, vec![p], mk(self, K::TakeWhile, &[base, p]))
                } else {
                    (B::IterSkipWhile, vec![p], mk(self, K::SkipWhile, &[base, p]))
                }
            }
            "peekable" => (B::IterPeekable, vec![], mk(self, K::Peekable, &[base])),
            "peek" => {
                let Some((K::Peekable, _)) = it::iter_parts(&self.tys, base) else { return self.iter_unknown(n, span, base, args) };
                let ri = r(self, item);
                let o = self.tys.mk_option(ri);
                (B::IterPeek, vec![], o)
            }
            // `next_if(func: impl FnOnce(&Item) -> bool)` is taken as
            // `FnMut`, a stronger bound: a closure that is only `FnOnce`
            // rejects. `next_if_eq(&T)` with `Item: PartialEq<T>`
            // compares items of one type (`T = Item`).
            "next_if" | "next_if_eq" => {
                let Some((K::Peekable, _)) = it::iter_parts(&self.tys, base) else { return self.iter_unknown(n, span, base, args) };
                let o = self.tys.mk_option(item);
                if n == "next_if" {
                    let ri = r(self, item);
                    let f = self.fn_mut_param_ref(&[ri], BOOL, span);
                    (B::IterNextIf, vec![f], o)
                } else {
                    let [a] = args else {
                        self.err("L0105", span, "`next_if_eq` takes 1 argument", "Pass a reference to the expected item.");
                        return self.iter_bail(args);
                    };
                    let at = self.expr(*a, None);
                    let at = self.deep(at);
                    // `Item: PartialEq<T>` does not fix an open `Item`
                    // in rustc (E0282): no inference from `T` here.
                    let it = self.deep(item);
                    let lit = matches!(self.kind(it), TyKind::Infer(v) if v.is_int() || v.is_float());
                    if self.has_unbound_var(it) && !lit {
                        self.err("L0610", span, "type annotations needed: the item type of this `next_if_eq` must be known here", "Annotate the element type.");
                        return ERROR;
                    }
                    let want = r(self, item);
                    if !self.coerce_expr(*a, at, want) {
                        return ERROR;
                    }
                    self.deferred_eq.push((item, span));
                    self.check_packed_borrow(recv, span);
                    self.set_res(e, Res::BuiltinMethod { b: B::IterNextIfEq, derefs, autoref: Autoref::RefMut });
                    return o;
                }
            }
            "inspect" => {
                let ri = r(self, item);
                let f = self.fn_mut_param_ref(&[ri], UNIT, span);
                (B::IterInspect, vec![f], mk(self, K::Inspect, &[base, f]))
            }
            "count" => (B::IterCount, vec![], USIZE),
            // `by_ref(&mut self) -> &mut Self`: the receiver reference.
            "by_ref" => (B::IterByRef, vec![], self.tys.mk_ref(true, actual_base)),
            "last" => {
                let o = self.tys.mk_option(item);
                (B::IterLast, vec![], o)
            }
            "nth" => {
                let o = self.tys.mk_option(item);
                (B::IterNth, vec![USIZE], o)
            }
            "any" | "all" => {
                let f = self.fn_mut_param(&[item], BOOL, span);
                (if n == "any" { B::IterAny } else { B::IterAll }, vec![f], BOOL)
            }
            "find" => {
                let ri = r(self, item);
                let p = self.fn_mut_param_ref(&[ri], BOOL, span);
                let o = self.tys.mk_option(item);
                (B::IterFind, vec![p], o)
            }
            "find_map" => {
                let out = self.new_var(false);
                let o = self.tys.mk_option(out);
                let f = self.fn_mut_param(&[item], o, span);
                (B::IterFindMap, vec![f], o)
            }
            "position" => {
                let p = self.fn_mut_param(&[item], BOOL, span);
                let o = self.tys.mk_option(USIZE);
                (B::IterPosition, vec![p], o)
            }
            // ExactSizeIterator + DoubleEndedIterator: the slice and
            // array iterators lust has, and Map, Copied, Cloned of them.
            "rposition" => {
                if !self.rposition_ok(base) {
                    let tn = self.tn(base);
                    self.err("L0640", span, format!("`rposition` on `{tn}` is not implemented yet"), "Use `rposition` on a slice iterator (`v.iter()`).");
                    return self.iter_bail(args);
                }
                let p = self.fn_mut_param(&[item], BOOL, span);
                let o = self.tys.mk_option(USIZE);
                (B::IterRposition, vec![p], o)
            }
            "fold" => {
                let Some(&init) = args.first() else {
                    self.err("L0105", span, "`fold` takes 2 arguments", "Pass the initial value and the closure.");
                    return self.iter_bail(args);
                };
                let acc = self.expr(init, None);
                let f = self.fn_mut_param(&[acc, item], acc, span);
                if args.len() != 2 {
                    self.err("L0105", span, "`fold` takes 2 arguments", "Pass the initial value and the closure.");
                }
                self.set_res(e, Res::BuiltinMethod { b: B::IterFold, derefs, autoref: Autoref::None });
                for &a in &args[1..] {
                    self.expect_expr(a, f);
                }
                return acc;
            }
            // `try_fold<B, F, R>(&mut self, init: B, f: F) -> R` with
            // `F: FnMut(B, Item) -> R` and `R: Try<Output = B>`: `R` is
            // whatever the closure returns, then its output must be the
            // accumulator, as rustc's projection fixes it before the
            // literal fallback (`try_fold(0, |v, &c| ..)` -> `Option<u32>`
            // makes `0` a `u32`). `Option`, `Result` and `ControlFlow<B,
            // C>` (whose output is `C`) are modeled.
            "try_fold" => {
                if args.len() != 2 {
                    self.err("L0105", span, "`try_fold` takes 2 arguments", "Pass the initial value and the closure.");
                    return self.iter_bail(args);
                }
                let acc = self.expr(args[0], None);
                let ret = self.new_var(false);
                let f = self.fn_mut_param(&[acc, item], ret, span);
                self.set_res(e, Res::BuiltinMethod { b: B::IterTryFold, derefs, autoref: Autoref::RefMut });
                self.check_packed_borrow(recv, span);
                self.expect_expr(args[1], f);
                if self.dyn_once_callable(f) {
                    let s = self.span(args[1]);
                    self.errs.push(generics::dyn_not_yet(s, "a `dyn FnOnce` value as an iterator adapter's callable"));
                    return ERROR;
                }
                // A fn item or fn pointer's output is an obligation's.
                if matches!(self.kind(ret), TyKind::Infer(_)) {
                    self.run_obligations(false);
                }
                let r = self.resolve(ret);
                let out = self.tys.option_elem(r).or_else(|| self.tys.result_args(r).map(|(t, _)| t)).or_else(|| self.control_flow_args(r).map(|(_, _, c)| c));
                let Some(out) = out else {
                    if r != ERROR {
                        let tn = self.tn(r);
                        self.err("L0640", span, format!("`try_fold` with a closure result of type `{tn}` is not implemented yet"), "Return an `Option`, a `Result` or a `ControlFlow` from the closure.");
                    }
                    return ERROR;
                };
                if !self.unify(out, acc) {
                    self.mismatch(span, acc, out);
                    return ERROR;
                }
                return ret;
            }
            "for_each" => {
                let f = self.fn_mut_param(&[item], UNIT, span);
                (B::IterForEach, vec![f], UNIT)
            }
            // `partition<B, F>(self, f) -> (B, B)` and `unzip<A, B, FromA,
            // FromB>(self) -> (FromA, FromB)`: lust builds Vec targets the
            // expected type (or `partition`'s turbofish) names.
            "partition" => {
                let ri = r(self, item);
                let f = self.fn_mut_param_ref(&[ri], BOOL, span);
                let named = turbofish.first().and_then(|g| match g {
                    ast::GenericArg::Type(t) => Some(self.resolve_written(*t)),
                    _ => None,
                });
                let want = self.pair_target(expected, named, [item, item], "partition", span);
                let Some(ret) = want else { return self.iter_bail(args) };
                (B::IterPartition, vec![f], ret)
            }
            "unzip" => {
                let d = self.deep(item);
                let TyKind::Tuple(ts) = self.kind(d) else {
                    let tn = self.tn(d);
                    self.err("L0612", span, format!("`unzip` needs items that are pairs, not `{tn}`"), "Map each item to a `(a, b)` pair first.");
                    return self.iter_bail(args);
                };
                let parts = self.tys.tys_of(ts).to_vec();
                let [a, b] = parts[..] else {
                    self.err("L0612", span, "`unzip` needs items that are pairs", "Map each item to a `(a, b)` pair first.");
                    return self.iter_bail(args);
                };
                let Some(ret) = self.pair_target(expected, None, [a, b], "unzip", span) else { return self.iter_bail(args) };
                (B::IterUnzip, vec![], ret)
            }
            "min" | "max" => {
                self.deferred_ord.push((item, span));
                let o = self.tys.mk_option(item);
                (if n == "min" { B::IterMin } else { B::IterMax }, vec![], o)
            }
            "min_by_key" | "max_by_key" => {
                let key = self.new_var(false);
                let ri = r(self, item);
                let f = self.fn_mut_param_ref(&[ri], key, span);
                self.deferred_ord.push((key, span));
                let o = self.tys.mk_option(item);
                (if n == "min_by_key" { B::IterMinByKey } else { B::IterMaxByKey }, vec![f], o)
            }
            "collect" => {
                let target = self.iter_target(turbofish, expected, span);
                // A turbofish with holes (`collect::<Result<_, _>>()?`)
                // gets its holes from the expected type, as rustc's
                // inference does; without this the inner collection
                // of a try target is a bare variable here (F3). A
                // mismatch rolls back: the caller's own check of the
                // result type reports it.
                if !turbofish.is_empty()
                    && let Some(want) = expected
                {
                    let snap = self.snapshot_vars();
                    if !self.unify(target, want) {
                        self.rollback_vars(snap);
                    }
                }
                if args.is_empty() {
                    match self.from_iter_route(e, target, item, span, actual_base, derefs) {
                        Some(true) => return self.resolve(target),
                        Some(false) => return self.iter_bail(args),
                        None => {}
                    }
                }
                let tr = self.resolve(target);
                // A target only a later use fixes: checked at finish. A
                // `FromIterator` ADT found then is still rejected there
                // (its route above is picked here).
                // So is a `Box`, `Rc`, or `Arc` whose pointee is still open
                // (`Box::leak(it.collect())` coerced to `&[T]` later).
                let open_pointee = matches!(self.tys.builtin_of(tr), Some(BuiltinTy::Boxed | BuiltinTy::Rc | BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc)))
                    && self.tys.adt_first_ty_arg(tr).is_some_and(|e| matches!(self.kind(self.resolve(e)), TyKind::Infer(_)));
                if (matches!(self.kind(tr), TyKind::Infer(_)) && !self.kind(tr).is_int_infer() && !self.kind(tr).is_float_infer()) || open_pointee {
                    self.deferred_collect.push((target, item, span, (e, actual_base, derefs)));
                } else if !self.collect_target_ok(target, item, span) {
                    return self.iter_bail(args);
                }
                (B::IterCollect, vec![], target)
            }
            "sum" | "product" => {
                let target = self.iter_target(turbofish, expected, span);
                // An open item type picks no impl (`u32: Sum<u32>` and
                // `Sum<&u32>`), nor does an open result type (`S: Sum<&u64>`
                // with `S` fixed by a later `.unwrap_or(0)` into a `u64`
                // return): rustc keeps `S: Sum<Item>` pending, and body end
                // checks it.
                // A literal variable result (`match .. { 0 => 0, _ => it.sum() }`)
                // waits too: body end picks the one impl it fits.
                let open = {
                    let (i, t) = (self.resolve(item), self.resolve(target));
                    let tk = self.kind(t);
                    self.sum_item_open(t, i) || self.unbound_general(t) || tk.is_int_infer() || tk.is_float_infer()
                };
                if open {
                    self.deferred_sum.push((target, item, n == "sum", span));
                } else if !self.sum_target_ok(target, item, n, span) {
                    return self.iter_bail(args);
                }
                (if n == "sum" { B::IterSum } else { B::IterProduct }, vec![], target)
            }
            _ => return self.iter_unknown(n, span, base, args),
        };
        let autoref = match (by_value, n) {
            (true, _) => Autoref::None,
            // `size_hint(&self)`, and the drain views (`len`, `as_slice`, `as_str`).
            (false, "size_hint" | "len" | "as_slice" | "as_str" | "remainder") => Autoref::Ref,
            (false, _) => Autoref::RefMut,
        };
        if autoref != Autoref::None {
            self.check_packed_borrow(recv, span);
        }
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref });
        // An expected adapter type fixes the adapter's own args first, so
        // a closure argument sees a fn pointer the result names
        // (`-> Map<Iter<'_, T>, fn(&T) -> U>`), as rustc's expected
        // inputs do (W-CHECK-G).
        if !params.is_empty()
            && let Some(want) = expected.map(|t| self.resolve(t))
            && it::is_iter(&self.tys, ret)
            && self.tys.adt_parts(want).zip(self.tys.adt_parts(ret)).is_some_and(|((a, _), (b, _))| a == b)
        {
            let snap = self.snapshot_vars();
            if !self.unify(ret, want) {
                self.rollback_vars(snap);
            }
        }
        self.check_args(&params, args, span);
        // The expansion calls closures, fn items, fn pointers, and `dyn Fn`
        // and `dyn FnMut` values, also behind references and boxes
        // (`.for_each(&mut f)`, `.map(r)` for `r: &dyn Fn(..)`, lower/iter.rs
        // `call_fn`). Callable params are the `fn_mut_param` vars.
        for (&a, &pt) in args.iter().zip(&params) {
            let is_callable_param = matches!(self.tys.kind(pt), TyKind::Infer(v) if self.iter_closure_args.contains_key(&v.0));
            if is_callable_param && self.dyn_once_callable(pt) {
                let s = self.span(a);
                self.errs.push(generics::dyn_not_yet(s, "a `dyn FnOnce` value as an iterator adapter's callable"));
                return ERROR;
            }
        }
        ret
    }

    /// A builtin whose C adapter calls its callable argument (the sort
    /// and binary search `_by` family, `retain`, `bool::then`) calls a
    /// closure, a fn item or a fn pointer, behind references or boxes
    /// (cgen `adapter_callee`). A `dyn Fn` one meets the bound but has
    /// no adapter yet: an error here, never a codegen failure. The
    /// callable is the last argument. `once`: the builtin calls it as
    /// `FnOnce` (`bool::then`, and the Entry closure forms that
    /// lower/iter.rs expands), where a boxed closure would be moved out
    /// of its box, which neither does.
    pub(crate) fn adapter_callable_gap(&mut self, args: &[ExprId], params: &[Ty], once: bool) -> bool {
        let (Some(&a), Some(&pt)) = (args.last(), params.last()) else { return false };
        let mut t = self.deep(pt);
        let mut boxed = false;
        loop {
            t = match self.kind(t) {
                TyKind::Ref(_, _, i) => self.resolve(i),
                _ if let Some(i) = self.tys.boxed_elem(t) => {
                    boxed = true;
                    self.resolve(i)
                }
                _ => break,
            };
        }
        let what = if matches!(self.kind(t), TyKind::Dyn(..)) {
            "a `dyn Fn` value as this method's callable"
        } else if once && boxed {
            "a boxed callable as this method's fn"
        } else {
            return false;
        };
        let s = self.span(a);
        self.errs.push(generics::dyn_not_yet(s, what));
        true
    }

    /// True when `t` is an iterator only through a bound: a type
    /// parameter (`impl Iterator<..>` in argument position is one), an
    /// opaque type, or a projection, with `Iterator` evidence from the
    /// env. Its adapters wrap it as the builtin adapter types do; an
    /// instance checks again with the concrete type, so codegen only
    /// sees builtin chains. A user iterator type by impl is not one:
    /// its chains would need the unfused path (docs/ARCH.md 3.4).
    pub(super) fn param_iter(&mut self, t: Ty) -> bool {
        let t = self.deep(t);
        if !self.p.gx.has_generics || !matches!(self.kind(t), TyKind::Param(_) | TyKind::Opaque(..) | TyKind::Projection(..)) {
            return false;
        }
        let args_id = self.tys.intern_args(&[GenericArg::Ty(t)]);
        let tr = TraitRef { def: self.p.gx.lang.iterator, args: args_id, bindings: EMPTY_BINDINGS };
        let snap = self.snapshot_vars();
        let ok = matches!(self.select_obligation(tr), Selection::ParamEvidence(_));
        self.rollback_vars(snap);
        ok
    }

    /// The item type of an iterator `iter_call` accepts: the builtin
    /// table, else `<t as Iterator>::Item` normalized in this env (a
    /// chain over a bound-only iterator, see `param_iter`).
    pub(super) fn iter_item(&mut self, t: Ty) -> Ty {
        if let Some(item) = it::item_ty(&mut self.tys, t) {
            return item;
        }
        let args_id = self.tys.intern_args(&[GenericArg::Ty(t)]);
        let proj = self.tys.intern(TyKind::Projection(AssocId(self.p.gx.lang.assoc_item), args_id));
        self.normalize_here(proj)
    }

    /// True for the Iterator methods `iter_call` types.
    fn iter_method_known(&self, n: &str) -> bool {
        matches!(
            n,
            "rposition"
                | "by_ref"
                | "size_hint"
                | "next_back"
                | "flatten"
                | "flat_map"
                | "map"
                | "filter"
                | "filter_map"
                | "map_while"
                | "enumerate"
                | "rev"
                | "take"
                | "skip"
                | "step_by"
                | "chain"
                | "zip"
                | "copied"
                | "cloned"
                | "take_while"
                | "skip_while"
                | "peekable"
                | "inspect"
                | "count"
                | "last"
                | "fold"
                | "try_fold"
                | "for_each"
                | "min"
                | "max"
                | "min_by_key"
                | "max_by_key"
                | "collect"
                | "partition"
                | "size_hint"
                | "sum"
                | "product"
        )
    }

    /// Checks the arguments of a rejected call (so their own errors
    /// still report) and answers the error type.
    pub(super) fn iter_bail(&mut self, args: &[ExprId]) -> Ty {
        for &a in args {
            self.expr(a, None);
        }
        ERROR
    }

    /// A method name that is not one `iter_call` types.
    fn iter_unknown(&mut self, n: &str, span: Span, base: Ty, args: &[ExprId]) -> Ty {
        let tn = self.tn(base);
        if DECLARED_ONLY.contains(&n) || n == "peek" {
            self.err("L0640", span, format!("the iterator method `{n}` is not implemented yet"), "Rewrite the chain with the supported adapters, or with a loop.");
        } else {
            self.err(
                "L0104",
                span,
                format!("no method `{n}` on type `{tn}`"),
                "Check the method name. The prototype supports a small std subset (see README).",
            );
        }
        self.iter_bail(args)
    }

    /// An `IntoIterator` argument `a` of type `t` that is an iterator lust
    /// calls through its own `next` (a user type, or a type parameter
    /// with an `Iterator` bound): moved into its `ByNext` wrapper, whose
    /// type is returned. None for other types; ERROR after L0640.
    pub(super) fn by_next_arg(&mut self, a: ExprId, t: Ty) -> Option<Ty> {
        if it::is_iter(&self.tys, t) {
            return None;
        }
        let item = self.by_next_item(t, self.span(a))?;
        if item == ERROR {
            return Some(ERROR);
        }
        let wrapped = it::mk_iter(&mut self.tys, IterKind::ByNext, &[t, item]);
        let i = self.ei(a);
        self.out.coerce[i] = Coerce::ByNext(wrapped);
        Some(wrapped)
    }

    /// A std collection iterator struct as an `extend` argument (a local
    /// of type `btree_map::Values`, as an instance with I = Values has):
    /// its inner builtin iterator, which lowering unwraps (lower/iter.rs
    /// `into_iter`). BTree `range` iterators stay out: std hints them
    /// `(0, None)`, which the reserve would read.
    pub(super) fn wrapper_iter_arg(&mut self, t: Ty) -> Option<Ty> {
        if self.p.is_btree_range_iter(&mut self.tys, t) {
            return None;
        }
        self.p.collection_iter_inner(&mut self.tys, t)
    }

    /// An `IntoIterator` argument (`extend`, `from_iter`) of type `t`:
    /// an iterator through its own `next` (`by_next_arg`), else a value
    /// through a written or bound `IntoIterator` impl, moved by its
    /// `into_iter` call (`Coerce::IntoIterCall`) into the final iterator
    /// type. The `for` head's conversion (`for_trait_iter`) for a plain
    /// argument: a template bound answers with a stub instance, and an
    /// instance check re-selects with the concrete type.
    pub(super) fn into_iterator_arg(&mut self, a: ExprId, t: Ty) -> Option<Ty> {
        if let Some(x) = self.by_next_arg(a, t) {
            return Some(x);
        }
        if !self.generic_mode() || it::is_iter(&self.tys, t) {
            return None;
        }
        self.run_obligations(false);
        let t = self.deep(t);
        if matches!(self.kind(t), TyKind::Error) {
            return None;
        }
        let p = self.p;
        let lang = p.gx.lang;
        let span = self.span(a);
        let args_id = self.tys.intern_args(&[GenericArg::Ty(t)]);
        let tr = TraitRef { def: lang.into_iterator, args: args_id, bindings: EMPTY_BINDINGS };
        let into_local = match self.select_obligation(tr) {
            Selection::Impl { id, args: impl_args, .. } => {
                // A synthetic impl (the `Iterator` blanket, a builtin
                // collection) has no `into_iter` body to call: the
                // builtin routes above already cover those types.
                if p.gx.index.get(id).items.is_empty() {
                    return None;
                }
                self.method_instance_for_impl(a, id, tr, impl_args, "into_iter", span)?.0
            }
            Selection::ParamEvidence(_) => self.stub_instance(a, tr, p.interner.intern("into_iter"), span)?.0,
            _ => return None,
        };
        let iter_proj = self.tys.intern(TyKind::Projection(AssocId(lang.assoc_into_iter), args_id));
        let iter_ty = self.normalize_here(iter_proj);
        let iter_ty = self.deep(iter_ty);
        // The final iterator: a builtin iterator as itself, else
        // through its `next` in the `ByNext` wrapper.
        let wrapped = if it::is_iter(&self.tys, iter_ty) {
            iter_ty
        } else {
            let item = self.by_next_item(iter_ty, span)?;
            if item == ERROR {
                return Some(ERROR);
            }
            it::mk_iter(&mut self.tys, IterKind::ByNext, &[iter_ty, item])
        };
        let i = self.ei(a);
        self.out.coerce[i] = Coerce::IntoIterCall(wrapped);
        self.out.into_iter_calls.push((a, into_local));
        Some(wrapped)
    }

    /// The item type of `base` as an iterator through its own `next`
    /// (`IterKind::ByNext`, W-STD-B), or None when it is not one. A type
    /// parameter, projection, or opaque type needs an `Iterator` bound
    /// (template checks; instances see the concrete type). A user type
    /// needs an `Iterator` impl lust can call (`by_next_fns`); one it
    /// cannot call is L0640 and gives ERROR.
    fn by_next_item(&mut self, base: Ty, span: Span) -> Option<Ty> {
        // A collection iterator struct is handled as its inner
        // iterator (iter_method), never through `ByNext`.
        if self.p.collection_iter_inner(&mut self.tys, base).is_some() {
            return None;
        }
        let p = self.p;
        let lang = p.gx.lang;
        // `&mut I` over a builtin iterator (`v.extend(&mut it)`, `I = &mut
        // array::IntoIter<T, N>` in an instance) yields `I`'s items through
        // its `next` (core's `Iterator for &mut I`).
        if let TyKind::Ref(_, Mut::Mut, inner) = self.kind(base) {
            let inner = self.deep(inner);
            if it::is_iter(&self.tys, inner) {
                return it::item_ty(&mut self.tys, inner);
            }
            // A std collection iterator struct: its inner iterator's items
            // (not BTree `range` iterators, as above).
            if !self.p.is_btree_range_iter(&mut self.tys, inner)
                && let Some(i) = self.p.collection_iter_inner(&mut self.tys, inner)
            {
                return it::item_ty(&mut self.tys, i);
            }
            // A user iterator: its own items; the `ByNext` wrapper holds the
            // `&mut` and steps `*r` in place, as `(&mut it).take(2)` does
            // (`v.extend(&mut it)`, `I = &mut UserIter` in an instance).
            if self.tys.local_adt(inner).is_some() && self.tys.builtin_of(inner).is_none() {
                return self.by_next_item(inner, span);
            }
        }
        let user = match self.kind(base) {
            TyKind::Param(_) | TyKind::Projection(..) | TyKind::Opaque(..) => false,
            TyKind::Adt(..) if self.tys.local_adt(base).is_some() && self.tys.builtin_of(base).is_none() => true,
            _ => return None,
        };
        let args = self.tys.intern_args(&[GenericArg::Ty(base)]);
        // `<I as IntoIterator>::IntoIter` is an Iterator with the
        // IntoIterator's own Item, by the trait's declaration.
        if let TyKind::Projection(AssocId(a), pargs) = self.kind(base)
            && a == lang.assoc_into_iter
        {
            let item = self.tys.intern(TyKind::Projection(AssocId(lang.assoc_into_item), pargs));
            return Some(self.normalize_here(item));
        }
        let tr = TraitRef { def: lang.iterator, args, bindings: EMPTY_BINDINGS };
        let impl_id = match self.select_obligation(tr) {
            Selection::Impl { id, .. } => Some(id),
            Selection::ParamEvidence(_) => None,
            _ => return None,
        };
        // The selected impl's shape is the gate: a written impl the
        // expansion can call through `next` (generic impls included;
        // their instances are made with the Adt, `ensure_iter_methods`).
        if user
            && let Some(id) = impl_id
            && let Err(why) = iter_impl_shape(p, p.gx.index.get(id))
        {
            let tn = self.tn(base);
            self.err("L0640", span, format!("adapters over `{tn}` are not implemented yet: {why}"), "Loop with `while let Some(x) = it.next()` instead.");
            return Some(ERROR);
        }
        let item = self.tys.intern(TyKind::Projection(AssocId(lang.assoc_item), args));
        Some(self.normalize_here(item))
    }

    /// A fresh closure-argument type with the pending obligations of
    /// `F: FnMut(inputs..) -> output`. The closure's arguments are items
    /// (or `&item`, see `fn_mut_param_ref`): free regions, which the
    /// borrow checker takes from the iterator (W-BORROW).
    pub(super) fn fn_mut_param(&mut self, inputs: &[Ty], output: Ty, span: Span) -> Ty {
        self.fn_mut_param_flags(inputs, &vec![false; inputs.len()], output, span)
    }

    /// `fn_mut_param` for `F: FnMut(&Self::Item) -> output`: the outer
    /// reference is higher-ranked (Rust's bound writes it).
    pub(super) fn fn_mut_param_ref(&mut self, inputs: &[Ty], output: Ty, span: Span) -> Ty {
        self.fn_mut_param_flags(inputs, &vec![true; inputs.len()], output, span)
    }

    fn fn_mut_param_flags(&mut self, inputs: &[Ty], by_ref: &[bool], output: Ty, span: Span) -> Ty {
        let f = self.new_var(false);
        if let TyKind::Infer(v) = self.kind(f) {
            self.iter_closure_args.insert(v.0, by_ref.to_vec());
        }
        let lang = self.p.gx.lang;
        let ts = self.tys.intern_tys(inputs);
        let tuple = self.tys.intern(TyKind::Tuple(ts));
        let args = self.tys.intern_args(&[GenericArg::Ty(f), GenericArg::Ty(tuple)]);
        self.obl.push(obligations::Obligation::Trait { trait_ref: TraitRef { def: lang.fn_mut, args, bindings: EMPTY_BINDINGS }, span, depth: 0 });
        if let Some(out) = lang.output_of(lang.fn_once) {
            self.obl.push(obligations::Obligation::AssocEq { eq: AssocEqData { assoc: out, trait_args: args, ty: output }, span });
        }
        f
    }

    /// The iterator a `chain` or `zip` argument turns into through
    /// `IntoIterator`: an iterator as itself, `&X`/`&mut X` over a
    /// Vec, slice, or array as a slice iterator, a Vec by value as
    /// `vec::IntoIter`, a map or set (or a reference to one) as its
    /// hash iterator. None for other types.
    pub(super) fn into_iter_ty(&mut self, t: Ty) -> Option<Ty> {
        // A collection iterator struct chains as its inner iterator.
        if let Some(inner) = self.p.collection_iter_inner(&mut self.tys, t) {
            return Some(inner);
        }
        // A user iterator type by value: its `next` wrapper (W-STD-B).
        if !it::is_iter(&self.tys, t)
            && !self.param_iter(t)
            && let Some(item) = self.by_next_item(t, Span::default())
        {
            return (item != ERROR).then(|| it::mk_iter(&mut self.tys, IterKind::ByNext, &[t, item]));
        }
        if it::is_iter(&self.tys, t) || self.param_iter(t) {
            return Some(t);
        }
        if let Some(t2) = it::opt_res_iter(&mut self.tys, t) {
            return Some(t2);
        }
        if let Some(el) = self.tys.vec_elem(t) {
            return Some(it::mk_iter(&mut self.tys, IterKind::VecIntoIter, &[el]));
        }
        // An array by value (`array::IntoIter`, fx-array-iter).
        if let TyKind::Array(el, _) = self.kind(t) {
            return Some(it::mk_iter(&mut self.tys, IterKind::ArrayIntoIter, &[el, t]));
        }
        if let Some(h) = self.hash_into_iter_ty(t) {
            return Some(h);
        }
        let TyKind::Ref(_, m, inner) = self.kind(t) else { return None };
        let inner = self.resolve(inner);
        let el = match self.kind(inner) {
            TyKind::Slice(el) | TyKind::Array(el, _) => el,
            _ => self.tys.vec_elem(inner)?,
        };
        let k = if m.is_mut() { IterKind::SliceIterMut } else { IterKind::SliceIter };
        Some(it::mk_iter(&mut self.tys, k, &[el]))
    }

    /// std's `IntoIterator` for a map or set: by value the owning
    /// iterator, `&` the shared one, `&mut` a map's `IterMut` (there is
    /// no `IntoIterator for &mut HashSet`). None for other types.
    fn hash_into_iter_ty(&mut self, t: Ty) -> Option<Ty> {
        use crate::ty::hash::{HashTy, hash_parts};
        let (by_ref, m, tab) = match self.kind(t) {
            TyKind::Ref(_, m, inner) => (true, m.is_mut(), self.resolve(inner)),
            _ => (false, false, t),
        };
        let (kind, a) = hash_parts(&self.tys, tab)?;
        let (ik, args) = match (kind, by_ref, m) {
            (HashTy::HashMap, false, _) => (IterKind::MapIntoIter, vec![a[0], a[1]]),
            (HashTy::HashMap, true, false) => (IterKind::MapIter, vec![a[0], a[1]]),
            (HashTy::HashMap, true, true) => (IterKind::MapIterMut, vec![a[0], a[1]]),
            (HashTy::HashSet, false, _) => (IterKind::SetIntoIter, vec![a[0]]),
            (HashTy::HashSet, true, false) => (IterKind::SetIter, vec![a[0]]),
            _ => return None,
        };
        Some(it::mk_iter(&mut self.tys, ik, &args))
    }

    /// The `(Vec<x>, Vec<y>)` result of `partition` or `unzip`: the
    /// written target (`named`, both halves) or the expected pair. Each
    /// half must be a Vec of its items; an open half is rustc's E0282
    /// when nothing fixes it, so lust asks for the annotation (L0610),
    /// and another collection is L0640.
    /// True when `el` is a known `Copy` type and `item` is `&el`: a Vec of `el` extends
    /// from such items (alloc's `impl Extend<&T> for Vec<T>`), as `partition` and
    /// `unzip` fill their sides.
    fn extends_by_copy(&mut self, el: Ty, item: Ty) -> bool {
        let (el, item) = (self.deep(el), self.deep(item));
        if self.tys.flags(el).contains_infer() || !is_copy(self.p, &self.tys, el) {
            return false;
        }
        matches!(self.kind(item), TyKind::Ref(_, Mut::Shared, inner) if self.deep(inner) == el)
    }

    fn pair_target(&mut self, expected: Option<Ty>, named: Option<Ty>, items: [Ty; 2], what: &str, span: Span) -> Option<Ty> {
        let halves = match (named, expected.map(|t| self.deep(t))) {
            (Some(b), _) => [b, b],
            (None, Some(t)) => match self.kind(t) {
                TyKind::Tuple(ts) if self.tys.tys_of(ts).len() == 2 => {
                    let v = self.tys.tys_of(ts).to_vec();
                    [v[0], v[1]]
                }
                _ => [ERROR, ERROR],
            },
            (None, None) => [ERROR, ERROR],
        };
        for (h, it) in halves.into_iter().zip(items) {
            let h = self.deep(h);
            // `partition` into a `VecDeque` (lowered as a Vec, then
            // wrapped; lower/iter.rs `partition_unzip`). `unzip` stays
            // out: its `extend_reserve` path is not lowered for a deque.
            let deque = self.tys.local_adt(h).filter(|&a| what == "partition" && self.p.is_vec_deque_adt(a));
            match deque.and_then(|_| self.tys.adt_first_ty_arg(h)).or_else(|| self.tys.vec_elem(h)) {
                // `Vec<T>: Extend<&T>` for `T: Copy`: a known element type also
                // takes shared refs to itself (lowering copies through them).
                Some(el) if self.extends_by_copy(el, it) => {}
                Some(el) => {
                    if !self.unify(el, it) {
                        self.mismatch(span, el, it);
                        return None;
                    }
                }
                None if h == ERROR || matches!(self.kind(h), TyKind::Infer(_)) => {
                    self.err("L0610", span, format!("type annotations needed: the target of this `{what}` must be known here"), "Annotate the result, such as `let (a, b): (Vec<_>, Vec<_>) = ..`.");
                    return None;
                }
                None => {
                    let tn = self.tn(h);
                    self.err("L0640", span, format!("`{what}` into `{tn}` is not implemented yet"), "Collect into a pair of `Vec`s.");
                    return None;
                }
            }
        }
        // `partition` answers `(B, B)`: one collection type for both sides.
        if what == "partition" && named.is_none() && !self.unify(halves[0], halves[1]) {
            self.mismatch(span, halves[0], halves[1]);
            return None;
        }
        let pair = [self.deep(halves[0]), self.deep(halves[1])];
        let ts = self.tys.intern_tys(&pair);
        Some(self.tys.intern(TyKind::Tuple(ts)))
    }

    /// `collect` into a source collection (`BTreeMap`, `BTreeSet`, or a
    /// user or adapter type with a `FromIterator` impl, `IndexMap`): the
    /// call `e` becomes `FromIterator::from_iter(self)`, picked at body
    /// end (W-STD-C). Used at the call when the target is known, and at
    /// body end for a target only a later use fixed (`opt.map(|a|
    /// a.iter().collect()).unwrap_or_default()` into a field). None when
    /// the target is not such a type; Some(false) after a reported error.
    pub(super) fn from_iter_route(&mut self, e: ExprId, target: Ty, item: Ty, span: Span, actual_base: Ty, derefs: u8) -> Option<bool> {
        // A source collection (`BTreeMap`, `BTreeSet`, or a user
        // type with a `FromIterator` impl, adapter crates
        // included): the call is its
        // `FromIterator::from_iter(self)`, as rustc's `collect`
        // is (W-STD-C). The `Target: FromIterator<Item>`
        // obligation proves the impl's bounds (`T: Ord`).
        // Std ADTs take this route as before; a user ADT only
        // when selection finds an impl for this item type, so
        // a user type without one keeps the L0640 below.
        // `VecDeque` is not here: its collect capacity is
        // `Vec`'s, so it takes the builtin collect below and
        // lowering wraps the `Vec` (W-STD-C phase 2).
        let tr = self.resolve(target);
        // A try target (`Result<C, E>` or `Option<C>`, F3):
        // the source-collection question is about `C` and
        // the item inside the `Result` or `Option`; the
        // obligation and the deferred call stay on the whole
        // target, as core's generic impl names it.
        let (route_ty, route_item) = match self.try_collect_parts(tr) {
            Some((c, err)) => match self.try_collect_inner(item, err, span) {
                Some(inner) => (c, inner),
                None => return Some(false),
            },
            None => (tr, item),
        };
        let from_iter_ref = |this: &mut Self| TraitRef {
            def: this.p.gx.lang.from_iterator,
            args: this.tys.intern_args(&[GenericArg::Ty(route_ty), GenericArg::Ty(route_item)]),
            bindings: EMPTY_BINDINGS,
        };
        if let Some(a) = self.tys.local_adt(route_ty).or_else(|| self.tys.adt_parts(route_ty).and_then(|(d, _)| self.tys.local_adt_of(d)))
            && !self.p.is_vec_deque_adt(a)
            && (self.p.is_std_adt(a)
                || {
                    let r = from_iter_ref(self);
                    matches!(self.select_obligation(r), Selection::Impl { .. })
                })
        {
            // Each has one `FromIterator` impl, whose item is the
            // entry type: unify it now, so `collect::<BTreeMap<_,
            // _>>()` infers `K` and `V` from the items.
            let path = self.p.display_path(self.p.adt(a).name);
            let targs: Vec<Ty> = self.tys.adt_parts(route_ty).map(|(_, x)| self.tys.args(x).iter().filter_map(|g| g.as_ty()).collect()).unwrap_or_default();
            let entry = match (path.rsplit("::").next(), targs.as_slice()) {
                (Some("BTreeMap"), [k, v]) => {
                    let ts = self.tys.intern_tys(&[*k, *v]);
                    Some(self.tys.intern(TyKind::Tuple(ts)))
                }
                (Some("BTreeSet"), [t]) => Some(*t),
                _ => None,
            };
            if let Some(entry) = entry
                && !self.unify(entry, route_item)
            {
                self.mismatch(span, entry, route_item);
                return Some(false);
            }
            let targs = self.tys.intern_args(&[GenericArg::Ty(tr), GenericArg::Ty(item)]);
            let trait_ref = TraitRef { def: self.p.gx.lang.from_iterator, args: targs, bindings: EMPTY_BINDINGS };
            self.obl.push_trait_at(trait_ref, span, 0);
            let name = self.p.interner.intern("from_iter");
            super::method::defer_trait_method(self, e, trait_ref, name, vec![GenericArg::Ty(actual_base)], derefs, Autoref::None);
            return Some(true);
        }
        None
    }

    /// The deferred collects whose target a later use made a `Vec`: that
    /// target has one `FromIterator` impl, so rustc's obligation pass
    /// before the next method call has already unified its element with
    /// the items (`let rs = it.collect(); wait(rs).into_iter().flatten()`).
    /// True when one settled.
    fn settle_vec_collects(&mut self) -> bool {
        let mut settled = false;
        for (target, item, span, call) in std::mem::take(&mut self.deferred_collect) {
            let t = self.resolve(target);
            if self.tys.vec_elem(t).is_some() {
                self.collect_target_ok(t, item, span);
                settled = true;
            } else {
                self.deferred_collect.push((target, item, span, call));
            }
        }
        settled
    }

    /// The `IntoIter` of a `flatten` or `flat_map` inner value: what
    /// `zip` takes, or `option::IntoIter` for an Option. Anything else is
    /// L0612, and None.
    fn flat_inner(&mut self, t: Ty, span: Span) -> Option<Ty> {
        let mut t = self.deep(t);
        let mut core = t;
        while let TyKind::Ref(_, _, inner) = self.kind(core) {
            core = inner;
        }
        if matches!(self.kind(core), TyKind::Infer(_)) && self.settle_vec_collects() {
            t = self.deep(t);
        }
        if let Some(el) = self.tys.option_elem(t) {
            return Some(it::mk_iter(&mut self.tys, IterKind::OptionIntoIter, &[el]));
        }
        match self.into_iter_ty(t) {
            Some(u) => Some(u),
            None => {
                if !matches!(self.kind(t), TyKind::Error) {
                    let tn = self.tn(t);
                    self.err("L0612", span, format!("`{tn}` is not an iterator lust can flatten yet"), "Flatten Vecs, slices, arrays, Options, or iterators.");
                }
                None
            }
        }
    }

    /// For `flatten` over items of a user or adapter type (or a
    /// reference to one) with a written `IntoIterator` impl (R15, `&IndexMap`):
    /// the impl's `into_iter` as a fn item type, and its `IntoIter`.
    /// The instance is requested for mono here, so the item type must be
    /// fully known and free of type parameters. None for every other
    /// item type (the builtin routes of `flat_inner`).
    fn written_into_iter_fn(&mut self, item: Ty) -> Option<(Ty, Ty)> {
        use crate::traits::index::Synthetic;
        let t = self.deep(item);
        let mut core = t;
        while let TyKind::Ref(_, _, inner) = self.kind(core) {
            core = self.resolve(inner);
        }
        if self.tys.local_adt(core).is_none() || self.tys.builtin_of(core).is_some() || self.has_general_var(t) || self.tys.flags(t).contains_param() {
            return None;
        }
        let lang = self.p.gx.lang;
        let args_id = self.tys.intern_args(&[GenericArg::Ty(t)]);
        let tr = TraitRef { def: lang.into_iterator, args: args_id, bindings: EMPTY_BINDINGS };
        // A bail-out rolls back the variables and the obligations the
        // selection queued.
        let (snap, obl_len) = (self.snapshot_vars(), self.obl_len());
        let Selection::Impl { id, args: impl_args, unbound, .. } = self.select_obligation(tr) else {
            self.rollback_vars(snap);
            self.obl_truncate(obl_len);
            return None;
        };
        let rec = self.p.gx.index.get(id).clone();
        let name = self.p.interner.intern("into_iter");
        let f = rec.items.iter().find_map(|&d| self.p.local_def(d.hash).filter(|&l| self.p.fn_info(l).name == name));
        let (Some(f), Some(out)) = (f, rec.assoc_ty(lang.assoc_into_iter)) else {
            self.rollback_vars(snap);
            self.obl_truncate(obl_len);
            return None;
        };
        if rec.synthetic != Synthetic::None || !unbound.is_empty() {
            self.rollback_vars(snap);
            self.obl_truncate(obl_len);
            return None;
        }
        let prefix: Vec<GenericArg> = impl_args.into_iter().map(|g| if let GenericArg::Ty(x) = g { GenericArg::Ty(self.deep(x)) } else { g }).collect();
        if prefix.iter().any(|g| g.as_ty().is_some_and(|x| self.has_general_var(x))) {
            self.rollback_vars(snap);
            self.obl_truncate(obl_len);
            return None;
        }
        if self.p.gx.fn_templates.get(f.0 as usize).is_some_and(|x| x.is_some()) {
            self.into_iter_requests.push((f, prefix.clone()));
        }
        let fargs = self.tys.intern_args(&prefix);
        let g = self.tys.intern(TyKind::FnDef(self.p.def_id(f), fargs));
        let out = self.subst_settled(out, &prefix);
        Some((g, self.deep(out)))
    }

    /// True when `t` holds an inference variable that is not an integer
    /// or float literal's (those settle by fallback at the latest).
    fn has_general_var(&mut self, t: Ty) -> bool {
        let t = self.resolve(t);
        if !self.tys.flags(t).contains_infer() {
            return false;
        }
        match self.kind(t) {
            TyKind::Infer(v) => !v.is_int() && !v.is_float(),
            TyKind::Ref(_, _, e) | TyKind::Slice(e) | TyKind::Array(e, _) => self.has_general_var(e),
            TyKind::Tuple(ts) => self.tys.tys_of(ts).to_vec().into_iter().any(|e| self.has_general_var(e)),
            TyKind::Adt(_, a) => self.tys.args(a).to_vec().into_iter().any(|g| g.as_ty().is_some_and(|e| self.has_general_var(e))),
            _ => true,
        }
    }

    /// True when `rposition` runs on `t` here: an ExactSizeIterator and
    /// DoubleEndedIterator whose length the expansion knows (`size`).
    fn rposition_ok(&self, t: Ty) -> bool {
        use IterKind as K;
        match it::iter_parts(&self.tys, t) {
            Some((K::SliceIter | K::SliceIterMut | K::ArrayIntoIter | K::Windows, _)) => true,
            Some((K::Chunks | K::ChunksMut | K::ChunksExact | K::ChunksExactMut, _)) => true,
            Some((k, _)) if it::is_opt_res(k) => true,
            Some((K::Map | K::Copied | K::Cloned, a)) => self.rposition_ok(a[0]),
            _ => false,
        }
    }

    /// True when an iterator type is double-ended in Rust 1.99, for
    /// the kinds lust has. `span` places the deferred range checks of
    /// `exact_size` (a `Zip` side).
    fn double_ended(&mut self, t: Ty, span: Span) -> bool {
        use IterKind as K;
        if self.tys.is_chars_ty(t) {
            return true;
        }
        let Some((k, a)) = it::iter_parts(&self.tys, t) else { return false };
        match k {
            K::SliceIter
            | K::SliceIterMut
            | K::VecIntoIter
            | K::Range
            | K::RangeInclusive
            | K::Windows
            | K::Chunks
            | K::ChunksMut
            | K::ChunksExact
            | K::ChunksExactMut
            | K::CharIndices
            | K::VecDrain
            | K::StringDrain
            | K::Args => true,
            K::Map | K::Filter | K::FilterMap | K::Rev | K::Copied | K::Cloned | K::Inspect | K::Peekable => self.double_ended(a[0], span),
            K::Chain => self.double_ended(a[0], span) && self.double_ended(a[1], span),
            // `A, B: DoubleEndedIterator + ExactSizeIterator` (R22). A
            // side over a user iterator stays out: core's `next_back`
            // trusts its `len`, and a dishonest `size_hint` would reach
            // core's `unreachable!` (lower/iter.rs `zip_next_back`).
            K::Zip => (0..2).all(|i| self.double_ended(a[i], span) && self.exact_size(a[i], span) && !it::has_by_next(&self.tys, a[i])),
            // `I: DoubleEndedIterator + ExactSizeIterator`; not over a user
            // iterator, whose `len` core trusts (as for `Zip`).
            K::Enumerate | K::Skip => self.double_ended(a[0], span) && self.exact_size(a[0], span) && !it::has_by_next(&self.tys, a[0]),
            // These need ExactSizeIterator too: lust keeps them forward-only for now.
            K::Take | K::StepBy => false,
            K::TakeWhile | K::SkipWhile | K::MapWhile => false,
            K::RecvIter | K::RecvTryIter | K::RecvIntoIter | K::Utf8Chunks | K::Utf8Chunk => false,
            // hashbrown's iterators are forward-only.
            K::MapIter
            | K::MapIterMut
            | K::MapKeys
            | K::MapValues
            | K::MapValuesMut
            | K::MapIntoIter
            | K::MapIntoKeys
            | K::MapIntoValues
            | K::SetIter
            | K::SetIntoIter
            | K::MapDrain
            | K::SetDrain => false,
            K::ArrayIntoIter | K::RepeatN => true,
            // A user iterator with a `DoubleEndedIterator` impl.
            // Through `&mut I` (core: `impl<I: DoubleEndedIterator + ?Sized>
            // DoubleEndedIterator for &mut I`): as `I`.
            K::ByNext => match self.kind(a[0]) {
                TyKind::Ref(_, Mut::Mut, inner) if self.tys.is_chars_ty(inner) || it::iter_parts(&self.tys, inner).is_some() => self.double_ended(inner, span),
                TyKind::Ref(_, Mut::Mut, inner) => by_next_fns(self.p, &self.tys, inner).is_ok_and(|f| f.next_back.is_some()),
                _ => by_next_fns(self.p, &self.tys, a[0]).is_ok_and(|f| f.next_back.is_some()),
            },
            // Forward only in lust so far.
            K::Flatten | K::TryCollect => false,
            K::OptionIntoIter | K::OptionIter | K::OptionIterMut | K::ResultIter | K::ResultIterMut | K::ResultIntoIter => true,
            // The str splits (W-STD-B): by mode, and over a pattern whose
            // searcher is double-ended (char, char sets, predicates; not
            // `&str`), as std's `DoubleEndedSearcher` impls.
            // The str matches forms (tf-str): double-ended over a
            // double-ended searcher (not a string needle), as in std.
            K::StrMatches => !crate::check::split::str_needle(&self.tys, a[0]),
            K::EncodeUtf16 => false,
            // The slice splits: every form but the N ones, as core's impls.
            K::SliceSplit | K::SliceSplitMut => it::slice_split_mode(&self.tys, a[2]).is_some_and(|m| m.double_ended()),
            K::ChunkBy | K::ChunkByMut => true,
            K::StrSplit => match it::split_mode(&self.tys, a[1]) {
                Some(it::SplitMode::SplitN | it::SplitMode::RSplitN) | None => false,
                Some(it::SplitMode::SplitWhitespace | it::SplitMode::SplitAsciiWhitespace | it::SplitMode::Lines) => true,
                Some(_) => !crate::check::split::str_needle(&self.tys, a[0]),
            },
            K::RangeFrom | K::RangeTo | K::RangeFull | K::RangeToInclusive => false,
        }
    }

    /// True when Rust 1.99 implements `ExactSizeIterator` for an
    /// iterator type (the kinds lust has): the gate for `.len()`.
    pub(super) fn exact_size(&mut self, t: Ty, span: Span) -> bool {
        use IterKind as K;
        if self.tys.is_chars_ty(t) {
            return false;
        }
        let Some((k, a)) = it::iter_parts(&self.tys, t) else {
            // A bound-only iterator (`I: ExactSizeIterator`): the env
            // proves it, as `param_iter` proves `Iterator`.
            let t = self.deep(t);
            if !matches!(self.kind(t), TyKind::Param(_) | TyKind::Opaque(..) | TyKind::Projection(..)) {
                return false;
            }
            let args_id = self.tys.intern_args(&[GenericArg::Ty(t)]);
            let tr = TraitRef { def: self.p.gx.lang.exact_size_iterator, args: args_id, bindings: EMPTY_BINDINGS };
            let snap = self.snapshot_vars();
            let ok = matches!(self.select_obligation(tr), Selection::ParamEvidence(_));
            self.rollback_vars(snap);
            return ok;
        };
        match k {
            // core's `range_exact_iter_impl!` list: not the 64-bit,
            // 128-bit, or `char` ranges (inclusive: 8 and 16 bits only).
            // An integer variable is checked once it is known
            // (`deferred_exact_range`).
            K::Range | K::RangeInclusive => {
                let incl = k == K::RangeInclusive;
                match self.kind(self.resolve(a[0])) {
                    TyKind::Int(i) => exact_range_int(i, incl),
                    k if k.is_int_infer() => {
                        self.deferred_exact_range.push((a[0], incl, span));
                        true
                    }
                    _ => false,
                }
            }
            K::SliceIter
            | K::SliceIterMut
            | K::VecIntoIter
            | K::VecDrain
            | K::ArrayIntoIter
            | K::RepeatN
            | K::Windows
            | K::Chunks
            | K::ChunksMut
            | K::ChunksExact
            | K::ChunksExactMut
            | K::OptionIter
            | K::OptionIterMut
            | K::OptionIntoIter
            | K::ResultIter
            | K::ResultIterMut
            | K::ResultIntoIter
            | K::MapIter
            | K::MapIterMut
            | K::MapKeys
            | K::MapValues
            | K::MapValuesMut
            | K::MapIntoIter
            | K::MapIntoKeys
            | K::MapIntoValues
            | K::SetIter
            | K::SetIntoIter
            | K::MapDrain
            | K::SetDrain
            | K::Args => true,
            K::Map | K::Enumerate | K::Rev | K::Take | K::Skip | K::StepBy | K::Copied | K::Cloned | K::Peekable | K::Inspect => {
                self.exact_size(a[0], span)
            }
            // `Chain` is not `ExactSizeIterator` in std, even over
            // exact iterators; `Zip` is, over two exact ones. Neither
            // is `RangeInclusive` (its length can overflow `usize`).
            K::Zip => self.exact_size(a[0], span) && self.exact_size(a[1], span),
            // A user iterator through its `next`: exact when its
            // written `ExactSizeIterator` impl takes the default
            // `len` over an honest `size_hint` (`ByNextFns::exact`),
            // or when the held type is bound-only exact evidence.
            // Through `&mut I`: as `I` (core's `impl ExactSizeIterator for
            // &mut I`).
            K::ByNext if let TyKind::Ref(_, Mut::Mut, inner) = self.kind(a[0]) => match by_next_fns(self.p, &self.tys, inner) {
                Ok(f) => f.exact,
                Err(_) => self.exact_size(inner, span),
            },
            K::ByNext => match by_next_fns(self.p, &self.tys, a[0]) {
                Ok(f) => f.exact,
                Err(_) => self.exact_size(a[0], span),
            },
            _ => false,
        }
    }

    /// The target type of `collect`, `sum`, or `product`: the
    /// turbofish when written, else the expected type, else a fresh
    /// variable (which the target check then rejects).
    fn iter_target(&mut self, turbofish: &[ast::GenericArg], expected: Option<Ty>, span: Span) -> Ty {
        if let Some(ast::GenericArg::Type(t)) = turbofish.first() {
            let t = match self.turbofish_ty.take() {
                Some(t) => t,
                None => self.resolve_written(*t),
            };
            if turbofish.len() > 1 {
                self.err("L0105", span, "this method takes 1 type argument", "Write one target type.");
            }
            return t;
        }
        match expected {
            Some(t) => t,
            None => self.new_var(false),
        }
    }

    /// The parts of a try collect target (F3, core's two generic
    /// impls): `Result<C, E>` gives `(C, Some(E))`, `Option<C>`
    /// gives `(C, None)`. No other `FromIterator` impl names a
    /// `Result` or `Option`, so every such target is a try one.
    fn try_collect_parts(&mut self, t: Ty) -> Option<(Ty, Option<Ty>)> {
        let t = self.resolve(t);
        if let Some((c, e)) = self.tys.result_args(t) {
            return Some((self.resolve(c), Some(self.resolve(e))));
        }
        self.tys.option_elem(t).map(|c| (self.resolve(c), None))
    }

    /// The item inside a try collect's items: `item` unified with
    /// `Result<T, E>` or `Option<T>` for a fresh `T`, which it then
    /// returns. A mismatch is rustc's trait error, reported so.
    fn try_collect_inner(&mut self, item: Ty, err: Option<Ty>, span: Span) -> Option<Ty> {
        let inner = self.new_var(false);
        let want = match err {
            Some(e) => self.tys.mk_result(inner, e),
            None => self.tys.mk_option(inner),
        };
        if !self.unify(want, item) {
            self.mismatch(span, want, item);
            return None;
        }
        Some(inner)
    }

    /// `collect` into `Vec<Item>`, `Box<[Item]>`, or `String` (from
    /// `char`, `&char`, `&str`, or `String` items): the targets lust
    /// builds. A try target recurses on its inner collection with
    /// the item inside the `Result` or `Option` (F3).
    pub(super) fn collect_target_ok(&mut self, target: Ty, item: Ty, span: Span) -> bool {
        let t = self.resolve(target);
        if let Some((c, err)) = self.try_collect_parts(t) {
            return match self.try_collect_inner(item, err, span) {
                Some(inner) => self.collect_target_ok(c, inner, span),
                None => false,
            };
        }
        // alloc's `FromIterator<T> for Box<[T]>` collects a `Vec`
        // and boxes it (lowering mirrors that). The deep form: a
        // `Box::from_iter` target's element is a variable the
        // expected type already bound to a slice.
        let dt = self.deep(t);
        if let Some(el) = self.tys.boxed_slice_elem(dt) {
            if !self.unify(el, item) {
                self.mismatch(span, el, item);
                return false;
            }
            return true;
        }
        // A pointee no use fixed: rustc takes the one impl the item fits,
        // `[Item]` (`Box<str>` also fits the str items, so those stay
        // ambiguous, E0283).
        if let Some(k) = match self.tys.builtin_of(dt) {
            Some(BuiltinTy::Boxed) => Some(crate::check::own::Owner::Box),
            Some(BuiltinTy::Rc) => Some(crate::check::own::Owner::Rc),
            Some(BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc)) => Some(crate::check::own::Owner::Arc),
            _ => None,
        } && let Some(e) = self.tys.adt_first_ty_arg(dt)
            && matches!(self.kind(self.resolve(e)), TyKind::Infer(_))
        {
            let it_t = self.deep(item);
            let str_item = it_t == CHAR
                || it_t == STRING
                || matches!(self.kind(it_t), TyKind::Ref(_, Mut::Shared, x) if self.resolve(x) == CHAR || self.resolve(x) == STR)
                || crate::check::own::own_parts(&self.tys, it_t).is_some_and(|(_, e)| self.kind(e) == TyKind::Str)
                || crate::check::collect::cow_arg(&self.tys, it_t).is_some_and(|b| self.kind(b) == TyKind::Str);
            if k == crate::check::own::Owner::Box && str_item {
                self.err("L0610", span, "type annotations needed: the `collect` target type must be known here", "Annotate the target, such as `Box<[T]>` or `Box<str>`.");
                return false;
            }
            let sl = self.tys.intern(TyKind::Slice(item));
            self.unify(e, sl);
            return true;
        }
        // alloc's `FromIterator<T>` for `Rc<[T]>` and `Arc<[T]>`
        // (`ToRcSlice`, lowering mirrors it). `Rc<str>` and `Arc<str>`
        // have no impl (rustc E0277).
        if let Some((k, e)) = crate::check::own::own_parts(&self.tys, dt).filter(|&(k, _)| k != crate::check::own::Owner::Box) {
            let e = self.resolve(e);
            if let TyKind::Slice(el) = self.kind(e) {
                if !self.unify(el, item) {
                    self.mismatch(span, el, item);
                    return false;
                }
                return true;
            }
            if self.kind(e) == TyKind::Str {
                let n = if k == crate::check::own::Owner::Rc { "Rc<str>" } else { "Arc<str>" };
                let it_t = self.deep(item);
                let tn = self.tn(it_t);
                self.err("L0612", span, format!("a value of type `{n}` cannot be built from an iterator over `{tn}`"), "Collect into a `String`, then convert it with `.into()`.");
                return false;
            }
        }
        // alloc's `FromIterator` for `Box<str>`: a `String` collect,
        // then `into_boxed_str` (lowering mirrors it).
        if crate::check::own::own_parts(&self.tys, dt).is_some_and(|(k, e)| k == crate::check::own::Owner::Box && self.kind(e) == TyKind::Str) {
            return self.string_items_ok(item, span, "Box<str>");
        }
        if let Some(el) = self.tys.vec_elem(t) {
            if !self.unify(el, item) {
                let want = self.tys.mk_vec(item);
                self.mismatch(span, want, t);
                return false;
            }
            return true;
        }
        // `VecDeque<T>` collects like `Vec<T>` (see the collect arm).
        if let Some(a) = self.tys.local_adt(t).or_else(|| self.tys.adt_parts(t).and_then(|(d, _)| self.tys.local_adt_of(d)))
            && self.p.is_vec_deque_adt(a)
        {
            let el = self.tys.adt_first_ty_arg(t).unwrap_or(ERROR);
            if !self.unify(el, item) {
                self.mismatch(span, el, item);
                return false;
            }
            return true;
        }
        if self.tys.is_string_ty(t) {
            return self.string_items_ok(item, span, "String");
        }
        if let Some(ok) = self.hash_collect_target(t, item, span) {
            return ok;
        }
        if matches!(self.kind(t), TyKind::Infer(_)) {
            self.err("L0610", span, "type annotations needed: the `collect` target type must be known here", "Write `collect::<Vec<_>>()`, or annotate the binding.");
        } else {
            let tn = self.tn(t);
            self.err("L0640", span, format!("`collect` into `{tn}` is not implemented yet"), "Collect into a `Vec` or a `String`.");
        }
        false
    }

    /// Whether `item` is one of the `String` collect items lust builds
    /// (`char`, `&char`, `&str`, `String`), else L0612 naming `target`.
    fn string_items_ok(&mut self, item: Ty, span: Span, target: &str) -> bool {
        let it_t = self.deep(item);
        let ok = it_t == CHAR
            || it_t == STRING
            || matches!(self.kind(it_t), TyKind::Ref(_, Mut::Shared, x) if self.resolve(x) == CHAR || self.resolve(x) == STR);
        if !ok {
            let tn = self.tn(it_t);
            self.err("L0612", span, format!("a `{target}` cannot be built from an iterator over `{tn}`"), "Collect into a `Vec` instead.");
        }
        ok
    }

    /// Body end: a `sum` or `product` whose item or result type was open
    /// when it was checked (`deferred_sum`), now that later uses fixed
    /// them, or rustc's E0282/E0283 when one is still open.
    pub(super) fn finish_deferred_sums(&mut self) {
        for (target, item, sum, span) in std::mem::take(&mut self.deferred_sum) {
            let n = if sum { "sum" } else { "product" };
            let t = self.resolve(target);
            if self.sum_item_open(t, self.resolve(item)) {
                self.err(
                    "L0610",
                    span,
                    format!("type annotations needed: the item type of this `{n}` must be known here"),
                    "Annotate the element type of the source, such as `let v: Vec<u32> = Vec::new();`.",
                );
            } else {
                // A result fixed only as a literal (`.unwrap_or(0)`): an
                // integer variable matches every integer impl, so rustc
                // selects the one `Sum<Item>` impl whose type is the item's
                // (`u64: Sum<&u64>`); with none, literal fallback names it.
                let t = self.resolve(target);
                let tk = self.kind(t);
                if tk.is_int_infer() || tk.is_float_infer() {
                    let i = self.resolve(item);
                    let plain = match self.kind(i) {
                        TyKind::Ref(_, Mut::Shared, x) => self.resolve(x),
                        _ => i,
                    };
                    let fits = match self.kind(plain) {
                        TyKind::Int(_) => tk.is_int_infer(),
                        TyKind::Float(_) => tk.is_float_infer(),
                        _ => false,
                    };
                    if !(fits && self.unify(t, plain)) {
                        self.literal_fallback_ty(t);
                    }
                }
                self.sum_target_ok(target, item, n, span);
            }
        }
    }

    /// True when a `sum`'s item type picks no impl yet: it is open, or
    /// it is the `Option`/`Result` of an open type for a try target
    /// (`S: Sum<?U>` has two impls, `Sum<S>` and `Sum<&S>`, so rustc
    /// does not infer `U` from `S`: E0284).
    fn sum_item_open(&mut self, target: Ty, item: Ty) -> bool {
        if self.unbound_general(item) {
            return true;
        }
        if self.tys.option_elem(target).is_none() && self.tys.result_args(target).is_none() {
            return false;
        }
        let payload = self.tys.option_elem(item).or_else(|| self.tys.result_args(item).map(|(u, _)| u));
        payload.is_some_and(|u| {
            let u = self.resolve(u);
            self.unbound_general(u)
        })
    }

    /// `sum` and `product` into an integer or float type `S`, from
    /// items `S` or `&S` (Rust's `Sum<S>` and `Sum<&S>` impls), or into
    /// `Option<S>` / `Result<S, E>` from items `Option<U>` / `Result<U, E>`.
    fn sum_target_ok(&mut self, target: Ty, item: Ty, n: &str, span: Span) -> bool {
        let t = self.resolve(target);
        // `Option<T>: Sum<Option<U>>` and `Result<T, E>: Sum<Result<U, E>>`
        // (core's try_process) when `T: Sum<U>`.
        let i = self.resolve(item);
        let shunt = match (self.tys.option_elem(t), self.tys.result_args(t)) {
            (Some(tt), _) => Some((tt, self.tys.option_elem(i), None)),
            (_, Some((tt, te))) => Some((tt, self.tys.result_args(i).map(|r| r.0), self.tys.result_args(i).map(|r| (te, r.1)))),
            _ => None,
        };
        if let Some((tt, iu, errs)) = shunt {
            // The expansion sums one level (lower/iter.rs `sum_try`).
            let tt_r = self.resolve(tt);
            if self.tys.option_elem(tt_r).is_some() || self.tys.result_args(tt_r).is_some() {
                let tn = self.tn(t);
                self.err("L0640", span, format!("`{n}` into the nested `{tn}` is not implemented yet"), "Sum the inner values first.");
                return false;
            }
            let errs_ok = errs.is_none_or(|(a, b)| self.unify(a, b));
            let Some(iu) = iu.filter(|_| errs_ok) else {
                let (tn, iname) = (self.tn(t), self.tn(item));
                self.err("L0612", span, format!("`{tn}` cannot be the {n} of `{iname}` items"), format!("Map the items to `{tn}` first."));
                return false;
            };
            return self.sum_target_ok(tt, iu, n, span);
        }
        let numeric = |k: TyKind| matches!(k, TyKind::Int(_) | TyKind::Float(_));
        if !numeric(self.kind(t)) {
            if matches!(self.kind(t), TyKind::Infer(_)) {
                self.err("L0610", span, format!("type annotations needed: the `{n}` result type must be known here"), format!("Write `{n}::<i64>()`, or annotate the binding."));
            } else {
                let tn = self.tn(t);
                self.err("L0612", span, format!("`{tn}` cannot be the result of `{n}`"), "Sum into an integer or float type.");
            }
            return false;
        }
        let i = self.resolve(item);
        let plain = match self.kind(i) {
            TyKind::Ref(_, Mut::Shared, x) => self.resolve(x),
            _ => i,
        };
        if !self.unify(plain, t) {
            let tn = self.tn(t);
            let iname = self.tn(item);
            self.err("L0612", span, format!("`{tn}` cannot be the {n} of `{iname}` items"), format!("Map the items to `{tn}` first."));
            return false;
        }
        true
    }

    /// `Type::from_iter(arg)` by path, in every spelling rustc reads
    /// as `FromIterator::from_iter` (fx-fromiter; the trait is in the
    /// 2021+ prelude): a type path or alias, `<T>::`, `<T as
    /// FromIterator<..>>::`, and `FromIterator::from_iter` with the
    /// Self type from the expected type. The builtin targets build
    /// through the `collect` expansion (`Builtin::FromIter`), as
    /// rustc's `collect` is `from_iter`; the std source collections
    /// (`BTreeMap`, `BTreeSet`) call their source `from_iter`, as
    /// `collect` into them does. None when the callee is another
    /// form: a user type, an inherent fn, a type parameter, and a
    /// user trait's `from_iter` keep the ordinary path-call flow.
    pub(super) fn from_iter_call(&mut self, e: ExprId, callee: ExprId, args: &[ExprId], span: Span, expected: Option<Ty>) -> Option<Ty> {
        let p = self.p;
        let mut hint_item: Option<Ty> = None;
        let mut qpath_trait = false;
        let (found, turbo) = match p.ast.expr(callee) {
            Expr::QPath { self_ty, trait_path, segs } => {
                if segs.len() != 1 || self.name(segs[0].name) != "from_iter" {
                    return None;
                }
                let turbo = segs[0].clone();
                let base = self.resolve_written(*self_ty);
                if base == ERROR {
                    return None;
                }
                if !trait_path.is_empty() {
                    let tref = ast::TraitRefExpr { path: trait_path.clone() };
                    match self.written_trait_ref(base, &tref, span) {
                        Ok(tr) if tr.def == p.gx.lang.from_iterator => {
                            qpath_trait = true;
                            hint_item = self.tys.args(tr.args).get(1).and_then(|g| g.as_ty());
                            if let Some(t) = hint_item {
                                let t = self.instantiate_infers(t);
                                hint_item = Some(self.deep(t));
                            }
                        }
                        _ => return None,
                    }
                }
                (FromIterProbe::Found(base), turbo)
            }
            Expr::Path(segs) => {
                let segs = segs.clone();
                let n = segs.len();
                if n < 2 || self.name(segs[n - 1].name) != "from_iter" {
                    return None;
                }
                let turbo = segs[n - 1].clone();
                // `T::from_iter` on a type parameter (or a param named
                // like a std type) is the bound flow's, not this one's.
                if n == 2 && self.body_scope.param_tys.iter().any(|(nm, _)| *nm == segs[0].name) {
                    return None;
                }
                let prefix: Vec<String> = segs[..n - 1].iter().map(|s| self.name(s.name).to_string()).collect();
                let trait_form = prefix.len() == 1 && prefix[0] == "FromIterator"
                    || prefix.len() == 3
                        && matches!(prefix[0].as_str(), "std" | "core")
                        && prefix[1] == "iter"
                        && prefix[2] == "FromIterator";
                if trait_form {
                    let ta = segs[n - 2].type_args();
                    if ta.len() > 1 {
                        return None;
                    }
                    if let Some(t) = ta.first() {
                        let t = self.resolve_written(*t);
                        let t = self.instantiate_infers(t);
                        hint_item = Some(self.deep(t));
                    }
                    let exp = expected.map(|t| self.deep(t)).filter(|&b| b != ERROR && !self.unbound_general(b))?;
                    (FromIterProbe::Found(exp), turbo)
                } else {
                    (self.from_iter_ty_target(&segs, callee, span), turbo)
                }
            }
            _ => return None,
        };
        let target = match found {
            FromIterProbe::Found(t) => self.resolve(t),
            FromIterProbe::Bail => {
                for &a in args {
                    self.expr(a, None);
                }
                return Some(ERROR);
            }
            FromIterProbe::Miss => return None,
        };
        // Another in-scope trait with a `from_iter` method: rustc's
        // probe sees it next to std's (E0034 when both apply). Keep
        // the ordinary flow (and its L0643/L0720 answers) for that
        // case; a written `<T as FromIterator>::` names its trait.
        if !qpath_trait {
            let name = turbo.name;
            let other = p
                .gx
                .traits_by_method
                .get(&name)
                .is_some_and(|ds| ds.iter().any(|&d| d != p.gx.lang.from_iterator && method::trait_in_scope_pub(self, d)));
            if other {
                return None;
            }
        }
        // The expected result fixes the target's open args first (a
        // table's `S` most of all), as rustc's expected output does;
        // classification then sees the type as written there.
        if let Some(want) = expected {
            let want = self.resolve(want);
            if want != ERROR {
                let snap = self.snapshot_vars();
                if !self.unify(target, want) {
                    self.rollback_vars(snap);
                }
            }
        }
        let class = self.from_iter_class(target)?;
        // The item turbofish names the `IntoIterator` type itself
        // (`Vec::from_iter::<Vec<u32>>(v)`), as the method's own
        // generic parameter: exactly one type argument.
        let turbo_ty = if turbo.has_args() {
            let ta = turbo.type_args();
            if ta.len() != 1 {
                self.err("L0107", span, "this associated function takes 1 generic argument", "Write the iterator type.");
                for &a in args {
                    self.expr(a, None);
                }
                return Some(ERROR);
            }
            let t = self.resolve_written(ta[0]);
            Some(self.instantiate_infers(t))
        } else {
            None
        };
        let [a] = args else {
            self.err("L0105", span, "`from_iter` takes 1 argument", "Pass the iterator.");
            for &a in args {
                self.expr(a, None);
            }
            return Some(ERROR);
        };
        let aspan = self.span(*a);
        let at = self.expr(*a, None);
        let at = self.deep(at);
        if at == ERROR {
            return Some(ERROR);
        }
        if class == FromIterClass::Unsupported {
            let tn = self.tn(target);
            self.err(
                "L0640",
                span,
                format!("`from_iter` into `{tn}` is not implemented yet"),
                "Collect into a `Vec`, `String`, map, or set instead.",
            );
            return Some(ERROR);
        }
        if let Some(tt) = turbo_ty
            && !self.unify(at, tt)
        {
            self.mismatch(aspan, tt, at);
            return Some(ERROR);
        }
        // The argument through `IntoIterator`, as `extend` reads it.
        let it = self
            .p
            .collection_iter_inner(&mut self.tys, at)
            .or_else(|| it::as_iter_ty(&mut self.tys, at))
            .or_else(|| self.into_iterator_arg(*a, at));
        let Some(it) = it else {
            let tn = self.tn(at);
            self.err("L0640", aspan, format!("`from_iter` from `{tn}` is not implemented yet"), "Pass an iterator, a Vec, or a slice.");
            return Some(ERROR);
        };
        if it == ERROR {
            return Some(ERROR);
        }
        let item = self.iter_item(it);
        if let Some(hint) = hint_item
            && !self.unify(item, hint)
        {
            self.mismatch(span, hint, item);
            return Some(ERROR);
        }
        match class {
            FromIterClass::Collect => {
                if !self.collect_target_ok(target, item, span) {
                    return Some(ERROR);
                }
                self.set_res(callee, Res::Builtin(Builtin::FromIter));
                self.set_res(e, Res::Builtin(Builtin::FromIter));
                Some(self.resolve(target))
            }
            FromIterClass::BTree => Some(self.btree_from_iter(e, target, at, item, turbo.name, span)),
            FromIterClass::TryBTree => Some(self.try_btree_from_iter(e, target, at, item, turbo.name, span)),
            FromIterClass::Unsupported => unreachable!(),
        }
    }

    /// `BTreeMap::from_iter(arg)` and `BTreeSet::from_iter(arg)`: the
    /// std source impl's fn, selected at body end like `collect`
    /// into them (see the `collect` arm). The method's own generic
    /// is the argument's `IntoIterator` type as written.
    fn btree_from_iter(&mut self, e: ExprId, target: Ty, at: Ty, item: Ty, name: Sym, span: Span) -> Ty {
        let tr = self.resolve(target);
        let Some(a) = self.tys.local_adt(tr).or_else(|| self.tys.adt_parts(tr).and_then(|(d, _)| self.tys.local_adt_of(d))) else {
            return ERROR;
        };
        let path = self.p.display_path(self.p.adt(a).name);
        let targs: Vec<Ty> =
            self.tys.adt_parts(tr).map(|(_, x)| self.tys.args(x).iter().filter_map(|g| g.as_ty()).collect()).unwrap_or_default();
        let entry = match (path.rsplit("::").next(), targs.as_slice()) {
            (Some("BTreeMap"), [k, v]) => {
                let ts = self.tys.intern_tys(&[*k, *v]);
                Some(self.tys.intern(TyKind::Tuple(ts)))
            }
            (Some("BTreeSet"), [t]) => Some(*t),
            _ => None,
        };
        let Some(entry) = entry else { return ERROR };
        if !self.unify(entry, item) {
            self.mismatch(span, entry, item);
            return ERROR;
        }
        let targs = self.tys.intern_args(&[GenericArg::Ty(tr), GenericArg::Ty(item)]);
        let trait_ref = TraitRef { def: self.p.gx.lang.from_iterator, args: targs, bindings: EMPTY_BINDINGS };
        self.obl.push_trait_at(trait_ref, span, 0);
        method::defer_path_call(self, e, trait_ref, name, vec![GenericArg::Ty(at)]);
        self.record_pick(e, self.p.gx.lang.from_iterator, 0, ReceiverForm::Value);
        tr
    }

    /// `Result::<BTreeMap<..>, E>::from_iter(arg)` and kin (F3):
    /// core's generic try-collect impl, selected at body end like
    /// `btree_from_iter`. The entry type unifies with the item
    /// inside the `Result` or `Option`.
    fn try_btree_from_iter(&mut self, e: ExprId, target: Ty, at: Ty, item: Ty, name: Sym, span: Span) -> Ty {
        let tr = self.resolve(target);
        let Some((c, err)) = self.try_collect_parts(tr) else { return ERROR };
        let Some(inner) = self.try_collect_inner(item, err, span) else { return ERROR };
        let Some(a) = self.tys.local_adt(c).or_else(|| self.tys.adt_parts(c).and_then(|(d, _)| self.tys.local_adt_of(d))) else {
            return ERROR;
        };
        let path = self.p.display_path(self.p.adt(a).name);
        let targs: Vec<Ty> =
            self.tys.adt_parts(c).map(|(_, x)| self.tys.args(x).iter().filter_map(|g| g.as_ty()).collect()).unwrap_or_default();
        let entry = match (path.rsplit("::").next(), targs.as_slice()) {
            (Some("BTreeMap"), [k, v]) => {
                let ts = self.tys.intern_tys(&[*k, *v]);
                Some(self.tys.intern(TyKind::Tuple(ts)))
            }
            (Some("BTreeSet"), [t]) => Some(*t),
            _ => None,
        };
        let Some(entry) = entry else { return ERROR };
        if !self.unify(entry, inner) {
            self.mismatch(span, entry, inner);
            return ERROR;
        }
        let targs = self.tys.intern_args(&[GenericArg::Ty(tr), GenericArg::Ty(item)]);
        let trait_ref = TraitRef { def: self.p.gx.lang.from_iterator, args: targs, bindings: EMPTY_BINDINGS };
        self.obl.push_trait_at(trait_ref, span, 0);
        method::defer_path_call(self, e, trait_ref, name, vec![GenericArg::Ty(at)]);
        self.record_pick(e, self.p.gx.lang.from_iterator, 0, ReceiverForm::Value);
        tr
    }

    /// The target type of a type-relative `from_iter` path (not the
    /// trait-name form): an alias's type, the written type with its
    /// turbofish, or a bare std collection name at fresh arguments,
    /// as rustc infers them there. `Miss` keeps the ordinary flow.
    fn from_iter_ty_target(&mut self, segs: &[ast::PathSeg], callee: ExprId, span: Span) -> FromIterProbe {
        let p = self.p;
        let n = segs.len();
        let ty_seg = &segs[n - 2];
        let cspan = self.span(callee);
        if n == 2 && ty_seg.name == p.known.self_type {
            return match self.body_scope.self_ty {
                Some(t) if !matches!(self.kind(self.resolve(t)), TyKind::Param(_)) => FromIterProbe::Found(self.resolve(t)),
                _ => FromIterProbe::Miss,
            };
        }
        if n == 2 {
            if let Some(t) = self.alias_path_target(segs, cspan, false) {
                if t == ERROR {
                    return FromIterProbe::Miss;
                }
                // Like the alias flow in `call`: args on a plain alias
                // are rustc's E0107, and the type must be well-formed.
                if let Some((_, generic)) = self.alias_seg_ty(&segs[0], cspan, false)
                    && !generic
                    && segs[0].has_args()
                {
                    self.err("L0107", cspan, "this type alias takes no generic arguments", "Remove the `<..>` part.");
                    return FromIterProbe::Bail;
                }
                self.wf_check_written(t, cspan);
                return FromIterProbe::Found(t);
            }
            // An alias of a std source collection (`type BM =
            // BTreeMap<u32, u32>`): its args are the alias's.
            if let Some((a, aargs, _)) = self.alias_adt_seg(&segs[0], cspan) {
                if !p.is_std_adt(a) {
                    return FromIterProbe::Miss;
                }
                let generics = p.gx.adt_generics[a.0 as usize].clone();
                let args = if aargs.is_empty() { self.fresh_args(&generics) } else { aargs };
                return FromIterProbe::Found(self.tys.mk_adt(p.def_of_adt(a), &args));
            }
        }
        if ty_seg.has_args() {
            return match self.written_builtin_self(segs, span) {
                Some(t) if t == ERROR => FromIterProbe::Bail,
                Some(t) => FromIterProbe::Found(t),
                None => FromIterProbe::Miss,
            };
        }
        if n > 2 {
            let path: Vec<&str> = segs[..n - 1].iter().map(|s| self.name(s.name)).collect();
            if crate::ty::lang::std_path_kind(&path) != Some(crate::ty::lang::StdPathKind::Type) {
                return FromIterProbe::Miss;
            }
        }
        let (k, ty_name) = (&*p.known, ty_seg.name);
        if ty_name == k.Vec {
            let el = self.new_var(false);
            return FromIterProbe::Found(self.tys.mk_vec(el));
        }
        if ty_name == k.String {
            return FromIterProbe::Found(STRING);
        }
        // Targets with rustc `FromIterator` impls lust does not build
        // yet, so they reach the L0640 in `from_iter_call` instead of
        // the builtin tables' L0104 (a user type of the name shadows
        // std's, never on a std path, as in `resolve_assoc`).
        let lang_ty = n > 2 || !p.type_names.contains_key(&ty_name);
        if ty_name == k.Option && lang_ty {
            let el = self.new_var(false);
            return FromIterProbe::Found(self.tys.mk_option(el));
        }
        if ty_name == k.Result && lang_ty {
            let (a, b) = (self.new_var(false), self.new_var(false));
            return FromIterProbe::Found(self.tys.mk_result(a, b));
        }
        if ty_name == k.Box {
            let el = self.new_var(false);
            return FromIterProbe::Found(self.tys.mk_boxed(el));
        }
        if ty_name == k.Rc && lang_ty {
            let el = self.new_var(false);
            return FromIterProbe::Found(self.tys.mk_rc(el));
        }
        // The hashed collections arrive by their std path, as in
        // `hash_ctor_sig` (a bare name was not imported).
        if n > 2
            && let Some(hk) = crate::ty::hash::HashTy::by_name(self.name(ty_name))
            && matches!(hk, crate::ty::hash::HashTy::HashMap | crate::ty::hash::HashTy::HashSet)
        {
            let args: Vec<Ty> = (0..hk.n_args()).map(|_| self.new_var(false)).collect();
            return FromIterProbe::Found(crate::ty::hash::mk_hash(&mut self.tys, hk, &args));
        }
        // A std source collection by its imported or std-path name.
        if let BindingRes::Def(def) = self.binding(Namespace::Type, ty_name)
            && let Some(a) = p.adt_by_def(def)
            && p.is_std_adt(a)
        {
            let generics = p.gx.adt_generics[a.0 as usize].clone();
            let args = self.fresh_args(&generics);
            return FromIterProbe::Found(self.tys.mk_adt(def, &args));
        }
        FromIterProbe::Miss
    }

    /// What a concrete `from_iter` target builds through (see
    /// `from_iter_call`). None for a target this package does not
    /// claim: user types, type parameters, and types with no
    /// `FromIterator` impl in rustc keep the ordinary flow and its
    /// errors. `Unsupported` is a rustc impl lust has no build for.
    fn from_iter_class(&mut self, t: Ty) -> Option<FromIterClass> {
        let t = self.resolve(t);
        // A try target builds through its inner collection's class
        // (F3): builtin inners take the `Collect` expansion, BTree
        // inners core's generic try impl.
        if let Some((c, _)) = self.try_collect_parts(t) {
            return match self.from_iter_class(c) {
                Some(FromIterClass::BTree) => Some(FromIterClass::TryBTree),
                other => other,
            };
        }
        let dt = self.deep(t);
        if self.tys.vec_elem(t).is_some() || self.tys.boxed_slice_elem(dt).is_some() || self.tys.is_string_ty(t) {
            return Some(FromIterClass::Collect);
        }
        if let Some((hk, _)) = crate::ty::hash::hash_parts(&self.tys, t) {
            return match hk {
                crate::ty::hash::HashTy::HashMap | crate::ty::hash::HashTy::HashSet => Some(FromIterClass::Collect),
                _ => None,
            };
        }
        if let Some(a) = self.tys.local_adt(t).or_else(|| self.tys.adt_parts(t).and_then(|(d, _)| self.tys.local_adt_of(d))) {
            if self.p.is_vec_deque_adt(a) {
                return Some(FromIterClass::Collect);
            }
            if !self.p.is_std_adt(a) {
                return None;
            }
            let path = self.p.display_path(self.p.adt(a).name);
            return match path.rsplit("::").next() {
                Some("BTreeMap" | "BTreeSet") => Some(FromIterClass::BTree),
                Some("PathBuf" | "OsString" | "LinkedList" | "BinaryHeap") => Some(FromIterClass::Unsupported),
                _ => None,
            };
        }
        // `Box<str>`, `Rc<[T]>`, and `Arc<[T]>` collect (`Rc<str>` and
        // `Arc<str>` too: the collect check rejects them as rustc does);
        // a sized pointee has no impl (the ordinary flow's L0104).
        if crate::check::own::own_parts(&self.tys, dt).is_some_and(|(_, e)| matches!(self.kind(self.resolve(e)), TyKind::Slice(_) | TyKind::Str)) {
            return Some(FromIterClass::Collect);
        }
        (t == UNIT).then_some(FromIterClass::Unsupported)
    }

    /// A range expression used as a value: one of the six
    /// `std::ops` range types over an integer type or `char` (`for`
    /// loops over integers and slicing keep their own syntactic
    /// paths). The open-ended forms are `RangeBounds` values here,
    /// not iterators. A `char` range steps over the surrogate gap
    /// (lower/iter/char_range.rs).
    pub(super) fn range_value(&mut self, e: ExprId, lo: Option<ExprId>, hi: Option<ExprId>, inclusive: bool, span: Span) -> Ty {
        let _ = e;
        let bound_ty = |this: &mut Self, b: ExprId, want: Option<Ty>| -> Option<Ty> {
            let t = this.expr(b, want);
            let t = this.resolve(t);
            if matches!(this.kind(t), TyKind::Int(_) | TyKind::Char | TyKind::Error) || this.kind(t).is_int_infer() {
                Some(t)
            } else {
                let tn = this.tn(t);
                this.err("L0641", span, format!("ranges of `{tn}` are not supported yet"), "Use an integer range.");
                None
            }
        };
        match (lo, hi) {
            (Some(lo), Some(hi)) => {
                let Some(lt) = bound_ty(self, lo, None) else { return ERROR };
                let Some(ht) = bound_ty(self, hi, Some(lt)) else { return ERROR };
                if !self.unify(lt, ht) {
                    self.mismatch(self.span(hi), lt, ht);
                    return ERROR;
                }
                let k = if inclusive { IterKind::RangeInclusive } else { IterKind::Range };
                it::mk_iter(&mut self.tys, k, &[lt])
            }
            (Some(lo), None) => {
                let Some(lt) = bound_ty(self, lo, None) else { return ERROR };
                it::mk_iter(&mut self.tys, IterKind::RangeFrom, &[lt])
            }
            (None, Some(hi)) => {
                let Some(ht) = bound_ty(self, hi, None) else { return ERROR };
                let k = if inclusive { IterKind::RangeToInclusive } else { IterKind::RangeTo };
                it::mk_iter(&mut self.tys, k, &[ht])
            }
            (None, None) => it::mk_iter(&mut self.tys, IterKind::RangeFull, &[]),
        }
    }
}

/// Every method of core's `Iterator`, `DoubleEndedIterator`, and
/// `ExactSizeIterator` (rust-src 1.99). On a builtin iterator these go to
/// `iter_call`, which answers or rejects them; other names resolve normally.
const STD_ITER_TRAIT_METHODS: &[&str] = &[
    "advance_back_by", "advance_by", "all", "any", "array_chunks", "by_ref", "chain", "cloned", "cmp", "cmp_by", "collect",
    "collect_into", "copied", "count", "cycle", "enumerate", "eq", "eq_by", "filter", "filter_map", "find", "find_map", "flat_map",
    "flatten", "fold", "for_each", "fuse", "ge", "gt", "inspect", "intersperse", "intersperse_with", "is_empty", "is_partitioned",
    "is_sorted", "is_sorted_by", "is_sorted_by_key", "last", "le", "len", "lt", "map", "map_while", "map_windows", "max", "max_by",
    "max_by_key", "min", "min_by", "min_by_key", "ne", "next", "next_back", "next_chunk", "next_chunk_back", "nth", "nth_back",
    "partial_cmp", "partial_cmp_by", "partition", "partition_in_place", "peekable", "position", "product", "reduce", "rev", "rfind",
    "rfold", "rposition", "scan", "size_hint", "skip", "skip_while", "step_by", "sum", "take", "take_while", "try_collect", "try_find",
    "try_fold", "try_for_each", "try_reduce", "try_rfold", "unzip", "zip",
];

/// The methods `iter_call` answers that belong to an iterator type
/// itself, not to an iterator trait (`Peekable::peek`, `Chars::as_str`).
const ITER_VIEW_METHODS: &[&str] = &["peek", "next_if", "next_if_eq", "as_slice", "as_mut_slice", "as_str", "remainder", "into_remainder"];

/// The adapter and consumer methods `iter_call` answers, by name: the
/// calls that wrap a `ByNext` iterator.
const ITER_METHODS: &[&str] = &[
    "eq",
    "ne",
    "flatten",
    "flat_map",
    "map",
    "filter",
    "filter_map",
    "map_while",
    "enumerate",
    "rev",
    "take",
    "skip",
    "step_by",
    "chain",
    "zip",
    "copied",
    "cloned",
    "take_while",
    "skip_while",
    "peekable",
    "inspect",
    "count",
    "last",
    "nth",
    "any",
    "all",
    "find",
    "find_map",
    "position",
    "fold",
    "try_fold",
    "for_each",
    "min",
    "max",
    "min_by_key",
    "max_by_key",
    "collect",
    "partition",
    "sum",
    "product",
    "size_hint",
    "len",
];

/// The user fns a `ByNext` iterator calls: `next`, and `size_hint`
/// when the impl defines it. The impl must define no other method:
/// std's adapters call the inner iterator's own `fold`, `nth`,
/// `count`, and kin, which lust's expansions do not (they loop on
/// `next`), so an impl that overrides one of those is refused rather
/// than run differently. A generic impl is called at its instance
/// for the concrete type (`gx.iter_methods`, made in
/// check/generics.rs `ensure_iter_methods`).
pub struct ByNextFns {
    pub next: LocalDefId,
    pub size_hint: Option<LocalDefId>,
    /// `next_back` from the type's `DoubleEndedIterator` impl, if any.
    pub next_back: Option<LocalDefId>,
    /// The impl's `type Item`.
    pub item: Option<Ty>,
    /// The type has a written `ExactSizeIterator` impl with no
    /// `len` override and a `size_hint`: `len()` is std's default,
    /// which the expansion computes from that hint.
    pub exact: bool,
}

/// The `next` and `size_hint` items of one written `Iterator` impl,
/// or why the `ByNext` expansion cannot call it.
pub(crate) fn iter_impl_shape(p: &Program, rec: &crate::traits::index::ImplRecord) -> Result<(LocalDefId, Option<LocalDefId>), &'static str> {
    let (mut next, mut size_hint) = (None, None);
    for &d in &rec.items {
        let Some(l) = p.local_def(d.hash) else { continue };
        match p.name(p.fn_info(l).name) {
            "next" => next = Some(l),
            "size_hint" => size_hint = Some(l),
            _ => return Err("its `Iterator` impl defines a provided method"),
        }
    }
    Ok((next.ok_or("its `Iterator` impl has no `next`")?, size_hint))
}

/// True when impl Self pattern `pat` (read in the global store)
/// matches type `t` (read in `src`, the caller's store: a body
/// store's ids past the global ones are not global ids), binding
/// each pattern parameter in `binds` (by parameter index). Only the
/// shapes a written impl's Self type takes: parameters, Adts,
/// references, tuples, slices, and arrays over them.
pub(crate) fn match_self_pat(p: &Program, src: &dyn crate::ty::TyIntern, pat: Ty, t: Ty, binds: &mut Vec<Option<GenericArg>>) -> bool {
    match p.tys.kind(pat) {
        TyKind::Param(i) => {
            let i = i.0 as usize;
            if binds.len() <= i {
                binds.resize(i + 1, None);
            }
            match binds[i] {
                None => {
                    binds[i] = Some(GenericArg::Ty(t));
                    true
                }
                Some(GenericArg::Ty(u)) => u == t,
                _ => false,
            }
        }
        TyKind::Adt(d, a) => match src.kind(t) {
            TyKind::Adt(d2, a2) if d == d2 => {
                let (pa, ta) = (p.tys.args(a).to_vec(), src.args(a2).to_vec());
                pa.len() == ta.len() && pa.iter().zip(&ta).all(|(&g, &h)| match_arg_pat(p, src, g, h, binds))
            }
            _ => false,
        },
        TyKind::Ref(_, _, e) => match src.kind(t) {
            TyKind::Ref(_, _, e2) => match_self_pat(p, src, e, e2, binds),
            _ => false,
        },
        TyKind::Tuple(ts) => match src.kind(t) {
            TyKind::Tuple(ts2) => {
                let (pa, ta) = (p.tys.tys_of(ts).to_vec(), src.tys_of(ts2).to_vec());
                pa.len() == ta.len() && pa.iter().zip(&ta).all(|(&x, &y)| match_self_pat(p, src, x, y, binds))
            }
            _ => false,
        },
        TyKind::Slice(e) => match src.kind(t) {
            TyKind::Slice(e2) => match_self_pat(p, src, e, e2, binds),
            _ => false,
        },
        TyKind::Array(e, _) => match src.kind(t) {
            TyKind::Array(e2, _) => match_self_pat(p, src, e, e2, binds),
            _ => false,
        },
        // A leaf kind (a scalar, `str`, a raw pointer head): the ids
        // of plain kinds agree across stores.
        k => k == src.kind(t),
    }
}

/// One generic argument of a Self pattern against its concrete form.
fn match_arg_pat(p: &Program, src: &dyn crate::ty::TyIntern, pat: GenericArg, t: GenericArg, binds: &mut Vec<Option<GenericArg>>) -> bool {
    match (pat, t) {
        (GenericArg::Ty(x), GenericArg::Ty(y)) => match_self_pat(p, src, x, y, binds),
        (GenericArg::Lifetime(_), GenericArg::Lifetime(_)) => true,
        (GenericArg::Const(c), GenericArg::Const(c2)) => match p.tys.const_kind(c) {
            ConstKind::Param(i) => {
                let i = i as usize;
                if binds.len() <= i {
                    binds.resize(i + 1, None);
                }
                match binds[i] {
                    None => {
                        binds[i] = Some(GenericArg::Const(c2));
                        true
                    }
                    Some(GenericArg::Const(u)) => u == c2,
                    _ => false,
                }
            }
            _ => c == c2,
        },
        _ => false,
    }
}

/// `by_next_fns` for a user type `t` (read in `src`, the caller's
/// store), or why lust cannot iterate it through `next` yet. A
/// concrete Adt with made instances (`gx.iter_methods`) answers
/// with them; any other type (a template check, or a non-generic
/// impl) answers with the impl's own fns, found by matching the
/// impl's Self pattern against `t`.
/// The written `FromStr::from_str` fn for a concrete type (tf-str),
/// for `str::parse`'s expansion (lower/iter.rs), found as
/// `by_next_fns` finds a written `Iterator` impl. The primitive
/// types' synthetic impls have no fn here: the caller runs their
/// runtime parsers instead.
pub fn from_str_fn(p: &Program, src: &dyn crate::ty::TyIntern, t: Ty) -> Result<LocalDefId, &'static str> {
    let lang = p.gx.lang;
    for &id in p.gx.index.impls_of_trait(lang.from_str) {
        let r = p.gx.index.get(id);
        if r.synthetic != crate::traits::index::Synthetic::None {
            continue;
        }
        if match_self_pat(p, src, r.self_ty(&p.tys), t, &mut Vec::new()) {
            return r
                .items
                .iter()
                .find_map(|d| p.local_def(d.hash).filter(|&l| p.name(p.fn_info(l).name) == "from_str"))
                .ok_or("the `FromStr` impl has no `from_str` lust can call");
        }
    }
    Err("it has no written `FromStr` impl lust can call")
}

pub fn by_next_fns(p: &Program, src: &dyn crate::ty::TyIntern, t: Ty) -> Result<ByNextFns, &'static str> {
    let lang = p.gx.lang;
    if let Some(a) = src.local_adt(t)
        && let Some(&(next, size_hint, next_back, item, exact)) = p.gx.iter_methods.get(&a)
    {
        return Ok(ByNextFns { next, size_hint, next_back, item, exact });
    }
    let mut found = None;
    for &id in p.gx.index.impls_of_trait(lang.iterator) {
        let r = p.gx.index.get(id);
        if r.synthetic != crate::traits::index::Synthetic::None {
            continue;
        }
        let mut binds = Vec::new();
        if match_self_pat(p, src, r.self_ty(&p.tys), t, &mut binds) {
            found = Some((r, binds));
            break;
        }
    }
    let (rec, binds) = found.ok_or("it has no written `Iterator` impl lust can call")?;
    let (next, size_hint) = iter_impl_shape(p, rec)?;
    // The impl's Item pattern at `t`: a plain type as itself, a bare
    // parameter through its binding; anything richer is left to the
    // projection the check already normalized.
    let item = rec.assoc_ty(lang.assoc_item).and_then(|it| match p.tys.kind(it) {
        TyKind::Param(i) => binds.get(i.0 as usize).copied().flatten().and_then(|g| g.as_ty()),
        _ if !p.tys.flags(it).contains_param() => Some(it),
        _ => None,
    });
    // A `DoubleEndedIterator` impl for the same Self gives `next_back`.
    let next_back = p
        .gx
        .index
        .impls_of_trait(lang.double_ended_iterator)
        .iter()
        .map(|&id| p.gx.index.get(id))
        .find(|r| {
            r.synthetic == crate::traits::index::Synthetic::None && match_self_pat(p, src, r.self_ty(&p.tys), t, &mut Vec::new())
        })
        .and_then(|r| r.items.iter().find_map(|d| p.local_def(d.hash).filter(|&l| p.name(p.fn_info(l).name) == "next_back")));
    // An `ExactSizeIterator` impl for the same Self, with no `len`
    // override: `len()` is the trait default over `size_hint`.
    let exact = size_hint.is_some()
        && p.gx
            .index
            .impls_of_trait(lang.exact_size_iterator)
            .iter()
            .map(|&id| p.gx.index.get(id))
            .any(|r| {
                r.synthetic == crate::traits::index::Synthetic::None
                    && match_self_pat(p, src, r.self_ty(&p.tys), t, &mut Vec::new())
                    && !r.items.iter().any(|d| p.local_def(d.hash).is_some_and(|l| p.name(p.fn_info(l).name) == "len"))
            });
    Ok(ByNextFns { next, size_hint, next_back, item, exact })
}

/// The integer types core's `range_exact_iter_impl!` makes
/// `Range<T>: ExactSizeIterator`.
pub(super) fn exact_range_int(i: IntTy, inclusive: bool) -> bool {
    if inclusive {
        // core's `range_incl_exact_iter_impl!` list: a wider inclusive
        // range can hold `usize::MAX + 1` items.
        return matches!(i, IntTy::U8 | IntTy::U16 | IntTy::I8 | IntTy::I16);
    }
    matches!(i, IntTy::Usize | IntTy::U8 | IntTy::U16 | IntTy::U32 | IntTy::Isize | IntTy::I8 | IntTy::I16 | IntTy::I32)
}
