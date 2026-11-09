// The derive delta's driver half (sigedit PLAN P3.2, R13): derive list
// edits applied to the daemon's kept program in place of a full collect.
// The item patch (driver/patch.rs `find_derive_edit`) has already put the
// new derive lists in the Ast and moved the program's spans; `reuse_kept`
// then calls `apply_derives` with the edited struct and enum items.
//
// For each item, check/derive_delta.rs `apply` sets the new list, toggles
// the synthetic derive records, and runs the collect checks a derive list
// feeds again, over the items that reach the Adt. Then:
//
// - the keyed dirty walk (graph.rs `keyed_dirty_slots`): only the read keys the
//   delta can move (`Program::derive_key_moves`) are compared, for the
//   slots that read them; the compile tail rechecks the dirty slots and
//   reuses the rest without walking their reads;
// - mono's checks that read derive lists replay, as a collect with generic
//   reuse runs them: a template whose stored check (or, with closures, the
//   reads of its last check) no longer holds checks again, and so does one
//   with neither whose body names the reach; plain bodies go through the header delta's
//   discovery replay (`kept_delta::replay_discovery`): a body whose entry
//   reads no key the delta can move stands, the others must give the same
//   discovery output. Entries stored after the program's `reuse_stamp`
//   (the other mode's program) check again.
//
// Refused (the caller collects in full): a build (its instances and glue
// would keep the old lists), an item the kept program has no Adt for, and
// whatever `derive_delta::apply` refuses. Any diagnostic falls back too:
// the full path gives the answer. A refusal before the replays undoes the
// delta (`derive_delta::Undo`), and `reuse_kept` keeps the program for the
// next request (usually the fix or the revert), as for a refused header
// delta. The keyed shortcuts apply only when the request's edits are all
// derive list edits (`Caches::derive_only`).
use super::session::{Caches, Front};
use crate::ast::ItemKind;
use crate::check::collect::Program;
use crate::check::generics::{self, FnClass};
use crate::def::LocalDefId;

/// Applies the derive list edits of the struct and enum items `items`
/// (`ast.items` indexes) to the kept program `p`. Err names the first
/// condition that failed (the `kept=` of `lust-visits:`), and says whether
/// `p` is as it was: a refusal before the replays undoes the derive
/// delta's writes (`derive_delta::Undo`), so the program can stay for the
/// next request; after a replay wrote template or discovery state, `p` is
/// half edited and must be dropped. Ok holds the reach of the moved
/// derive flags (None: no flag moved), for the const body check's keyed
/// walk (`const_body::check_dirty`).
pub(super) fn apply_derives(
    p: &mut Program,
    front: &Front,
    caches: &mut Caches,
    items: &[usize],
    walk_instances: bool,
) -> Result<Option<crate::check::derive_delta::Reach>, (&'static str, bool)> {
    if walk_instances {
        return Err(("derive build", false));
    }
    // The node answers of the last collect read the old lists.
    caches.gen_reuse.forget_answers();
    // Entries stored from here on are this delta's own (`replay_discovery`).
    let start = crate::driver::gen_reuse::stamp_now();
    let mut undo = crate::check::derive_delta::Undo::default();
    let mut moved: Option<crate::check::derive_delta::Reach> = None;
    for &item in items {
        match apply_item(p, item, &mut undo) {
            Ok(Some(reach)) => {
                moved = Some(match moved.take() {
                    Some(m) => m.union(reach),
                    None => reach,
                })
            }
            Ok(None) => {}
            Err(why) => {
                undo.restore(p);
                return Err((why, true));
            }
        }
    }
    crate::driver::laps::mark("kept/derive delta");
    // No flag moved (say the revert of a refused edit, whose delta was
    // undone): no read key moved either.
    let Some(reach) = moved else {
        caches.derive_dirty = Some(vec![false; caches.slot_reads.len()]);
        return Ok(None);
    };
    // The keyed dirty walk: only the read keys the delta can move
    // (graph.rs `Program::derive_key_moves`) are compared, for the slots
    // that read them. The compile tail rechecks the dirty slots.
    // Deep Adt fingerprints are kept across the moved keys (the derive
    // lists are final here; the memo goes before any other write).
    p.fp_memo.on(p.adts.len());
    // The moved fn names and signature answers, made once for every key
    // test of this delta (and the const body check after it, patch.rs).
    let tables = {
        let mut named = crate::driver::graph::DeriveMoves::new(&reach);
        p.moved_fn_names(&mut named);
        named.tables().ok_or(("derive names", false))?
    };
    caches.derive_tables = Some(tables.clone());
    let (is_dirty, has_checked) = {
        let prog: &Program = p;
        // `named` gives the tables of the parallel classifier; `moves` the
        // walk for the keys it leaves.
        let named = crate::driver::graph::DeriveMoves::with_tables(&reach, tables.clone());
        let quick = named.quick();
        let quick = |k: &crate::driver::graph::ReadKey| quick.as_ref().and_then(|q| prog.derive_key_moves_quick(q, k));
        let mut moves = crate::driver::graph::DeriveMoves::with_tables(&reach, tables.clone());
        crate::driver::laps::mark("kept/derive names");
        let def_slot = |d: crate::def::DefId| prog.local_def(d.hash).and_then(|l| front.fn_slot.get(l.0 as usize)).map(|&s| s as usize);
        let answers = &mut caches.gen_reuse.memo;
        // `LUST_DERIVE_KEYLOG` (with `LUST_DERIVE_LOG`): keys asked and
        // moved, and the time, per kind.
        let logging = std::env::var_os("LUST_DERIVE_KEYLOG").is_some();
        let mut kinds: Vec<(&str, usize, usize, f64)> = Vec::new();
        let out = crate::driver::graph::keyed_dirty_slots(
            prog,
            &caches.slot_reads,
            &caches.read_keys,
            &quick,
            &mut |k| {
                if !logging {
                    return prog.derive_key_moves(&mut moves, k);
                }
                let t = std::time::Instant::now();
                let b = prog.derive_key_moves(&mut moves, k);
                use crate::driver::graph::ReadKey as K;
                let name = match k {
                    K::Binding(_) => "Binding",
                    K::MethodSet(_) => "MethodSet",
                    K::ImplSet(_) => "ImplSet",
                    K::Sig(_) => "Sig",
                    K::Layout(_) => "Layout",
                    K::DropCk(_) => "DropCk",
                    K::Checked(_) => "Checked",
                    K::TraitInScope(..) => "TraitInScope",
                    K::StdImport(..) => "StdImport",
                    K::Derives(_) => "Derives",
                    K::DeriveBits(..) => "DeriveBits",
                    K::LangImpls(_) => "LangImpls",
                    K::AssocVis(..) => "AssocVis",
                    K::Structure(_) => "Structure",
                    K::Answer(..) => "Answer",
                };
                let ms = t.elapsed().as_secs_f64() * 1e3;
                match kinds.iter_mut().find(|x| x.0 == name) {
                    Some(x) => (x.1, x.2, x.3) = (x.1 + 1, x.2 + usize::from(b), x.3 + ms),
                    None => kinds.push((name, 1, usize::from(b), ms)),
                }
                b
            },
            &def_slot,
            answers,
        );
        crate::check::derive_delta::log(&|| {
            if !logging {
                return String::new();
            }
            let parts: Vec<String> = kinds.iter().map(|(n, a, m, ms)| format!("{n} {a}/{m} {ms:.1} ms")).collect();
            format!("keyed classify: {}\n", parts.join(", "))
        });
        out
    };
    p.fp_memo.off();
    // The shortcuts below hold only when the derive delta is the request's
    // only change (no header or body edit moved other keys).
    let keyed = caches.derive_only;
    // Without user generics collect runs no template check or discovery
    // (the lang enum methods' generics alone need neither in a check).
    if p.gx.user_generics {
        let rechecked = match replay_templates(p, front, caches, &reach, &tables, keyed, start) {
            Ok(r) => r,
            Err(why) => {
            // A parallel template check failed before the replays wrote
            // (`recheck_par`): the delta undoes as a refusal above does.
                if why == TEMPLATE_DIAG {
                    caches.gen_reuse.forget_answers();
                    undo.restore(p);
                    return Err((why, true));
                }
                return Err((why, false));
            }
        };
        crate::driver::laps::mark("kept/derive templates");
        // A body's discovery reads are a subset of its check's, taken on
        // the same program: a body whose stored check reads are all clean
        // keeps its discovery output too (the keyed walk's dirty slots, no
        // per-read scan). Not one with a `Checked` read (a collect checks
        // it again each time), and its stored output must be of the body's
        // current source. Else a body whose entry reads no key the delta
        // can move stands (`GenReuse::untouched`, as in the header delta);
        // with other edits in the request every key counts as moved.
        let clean: Vec<bool> = (0..front.fn_loc.len() as u32)
            .map(LocalDefId)
            .map(|g| {
                let s = front.fn_slot.get(g.0 as usize).map_or(usize::MAX, |&s| s as usize);
                keyed
                    && caches.slot_reads.get(s).is_some_and(Option::is_some)
                    && !is_dirty[s]
                    && !has_checked[s]
                    && caches.gen_reuse.discovery_src_same(p, g)
            })
            .collect();
        crate::check::derive_delta::log(&|| {
            // Why each plain body that is not clean is not (counts, a few names).
            let mut why: Vec<(&str, usize, Vec<String>)> = Vec::new();
            for g in (0..front.fn_loc.len() as u32).map(LocalDefId) {
                if clean[g.0 as usize] || p.gx.fn_class.get(g.0 as usize) != Some(&FnClass::Plain) || p.check_skips(g) {
                    continue;
                }
                let s = front.fn_slot.get(g.0 as usize).map_or(usize::MAX, |&s| s as usize);
                let w = if !keyed {
                    "not keyed"
                } else if !caches.slot_reads.get(s).is_some_and(Option::is_some) {
                    "no slot reads"
                } else if is_dirty[s] {
                    "dirty"
                } else if has_checked[s] {
                    "checked read"
                } else {
                    "discovery src"
                };
                match why.iter_mut().find(|x| x.0 == w) {
                    Some(x) => {
                        x.1 += 1;
                        if x.2.len() < 6 {
                            x.2.push(p.display_name(p.fn_def(g).name).to_string());
                        }
                    }
                    None => why.push((w, 1, vec![p.display_name(p.fn_def(g).name).to_string()])),
                }
            }
            let parts: Vec<String> = why.iter().map(|(w, n, names)| format!("{w} {n} ({})", names.join(" "))).collect();
            format!("derive clean: not clean: {}\n", parts.join("; "))
        });
        crate::driver::laps::mark("kept/derive clean");
        let mut moves = crate::driver::graph::DeriveMoves::with_tables(&reach, tables.clone());
        let mut touched = |p: &Program, k: &crate::driver::graph::ReadKey| !keyed || p.derive_key_moves(&mut moves, k);
        let stands = |g: LocalDefId| clean.get(g.0 as usize).copied().unwrap_or(false);
        super::kept_delta::replay_discovery(p, caches, front.fn_loc.len(), walk_instances, start, &Vec::new(), &mut touched, &stands, None, &rechecked, true)
            .map_err(|why| (why, false))?;
        crate::driver::laps::mark("kept/derive discovery");
    }
    caches.derive_dirty = Some(is_dirty);
    Ok(Some(reach))
}

/// The derive delta of one struct or enum item (`ast.items` index `item`):
/// its new list from the Ast, applied by `derive_delta::apply`, which logs
/// its writes in `undo`. None when no flag moved.
fn apply_item(p: &mut Program, item: usize, undo: &mut crate::check::derive_delta::Undo) -> Result<Option<crate::check::derive_delta::Reach>, &'static str> {
    let (name, names, is_enum) = match &p.ast.items.get(item).ok_or("derive adt")?.kind {
        ItemKind::Struct(s) => (s.name, s.derives.iter().map(|d| d.0).collect::<Vec<_>>(), false),
        ItemKind::Enum(e) => (e.name, e.derives.iter().map(|d| d.0).collect::<Vec<_>>(), true),
        _ => return Err("derive adt"),
    };
    let a = *p.type_names.get(&name).ok_or("derive adt")?;
    if p.adt(a).name != name || p.adt(a).is_enum != is_enum {
        return Err("derive adt");
    }
    let new = crate::check::derive_delta::derives_from_names(p, p.adt(a).derives_unlogged(), is_enum, &names)?;
    crate::check::derive_delta::apply(p, a, new, undo)
}

/// The source names a body must mention to read a derive list of the
/// reach: the reach Adts and their variants, the type aliases of a reach
/// type, and the fns, consts, and statics whose types name one (a value of
/// a reach type comes from one of them, or from a type the body writes).
fn reach_names(p: &Program, reach: &crate::check::derive_delta::Reach, fn_count: usize) -> crate::intern::FxMap<crate::intern::Sym, ()> {
    let mut out = crate::intern::FxMap::default();
    let mut add = |s: crate::intern::Sym| {
        let text = crate::intern::source_name(p.interner.get(s)).to_string();
        out.insert(p.interner.intern(&text), ());
    };
    for (i, adt) in p.adts.iter().enumerate() {
        if p.gx.adt_instance_of.get(i).copied().flatten().is_none() && reach.row(p, i) {
            add(adt.name);
            adt.variants.iter().for_each(|v| add(v.name));
        }
    }
    let mut moves = crate::driver::graph::DeriveMoves::new(reach);
    // The AST fns, trait default bodies and trait method stubs (instances
    // and closures share their names or have none).
    for i in 0..p.fns.len() {
        let f = LocalDefId(i as u32);
        if i >= fn_count && !matches!(p.gx.fn_class.get(i), Some(FnClass::Template | FnClass::Stub)) {
            continue;
        }
        if p.sig_moves(&mut moves, p.def_id(f)) {
            add(p.fn_def(f).name);
        }
    }
    // Type aliases of a reach type (`type T2 = S;` names S as T2).
    p.aliases.iter().filter(|&(_, &t)| reach.ty(&p.tys, t)).for_each(|(&n, _)| add(n));
    p.gx.generic_aliases.iter().filter(|(_, a)| reach.ty(&p.tys, a.erased)).for_each(|(&n, _)| add(n));
    p.consts.iter().filter(|c| reach.ty(&p.tys, c.ty)).for_each(|c| add(c.name));
    p.statics.iter().filter(|s| reach.ty(&p.tys, s.ty)).for_each(|s| add(s.name));
    out
}

/// The refusal of a template body check that failed before any write
/// (`recheck_par`); `p` is then as the derive delta left it.
const TEMPLATE_DIAG: &str = "derive template diag";

/// `kept_delta::recheck_template` for templates `ts`, which have no
/// closures and define no opaque type: their body checks only read the
/// program, so they run in parallel (`generics::template_body`), and the
/// writes follow in fn order. Their checks do not read the picks or
/// closure call kinds the others write (a template check records picks,
/// it does not replay them).
fn recheck_par(p: &mut Program, ts: &[LocalDefId], caches: &mut Caches) -> Result<(), &'static str> {
    if ts.is_empty() {
        return Ok(());
    }
    // Collect's template check runs before mono.
    let (counts, mono_done) = (p.gx.mono_counts, std::mem::replace(&mut p.gx.mono_done, false));
    let prog: &Program = p;
    let bodies = crate::driver::compile::par_map(ts, |&t| generics::template_body(prog, t, &[]));
    // A body error before any write: the caller undoes the derive delta
    // and keeps the program (a rejected derive edit, then its revert).
    if bodies.iter().any(generics::TemplateBody::failed) {
        (p.gx.mono_counts, p.gx.mono_done) = (counts, mono_done);
        return Err(TEMPLATE_DIAG);
    }
    let mut ranges: Vec<(u32, u32)> = ts.iter().map(|&t| p.fn_def(t).exprs).collect();
    ranges.sort_unstable();
    let inside = |x: u32| {
        let k = ranges.partition_point(|r| r.0 <= x);
        k > 0 && x < ranges[k - 1].1
    };
    p.gx.trait_picks.retain(|x, _| !inside(x.0));
    let set: crate::intern::FxMap<LocalDefId, ()> = ts.iter().map(|&t| (t, ())).collect();
    p.gx.template_closures.retain(|k, _| !set.contains_key(&k.0));
    p.gx.template_calls.retain(|k, _| !set.contains_key(&k.0));
    let mut out = Ok(());
    for (&t, body) in ts.iter().zip(bodies) {
        if !generics::finish_template(p, t, &mut caches.gen_reuse, body, &[], &Vec::new()).is_empty() {
            out = Err("sig diag");
            break;
        }
    }
    (p.gx.mono_counts, p.gx.mono_done) = (counts, mono_done);
    out
}

/// Mono's template checks after a derive delta (the templates that
/// checked again, for `kept_delta::replay_discovery`): a template (trait default
/// bodies included) whose stored check no longer holds checks again as
/// collect's template check does,
/// and must pass with the same hidden type (`kept_delta::recheck_template`).
/// A template with closures has no stored check; the reads of its last
/// check answer for it (`GenReuse::closure_template_holds`). One with
/// neither checks again when its body mentions a name of `reach_names`, or
/// its own signature moves. Templates with no closures and no opaque type
/// check in parallel (`recheck_par`).
/// With `keyed` (a derive-only request) a stored check compares only the
/// read keys the delta can move. A check stored after the program's
/// `reuse_stamp` (by the other mode's program) checks again: this program
/// may hold another.
fn replay_templates(
    p: &mut Program,
    front: &Front,
    caches: &mut Caches,
    reach: &crate::check::derive_delta::Reach,
    tables: &crate::driver::graph::DeriveTables,
    keyed: bool,
    start: u64,
) -> Result<Vec<LocalDefId>, &'static str> {
    let fn_count = front.fn_loc.len();
    // Only a template with no stored reads needs the names (made once).
    let mut names: Option<crate::intern::FxMap<crate::intern::Sym, ()>> = None;
    let mut moves = crate::driver::graph::DeriveMoves::with_tables(reach, tables.clone());
    let idents = &front.chunks[0].scan.body_idents;
    // The moved keys the stored checks read, fingerprinted once and in
    // parallel into the request's answers (`GenReuse::memo`), which
    // `template_holds_moved` reads below.
    if keyed {
        p.fp_memo.on(p.adts.len());
        let prog: &Program = p;
        let reuse = &caches.gen_reuse;
        let mut seen: crate::intern::FxMap<crate::driver::graph::ReadKey, ()> = crate::intern::FxMap::default();
        let asked: Vec<crate::driver::graph::ReadKey> =
            reuse.template_read_keys().filter(|k| !reuse.memo.contains_key(*k) && seen.insert(**k, ()).is_none()).copied().collect();
        let moved = prog.derive_keys_moved(&mut moves, &asked);
        let keys: Vec<crate::driver::graph::ReadKey> = asked.into_iter().zip(moved).filter(|(_, m)| *m).map(|(k, _)| k).collect();
        let fresh = crate::driver::compile::par_map(&keys, |k| prog.read_fingerprint(k));
        caches.gen_reuse.memo.extend(keys.into_iter().zip(fresh));
        p.fp_memo.off();
    }
    crate::driver::laps::mark("kept/derive templates keys");
    let (record_reads, unwind) = (p.record_reads, p.unwind);
    (p.record_reads, p.unwind) = (true, false);
    let out = (|| {
        let mut again = Vec::new();
        // Every template: the trait default bodies come after the AST fns
        // (`generics::append_trait_default_templates`).
        for t in (0..p.fns.len() as u32).map(LocalDefId) {
            if p.gx.fn_class.get(t.0 as usize) != Some(&FnClass::Template) || p.is_lang_method(t) {
                continue;
            }
            if p.check_skips(t) && !p.gx.opaques.contains_key(&generics::opaque_def(p.def_id(t))) {
                continue;
            }
            let holds = if caches.gen_reuse.newer(p, t, p.reuse_stamp, start) {
                Some(false)
            } else if keyed {
                let prog: &Program = p;
                caches.gen_reuse.template_holds_moved(prog, t, &mut |k| prog.derive_key_moves(&mut moves, k))
            } else {
                caches.gen_reuse.template_holds(p, t)
            };
            // A template with closures has no stored check: the reads of
            // its last check answer instead (`closure_template_holds`).
            let holds = holds.or_else(|| {
                let prog: &Program = p;
                let moves: Option<&mut dyn FnMut(&crate::driver::graph::ReadKey) -> bool> =
                    if keyed { Some(&mut |k| prog.derive_key_moves(&mut moves, k)) } else { None };
                caches.gen_reuse.closure_template_holds(prog, t, prog.reuse_stamp, moves)
            });
            match holds {
                Some(true) => continue,
                Some(false) => {}
                None => {
                    let names = names.get_or_insert_with(|| reach_names(p, reach, fn_count));
                    let named = match idents.get(t.0 as usize) {
                        Some(ids) if (t.0 as usize) < fn_count => ids.iter().any(|s| names.contains_key(s)),
                        _ => super::ast_hash::body_scan(&p.ast, p.fn_def(t), None).1.iter().any(|s| names.contains_key(s)),
                    };
                    if !named && !p.sig_moves(&mut moves, p.def_id(t)) {
                        continue;
                    }
                }
            }
            again.push(t);
        }
        crate::driver::laps::mark("kept/derive templates walk");
        // Templates with no closures and no opaque type of their own check
        // in parallel (`recheck_par`); the others, and a check the other
        // mode stored, one at a time in fn order.
        let (par, again): (Vec<LocalDefId>, Vec<LocalDefId>) = again.into_iter().partition(|&t| {
            !caches.gen_reuse.newer(p, t, p.reuse_stamp, start)
                && !generics::fn_has_closures(p, t)
                && !p.gx.opaques.contains_key(&generics::opaque_def(p.def_id(t)))
                && !p.gx.closures.values().any(|e| e.parent == t)
        });
        recheck_par(p, &par, caches)?;
        crate::driver::laps::mark("kept/derive templates par");
        for &t in &again {
            let body = p.fn_def(t).exprs;
            super::kept_delta::recheck_template(p, t, &[body], caches)?;
        }
        Ok(par.into_iter().chain(again).collect())
    })();
    (p.record_reads, p.unwind) = (record_reads, unwind);
    out
}
