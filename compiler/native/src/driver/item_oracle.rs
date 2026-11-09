// The patch oracle (`LUST_CHECK_ITEMPATCH=1`; sigedit PLAN, P1.1). After
// each request on a patch path (`path=patch`, the body patch, and
// `path=item`, the item patch), a cold resolve and collect of the same
// text, in the same interner, must agree with the warm state:
// - resolve: the rewrite epoch, the item scans, and every fn's body hash
//   and line;
// - collect: the same errors, or a program with the same `Digest` (Sig
//   and Layout per def, ImplSet buckets, fn classes, hidden types).
// On every path (the single-file chunk path and full resolves too):
// - the dirty set: an AST fn the warm request does not check again must
//   have stored reads that are clean against the cold program, or, with
//   no stored reads, the cold digest must equal the last request's;
// - the response: every daemon check request compiles again in a fresh
//   session (`compare_response`), which must answer the same bytes and
//   reach the same fn lines, collect errors and `Digest`.
// Any difference appends one line to the file `LUST_ITEMPATCH_LOG` names,
// prints it, and panics with a `Failure` payload. The daemon answers that
// request as an internal error (code 3) and never retries it in a fresh
// session, so the failure shows in the answer too. Passing checks log
// `itempatch oracle: ok ...`; a sweep fails on any other line and on a log
// with no `ok` line (a suite the oracle never saw).
use super::session::{Caches, Front, Session};
use crate::ast::ItemKind;
use crate::check::collect::Program;
use crate::check::generics::FnClass;
use crate::def::{DefId, LocalDefId};
use crate::diag::Diag;
use crate::driver::graph::{Fingerprint, Head, ImplSetKey, ReadKey};
use crate::intern::FxMap;
use crate::protocol::{Cmd, Request, Response};
use crate::resolve::imports::ResolvedTables;
use std::sync::Arc;

/// True when `LUST_CHECK_ITEMPATCH` is set (and not `0`).
pub(crate) fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("LUST_CHECK_ITEMPATCH").is_ok_and(|v| !v.is_empty() && v != "0"))
}

/// What the cold resolve of a patched request needs: the last full
/// resolve's crate input (the root text, splice start, and client view
/// come from the patched front) and its std platform choice.
pub(crate) struct ColdInput {
    pub(crate) input: crate::resolve::worklist::CrateInput,
    pub(crate) platform: bool,
}

/// One def's caller-visible facts.
#[derive(Clone, PartialEq, Eq, Debug)]
struct DefFacts {
    sig: Fingerprint,
    layout: Option<Fingerprint>,
    class: Option<FnClass>,
}

/// The payload of an oracle failure panic (`fail`). The daemon answers
/// it as an internal error and never retries it in a fresh session,
/// which would hide the failure behind a correct answer.
pub(crate) struct Failure(pub(crate) String);

/// The oracle failure line in a panic payload, if it is one.
pub(crate) fn failure(payload: &(dyn std::any::Any + Send)) -> Option<&str> {
    payload.downcast_ref::<Failure>().map(|f| f.0.as_str())
}

/// The oracle's per-session state, on `Front` (interior mutability: the
/// compile tail holds the front shared).
#[derive(Default)]
pub(crate) struct State {
    /// The last request's program digest, for the no-reads rule.
    prev: Option<Digest>,
    /// This request's facts, for `compare_response`.
    snap: Option<Snap>,
    /// This request took a patch path.
    patched: bool,
}

/// What one request reached: its path, fn lines, and its collect errors
/// or program digest. `compare_response` compares the warm request's
/// with a fresh session's.
struct Snap {
    path: String,
    lines: Vec<u32>,
    collected: Result<Digest, Vec<ErrKey>>,
}

/// A collect error as the oracle compares it: code, text, file, span.
type ErrKey = (&'static str, String, crate::diag::FileId, u32, u32);

fn err_keys(ds: &[Diag]) -> Vec<ErrKey> {
    ds.iter().map(|d| (d.code, d.message.clone(), d.span.file, d.span.lo, d.span.hi)).collect()
}

/// The program facts the oracle compares, keyed by stable ids.
#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct Digest {
    defs: FxMap<DefId, DefFacts>,
    impl_sets: FxMap<(DefId, Head), Fingerprint>,
    /// Each user Adt's written std and lang trait impls (`LangImpls`).
    lang_impls: FxMap<DefId, Fingerprint>,
    hidden: FxMap<DefId, u64>,
    /// Each Adt instance's type (deep hash), in creation order: a kept
    /// program must hold the instances a cold collect makes.
    instances: Vec<u64>,
    /// Def names, for messages only (not compared).
    names: FxMap<DefId, String>,
}

impl Digest {
    /// The digest of `p` over its first `fn_count` fns (the AST fns),
    /// its Adts, traits, consts, statics, and user trait impl buckets.
    fn of(p: &Program, fn_count: usize) -> Digest {
        let mut d = Digest::default();
        let sig = |def: DefId| p.read_fingerprint(&ReadKey::Sig(def)).fingerprint;
        let layout = |def: DefId| p.read_fingerprint(&ReadKey::Layout(def)).fingerprint;
        for i in 0..fn_count.min(p.fns.len()) {
            let f = LocalDefId(i as u32);
            let def = p.def_id(f);
            d.names.insert(def, format!("fn {}", p.interner.get(p.fn_def(f).name)));
            d.defs.insert(def, DefFacts { sig: sig(def), layout: None, class: p.gx.fn_class.get(i).copied() });
        }
        for a in 0..p.adts.len() {
            // Generic Adt instances share their template's def; std
            // Adts may have none.
            let Some(def) = p.defs.def_of_adt(crate::ty::AdtId(a as u32)) else { continue };
            if p.adt_by_def(def) != Some(crate::ty::AdtId(a as u32)) {
                continue;
            }
            d.names.insert(def, format!("type {}", p.interner.get(p.adts[a].name)));
            d.defs.insert(def, DefFacts { sig: sig(def), layout: Some(layout(def)), class: None });
            if p.user_adt(def) {
                d.lang_impls.insert(def, p.read_fingerprint(&ReadKey::LangImpls(def)).fingerprint);
            }
        }
        for rec in p.gx.index.trait_records() {
            d.names.insert(rec.def, format!("trait {}", p.interner.get(rec.name)));
            d.defs.insert(rec.def, DefFacts { sig: sig(rec.def), layout: None, class: None });
            if !p.impls_read_covered(rec.def) {
                continue;
            }
            let mut heads = vec![Head::Unknown];
            for &id in p.gx.index.impls_of_trait(rec.def) {
                heads.push(Head::of(&p.tys, p.gx.index.get(id).self_ty(&p.tys)));
            }
            for head in heads {
                let key = ImplSetKey { trait_: rec.def, head };
                d.impl_sets.insert((rec.def, head), p.read_fingerprint(&ReadKey::ImplSet(key)).fingerprint);
            }
        }
        let consts = p.const_defs.iter().zip(p.consts.iter().map(|c| ("const", c.name)));
        // A hole of the statics table (`Program::static_live`) is no def.
        let statics = p.static_defs.iter().zip(p.statics.iter().map(|s| ("static", s.name))).filter(|(d, _)| d.hash.0 != 0);
        for (&def, (kind, name)) in consts.chain(statics) {
            d.names.entry(def).or_insert_with(|| format!("{kind} {}", p.interner.get(name)));
            d.defs.insert(def, DefFacts { sig: sig(def), layout: None, class: None });
        }
        for &a in &p.gx.adt_created {
            let Some(t) = p.tys.full_adt_ty(a) else { continue };
            let mut h = crate::intern::FxHasher::default();
            crate::driver::graph::hash_ty_deep(&mut h, p, &p.tys, t, &mut Default::default());
            d.instances.push(std::hash::Hasher::finish(&h));
        }
        for (&od, &t) in &p.gx.opaque_hidden {
            let mut h = crate::intern::FxHasher::default();
            crate::driver::graph::hash_ty_deep(&mut h, p, &p.tys, t, &mut Default::default());
            d.hidden.insert(od, std::hash::Hasher::finish(&h));
        }
        d
    }

    /// The first few differences between `self` (warm) and `cold`.
    fn diff(&self, cold: &Digest) -> Vec<String> {
        let name = |d: &DefId| cold.names.get(d).or_else(|| self.names.get(d)).cloned().unwrap_or_else(|| format!("{d:?}"));
        let mut out = Vec::new();
        for (d, c) in &cold.defs {
            match self.defs.get(d) {
                None => out.push(format!("{} only in cold", name(d))),
                Some(w) if w != c => out.push(format!("{}: warm {w:?}, cold {c:?}", name(d))),
                _ => {}
            }
        }
        out.extend(self.defs.keys().filter(|d| !cold.defs.contains_key(d)).map(|d| format!("{} only in warm", name(d))));
        for (k, c) in &cold.impl_sets {
            if self.impl_sets.get(k) != Some(c) {
                out.push(format!("impl set of {} for {:?}", name(&k.0), k.1));
            }
        }
        out.extend(
            self.impl_sets
                .keys()
                .filter(|k| !cold.impl_sets.contains_key(k))
                .map(|k| format!("impl set of {} for {:?} only in warm", name(&k.0), k.1)),
        );
        for (d, c) in &cold.lang_impls {
            if self.lang_impls.get(d) != Some(c) {
                out.push(format!("std and lang trait impls of {}", name(d)));
            }
        }
        out.extend(self.lang_impls.keys().filter(|d| !cold.lang_impls.contains_key(d)).map(|d| format!("std and lang trait impls of {} only in warm", name(d))));
        if self.instances != cold.instances {
            let at = self.instances.iter().zip(&cold.instances).position(|(a, b)| a != b);
            let same_set = self.instances.iter().all(|h| cold.instances.contains(h)) && cold.instances.iter().all(|h| self.instances.contains(h));
            out.push(format!("Adt instances: warm {}, cold {} (first difference at {at:?}, same set {same_set})", self.instances.len(), cold.instances.len()));
        }
        if self.hidden != cold.hidden {
            out.push(format!("hidden types: warm {} entries, cold {}", self.hidden.len(), cold.hidden.len()));
        }
        out.sort();
        out.truncate(8);
        out
    }
}

/// Appends one line to the file `LUST_ITEMPATCH_LOG` names, if set.
fn log(line: &str) {
    if let Some(log) = std::env::var_os("LUST_ITEMPATCH_LOG") {
        use std::io::Write;
        // One write per line: parallel writers append whole lines.
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log) {
            let _ = f.write_all(format!("{line}\n").as_bytes());
        }
    }
}

/// Reports one oracle failure: a log line, the same line on stderr, then
/// a panic with a `Failure` payload. Passing checks log
/// `itempatch oracle: ok ...`, so a sweep can count them; every other
/// line is a failure.
pub(super) fn fail(what: &str, path: &str, detail: &str) -> ! {
    let line = format!("itempatch oracle: {what} (path={path}): {detail}");
    log(&line);
    eprintln!("{line}");
    std::panic::panic_any(Failure(line));
}

/// The oracle check of one request (see the module docs), called by the
/// compile tail with the warm collect result and the fns it will check
/// (`work`). On a path that is not a patch the warm program is a full
/// collect: it checks the dirty set against it and stores the facts for
/// `compare_response`.
#[allow(clippy::too_many_arguments)]
pub(super) fn check(
    front: &Front,
    caches: &Caches,
    warm: Result<&Program, &[Diag]>,
    work: &[usize],
    fn_count: usize,
    body_hashes: &[u64],
    fn_lines: &[u32],
    path: &str,
    crate_ids: crate::def::CrateIds,
    build: bool,
    tables: Option<&Arc<ResolvedTables>>,
) {
    let warm_digest = warm.ok().map(|p| Digest::of(p, fn_count));
    let patched = matches!(path, "patch" | "item");
    let prev = {
        let mut st = front.oracle.lock().unwrap_or_else(|e| e.into_inner());
        let collected = warm_digest.clone().ok_or_else(|| warm.err().map(err_keys).unwrap_or_default());
        st.snap = Some(Snap { path: path.to_string(), lines: fn_lines.to_vec(), collected });
        st.patched = patched;
        std::mem::replace(&mut st.prev, warm_digest.clone())
    };
    if !patched {
        // A full collect: the warm program is the cold one, so only the
        // dirty set needs a check (signatures mode reuses bodies). A
        // one-shot session reuses nothing and logs nothing, so a sweep's
        // cold side cannot count as oracle coverage.
        if !caches.persistent {
            return;
        }
        if let (Ok(p), Some(d)) = (warm, warm_digest.as_ref()) {
            dirty_check(front, caches, p, p, work, fn_count, body_hashes, prev.as_ref(), d, path);
            log(&format!("itempatch oracle: ok dirty path={path}"));
        }
        return;
    }
    let Some(base) = front.patch.as_ref() else { fail("patch path with no patch base", path, "") };
    let (Some(cold_in), Some(tables)) = (base.oracle_input.as_ref(), tables) else {
        fail("patch path with no cold resolve input", path, "")
    };
    let mut input = cold_in.input.clone();
    input.root_text.clone_from(&front.src);
    input.client = front.client.clone();
    input.std_splice_lo = base.std_ranges().first().map(|r| r.0);
    let platform = cold_in.platform;
    let (std_ranges, std_routes) = (base.std_ranges().to_vec(), base.std_routes().to_vec());
    let mut cache = crate::resolve::worklist::ResolveCache::default();
    let cold = match crate::resolve::worklist::resolve_crate_with(&input, &front.interner, &front.known, &mut cache, platform) {
        Ok(r) => r,
        Err(e) => fail("cold resolve rejects", path, &e.diags.first().map_or(String::new(), |d| d.message.clone())),
    };
    // Resolve: epoch, item scans, fn hashes and lines.
    let (we, ce) = (tables.rewrite_epoch(), cold.tables.rewrite_epoch());
    if we != ce {
        fail("rewrite epoch differs", path, &format!("warm {we:x}, cold {ce:x}"));
    }
    let mut cold_items = cold.scans.items.clone();
    cold_items.push(crate::scan::ItemScan { hash: cold.structure_hash, fn_name: None });
    if cold_items != front.items {
        let k = cold_items.iter().zip(&front.items).position(|(a, b)| a != b).unwrap_or(cold_items.len().min(front.items.len()));
        let name = |a: &crate::ast::Ast| {
            a.items.get(k).map_or_else(String::new, |it| match &it.kind {
                ItemKind::Fn(f) => format!("fn {}", front.interner.get(f.name)),
                ItemKind::Struct(st) => format!("struct {}", front.interner.get(st.name)),
                ItemKind::Enum(e) => format!("enum {}", front.interner.get(e.name)),
                ItemKind::Impl(_) => "impl".to_string(),
                _ => "other".to_string(),
            })
        };
        let detail = format!(
            "first at item {k} of {} (warm {}): warm {} {:?}, cold {} {:?}",
            cold_items.len(),
            front.items.len(),
            name(&front.ast),
            front.items.get(k).map(|s| (s.hash, s.fn_name.is_some())),
            name(&cold.ast),
            cold_items.get(k).map(|s| (s.hash, s.fn_name.is_some()))
        );
        fail("item scans differ", path, &detail);
    }
    let locs = super::compile::fn_locations(&cold.ast);
    if locs.len() != fn_count || cold.fn_lines.len() != fn_count {
        fail("fn count differs", path, &format!("warm {fn_count}, cold {}", locs.len()));
    }
    for (i, ((&(item, sub), b), &line)) in locs.iter().zip(&cold.scans.bodies).zip(&cold.fn_lines).enumerate() {
        let def = super::compile::fn_def_at(&cold.ast, item, sub);
        let h = super::compile::fn_text_hash(&cold.ast, &cold.map, def, line, b.0);
        if body_hashes.get(i) != Some(&h) || fn_lines.get(i) != Some(&line) {
            let warm = (body_hashes.get(i).copied().unwrap_or(0), fn_lines.get(i).copied().unwrap_or(0));
            let wloc = front.fn_loc.get(i).copied().unwrap_or((usize::MAX, None));
            let wdef = super::compile::fn_def_at(&front.ast, wloc.0, wloc.1);
            let wraw = super::ast_hash::body_scan(&front.ast, wdef, None).0;
            let craw = super::ast_hash::body_scan(&cold.ast, def, None).0;
            let detail = format!(
                "fn {} (#{i}): warm {:x} line {}, cold {h:x} line {line}; raw body hash warm {wraw:x} cold {craw:x} (cold scan {:x})",
                front.interner.get(def.name),
                warm.0,
                warm.1,
                b.0
            );
            fail("fn body hash or line differs", path, &detail);
        }
    }
    // Collect, as the compile tail runs it over a full resolve.
    let mut ast = cold.ast;
    ast.shipped.clone_from(&std_ranges);
    ast.lang_methods = front.ast.lang_methods;
    let mut cold_tables = cold.tables;
    cold_tables.std_ranges.clone_from(&std_ranges);
    let mut reuse = crate::driver::gen_reuse::GenReuse::default();
    // The session's def numbering (static and Adt indices), which the
    // warm collect grew: the cold program numbers its defs the same way.
    let o = &front.def_order;
    let mut order = crate::check::collect::DefOrder { adts: o.adts.clone(), statics: o.statics.clone(), instances: o.instances.clone() };
    let collected = crate::check::collect::collect_reuse(
        Arc::new(ast),
        front.interner.clone(),
        front.known.clone(),
        crate_ids,
        build,
        &mut order,
        Some(Arc::new(cold_tables)),
        &mut reuse,
        &std_routes,
    );
    let mut cold_p = match (warm, collected) {
        (Err(w), Err(c)) => {
            let (wk, ck) = (err_keys(w), err_keys(&c));
            if wk != ck {
                fail("collect errors differ", path, &format!("warm {:?}, cold {:?}", wk.first(), ck.first()));
            }
            log(&format!("itempatch oracle: ok errors path={path}"));
            return;
        }
        (Ok(_), Err(c)) => fail("cold collect rejects", path, &format!("{} {}", c[0].code, c[0].message)),
        (Err(w), Ok(_)) => fail("warm collect rejects", path, &format!("{} {}", w[0].code, w[0].message)),
        (Ok(_), Ok(p)) => p,
    };
    let warm = warm.ok().expect("warm program");
    cold_p.shipped_ranges.clone_from(&warm.shipped_ranges);
    cold_p.std_fns.clone_from(&warm.std_fns);
    let cold_digest = Digest::of(&cold_p, fn_count);
    let diff = warm_digest.as_ref().map(|w| w.diff(&cold_digest)).unwrap_or_default();
    if !diff.is_empty() {
        fail("program digest differs", path, &diff.join("; "));
    }
    let reused = dirty_check(front, caches, warm, &cold_p, work, fn_count, body_hashes, prev.as_ref(), &cold_digest, path);
    // The cold collect marked laps of its own.
    let _ = super::laps::take();
    log(&format!("itempatch oracle: ok program path={path} reused={reused}"));
}

/// The dirty set half (see the module docs): every AST fn the request
/// does not check again (`work`) saw its current body at its last check,
/// and its stored reads are clean against `cold`; with no stored reads,
/// the digest (`now`) must equal the last request's (`prev`). Returns
/// how many fns it reuses.
#[allow(clippy::too_many_arguments)]
fn dirty_check(
    front: &Front,
    caches: &Caches,
    warm: &Program,
    cold_p: &Program,
    work: &[usize],
    fn_count: usize,
    body_hashes: &[u64],
    prev: Option<&Digest>,
    now: &Digest,
    path: &str,
) -> usize {
    let mut checked = vec![false; fn_count];
    work.iter().filter(|&&i| i < fn_count).for_each(|&i| checked[i] = true);
    for i in 0..fn_count {
        let f = LocalDefId(i as u32);
        let concrete = matches!(warm.gx.fn_class.get(i), Some(FnClass::Plain) | None);
        let slot = front.fn_slot[i] as usize;
        if checked[i] || !concrete || warm.check_skips(f) || !caches.ok.get(slot).copied().unwrap_or(false) {
            continue;
        }
        let name = || format!("fn {} (#{i})", front.interner.get(warm.fn_def(f).name));
        // Its last check saw this body (`slot_hash`, set at that check).
        if front.slot_hash.get(slot) != body_hashes.get(i) {
            fail("reused fn's body changed since its check", path, &name());
        }
        match caches.slot_reads.get(slot).and_then(Option::as_ref) {
            Some(reads) => {
                // A `Checked` read names a body: dirty when that body checks again.
                let dirty: Vec<String> = reads
                    .iter(&caches.read_keys)
                    .filter(|(k, stored)| match k {
                        ReadKey::Checked(d) => warm.local_def(d.hash).is_some_and(|l| checked.get(l.0 as usize).copied().unwrap_or(false)),
                        ReadKey::Binding(b) if cold_p.read_fingerprint(k) != *stored => !pattern_read(cold_p, front, i, b),
                        _ => cold_p.read_fingerprint(k) != *stored,
                    })
                    .map(|(k, stored)| format!("{k:?} stored {:?} now {:?}", stored, cold_p.read_fingerprint(k)))
                    .take(3)
                    .collect();
                if !dirty.is_empty() {
                    fail("reused fn has dirty reads", path, &format!("{}: {}", name(), dirty.join(", ")));
                }
            }
            None if prev.is_some_and(|p| p != now) => {
                fail("reused fn with no stored reads after a digest change", path, &name());
            }
            None => {}
        }
    }
    (0..fn_count).filter(|&i| !checked[i]).count()
}

/// True for a value binding read that names a fn the reused body never
/// mentions (its idents hold no path to it): the check read it only to
/// tell a binding pattern from a const or unit struct pattern (a
/// parameter named like the fn). A signature edit moves its fingerprint
/// but not that answer, which only an item kind change moves (an outline
/// change, so full mode). Signatures mode walks only mentioning bodies.
fn pattern_read(p: &Program, front: &Front, i: usize, b: &crate::driver::graph::BindingKey) -> bool {
    let crate::driver::graph::BindingRes::Def(d) = p.binding_res_at(b) else {
        return false;
    };
    let Some(f) = p.local_def(d.hash).filter(|l| p.def_id(*l) == d) else { return false };
    let flat = p.fn_def(f).name;
    front.chunks.first().and_then(|c| c.scan.body_idents.get(i)).is_some_and(|ids| !ids.contains(&flat))
}

/// True when `LUST_CHECK_B1` is set (and not `0`): `check_pool` runs.
pub(crate) fn pool_on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("LUST_CHECK_B1").is_ok_and(|v| !v.is_empty() && v != "0"))
}

/// The type pool oracle (`LUST_CHECK_B1=1`, B1-PLAN.md): a collect that
/// started from the last program's type store (`ty::TypePool`) must give
/// what a collect from a fresh store gives, with fresh reuse caches: the
/// same errors, or the same program digest over every fn row (instances and
/// closures too), Adt, trait, const and static, impl sets, hidden types and
/// Adt instances. The digest reads structure (fingerprints, deep type
/// hashes), never type ids, so the two stores compare. The fresh collect
/// starts from a copy of the session's def order, as `check` does.
#[allow(clippy::too_many_arguments)]
pub(super) fn check_pool(
    front: &Front,
    crate_ids: crate::def::CrateIds,
    build: bool,
    tables: Option<Arc<ResolvedTables>>,
    std_routes: &[&'static crate::ty::lang::StdFnDef],
    pooled: &Result<Program, Vec<Diag>>,
) {
    let o = &front.def_order;
    let mut order = crate::check::collect::DefOrder { adts: o.adts.clone(), statics: o.statics.clone(), instances: o.instances.clone() };
    let mut reuse = crate::driver::gen_reuse::GenReuse::default();
    let fresh = crate::check::collect::collect_reuse(
        front.ast.clone(),
        front.interner.clone(),
        front.known.clone(),
        crate_ids,
        build,
        &mut order,
        tables,
        &mut reuse,
        std_routes,
    );
    let _ = super::laps::take();
    match (pooled, &fresh) {
        (Err(w), Err(c)) => {
            let (wk, ck) = (err_keys(w), err_keys(c));
            if wk != ck {
                fail("type pool collect errors differ from a fresh store", "pool", &format!("pool {:?}, fresh {:?}", wk.first(), ck.first()));
            }
        }
        (Ok(_), Err(c)) => fail("type pool collect accepts, a fresh store rejects", "pool", &format!("{} {}", c[0].code, c[0].message)),
        (Err(w), Ok(_)) => fail("type pool collect rejects, a fresh store accepts", "pool", &format!("{} {}", w[0].code, w[0].message)),
        (Ok(p), Ok(f)) => {
            let (dp, df) = (Digest::of(p, p.fns.len()), Digest::of(f, f.fns.len()));
            let diff = dp.diff(&df);
            if !diff.is_empty() || p.fns.len() != f.fns.len() {
                fail("type pool program differs from a fresh store", "pool", &format!("fns {} vs {}; {}", p.fns.len(), f.fns.len(), diff.join("; ")));
            }
        }
    }
    log("itempatch oracle: ok pool");
}

/// Drops this request's facts unchecked (`driver::compile_background`).
pub(crate) fn forget(session: &mut Session) {
    let mut st = session.front.oracle.lock().unwrap_or_else(|e| e.into_inner());
    (st.snap, st.patched) = (None, false);
}

/// The response half of the oracle: every daemon check request compiles
/// again in a fresh session, which must answer the same bytes and reach
/// the same facts (`Snap`) when both requests reached collect. The fresh
/// session starts from this session's def numbering, as a restarted
/// daemon does from its disk cache: fingerprints hold Adt and static
/// indices. One-shot sessions are cold already and skip it. A
/// `--timings` request compares with an untimed cold compile, up to its
/// timing report (`lust check <file>: ok (...` and the laps after it).
pub(crate) fn compare_response(req: &Request, session: &mut Session, warm: &Response) {
    let (snap, patched) = {
        let mut st = session.front.oracle.lock().unwrap_or_else(|e| e.into_inner());
        (st.snap.take(), std::mem::take(&mut st.patched))
    };
    if !session.caches.persistent || req.cmd != Cmd::Check {
        return;
    }
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("lust-itempatch-{}-{n}", std::process::id()));
    let mut cold = Session::new(dir.clone(), false, req.session.clone());
    // No disk cache: it would read and write the daemon's stored state.
    cold.cache_tried = true;
    let o = &session.front.def_order;
    cold.front.def_order = crate::check::collect::DefOrder { adts: o.adts.clone(), statics: o.statics.clone(), instances: o.instances.clone() };
    let untimed = Request { timings: false, ..req.clone() };
    let resp = super::compile::compile(&untimed, &mut cold);
    let cold_snap = cold.front.oracle.lock().unwrap_or_else(|e| e.into_inner()).snap.take();
    drop(cold);
    let _ = std::fs::remove_dir_all(&dir);
    let path = snap.as_ref().map_or("none", |s| s.path.as_str());
    // `LUST_STATS` lines count work, which differs by design. A failed
    // `cargo metadata` quotes Cargo's own text, which holds lock waits
    // ("Blocking waiting for file lock") by timing: only the code counts.
    let answer = |s: &str| {
        let lines = s.lines().take_while(|l| !(req.timings && l.starts_with("lust check ") && l.contains(": ok (")));
        lines.filter(|l| !l.starts_with("lust-stats: ") && !l.starts_with("lust-visits: ")).collect::<Vec<_>>().join("\n")
    };
    let cargo_failed = |s: &str| s.contains("`cargo metadata` failed");
    let same_text = answer(&resp.stderr) == answer(&warm.stderr) || cargo_failed(&resp.stderr) && cargo_failed(&warm.stderr);
    if resp.code != warm.code || resp.stdout != warm.stdout || !same_text {
        let first = |s: &str| s.lines().next().unwrap_or("").chars().take(160).collect::<String>();
        fail(
            "response differs from a cold compile",
            path,
            &format!("warm {} `{}`, cold {} `{}`", warm.code, first(&answer(&warm.stderr)), resp.code, first(&answer(&resp.stderr))),
        );
    }
    if let (Some(w), Some(c)) = (&snap, &cold_snap) {
        // Body hashes fold session interner ids, so only lines compare
        // across sessions (the patch half compares hashes in one).
        if w.lines != c.lines {
            let detail = match (0..w.lines.len().min(c.lines.len())).find(|&i| w.lines[i] != c.lines[i]) {
                Some(i) => format!("fn #{i}: warm line {}, cold line {}", w.lines[i], c.lines[i]),
                None => format!("fn count: warm {}, cold {}", w.lines.len(), c.lines.len()),
            };
            fail("fn lines differ from a cold compile", path, &detail);
        }
        match (&w.collected, &c.collected) {
            (Ok(wd), Ok(cd)) => {
                let diff = wd.diff(cd);
                if !diff.is_empty() {
                    fail("program digest differs from a cold compile", path, &diff.join("; "));
                }
            }
            (Err(we), Err(ce)) if we == ce => {}
            (we, ce) => {
                let first = |r: &Result<Digest, Vec<ErrKey>>| r.as_ref().err().and_then(|e| e.first().cloned());
                fail("collect errors differ from a cold compile", path, &format!("warm {:?}, cold {:?}", first(we), first(ce)));
            }
        }
    }
    let facts = if snap.is_some() && cold_snap.is_some() { "facts" } else { "bytes" };
    log(&format!("itempatch oracle: ok response path={path} patched={patched} compared={facts}"));
}
