// Derive reads (PLAN R12, package P3). A derive list answers many
// questions a body check asks without naming the Adt: is `(A, u8)` Copy,
// can `Option<A>` be cloned, does `A == A` have glue, does a `#[derive]`
// impl select. Every reader notes the Adt's template def in a per-thread
// log while a scope is open: `Program::derives_of` (the only way to read
// a derive list), and the trait index when it hands out a derived impl
// record (traits/index.rs `get`). Each body check (and its lowering and
// borrowck) runs inside a scope and stores one `ReadKey::Derives` read
// per noted def, so a derive edit rechecks the bodies that asked
// (driver/graph.rs). Trait selection records the same read for its Self
// head (`Program::impl_set`), which covers a derive the Adt lacks.
//
// The log is per thread because the walkers that read derives (is_copy,
// glue_*, has_default, ...) take no read sink, and body checks run in
// parallel. Outside a scope (collect's own passes, a one-shot compile)
// a note costs one thread-local load.
use crate::check::collect::{Derives, Program};
use crate::def::DefId;
use crate::driver::graph::Reads;
use crate::intern::Sym;
use crate::traits::ImplId;
use crate::traits::index::{ImplRecord, SelfHead};
use crate::ty::{AdtId, ArgsId, Predicate, TraitRef, Ty, TyIntern, TyKind};
use std::cell::{Cell, RefCell};

thread_local! {
    /// Open scopes on this thread; zero means nothing is logged.
    static DEPTH: Cell<u32> = const { Cell::new(0) };
    /// The defs noted since the outermost scope opened, each with the
    /// derive traits read (a `DeriveBits` mask), with repeats.
    static LOG: RefCell<Vec<(DefId, u16)>> = const { RefCell::new(Vec::new()) };
}

/// One bit per derivable trait, in `derive_traits` order (Clone, Copy,
/// Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default): a reader that
/// asks one trait notes one bit, so a toggle of another trait leaves its
/// body alone (sigedit P3.2). `ALL` is a reader of the whole list.
pub(crate) mod bit {
    pub(crate) const CLONE: u16 = 1 << 0;
    pub(crate) const COPY: u16 = 1 << 1;
    pub(crate) const DEBUG: u16 = 1 << 2;
    pub(crate) const PARTIAL_EQ: u16 = 1 << 3;
    pub(crate) const EQ: u16 = 1 << 4;
    pub(crate) const HASH: u16 = 1 << 5;
    pub(crate) const PARTIAL_ORD: u16 = 1 << 6;
    pub(crate) const ORD: u16 = 1 << 7;
    pub(crate) const DEFAULT: u16 = 1 << 8;
    pub(crate) const ALL: u16 = (1 << 9) - 1;
    /// No derive trait: a read of the written std and lang trait impls
    /// that name the Adt (traits/index.rs `get`, `ReadKey::LangImpls`).
    pub(crate) const LANG: u16 = 1 << 15;
    /// No derive trait: a read of the Adt's deep layout
    /// (`ReadKey::Layout`), by a reader that sees it through a pointer
    /// (`Program::note_adt_layout`).
    pub(crate) const LAYOUT: u16 = 1 << 14;
    /// No derive trait: a read of the Adt's drop-check constraint
    /// (`ReadKey::DropCk`, check/dropck.rs).
    pub(crate) const DROPCK: u16 = 1 << 13;
}

/// The derive bit of lang trait `tr`, or 0 for a trait no derive makes.
pub(crate) fn trait_bit(p: &Program, tr: DefId) -> u16 {
    derive_traits(p).iter().position(|&(d, _)| d == tr).map_or(0, |k| 1 << k)
}

/// The flags of `d` that `mask` selects, as bits.
pub(crate) fn masked(d: &Derives, mask: u16) -> u16 {
    let flags = [d.clone, d.copy, d.debug, d.partial_eq, d.eq, d.hash, d.partial_ord, d.ord, d.default];
    flags.iter().enumerate().filter(|&(k, &on)| on && mask & (1 << k) != 0).fold(0, |acc, (k, _)| acc | (1 << k))
}

/// True while a scope is open on this thread.
pub(crate) fn open() -> bool {
    DEPTH.with(Cell::get) != 0
}

/// Runs `f` with this thread's log closed and set aside: a fingerprint
/// that computes a structural answer (`ReadKey::Answer`) inside a body's
/// scope notes nothing into that body.
pub(crate) fn unlogged<R>(f: impl FnOnce() -> R) -> R {
    struct Back(u32, Vec<(DefId, u16)>);
    impl Drop for Back {
        fn drop(&mut self) {
            DEPTH.with(|d| d.set(self.0));
            LOG.with(|l| *l.borrow_mut() = std::mem::take(&mut self.1));
        }
    }
    let _back = Back(DEPTH.with(|d| d.replace(0)), LOG.with(|l| std::mem::take(&mut *l.borrow_mut())));
    f()
}

/// Notes a read of the derive traits `mask` of the Adt template `def`.
pub(crate) fn note_def(def: DefId, mask: u16) {
    if !open() {
        return;
    }
    LOG.with(|l| {
        let mut l = l.borrow_mut();
        if l.last() != Some(&(def, mask)) {
            l.push((def, mask));
        }
    });
}

/// `note_def` for each of `defs` with one mask (the index's `LangImpls`
/// notes), with one scope test for all.
pub(crate) fn note_defs(defs: &[DefId], mask: u16) {
    if defs.is_empty() || !open() {
        return;
    }
    LOG.with(|l| l.borrow_mut().extend(defs.iter().map(|&d| (d, mask))));
}

/// Notes a read of the derive traits `mask` of Adt `a` (its template's,
/// for an instance). Called by `Program::derives_of` and `derive_has`.
pub(crate) fn note(p: &Program, a: AdtId, mask: u16) {
    if open()
        && let Some(def) = p.derive_def(a)
    {
        note_def(def, mask);
    }
}

/// Runs `f` with the derive log open and returns the Adts it noted, each
/// with the union of the trait bits read (sorted, one entry per def). Scopes nest: an inner scope's notes also count
/// for every outer scope (a closure body checks as its parent; a body
/// check runs inside the driver's per-body scope). With `on` false (a
/// sink nobody replays) `f` runs with no scope and nothing is noted.
pub(crate) fn scoped<R>(on: bool, f: impl FnOnce() -> R) -> (R, Vec<(DefId, u16)>) {
    if !on {
        return (f(), Vec::new());
    }
    let start = LOG.with(|l| l.borrow().len());
    DEPTH.with(|d| d.set(d.get() + 1));
    // The guard closes the scope on unwind too (a checker panic caught
    // by the daemon must not leave this thread logging).
    struct Close;
    impl Drop for Close {
        fn drop(&mut self) {
            let depth = DEPTH.with(|d| {
                d.set(d.get() - 1);
                d.get()
            });
            if depth == 0 {
                LOG.with(|l| l.borrow_mut().clear());
            }
        }
    }
    let close = Close;
    let out = f();
    let mut noted = LOG.with(|l| l.borrow()[start..].to_vec());
    drop(close);
    noted.sort_unstable_by_key(|(d, _)| (d.krate.0, d.hash.0));
    noted.dedup_by(|b, a| {
        let same = a.0 == b.0;
        if same {
            a.1 |= b.1;
        }
        same
    });
    (out, noted)
}

/// Records one read per noted def into `reads`: `Derives` for a reader of
/// the whole list, else `DeriveBits` of the traits it asked; `LangImpls`
/// for a def noted with `bit::LANG`, and `Layout` with `bit::LAYOUT`.
pub(crate) fn record(p: &Program, reads: &mut Reads, noted: &[(DefId, u16)]) {
    for &(def, mask) in noted {
        if mask & bit::LANG != 0 {
            p.record_lang_impls(reads, def);
        }
        if mask & bit::LAYOUT != 0 {
            let _ = p.layout_of(reads, def);
        }
        if mask & bit::DROPCK != 0 {
            p.record_drop_check(reads, def);
        }
        match mask & !(bit::LANG | bit::LAYOUT | bit::DROPCK) {
            0 => {}
            bit::ALL => p.record_derives(reads, def),
            m => p.record_derive_bits(reads, def, m),
        }
    }
}

/// The derivable traits in `add_synthetic_impls` order, with each one's
/// flag in a `Derives`.
fn derive_traits(p: &Program) -> [(DefId, fn(&Derives) -> bool); 9] {
    let l = p.gx.lang;
    [
        (l.clone, |d| d.clone),
        (l.copy, |d| d.copy),
        (l.debug, |d| d.debug),
        (l.partial_eq, |d| d.partial_eq),
        (l.eq, |d| d.eq),
        (l.hash, |d| d.hash),
        (l.partial_ord, |d| d.partial_ord),
        (l.ord, |d| d.ord),
        (l.default, |d| d.default),
    ]
}

/// The derive list collect would give the Adt from its new derive names,
/// on top of the old list (`default_variant` stands: the enum's
/// `#[default]` marks are outside the edit). Err when collect would
/// report an error (a name it cannot derive, a name given twice), or for a
/// `Default` toggle on an enum (its `#[default]` rules).
pub(crate) fn derives_from_names(p: &Program, old: Derives, is_enum: bool, names: &[Sym]) -> Result<Derives, &'static str> {
    let mut d = Derives { default_variant: old.default_variant, ..Derives::default() };
    for &name in names {
        let flag = match p.interner.get(name) {
            "Debug" => &mut d.debug,
            "Clone" => &mut d.clone,
            "Copy" => &mut d.copy,
            "PartialEq" => &mut d.partial_eq,
            "Eq" => &mut d.eq,
            "Hash" => &mut d.hash,
            "PartialOrd" => &mut d.partial_ord,
            "Ord" => &mut d.ord,
            "Default" => &mut d.default,
            _ => return Err("derive name"),
        };
        // A trait derived twice is rustc's E0119; the flags cannot tell, so
        // the full path answers.
        if *flag {
            return Err("derive duplicate");
        }
        *flag = true;
    }
    if is_enum && d.default != old.default {
        return Err("derive default");
    }
    Ok(d)
}

/// What derive deltas wrote to a kept program, oldest first, so a refused
/// delta can put the program back as it was (`restore`): each Adt row's
/// old derive list, and each synthetic record's old dead flag (a record
/// made for a trait the Adt never derived was dead before: it did not
/// exist). The caller then keeps the program for the next request, as a
/// refused header delta does (sigedit R9).
#[derive(Default)]
pub(crate) struct Undo {
    rows: Vec<(usize, Derives)>,
    records: Vec<(ImplId, bool)>,
}

impl Undo {
    /// Puts back every derive list and record flag, newest first, and the
    /// glue tables collect builds from them.
    pub(crate) fn restore(self, p: &mut Program) {
        if self.rows.is_empty() && self.records.is_empty() {
            return;
        }
        for (id, dead) in self.records.into_iter().rev() {
            p.gx.index.set_dead(id, dead);
        }
        for (i, d) in self.rows.into_iter().rev() {
            p.adts[i].set_derives(d);
        }
        crate::check::generics::compute_glue_tables(p);
    }
}

/// Applies a derive list edit of the local Adt template `a` to a kept
/// program (sigedit PLAN R13) in place of a collect: the Adt and its
/// instance rows take the new list, each toggled trait's synthetic
/// record goes dead or comes back (no id or ordinal moves), and every
/// collect check a derive list feeds runs again. A trait the Adt never
/// derived in this program gets its record from collect's own kernel
/// (generics.rs `insert_derived`), appended with a new id: no other id
/// moves. Err names why the delta cannot stand (the caller collects in
/// full): an added trait with a written impl for the Adt (E0119), a user
/// impl with associated types whose bounds name a toggled trait and whose
/// projections can involve the reach (`assoc_impl_involved`), a written
/// projection type that names the reach (its signature was normalized),
/// or any diagnostic. Every write goes to `undo` first; on Err the caller
/// restores it. Ok gives the delta's reach (the keyed dirty walk reads
/// it), or None when no flag moved.
pub(crate) fn apply(p: &mut Program, a: AdtId, new: Derives, undo: &mut Undo) -> Result<Option<Reach>, &'static str> {
    let def = p.derive_def(a).ok_or("derive adt")?;
    if p.gx.adt_instance_of.get(a.0 as usize).copied().flatten().is_some() {
        return Err("derive adt");
    }
    let old = p.adt(a).derives_unlogged();
    // Each toggle: its trait, its flag order, and its record (None: a
    // trait the Adt never derived in this program, made below).
    let mut toggles: Vec<(DefId, usize, Option<ImplId>, bool)> = Vec::new();
    for (k, (tr, on)) in derive_traits(p).into_iter().enumerate() {
        if on(&old) == on(&new) {
            continue;
        }
        let on = on(&new);
        let id = p.gx.index.derived_record(def, tr);
        match id {
            Some(id) if on == p.gx.index.is_dead(id) => {}
            None if on => {}
            _ => return Err("derive state"),
        }
        // An added derive next to a written impl of the trait for the Adt
        // conflicts (E0119); the full path answers.
        if on && p.gx.index.bucket(tr, SelfHead::Adt(def)).iter().any(|&o| Some(o) != id) {
            return Err("derive written impl");
        }
        toggles.push((tr, k, id, on));
    }
    if toggles.is_empty() {
        return Ok(None);
    }
    let reach = Reach::of(p, def);
    crate::driver::laps::mark("kept/derive reach");
    // A signature normalized through such an impl's associated type would
    // keep its old type (the delta does not normalize again).
    if toggles.iter().any(|&(tr, ..)| assoc_impl_involved(p, tr, &reach)) {
        return Err("derive assoc impl");
    }
    for i in 0..p.adts.len() {
        if i == a.0 as usize || p.gx.adt_instance_of.get(i).copied().flatten().is_some_and(|(t, _)| t == def) {
            undo.rows.push((i, p.adts[i].derives_unlogged()));
            p.adts[i].set_derives(new);
        }
    }
    for (_, k, id, on) in toggles {
        let id = match id {
            Some(id) => id,
            // Ids below `record_count` stand; the def's ordinal counts
            // down from u32::MAX, past every ordinal collect gives out.
            None => {
                let mut ordinal = u32::MAX - p.gx.index.record_count() as u32;
                let id = crate::check::generics::insert_derived(p, a.0 as usize, k, &mut ordinal).ok_or("derive adt")?;
                undo.records.push((id, true));
                continue;
            }
        };
        // Its old dead flag: dead when the toggle puts it back.
        undo.records.push((id, on));
        p.gx.index.set_dead(id, !on);
    }
    // Collect normalized these written types under the old derive lists.
    if !written_projections_hold(p, &reach) {
        return Err("derive projection");
    }
    let mut errs = Vec::new();
    crate::check::generics::recheck_derive_items(p, &reach, &mut errs);
    if errs.is_empty() {
        return Ok(Some(reach));
    }
    let name = p.display_name(p.adt(a).name).to_string();
    log(&|| errs.iter().take(3).map(|e| format!("derive diag {name}: {} {}\n", e.code, e.message)).collect());
    Err("derive diag")
}

/// True when every written type collect normalized (`written_projections`)
/// that names the reach normalizes, under the new derive lists, to the
/// type the program holds: a fn signature (under the fn's env) or an Adt's
/// fields (under the Adt's env). Their well-formedness checks again on the
/// written types (`check_written_wf`). Any other written type that names
/// the reach (an impl's Self type, a const item's type) answers false.
fn written_projections_hold(p: &mut Program, reach: &Reach) -> bool {
    let named: Vec<(DefId, Vec<Ty>)> =
        p.gx.written_projections
            .iter()
            .filter(|(_, tys)| tys.iter().any(|&t| reach.ty(&p.tys, t)))
            .map(|(&d, tys)| (d, tys.clone()))
            .collect();
    if named.is_empty() {
        return true;
    }
    for (d, written) in named {
        let (env, now): (Vec<Predicate>, Vec<Ty>) = if let Some(i) = p.fn_defs.iter().position(|&f| f == d) {
            let env = p.gx.fn_scopes.get(i).map(|s| s.env.clone()).unwrap_or_default();
            (env, p.fns[i].params.iter().copied().chain([p.fns[i].ret]).collect())
        } else if let Some(a) = p.defs.local_adt(d) {
            let a = a.0 as usize;
            (
                p.gx.adt_envs.get(a).cloned().unwrap_or_default(),
                p.adts[a].variants.iter().flat_map(|v| v.fields.iter().map(|f| f.ty)).collect(),
            )
        } else {
            log(&|| format!("derive projection: {d:?} is no fn or Adt\n"));
            return false;
        };
        if now.len() != written.len() {
            return false;
        }
        for (w, n) in written.into_iter().zip(now) {
            let t = crate::check::generics::normalize_global_in(p, &env, w);
            if t != n {
                log(&|| {
                    let name = |t| crate::check::collect::ty_name(p, &p.tys, t);
                    format!("derive projection: {d:?} {} now {} was {}\n", name(w), name(t), name(n))
                });
                return false;
            }
        }
    }
    true
}

/// `LUST_DERIVE_LOG=<file>`: appends why a kept-program delta fell back
/// (a diagnostic, a discovery output that differs). `lines` runs only when
/// the log is on.
pub(crate) fn log(lines: &dyn Fn() -> String) {
    if let Some(path) = std::env::var_os("LUST_DERIVE_LOG") {
        use std::io::Write as _;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            let _ = f.write_all(lines().as_bytes());
        }
    }
}

/// True when a user blanket impl (a written local impl whose Self is a
/// bare or wrapped parameter) has a bound that names trait `tr`, itself
/// or through its supertraits: whether a type implements the blanket's
/// trait then follows its derive list. R13 v1 refused these toggles; v2
/// takes them (the census counts them): the reach-scoped coherence,
/// obligation and WF reruns cover what collect decides, and the blanket's
/// `ImplSet` reads for the Adt fold its derive bits (graph.rs).
fn blanket_names(p: &Program, tr: DefId) -> bool {
    use crate::traits::index::Synthetic;
    p.gx.index.all_impls().any(|(_, rec)| {
        rec.blanket.is_some()
            && rec.synthetic == Synthetic::None
            && rec.local
            && !p.std_source_span(rec.span)
            && p.shipped_crate_at(rec.span).is_none()
            && rec.generics.predicates.iter().any(|pred| match pred {
                crate::ty::Predicate::Trait(b) => trait_reaches(p, b.value.def, tr, &mut Vec::new()),
                _ => false,
            })
    })
}

/// True when a user impl with associated types (a written local impl)
/// has a bound that names trait `tr`, itself or through its supertraits,
/// and a type collect normalized may have gone through it with a reach
/// Adt in its args: that type may hold an associated type the impl no
/// longer gives (the delta does not normalize again). Such a type is the
/// impl's own header, bounds or associated types, when they name the
/// reach, or a projection of the impl's trait whose Self type can select
/// the impl, in the written types (`written_projections`) of a fn or Adt
/// template with an instance whose args name the reach (instances
/// normalize after substitution). Written types that name the reach
/// refuse on their own ("derive projection"); a body's projections are
/// its reads, and its check runs again when one moves.
fn assoc_impl_involved(p: &Program, tr: DefId, reach: &Reach) -> bool {
    use crate::traits::index::Synthetic;
    let impls: Vec<&ImplRecord> = p
        .gx
        .index
        .all_impls()
        .map(|(_, rec)| rec)
        .filter(|rec| {
            !rec.assoc_tys.is_empty()
                && rec.synthetic == Synthetic::None
                && rec.local
                && !p.std_source_span(rec.span)
                && p.shipped_crate_at(rec.span).is_none()
                && rec.generics.predicates.iter().any(|pred| match pred {
                    crate::ty::Predicate::Trait(b) => trait_reaches(p, b.value.def, tr, &mut Vec::new()),
                    _ => false,
                })
        })
        .collect();
    if impls.is_empty() {
        return false;
    }
    if impls.iter().any(|rec| reach.record(&p.tys, rec)) {
        return true;
    }
    let instances = p.gx.instance_by_key.keys().copied().chain(p.gx.adt_instance_of.iter().flatten().copied());
    instances.into_iter().any(|(tdef, args)| {
        reach.args(&p.tys, args)
            && p.gx.written_projections.get(&tdef).is_some_and(|tys| tys.iter().any(|&t| projection_may_select(p, t, args, &impls)))
    })
}

/// True when type `t` (written in a template, `args` its instance's
/// args) holds a projection that may select one of `impls`: its trait is
/// an impl's trait, and its Self type, with the template's params taken
/// from `args`, has that impl's head, or a head no index lookup knows
/// (a projection, an opaque, a param). An impl of a trait with an
/// associated type that is itself a projection can pass any projection
/// on, so every projection of its trait counts.
fn projection_may_select(p: &Program, t: Ty, args: ArgsId, impls: &[&ImplRecord]) -> bool {
    let tys = &p.tys;
    let walk = |e: Ty| projection_may_select(p, e, args, impls);
    match tys.kind(t) {
        TyKind::Projection(assoc, pa) => {
            let Some(tr) = p.gx.index.trait_of_assoc(assoc.0) else { return true };
            let self_ty = tys.args(pa).first().and_then(|g| g.as_ty()).map(|s| match tys.kind(s) {
                TyKind::Param(i) => tys.args(args).get(i.0 as usize).and_then(|g| g.as_ty()),
                _ => Some(s),
            });
            let head = self_ty.flatten().and_then(|s| crate::traits::index::head_of(tys, s, &|x| x));
            let selects = |rec: &&ImplRecord| {
                rec.trait_def == tr
                    && (rec.blanket.is_some() || head.is_none_or(|h| h == rec.head) || rec.assoc_tys.iter().any(|&(_, a)| tys.flags(a).contains_projection()))
            };
            impls.iter().any(selects) || tys.args(pa).iter().any(|g| g.as_ty().is_some_and(walk))
        }
        TyKind::Ref(_, _, e) | TyKind::RawPtr(_, e) | TyKind::Slice(e) | TyKind::Array(e, _) => walk(e),
        TyKind::Tuple(ts) => tys.tys_of(ts).iter().any(|&e| walk(e)),
        TyKind::Adt(_, a) | TyKind::FnDef(_, a) | TyKind::Closure(_, a) | TyKind::Opaque(_, a) => tys.args(a).iter().any(|g| g.as_ty().is_some_and(walk)),
        TyKind::FnPtr(sig) => {
            let sig = tys.sig(sig).value;
            tys.tys_of(sig.inputs).iter().any(|&e| walk(e)) || walk(sig.output)
        }
        // A projection in a dyn type's bindings, conservatively.
        TyKind::Dyn(..) => tys.flags(t).contains_projection(),
        _ => false,
    }
}

/// The derive bits of the derivable traits that `preds` name, directly or
/// through supertraits (a blanket impl's bounds; graph.rs folds them into
/// its `ImplSet` fingerprint for an Adt head).
pub(crate) fn bound_mask(p: &Program, preds: &[crate::ty::Predicate]) -> u16 {
    let mut mask = 0;
    for pred in preds {
        if let crate::ty::Predicate::Trait(b) = pred {
            for (k, &(t, _)) in derive_traits(p).iter().enumerate() {
                if mask & (1 << k) == 0 && trait_reaches(p, b.value.def, t, &mut Vec::new()) {
                    mask |= 1 << k;
                }
            }
        }
    }
    mask
}

/// True when trait `from` is `to` or has it among its supertraits.
fn trait_reaches(p: &Program, from: DefId, to: DefId, seen: &mut Vec<DefId>) -> bool {
    if from == to {
        return true;
    }
    if seen.contains(&from) {
        return false;
    }
    seen.push(from);
    let Some(rec) = p.gx.index.trait_record(from) else { return false };
    rec.generics.predicates.iter().any(|pred| match pred {
        crate::ty::Predicate::Trait(b) => trait_reaches(p, b.value.def, to, seen),
        _ => false,
    })
}

/// What a derive delta of one Adt can reach (PLAN R13): `defs` holds the
/// Adt's template def and every Adt template with a row whose field
/// types name one of them, to a fixpoint (`field_users`, transitive: the
/// glue walkers and derive bound checks look through fields). A check
/// item that names none of `defs` in its types or bounds reads no derive
/// list the delta moved, so its answer stands; the scoped reruns
/// (generics.rs `recheck_derive_items`) take only items that name one.
pub(crate) struct Reach {
    defs: crate::intern::FxMap<DefId, ()>,
    memo: RefCell<crate::intern::FxMap<Ty, bool>>,
    /// The instance rows at each derive mark (`cold_row`), built on use.
    cold: RefCell<[Option<crate::intern::FxMap<u32, ()>>; 3]>,
}

impl Reach {
    /// A reach over `defs` (another reach's, `defs()`) with empty memos,
    /// for one thread of a parallel walk (`Program::moved_fn_names`): the
    /// same answers as the reach it copies.
    pub(crate) fn with_defs(defs: crate::intern::FxMap<DefId, ()>) -> Reach {
        Reach { defs, memo: Default::default(), cold: Default::default() }
    }

    /// The reach of the Adt template `def`: field users over every Adt
    /// row (templates and instances; an instance adds its template).
    pub(crate) fn of(p: &Program, def: DefId) -> Reach {
        let mut r = Reach { defs: Default::default(), memo: Default::default(), cold: Default::default() };
        r.defs.insert(def, ());
        loop {
            let mut grew = false;
            for i in 0..p.adts.len() {
                let Some(d) = p.derive_def(AdtId(i as u32)) else { continue };
                if r.defs.contains_key(&d) {
                    continue;
                }
                if p.adts[i].variants.iter().any(|v| v.fields.iter().any(|f| r.ty(&p.tys, f.ty))) {
                    r.defs.insert(d, ());
                    // A new member: earlier "no" answers may now be "yes".
                    r.memo.borrow_mut().clear();
                    grew = true;
                }
            }
            if !grew {
                return r;
            }
        }
    }

    /// The reach of two deltas in one request.
    pub(crate) fn union(self, other: Reach) -> Reach {
        let mut defs = self.defs;
        defs.extend(other.defs);
        Reach { defs, memo: Default::default(), cold: Default::default() }
    }

    /// The number of Adt templates in the reach (the census reports it).
    pub(crate) fn len(&self) -> usize {
        self.defs.len()
    }

    /// True when Adt row `i` existed at collect's derive check `mark`
    /// (`GenTables::derive_marks`): a template, or an instance made
    /// before that point.
    pub(crate) fn cold_row(&self, p: &Program, i: usize, mark: usize) -> bool {
        if p.gx.adt_instance_of.get(i).copied().flatten().is_none() {
            return true;
        }
        let created = &p.gx.adt_created[..p.gx.derive_marks[mark].min(p.gx.adt_created.len())];
        let mut cache = self.cold.borrow_mut();
        let set = cache[mark].get_or_insert_with(|| created.iter().map(|a| (a.0, ())).collect());
        set.contains_key(&(i as u32))
    }

    /// The Adt templates of the reach (a `Sync` view for parallel walks).
    pub(crate) fn defs(&self) -> &crate::intern::FxMap<DefId, ()> {
        &self.defs
    }

    /// True when Adt row `i` (a template or an instance) is in the reach.
    pub(crate) fn row(&self, p: &Program, i: usize) -> bool {
        p.derive_def(AdtId(i as u32)).is_some_and(|d| self.defs.contains_key(&d))
    }

    /// True when type `t` names an Adt of the reach anywhere inside it.
    pub(crate) fn ty(&self, tys: &impl TyIntern, t: Ty) -> bool {
        if let Some(&b) = self.memo.borrow().get(&t) {
            return b;
        }
        let b = match tys.kind(t) {
            TyKind::Ref(_, _, e) | TyKind::RawPtr(_, e) | TyKind::Slice(e) | TyKind::Array(e, _) => self.ty(tys, e),
            TyKind::Tuple(ts) => tys.tys_of(ts).iter().any(|&e| self.ty(tys, e)),
            TyKind::Adt(d, a) => self.defs.contains_key(&d) || self.args(tys, a),
            TyKind::FnDef(_, a) | TyKind::Closure(_, a) | TyKind::Opaque(_, a) | TyKind::Projection(_, a) => self.args(tys, a),
            TyKind::FnPtr(s) => {
                let sig = tys.sig(s).value;
                tys.tys_of(sig.inputs).iter().any(|&e| self.ty(tys, e)) || self.ty(tys, sig.output)
            }
            TyKind::Dyn(b, _) => {
                let b = tys.dyn_bounds(b);
                let refs: Vec<TraitRef> = b.principal.iter().chain(b.supertraits.iter()).map(|x| x.value).collect();
                refs.iter().any(|tr| self.trait_ref(tys, tr))
            }
            _ => false,
        };
        self.memo.borrow_mut().insert(t, b);
        b
    }

    /// True when generic args `a` name an Adt of the reach.
    pub(crate) fn args(&self, tys: &impl TyIntern, a: ArgsId) -> bool {
        tys.args(a).iter().any(|g| g.as_ty().is_some_and(|t| self.ty(tys, t)))
    }

    /// True when a trait ref (its args and bindings) names an Adt of the reach.
    pub(crate) fn trait_ref(&self, tys: &impl TyIntern, tr: &TraitRef) -> bool {
        self.args(tys, tr.args) || tys.bindings(tr.bindings).iter().any(|(_, g)| g.as_ty().is_some_and(|t| self.ty(tys, t)))
    }

    /// True when a predicate names an Adt of the reach.
    pub(crate) fn pred(&self, tys: &impl TyIntern, pred: &Predicate) -> bool {
        match pred {
            Predicate::Trait(b) => self.trait_ref(tys, &b.value),
            Predicate::AssocEq(b) => self.args(tys, b.value.trait_args) || self.ty(tys, b.value.ty),
            Predicate::TypeOutlives(t, _) => self.ty(tys, *t),
            _ => false,
        }
    }

    /// True when an impl record's header, bounds, or associated types name
    /// an Adt of the reach.
    pub(crate) fn record(&self, tys: &impl TyIntern, rec: &ImplRecord) -> bool {
        self.trait_ref(tys, &rec.trait_ref)
            || rec.generics.predicates.iter().any(|q| self.pred(tys, q))
            || rec.assoc_tys.iter().any(|&(_, t)| self.ty(tys, t))
    }
}

/// `LUST_DERIVE_CENSUS=1`: after a clean collect, prints to stderr what
/// the derive delta would do with each (local Adt, derived trait) toggle:
/// the toggles a user blanket impl refuses, per trait, and the sizes of
/// the Adts' reaches (`Reach`). Reads only.
pub(crate) fn census_if_asked(p: &Program) {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*ON.get_or_init(|| std::env::var("LUST_DERIVE_CENSUS").is_ok_and(|v| v == "1")) {
        return;
    }
    let traits = derive_traits(p);
    let blanket: Vec<bool> = traits.iter().map(|&(tr, _)| blanket_names(p, tr)).collect();
    let (mut toggles, mut refused) = (vec![0usize; 9], vec![0usize; 9]);
    let mut sizes: Vec<usize> = Vec::new();
    let t0 = std::time::Instant::now();
    for i in 0..p.adts.len() {
        let a = AdtId(i as u32);
        let span = p.adts[i].span;
        if p.gx.adt_instance_of.get(i).copied().flatten().is_some() || p.std_source_span(span) || p.shipped_crate_at(span).is_some() {
            continue;
        }
        let Some(def) = p.derive_def(a) else { continue };
        let d = p.adts[i].derives_unlogged();
        let mut any = false;
        for (k, (_, on)) in traits.iter().enumerate() {
            if on(&d) {
                any = true;
                toggles[k] += 1;
                refused[k] += usize::from(blanket[k]);
            }
        }
        if any {
            sizes.push(Reach::of(p, def).len());
        }
    }
    let names = ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Hash", "PartialOrd", "Ord", "Default"];
    let per: Vec<String> = (0..9).map(|k| format!("{} {}/{}", names[k], refused[k], toggles[k])).collect();
    sizes.sort_unstable();
    let pct = |q: usize| sizes.get(sizes.len().saturating_sub(1) * q / 100).copied().unwrap_or(0);
    eprintln!(
        "derive-census: toggles a user blanket bound names (blanket/toggles): {}; total {}/{}; reach sizes over {} Adts: median {} p95 {} max {}; {:.1} ms",
        per.join(", "),
        refused.iter().sum::<usize>(),
        toggles.iter().sum::<usize>(),
        sizes.len(),
        pct(50),
        pct(95),
        pct(100),
        t0.elapsed().as_secs_f64() * 1000.0
    );
}
