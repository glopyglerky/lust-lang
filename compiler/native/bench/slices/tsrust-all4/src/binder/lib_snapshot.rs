//! Rust-only: the lib bind snapshot (perf9 round 2, `r2-lib-bind-snapshot`).
//!
//! PERF: binding lib.dom is most of the query bind window (about 10 ms on
//! one bind thread, 2.5 to 3 times Go). The bind output of the large bundled
//! lib files is embedded in `lib_bind.bin`, and `bind_source_file_detached`
//! loads it instead of binding when the key matches. The loaded output
//! equals a live bind of the file into a new arena: the same symbols in the
//! same order, the tables with their entries in insertion order, the same
//! ids, node data, flow nodes and file data (bind diagnostics included).
//! The file then joins the program arena like any other bound file.
//!
//! The key of a lib file (`SnapshotKey`) is two hashes of its text (xxh3,
//! and `const_hash`, which the embed build has from compile time), its
//! slot count, an xxh3 hash of its kinds, parser flags and names columns
//! and of the file facts that the binder reads, and a compile-time hash of
//! the binder sources (`SOURCES_HASH`). Any mismatch or load error binds the
//! file live, so a stale blob costs time, not output.
//!
//! Node references and flow ids are stored without the file index (the low
//! half of the handle), and the load adds the program file index. Symbol
//! and table ids are ids of the file arena. A name with a stable id
//! (`Name::stable_id`: the lib names of `core/lib_names.rs`) is stored by
//! id; any other name is stored as text and interned at load.
//!
//! After a change to the binder, `core.rs`, the parser or a bundled lib,
//! write the blob again (the integrator does this after each merge round,
//! as for PGO training):
//!
//! ```text
//! cargo test -p ts_goport --lib binder::lib_snapshot::tests::generate_lib_bind_snapshot -- --ignored --exact
//! ```
//!
//! `snapshot_matches_live_bind` fails while the blob is stale.
//! `GOPORT_LIB_SNAPSHOT=0` turns the snapshot off (for A/B timing), and
//! `GOPORT_LIB_SNAPSHOT=trace` writes one line per lib file to stderr.
//!
//! Blob layout, all integers little-endian: `MAGIC`, the lib count (u32),
//! then per lib its base name (`lib.dom.d.ts`) and its section, each as a
//! u32 length and the bytes. A section is the key (`SnapshotKey::write`),
//! the name, symbol, table and flow node counts (u32 each), the names, then
//! the body that `encode` writes.

use crate::frontend::bundled;
use crate::prelude::*;
use std::sync::OnceLock;
use crate::xxhash_rust::xxh3::{Xxh3, xxh3_64};


/// The first bytes of `BLOB`.
const MAGIC: &[u8; 8] = b"TSLIBBND";

/// Bundled lib files with at least this many text bytes get a snapshot
/// (see `bundled::SNAPSHOT_TEXT_MIN`).
pub const MIN_TEXT_LEN: usize = bundled::SNAPSHOT_TEXT_MIN;

/// Set in a stored name entry (and in a names column entry of the parse
/// hash) when text follows. The bits below it hold the text length. A
/// clear bit means the entry is a stable name id, which is always lower.
pub(crate) const TEXT_BIT: u32 = 1 << 31;

// Symbol mask bits: which optional fields follow.
const S_CHECK_FLAGS: u8 = 1;
const S_MEMBERS: u8 = 2;
const S_EXPORTS: u8 = 4;
const S_PARENT: u8 = 8;
const S_EXPORT_SYMBOL: u8 = 16;
/// `value_declaration` follows.
const S_VALUE_DECLARATION: u8 = 32;
/// `value_declaration` is the first declaration and is not stored.
const S_VALUE_IS_FIRST: u8 = 64;
/// Exactly one declaration follows, with no count.
const S_ONE_DECLARATION: u8 = 128;

// `NodeBindData` mask bits: which fields follow (the others are nil).
const N_SYMBOL: u8 = 1;
const N_LOCAL_SYMBOL: u8 = 2;
const N_LOCALS: u8 = 4;
const N_NEXT_CONTAINER: u8 = 8;
const N_FLOW_NODE: u8 = 16;
const N_END_FLOW_NODE: u8 = 32;
const N_RETURN_FLOW_NODE: u8 = 64;
const N_ADDED_FLAGS: u8 = 128;

// `FlowNode` mask bits.
const F_NODE: u8 = 1;
const F_ANTECEDENT: u8 = 2;
const F_ANTECEDENTS: u8 = 4;

// `Diagnostic` bool bits.
const D_REPORTS_UNNECESSARY: u8 = 1;
const D_REPORTS_DEPRECATED: u8 = 2;
const D_SKIPPED_ON_NO_EMIT: u8 = 4;

// ──────────────────────────────────────────────────────────────────────
// Key
// ──────────────────────────────────────────────────────────────────────


// One item per file, so each compile-time evaluation stays short. The
// source text is only read at compile time and is not in the binary.

/// `SOURCES_HASH` and the text part of the keys (`SnapshotKey::text_const`)
/// use the compile-time hash of `frontend/bundled.rs`. The lib parse
/// snapshot (`frontend/parser/lib_parse_snapshot.rs`) uses it too.
pub(crate) use crate::frontend::bundled::{mix};

/// What must match for a lib file to load its section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SnapshotKey {
    /// `SOURCES_HASH` of the build that wrote the section.
    sources: u64,
    /// The slot count (`GoFile::parser_flags` length).
    nodes: u32,
    /// xxh3 of the file text.
    text: u64,
    /// `const_hash` of the file text (`text_matches`).
    text_const: u64,
    /// `parse_hash` of the file.
    parse: u64,
}

impl SnapshotKey {



}






// ──────────────────────────────────────────────────────────────────────
// Load
// ──────────────────────────────────────────────────────────────────────

/// A snapshot switch (`GOPORT_LIB_SNAPSHOT`, and
/// `GOPORT_LIB_PARSE_SNAPSHOT` for the lib parse snapshot).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Off,
    On,
    /// On, with one stderr line per lib file.
    Trace,
}

impl Mode {
    /// Environment variable `var`: "0" turns the snapshot off, "trace"
    /// traces it.
    pub(crate) fn from_env(var: &str) -> Mode {
        match std::env::var(var).as_deref() {
            Ok("0") => Mode::Off,
            Ok("trace") => Mode::Trace,
            _ => Mode::On,
        }
    }
}

/// `GOPORT_LIB_SNAPSHOT`: "0" turns the snapshot off, "trace" traces it.
fn mode() -> Mode {
    static MODE: OnceLock<Mode> = OnceLock::new();
    *MODE.get_or_init(|| Mode::from_env("GOPORT_LIB_SNAPSHOT"))
}

/// One lib file in a snapshot blob.
pub(crate) struct SnapshotEntry {
    /// The base name, for example `lib.dom.d.ts`.
    pub(crate) name: &'static str,
    pub(crate) section: &'static [u8],
}





/// A cursor over blob bytes. Every read returns `None` past the end.
pub(crate) struct SnapshotReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> SnapshotReader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        SnapshotReader { data, pos: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }


    pub(crate) fn u8(&mut self) -> Option<u8> {
        let value = *self.data.get(self.pos)?;
        self.pos += 1;
        Some(value)
    }





}

/// Reads a section body into a new arena. Each id and reference is checked
/// against its count, so a bad section fails here, not in a later read.
struct Decoder<'a> {
    r: SnapshotReader<'a>,
    /// `file index << 32`: the high half of each node and flow id.
    base: u64,
    nodes: u32,
    symbols: u32,
    tables: u32,
    flows: u32,
    names: Vec<Name>,
}

impl Decoder<'_> {










}


// ──────────────────────────────────────────────────────────────────────
// Dump (the generator and its test)
// ──────────────────────────────────────────────────────────────────────

/// Writes a section body. `names` numbers each name at its first use.
struct Encoder {
    file: usize,
    body: Vec<u8>,
    names: FxHashMap<Name, u32>,
    name_list: Vec<Name>,
}

impl Encoder {
    fn u8(&mut self, value: u8) {
        self.body.push(value);
    }











}



/// The bundled libs that get a snapshot, largest first: every lib with at
/// least `MIN_TEXT_LEN` text bytes. The lib parse snapshot uses the same
/// list.
#[cfg(test)]
pub(crate) fn snapshot_libs() -> Vec<&'static str> {
    let mut libs: Vec<(&'static str, usize)> = bundled::LIB_NAMES
        .iter()
        .filter_map(|&name| {
            let text = lib_text(name)?;
            (text.len() >= MIN_TEXT_LEN).then_some((name, text.len()))
        })
        .collect();
    libs.sort_by_key(|&(_, len)| std::cmp::Reverse(len));
    libs.into_iter().map(|(name, _)| name).collect()
}

/// The text of bundled lib `name`: the embedded text, or in a `noembed`
/// build the file in `bundled::lib_path()` (read and leaked on each call).
/// The lib snapshot tests use it, so they run on both builds.
#[cfg(test)]
pub(crate) fn lib_text(name: &str) -> Option<&'static str> {
    let path = format!("{}/{name}", bundled::lib_path());
    bundled::bundled_text(&path).or_else(|| {
        let text = std::fs::read_to_string(&path).ok()?;
        Some(Box::leak(text.into_boxed_str()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How to write the blob again.
    const REGENERATE: &str = "run `cargo test -p ts_goport --lib \
        binder::lib_snapshot::tests::generate_lib_bind_snapshot -- --ignored --exact` \
        (through scripts/run-cargo-capped.sh)";

    /// Loads a program version that holds every snapshot lib and runs `f`
    /// on each lib file, in `snapshot_libs` order, with the program current.
    fn for_each_lib_file(label: &str, mut f: impl FnMut(&'static str, Node)) {
        let libs = snapshot_libs();
        assert!(libs.contains(&"lib.dom.d.ts"), "no snapshot for lib.dom");
        let options: Vec<String> = libs
            .iter()
            .map(|name| {
                let option = name
                    .strip_prefix("lib.")
                    .and_then(|rest| rest.strip_suffix(".d.ts"))
                    .unwrap_or_else(|| panic!("no lib option for {name}"));
                format!("\"{option}\"")
            })
            .collect();
        let dir = std::env::temp_dir().join(format!(
            "ts_goport_lib_snapshot_{label}_{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ts"), "export {};\n").unwrap();
        std::fs::write(
            dir.join("tsconfig.json"),
            format!(
                r#"{{ "compilerOptions": {{ "lib": [{}], "types": [] }}, "files": ["a.ts"] }}"#,
                options.join(", ")
            ),
        )
        .unwrap();
        let config = dir.join("tsconfig.json");
        let program = crate::program::try_load_version(&config.to_string_lossy(), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {}: {e}", config.display()));
        let _ = std::fs::remove_dir_all(&dir);
        let _scope = crate::core::enter_program(Some(program));
        for lib in libs {
            let file = program
                .source_files()
                .find(|file| bundled::bundled_lib_name(&file.info.file_name) == Some(lib))
                .unwrap_or_else(|| panic!("{lib} is not in the program"))
                .root;
            f(lib, file);
        }
    }

    /// A live bind of `file` into a new arena. It fails when the bind
    /// changes state of this thread that a snapshot load would not change.
    fn live_bind(file: Node) -> Result<(BoundFile, SymbolArena), String> {
        let before = (synthetic_slot_count(), next_ids());
        let mut arena = SymbolArena::new_file();
        let bound = bind_source_file_live(file, &mut arena);
        if (synthetic_slot_count(), next_ids()) != before {
            return Err("the live bind made synthetic nodes or ids".into());
        }
        Ok((bound, arena))
    }

    /// Writes `lib_bind.bin` from a live bind of each snapshot lib, and
    /// prints the sizes and times (times mean something only in an
    /// optimized test build).
    #[test]
    #[ignore = "writes src/binder/lib_bind.bin"]
    fn generate_lib_bind_snapshot() {
        let mut sections = Vec::new();
        for_each_lib_file("generate", |lib, file| {
            // The process-wide unported counts: run this test alone.
            let unported = unported_report();
            let start = std::time::Instant::now();
            let live = live_bind(file).unwrap_or_else(|e| panic!("{lib}: {e}"));
            let bind_time = start.elapsed();
            assert_eq!(
                unported_report(),
                unported,
                "{lib}: the live bind hit unported code, which a load would skip"
            );
            let key = SnapshotKey::of(file).unwrap_or_else(|| panic!("{lib}: no key"));
            let section =
                encode(file, &key, &live.0, &live.1).unwrap_or_else(|e| panic!("{lib}: {e}"));
            let start = std::time::Instant::now();
            let loaded = load_section(file, &section)
                .unwrap_or_else(|part| panic!("{lib}: the new section does not load ({part})"));
            let load_time = start.elapsed();
            assert_same_bind(lib, &live, &loaded);
            eprintln!(
                "{lib}: text {} bytes, {} slots, section {} bytes, bind {bind_time:?}, load {load_time:?}",
                source_file_text(file).len(),
                key.nodes,
                section.len()
            );
            sections.push((lib, section));
        });
        let blob = write_blob(MAGIC, &sections);
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/binder/lib_bind.bin");
        std::fs::write(path, &blob).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
        eprintln!("wrote {path}: {} bytes", blob.len());
    }

    /// The embedded blob holds a section for each snapshot lib that equals
    /// a live bind of it now, and its load gives that live bind.
    #[test]
    fn snapshot_matches_live_bind() {
        for_each_lib_file("compare", |lib, file| {
            let live = live_bind(file).unwrap_or_else(|e| panic!("{lib}: {e}"));
            let key = SnapshotKey::of(file).unwrap_or_else(|| panic!("{lib}: no key"));
            let section =
                encode(file, &key, &live.0, &live.1).unwrap_or_else(|e| panic!("{lib}: {e}"));
            let entry = entries()
                .iter()
                .find(|entry| entry.name == lib)
                .unwrap_or_else(|| panic!("lib_bind.bin has no {lib}: {REGENERATE}"));
            if entry.section != &section[..] {
                let stored = SnapshotKey::read(&mut SnapshotReader::new(entry.section));
                panic!(
                    "lib_bind.bin is stale for {lib} (stored key {stored:?}, key now {key:?}): \
                     {REGENERATE}"
                );
            }
            let loaded = load_section(file, entry.section)
                .unwrap_or_else(|part| panic!("{lib}: the section does not load ({part})"));
            assert_same_bind(lib, &live, &loaded);
        });
    }

    /// Asserts that `loaded` equals `live`, field by field, and that the
    /// loaded tables find each entry.
    fn assert_same_bind(
        lib: &str,
        live: &(BoundFile, SymbolArena),
        loaded: &(BoundFile, SymbolArena),
    ) {
        let (live_file, live_arena) = live;
        let (file, arena) = loaded;
        assert_eq!(
            arena.symbol_count(),
            live_arena.symbol_count(),
            "{lib}: symbol count"
        );
        for index in 1..arena.symbol_count() {
            let id = SymbolId(index as u32);
            let (a, b) = (arena.sym(id), live_arena.sym(id));
            assert!(
                a.flags == b.flags
                    && a.check_flags == b.check_flags
                    && a.name == b.name
                    && *a.declarations == *b.declarations
                    && a.value_declaration == b.value_declaration
                    && a.members == b.members
                    && a.exports == b.exports
                    && a.parent == b.parent
                    && a.export_symbol == b.export_symbol,
                "{lib}: symbol {index}: {a:?} != {b:?}"
            );
        }
        assert_eq!(
            arena.table_count(),
            live_arena.table_count(),
            "{lib}: table count"
        );
        for index in 1..arena.table_count() {
            let table = SymbolTable(index as u32);
            let entries = live_arena.entries(table);
            assert_eq!(arena.entries(table), entries, "{lib}: table {index}");
            for (name, symbol) in &entries {
                assert_eq!(arena.get_name(table, name), *symbol, "{lib}: {name:?}");
                assert_eq!(arena.get(table, name), *symbol, "{lib}: {name:?}");
            }
        }
        assert_eq!(
            arena.private_names().collect::<Vec<_>>(),
            live_arena.private_names().collect::<Vec<_>>(),
            "{lib}: private names"
        );
        assert!(
            file.node_bind.compact().parts() == live_file.node_bind.compact().parts(),
            "{lib}: node data"
        );
        assert_eq!(
            file.flow_nodes.len(),
            live_file.flow_nodes.len(),
            "{lib}: flow node count"
        );
        for (index, (a, b)) in file
            .flow_nodes
            .iter()
            .zip(&live_file.flow_nodes)
            .enumerate()
        {
            assert!(
                a.flags == b.flags
                    && a.node == b.node
                    && a.antecedent == b.antecedent
                    && a.antecedents == b.antecedents,
                "{lib}: flow node {index}: {a:?} != {b:?}"
            );
        }
        assert_eq!(
            format!("{:?}", file.file_bind),
            format!("{:?}", live_file.file_bind),
            "{lib}: file data"
        );
    }
}
