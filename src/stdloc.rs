// Panic locations inside lust's shipped std source (tf-panicloc).
//
// rustc reports a panic that std raises in its own code (no
// `#[track_caller]` chain up to the user's call) at std's file, line, and
// column, e.g. `/rustc/<hash>/library/alloc/src/collections/vec_deque/mod.rs:2564:9`.
// lust's std source is its own text, so its lines are not std's. Each
// such panic site in library/{core,alloc,std} carries std's location as a
// marker comment at the end of the line where the site starts:
//
//     panic!("index out of bounds"); //@ alloc/src/collections/vec_deque/mod.rs:2564:9
//
// cgen's `loc` reports the marker's location for every panic site on that
// line of a fn without `#[track_caller]` (a `#[track_caller]` fn reports its
// caller's location, as rustc does). `//@ never` marks a site that cannot
// panic, and `//@ gap` a known gap whose panic reports lust's line (say why
// next to it). The lint test below wants a marker on each explicit panic
// site outside a `#[track_caller]` fn.
//
// The marker path indexes `STD_FILES`, which must equal `lust_std_files` in
// src/runtime/panic.c (a test checks it): the packed location carries the
// index (cgen `std_loc`). Indexes stay below 0x40, the `pow` bit.
use crate::check::collect::Program;
use crate::diag::{FileId, SourceMap, Span};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// The std files a packed std location can name, in `lust_std_files` order.
pub const STD_FILES: &[&str] = &[
    "core/src/num/mod.rs",
    "core/src/char/methods.rs",
    "core/src/char/convert.rs",
    "alloc/src/vec/mod.rs",
    "core/src/cmp.rs",
    "core/src/iter/adapters/step_by.rs",
    "alloc/src/vec/spec_from_iter_nested.rs",
    "core/src/iter/traits/accum.rs",
    "core/src/iter/range.rs",
    "core/src/cell.rs",
    "std/src/thread/local.rs",
    "core/src/ops/arith.rs",
    "core/src/slice/index.rs",
    "core/src/iter/traits/exact_size.rs",
    "core/src/slice/mod.rs",
    "core/src/ops/function.rs",
    "std/src/env.rs",
    "core/src/time.rs",
    "std/src/time.rs",
    "alloc/src/collections/btree/search.rs",
    "alloc/src/collections/vec_deque/mod.rs",
    "std/src/path.rs",
    "std/src/panicking.rs",
    "core/src/cell/once.rs",
    "core/src/cell/lazy.rs",
    "std/src/sync/lazy_lock.rs",
    "core/src/io/write.rs",
    "alloc/src/io/read.rs",
    "alloc/src/io/impls.rs",
    "core/src/io/borrowed_buf.rs",
    "alloc/src/rc.rs",
    "alloc/src/sync.rs",
    "core/src/str/mod.rs",
    "std/src/os/unix/fs.rs",
    "alloc/src/str.rs",
];

/// One marker: an index into `STD_FILES` with std's line and column, a
/// site that cannot panic (`never`), or a known gap (`gap`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Marker {
    At(u32, u32, u32),
    Never,
    Gap,
}

/// The marker of the `//@` comment on `line`, if any. A malformed marker or
/// an unknown path is a std authoring bug; the test below rejects both.
pub fn parse_marker(line: &str) -> Option<Result<Marker, String>> {
    let at = line.find("//@ ")?;
    let text = line[at + 4..].trim();
    match text {
        "never" => return Some(Ok(Marker::Never)),
        "gap" => return Some(Ok(Marker::Gap)),
        _ => {}
    }
    let mut parts = text.rsplitn(3, ':');
    let (col, ln, path) = (parts.next(), parts.next(), parts.next());
    let parsed = (|| {
        let file = STD_FILES.iter().position(|f| Some(*f) == path)?;
        Some(Marker::At(file as u32, ln?.parse().ok()?, col?.parse().ok()?))
    })();
    Some(parsed.ok_or_else(|| format!("bad std location marker `{text}` (path not in stdloc::STD_FILES, or no line:col)")))
}

/// The markers of one std file, by 1-based line.
fn markers_of(text: &str) -> HashMap<u32, Marker> {
    text.lines()
        .enumerate()
        .filter_map(|(i, l)| match parse_marker(l)? {
            Ok(m) => Some((i as u32 + 1, m)),
            Err(_) => None,
        })
        .collect()
}

/// The marker table of one std file, cached per process under `key`;
/// `text` gives the file text on a miss.
fn markers(key: (String, usize, usize), text: impl FnOnce() -> Option<String>) -> Option<Arc<HashMap<u32, Marker>>> {
    static CACHE: OnceLock<Mutex<HashMap<(String, usize, usize), Arc<HashMap<u32, Marker>>>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().unwrap_or_else(|e| e.into_inner());
    if let Some(m) = cache.get(&key) {
        return Some(m.clone());
    }
    let m = Arc::new(markers_of(&text()?));
    cache.insert(key, m.clone());
    Some(m)
}

/// The std location rustc reports for a panic site at `span` (a call-site
/// span), when the site is in shipped std source and its line has a
/// marker: `(file index, line, col)`. None for user source, for std lines
/// without a marker, and for `//@ never` and `//@ gap` lines.
pub fn site(p: &Program, map: &SourceMap, span: Span) -> Option<(u32, u32, u32)> {
    let (line, table) = if span.file == FileId(0) {
        // A std source module: a hidden inline module after the user text
        // (stdmods::splice_text puts the file's line 1 on the line after
        // `pub mod <name> {`).
        if !p.std_source_span(span) {
            return None;
        }
        // The hidden module holding `span`: the last one that starts
        // before it (the modules are the splice's last blocks).
        let t = p.resolved.as_ref()?;
        let (name, at) = t
            .std_modules
            .iter()
            .map(|&(name, m)| (name, t.modules[m.0 as usize].span))
            .filter(|(_, at)| at.file == FileId(0) && at.lo <= span.lo)
            .max_by_key(|(_, at)| at.lo)?;
        let row = crate::stdmods::by_name(p.name(name))?;
        let line = map.line_display_col(span).0.checked_sub(map.line_col(at).0)?;
        // The splice text is fixed for the process (built in, or read once
        // per session from `LUST_STD_SRC`), so the path is the key.
        (line, markers((row.rel.to_string(), 0, 0), || crate::stdload::source_text(row.rel).ok())?)
    } else {
        // The std platform crate: its own files under library/.
        if !matches!(
            p.shipped_crate_at(span),
            Some(crate::ty::lang::ShippedCrate::Std | crate::ty::lang::ShippedCrate::Core | crate::ty::lang::ShippedCrate::Alloc)
        ) {
            return None;
        }
        let path = std::path::Path::new(map.path(span.file));
        let rel = path.strip_prefix(crate::resolve::adapters::library_root()).ok()?.to_str()?.to_string();
        // Keyed by the text's address and length: a reloaded file is a new text.
        let text = map.text(span.file)?;
        (map.line_display_col(span).0, markers((rel, text.as_ptr() as usize, text.len()), || Some(text.to_string()))?)
    };
    match table.get(&line)? {
        Marker::At(f, l, c) => Some((*f, *l, *c)),
        Marker::Never | Marker::Gap => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `STD_FILES` and the runtime's `lust_std_files` list the same paths
    /// in the same order: a packed location's index means one file.
    #[test]
    fn std_files_match_the_runtime_table() {
        let c = include_str!("runtime/panic.c");
        let start = c.find("lust_std_files[] = {").expect("lust_std_files in panic.c");
        let body = &c[start..start + c[start..].find("};").unwrap()];
        let paths: Vec<&str> = body.lines().filter_map(|l| l.trim().strip_prefix("LUST_STD_SRC \"")?.strip_suffix("\",")).collect();
        assert_eq!(paths, STD_FILES);
        assert!(STD_FILES.len() <= 0x40, "index 0x40 is the pow bit");
    }

    /// Every marker in the shipped std source parses, and every explicit
    /// panic site (`panic!`, `assert!`, `unreachable!`, `expect`, `unwrap`)
    /// outside a `#[track_caller]` fn has a marker on its line: without
    /// one, its panic would name a line of lust's text under the user's
    /// file name.
    #[test]
    fn std_panic_sites_have_markers() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("library");
        let mut bad = Vec::new();
        for dir in ["core/src", "alloc/src", "std/src"] {
            let mut files: Vec<_> = std::fs::read_dir(root.join(dir)).unwrap().map(|e| e.unwrap().path()).collect();
            files.sort();
            for path in files.into_iter().filter(|p| p.extension().is_some_and(|e| e == "rs")) {
                let text = std::fs::read_to_string(&path).unwrap();
                let rel = path.strip_prefix(&root).unwrap().display().to_string();
                // The lang enum methods splice as one block, not a hidden
                // module, so `site` reads no marker there: every panic in
                // them is in a `#[track_caller]` fn or unreachable.
                if rel == "core/src/option.rs" || rel == "core/src/result.rs" {
                    continue;
                }
                bad.extend(unmarked_sites(&text).into_iter().map(|(n, l)| format!("library/{rel}:{n}: {l}")));
            }
        }
        assert!(bad.is_empty(), "std panic sites without a `//@` marker (see src/stdloc.rs):\n{}", bad.join("\n"));
    }

    /// The explicit panic sites of `text` outside `#[track_caller]` fns
    /// whose line has no valid marker, as (line, text). A lexical scan: a
    /// fn's attribute is the `#[track_caller]` line right above it.
    fn unmarked_sites(text: &str) -> Vec<(usize, String)> {
        const SITES: &[&str] = &["panic!(", "assert!(", "assert_eq!(", "assert_ne!(", "unreachable!(", ".expect(", ".unwrap()"];
        let mut out = Vec::new();
        let (mut tc, mut pending) = (false, false);
        for (i, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            if code.starts_with("#[track_caller]") {
                pending = true;
            } else if !code.starts_with("#[") && (code.contains("fn ") && (code.starts_with("fn ") || code.contains(" fn "))) {
                tc = pending;
                pending = false;
            }
            let code = code.split("//").next().unwrap_or("");
            if tc || !SITES.iter().any(|s| code.contains(s)) {
                continue;
            }
            match parse_marker(line) {
                Some(Ok(_)) => {}
                _ => out.push((i + 1, line.trim().to_string())),
            }
        }
        out
    }

    #[test]
    fn markers_parse() {
        assert_eq!(parse_marker("x //@ core/src/time.rs:12:5"), Some(Ok(Marker::At(17, 12, 5))));
        assert_eq!(parse_marker("x //@ never"), Some(Ok(Marker::Never)));
        assert!(parse_marker("x //@ nowhere.rs:1:1").unwrap().is_err());
        assert_eq!(parse_marker("x // plain"), None);
    }
}
