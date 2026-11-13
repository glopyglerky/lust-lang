// The fn signature delta (sigedit PLAN, P2.2-3): header edits of plain
// fns applied to the daemon's kept program in place of a full collect.
// The item patch (driver/patch.rs) has already put the new items in the
// Ast, moved the program's spans, and attached this request's Ast and
// resolver tables. `apply_headers` then does for each edited fn what
// collect does for it, and refuses whatever it cannot prove equal to a
// cold collect:
//
// - the record and the signature checks collect runs, for this fn only
//   (`delta_audit::redo_sig`); an `impl Trait` return of the fn reveals
//   again from its body and its hidden type must stand (`delta_audit::reveal_again`);
// - mono's body checks that read the signature: every plain body whose
//   stored discovery no longer holds checks again and must make the same
//   instances, closures, and vtables. A template whose stored check no
//   longer holds, or that stores none and names an edited fn, checks
//   again in a check (no diagnostic, same hidden type) and refuses in a
//   build.
//   The check and build programs share the stored checks; one stored
//   after the kept program's `reuse_stamp` (by the other mode's request)
//   refuses, since this program may not hold that output.
//
// Also refused (the caller collects in full): a fn whose class moves (a
// template keeps its parameter kinds), and a template in a build or in a
// program with generic associated consts (its instances would keep their
// old signatures); const fns and any fn const evaluation may reach
// through const fn bodies (`ConstReach`, transitive); trait methods; trait
// impl methods in a build, or of a trait whose impls no `ImplSet` read
// covers (`generics::delta_impl_trait`); lang and shipped std fns;
// main and #[test] fns; an `impl Trait` return that comes or goes; a fn
// that a body revealing an opaque type names, unless that body reveals
// again with the same hidden type (a plain fn's, or a template's check in
// a check only): then the auto traits callers read through it, and any
// opaque type that names it, stand too; a header with a macro type, or
// with a `dyn` type when the Ast's `dyn` types no longer name the traits
// the eager vtables started from; programs without user generics (collect's whole-Ast flag
// scans may turn them on); and any diagnostic. The full path answers
// then. A refusal before any write, or a diagnostic of a new signature
// (its old records go back, `delta_audit::undo`), leaves the program as
// it was: `reuse_kept` keeps it for the next request (sigedit R9).
//
// check/delta_audit.rs lists what the delta does with each Program field.
use super::compile::fn_def_at;
use super::const_reach::ConstReach;
use super::session::{Caches, Front};
use crate::check::collect::{FnLoc, Program};
use crate::check::generics::{self, FnClass};
use crate::def::LocalDefId;
use crate::diag::Diag;
use crate::driver::graph::{BindingRes, Head, ReadKey};
use crate::intern::FxMap;

/// One header edit: the edited AST fn (its `LocalDefId` index) and its
/// old body's expression range (a template's picks key on it).
#[derive(Clone, Copy)]
#[allow(dead_code)] // The item patch (sigedit P1.3) builds these in `reuse_kept`.
pub(crate) struct HeaderEdit {
    pub(crate) fn_index: usize,
    pub(crate) old_exprs: (u32, u32),
}

/// Applies header edits to the kept program `p`. `reach` is the patch
/// base's const reach, if it has one. Err names the first condition that
/// failed (the `kept=` of `lust-visits:`), and says whether `p` is as it
/// was (a refusal before any write, a diagnostic of a new signature, or
/// a hidden type that moved before any template checked again: the old
/// records go back, and the caller may keep it for the next request,
/// sigedit R9) or half edited (it must be dropped). On Ok, the errors of
/// the templates that failed their check again are in
/// `Caches::delta_template_errs` (by template), for `reuse_kept` to answer.
#[allow(dead_code)] // The item patch (sigedit P1.3) calls it from `reuse_kept`.
pub(crate) fn apply_headers(
    p: &mut Program,
    front: &Front,
    caches: &mut Caches,
    reach: Option<&ConstReach>,
    edits: &[HeaderEdit],
    walk_instances: bool,
    bases: &mut Bases,
) -> Result<(), (&'static str, bool)> {
    caches.delta_template_errs = Some(Vec::new());
    caches.delta_sig_errs = None;
    let out = apply(p, front, caches, reach, edits, walk_instances, bases);
    let template_errs = caches.delta_template_errs.take().unwrap_or_default();
    if out.is_ok() {
        caches.delta_template_errs = Some(template_errs);
    }
    // The keys this delta can move, for the compile tail's keyed walk.
    if out.is_ok() {
        let mut fns: Vec<usize> = edits.iter().map(|e| e.fn_index).collect();
        fns.sort_unstable();
        fns.dedup();
        caches.header_moves = Some(Touched::new(p, &fns));
    }
    // A program kept after a refusal or a diagnostic keeps the outputs its
    // callers' bodies have now (`own_bases`): the collect that answers this
    // request may store new entries for them.
    if let Err((why, true)) = out {
        let mut fns: Vec<usize> = edits.iter().map(|e| e.fn_index).collect();
        fns.sort_unstable();
        fns.dedup();
        for b in own_bases(p, front, caches, &fns) {
            if !bases.iter().any(|x| x.0 == b.0) {
                bases.push(b);
            }
        }
        return Err((why, true));
    }
    out
}

/// `apply_headers` without the bases of a refused program.
fn apply(
    p: &mut Program,
    front: &Front,
    caches: &mut Caches,
    reach: Option<&ConstReach>,
    edits: &[HeaderEdit],
    walk_instances: bool,
    bases: &Bases,
) -> Result<(), (&'static str, bool)> {
    // Entries this delta stores come after this stamp (`replay_bodies`).
    let start = crate::driver::gen_reuse::stamp_now();
    let instanced = super::patch::instanced_templates(p);
    let fns = untouched_refusals(p, front, reach, edits, walk_instances).map_err(|e| (e, true))?;
    super::laps::mark("collect/delta refusals");
    let opaque_idents = opaque_body_idents(p);
    super::laps::mark("collect/delta opaque idents");
    let (namers, template_namers) = rpit_namers(p, &opaque_idents, &fns, walk_instances).map_err(|e| (e, true))?;
    super::laps::mark("collect/delta rpit namers");
    // The node answers of the last collect describe the old signatures.
    caches.gen_reuse.forget_answers();
    // Every new signature first: a body checked below may call any of them.
    let mut errs: Vec<Diag> = Vec::new();
    let mut old_rets = Vec::with_capacity(fns.len());
    let mut undos = Vec::with_capacity(fns.len());
    // An edited template's generic params: its instances' args fit only
    // while they stand.
    let params_of = |p: &Program, i: usize| {
        p.gx.fn_templates.get(i).and_then(|t| t.as_ref()).map(|t| t.generics.params.iter().map(|d| (d.name, d.kind)).collect::<Vec<_>>())
    };
    let old_params: Vec<_> = fns.iter().map(|&i| params_of(p, i)).collect();
    for &i in &fns {
        let f = LocalDefId(i as u32);
        let rpit = p.gx.opaques.contains_key(&generics::opaque_def(p.def_id(f)));
        old_rets.push((rpit, p.fns[i].ret));
        undos.push(crate::check::delta_audit::sig_undo(p, f));
        let out = crate::check::delta_audit::redo_sig(p, f, rpit, &mut errs);
        let out = out.and_then(|()| if errs.is_empty() { Ok(()) } else { Err("sig diag") });
        if let Err(why) = out.and_then(|()| if order_bound(p, f) { Err("sig trait impl") } else { Ok(()) }) {
            undos.into_iter().rev().for_each(|u| crate::check::delta_audit::undo(p, u));
            // One edited header whose own signature checks fail: those
            // errors are collect's answer (it checks no body after a
            // signature error), `Caches::delta_sig_errs`.
            // Not a method of a trait or a trait impl: collect checks those
            // against each other too, with more errors than the fn's own.
            let parent = p.defs.get(p.def_id(f)).and_then(|d| d.parent);
            let trait_side = parent.is_some_and(|d| p.gx.index.by_def(d).is_some() || p.gx.index.trait_record(d).is_some());
            if why == "sig diag" && fns.len() == 1 && !trait_side {
                caches.delta_sig_errs = Some(std::mem::take(&mut errs));
            }
            return Err((why, true));
        }
    }
    super::laps::mark("collect/delta redo");
    // Until the replay stores its outputs, every write has an undo: the
    // old records and template checks go back and `p` stands.
    let mut undos = Some(Undos { sigs: undos, templates: Vec::new(), instances: Vec::new() });
    for (&i, &(rpit, old_ret)) in fns.iter().zip(&old_rets) {
        let f = LocalDefId(i as u32);
        if p.gx.fn_class[i] == FnClass::Template {
            let ranges: Vec<(u32, u32)> = edits.iter().filter(|e| e.fn_index == i).map(|e| e.old_exprs).collect();
            if let Err(why) = recheck_undoable(p, f, &ranges, caches, &mut undos) {
                return Err(fail(p, caches, &mut undos, why));
            }
        } else if rpit && let Err(why) = crate::check::delta_audit::reveal_again(p, f, old_ret) {
            return Err(fail(p, caches, &mut undos, why));
        }
    }
    // An edited template's instances in a check's program take its new
    // signature, as collect's `fn_instance` would make them; their bodies
    // replay below (`replay_discovery`).
    for (k, &i) in fns.iter().enumerate() {
        let f = LocalDefId(i as u32);
        if !instanced.contains_key(&f) {
            continue;
        }
        if params_of(p, i) != old_params[k] {
            return Err(fail(p, caches, &mut undos, "sig params"));
        }
        let insts: Vec<LocalDefId> = p.gx.instances.iter().filter(|(_, info)| info.template_local == Some(f)).map(|(&g, _)| g).collect();
        for g in insts {
            match generics::refresh_instance(p, g) {
                Ok(u) => undos.as_mut().map_or((), |x| x.instances.push(u)),
                Err(why) => return Err(fail(p, caches, &mut undos, why)),
            }
        }
    }
    // A plain fn that reveals an opaque type and names an edited fn: its
    // hidden type (and so the auto traits callers read through it) must
    // stand under the new signatures. Then the opaque types that name it
    // see no change either.
    for g in namers {
        let (record_reads, unwind) = (p.record_reads, p.unwind);
        (p.record_reads, p.unwind) = (true, false);
        let again = crate::check::opaque::hidden_again(p, g);
        (p.record_reads, p.unwind) = (record_reads, unwind);
        match again {
            crate::check::opaque::HiddenAgain::Stands => {}
            crate::check::opaque::HiddenAgain::Errors => return Err(fail(p, caches, &mut undos, "hidden diag")),
            crate::check::opaque::HiddenAgain::Moved => return Err(fail(p, caches, &mut undos, "hidden type")),
        }
    }
    // A template that does: its check runs again (the hidden type must
    // stand), as for a template that calls an edited fn.
    for &t in &template_namers {
        let (lo, hi) = p.fn_def(t).exprs;
        if let Err(why) = recheck_undoable(p, t, &[(lo, hi)], caches, &mut undos) {
            return Err(fail(p, caches, &mut undos, why));
        }
    }
    super::laps::mark("collect/delta reveal");
    let out = replay_bodies(p, front, caches, &fns, &template_namers, walk_instances, start, bases, &mut undos).map_err(|why| fail(p, caches, &mut undos, why));
    super::laps::mark("collect/delta replay");
    out
}

/// What a header delta wrote before its replay stored anything: the old
/// records of the edited fns and the template checks it ran again, in
/// order. `fail` takes it.
struct Undos {
    sigs: Vec<crate::check::delta_audit::SigUndo>,
    templates: Vec<TemplateUndo>,
    /// Instances of edited templates given the new signature, last.
    instances: Vec<generics::InstanceUndo>,
}

/// The delta's error for `why`: with every write undone (newest first)
/// `p` is as it was and stays for the next request (sigedit R9), else it
/// is half edited.
fn fail(p: &mut Program, caches: &mut Caches, undos: &mut Option<Undos>, why: &'static str) -> (&'static str, bool) {
    let Some(u) = undos.take() else { return (why, false) };
    u.instances.into_iter().rev().for_each(|x| generics::undo_instance(p, x));
    u.templates.into_iter().rev().for_each(|t| undo_template(p, caches, t));
    u.sigs.into_iter().rev().for_each(|s| crate::check::delta_audit::undo(p, s));
    (why, true)
}

/// What one template check writes (`recheck_template`): its picks, closure
/// call kinds, the hidden type of its `impl Trait`, and its output
/// fingerprint for instances (`GenReuse::tpl_out_of`). The entry it may
/// store stays: a later use compares its reads with the program's.
struct TemplateUndo {
    t: LocalDefId,
    picks: Vec<(crate::ast::ExprId, crate::check::picks::TraitPick)>,
    closures: Vec<((LocalDefId, crate::ast::ExprId), (crate::check::closure::ClosureKind, Vec<generics::CaptureShape>))>,
    calls: Vec<((LocalDefId, crate::ast::ExprId), crate::check::closure::ClosureKind)>,
    hidden: (crate::def::DefId, Option<crate::ty::Ty>),
    tpl_out: Option<u64>,
}

/// `recheck_template` that first takes its undo into `undos`. A template
/// with closures has one too: `recheck_template` refuses before any write
/// unless its closures are the registered ones, and then the check's
/// `register_closures` writes the values those records hold.
fn recheck_undoable(p: &mut Program, t: LocalDefId, old: &[(u32, u32)], caches: &mut Caches, undos: &mut Option<Undos>) -> Result<(), &'static str> {
    if let Some(u) = undos.as_mut() {
        let in_old = |e: crate::ast::ExprId| old.iter().any(|r| e.0 >= r.0 && e.0 < r.1);
        let od = generics::opaque_def(p.def_id(t));
        u.templates.push(TemplateUndo {
            t,
            picks: p.gx.trait_picks.iter().filter(|(e, _)| in_old(**e)).map(|(e, k)| (*e, *k)).collect(),
            closures: p.gx.template_closures.iter().filter(|(k, _)| k.0 == t).map(|(k, v)| (*k, v.clone())).collect(),
            calls: p.gx.template_calls.iter().filter(|(k, _)| k.0 == t).map(|(k, v)| (*k, *v)).collect(),
            hidden: (od, p.gx.opaque_hidden.get(&od).copied()),
            tpl_out: caches.gen_reuse.tpl_out_of(t),
        });
    }
    recheck_template(p, t, old, caches)
}

/// Writes back what `recheck_undoable` took: the check's new picks (in the
/// template's current body) go, the old ones return.
fn undo_template(p: &mut Program, caches: &mut Caches, u: TemplateUndo) {
    let (lo, hi) = p.fn_def(u.t).exprs;
    p.gx.trait_picks.retain(|e, _| !(e.0 >= lo && e.0 < hi));
    p.gx.trait_picks.extend(u.picks);
    p.gx.template_closures.retain(|k, _| k.0 != u.t);
    p.gx.template_closures.extend(u.closures);
    p.gx.template_calls.retain(|k, _| k.0 != u.t);
    p.gx.template_calls.extend(u.calls);
    match u.hidden.1 {
        Some(h) => p.gx.opaque_hidden.insert(u.hidden.0, h),
        None => p.gx.opaque_hidden.remove(&u.hidden.0),
    };
    caches.gen_reuse.set_tpl_out(u.t, u.tpl_out);
}

/// The edited fns of `edits`, sorted, or the first refusal that needs no
/// write to the program (`refusal` and the const reach).
fn untouched_refusals(
    p: &Program,
    front: &Front,
    reach: Option<&ConstReach>,
    edits: &[HeaderEdit],
    walk_instances: bool,
) -> Result<Vec<usize>, &'static str> {
    if !p.gx.user_generics {
        return Err("sig flags");
    }
    let built;
    let reach = match reach {
        Some(r) => r,
        None => {
            built = ConstReach::build(front);
            super::laps::mark("collect/delta const reach");
            &built
        }
    };
    let mut fns: Vec<usize> = edits.iter().map(|e| e.fn_index).collect();
    fns.sort_unstable();
    fns.dedup();
    for &i in &fns {
        if i >= front.fn_loc.len() {
            return Err("sig class");
        }
        let f = LocalDefId(i as u32);
        refusal(p, f, walk_instances)?;
        // An inherent method: a root `Q::name` names it only when `Q` may
        // denote its Adt (`ConstReach::reaches_inherent`).
        // `refusal` took every fn that was const; the new header may be.
        let adt = p.gx.inherent_methods.iter().find(|(_, v)| v.contains(&f)).and_then(|(k, _)| p.adt_by_def(k.0));
        let reached = match adt {
            Some(a) => {
                let (item, sub) = front.fn_loc[i];
                reach.reaches_inherent(front, i, p.name(p.adt(a).name), fn_def_at(&front.ast, item, sub).qual.is_const)
            }
            None => reach.reaches(front, i),
        };
        if reached {
            crate::check::derive_delta::log(&|| {
                let why = adt.map_or("free fn".to_string(), |a| reach.inherent_why(front, i, p.name(p.adt(a).name)));
                format!("sig const {}: {why}\n", p.display_name(p.fn_def(f).name))
            });
            return Err("sig const");
        }
    }
    Ok(fns)
}

/// An edited template checks again as collect's template check does
/// (`reuse_kept` does the same for a body edit): its old picks and
/// closure call kinds go, the check records new ones. Its closures must be
/// the registered ones, and its own `impl Trait` must keep its hidden type.
pub(super) fn recheck_template(p: &mut Program, f: LocalDefId, old: &[(u32, u32)], caches: &mut Caches) -> Result<(), &'static str> {
    if !generics::template_closures_stand(p, f) {
        return Err("closures");
    }
    p.gx.trait_picks.retain(|x, _| !old.iter().any(|r| x.0 >= r.0 && x.0 < r.1));
    p.gx.template_closures.retain(|k, _| k.0 != f);
    p.gx.template_calls.retain(|k, _| k.0 != f);
    // Collect's template check runs before mono.
    let (counts, mono_done) = (p.gx.mono_counts, std::mem::replace(&mut p.gx.mono_done, false));
    let od = generics::opaque_def(p.def_id(f));
    let hidden = p.gx.opaque_hidden.get(&od).copied();
    let errs = generics::check_template(p, f, &mut caches.gen_reuse);
    (p.gx.mono_counts, p.gx.mono_done) = (counts, mono_done);
    if !errs.is_empty() {
        // In a header delta the errors are collect's answer (`reuse_kept`);
        // the hidden type test below still refuses an `impl Trait` it moved.
        match caches.delta_template_errs.as_mut() {
            Some(all) => all.push((f.0 as usize, errs)),
            None => return Err("sig diag"),
        }
    }
    if p.gx.opaque_hidden.get(&od).copied() != hidden {
        return Err("hidden type");
    }
    Ok(())
}

/// The identifier sets of the bodies that define an opaque type, by
/// owner def (`refusal` reads them).
fn opaque_body_idents(p: &Program) -> Vec<(crate::def::DefId, Vec<crate::intern::Sym>)> {
    p.gx.opaques
        .values()
        .map(|o| {
            let ids = p.local_def(o.owner.hash).map(|g| super::ast_hash::body_scan(&p.ast, p.fn_def(g), None).1);
            (o.owner, ids.unwrap_or_default())
        })
        .collect()
}

/// Why fn `f` cannot take a header delta, if anything (see the module
/// docs; `apply_headers` adds the const reach and `rpit_namers`). Reads
/// the new Ast and the old program.
fn refusal(p: &Program, f: LocalDefId, walk_instances: bool) -> Result<(), &'static str> {
    let i = f.0 as usize;
    // A template's instances keep their old signatures: a build's refuse.
    // A check's (with generic associated consts, `patch::instanced_templates`)
    // take the new one (`generics::refresh_instance`).
    match p.gx.fn_class.get(i) {
        Some(FnClass::Plain) => {}
        Some(FnClass::Template) if !walk_instances => {}
        _ => return Err("sig class"),
    }
    let info = p.fn_info(f);
    let def = info.def(&p.ast);
    match info.loc {
        FnLoc::Item(_) => {}
        // An inherent method of a local Adt (`inherent_self_pats`).
        FnLoc::Impl(item, _) if p.gx.inherent_self_pats.contains_key(&f) && p.impl_parents.contains_key(&item) => {}
        // A method of an impl of a user trait (`generics::delta_impl_trait`),
        // in a check: `redo_sig` compares it with the trait method again,
        // and every body that reads the trait's impl buckets checks again
        // (`Touched`). A build's instances call it with their old checks;
        // an `impl Trait` return in a trait impl refuses.
        FnLoc::Impl(item, _) if p.impl_parents.contains_key(&item) && is_trait_impl(p, item) => {
            if walk_instances || generics::delta_impl_trait(p, f).is_none() || p.gx.opaques.contains_key(&generics::opaque_def(p.def_id(f))) {
                return Err("sig trait impl");
            }
        }
        FnLoc::Trait(..) => return Err("sig trait"),
        FnLoc::Nested(_) => return Err("sig nested"),
        _ => return Err("sig loc"),
    }
    if p.is_lang_method(f) || p.fn_shipped_crate(f).is_some() || p.std_source_span(def.name_span) {
        return Err("sig std");
    }
    if p.main == Some(f) || def.name == p.known.main || def.is_test {
        return Err("sig main");
    }
    if info.is_const || def.qual.is_const {
        return Err("sig const");
    }
    if p.gx.static_dyn_fns.contains(&f) || p.gx.dyn_inherent.values().any(|&g| g == f) {
        return Err("sig dyn");
    }
    // An `impl Trait` return of the fn itself reveals again from its body
    // (`delta_audit::reveal_again`, or the template check); the new header
    // must keep one (`redo_sig`). Other bodies that reveal one and name
    // the fn: `rpit_namers`.
    // Collect's eager vtables scan every written `dyn` type: a header
    // with one stands while the Ast names the traits the program's pass
    // took (`dyn_stands`). A macro type may expand to anything.
    if let Some(body) = def.body {
        let (lo, hi, file) = (def.name_span.lo, p.ast.blocks[body.0 as usize].span.lo, def.name_span.file);
        let in_header = |s: &crate::diag::Span| s.file == file && s.lo >= lo && s.hi <= hi;
        let tys = || p.ast.tys.iter().zip(&p.ast.ty_spans).filter(|(_, s)| in_header(s)).map(|(t, _)| t);
        if tys().any(|t| matches!(t, crate::ast::TyExpr::MacroCall(_))) {
            return Err("sig dyn");
        }
        if tys().any(|t| matches!(t, crate::ast::TyExpr::Dyn { .. })) && !dyn_stands(p) {
            return Err("sig dyn");
        }
    } else {
        return Err("sig loc");
    }
    Ok(())
}

/// True when the `dyn` types of the new Ast name the traits the program's
/// eager vtable pass started from (`GenTables::eager_dyn_traits`): a cold
/// collect then builds the same eager vtables. The patch blanked the old
/// header's nodes, so the scan sees the live ones only.
fn dyn_stands(p: &Program) -> bool {
    p.gx.mono_done && crate::mono::vtable::dyn_named_traits(p) == p.gx.eager_dyn_traits
}

/// The nodes a header delta of `fns` can move: the edited fns'
/// signatures, and the bindings, method sets and (for a trait impl
/// method) impl buckets of its trait whose answers hash them (graph.rs
/// `binding_fingerprint`, `method_set_fingerprint`, `impl_content`). No
/// other node reads an edited fn's record (every other fn keeps its
/// signature and hidden type). A binding hashes the def it resolves to,
/// and the delta moves no resolution (the item patch keeps the resolver
/// tables, and a header edit keeps the fn's name: `fn_outline`), so only
/// one that resolves to an edited fn moves. A method set holds only the
/// inherent methods of its Adt head (`method_candidates`), so only the
/// sets of an edited method's Adt and name move. A `Checked` read never
/// holds by fingerprint.
/// A successful delta leaves its set in `Caches::header_moves` (the
/// compile tail's keyed walk, read only).
pub(crate) struct Touched {
    defs: Vec<crate::def::DefId>,
    methods: Vec<(crate::def::DefId, crate::intern::Sym)>,
    traits: Vec<crate::def::DefId>,
    seen: crate::intern::FxMap<ReadKey, bool>,
}

impl Touched {
    fn new(p: &Program, fns: &[usize]) -> Touched {
        let defs = fns.iter().map(|&i| p.def_id(LocalDefId(i as u32))).collect();
        Touched { defs, methods: method_sets(p, fns), traits: impl_traits(p, fns), seen: Default::default() }
    }

    /// True when the delta may move the fingerprint of `k`.
    pub(crate) fn hit(&mut self, p: &Program, k: &ReadKey) -> bool {
        if let Some(&h) = self.seen.get(k) {
            return h;
        }
        let h = self.hit_pure(p, k);
        self.seen.insert(*k, h);
        h
    }

    /// `hit` without its memo: a keyed walk asks it for every key the slot
    /// read sets hold at once, on all cores (`keyed_dirty_slots`' quick
    /// answers).
    pub(crate) fn hit_pure(&self, p: &Program, k: &ReadKey) -> bool {
        let (defs, methods, traits) = (&self.defs, &self.methods, &self.traits);
        match k {
            ReadKey::Sig(d) => defs.contains(d),
            ReadKey::MethodSet(m) => matches!(m.head, Head::Adt(a) if methods.contains(&(a, m.name))),
            ReadKey::Binding(b) => matches!(p.binding_res_at(b), BindingRes::Def(d) if defs.contains(&d)),
            ReadKey::Checked(_) => true,
            ReadKey::ImplSet(s) => traits.contains(&s.trait_),
            ReadKey::AssocVis(a, name) => methods.contains(&(*a, *name)),
            // A header delta refuses impl methods of std and lang traits
            // (`generics::delta_impl_trait`), so no `LangImpls` read moves.
            ReadKey::Layout(_) | ReadKey::DropCk(_) | ReadKey::Structure(_) | ReadKey::Answer(..) | ReadKey::TraitInScope(..) | ReadKey::StdImport(..) | ReadKey::Derives(_) | ReadKey::DeriveBits(..) | ReadKey::LangImpls(_) => false,
        }
    }
}

/// The method sets (Adt def, name) that hold one of the edited fns `fns`
/// as an inherent method candidate (collect's `inherent_methods`).
fn method_sets(p: &Program, fns: &[usize]) -> Vec<(crate::def::DefId, crate::intern::Sym)> {
    let fs: Vec<LocalDefId> = fns.iter().map(|&i| LocalDefId(i as u32)).collect();
    let mut out: Vec<_> = p.gx.inherent_methods.iter().filter(|(_, v)| v.iter().any(|f| fs.contains(f))).map(|(k, _)| *k).collect();
    out.sort_unstable_by_key(|(d, n)| (d.krate, d.hash.0, *n));
    out
}

/// True when fn `f`, a method of a trait impl that is a template (a
/// generic impl or method), has a bound over concrete types only that
/// collect's compare with the trait method (`generics::validate_method_sig`)
/// decides with the solver: it proves the goal with the impl records it has
/// at that point of collect (no scalar impls of the derive family yet), so
/// a finished program can answer otherwise (`where i64: Default`: cold
/// rejects, the delta would accept). `Copy`, `Clone` and `Debug` it
/// decides by structure, the same at any point.
fn order_bound(p: &Program, f: LocalDefId) -> bool {
    use crate::ty::{GenericArg, Predicate, TyIntern};
    if generics::delta_impl_trait(p, f).is_none() {
        return false;
    }
    let Some(Some(t)) = p.gx.fn_templates.get(f.0 as usize) else { return false };
    let lang = &p.gx.lang;
    t.generics.predicates.iter().any(|pr| {
        let Predicate::Trait(b) = pr else { return false };
        let by_structure = [lang.sized, lang.copy, lang.clone, lang.debug].contains(&b.value.def);
        let concrete = p.tys.args(b.value.args).iter().all(|g| match *g {
            GenericArg::Ty(t) => !p.tys.flags(t).contains_param() && !p.tys.flags(t).contains_infer(),
            _ => true,
        });
        b.count == 0 && concrete && !by_structure
    })
}

/// The traits of the trait impls that hold the edited fns `fns`.
fn impl_traits(p: &Program, fns: &[usize]) -> Vec<crate::def::DefId> {
    let mut out: Vec<crate::def::DefId> = fns.iter().filter_map(|&i| generics::delta_impl_trait(p, LocalDefId(i as u32))).collect();
    out.sort_unstable_by_key(|d| (d.krate, d.hash.0));
    out.dedup();
    out
}

/// True when item `item` is a trait impl.
fn is_trait_impl(p: &Program, item: u32) -> bool {
    matches!(&p.ast.items[item as usize].kind, crate::ast::ItemKind::Impl(imp) if imp.trait_ref.is_some())
}

/// Discovery bases one kept program holds: (AST fn index, its output as
/// the program has it), the `KeptProgram::bases` of driver/patch.rs.
pub(crate) type Bases = Vec<(usize, Option<crate::driver::gen_reuse::OutCopy>)>;

/// The discovery outputs `p` holds for the plain bodies a header delta of
/// `fns` would check again (their stored reads name a touched node), as
/// the stored entries no newer than `p` give them. A program kept after a
/// diagnostic takes them: the collect that answers the request may store
/// new entries for those bodies, and the next delta compares with these.
fn own_bases(p: &Program, front: &Front, caches: &Caches, fns: &[usize]) -> Bases {
    let reuse = &caches.gen_reuse;
    let readers = readers(p, reuse, fns);
    let now = crate::driver::gen_reuse::stamp_now();
    (0..front.fn_loc.len() as u32)
        .map(LocalDefId)
        .filter(|&g| p.gx.fn_class.get(g.0 as usize) == Some(&FnClass::Plain) && !p.check_skips(g))
        .filter(|&g| !reuse.newer(p, g, p.reuse_stamp, now) && !reuse.untouched_by(p, g, &readers))
        .map(|g| (g.0 as usize, reuse.out_copy(p.def_id(g))))
        .collect()
}

/// The defs whose stored entry may read a node `Touched` names for the
/// edited fns `fns`, through gen_reuse's reverse index (the same keys:
/// `GenReuse::readers`). The header delta tests entries against it in
/// place of scanning each entry's reads.
fn readers(p: &Program, reuse: &crate::driver::gen_reuse::GenReuse, fns: &[usize]) -> crate::intern::FxMap<crate::def::DefId, ()> {
    let defs: Vec<crate::def::DefId> = fns.iter().map(|&i| p.def_id(LocalDefId(i as u32))).collect();
    reuse.readers(&defs, &method_sets(p, fns), &impl_traits(p, fns))
}

/// The fns whose bodies reveal an opaque type and name one of the edited
/// fns `fns` (by identifier, as `body_scan` sees the rewritten body), not
/// counting the edited fns themselves: plain fns (`apply_headers` reveals
/// each again) and templates (their check runs again: in a check only,
/// and not a lang method). Refuses any other such fn, and an opaque owner
/// with no local body to scan.
#[allow(clippy::type_complexity)]
fn rpit_namers(
    p: &Program,
    opaque_idents: &[(crate::def::DefId, Vec<crate::intern::Sym>)],
    fns: &[usize],
    walk_instances: bool,
) -> Result<(Vec<LocalDefId>, Vec<LocalDefId>), &'static str> {
    let names: Vec<crate::intern::Sym> = fns.iter().map(|&i| p.fn_def(LocalDefId(i as u32)).name).collect();
    let (mut plain, mut tmpl) = (Vec::new(), Vec::new());
    for (owner, ids) in opaque_idents {
        let Some(g) = p.local_def(owner.hash) else {
            crate::check::derive_delta::log(&|| format!("sig rpit: opaque owner {owner:?} has no local fn\n"));
            return Err("sig rpit");
        };
        if fns.contains(&(g.0 as usize)) || !ids.iter().any(|s| names.contains(s)) {
            continue;
        }
        match p.gx.fn_class.get(g.0 as usize) {
            Some(FnClass::Plain) if p.gx.opaque_ret_of.contains_key(&g) => plain.push(g),
            Some(FnClass::Template) if !walk_instances && !p.is_lang_method(g) => tmpl.push(g),
            c => {
                crate::check::derive_delta::log(&|| {
                    let lang = p.is_lang_method(g);
                    let std = p.std_source_span(p.fn_def(g).name_span) || p.fn_shipped_crate(g).is_some();
                    format!("sig rpit: namer {} class {c:?} lang {lang} std {std}\n", p.display_name(p.fn_def(g).name))
                });
                return Err("sig rpit");
            }
        }
    }
    for v in [&mut plain, &mut tmpl] {
        v.sort_unstable_by_key(|g| g.0);
        v.dedup();
    }
    Ok((plain, tmpl))
}

/// `GenReuse::newer` for template `t` against the program's stamp (a log line).
fn reuse_newer(caches: &Caches, p: &Program, t: LocalDefId, start: u64) -> bool {
    caches.gen_reuse.newer(p, t, p.reuse_stamp, start)
}

/// Mono's body checks that read the edited signatures, as a collect with
/// generic reuse runs them (`materialize`, `template_check`): a stored
/// result that no longer holds checks again, and a plain body must give
/// the discovery output collect built the program with. `start` is the
/// store stamp when the delta began: entries after it are its own. A
/// template checked again takes its undo into `undos` (`recheck_undoable`).
#[allow(clippy::too_many_arguments)]
fn replay_bodies(
    p: &mut Program,
    front: &Front,
    caches: &mut Caches,
    fns: &[usize],
    namers: &[LocalDefId],
    walk_instances: bool,
    start: u64,
    bases: &Bases,
    undos: &mut Option<Undos>,
) -> Result<(), &'static str> {
    let names: Vec<crate::intern::Sym> = fns.iter().map(|&i| p.fn_def(LocalDefId(i as u32)).name).collect();
    let idents = &front.chunks[0].scan.body_idents;
    let fn_count = front.fn_loc.len();
    let mut touched = Touched::new(p, fns);
    let mut moves = Touched::new(p, fns);
    let mut touched = |p: &Program, k: &ReadKey| touched.hit(p, k);
    let readers = readers(p, &caches.gen_reuse, fns);
    let (record_reads, unwind) = (p.record_reads, p.unwind);
    (p.record_reads, p.unwind) = (true, false);
    // The new signatures stand from here: the deep Adt fingerprints the
    // checks below record follow the program (`FpMemo::follow_scoped`).
    let followed = p.fp_memo.follow_scoped();
    let out = (|| {
        let reuse = &mut caches.gen_reuse;
        let mut again = Vec::new();
        // Every template: the trait default bodies come after the AST fns
        // (`generics::append_trait_default_templates`).
        for t in (0..p.fns.len() as u32).map(LocalDefId) {
            if p.gx.fn_class.get(t.0 as usize) != Some(&FnClass::Template) || p.is_lang_method(t) || fns.contains(&(t.0 as usize)) {
                continue;
            }
            if p.check_skips(t) && !p.gx.opaques.contains_key(&generics::opaque_def(p.def_id(t))) {
                continue;
            }
            // Another program stored this check (the other mode's, or the
            // collect of a request this program sat out): this one may
            // hold another, so it checks again (in a check, below).
            if reuse.newer(p, t, p.reuse_stamp, start) {
                again.push(t);
                continue;
            }
            // Every entry no newer than the program held for it.
            if reuse.untouched_by(p, t, &readers) {
                continue;
            }
            // Only the keys this delta can move compare, as for the
            // discovery bodies. A template with closures has no stored plain
            // check: its last clean check's reads stand in
            // (`closure_template_holds`), and with none it stands while its
            // body does not name an edited fn.
            let holds = reuse.template_holds_moved(p, t, &mut |k| moves.hit(p, k));
            let holds = holds.or_else(|| reuse.closure_template_holds(p, t, p.reuse_stamp, Some(&mut |k| moves.hit(p, k))));
            let holds = holds.unwrap_or_else(|| {
                !match idents.get(t.0 as usize) {
                    Some(ids) if (t.0 as usize) < fn_count => ids.iter().any(|s| names.contains(s)),
                    _ => super::ast_hash::body_scan(&p.ast, p.fn_def(t), None).1.iter().any(|s| names.contains(s)),
                }
            });
            if !holds {
                again.push(t);
            }
        }
        // A template whose check no longer holds checks again, as collect
        // does, in a check only: a build's instances of it would keep
        // their old checks. A check's instances of it (with generic
        // associated consts) check again in `replay_discovery`.
        if !again.is_empty() && walk_instances {
            return Err("sig template");
        }
        // The edited templates' instances replay too (`refresh_instance`),
        // and the instances of the templates that reveal an opaque type
        // naming one (`rpit_namers`, checked again in `apply`).
        let mut rechecked = again.clone();
        rechecked.extend(fns.iter().map(|&i| LocalDefId(i as u32)).filter(|f| p.gx.fn_class[f.0 as usize] == FnClass::Template));
        rechecked.extend_from_slice(namers);
        let mut times: Vec<(f64, LocalDefId)> = Vec::new();
        for t in again {
            let (lo, hi) = p.fn_def(t).exprs;
            let t0 = std::time::Instant::now();
            recheck_undoable(p, t, &[(lo, hi)], caches, undos)?;
            times.push((t0.elapsed().as_secs_f64() * 1e3, t));
        }
        crate::check::derive_delta::log(&|| {
            let mut slow = times.clone();
            slow.sort_by(|x, y| y.0.total_cmp(&x.0));
            let why = |t: LocalDefId| match (reuse_newer(caches, p, t, start), caches.gen_reuse.has_template_entry(p, t)) {
                (true, _) => "newer",
                (false, true) => "entry moved",
                (false, false) => "no entry",
            };
            let list = slow.iter().take(8).map(|&(ms, t)| format!("{} {ms:.1} ({})", p.display_name(p.fn_def(t).name), why(t))).collect::<Vec<_>>();
            format!("replay templates: {} again; {}\n", times.len(), list.join(", "))
        });
        super::laps::mark("collect/delta replay templates");
        replay_discovery(p, caches, fn_count, walk_instances, start, bases, &mut touched, &|_| false, Some(&readers), &rechecked, true)
    })();
    if followed {
        p.fp_memo.follow_end();
    }
    (p.record_reads, p.unwind) = (record_reads, unwind);
    out
}

/// Mono's discovery checks of plain bodies after a kept-program delta (the
/// header delta above, the derive delta in driver/kept_derive.rs), as a
/// collect with generic reuse runs them. A body stands when the program
/// holds its entry (none stored after `Program::reuse_stamp`) and the entry
/// reads no node `touched` names (or, with `readers`, is not in that set:
/// the header delta's reverse index of the same nodes), or when each read
/// of its entry that `touched` names keeps its fingerprint (in `lust check`
/// also the reads of a pending check's note), or when the caller proved it
/// (`stands`: the derive delta's keyed walk).
/// An entry stored after the program's stamp (and before `start`, when the
/// delta began) checks against the output the program keeps for it
/// (`bases`). Every other body checks again and must give the output
/// collect built the program with (`GenReuse::out_copy`). A check still
/// pending gives it too, as collect's last check of that body was; it is
/// kept, not stored, as collect keeps it. Outputs are stored only when
/// every body passes, so an Err leaves the stored entries as they were
/// (the header delta then undoes its writes, `fail`).
#[allow(clippy::too_many_arguments)]
pub(super) fn replay_discovery(
    p: &mut Program,
    caches: &mut Caches,
    fn_count: usize,
    walk_instances: bool,
    start: u64,
    bases: &Bases,
    touched: &mut dyn FnMut(&Program, &ReadKey) -> bool,
    stands: &dyn Fn(LocalDefId) -> bool,
    readers: Option<&crate::intern::FxMap<crate::def::DefId, ()>>,
    rechecked: &[LocalDefId],
    bodies: bool,
) -> Result<(), &'static str> {
    let (record_reads, unwind) = (p.record_reads, p.unwind);
    (p.record_reads, p.unwind) = (true, false);
    // (body, its reads when it is stored, its output).
    // (body, its reads when it is stored or a pending check of `lust check`
    // (`GenReuse::note_unstored`), whether it is pending, its output).
    let mut outs: Vec<(LocalDefId, Option<crate::driver::graph::Reads>, bool, crate::driver::gen_reuse::Discovered)> = Vec::new();
    // As in `replay_bodies`: the delta's edits are in (a derive delta's
    // toggles too), so the deep fingerprints follow the program.
    let followed = p.fp_memo.follow_scoped();
    crate::driver::graph::FpMemo::sync(p);
    let out = (|| {
        let reuse = &mut caches.gen_reuse;
        // The bodies that check again, each with the output it must give,
        // in fn order. A body with no output to compare ends the list: its
        // Err comes after the answers of the bodies before it, as in a
        // walk one body at a time.
        let mut work: Vec<(LocalDefId, crate::driver::gen_reuse::OutCopy, bool)> = Vec::new();
        let mut stop: Option<&'static str> = None;
        // `LUST_DERIVE_LOG` counts: bodies that stand by the caller, by
        // their reads, by their stored output.
        let mut counts = [0usize; 3];
        let t0 = std::time::Instant::now();
        for g in (0..fn_count as u32).map(LocalDefId).filter(|_| bodies) {
            if p.gx.fn_class.get(g.0 as usize) != Some(&FnClass::Plain) || p.check_skips(g) {
                continue;
            }
            let own = bases.iter().find(|b| b.0 == g.0 as usize).map(|b| b.1.as_ref());
            let newer = reuse.newer(p, g, p.reuse_stamp, start);
            let base = if newer {
                // Another collect replaced the entry: the output the program
                // has is its base, or the one kept for it (`GenReuse::hold`).
                match own.map(|b| b.cloned()).or_else(|| reuse.held_copy(p, g, p.reuse_stamp).map(Some)) {
                    Some(b) => b,
                    None => {
                        stop = Some("sig mode");
                        break;
                    }
                }
            } else if stands(g) {
                counts[0] += 1;
                continue;
            } else if readers.map_or_else(|| reuse.untouched(p, g, &mut |k| touched(p, k)), |r| reuse.untouched_by(p, g, r)) {
                counts[1] += 1;
                continue;
            } else if reuse.discovery_holds_moved(p, g, &mut |k| touched(p, k)) {
                counts[2] += 1;
                continue;
            } else if !walk_instances && reuse.pending_holds_moved(p, g, &mut |k| touched(p, k)) {
                counts[1] += 1;
                continue;
            } else {
                reuse.out_copy(p.def_id(g))
            };
            let Some(base) = base else {
                crate::check::derive_delta::log(&|| format!("discovery {}: no output kept\n", p.display_name(p.fn_def(g).name)));
                stop = Some("discovery");
                break;
            };
            work.push((g, base, newer));
        }
        // A check of a program with generic associated consts made
        // instances (`patch::walks_requests`), which collect walks again
        // from the new bodies: an instance of a template this delta checked
        // again (`rechecked`, through closure parents) replays the new
        // check, and must give no error and its stored output, requests
        // included (`OutCopy::same`), so collect would make the same
        // instances. An instance of a template that holds replays the same
        // resolutions: the delta reaches it only through them (an edited
        // trait impl method through the trait's impl buckets, whose reader
        // templates check again).
        let mut instances: crate::intern::FxMap<LocalDefId, ()> = crate::intern::FxMap::default();
        if stop.is_none() && !walk_instances && !rechecked.is_empty() && super::patch::walks_requests(p) {
            let owners: crate::intern::FxMap<LocalDefId, ()> = rechecked.iter().map(|&t| (t, ())).collect();
            // Up from an instance to its template, from a closure body fn to
            // the fn it is checked in, until a rechecked template (or none).
            let owned_by_rechecked = |p: &Program, g: LocalDefId| {
                let mut f = Some(g);
                for _ in 0..64 {
                    let Some(h) = f else { return false };
                    if owners.contains_key(&h) {
                        return true;
                    }
                    f = match p.gx.instances.get(&h) {
                        Some(info) => info.template_local,
                        None => p.gx.closure_fns.get(&h).and_then(|d| p.gx.closures.get(d)).map(|e| e.parent),
                    };
                }
                true
            };
            for g in (fn_count as u32..p.fns.len() as u32).map(LocalDefId) {
                // A closure body fn checks with the body it is in.
                if p.gx.fn_class.get(g.0 as usize) != Some(&FnClass::Instance) || p.check_skips(g) {
                    continue;
                }
                if !owned_by_rechecked(p, g) {
                    continue;
                }
                // Another collect replaced the entry (a refused request's):
                // the output the program has is the one kept for it, as for
                // a plain body (`GenReuse::hold`).
                let newer = reuse.newer(p, g, p.reuse_stamp, start);
                let base = if newer { reuse.held_copy(p, g, p.reuse_stamp) } else { reuse.out_copy(p.def_id(g)) };
                let Some(base) = base else {
                    crate::check::derive_delta::log(&|| format!("discovery instance {}: no output kept (newer {newer})\n", p.display_name(p.fn_def(g).name)));
                    stop = Some(if newer { "sig mode" } else { "discovery" });
                    break;
                };
                instances.insert(g, ());
                work.push((g, base, newer));
            }
        }
        // The checks read the program only: they run in parallel, as the
        // compile tail's body checks do.
        let walk_ms = t0.elapsed().as_secs_f64() * 1e3;
        if std::env::var_os("LUST_DERIVE_LOG").is_some() {
            let mut why = Vec::new();
            for &(g, ..) in work.iter().take(8) {
                why.push(format!("{}: {}", p.display_name(p.fn_def(g).name), reuse.pending_why(p, g, &mut |k| touched(p, k))));
            }
            crate::check::derive_delta::log(&|| format!("discovery replay pending: {}\n", why.join("; ")));
        }
        crate::driver::graph::FpMemo::sync(p);
        let prog: &Program = p;
        let checked = crate::driver::compile::par_map(&work, |&(g, _, _)| {
            let t = std::time::Instant::now();
            let (mut checked, discarded) = crate::check::check_fn(prog, g);
            let pending = checked.closure_map_pending;
            let out = generics::discovery_out(prog, &checked, &discarded);
            let keep = discarded.is_empty() && (!pending || (!walk_instances && !checked.closure_kind_pending));
            let reads = keep.then(|| std::mem::take(&mut checked.reads));
            // Why the output is not stored (`LUST_DERIVE_LOG`).
            let why = discarded.first().map(|d| format!("{} {}", d.code, d.message)).unwrap_or_default();
            (out, reads, pending, t.elapsed().as_secs_f64() * 1e3, why)
        });
        crate::check::derive_delta::log(&|| {
            let [a, b, c] = counts;
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            let mut slow: Vec<(f64, String)> = work
                .iter()
                .zip(&checked)
                .map(|(&(g, _, newer), x)| {
                    let unstored = x.1.is_none() || x.2;
                    let tags = if newer { " (newer)" } else { "" }.to_string() + if unstored { " (unstored" } else { "" };
                    let tags = if unstored { format!("{tags}: {}{})", if x.2 { "pending " } else { "" }, x.4) } else { tags };
                    (x.3, format!("{}{tags}", p.display_name(p.fn_def(g).name)))
                })
                .collect();
            slow.sort_by(|x, y| y.0.total_cmp(&x.0));
            let slow = slow.iter().take(6).map(|(t, n)| format!("{n} {t:.1}")).collect::<Vec<_>>().join(", ");
            let n = work.len();
            format!("discovery replay: {a} stand, {b} untouched, {c} hold, {n} check; walk {walk_ms:.1} ms, all {ms:.1} ms; {slow}\n")
        });
        for ((g, base, newer), (out, reads, pending, ..)) in work.into_iter().zip(checked) {
            // An output that differs only in types whose instances were all
            // made before the body's turn makes what collect makes (a caller
            // with an error, or one that names a new type); bodies in fn order.
            // An instance's error, or a closure kind still pending, is
            // collect's to answer (a pending closure map stands as for a
            // plain body: collect's last check of it was pending too).
            if instances.contains_key(&g) && reads.is_none() {
                crate::check::derive_delta::log(&|| format!("discovery instance {}: error or pending kind\n", p.display_name(p.fn_def(g).name)));
                return Err("discovery");
            }
            let same = base.same(p, &out, walk_instances);
            let made = !same
                && !super::patch::walks_requests(p)
                && base.makers(&out).is_some_and(|(old, new)| super::patch::made_before(p, g, &old, &new));
            if !same && !made {
                crate::check::derive_delta::log(&|| {
                    format!("discovery {}: output differs (pending {pending}, newer {newer}): {}\n", p.display_name(p.fn_def(g).name), base.differs(p, &out))
                });
                return Err("discovery");
            }
            reuse.discovery_checked += 1;
            // Every body type enters the global store, as discovery does.
            for t in out.0.iter().map(|t| &**t).chain(out.3.iter().flat_map(|(a, b)| [a, b])) {
                generics::build_owned_ty(p, t);
            }
            outs.push((g, reads, pending, out));
        }
        if let Some(why) = stop {
            return Err(why);
        }
        for (g, reads, pending, (adt_tys, reqs, closures, vts)) in &outs {
            match reads {
                Some(r) if !pending => reuse.store_discovered(p, *g, r, adt_tys, reqs, closures, vts),
                _ => reuse.note_unstored(p, *g, adt_tys, reqs, closures, vts, reads.as_ref()),
            }
        }
        Ok(())
    })();
    if followed {
        p.fp_memo.follow_end();
    }
    (p.record_reads, p.unwind) = (record_reads, unwind);
    out
}

/// The header delta's answer for every user fn of `p` (a census for
/// `LUST_SIGDELTA_SELFTEST=v`): the number of fns per refusal reason, and
/// "ok" for those it takes. Const reach is not counted (it needs the
/// patch base); std and lang fns are left out.
pub(crate) fn census(p: &Program) -> Vec<(&'static str, usize)> {
    let opaque_idents = opaque_body_idents(p);
    let mut out: Vec<(&'static str, usize)> = Vec::new();
    for i in 0..p.fns.len() {
        let f = LocalDefId(i as u32);
        if !matches!(p.gx.fn_class[i], FnClass::Plain | FnClass::Template)
            || p.gx.closure_fns.contains_key(&f)
            || p.is_lang_method(f)
            || p.fn_shipped_crate(f).is_some()
            || p.std_source_span(p.fn_def(f).name_span)
        {
            continue;
        }
        let why = refusal(p, f, false).and_then(|()| rpit_namers(p, &opaque_idents, &[i], false).map(drop)).err().unwrap_or("ok");
        match out.iter_mut().find(|e| e.0 == why) {
            Some(e) => e.1 += 1,
            None => out.push((why, 1)),
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1));
    out
}
