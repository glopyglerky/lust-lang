// The resolve oracle (`LUST_CHECK_RESOLVE=1`; B3 step 1, after the review in
// B3-REVIEW-SOL.md). After each request's front end, a cold resolve of the
// same input, in the same process and interner but with a fresh
// `ResolveCache`, must agree with the warm state:
// - a full resolve through the session's cache: the same answer. When both
//   resolve: every `ResolvedTables` field, the source map, and the Ast (item
//   scans, fn body hashes and lines, the structure hash, node counts, and
//   every item, expression, pattern and type span). Not the Ast's string
//   table: a fresh lex keeps literals that a seeded or cached one leaves out
//   (a `macro_rules!` body's), which no node reads; every literal a node
//   reads joins the item and body hashes by content. When both reject: the
//   same diagnostics in order, with labels, fixes and positions.
// - a body or item patch: the patched tables, item scans and fn lines.
// - a reused answer (no file changed): the patch base's tables, or the same
//   rejection.
// Ids compare as numbers: both sides list modules and files in the same
// order, or their dumps differ there. Expansion contexts compare by what they
// stand for (`diag::ctxt_hash`), hash maps by sorted entries. Any difference
// appends one line to the file `LUST_RESOLVE_LOG` (else `LUST_ITEMPATCH_LOG`)
// names, prints it, and panics
// with the item oracle's `Failure` payload, so a daemon answers code 3 and
// never retries the request in a fresh session. Passing checks log
// `resolve oracle: ok path=...`.
use super::item_oracle::{ColdInput, Failure};
use super::session::Front;
use crate::diag::{CtxtId, Diag, SourceMap, Span};
use crate::intern::{FxMap, Interner, Sym};
use crate::protocol::Response;
use crate::resolve::imports::ResolvedTables;
use crate::resolve::worklist::{self, ResolveCache, ResolveError, ResolvedCrate};
use std::fmt::Debug;

/// True when `LUST_CHECK_RESOLVE` is set (and not `0`).
pub(crate) fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("LUST_CHECK_RESOLVE").is_ok_and(|v| !v.is_empty() && v != "0"))
}

/// Appends one line to the file `LUST_RESOLVE_LOG` names, else the item
/// oracle's `LUST_ITEMPATCH_LOG` (the sweeps read one log per daemon).
fn log(line: &str) {
    if let Some(log) = std::env::var_os("LUST_RESOLVE_LOG").or_else(|| std::env::var_os("LUST_ITEMPATCH_LOG")) {
        use std::io::Write;
        // One write per line: parallel writers append whole lines.
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(log) {
            let _ = f.write_all(format!("{line}\n").as_bytes());
        }
    }
}

/// Reports one difference: a log line, the same line on stderr, then a panic
/// with a `Failure` payload.
fn fail(what: &str, path: &str, detail: &str) -> ! {
    let line = format!("resolve oracle: {what} (path={path}): {detail}");
    log(&line);
    eprintln!("{line}");
    std::panic::panic_any(Failure(line));
}

/// `text` with each `name(<digits>)` replaced by `f` of the number.
fn rewrite(text: &str, name: &str, mut f: impl FnMut(u32) -> String) -> String {
    let pat = format!("{name}(");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(&pat) {
        let after = &rest[i + pat.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        // `XCtxtId(` is another name.
        let starts = rest[..i].bytes().next_back().is_none_or(|b| !b.is_ascii_alphanumeric() && b != b'_');
        if starts && digits > 0 && after[digits..].starts_with(')') {
            out.push_str(&rest[..i]);
            out.push_str(&f(after[..digits].parse().unwrap_or(0)));
            rest = &after[digits + 1..];
        } else {
            out.push_str(&rest[..i + pat.len()]);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

/// Debug lines of one side, with contexts by hash.
#[derive(Default)]
struct Lines {
    memo: FxMap<u32, u64>,
    out: Vec<String>,
}

impl Lines {
    fn text(&mut self, v: &dyn Debug) -> String {
        let memo = &mut self.memo;
        rewrite(&format!("{v:?}"), "CtxtId", |n| format!("#{:x}", crate::diag::ctxt_hash(CtxtId(n), memo)))
    }

    fn line(&mut self, field: &str, key: &dyn Debug, value: &dyn Debug) {
        let (k, v) = (self.text(key), self.text(value));
        self.out.push(format!("{field} {k} = {v}"));
    }

    fn span(&mut self, s: Span) -> (crate::diag::FileId, u32, u32, u64) {
        (s.file, s.lo, s.hi, crate::diag::ctxt_hash(s.ctxt, &mut self.memo))
    }
}

/// Every field of `t` as sorted lines. Hash map entries go one per line, so
/// their order does not count; vectors keep their order (their index is the
/// key, or the whole vector is one line).
fn table_lines(t: &ResolvedTables) -> Vec<String> {
    fn sorted<K: Debug + Ord, V: Debug>(m: impl IntoIterator<Item = (K, V)>) -> Vec<(K, V)> {
        let mut v: Vec<(K, V)> = m.into_iter().collect();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }
    let mut d = Lines::default();
    for (i, m) in t.modules.iter().enumerate() {
        d.line("modules", &i, m);
    }
    for (i, maps) in t.maps.iter().enumerate() {
        for (ns, map) in maps.iter().enumerate() {
            for (s, b) in map {
                d.line("maps", &(i, ns, s), b);
            }
        }
    }
    d.line("glob_log", &"", &t.glob_log);
    d.line("externs", &"", &t.externs);
    for (def, m) in &t.assoc_vis {
        for (s, v) in m {
            d.line("assoc_vis", &(def, s), v);
        }
    }
    for (def, (m, fields)) in &t.field_vis {
        d.line("field_vis", def, &(m, sorted(fields)));
    }
    for (def, v) in &t.tuple_field_vis {
        d.line("tuple_field_vis", def, v);
    }
    for (def, v) in &t.variants {
        d.line("variants", def, v);
    }
    for (def, v) in &t.def_info {
        d.line("def_info", def, v);
    }
    for (s, m) in &t.const_init_module {
        let s = d.span(*s);
        d.line("const_init_module", &s, m);
    }
    for (a, b) in &t.display_names {
        d.line("display_names", a, b);
    }
    d.line("module_defs", &"", &t.module_defs);
    for (ns, m) in t.flat.iter().enumerate() {
        for (s, def) in m {
            d.line("flat", &(ns, s), def);
        }
    }
    d.line("fn_module", &"", &t.fn_module);
    for (s, m) in &t.crate_magic {
        d.line("crate_magic", s, m);
    }
    for (k, v) in &t.std_aliases {
        d.line("std_aliases", k, v);
    }
    for (k, v) in &t.std_glob_aliases {
        d.line("std_glob_aliases", k, v);
    }
    for (k, v) in t.glob_src.iter() {
        d.line("glob_src", &k, &v);
    }
    for (k, v) in &t.pending_single {
        d.line("pending_single", k, v);
    }
    d.line("import_stall", &"", &t.import_stall);
    d.line("single_binds", &"", &t.single_binds);
    for (k, v) in &t.ambig_vis {
        d.line("ambig_vis", k, v);
    }
    d.line("ambig_single", &"", &t.ambig_single);
    for (k, v) in &t.order_dep {
        d.line("order_dep", k, v);
    }
    for (k, v) in &t.macro_textual_binds {
        d.line("macro_textual_binds", k, v);
    }
    for (k, v) in &t.macro_scopes {
        d.line("macro_scopes", k, v);
    }
    for (k, v) in &t.crate_cfgs {
        d.line("crate_cfgs", k, v);
    }
    d.line("adapters", &"", &t.adapters);
    d.line("std_globs", &"", &t.std_globs);
    d.line("std_platform", &"", &t.std_platform);
    d.line("std_modules", &"", &t.std_modules);
    d.line("std_splice_lo", &"", &t.std_splice_lo);
    d.line("std_ranges", &"", &t.std_ranges);
    d.line("alloc_roots", &"", &t.alloc_roots);
    d.line("module_alts", &"", &t.module_alts);
    d.out.sort_unstable();
    d.out
}

/// `line` for a message: symbols by name, cut to 400 bytes.
fn readable(line: &str, interner: &Interner) -> String {
    let mut s = rewrite(line, "Sym", |n| format!("`{}`", interner.get(Sym(n))));
    if s.len() > 400 {
        let mut cut = 400;
        while !s.is_char_boundary(cut) {
            cut -= 1;
        }
        s.truncate(cut);
        s.push_str("...");
    }
    s
}

/// Fails when two sorted line lists differ, naming the first few lines only
/// one side has.
fn compare_lines(what: &str, path: &str, warm: &[String], cold: &[String], interner: &Interner) {
    if warm == cold {
        return;
    }
    let (mut i, mut j) = (0, 0);
    let mut diffs: Vec<String> = Vec::new();
    while (i < warm.len() || j < cold.len()) && diffs.len() < 6 {
        match (warm.get(i), cold.get(j)) {
            (Some(w), Some(c)) if w == c => {
                i += 1;
                j += 1;
            }
            (Some(w), Some(c)) if w < c => {
                diffs.push(format!("warm only: {}", readable(w, interner)));
                i += 1;
            }
            (Some(w), None) => {
                diffs.push(format!("warm only: {}", readable(w, interner)));
                i += 1;
            }
            (_, Some(c)) => {
                diffs.push(format!("cold only: {}", readable(c, interner)));
                j += 1;
            }
            (None, None) => break,
        }
    }
    fail(what, path, &format!("{} warm lines, {} cold lines; {}", warm.len(), cold.len(), diffs.join("; ")));
}

/// Each diagnostic in full, in order.
fn diag_lines(ds: &[Diag]) -> Vec<String> {
    let mut d = Lines::default();
    for (i, x) in ds.iter().enumerate() {
        d.line("diag", &i, x);
    }
    d.out
}

/// Each file of `map`: its path and text, in id order.
fn map_lines(map: &SourceMap) -> Vec<String> {
    (0..map.file_count())
        .map(|k| {
            let f = crate::diag::FileId(k as u32);
            format!("file {k} {} {:x}", map.path(f), map.text(f).map_or(0, |t| crate::driver::cache::hash128(&[t.as_bytes()], 0).0))
        })
        .collect()
}

/// The first index where two span lists differ, as a message.
fn first_span_diff(warm: &[Span], cold: &[Span], wl: &mut Lines, cl: &mut Lines) -> Option<String> {
    if warm.len() != cold.len() {
        return Some(format!("warm {} spans, cold {}", warm.len(), cold.len()));
    }
    warm.iter().zip(cold).enumerate().find_map(|(i, (&w, &c))| {
        let (w, c) = (wl.span(w), cl.span(c));
        (w != c).then(|| format!("#{i}: warm {w:?}, cold {c:?}"))
    })
}

/// The Ast half of a full resolve's comparison.
fn compare_crates(w: &ResolvedCrate, c: &ResolvedCrate, path: &str, interner: &Interner) {
    compare_lines("source map differs", path, &map_lines(&w.map), &map_lines(&c.map), interner);
    if w.all_text != c.all_text {
        fail("crate text differs", path, &format!("warm {} bytes, cold {}", w.all_text.len(), c.all_text.len()));
    }
    let scalars = |r: &ResolvedCrate| format!("{:?} {:?} {:?} {:x}", r.std_asks, r.platform_asks, r.expand_deps, r.structure_hash);
    if scalars(w) != scalars(c) {
        fail("std asks, expansion inputs or structure hash differ", path, &format!("warm {}, cold {}", scalars(w), scalars(c)));
    }
    if w.scans.items != c.scans.items {
        let k = w.scans.items.iter().zip(&c.scans.items).position(|(a, b)| a != b);
        fail("item scans differ", path, &format!("warm {} items, cold {}, first difference at {k:?}", w.scans.items.len(), c.scans.items.len()));
    }
    if w.scans.bodies != c.scans.bodies || w.fn_lines != c.fn_lines || w.body_hashes != c.body_hashes {
        let k = (0..w.fn_lines.len().max(c.fn_lines.len())).find(|&i| {
            w.fn_lines.get(i) != c.fn_lines.get(i) || w.body_hashes.get(i) != c.body_hashes.get(i) || w.scans.bodies.get(i) != c.scans.bodies.get(i)
        });
        fail("fn bodies or lines differ", path, &format!("warm {} fns, cold {}, first difference at fn #{k:?}", w.fn_lines.len(), c.fn_lines.len()));
    }
    let (wa, ca) = (&w.ast, &c.ast);
    let sizes = |a: &crate::ast::Ast| [a.inner_attrs.len(), a.items.len(), a.stmt_items.len(), a.exprs.len(), a.pats.len(), a.tys.len(), a.blocks.len(), a.blobs.len()];
    if sizes(wa) != sizes(ca) {
        fail("Ast sizes differ", path, &format!("warm {:?}, cold {:?} (attrs, items, body items, exprs, pats, tys, blocks, blobs)", sizes(wa), sizes(ca)));
    }
    let (mut wl, mut cl) = (Lines::default(), Lines::default());
    let item_spans = |a: &crate::ast::Ast| a.items.iter().chain(&a.stmt_items).map(|it| it.span).collect::<Vec<Span>>();
    let (wi, ci) = (item_spans(wa), item_spans(ca));
    let lists: [(&str, &[Span], &[Span]); 4] = [
        ("item", &wi, &ci),
        ("expression", &wa.expr_spans, &ca.expr_spans),
        ("pattern", &wa.pat_spans, &ca.pat_spans),
        ("type", &wa.ty_spans, &ca.ty_spans),
    ];
    for (what, ws, cs) in lists {
        if let Some(d) = first_span_diff(ws, cs, &mut wl, &mut cl) {
            fail(&format!("{what} spans differ"), path, &d);
        }
    }
}

/// The cold resolve of `input`.
fn cold(front: &Front, cold_in: &ColdInput) -> Result<ResolvedCrate, ResolveError> {
    let mut cache = ResolveCache::default();
    let out = worklist::resolve_crate_with(&cold_in.input, &front.interner, &front.known, &mut cache, cold_in.platform);
    // The cold resolve marked laps of its own.
    let _ = super::laps::take();
    out
}

/// The oracle check of a full resolve: `warm` is its result, `cold_in` its
/// last pass's input, `why` why no patch path took the request.
pub(super) fn check_full(front: &Front, cold_in: &ColdInput, warm: &Result<ResolvedCrate, ResolveError>, why: &str) {
    let path = format!("resolve why={}", if why.is_empty() { "-" } else { why });
    let interner = &front.interner;
    match (warm, cold(front, cold_in)) {
        (Ok(w), Ok(c)) => {
            compare_lines("tables differ", &path, &table_lines(&w.tables), &table_lines(&c.tables), interner);
            compare_crates(w, &c, &path, interner);
            log(&format!("resolve oracle: ok path={path} tables"));
        }
        (Err(w), Err(c)) => {
            compare_lines("diagnostics differ", &path, &diag_lines(&w.diags), &diag_lines(&c.diags), interner);
            compare_lines("source map differs", &path, &map_lines(&w.map), &map_lines(&c.map), interner);
            log(&format!("resolve oracle: ok path={path} errors={}", w.diags.len()));
        }
        (Ok(_), Err(c)) => fail("cold resolve rejects", &path, &c.diags.first().map_or(String::new(), |d| format!("{} {}", d.code, d.message))),
        (Err(w), Ok(_)) => fail("warm resolve rejects", &path, &w.diags.first().map_or(String::new(), |d| format!("{} {}", d.code, d.message))),
    }
}

/// The oracle check of a patch path: the patched tables, item scans (the
/// structure hash last) and fn lines, and `reused`, the last answer when the
/// request takes it again.
pub(super) fn check_patch(front: &Front, tables: &ResolvedTables, items: &[crate::scan::ItemScan], fn_lines: &[u32], reused: Option<&Response>) {
    let path = if reused.is_some() { "reuse" } else { "patch" };
    let Some(base) = front.patch.as_ref() else { fail("patch path with no patch base", path, "") };
    let Some(cold_in) = base.oracle_input.as_ref() else { fail("patch path with no cold resolve input", path, "") };
    let mut input = ColdInput { input: cold_in.input.clone(), platform: cold_in.platform };
    input.input.root_text.clone_from(&front.src);
    input.input.client = front.client.clone();
    input.input.std_splice_lo = base.std_ranges().first().map(|r| r.0);
    let interner = &front.interner;
    let c = match cold(front, &input) {
        Ok(c) => c,
        // A cold rejection stands only for a reused rejection that says the same.
        Err(e) => {
            let first = e.diags.first().map_or(String::new(), |d| d.message.clone());
            match reused {
                Some(r) if r.code != 0 && r.stderr.contains(&first) => {
                    log(&format!("resolve oracle: ok path={path} errors={}", e.diags.len()));
                    return;
                }
                _ => fail("cold resolve rejects", path, &first),
            }
        }
    };
    // The driver sets the splice ranges after a resolve (`resolved_setup`); a patched base keeps them.
    let mut c = c;
    c.tables.std_ranges = base.std_ranges().to_vec();
    compare_lines("tables differ", path, &table_lines(tables), &table_lines(&c.tables), interner);
    let mut cold_items = c.scans.items.clone();
    cold_items.push(crate::scan::ItemScan { hash: c.structure_hash, fn_name: None });
    if cold_items != items {
        let k = cold_items.iter().zip(items).position(|(a, b)| a != b);
        fail("item scans differ", path, &format!("warm {} items, cold {}, first difference at {k:?}", items.len(), cold_items.len()));
    }
    if fn_lines != c.fn_lines {
        let k = (0..fn_lines.len().max(c.fn_lines.len())).find(|&i| fn_lines.get(i) != c.fn_lines.get(i));
        fail("fn lines differ", path, &format!("warm {} fns, cold {}, first difference at fn #{k:?}", fn_lines.len(), c.fn_lines.len()));
    }
    log(&format!("resolve oracle: ok path={path} tables"));
}
