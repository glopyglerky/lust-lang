// UFCS calls of builtin methods: `Vec::len(&v)`, `Cell::get(&c)`,
// `std::thread::Builder::spawn(b, f)`.
//
// lust's builtin methods live in the method-call tables (`method_call_at`),
// not as items a path can name. A type-relative path call whose type is a
// builtin with no item of that name is checked as the method call with the
// first argument as the receiver, then held to UFCS rules: a type-relative
// path never autoderefs past the written type, and its first argument is a
// coercion site for `Self`, `&Self`, or `&mut Self`, not an autoref site.
// So the receiver's adjustment must be one rustc's argument coercion makes:
// a by-value `self` takes the argument as is, and `&self` / `&mut self`
// take a reference (of that mutability) that derefs to the written type
// through references and smart pointers. Anything else rejects. A `Vec`
// or `String` path names only its own items and its traits' (`VEC_ITEMS`,
// `STRING_ITEMS`), never a slice or `str` method the table also keys on it
// (`Vec::iter` is rustc E0599).
//
// Lowering calls `method_into` with the first argument as the receiver
// (lower.rs, `Expr::Call` with a `Res::BuiltinMethod`; a `Res::Method`
// call already lowers that way).
use super::*;

impl<'p> Ctx<'p> {
    /// `Type::method(recv, args..)` for a builtin type path `segs` whose
    /// method is a builtin one: `written` is the path's type when it
    /// resolves with its args (else the receiver's type must be the
    /// builtin the path names). None, with nothing checked, when the call
    /// has no receiver argument or the path names a slice or `str` item.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn builtin_method_by_path(
        &mut self,
        e: ExprId,
        segs: &[ast::PathSeg],
        written: Option<Ty>,
        name_span: Span,
        args: &[ExprId],
        span: Span,
        expected: Option<Ty>,
    ) -> Option<Ty> {
        let (&recv, rest) = args.split_first()?;
        let n = segs.len();
        let name = segs[n - 1].name;
        let mut ty_name = crate::intern::source_name(self.name(segs[n - 2].name)).to_string();
        let item = self.name(name).to_string();
        // A type path of a std builtin with methods (a module path such as
        // `mem::size_of_val`, a user type, or an associated fn with no
        // receiver keeps its own diagnostics).
        let mut std_prefix = segs[..n - 2].iter().all(|s| matches!(self.name(s.name), "std" | "core" | "alloc" | "cell" | "collections" | "sync" | "thread" | "vec" | "string" | "rc" | "boxed" | "hash_map" | "vec_deque"));
        // A type alias of one (`FxHashMap::clone(&m)`): the written type
        // names the builtin, and `ufcs_recv_ok` unifies the argument with it.
        let written_d = written.map(|w| self.deep(w));
        if !BUILTIN_TYPES.contains(&ty_name.as_str())
            && let Some(w) = written_d
            && let Some(path) = super::alias_path::builtin_std_path(&self.tys, w)
            && let Some(last) = path.last().filter(|l| BUILTIN_TYPES.contains(l))
        {
            ty_name = last.to_string();
            std_prefix = true;
        }
        if !BUILTIN_TYPES.contains(&ty_name.as_str()) || !std_prefix {
            return None;
        }
        match ty_name.as_str() {
            "Vec" if !VEC_ITEMS.contains(&item.as_str()) => return None,
            "String" if !STRING_ITEMS.contains(&item.as_str()) => return None,
            _ => {}
        }
        if item == "clone"
            && let Some(t) = self.builtin_clone_by_path(e, segs, written, args, span)
        {
            return Some(t);
        }
        // A by-value `self` of `Vec` takes its argument as `Vec<T>` exactly
        // (no autoref or deref), so that type guides the argument's own
        // inference (`Vec::leak(it.collect())`).
        let hint = match (ty_name.as_str(), item.as_str()) {
            ("Vec", "leak" | "into_boxed_slice" | "into_flattened") => Some(match written {
                Some(w) => w,
                None => {
                    let v = self.new_var(false);
                    self.tys.mk_vec(v)
                }
            }),
            _ => None,
        };
        let at = self.expr(recv, hint);
        let err_mark = self.errs.len();
        let deref_mark = (self.out.deref_recv.len(), self.deref_sites.len());
        let ret = self.method_call_with_rt(e, recv, at, name, name_span, rest, &[], expected);
        let at = self.deep(at);
        let ok = match self.out.expr_res[self.ei(e)] {
            // A builtin's std source method (`HashMap::insert`) is no
            // item a path finds either: the same rules apply.
            Res::BuiltinMethod { derefs, autoref, .. } | Res::Method { derefs, autoref, .. } => {
                // No step through a user `Deref`, and no receiver view (a
                // Cow's or an iterator wrapper's): plain builtin steps only.
                let plain = self.out.deref_recv.len() == deref_mark.0
                    && self.deref_sites.len() == deref_mark.1
                    && self.out.coerce[self.ei(recv)] == Coerce::None;
                // An eta closure's argument stands for the fn item's
                // parameter: exactly `Self`, `&Self`, or `&mut Self`
                // (`map(Vec::len)` over `&Box<Vec<_>>` items is rustc
                // E0631), never a deref coercion.
                let exact = !self.is_eta_arg(recv) || derefs <= 1;
                plain && exact && self.ufcs_recv_ok(at, segs, written, derefs, autoref)
            }
            // No method of that name for the argument (`Duration::from_days(2)`
            // is an associated fn): the path's own diagnostics stand.
            _ if self.errs.len() > err_mark => {
                self.errs.truncate(err_mark);
                return None;
            }
            // Any other form (an operator such as `String::eq` as `==`,
            // a comparison builtin) has other argument rules: reject.
            _ => false,
        };
        if !ok {
            let at_n = self.tn(at);
            self.err(
                "L0110",
                span,
                format!("mismatched types: `{ty_name}::{item}` does not take `{at_n}` as its first argument here"),
                "Pass the receiver at the method's own `self` type (`&value` for `&self`).",
            );
            return Some(ERROR);
        }
        Some(ret)
    }

    /// The prelude's `drop` used as a value (`.for_each(drop)`), when no
    /// binding takes the name: the fn item `mem::drop::<T>`, as a prim
    /// item over `T` (`prim_item_value`; `std_free_item_sig` gives its
    /// signature). None for any other name.
    pub(super) fn prelude_fn_value(&mut self, e: ExprId, seg: &ast::PathSeg, span: Span) -> Option<Ty> {
        if self.name(seg.name) != "drop" {
            return None;
        }
        let written = match &seg.args {
            ast::GenericArgs::Angle(a) => a.len(),
            ast::GenericArgs::None => 0,
            _ => 2,
        };
        let t = match seg.type_args().as_slice() {
            [] if written == 0 => self.new_var(false),
            [a] if written == 1 => self.resolve_written(*a),
            _ => {
                self.err("L0107", span, "`drop` takes one generic argument", "Write `drop::<T>`.");
                return Some(ERROR);
            }
        };
        self.set_res(e, Res::PrimAssoc { b: Builtin::MemDrop, self_ty: t, name: seg.name });
        Some(self.prim_item_value(Builtin::MemDrop, t, seg.name, span))
    }

    /// A method of a std type by path as a `PrimAssoc` (`Weak::upgrade`):
    /// its Self agrees with the type the path writes (`Receiver::<u16>::
    /// recv`, an alias's type), as the item's own Self does in rustc
    /// (E0308 on a mismatch). Other resolutions pass through.
    pub(super) fn written_prim_self(&mut self, r: Res, segs: &[ast::PathSeg], span: Span) -> Res {
        if let Res::PrimAssoc { self_ty, .. } = r
            && let Some(want) = self.written_builtin_self(segs, span)
            && !self.unify(self_ty, want)
        {
            self.mismatch(span, want, self_ty);
        }
        r
    }

    /// `Type::from` as a value (`.map(OnceCell::from)`, `.map(Arc::from)`),
    /// when the path names no inherent `from`: `<Type<..> as From<_>>::from`
    /// with its input open, as rustc's probe takes the one trait in scope
    /// with that item (traitpath.rs selects the impl at body end, by the
    /// input the uses give). `base` is the path's type, its omitted args
    /// fresh. None when another trait in scope also has a `from` that may
    /// apply (rustc E0034: the caller reports), or for another item.
    pub(super) fn from_path_value(&mut self, e: ExprId, segs: &[ast::PathSeg], base: Option<Ty>, span: Span) -> Option<Ty> {
        let name = segs.last()?.name;
        if name != self.p.known.from || segs.len() < 2 {
            return None;
        }
        let base = match base {
            Some(b) => b,
            None => self.builtin_path_base(segs, span, None)?,
        };
        let from = self.p.gx.lang.from;
        if !self.only_trait_item(from, base, name) {
            return None;
        }
        let tr = self.fresh_trait_ref(from, base, 1);
        self.trait_ref_fn_value(e, tr, name, span)
    }

    /// True when trait `tdef` is in scope at `base` and no other trait in
    /// scope has an item `name` that may apply to `base` (rustc's probe
    /// after the inherent items: two such traits are E0034).
    fn only_trait_item(&mut self, tdef: DefId, base: Ty, name: Sym) -> bool {
        let p = self.p;
        if !method::trait_in_scope_at(self, tdef, base) {
            return false;
        }
        for other in p.gx.traits_by_method.get(&name).cloned().unwrap_or_default() {
            if other == tdef || !method::trait_in_scope_at(self, other, base) {
                continue;
            }
            let Some(rec) = p.gx.index.trait_record(other) else { return false };
            let n_inputs = rec.generics.params.len().saturating_sub(1);
            let (snap, obl_len) = (self.snapshot_vars(), self.obl_len());
            let tr = self.fresh_trait_ref(other, base, n_inputs);
            let sel = self.select_obligation(tr);
            self.rollback_vars(snap);
            self.obl_truncate(obl_len);
            if !matches!(sel, Selection::NoImpl) {
                return false;
            }
        }
        true
    }

    /// `Vec::clone(x)`, `HashMap::clone(&arc_map)` on a builtin type
    /// path: `<Type<..> as Clone>::clone(x)`, its argument a coercion
    /// site for `&Type<..>` (a deref coercion from `&Arc<Type>` too), as
    /// the trait fn takes it. The method probe would pick `Clone for &T`
    /// on a `&Type` argument instead. None when another trait in scope has
    /// a `clone` for the type (the method form then decides).
    fn builtin_clone_by_path(&mut self, e: ExprId, segs: &[ast::PathSeg], written: Option<Ty>, args: &[ExprId], span: Span) -> Option<Ty> {
        let base = match written {
            Some(w) => w,
            None => self.builtin_path_base(segs, span, None)?,
        };
        self.clone_call_at(e, base, args, span)
    }

    /// `<base as Clone>::clone(args)` for `builtin_clone_by_path` and an
    /// alias's `A::clone(x)` (`std_trait_assoc_call`).
    pub(super) fn clone_call_at(&mut self, e: ExprId, base: Ty, args: &[ExprId], span: Span) -> Option<Ty> {
        let name = self.p.known.clone;
        let clone = self.p.gx.lang.clone;
        if !self.only_trait_item(clone, base, name) {
            if !self.clone_path_unique(base, span) {
                for &a in args {
                    self.expr(a, None);
                }
                return Some(ERROR);
            }
            return None;
        }
        let tr = self.fresh_trait_ref(clone, base, 0);
        self.trait_ref_call(e, tr, name, args, span)
    }

    /// `Type::clone(x)` on a builtin type: false, after reporting L0720
    /// (rustc E0034, the method form's code), when another trait in scope
    /// has a `clone` that may apply to `base` too.
    pub(super) fn clone_path_unique(&mut self, base: Ty, span: Span) -> bool {
        let (clone, name) = (self.p.gx.lang.clone, self.p.known.clone);
        if !method::trait_in_scope_at(self, clone, base) || self.only_trait_item(clone, base, name) {
            return true;
        }
        self.err("L0720", span, "multiple trait methods named `clone` apply here", "Name the trait: `Clone::clone(x)` or `<T as Trait>::clone(x)`.");
        false
    }

    /// `Rc::default()` and `Arc::default()`: `() -> Rc<T>` with `T:
    /// Default` (std's `impl<T: Default> Default for Rc<T>`), the element
    /// inferred or written (`Rc::<u8>::default()`, `unify_builtin_self`);
    /// the default glue builds the value. An alias of an `Rc`, `Arc`, or
    /// `Box` names its target (`type D = Arc<Mutex<P>>; .then(D::default)`).
    /// None for other paths.
    pub(super) fn smart_default_sig(&mut self, callee: ExprId, span: Span) -> Option<(Vec<Ty>, Ty)> {
        let Expr::Path(segs) = self.p.ast.expr(callee) else { return None };
        let [.., ty_seg, _] = segs.as_slice() else { return None };
        use crate::check::own::Owner;
        let owner = match self.builtin_alias_ty(segs, span, false) {
            Some(t) => match self.tys.builtin_of(t) {
                Some(BuiltinTy::Rc) => Owner::Rc,
                Some(BuiltinTy::Sync(crate::ty::sync::SyncTy::Arc)) => Owner::Arc,
                Some(BuiltinTy::Boxed) => Owner::Box,
                _ => return None,
            },
            // `Box::default()` is `box_default_sig`'s.
            None => match self.name(ty_seg.name) {
                "Rc" => Owner::Rc,
                "Arc" => Owner::Arc,
                _ => return None,
            },
        };
        let elem = self.new_var(false);
        let t = match owner {
            Owner::Rc => self.tys.mk_rc(elem),
            Owner::Arc => crate::ty::sync::sync_ty(&mut self.tys, crate::ty::sync::SyncTy::Arc, &[elem]),
            Owner::Box => self.tys.mk_boxed(elem),
        };
        self.deferred_default.push((t, span));
        Some((vec![], t))
    }

    /// The (params, return) of a std free fn kept as a prim item over
    /// one type `t` (see `prelude_fn_value`): `drop::<T>(T)`.
    pub(super) fn std_free_item_sig(&mut self, b: Builtin, t: Ty) -> Option<(Vec<Ty>, Ty)> {
        match b {
            Builtin::MemDrop => Some((vec![t], UNIT)),
            _ => None,
        }
    }

    /// True when a receiver of type `at` adjusted by `derefs` and
    /// `autoref` is an argument coercion to the method's `self` type at
    /// the written type (see the file head).
    fn ufcs_recv_ok(&mut self, at: Ty, segs: &[ast::PathSeg], written: Option<Ty>, derefs: u8, autoref: Autoref) -> bool {
        let mut t = at;
        for i in 0..derefs {
            let next = match self.kind(t) {
                // The first step must be through the argument's own
                // reference for `&self` and `&mut self`.
                TyKind::Ref(_, m, inner) => {
                    if i == 0 && autoref == Autoref::RefMut && !m.is_mut() {
                        return false;
                    }
                    inner
                }
                _ if i == 0 => return false,
                _ => match self.tys.smart_elem(t) {
                    Some(inner) => inner,
                    None => return false,
                },
            };
            t = self.resolve(next);
        }
        let shape_ok = match autoref {
            Autoref::None => derefs == 0,
            Autoref::Ref | Autoref::RefMut => derefs >= 1,
            _ => false,
        };
        shape_ok
            && match written {
                Some(w) => self.unify(t, w),
                None => self.names_builtin(t, segs),
            }
    }

    /// True when the type path `segs` (then the item) names builtin type
    /// `t`: its full std path, or its bare name when no user type of that
    /// name is in scope (std's own source types, such as `HashMap`'s, may
    /// bind the name).
    fn names_builtin(&mut self, t: Ty, segs: &[ast::PathSeg]) -> bool {
        let Some(named) = super::alias_path::builtin_std_path(&self.tys, t) else { return false };
        let n = segs.len();
        let path: Vec<&str> = segs[..n - 1].iter().map(|s| crate::intern::source_name(self.name(s.name))).collect();
        // A full path: the canonical one, or a std re-export of the same
        // type (`use std::collections::HashMap` resolves the bare name to
        // `std::collections::HashMap`, not `hash_map::HashMap`).
        if n > 2 {
            return named == path || (named.last() == path.last() && matches!(path.first(), Some(&("std" | "core" | "alloc"))));
        }
        if named.last() != path.last() {
            return false;
        }
        let p = self.p;
        match self.binding(Namespace::Type, segs[0].name) {
            BindingRes::Def(d) => p.adt_by_def(d).is_none_or(|a| generics::is_std_adt(&p.tys, a)),
            _ => true,
        }
    }
}

/// More builtins a path names as a fn value (`.map(JoinHandle::join)`),
/// after `PRIM_VALUE_BUILTINS` in the item code space (a value's type
/// carries its builtin by index: append only). None is
/// `#[track_caller]` (`RefCell::replace` is a path call only: a value's
/// panic would report another location).
pub(crate) const STD_VALUE_BUILTINS: &[Builtin] = &[
    Builtin::Sync(crate::check::sync::SyncOp::ChanRecv),
    Builtin::Sync(crate::check::sync::SyncOp::ChanTryRecv),
    Builtin::Sync(crate::check::sync::SyncOp::ChanRecvTimeout),
    Builtin::Sync(crate::check::sync::SyncOp::JoinHandleJoin),
    Builtin::MemDrop,
    Builtin::WeakUpgrade,
    Builtin::WeakStrongCount,
    Builtin::WeakWeakCount,
    Builtin::RefCellIntoInner,
    Builtin::RefCellGetMut,
    Builtin::CellSet,
    Builtin::CellReplace,
    Builtin::CellIntoInner,
    Builtin::CellGetMut,
    Builtin::CellTake,
];

/// True for a `STD_VALUE_BUILTINS` item that cannot panic itself, so a
/// fn pointer to it needs no def site for a panic location (`drop`'s
/// value's own `Drop` panics at its impl, as through rustc's pointer).
pub(crate) fn no_panic_std_item(b: Builtin) -> bool {
    STD_VALUE_BUILTINS.contains(&b)
}

/// The std types whose methods are builtin (or std source methods no path
/// resolves as items), by their bare name.
const BUILTIN_TYPES: &[&str] = &[
    "Vec", "String", "VecDeque", "HashMap", "HashSet", "Cell", "RefCell", "Mutex", "RwLock", "Condvar", "Builder",
    "OnceCell", "OnceLock",
];

/// The items a `Vec<T>` path names that lust's builtin method table has:
/// `Vec`'s inherent methods and its traits' (`Clone`, `Extend`,
/// `IntoIterator`, `Deref`, ...), not the slice methods it derefs to.
const VEC_ITEMS: &[&str] = &[
    "capacity", "reserve", "reserve_exact", "try_reserve", "try_reserve_exact", "shrink_to_fit", "shrink_to",
    "into_boxed_slice", "truncate", "as_slice", "as_mut_slice", "as_ptr", "as_mut_ptr", "swap_remove", "insert", "remove",
    "retain", "retain_mut", "dedup_by_key", "dedup_by", "dedup", "push", "pop", "pop_if", "append", "drain", "clear", "len",
    "is_empty", "split_off", "resize_with", "resize", "leak", "spare_capacity_mut", "extend_from_slice", "extend_from_within",
    "splice", "extract_if", "into_flattened", "clone", "clone_from", "extend", "into_iter", "deref", "deref_mut", "as_ref",
    "as_mut", "borrow", "borrow_mut", "eq", "ne", "cmp", "partial_cmp", "lt", "le", "gt", "ge", "max", "min", "hash", "fmt",
    "index", "index_mut", "to_owned",
];

/// The items a `String` path names that lust's builtin method table has:
/// `String`'s inherent methods and its traits', not the `str` methods.
const STRING_ITEMS: &[&str] = &[
    "into_bytes", "as_str", "as_mut_str", "push_str", "capacity", "reserve", "reserve_exact", "try_reserve",
    "try_reserve_exact", "shrink_to_fit", "shrink_to", "push", "as_bytes", "truncate", "pop", "remove", "retain", "insert",
    "insert_str", "as_mut_vec", "len", "is_empty", "split_off", "clear", "drain", "replace_range", "into_boxed_str", "leak",
    "extend_from_within", "into_chars", "clone", "clone_from", "extend", "deref", "deref_mut", "as_ref", "as_mut", "borrow",
    "borrow_mut", "eq", "ne", "cmp", "partial_cmp", "lt", "le", "gt", "ge", "max", "min", "hash", "fmt", "index", "index_mut",
    "to_owned", "to_string", "add", "add_assign",
];
