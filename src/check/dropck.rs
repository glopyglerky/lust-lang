// Drop-check overflow (rustc E0320, lust L0730).
//
// rustc's borrow checker computes the drop-check constraints of every
// body local's type (`dropck_outlives`): a worklist that starts at the
// type and follows, per Adt, the types its fields can drop
// (`adt_dtorck_constraint`). A polymorphically recursive type
// (`struct Grow<T> { v: T, g: Option<Box<Grow<(T, T)>>> }`) makes the
// worklist grow without end; past the recursion limit (128) rustc
// rejects the body. This module runs the same worklist, in the same
// order, over the types rustc gives MIR locals: parameters, the return
// place, bindings, and the temporary of every value expression.
//
// Only template and plain bodies are checked, as rustc borrow-checks
// the generic body once and never an instance. Std types lust models
// as builtins use rustc's real field layouts (`builtin_items`), so the
// depth at which a deep but finite nesting overflows is rustc's too.

use super::Checked;
use crate::ast::Expr;
use crate::check::collect::Program;
use crate::check::Res;
use crate::check::collect::closure_fields;
use crate::def::LocalDefId;
use crate::diag::{Diag, Span};
use crate::intern::FxMap;

type FxSet<T> = std::collections::HashSet<T, std::hash::BuildHasherDefault<crate::intern::FxHasher>>;
use crate::ty::hash::HashTy;
use crate::ty::iter::IterKind;
use crate::ty::sync::SyncTy;
use crate::ty::{AdtId, BuiltinTy, GenericArg, IntTy, Ty, TyIntern, TyKind, Tys};


/// One worklist entry: a type, or a std-internal type a builtin's
/// layout names and lust has no def for (rustc dedups them by type,
/// so each has its own identity here).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Item {
    Ty(Ty),
    /// `alloc::alloc::Global`, the default allocator param.
    Global,
    /// `rc::RcInner<T>` and `sync::ArcInner<T>`: `[usize, T]`.
    CountedInner(Ty, bool),
    /// `btree::node::NodeRef<Owned, K, V, LeafOrInternal>`, with `V`
    /// `None` for `BTreeSet`'s `SetValZST`: `[(Owned, LeafOrInternal)]`.
    NodeRef(Ty, Option<Ty>),
    /// The tuple `(Owned, LeafOrInternal)`.
    OwnedLeaf,
    /// `Box<(T, SetValZST)>`: `[(T, SetValZST), Global]`.
    SetBox(Ty),
    /// The tuple `(T, SetValZST)`.
    SetPair(Ty),
}

/// The drop-check facts of one step: the types it hands to the
/// worklist, and the first type that passed the limit.
#[derive(Default)]
struct Constraint {
    items: Vec<Item>,
    overflow: Option<Item>,
}

/// Per-body state: the store, and the field constraints of each
/// template Adt in its own params (rustc's `adt_dtorck_constraint`).
struct Dropck<'a, 'g> {
    p: &'a Program,
    /// The crate's recursion limit (128 unless `#![recursion_limit]`).
    limit: usize,
    tys: &'a mut Tys<'g>,
    adts: FxMap<AdtId, Constraint>,
}

/// The fingerprint of Adt template `def`'s drop-check constraint as `Dropck::adt`
/// makes it (its fields' items in its own params, std types by their real
/// layout, and an overflow), for its `ReadKey::DropCk` read. 0 for no Adt.
pub(crate) fn template_constraint_hash(p: &Program, def: crate::def::DefId) -> u64 {
    use std::hash::{Hash, Hasher};
    let Some(a) = p.defs.local_adt(def) else { return 0 };
    let mut tys = Tys::new(&p.tys);
    let mut cx = Dropck { p, limit: crate::check::where_overflow::recursion_limit(p), tys: &mut tys, adts: FxMap::default() };
    let mut h = crate::intern::FxHasher::default();
    "dropck-constraint-v1".hash(&mut h);
    let fields: Vec<Ty> = p.adt(a).variants.iter().flat_map(|v| v.fields.iter().map(|f| f.ty)).collect();
    let mut own = Constraint::default();
    for f in fields {
        cx.inner(Item::Ty(f), 0, &mut own);
    }
    let item = |h: &mut crate::intern::FxHasher, i: &Item, tys: &Tys| match *i {
        Item::Ty(t) => (0u8, crate::driver::graph::hash_ty_value(p, tys, t)).hash(h),
        Item::Global => 1u8.hash(h),
        Item::CountedInner(t, arc) => (2u8, arc, crate::driver::graph::hash_ty_value(p, tys, t)).hash(h),
        Item::NodeRef(k, v) => (3u8, crate::driver::graph::hash_ty_value(p, tys, k), v.map(|v| crate::driver::graph::hash_ty_value(p, tys, v))).hash(h),
        Item::OwnedLeaf => 4u8.hash(h),
        Item::SetBox(t) => (5u8, crate::driver::graph::hash_ty_value(p, tys, t)).hash(h),
        Item::SetPair(t) => (6u8, crate::driver::graph::hash_ty_value(p, tys, t)).hash(h),
    };
    own.items.len().hash(&mut h);
    for i in &own.items {
        item(&mut h, i, &*cx.tys);
    }
    match &own.overflow {
        Some(o) => {
            1u8.hash(&mut h);
            item(&mut h, o, &*cx.tys);
        }
        None => 0u8.hash(&mut h),
    }
    h.finish()
}

/// Checks one body for drop-check overflow. `fid` is a plain fn, a
/// template, or a closure's parent; mono instances are skipped by the
/// caller. Returns the first overflow as rustc's E0320.
pub fn check_body(p: &Program, fid: LocalDefId, ck: &mut Checked) -> Option<Diag> {
    let found = body_types(p, fid, ck);
    let mut seen: FxSet<Ty> = FxSet::default();
    let mut cx = Dropck { p, limit: crate::check::where_overflow::recursion_limit(p), tys: &mut ck.tys, adts: FxMap::default() };
    for (t, span, boring) in found {
        // A trivial type is skipped every time, so only the others need
        // the dedup set (most body types are scalars and references).
        if trivial(&*cx.tys, p, t) || !seen.insert(t) {
            continue;
        }
        // A body local whose type names a region is checked only where
        // it may be dropped (rustc's drop-live locals); a type with no
        // drop glue is never dropped.
        if !boring && has_region(&*cx.tys, t) && !may_drop(p, cx.tys, t, 0) {
            continue;
        }
        if let Some(over) = cx.overflows(t) {
            let ty = short_name(p, &*cx.tys, t);
            let on = cx.item_name(over);
            return Some(Diag::new(
                "L0730",
                span,
                format!("overflow while adding drop-check rules for `{ty}` (overflowed on `{on}`)"),
                "The type's drop check never ends: a field names the type again with larger type arguments \
                 (polymorphic recursion). Give the recursive field a fixed type argument.",
            ));
        }
    }
    None
}

/// The types rustc gives one body's MIR locals: the return place and the
/// parameters (`true`: rustc checks these always), then the bindings and
/// the value-expression temporaries. Place expressions name storage and
/// make no local.
pub(crate) fn body_types(p: &Program, fid: LocalDefId, ck: &Checked) -> Vec<(Ty, Span, bool)> {
    let mut found: Vec<(Ty, Span, bool)> = Vec::new();
    let f = p.fn_info(fid);
    let def = f.def(&p.ast);
    let name_span = def.name_span;
    // The return place and the parameters hold only signature regions:
    // rustc checks them always (its "boring" locals).
    found.push((f.ret, name_span, true));
    for &l in &ck.param_locals {
        let info = &ck.locals[l as usize];
        found.push((info.ty, info.span, true));
    }
    for (i, info) in ck.locals.iter().enumerate() {
        if !ck.param_locals.contains(&(i as u32)) {
            found.push((info.ty, info.span, false));
        }
    }
    for i in def.exprs.0..def.exprs.1 {
        let at = (i - ck.expr_base) as usize;
        let Some(&t) = ck.expr_ty.get(at) else { continue };
        if is_place(&p.ast.exprs[i as usize], ck.expr_res.get(at).copied()) {
            continue;
        }
        found.push((t, p.ast.expr_spans[i as usize], false));
    }
    found
}

/// rustc's place expressions: they name storage and make no temporary.
fn is_place(e: &Expr, res: Option<Res>) -> bool {
    match e {
        Expr::Path(_) => matches!(res, Some(Res::Local(_) | Res::Static(_))),
        Expr::SelfValue | Expr::Field { .. } | Expr::TupleField { .. } | Expr::Index { .. } | Expr::Deref(_) | Expr::Underscore => true,
        _ => false,
    }
}

/// True when a value of `t` may run drop glue (rustc's `needs_drop`):
/// an Adt reads its fields at its own args, a param may drop. `depth`
/// stops an unbounded type, which rustc reports on its own.
fn may_drop(p: &Program, tys: &mut Tys, t: Ty, depth: u32) -> bool {
    if depth > 256 {
        return true;
    }
    match tys.kind(t) {
        TyKind::Param(_) | TyKind::Projection(..) | TyKind::Opaque(..) | TyKind::Dyn(..) => true,
        TyKind::Array(e, n) => !matches!(tys.const_kind(n), crate::ty::ConstKind::Int(0)) && may_drop(p, tys, e, depth + 1),
        TyKind::Slice(e) => may_drop(p, tys, e, depth + 1),
        TyKind::Tuple(ts) => tys.tys_of(ts).to_vec().into_iter().any(|e| may_drop(p, tys, e, depth + 1)),
        TyKind::Closure(_, a) => closure_fields(&*tys, a).into_iter().any(|e| may_drop(p, tys, e, depth + 1)),
        TyKind::Adt(def, a) => {
            let args: Vec<GenericArg> = tys.args(a).to_vec();
            let arg_tys = || args.iter().filter_map(|g| g.as_ty()).collect::<Vec<_>>();
            if let Some(b) = tys.builtin_of(t) {
                // Wrappers that drop only what they hold; the other
                // builtins answer by their own rule.
                let wraps = matches!(
                    b,
                    BuiltinTy::Option | BuiltinTy::Result | BuiltinTy::Cell | BuiltinTy::RefCell
                ) || matches!(
                    b,
                    BuiltinTy::Sync(SyncTy::Mutex | SyncTy::RwLock | SyncTy::PoisonError | SyncTy::OnceLock | SyncTy::SendError)
                );
                return if wraps {
                    arg_tys().into_iter().any(|e| may_drop(p, tys, e, depth + 1))
                } else {
                    crate::check::collect::needs_drop(p, &*tys, t)
                };
            }
            let Some(adt) = p.defs.local_adt(def) else { return true };
            if p.is_manually_drop_adt(adt) {
                return false;
            }
            if p.gx.drop_methods.contains_key(&adt) || p.drop_adts.contains(&adt) {
                return true;
            }
            let fields: Vec<Ty> = p.adt(adt).variants.iter().flat_map(|v| v.fields.iter().map(|f| f.ty)).collect();
            fields.into_iter().any(|f| {
                let f = crate::ty::subst(tys, f, &args);
                may_drop(p, tys, f, depth + 1)
            })
        }
        _ => false,
    }
}

/// True when `t` names a region outside a fn pointer's binder: a body
/// local of such a type gets a region variable in rustc's MIR.
fn has_region(tys: &impl TyIntern, t: Ty) -> bool {
    match tys.kind(t) {
        TyKind::Ref(..) | TyKind::Dyn(..) => true,
        TyKind::Array(e, _) | TyKind::Slice(e) | TyKind::RawPtr(_, e) => has_region(tys, e),
        TyKind::Tuple(ts) => tys.tys_of(ts).iter().any(|&e| has_region(tys, e)),
        TyKind::Adt(_, a) | TyKind::Projection(_, a) | TyKind::Opaque(_, a) | TyKind::Closure(_, a) | TyKind::FnDef(_, a) => {
            tys.args(a).iter().any(|g| match g {
                GenericArg::Lifetime(_) => true,
                GenericArg::Ty(e) => has_region(tys, *e),
                GenericArg::Const(_) => false,
            })
        }
        _ => false,
    }
}

/// rustc's `trivial_dropck_outlives`: types whose drop check is empty.
fn trivial(tys: &impl TyIntern, p: &Program, t: Ty) -> bool {
    match tys.kind(t) {
        TyKind::Error
        | TyKind::Unit
        | TyKind::Bool
        | TyKind::Char
        | TyKind::Never
        | TyKind::Int(_)
        | TyKind::Float(_)
        | TyKind::Str
        | TyKind::Ref(..)
        | TyKind::RawPtr(..)
        | TyKind::FnPtr(_)
        | TyKind::FnDef(..)
        // An unresolved variable is a checker error elsewhere.
        | TyKind::Infer(_) => true,
        TyKind::Slice(e) => trivial(tys, p, e),
        TyKind::Array(e, n) => matches!(tys.const_kind(n), crate::ty::ConstKind::Int(0)) || trivial(tys, p, e),
        TyKind::Tuple(ts) => tys.tys_of(ts).iter().all(|&e| trivial(tys, p, e)),
        TyKind::Closure(_, a) => closure_fields(tys, a).iter().all(|&e| trivial(tys, p, e)),
        TyKind::Adt(def, _) => p.defs.local_adt(def).is_some_and(|a| p.is_manually_drop_adt(a)),
        TyKind::Dyn(..) | TyKind::Opaque(..) | TyKind::Param(_) | TyKind::Projection(..) => false,
    }
}

impl Dropck<'_, '_> {
    /// rustc's `compute_dropck_outlives_inner`: a LIFO worklist from
    /// `root`; each popped entry is expanded at its own depth, and
    /// each new type it names is pushed one deeper. Params and
    /// projections are never pushed. Returns the entry that passed
    /// the limit, if any.
    fn overflows(&mut self, root: Ty) -> Option<Item> {
        let mut stack: Vec<(Item, usize)> = vec![(Item::Ty(root), 0)];
        let mut set: FxSet<Item> = FxSet::default();
        while let Some((item, depth)) = stack.pop() {
            let mut c = Constraint::default();
            self.inner(item, depth, &mut c);
            if c.overflow.is_some() {
                return c.overflow;
            }
            for next in c.items {
                if let Item::Ty(t) = next
                    && matches!(self.tys.kind(t), TyKind::Param(_) | TyKind::Projection(..) | TyKind::Opaque(..))
                {
                    continue;
                }
                if set.insert(next) {
                    stack.push((next, depth + 1));
                }
            }
        }
        None
    }

    /// rustc's `dtorck_constraint_for_ty_inner`: expands one entry in
    /// place. Tuples, arrays, and closures expand their parts one
    /// level deeper in the same step; an Adt adds its field
    /// constraint's types, which the worklist then takes.
    fn inner(&mut self, item: Item, depth: usize, c: &mut Constraint) {
        if depth > self.limit {
            c.overflow.get_or_insert(item);
            return;
        }
        let t = match item {
            Item::Ty(t) => t,
            Item::Global => return,
            Item::CountedInner(e, _) => {
                c.items.push(Item::Ty(crate::ty::int(IntTy::Usize)));
                c.items.push(Item::Ty(e));
                return;
            }
            Item::NodeRef(..) => {
                c.items.push(Item::OwnedLeaf);
                return;
            }
            Item::OwnedLeaf => {
                // `Owned` and `LeafOrInternal` are field-less enums.
                if depth + 1 > self.limit {
                    c.overflow.get_or_insert(item);
                }
                return;
            }
            Item::SetBox(e) => {
                c.items.push(Item::SetPair(e));
                c.items.push(Item::Global);
                return;
            }
            Item::SetPair(e) => {
                self.inner(Item::Ty(e), depth + 1, c);
                if depth + 1 > self.limit {
                    c.overflow.get_or_insert(item);
                }
                return;
            }
        };
        if trivial(&*self.tys, self.p, t) {
            return;
        }
        match self.tys.kind(t) {
            TyKind::Array(e, _) | TyKind::Slice(e) => self.inner(Item::Ty(e), depth + 1, c),
            TyKind::Tuple(ts) => {
                for e in self.tys.tys_of(ts).to_vec() {
                    self.inner(Item::Ty(e), depth + 1, c);
                }
            }
            TyKind::Closure(_, a) => {
                for e in closure_fields(&*self.tys, a) {
                    self.inner(Item::Ty(e), depth + 1, c);
                }
            }
            TyKind::Adt(..) => self.adt(t, depth, c),
            TyKind::Param(_) | TyKind::Projection(..) | TyKind::Opaque(..) => c.items.push(Item::Ty(t)),
            // A dyn value's drop is its vtable's: an outlives fact only.
            _ => {}
        }
    }

    /// An Adt's constraint at its args: a builtin's from rustc's real
    /// layout, a source Adt's from its fields (memoized per template,
    /// then instantiated), deduped in first-seen order.
    fn adt(&mut self, t: Ty, depth: usize, c: &mut Constraint) {
        let (def, args) = self.tys.adt_parts(t).unwrap();
        let args: Vec<GenericArg> = self.tys.args(args).to_vec();
        if let Some(b) = self.tys.builtin_of(t) {
            if let BuiltinTy::Iter(IterKind::ByNext) = b {
                // lust's wrapper for a user iterator: the type rustc
                // sees is the iterator itself.
                if let Some(i) = args.first().and_then(|g| g.as_ty()) {
                    self.inner(Item::Ty(i), depth, c);
                }
                return;
            }
            let items = self.builtin_items(b, &args);
            push_dedup(c, items);
            return;
        }
        let Some(a) = self.p.defs.local_adt(def) else { return };
        // This Adt's constraint comes from its fields (`template_constraint_hash`):
        // a `DropCk` read of the body.
        crate::check::derive_delta::note_def(def, crate::check::derive_delta::bit::DROPCK);
        if let Some(items) = self.std_items(a, &args) {
            push_dedup(c, items);
            return;
        }
        if !self.adts.contains_key(&a) {
            let mut own = Constraint::default();
            let fields: Vec<Ty> = self.p.adt(a).variants.iter().flat_map(|v| v.fields.iter().map(|f| f.ty)).collect();
            for f in fields {
                self.inner(Item::Ty(f), 0, &mut own);
            }
            let mut dedup: FxSet<Item> = FxSet::default();
            own.items.retain(|i| dedup.insert(*i));
            self.adts.insert(a, own);
        }
        let own = &self.adts[&a];
        let (items, over) = (own.items.clone(), own.overflow);
        let items: Vec<Item> = items.into_iter().map(|i| self.subst_item(i, &args)).collect();
        if let Some(o) = over {
            c.overflow.get_or_insert(self.subst_item(o, &args));
        }
        push_dedup(c, items);
    }

    fn subst_item(&mut self, i: Item, args: &[GenericArg]) -> Item {
        let mut s = |t: Ty| crate::ty::subst(&mut *self.tys, t, args);
        match i {
            Item::Ty(t) => Item::Ty(s(t)),
            Item::CountedInner(t, arc) => Item::CountedInner(s(t), arc),
            Item::NodeRef(k, v) => Item::NodeRef(s(k), v.map(&mut s)),
            Item::SetBox(t) => Item::SetBox(s(t)),
            Item::SetPair(t) => Item::SetPair(s(t)),
            Item::Global | Item::OwnedLeaf => i,
        }
    }

    /// The std-source Adts whose lust fields differ from std's: their
    /// real constraints (std 1.99 layouts).
    fn std_items(&mut self, a: AdtId, args: &[GenericArg]) -> Option<Vec<Item>> {
        if !self.p.is_std_adt(a) {
            return None;
        }
        let ty = |i: usize| args.iter().filter_map(|g| g.as_ty()).nth(i);
        let path = self.p.display_path(self.p.adt(a).name);
        let name = path.rsplit("::").next().unwrap_or("");
        Some(match name {
            "PhantomData" => vec![Item::Ty(ty(0)?)],
            // `VecDeque { head, len, buf: RawVec<T> }`.
            "VecDeque" => vec![self.u8_item(), Item::Global, Item::Ty(ty(0)?)],
            // `BTreeMap { root: Option<Root<K, V>>, length, alloc:
            // ManuallyDrop<A>, _marker: PhantomData<Box<(K, V), A>> }`.
            "BTreeMap" => {
                let (k, v) = (ty(0)?, ty(1)?);
                let pair = self.tuple(&[k, v]);
                let boxed = self.tys.mk_builtin1(|b| b.boxed, pair);
                vec![Item::NodeRef(k, Some(v)), Item::Ty(boxed)]
            }
            // `BTreeSet { map: BTreeMap<T, SetValZST> }`.
            "BTreeSet" => {
                let t = ty(0)?;
                vec![Item::NodeRef(t, None), Item::SetBox(t)]
            }
            _ => return None,
        })
    }

    /// The constraint of a builtin Adt at `args`, from the field layout
    /// of the std type it stands for (rust 1.99 library source).
    fn builtin_items(&mut self, b: BuiltinTy, args: &[GenericArg]) -> Vec<Item> {
        let tys: Vec<Ty> = args.iter().filter_map(|g| g.as_ty()).collect();
        let at = |i: usize| tys.get(i).copied().map(Item::Ty);
        let all = || tys.iter().map(|&t| Item::Ty(t)).collect::<Vec<_>>();
        let first = || at(0).into_iter().collect::<Vec<_>>();
        match b {
            // `String { vec: Vec<u8> }`; `Vec { buf: RawVec<T> { inner:
            // RawVecInner { ptr: Unique<u8>, cap, alloc: A }, _marker:
            // PhantomData<T> }, len }`.
            BuiltinTy::String => vec![self.u8_item(), Item::Global],
            BuiltinTy::Vec => [Some(self.u8_item()), Some(Item::Global), at(0)].into_iter().flatten().collect(),
            BuiltinTy::Option | BuiltinTy::Cell => first(),
            BuiltinTy::Result => all(),
            // `Box(Unique<T>, A)`.
            BuiltinTy::Boxed => [at(0), Some(Item::Global)].into_iter().flatten().collect(),
            // `Rc { ptr, phantom: PhantomData<RcInner<T>>, alloc: A }`;
            // `Weak { ptr, alloc: A }`.
            BuiltinTy::Rc => tys.first().map(|&t| vec![Item::CountedInner(t, false), Item::Global]).unwrap_or_default(),
            BuiltinTy::Weak => vec![Item::Global],
            // `Chars { iter: slice::Iter<'a, u8> }`: `[&u8]`.
            BuiltinTy::Chars => {
                let r = self.tys.mk_ref(false, crate::ty::int(IntTy::U8));
                vec![Item::Ty(r)]
            }
            // `RefCell { borrow: Cell<isize>, value: UnsafeCell<T> }`.
            BuiltinTy::RefCell => [Some(Item::Ty(crate::ty::int(IntTy::Isize))), at(0)].into_iter().flatten().collect(),
            // `RefMut { value, borrow, marker: PhantomData<&mut T> }`.
            BuiltinTy::RefMutGuard => tys.first().map(|&t| vec![Item::Ty(self.tys.mk_ref(true, t))]).unwrap_or_default(),
            BuiltinTy::RefGuard | BuiltinTy::Fmt(_) | BuiltinTy::ExitCode | BuiltinTy::LocalKey | BuiltinTy::AccessError => Vec::new(),
            BuiltinTy::Sync(k) => match k {
                SyncTy::Arc => tys.first().map(|&t| vec![Item::CountedInner(t, true), Item::Global]).unwrap_or_default(),
                SyncTy::ArcWeak => vec![Item::Global],
                // `Mutex { inner: futex Mutex (AtomicU32), poison: Flag
                // (AtomicBool), data: UnsafeCell<T> }`; `RwLock` alike. The
                // futex is one Adt level deeper than a `u32` (`Cell<u32>`
                // stands for it): `Option` nested 127 deep around a `Mutex`
                // overflows (fx-traitdepth calibration); the worklist dedups
                // it, so a chain of `Mutex`es counts it once.
                SyncTy::Mutex | SyncTy::RwLock => [
                    Some(Item::Ty(self.tys.mk_builtin1(|b| b.cell, crate::ty::int(IntTy::U32)))),
                    Some(Item::Ty(crate::ty::BOOL)),
                    at(0),
                ]
                .into_iter()
                .flatten()
                .collect(),
                // `OnceLock { once: Once (AtomicU32), value, _marker:
                // PhantomData<T> }`.
                SyncTy::OnceLock => [Some(Item::Ty(crate::ty::int(IntTy::U32))), at(0)].into_iter().flatten().collect(),
                SyncTy::PoisonError | SyncTy::SendError | SyncTy::JoinHandle | SyncTy::ScopedJoinHandle => first(),
                // Channel ends hold a raw counter pointer; guards and the
                // rest hold references or plain data.
                _ => Vec::new(),
            },
            BuiltinTy::Hash(k) => match k {
                // `HashMap { base: hashbrown::HashMap { hash_builder: S,
                // table: RawTable<(K, V), A> { .., alloc: A, marker:
                // PhantomData<(K, V)> } } }`.
                HashTy::HashMap => {
                    let (Some(&key), Some(&val)) = (tys.first(), tys.get(1)) else { return all() };
                    let pair = self.tuple(&[key, val]);
                    [at(2), Some(Item::Global), Some(Item::Ty(pair))].into_iter().flatten().collect()
                }
                HashTy::HashSet => {
                    let Some(&key) = tys.first() else { return all() };
                    let pair = self.tuple(&[key, crate::ty::UNIT]);
                    [at(1), Some(Item::Global), Some(Item::Ty(pair))].into_iter().flatten().collect()
                }
                // `VacantEntry { key: K, .. }`; the occupied form holds a
                // bucket pointer.
                HashTy::Entry | HashTy::VacantEntry => first(),
                HashTy::IntoPairs | HashTy::IntoKeys | HashTy::IntoValues => all(),
                _ => Vec::new(),
            },
            BuiltinTy::Iter(k) => self.iter_items(k, &tys),
        }
    }

    /// The constraint of a builtin iterator: the adapters own their
    /// inner iterators and closures, the owning iterators their items,
    /// and the borrowing ones hold references only.
    fn iter_items(&mut self, k: IterKind, tys: &[Ty]) -> Vec<Item> {
        use IterKind as K;
        let at = |i: usize| tys.get(i).map(|&t| vec![Item::Ty(t)]).unwrap_or_default();
        let two = |i: usize, j: usize| [tys.get(i), tys.get(j)].into_iter().flatten().map(|&t| Item::Ty(t)).collect::<Vec<_>>();
        match k {
            // `PhantomData<&'a T>` / `PhantomData<&'a mut T>`.
            K::SliceIter | K::SliceIterMut => {
                tys.first().map(|&t| vec![Item::Ty(self.tys.mk_ref(k == K::SliceIterMut, t))]).unwrap_or_default()
            }
            K::VecIntoIter
            | K::Range
            | K::RangeInclusive
            | K::RangeFrom
            | K::RangeTo
            | K::RangeToInclusive
            | K::Enumerate
            | K::Rev
            | K::Take
            | K::Skip
            | K::Copied
            | K::Cloned
            | K::StepBy
            | K::Peekable
            | K::OptionIntoIter
            | K::ResultIntoIter
            | K::TryCollect
            | K::StrSplit
            | K::StrMatches => at(0),
            K::Map | K::Filter | K::Chain | K::Zip | K::FilterMap | K::MapWhile | K::TakeWhile | K::SkipWhile | K::Inspect | K::Flatten => two(0, 1),
            K::SliceSplit | K::SliceSplitMut | K::ChunkBy | K::ChunkByMut => at(1),
            K::MapIntoIter | K::MapIntoKeys | K::MapIntoValues | K::MapDrain => {
                let pair = self.tuple(tys);
                vec![Item::Ty(pair)]
            }
            K::SetIntoIter | K::SetDrain => match tys.first() {
                Some(&t) => vec![Item::Ty(self.tuple(&[t, crate::ty::UNIT]))],
                None => Vec::new(),
            },
            K::RecvIter
            | K::RecvTryIter
            | K::RecvIntoIter
            | K::MapIter
            | K::MapIterMut
            | K::MapKeys
            | K::MapValues
            | K::MapValuesMut
            | K::SetIter
            | K::ArrayIntoIter
            | K::RepeatN
            | K::Windows
            | K::CharIndices
            | K::OptionIter
            | K::OptionIterMut
            | K::ResultIter
            | K::ResultIterMut
            | K::EncodeUtf16
            | K::VecDrain
            | K::StringDrain
            | K::RangeFull
            | K::Chunks
            | K::ChunksMut
            | K::ChunksExact
            | K::ChunksExactMut
            | K::Utf8Chunks
            | K::Utf8Chunk
            | K::Args => Vec::new(),
            // Handled before the table (`adt`).
            K::ByNext => at(0),
        }
    }

    fn tuple(&mut self, ts: &[Ty]) -> Ty {
        let id = self.tys.intern_tys(ts);
        self.tys.intern(TyKind::Tuple(id))
    }

    fn u8_item(&self) -> Item {
        Item::Ty(crate::ty::int(IntTy::U8))
    }

    /// The printed name of an overflowed entry, as rustc names it.
    fn item_name(&self, i: Item) -> String {
        let name = |t: Ty| short_name(self.p, &*self.tys, t);
        match i {
            Item::Ty(t) => name(t),
            Item::Global => "alloc::alloc::Global".to_string(),
            Item::CountedInner(t, false) => format!("rc::RcInner<{}>", name(t)),
            Item::CountedInner(t, true) => format!("sync::ArcInner<{}>", name(t)),
            Item::NodeRef(..) | Item::OwnedLeaf => "alloc::collections::btree::node::marker::Owned".to_string(),
            Item::SetBox(t) | Item::SetPair(t) => format!("({}, SetValZST)", name(t)),
        }
    }
}

/// `ty_name`, or `_` for a type too large to print: a doubling type
/// argument (`(T, T)` at each level) has an exponential printed size.
pub(crate) fn short_name(p: &Program, tys: &impl TyIntern, t: Ty) -> String {
    fn small(tys: &impl TyIntern, t: Ty, budget: &mut u32) -> bool {
        if *budget == 0 {
            return false;
        }
        *budget -= 1;
        match tys.kind(t) {
            TyKind::Ref(_, _, e) | TyKind::RawPtr(_, e) | TyKind::Slice(e) | TyKind::Array(e, _) => small(tys, e, budget),
            TyKind::Tuple(ts) => tys.tys_of(ts).iter().all(|&e| small(tys, e, budget)),
            TyKind::Adt(_, a) | TyKind::Closure(_, a) | TyKind::Projection(_, a) | TyKind::Opaque(_, a) | TyKind::FnDef(_, a) => {
                tys.args(a).iter().filter_map(|g| g.as_ty()).all(|e| small(tys, e, budget))
            }
            _ => true,
        }
    }
    if small(tys, t, &mut 64) { crate::check::collect::ty_name(p, tys, t) } else { "_".to_string() }
}

/// Appends an Adt's types to a step (the worklist set dedups them,
/// as in rustc).
fn push_dedup(c: &mut Constraint, items: Vec<Item>) {
    c.items.extend(items);
}
