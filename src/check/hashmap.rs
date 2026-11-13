// The hashed collections in body checking (W-STD-B; ty/hash.rs,
// runtime/hash.h, cgen/hash.rs): `HashMap` and `HashSet` constructors
// and methods, and the std hashers. Bounds are proven through the
// obligation queue as the std impl blocks state them (`K: Eq + Hash`,
// `S: BuildHasher`, `K: Borrow<Q>`), so selection decides.
//
// Rule zero: a method Rust declares on these types that lust has no
// port of rejects with L0640 naming it.
use super::*;
use crate::ty::hash::{HashTy, hash_parts, mk_hash};

/// The methods of `core::hash::Hasher` lust declares, in record order:
/// `finish`, `write`, then `write_<int>` for every integer type.
pub const HASHER_METHODS: [&str; 14] = [
    "finish",
    "write",
    "write_u8",
    "write_u16",
    "write_u32",
    "write_u64",
    "write_u128",
    "write_usize",
    "write_i8",
    "write_i16",
    "write_i32",
    "write_i64",
    "write_i128",
    "write_isize",
];

/// The argument of a Hasher write method: `Some(Some(int))` for
/// `write_<int>`, `Some(None)` for `write(&[u8])`, None otherwise.
pub fn hasher_write_arg(name: &str) -> Option<Option<IntTy>> {
    if name == "write" {
        return Some(None);
    }
    let t = name.strip_prefix("write_")?;
    IntTy::ALL.iter().copied().find(|i| i.name() == t).map(Some)
}

/// HashMap and HashSet methods Rust declares that lust does not port
/// yet: a call rejects with L0640 naming the method.
/// True for `HashMap` and `HashSet` (not an Entry form).
fn table_kind(k: HashTy) -> bool {
    matches!(k, HashTy::HashMap | HashTy::HashSet)
}

const DECLARED_ONLY: &[&str] = &[
    "extract_if",
    "try_reserve",
    "shrink_to_fit",
    "shrink_to",
    "get_many_mut",
    "hasher",
    "union",
    "intersection",
    "difference",
    "symmetric_difference",
    "is_subset",
    "is_superset",
    "is_disjoint",
    "replace",
    "take",
    "get_or_insert",
    "get_or_insert_with",
];

impl<'p> Ctx<'p> {
    /// Pushes `t: trait_def` (one-parameter traits) to the queue.
    fn require(&mut self, trait_def: DefId, t: Ty, span: Span) {
        let args = self.tys.intern_args(&[GenericArg::Ty(t)]);
        self.obl.push_trait_at(TraitRef { def: trait_def, args, bindings: EMPTY_BINDINGS }, span, 0);
    }

    /// `K: Borrow<Q>`.
    fn require_borrow(&mut self, k: Ty, q: Ty, span: Span) {
        let args = self.tys.intern_args(&[GenericArg::Ty(k), GenericArg::Ty(q)]);
        self.obl.push_trait_at(TraitRef { def: self.p.gx.lang.borrow, args, bindings: EMPTY_BINDINGS }, span, 0);
    }

    /// The impl-block bounds of the hashing methods: `K: Eq + Hash`,
    /// `S: BuildHasher`.
    fn require_hash_key(&mut self, k: Ty, s: Ty, span: Span) {
        let lang = self.p.gx.lang;
        self.require(lang.eq, k, span);
        self.require(lang.hash, k, span);
        self.require(lang.build_hasher, s, span);
        self.deferred_hash_key.push((k, k, span));
    }

    /// A lookup by `&Q`: `K: Borrow<Q>`, `Q: Hash + Eq`, `S: BuildHasher`.
    fn require_lookup(&mut self, k: Ty, q: Ty, s: Ty, span: Span) {
        let lang = self.p.gx.lang;
        self.require_borrow(k, q, span);
        self.require(lang.hash, q, span);
        self.require(lang.eq, q, span);
        self.require(lang.build_hasher, s, span);
        self.deferred_hash_key.push((k, q, span));
    }

    /// The finish-time test of one key (or lookup) type, with types as
    /// bound by then: the hash glue hashes, compares, and borrows by
    /// structure (derives, core impls) and calls only the written impls
    /// of `gx.written_hash` and `gx.glue_methods`, so a key that reaches
    /// another written `Hash` or `PartialEq` impl (a generic one, say),
    /// or a lookup through a written `Borrow` impl the glue cannot call
    /// (not in `gx.borrow_impls`), rejects with L0640. A key that does
    /// not implement the traits at all is the obligation queue's L0612,
    /// already reported at this span when this runs (`finish`).
    pub(super) fn check_hash_key(&mut self, k: Ty, q: Ty, span: Span) {
        let (k, q) = (self.deep(k), self.deep(q));
        let (k, q) = (self.normalize_here(k), self.normalize_here(q));
        let unknown = |cx: &Self, t: Ty| matches!(cx.kind(t), TyKind::Error | TyKind::Infer(_)) || cx.tys.flags(t).contains_param();
        if unknown(self, k) || unknown(self, q) || self.errs.iter().any(|d| d.span == span) {
            return;
        }
        let p = self.p;
        let glue = |cx: &Self, t: Ty| has_hash(p, &cx.tys, t) && has_eq(p, &cx.tys, t);
        if crate::check::object::mentions_dyn(&self.tys, q) && !self.dyn_lookup_ok(k, q) {
            let tn = self.tn(q);
            self.errs.push(Diag::new(
                "L0640",
                span,
                format!("a map or set lookup by `{tn}` (a `dyn` type) is not supported yet: the hash glue cannot call its `Hash` and `PartialEq` impls"),
                "Look up by the key type itself.",
            ));
        } else if !(glue(self, q) || !crate::check::fmt::mentions_local_adt(&self.tys, q)) {
            let tn = self.tn(q);
            self.errs.push(Diag::new(
                "L0640",
                span,
                format!("a map or set key of type `{tn}` reaches a written `Hash` or `PartialEq` impl lust cannot call yet (a generic impl, an impl for a Box or reference, or an impl with its own `hash_slice`)"),
                "Derive Hash, PartialEq, and Eq on the key types, or write the impls for the plain types.",
            ));
        } else if !self.builtin_borrow(k, q) {
            let (kn, qn) = (self.tn(k), self.tn(q));
            self.errs.push(Diag::new(
                "L0640",
                span,
                format!("a lookup of `{kn}` keys by `{qn}` goes through a written `Borrow` impl lust cannot call yet (a generic impl, or one on a generic type)"),
                "Look up by the key type itself.",
            ));
        }
    }

    /// True when a lookup of `k` keys by the dyn type `q` goes through the
    /// glue's dyn key calls (`gx.dyn_keys`): `k`'s one dyn `Borrow` target
    /// is `q` (object lifetimes aside), with written `Hash` and `PartialEq`
    /// impls for it.
    fn dyn_lookup_ok(&mut self, k: Ty, q: Ty) -> bool {
        let q = crate::check::object::erase_dyn_region(&mut self.tys, q);
        self.tys.local_adt(k).and_then(|a| self.p.gx.dyn_keys.get(&a)).is_some_and(|dk| dk.q == q)
    }

    /// True when `K: Borrow<Q>` holds by one of core's impls the hash
    /// glue implements (`borrow_expr` in cgen/hash.rs), or by a written
    /// impl it calls (`gx.borrow_impls`).
    fn builtin_borrow(&self, k: Ty, q: Ty) -> bool {
        if k == q || self.tys.local_adt(k).is_some_and(|a| self.p.gx.borrow_impls.contains_key(&(a, q)) || self.p.gx.dyn_keys.get(&a).is_some_and(|dk| dk.q == q)) {
            return true;
        }
        let tys = &self.tys;
        match tys.kind(k) {
            TyKind::Ref(_, _, e) => return e == q,
            TyKind::Array(e, _) => return matches!(tys.kind(q), TyKind::Slice(x) if x == e),
            _ => {}
        }
        if tys.is_string_ty(k) {
            return q == STR;
        }
        if let Some(b) = crate::check::collect::cow_arg(tys, k) {
            return b == q;
        }
        if let Some(e) = tys.vec_elem(k) {
            return matches!(tys.kind(q), TyKind::Slice(x) if x == e);
        }
        tys.smart_elem(k) == Some(q)
    }

    /// True when `t` implements `trait_def` here: an impl (core's, a
    /// derive, or the user's) or a bound.
    pub(super) fn implements(&mut self, trait_def: DefId, t: Ty) -> bool {
        let args = self.tys.intern_args(&[GenericArg::Ty(t)]);
        let tr = TraitRef { def: trait_def, args, bindings: EMPTY_BINDINGS };
        matches!(self.select_obligation(tr), Selection::Impl { .. } | Selection::ParamEvidence(_))
    }

    /// The std hash trait `name` is in scope here for any receiver: imported
    /// by its std path or a glob of `std::hash`, or the trait of the impl
    /// this body is in (as rustc needs the trait in scope). A bound brings
    /// it in only for its own param (`method::trait_in_scope_at`); an
    /// instance still counts its template's bounds.
    pub(super) fn hash_trait_in_scope(&mut self, trait_def: DefId, name: &str) -> bool {
        self.std_trait_imported("hash", name)
            || self.std_glob("hash")
            || self.enclosing_impl_trait_pub() == Some(trait_def)
            || method::in_template_env(self, trait_def)
    }

    /// The written `BuildHasher` method `name` (`build_hasher`, or a
    /// written `hash_one`) of the impl `base` selects, instantiated
    /// at this call: (fn, params with the receiver first, return).
    /// None when the impl is synthetic (std's own builders) or does
    /// not write the method: the glue builtins answer those. A
    /// written `hash_one` must be called as written: rustc does not
    /// run the provided body over it.
    fn written_builder_method(&mut self, e: ExprId, base: Ty, name: &str, span: Span) -> Option<(LocalDefId, Vec<Ty>, Ty)> {
        let p = self.p;
        let args_id = self.tys.intern_args(&[GenericArg::Ty(base)]);
        let tr = TraitRef { def: p.gx.lang.build_hasher, args: args_id, bindings: EMPTY_BINDINGS };
        let Selection::Impl { id, args: impl_args, .. } = self.select_obligation(tr) else { return None };
        let rec = p.gx.index.get(id).clone();
        if rec.synthetic != crate::traits::index::Synthetic::None {
            return None;
        }
        if !rec.items.iter().any(|d| p.local_def(d.hash).is_some_and(|l| p.name(p.fn_info(l).name) == name)) {
            return None;
        }
        self.method_instance_for_impl(e, id, tr, impl_args, name, span)
    }

    /// `Type::build_hasher(&b)`, `Type::hash_one(&b, x)`, and the
    /// `<T as BuildHasher>::..` forms, from `assoc_call`: a written
    /// method is the impl's own fn; `hash_one` with no written one
    /// and std's builders are the glue builtins, whose operands are
    /// the path call's arguments as written. None for other calls.
    pub(super) fn builder_assoc_call(
        &mut self,
        e: ExprId,
        callee: ExprId,
        base: Ty,
        hint: Option<TraitRef>,
        name: Sym,
        args: &[ExprId],
        span: Span,
    ) -> Option<Ty> {
        use Builtin as B;
        let lang = self.p.gx.lang;
        let n = self.name(name).to_string();
        if !matches!(n.as_str(), "build_hasher" | "hash_one") || hint.is_some_and(|h| h.def != lang.build_hasher) {
            return None;
        }
        if hint.is_none() && !method::trait_in_scope_pub(self, lang.build_hasher) {
            return None;
        }
        let base = self.deep(base);
        if let Some((local, params, ret)) = self.written_builder_method(e, base, &n, span) {
            self.record_pick(e, lang.build_hasher, 0, ReceiverForm::Value);
            self.set_res(e, Res::Fn(local));
            self.check_args(&params, args, span);
            return Some(self.normalize_use(ret, span));
        }
        if !self.implements(lang.build_hasher, base) {
            return None;
        }
        let r = self.tys.mk_ref(false, base);
        match n.as_str() {
            "build_hasher" => {
                let h = match hash_parts(&self.tys, base) {
                    Some((HashTy::RandomState, _)) => mk_hash(&mut self.tys, HashTy::DefaultHasher, &[]),
                    Some((HashTy::BuildHasherDefault, a)) => a[0],
                    _ => return None,
                };
                self.set_res(callee, Res::Builtin(B::BuildHasher));
                self.set_res(e, Res::Builtin(B::BuildHasher));
                self.check_args(&[r], args, span);
                Some(h)
            }
            _ => {
                let x = self.new_var(false);
                self.set_res(callee, Res::Builtin(B::HashOne));
                self.set_res(e, Res::Builtin(B::HashOne));
                self.check_args(&[r, x], args, span);
                let x = self.deep(x);
                if args.len() == 2 {
                    let aspan = self.span(args[1]);
                    self.glue_hashable(x, aspan);
                }
                Some(int(IntTy::U64))
            }
        }
    }

    /// A value hashed by the glue (`x.hash(..)`, `hash_one(x)`): `T:
    /// Hash` goes to the queue, and at finish its Hash must be core's or
    /// derived all the way down (`glue_hash_ok`).
    pub(super) fn glue_hashable(&mut self, t: Ty, span: Span) {
        self.require(self.p.gx.lang.hash, t, span);
        self.deferred_glue_hash.push((t, span));
    }

    /// The finish-time test of a glue-hashed value: a written Hash impl
    /// inside that the glue cannot call (not in `gx.written_hash`) is
    /// L0640. A type parameter is checked in its instances.
    pub(super) fn glue_hash_ok(&mut self, t: Ty, span: Span) {
        let t = self.deep(t);
        let t = self.normalize_here(t);
        if self.tys.flags(t).contains_param() || matches!(self.kind(t), TyKind::Error | TyKind::Infer(_)) {
            return;
        }
        if self.implements(self.p.gx.lang.hash, t) && !has_hash(self.p, &self.tys, t) {
            let tn = self.tn(t);
            self.err("L0640", span, format!("hashing `{tn}`, which reaches a written `Hash` impl lust cannot call yet (a generic impl, an impl for a Box or reference, or an impl with its own `hash_slice`), is not implemented yet"), "Derive Hash instead, or write the impl for the plain type.");
        }
    }

    /// `x.hash(&mut state)` (Hash::hash for core and derived impls),
    /// `s.hash_one(x)` and `s.build_hasher()` (BuildHasher: a
    /// written method is the impl's own fn, std's builders and the
    /// provided `hash_one` are glue), and Hasher methods on a concrete
    /// hasher that lust's glue answers (DefaultHasher's, and the
    /// provided `write_*` of a user hasher; a written method is the
    /// impl's own fn, found before this). None for other calls.
    pub(super) fn hasher_method(&mut self, e: ExprId, base: Ty, derefs: u8, name: Sym, name_span: Span, args: &[ExprId]) -> Option<Ty> {
        use Builtin as B;
        let lang = self.p.gx.lang;
        let n = self.p.interner.get(name);
        let base = self.deep(base);
        match n {
            "hash" if args.len() == 1 && method::trait_in_scope_at(self, lang.hash, base) => {
                // The receiver must be Hash; the argument `&mut H`, H a hasher.
                if !self.tys.flags(base).contains_param() && !self.implements(lang.hash, base) {
                    return None;
                }
                // Typed against `&mut ?H`, so a `&mut H` variable reborrows.
                let hv = self.new_var(false);
                let want = self.tys.mk_ref(true, hv);
                self.check_args(&[want], args, name_span);
                let hty = self.deep(hv);
                if !self.implements(lang.hasher, hty) {
                    let tn = self.tn(hty);
                    self.err("L0612", self.span(args[0]), format!("`hash` takes `&mut H` with `H: Hasher`, not `&mut {tn}`"), "Pass `&mut` a hasher.");
                    return Some(ERROR);
                }
                self.glue_hashable(base, name_span);
                self.set_res(e, Res::BuiltinMethod { b: B::HashInto, derefs, autoref: Autoref::Ref });
                Some(UNIT)
            }
            "hash_one" if args.len() == 1 && method::trait_in_scope_at(self, lang.build_hasher, base) => {
                // A written `hash_one` runs as written; the glue is
                // only the provided body (fx-fromiter).
                if let Some((f, params, ret)) = self.written_builder_method(e, base, "hash_one", name_span) {
                    self.set_res(e, Res::Method { f, derefs, autoref: Autoref::Ref });
                    self.check_args(&params[1.min(params.len())..], args, name_span);
                    return Some(self.normalize_use(ret, name_span));
                }
                if !self.implements(lang.build_hasher, base) {
                    return None;
                }
                let at = self.expr(args[0], None);
                self.glue_hashable(at, self.span(args[0]));
                self.set_res(e, Res::BuiltinMethod { b: B::HashOne, derefs, autoref: Autoref::Ref });
                Some(int(IntTy::U64))
            }
            // A user builder's written `build_hasher` is its own fn
            // (fx-fromiter; the trait probe does not see the lang
            // trait here); std's builders are the glue: RandomState
            // builds a DefaultHasher, BuildHasherDefault<H> an
            // `H::default()`.
            "build_hasher" if args.is_empty() && method::trait_in_scope_at(self, lang.build_hasher, base) => {
                if let Some((f, params, ret)) = self.written_builder_method(e, base, "build_hasher", name_span) {
                    self.set_res(e, Res::Method { f, derefs, autoref: Autoref::Ref });
                    self.check_args(&params[1.min(params.len())..], args, name_span);
                    return Some(self.normalize_use(ret, name_span));
                }
                let h = match hash_parts(&self.tys, base) {
                    Some((HashTy::RandomState, _)) => mk_hash(&mut self.tys, HashTy::DefaultHasher, &[]),
                    Some((HashTy::BuildHasherDefault, a)) if self.implements(lang.build_hasher, base) => a[0],
                    _ => return None,
                };
                self.set_res(e, Res::BuiltinMethod { b: B::BuildHasher, derefs, autoref: Autoref::Ref });
                Some(h)
            }
            m if HASHER_METHODS.contains(&m) && method::trait_in_scope_at(self, lang.hasher, base) => {
                // A concrete hasher only: a bound's methods are its stubs.
                if self.tys.flags(base).contains_param() || !self.implements(lang.hasher, base) {
                    return None;
                }
                let (autoref, params, ret) = match hasher_write_arg(m) {
                    None => (Autoref::Ref, vec![], int(IntTy::U64)),
                    Some(None) => {
                        let s = self.tys.intern(TyKind::Slice(int(IntTy::U8)));
                        (Autoref::RefMut, vec![self.tys.mk_ref(false, s)], UNIT)
                    }
                    Some(Some(i)) => (Autoref::RefMut, vec![int(i)], UNIT),
                };
                self.set_res(e, Res::BuiltinMethod { b: B::HasherCall, derefs, autoref });
                self.check_args(&params, args, name_span);
                Some(ret)
            }
            _ => None,
        }
    }

    /// Methods on a HashMap, HashSet, or Entry receiver. None when the
    /// base is none of these, so other dispatch runs.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn hash_coll_method(
        &mut self,
        e: ExprId,
        recv: ExprId,
        rt: Ty,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
    ) -> Option<Ty> {
        use Builtin as B;
        let (kind, targs) = hash_parts(&self.tys, base)?;
        if !matches!(kind, HashTy::HashMap | HashTy::HashSet | HashTy::Entry | HashTy::OccupiedEntry | HashTy::VacantEntry) {
            return None;
        }
        let n = self.p.interner.get(name);
        if table_kind(kind)
            && matches!(n, "iter" | "iter_mut" | "keys" | "values" | "values_mut" | "into_iter" | "into_keys" | "into_values" | "drain")
            && let Some(t) = self.hash_iter_source(e, recv, rt, kind, &targs, derefs, n, name_span, args)
        {
            return Some(t);
        }
        let (k, v) = (targs[0], if kind == HashTy::HashSet { UNIT } else { targs[1] });
        let s = *targs.last().unwrap_or(&UNIT);
        let lookup = |cx: &mut Self| {
            let q = cx.new_var(false);
            let r = cx.tys.mk_ref(false, q);
            (q, r)
        };
        let table = matches!(kind, HashTy::HashMap | HashTy::HashSet);
        let found: Option<(B, Autoref, Vec<Ty>, Ty, Option<Ty>)> = match (kind, n) {
            (_, "len") if table => Some((B::HashLen, Autoref::Ref, vec![], USIZE, None)),
            (_, "is_empty") if table => Some((B::HashIsEmpty, Autoref::Ref, vec![], BOOL, None)),
            (_, "clear") if table => Some((B::HashClear, Autoref::RefMut, vec![], UNIT, None)),
            (_, "capacity") if table => Some((B::HashCapacity, Autoref::Ref, vec![], USIZE, None)),
            (_, "reserve") if table => Some((B::HashReserve, Autoref::RefMut, vec![USIZE], UNIT, None)),
            (_, "extend") if table => return Some(self.hash_extend(e, recv, kind, &targs, derefs, name_span, args)),
            (HashTy::HashMap, "insert") => {
                self.require_hash_key(k, s, name_span);
                let ov = self.tys.mk_option(v);
                Some((B::MapInsert, Autoref::RefMut, vec![k, v], ov, None))
            }
            // std's `F: FnMut(&K, &mut V) -> bool` and `F: FnMut(&T) ->
            // bool`, the references higher-ranked; no key bounds.
            (HashTy::HashMap, "retain") => {
                let (rk, rv) = (self.tys.mk_ref(false, k), self.tys.mk_ref(true, v));
                let f = self.fn_mut_param_ref(&[rk, rv], BOOL, name_span);
                Some((B::MapRetain, Autoref::RefMut, vec![f], UNIT, None))
            }
            (HashTy::HashSet, "retain") => {
                let rk = self.tys.mk_ref(false, k);
                let f = self.fn_mut_param_ref(&[rk], BOOL, name_span);
                Some((B::SetRetain, Autoref::RefMut, vec![f], UNIT, None))
            }
            (HashTy::HashMap, "get" | "get_mut" | "get_key_value" | "contains_key" | "remove" | "remove_entry") => {
                let (q, rq) = lookup(self);
                let (b, autoref, ret) = match n {
                    "get" => {
                        let r = self.tys.mk_ref(false, v);
                        (B::MapGet, Autoref::Ref, self.tys.mk_option(r))
                    }
                    "get_mut" => {
                        let r = self.tys.mk_ref(true, v);
                        (B::MapGetMut, Autoref::RefMut, self.tys.mk_option(r))
                    }
                    "contains_key" => (B::MapContainsKey, Autoref::Ref, BOOL),
                    "get_key_value" => {
                        let (rk, rv) = (self.tys.mk_ref(false, k), self.tys.mk_ref(false, v));
                        let ts = self.tys.intern_tys(&[rk, rv]);
                        let kv = self.tys.intern(TyKind::Tuple(ts));
                        (B::MapGetKeyValue, Autoref::Ref, self.tys.mk_option(kv))
                    }
                    "remove_entry" => {
                        let ts = self.tys.intern_tys(&[k, v]);
                        let kv = self.tys.intern(TyKind::Tuple(ts));
                        (B::MapRemoveEntry, Autoref::RefMut, self.tys.mk_option(kv))
                    }
                    _ => (B::MapRemove, Autoref::RefMut, self.tys.mk_option(v)),
                };
                Some((b, autoref, vec![rq], ret, Some(q)))
            }
            (HashTy::HashMap, "entry") => {
                self.require_hash_key(k, s, name_span);
                let en = mk_hash(&mut self.tys, HashTy::Entry, &[k, v]);
                Some((B::MapEntry, Autoref::RefMut, vec![k], en, None))
            }
            (HashTy::HashSet, "insert") => {
                self.require_hash_key(k, s, name_span);
                Some((B::SetInsert, Autoref::RefMut, vec![k], BOOL, None))
            }
            (HashTy::HashSet, "contains" | "remove" | "get") => {
                let (q, rq) = lookup(self);
                let (b, autoref, ret) = match n {
                    "contains" => (B::SetContains, Autoref::Ref, BOOL),
                    "remove" => (B::SetRemove, Autoref::RefMut, BOOL),
                    _ => {
                        let r = self.tys.mk_ref(false, k);
                        (B::SetGet, Autoref::Ref, self.tys.mk_option(r))
                    }
                };
                Some((b, autoref, vec![rq], ret, Some(q)))
            }
            (HashTy::Entry, "or_insert") => {
                let r = self.tys.mk_ref(true, v);
                Some((B::EntryOrInsert, Autoref::None, vec![v], r, None))
            }
            (HashTy::Entry, "or_default") => {
                self.require(self.p.gx.lang.default, v, name_span);
                let r = self.tys.mk_ref(true, v);
                Some((B::EntryOrDefault, Autoref::None, vec![], r, None))
            }
            (HashTy::Entry, "key") => {
                let r = self.tys.mk_ref(false, k);
                Some((B::EntryKey, Autoref::Ref, vec![], r, None))
            }
            // The closure forms take std's `FnOnce` bound: the expansion
            // calls an `FnOnce` closure by value (lower/iter.rs).
            (HashTy::Entry, "or_insert_with") => {
                let f = self.fn_once_param(&[], v, name_span);
                let r = self.tys.mk_ref(true, v);
                Some((B::EntryOrInsertWith, Autoref::None, vec![f], r, None))
            }
            (HashTy::Entry, "or_insert_with_key") => {
                let rk = self.tys.mk_ref(false, k);
                let f = self.fn_once_param(&[rk], v, name_span);
                let r = self.tys.mk_ref(true, v);
                Some((B::EntryOrInsertWithKey, Autoref::None, vec![f], r, None))
            }
            (HashTy::Entry, "and_modify") => {
                let rv = self.tys.mk_ref(true, v);
                let f = self.fn_once_param(&[rv], UNIT, name_span);
                Some((B::EntryAndModify, Autoref::None, vec![f], base, None))
            }
            (HashTy::Entry, "insert_entry") => {
                let occ = mk_hash(&mut self.tys, HashTy::OccupiedEntry, &[k, v]);
                Some((B::EntryInsertEntry, Autoref::None, vec![v], occ, None))
            }
            // `VacantEntry` (the payload of `Entry::Vacant`): the same steps
            // as the Entry's vacant branch.
            (HashTy::VacantEntry, "insert") => {
                let r = self.tys.mk_ref(true, v);
                Some((B::EntryInsert, Autoref::None, vec![v], r, None))
            }
            (HashTy::VacantEntry, "into_key") => Some((B::VacIntoKey, Autoref::None, vec![], k, None)),
            (HashTy::VacantEntry, "key") => Some((B::EntryKey, Autoref::Ref, vec![], self.tys.mk_ref(false, k), None)),
            (HashTy::VacantEntry, "insert_entry") => {
                let occ = mk_hash(&mut self.tys, HashTy::OccupiedEntry, &[k, v]);
                Some((B::EntryInsertEntry, Autoref::None, vec![v], occ, None))
            }
            (HashTy::OccupiedEntry, "key") => Some((B::OccKey, Autoref::Ref, vec![], self.tys.mk_ref(false, k), None)),
            (HashTy::OccupiedEntry, "get") => Some((B::OccGet, Autoref::Ref, vec![], self.tys.mk_ref(false, v), None)),
            (HashTy::OccupiedEntry, "get_mut") => Some((B::OccGetMut, Autoref::RefMut, vec![], self.tys.mk_ref(true, v), None)),
            (HashTy::OccupiedEntry, "into_mut") => Some((B::OccIntoMut, Autoref::None, vec![], self.tys.mk_ref(true, v), None)),
            (HashTy::OccupiedEntry, "insert") => Some((B::OccInsert, Autoref::RefMut, vec![v], v, None)),
            (HashTy::OccupiedEntry, "remove") => Some((B::OccRemove, Autoref::None, vec![], v, None)),
            (HashTy::OccupiedEntry, "remove_entry") => {
                let kv = self.tys.intern_tys(&[k, v]);
                Some((B::OccRemoveEntry, Autoref::None, vec![], self.tys.intern(TyKind::Tuple(kv)), None))
            }
            _ => None,
        };
        let Some((b, autoref, params, ret, q)) = found else {
            for &a in args {
                self.expr(a, None);
            }
            let tn = self.tn(base);
            if DECLARED_ONLY.contains(&n) {
                self.errs.push(Diag::new(
                    "L0640",
                    name_span,
                    format!("`{tn}::{n}` is not implemented in lust yet"),
                    "Rewrite this part with the supported subset.",
                ));
            } else {
                self.err("L0104", name_span, format!("no method `{n}` on type `{tn}`"), "Check the method name.");
            }
            return Some(ERROR);
        };
        if autoref != Autoref::None {
            self.check_packed_borrow(recv, name_span);
        }
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref });
        self.check_args(&params, args, name_span);
        if matches!(b, B::MapRetain | B::SetRetain) && self.adapter_callable_gap(args, &params, false) {
            return Some(ERROR);
        }
        if matches!(b, B::EntryOrInsertWith | B::EntryOrInsertWithKey | B::EntryAndModify) && self.adapter_callable_gap(args, &params, true) {
            return Some(ERROR);
        }
        if let Some(q) = q {
            let q = self.deep(q);
            self.require_lookup(k, q, s, name_span);
        }
        Some(ret)
    }

    /// `map[&q]`: std's `Index<&Q> for HashMap<K, V, S>` with `get`'s
    /// bounds (`K: Borrow<Q>`, `Q: Hash + Eq`, `S: BuildHasher`); the
    /// result is the value place. There is no `IndexMut`: lowering reads
    /// through `get`'s shared reference, so a write or a move out is the
    /// borrow checker's error, as rustc's E0594 and E0507.
    pub(super) fn map_index(&mut self, e: ExprId, base: ExprId, index: ExprId, targs: &[Ty], derefs: u8, span: Span) -> Ty {
        let (k, v, s) = (targs[0], targs[1], targs[2]);
        let q = self.new_var(false);
        let rq = self.tys.mk_ref(false, q);
        self.check_packed_borrow(base, span);
        self.check_args(&[rq], &[index], span);
        let q = self.deep(q);
        self.require_lookup(k, q, s, span);
        self.set_res(e, Res::MapIndex { derefs });
        v
    }

    /// The iterator sources of a map or set: `iter` and kin through an
    /// autoref, `into_keys`/`into_values` by value, and `into_iter` by
    /// Rust's probe (the first autoderef step with an `IntoIterator`
    /// impl: `&X` and `&mut X` borrow, `X` itself is consumed). None
    /// for a name the kind does not have.
    #[allow(clippy::too_many_arguments)]
    fn hash_iter_source(
        &mut self,
        e: ExprId,
        recv: ExprId,
        rt: Ty,
        kind: HashTy,
        targs: &[Ty],
        derefs: u8,
        n: &str,
        span: Span,
        args: &[ExprId],
    ) -> Option<Ty> {
        use crate::ty::iter::{IterKind as K, mk_iter};
        let set = kind == HashTy::HashSet;
        let iargs: Vec<Ty> = if set { vec![targs[0]] } else { vec![targs[0], targs[1]] };
        let (ik, autoref, derefs) = match n {
            "iter" if set => (K::SetIter, Autoref::Ref, derefs),
            "iter" => (K::MapIter, Autoref::Ref, derefs),
            "iter_mut" if !set => (K::MapIterMut, Autoref::RefMut, derefs),
            "keys" if !set => (K::MapKeys, Autoref::Ref, derefs),
            "values" if !set => (K::MapValues, Autoref::Ref, derefs),
            "values_mut" if !set => (K::MapValuesMut, Autoref::RefMut, derefs),
            // hashbrown's `drain` (R20): the `&mut self` loan lasts as long
            // as the `Drain`.
            "drain" => (if set { K::SetDrain } else { K::MapDrain }, Autoref::RefMut, derefs),
            "into_keys" | "into_values" if !set && derefs == 0 => {
                (if n == "into_keys" { K::MapIntoKeys } else { K::MapIntoValues }, Autoref::None, 0)
            }
            "into_iter" => {
                let mut t = self.resolve(rt);
                let mut found = None;
                for j in 0..=derefs {
                    match self.kind(t) {
                        TyKind::Ref(_, m, inner) => {
                            let inner = self.resolve(inner);
                            if hash_parts(&self.tys, inner).is_some_and(|(k, _)| k == kind) {
                                // There is no `IntoIterator for &mut HashSet`.
                                if !(set && m.is_mut()) {
                                    let ik = if set {
                                        K::SetIter
                                    } else if m.is_mut() {
                                        K::MapIterMut
                                    } else {
                                        K::MapIter
                                    };
                                    found = Some((ik, Autoref::None, j));
                                }
                                break;
                            }
                            t = inner;
                        }
                        _ if j == 0 && hash_parts(&self.tys, t).is_some_and(|(k, _)| k == kind) => {
                            found = Some((if set { K::SetIntoIter } else { K::MapIntoIter }, Autoref::None, 0));
                            break;
                        }
                        _ => break,
                    }
                }
                found?
            }
            _ if matches!(n, "into_keys" | "into_values") && !set => {
                self.err("L0640", span, format!("`{n}` through a reference is not implemented yet"), "Call it on the map value itself.");
                for &a in args {
                    self.expr(a, None);
                }
                return Some(ERROR);
            }
            _ => return None,
        };
        let ret = mk_iter(&mut self.tys, ik, &iargs);
        if autoref != Autoref::None {
            self.check_packed_borrow(recv, span);
        }
        self.set_res(e, Res::BuiltinMethod { b: Builtin::HashIterNew, derefs, autoref });
        self.check_args(&[], args, span);
        Some(ret)
    }

    /// `map.extend(iter)` (`Extend<(K, V)>`) and `set.extend(iter)`
    /// (`Extend<T>`) over anything `as_iter_ty` turns into a builtin
    /// iterator, or any other iterator through its `next` (`ByNext`);
    /// also the `&`-item forms (`Extend<(&K, &V)>`, `Extend<&T>`) for
    /// Copy parts.
    #[allow(clippy::too_many_arguments)]
    fn hash_extend(&mut self, e: ExprId, recv: ExprId, kind: HashTy, targs: &[Ty], derefs: u8, span: Span, args: &[ExprId]) -> Ty {
        let [a] = args else {
            self.err("L0105", span, "`extend` takes 1 argument", "Pass the iterator.");
            for &a in args {
                self.expr(a, None);
            }
            return ERROR;
        };
        let at = self.expr(*a, None);
        let at = self.deep(at);
        let Some(it) = crate::ty::iter::as_iter_ty(&mut self.tys, at).or_else(|| self.wrapper_iter_arg(at)).or_else(|| self.into_iterator_arg(*a, at)) else {
            let tn = self.tn(at);
            self.err("L0640", self.span(*a), format!("`extend` from `{tn}` is not implemented yet"), "Pass an iterator, a Vec, or a map.");
            return ERROR;
        };
        if it == ERROR {
            return ERROR;
        }
        let item = crate::ty::iter::item_ty(&mut self.tys, it).unwrap_or(ERROR);
        let item_d = self.deep(item);
        if self.extend_item_open(item_d, *a) {
            return ERROR;
        }
        let (k, s) = (targs[0], *targs.last().unwrap_or(&UNIT));
        let want = if kind == HashTy::HashMap {
            let ts = self.tys.intern_tys(&[targs[0], targs[1]]);
            self.tys.intern(TyKind::Tuple(ts))
        } else {
            k
        };
        // Both forms fit an open element: a later use decides
        // (check/extend.rs `finish_deferred_extends`).
        let map = kind == HashTy::HashMap;
        if matches!(self.extend_pick(want, item, map), super::extend::ExtendPick::Both) {
            let d = super::extend::DeferredExtend { elem: want, item, map, by_ref: true, span: self.span(*a) };
            self.deferred_extend.push(d);
            self.require_hash_key(k, s, span);
            self.check_packed_borrow(recv, span);
            self.set_res(e, Res::BuiltinMethod { b: Builtin::HashExtend, derefs, autoref: Autoref::RefMut });
            return UNIT;
        }
        // std's `Extend<&T>` for HashSet<T: Copy> and `Extend<(&K, &V)>`
        // for HashMap<K: Copy, V: Copy> (W-STD-B): the items copied in.
        let by_ref = {
            let it_t = self.deep(item);
            let refs: Option<Vec<Ty>> = if kind == HashTy::HashMap {
                match self.kind(it_t) {
                    TyKind::Tuple(ts) => {
                        let ts = self.tys.tys_of(ts).to_vec();
                        match ts[..] {
                            [a, b] => match (self.kind(a), self.kind(b)) {
                                (TyKind::Ref(_, Mut::Shared, x), TyKind::Ref(_, Mut::Shared, y)) => Some(vec![x, y]),
                                _ => None,
                            },
                            _ => None,
                        }
                    }
                    _ => None,
                }
            } else {
                match self.kind(it_t) {
                    TyKind::Ref(_, Mut::Shared, x) => Some(vec![x]),
                    _ => None,
                }
            };
            refs.filter(|r| {
                let want_parts: Vec<Ty> = if kind == HashTy::HashMap { vec![targs[0], targs[1]] } else { vec![k] };
                let snap = self.snapshot_vars();
                let ok = r.iter().zip(&want_parts).all(|(&x, &w)| self.unify(x, w));
                if !ok {
                    self.rollback_vars(snap);
                }
                ok
            })
        };
        if let Some(parts) = by_ref {
            for p in parts {
                self.deferred_copy.push((p, self.span(*a)));
            }
        } else if !self.unify(want, item) {
            let it_t = self.deep(item);
            if matches!(self.kind(it_t), TyKind::Ref(..)) || matches!(self.kind(it_t), TyKind::Tuple(_)) {
                let tn = self.tn(it_t);
                self.err(
                    "L0640",
                    self.span(*a),
                    format!("`extend` from items of type `{tn}` is not implemented yet"),
                    "Extend with owned items (`.copied()` or `.cloned()`).",
                );
            } else {
                self.mismatch(self.span(*a), want, it_t);
            }
            return ERROR;
        }
        self.require_hash_key(k, s, span);
        self.check_packed_borrow(recv, span);
        self.set_res(e, Res::BuiltinMethod { b: Builtin::HashExtend, derefs, autoref: Autoref::RefMut });
        UNIT
    }

    /// `collect` into a HashMap (`FromIterator<(K, V)>`) or a HashSet
    /// (`FromIterator<T>`): `S: BuildHasher + Default`. None when the
    /// target is neither.
    pub(super) fn hash_collect_target(&mut self, t: Ty, item: Ty, span: Span) -> Option<bool> {
        let (kind, targs) = hash_parts(&self.tys, t)?;
        let want = match kind {
            HashTy::HashMap => {
                let ts = self.tys.intern_tys(&[targs[0], targs[1]]);
                self.tys.intern(TyKind::Tuple(ts))
            }
            HashTy::HashSet => targs[0],
            _ => return None,
        };
        if !self.unify(want, item) {
            let it_t = self.deep(item);
            self.mismatch(span, want, it_t);
            return Some(false);
        }
        let s = *targs.last().unwrap_or(&UNIT);
        self.require(self.p.gx.lang.default, s, span);
        self.require_hash_key(targs[0], s, span);
        Some(true)
    }

    /// The signature of a hash type constructor (`HashMap::new` and
    /// kin), from the path's written args or fresh ones, with the std
    /// impl blocks' constraints (`new` and `with_capacity` are on
    /// `S = RandomState`).
    pub(super) fn hash_ctor_sig(&mut self, b: Builtin, callee: ExprId, span: Span) -> Option<(Vec<Ty>, Ty)> {
        use Builtin as B;
        // Through an alias (`type FxHashMap<K, V> = HashMap<K, V,
        // BuildHasherDefault<FxHasher>>; FxHashMap::default()`) or `<T>::`
        // the written type names the table and gives its args
        // (check/alias_path.rs).
        let (segs, written_ty) = match self.p.ast.expr(callee) {
            Expr::Path(segs) if segs.len() >= 2 => (segs.clone(), self.builtin_alias_ty(segs, span, false)),
            Expr::QPath { self_ty, segs, .. } => (segs.clone(), Some(self.resolve_written(*self_ty))),
            _ => return None,
        };
        let (k, written) = match written_ty {
            Some(t) => match self.tys.builtin_of(t) {
                Some(BuiltinTy::Hash(k)) => (k, self.builtin_self_args(t, k.n_args())),
                _ => return None,
            },
            None => {
                let ty_seg = &segs[segs.len() - 2];
                let k = HashTy::by_name(self.p.interner.get(ty_seg.name))?;
                (k, ty_seg.type_args().iter().map(|&t| self.resolve_written(t)).collect())
            }
        };
        let mut args: Vec<Ty> = (0..k.n_args()).map(|i| written.get(i).copied().unwrap_or_else(|| self.new_var(false))).collect();
        let rs = mk_hash(&mut self.tys, HashTy::RandomState, &[]);
        let fixed_rs = matches!(b, B::HashNew | B::HashWithCapacity) && matches!(k, HashTy::HashMap | HashTy::HashSet);
        if fixed_rs && let Some(last) = args.last_mut() {
            if !self.unify(*last, rs) {
                let tn = self.tn(*last);
                let f = self.p.interner.get(segs[segs.len() - 1].name);
                let what = if k == HashTy::HashSet { "set" } else { "map" };
                self.err(
                    "L0612",
                    span,
                    format!("`{f}` builds a {what} with `RandomState`, not `{tn}`"),
                    "Use `with_hasher` or `collect` for another hasher.",
                );
            }
            *last = rs;
        }
        let t = mk_hash(&mut self.tys, k, &args);
        let s = *args.last().unwrap_or(&UNIT);
        Some(match b {
            B::HashNew | B::HashDefault => {
                if b == B::HashDefault {
                    self.require(self.p.gx.lang.default, s, span);
                }
                (vec![], t)
            }
            B::HashWithCapacity => (vec![USIZE], t),
            B::HashWithHasher => (vec![s], t),
            B::HashWithCapacityAndHasher => (vec![USIZE, s], t),
            // The argument is typed in `hash_from_array_sig`.
            B::HashFromArray => (vec![], t),
            _ => return None,
        })
    }

    /// `HashMap::from(array)` and `HashSet::from(array)` at the table type
    /// `t`: std's `From<[(K, V); N]>` and `From<[T; N]>`, which build the
    /// table with `RandomState` through `from_iter` over the array's
    /// `array::IntoIter`. The keys must be `Hash + Eq`.
    pub(super) fn hash_from_array_sig(&mut self, t: Ty, args: &[ExprId], span: Span) -> (Vec<Ty>, Ty) {
        let Some((kind, targs)) = hash_parts(&self.tys, t) else { return (vec![], ERROR) };
        let at = match args {
            [a] => self.expr(*a, None),
            _ => {
                self.err("L0105", span, "`from` takes 1 argument", "Pass an array.");
                return (vec![], ERROR);
            }
        };
        let at = self.deep(at);
        let what = if kind == HashTy::HashSet { "set" } else { "map" };
        let el = match self.kind(at) {
            TyKind::Array(el, _) => el,
            TyKind::Error => return (vec![at], t),
            TyKind::Infer(_) => {
                self.err("L0610", span, "type annotations needed: the argument of `from` is not known", "Pass an array.");
                return (vec![at], ERROR);
            }
            // The identity `From<T> for T`.
            _ if hash_parts(&self.tys, at).is_some_and(|(k, _)| k == kind) => {
                self.err("L0640", span, format!("`from` of a {what} is not implemented yet"), "Use the value itself.");
                return (vec![at], ERROR);
            }
            _ => {
                let tn = self.tn(at);
                self.err("L0612", span, format!("the {what} has no `From<{tn}>`"), "Pass an array, or collect an iterator.");
                return (vec![at], ERROR);
            }
        };
        let rs = mk_hash(&mut self.tys, HashTy::RandomState, &[]);
        let s = *targs.last().unwrap_or(&UNIT);
        if !self.unify(s, rs) {
            let tn = self.tn(s);
            self.err("L0612", span, format!("`from` builds a {what} with `RandomState`, not `{tn}`"), "Use `with_hasher` or `collect` for another hasher.");
            return (vec![at], ERROR);
        }
        let want = if kind == HashTy::HashMap {
            let ts = self.tys.intern_tys(&[targs[0], targs[1]]);
            self.tys.intern(TyKind::Tuple(ts))
        } else {
            targs[0]
        };
        if !self.unify(el, want) {
            self.mismatch(span, want, el);
            return (vec![at], ERROR);
        }
        self.require_hash_key(targs[0], s, span);
        (vec![at], t)
    }
}

/// True for the hash constructor builtins (`hash_ctor_sig`).
pub fn is_hash_ctor(b: Builtin) -> bool {
    matches!(
        b,
        Builtin::HashNew
            | Builtin::HashWithCapacity
            | Builtin::HashWithHasher
            | Builtin::HashWithCapacityAndHasher
            | Builtin::HashDefault
            | Builtin::HashFromArray
    )
}

/// The constructor builtin a hash type's associated fn names.
pub fn hash_ctor_builtin(k: HashTy, item: &str) -> Option<Builtin> {
    let table = matches!(k, HashTy::HashMap | HashTy::HashSet);
    Some(match item {
        "new" if table || k == HashTy::RandomState || k == HashTy::DefaultHasher => Builtin::HashNew,
        // `BuildHasherDefault::new()` is its `default()` (a const fn).
        "new" if k == HashTy::BuildHasherDefault => Builtin::HashDefault,
        "with_capacity" if table => Builtin::HashWithCapacity,
        "with_hasher" if table => Builtin::HashWithHasher,
        "with_capacity_and_hasher" if table => Builtin::HashWithCapacityAndHasher,
        "from" if table => Builtin::HashFromArray,
        "default" if !matches!(k, HashTy::Entry | HashTy::OccupiedEntry | HashTy::VacantEntry) => Builtin::HashDefault,
        _ => return None,
    })
}

