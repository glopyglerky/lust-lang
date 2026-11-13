//! Rust-only: the lib parse snapshot (perf9 round 3, R3-1).
//!
//! PERF: at 16 cores the parse of lib.dom is the parse critical path of
//! query (about 50 M cycles). The parse output of the large bundled lib
//! files is embedded in `lib_parse.bin`. `parse_source_file` and
//! `parse_source_file_detached` load it into the new store of the file
//! instead of parsing, when the key matches, and freeze the store as a
//! parse does. So adoption, publish and `files_parser.rs` do not change.
//! The loaded store equals the store of a live parse: the same slots, with
//! the same headers, node data, names and R2-5 links, and the same tables
//! that the U1 and U4 build entries and the freeze make from them. The
//! `ParsedSourceFile` fields and the two `DetachedParse` fields are stored
//! too.
//!
//! The key of a lib file (`ParseKey`) is a compile-time hash of the
//! parser, scanner, factory, store and node sources, astdata and this file
//! (`SOURCES_HASH`), a hash of the parse options that the parse reads (file
//! name, script kind, module indicator options) and two hashes of the text
//! (xxh3, and `const_hash`, which the embed build has from compile time;
//! see `lib_snapshot::text_matches`). Any mismatch or load error parses the file live, so a stale blob
//! costs time, not output. A lib file is found by its base name
//! (`bundled_lib_name`), so the embed and noembed builds share the blob.
//!
//! The load leaves the state of the thread as a live parse does: the names
//! of the file are interned in the order in which the parse interns them
//! (first use, in slot order), and it makes no synthetic node and no node
//! or symbol id (a lib parse makes none either; the tests check this).
//!
//! After a change to the parser, the scanner, the factory, the store,
//! astdata or a bundled lib, write the blob again (the integrator does this
//! after each merge round, before the lib bind generator):
//!
//! ```text
//! cargo test -p ts_goport --lib frontend::parser::lib_parse_snapshot::tests::generate_lib_parse_snapshot -- --ignored --exact
//! ```
//!
//! `snapshot_matches_live_parse` fails while the blob is stale.
//! `GOPORT_LIB_PARSE_SNAPSHOT=0` turns the snapshot off (for A/B timing),
//! and `GOPORT_LIB_PARSE_SNAPSHOT=trace` writes one line per lib file to
//! stderr.
//!
//! Blob layout: `MAGIC`, then the sections as `lib_snapshot::write_blob`
//! writes them. A section is the key (four u64, little-endian), one byte
//! for `read_module_indicator_options`, then LEB128 varints ("zz" is the
//! zigzag form of a signed value):
//! - the slot count (slots after slot 0);
//! - the (kind, flags) palette: a count, then each kind and flags;
//! - the names: a count, then each as `lib_snapshot` stores a name (a u32
//!   stable id, or `TEXT_BIT` | length and the text);
//! - each slot `i`: its palette index, zz(pos - pos of slot i - 1),
//!   zz(end - pos), zz(parent - i), the two links (`link`), the name index
//!   for an Identifier or PrivateIdentifier, and the data: a variant tag
//!   (`SHARED_NAME` for the shared name node) and the fields in astdata
//!   order (`payload_codec!`). A child id is zz(id - i); a list range is
//!   zz(start - pos) and zz(end - start);
//! - the `ParsedSourceFile` fields in declaration order, with node handles
//!   as slot indexes (0 is nil), and the import specifiers.

use crate::ast::store::{LibParseSlot, };
#[cfg(test)]
use crate::ast::store::{lib_parse_slot_views, lib_parse_store_dump};
use crate::astdata::NodeData;
use crate::binder::lib_snapshot::{
    MIN_TEXT_LEN, Mode, SnapshotEntry, SnapshotReader, TEXT_BIT, mix, 
    
};
use crate::frontend::prelude::*;
use std::sync::OnceLock;


/// The first bytes of `BLOB`.
const MAGIC: &[u8; 8] = b"TSLIBPRS";

/// The variant tag of a slot that points at the shared name node of its
/// kind (S1). The `payload_codec!` tags are below it.
const SHARED_NAME: u8 = u8::MAX;

// ──────────────────────────────────────────────────────────────────────
// Key
// ──────────────────────────────────────────────────────────────────────


// One item per file, so each compile-time evaluation stays short. The
// source text is only read at compile time and is not in the binary.

/// What must match for a lib file to load its section.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ParseKey {
    /// `SOURCES_HASH` of the build that wrote the section.
    sources: u64,
    /// `options_hash` of the parse.
    options: u64,
    /// xxh3 of the file text.
    text: u64,
    /// `const_hash` of the file text (`text_matches`).
    text_const: u64,
}

impl ParseKey {



}


// ──────────────────────────────────────────────────────────────────────
// Load
// ──────────────────────────────────────────────────────────────────────

/// `GOPORT_LIB_PARSE_SNAPSHOT`: "0" turns the snapshot off, "trace" traces
/// it.
fn mode() -> Mode {
    static MODE: OnceLock<Mode> = OnceLock::new();
    *MODE.get_or_init(|| Mode::from_env("GOPORT_LIB_PARSE_SNAPSHOT"))
}


#[cfg(test)]
thread_local! {
    /// Set while a test parses live (`tests::live_parse`).
    static LIVE_ONLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The snapshot loads on this thread.
    static LOADS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}





/// The signed value of zigzag form `value`.
#[inline]
fn unzigzag(value: u64) -> i64 {
    (value >> 1) as i64 ^ -((value & 1) as i64)
}

/// The zigzag form of `value`: 0, -1, 1, -2, ... become 0, 1, 2, 3, ...
#[cfg(test)]
fn zigzag(value: i64) -> u64 {
    ((value << 1) ^ (value >> 63)) as u64
}

/// Reads a section body. Each id, slot and count is checked, so a bad
/// section fails here, not in a later read.
struct Decoder<'a> {
    r: SnapshotReader<'a>,
    /// The store id, the high half of each node handle.
    store: usize,
    /// The slot count of the store, slot 0 included.
    total: u32,
    /// The slot being read; child ids are relative to it.
    index: u32,
    /// `loc.pos()` of the slot being read (of the one before, until its
    /// position is read); list ranges are relative to it.
    pos: i64,
}

impl<'a> Decoder<'a> {
    #[inline]
    fn var(&mut self) -> Option<u64> {
        let mut value = 0u64;
        let mut shift = 0;
        loop {
            let byte = self.r.u8()?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte < 0x80 {
                return Some(value);
            }
            shift += 7;
            if shift >= 64 {
                return None;
            }
        }
    }

    #[inline]
    fn zz(&mut self) -> Option<i64> {
        Some(unzigzag(self.var()?))
    }


    fn i32(&mut self) -> Option<i32> {
        i32::try_from(self.zz()?).ok()
    }





    // The `payload_codec!` field readers.






    fn osym(&mut self) -> Option<Option<crate::astdata::SymbolId>> {
        match self.var()? {
            0 => Some(None),
            value => Some(Some(crate::astdata::SymbolId(
                u32::try_from(value - 1).ok()?,
            ))),
        }
    }

    fn oflow(&mut self) -> Option<Option<crate::astdata::FlowNodeId>> {
        match self.var()? {
            0 => Some(None),
            value => Some(Some(crate::astdata::FlowNodeId(
                u32::try_from(value - 1).ok()?,
            ))),
        }
    }

    fn table(&mut self) -> Option<crate::astdata::SymbolTable> {
        Some(crate::astdata::SymbolTable)
    }

    fn opaque(&mut self) -> Option<crate::astdata::OpaqueValue> {
        Some(crate::astdata::OpaqueValue)
    }

    fn flag(&mut self) -> Option<bool> {
        match self.r.u8()? {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        }
    }

    fn num(&mut self) -> Option<u32> {
        u32::try_from(self.var()?).ok()
    }



    fn tflags(&mut self) -> Option<crate::astdata::TokenFlags> {
        Some(crate::astdata::TokenFlags(self.num()?))
    }

    fn kind(&mut self) -> Option<SyntaxKind> {
        SyntaxKind::try_from(u16::try_from(self.var()?).ok()?).ok()
    }

    fn okind(&mut self) -> Option<Option<SyntaxKind>> {
        Some(if self.flag()? {
            Some(self.kind()?)
        } else {
            None
        })
    }




    // The `ParsedSourceFile` readers.



    fn range(&mut self) -> Option<TextRange> {
        let pos = self.i32()?;
        Some(TextRange::new(pos, self.i32()?))
    }



    fn tristate(&mut self) -> Option<Tristate> {
        match self.r.u8()? {
            0 => Some(Tristate::Unknown),
            1 => Some(Tristate::False),
            2 => Some(Tristate::True),
            _ => None,
        }
    }





    fn check_js_directive(&mut self) -> Option<Option<CheckJsDirective>> {
        if !self.flag()? {
            return Some(None);
        }
        Some(Some(CheckJsDirective {
            enabled: self.flag()?,
            range: self.range()?,
        }))
    }

    fn usize(&mut self) -> Option<usize> {
        usize::try_from(self.var()?).ok()
    }

}

// ──────────────────────────────────────────────────────────────────────
// Node data
// ──────────────────────────────────────────────────────────────────────

/// Makes `decode_payload` and (tests) `encode_payload`, which read and
/// write the node data of one slot: the variant tag, then each field with
/// the `Decoder` (`Encoder`) method named by its codec. The table lists
/// every `NodeData` variant and every field of its astdata struct, in
/// astdata order, so a new variant or field in astdata does not compile here (the
/// match must be exhaustive, the struct pattern and literal must name every
/// field). `src/astdata/ast_generated.rs` is in `SOURCES_HASH`.
macro_rules! payload_codec {
    ($($variant_tag:literal $variant:ident($data:ident) {
        $($field:ident: $codec:ident),* $(,)?
    })*) => {
        /// The node data with variant tag `variant_tag`, from `dec`.
        fn decode_payload(dec: &mut Decoder<'_>, variant_tag: u8) -> Option<NodeData> {
            Some(match variant_tag {
                $($variant_tag => NodeData::$variant(Box::new(crate::astdata::$data {
                    $($field: dec.$codec()?,)*
                })),)*
                _ => return None,
            })
        }

        /// Writes the variant tag and the fields of `payload`.
        #[cfg(test)]
        fn encode_payload(enc: &mut Encoder, payload: &NodeData) -> Result<(), String> {
            match payload {
                $(NodeData::$variant(payload) => {
                    let crate::astdata::$data { $($field),* } = &**payload;
                    enc.u8($variant_tag);
                    $(enc.$codec($field)?;)*
                })*
            }
            Ok(())
        }
    };
}

// The rows come from `src/astdata/ast_generated.rs` (one per
// `NodeData` variant, in enum order; the codec of each field follows from
// its type). The tags stay below `SHARED_NAME`.

// ──────────────────────────────────────────────────────────────────────
// Dump (the generator and its test)
// ──────────────────────────────────────────────────────────────────────

/// Writes a section. The mirror of `Decoder`.
#[cfg(test)]
struct Encoder {
    out: Vec<u8>,
    /// The store id of the parse.
    store: usize,
    /// The slot being written (see `Decoder::index`).
    index: u32,
    /// See `Decoder::pos`.
    pos: i64,
}

#[cfg(test)]
impl Encoder {
    fn u8(&mut self, value: u8) {
        self.out.push(value);
    }

    fn var(&mut self, mut value: u64) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.out.push(byte);
                return;
            }
            self.out.push(byte | 0x80);
        }
    }

    fn zz(&mut self, value: i64) {
        self.var(zigzag(value));
    }

    fn count(&mut self, len: usize) {
        self.var(len as u64);
    }

    /// `slot` relative to the slot being written.
    fn rel(&self, slot: u32) -> i64 {
        i64::from(slot) - i64::from(self.index)
    }

    fn link(&mut self, value: u32) {
        match value {
            LibParseSlot::LINK_NONE => self.var(0),
            LibParseSlot::LINK_END => self.var(1),
            slot => self.var(2 + zigzag(self.rel(slot))),
        }
    }

    fn slot_of(id: &crate::astdata::NodeId) -> Result<u32, String> {
        u32::try_from(id.index()).map_err(|_| format!("node id {id:?} too large"))
    }

    // The `payload_codec!` field writers.

    fn id(&mut self, id: &crate::astdata::NodeId) -> Result<(), String> {
        let slot = Self::slot_of(id)?;
        self.zz(self.rel(slot));
        Ok(())
    }

    fn oid(&mut self, id: &Option<crate::astdata::NodeId>) -> Result<(), String> {
        match id {
            None => self.var(0),
            Some(id) => {
                let slot = Self::slot_of(id)?;
                self.var(zigzag(self.rel(slot)) + 1);
            }
        }
        Ok(())
    }

    fn list(&mut self, l: &crate::astdata::NodeList) -> Result<(), String> {
        let start = i64::from(l.range.start.get());
        let end = i64::from(l.range.end.get());
        self.zz(start - self.pos);
        self.zz(end - start);
        self.var(((l.nodes.len() as u64) << 1) | u64::from(l.has_trailing_comma));
        for id in &l.nodes {
            self.id(id)?;
        }
        Ok(())
    }

    fn olist(&mut self, l: &Option<crate::astdata::NodeList>) -> Result<(), String> {
        match l {
            None => self.u8(0),
            Some(l) => {
                self.u8(1);
                self.list(l)?;
            }
        }
        Ok(())
    }

    fn mods(&mut self, m: &Option<crate::astdata::ModifierList>) -> Result<(), String> {
        match m {
            None => self.u8(0),
            Some(m) => {
                self.u8(1);
                self.list(&m.list)?;
                self.var(u64::from(m.flags.0));
            }
        }
        Ok(())
    }

    fn osym(&mut self, symbol: &Option<crate::astdata::SymbolId>) -> Result<(), String> {
        self.var(symbol.map_or(0, |symbol| u64::from(symbol.0) + 1));
        Ok(())
    }

    fn oflow(&mut self, flow: &Option<crate::astdata::FlowNodeId>) -> Result<(), String> {
        self.var(flow.map_or(0, |flow| u64::from(flow.0) + 1));
        Ok(())
    }

    fn table(&mut self, _: &crate::astdata::SymbolTable) -> Result<(), String> {
        Ok(())
    }

    fn opaque(&mut self, _: &crate::astdata::OpaqueValue) -> Result<(), String> {
        Ok(())
    }

    fn flag(&mut self, value: &bool) -> Result<(), String> {
        self.u8(u8::from(*value));
        Ok(())
    }

    fn num(&mut self, value: &u32) -> Result<(), String> {
        self.var(u64::from(*value));
        Ok(())
    }

    fn str(&mut self, text: &str) {
        self.count(text.len());
        self.out.extend_from_slice(text.as_bytes());
    }

    fn string(&mut self, text: &String) -> Result<(), String> {
        self.str(text);
        Ok(())
    }

    fn tflags(&mut self, flags: &crate::astdata::TokenFlags) -> Result<(), String> {
        self.var(u64::from(flags.0));
        Ok(())
    }

    fn kind(&mut self, kind: &SyntaxKind) -> Result<(), String> {
        self.var(*kind as u64);
        Ok(())
    }

    fn okind(&mut self, kind: &Option<SyntaxKind>) -> Result<(), String> {
        match kind {
            None => self.u8(0),
            Some(kind) => {
                self.u8(1);
                self.kind(kind)?;
            }
        }
        Ok(())
    }

    fn strings(&mut self, list: &Vec<String>) -> Result<(), String> {
        self.count(list.len());
        for text in list {
            self.str(text);
        }
        Ok(())
    }

    fn ids(&mut self, list: &Vec<crate::astdata::NodeId>) -> Result<(), String> {
        self.count(list.len());
        for id in list {
            self.id(id)?;
        }
        Ok(())
    }

    fn oids(&mut self, list: &Option<Vec<crate::astdata::NodeId>>) -> Result<(), String> {
        match list {
            None => self.u8(0),
            Some(list) => {
                self.u8(1);
                self.ids(list)?;
            }
        }
        Ok(())
    }

    // The `ParsedSourceFile` writers.

    fn node(&mut self, n: Node) -> Result<(), String> {
        if n.is_nil() {
            self.var(0);
            return Ok(());
        }
        if n.file_index() != self.store {
            return Err(format!("{n:?} is in another store"));
        }
        self.var((n.0 & 0xffff_ffff) - 1);
        Ok(())
    }

    fn nodes(&mut self, list: &[Node]) -> Result<(), String> {
        self.count(list.len());
        for &n in list {
            self.node(n)?;
        }
        Ok(())
    }

    fn range(&mut self, range: TextRange) {
        self.zz(i64::from(range.pos()));
        self.zz(i64::from(range.end()));
    }

    fn diagnostics(&mut self, list: &[Diagnostic]) -> Result<(), String> {
        self.count(list.len());
        for diagnostic in list {
            if diagnostic.repopulate_info.is_some() {
                return Err("a parse diagnostic with repopulate info".into());
            }
            self.node(diagnostic.file)?;
            self.zz(i64::from(diagnostic.pos));
            self.zz(i64::from(diagnostic.end));
            self.zz(i64::from(diagnostic.code));
            self.u8(diagnostic.category.0 as u8);
            let key = diagnostic.message.key();
            if crate::diagnostics::message_by_key(key) != Some(diagnostic.message) {
                return Err(format!("message key {key} does not find its message"));
            }
            self.str(key);
            self.strings(&diagnostic.message_args)?;
            self.diagnostics(&diagnostic.message_chain)?;
            self.diagnostics(&diagnostic.related_information)?;
            self.u8(u8::from(diagnostic.reports_unnecessary)
                | (u8::from(diagnostic.reports_deprecated) << 1)
                | (u8::from(diagnostic.skipped_on_no_emit) << 2));
        }
        Ok(())
    }

    /// The `ParsedSourceFile` fields that `Decoder::parsed_file` reads. The
    /// pattern names every field, so a new field does not compile here.
    fn parsed_file(&mut self, file: &ParsedSourceFile) -> Result<(), String> {
        let ParsedSourceFile {
            store,
            root,
            parse_options: _,
            text: _,
            end_of_file_token,
            hash: _,
            diagnostics,
            js_diagnostics,
            jsdoc_diagnostics,
            language_variant,
            script_kind,
            is_declaration_file,
            uses_uri_style_node_core_modules,
            identifier_count,
            imports,
            module_augmentations,
            ambient_module_names,
            comment_directives,
            jsdoc_cache,
            has_lazy_js_doc,
            reparsed_clones,
            pragmas,
            referenced_files,
            type_reference_directives,
            lib_reference_directives,
            check_js_directive,
            node_count,
            text_count,
            common_js_module_indicator,
            external_module_indicator,
            version: _,
        } = file;
        if *store != self.store {
            return Err("the file is in another store".into());
        }
        self.node(*root)?;
        self.node(*end_of_file_token)?;
        self.diagnostics(diagnostics)?;
        self.diagnostics(js_diagnostics)?;
        self.diagnostics(jsdoc_diagnostics)?;
        self.zz(i64::from(language_variant.0));
        self.zz(i64::from(script_kind.0));
        self.u8(u8::from(*is_declaration_file));
        self.u8(*uses_uri_style_node_core_modules as u8);
        self.zz(i64::from(*identifier_count));
        self.nodes(imports)?;
        self.nodes(module_augmentations)?;
        self.strings(ambient_module_names)?;
        self.count(comment_directives.len());
        for directive in comment_directives {
            self.range(directive.loc);
            self.zz(i64::from(directive.kind.0));
        }
        // By slot, so the blob does not depend on the map order. The load
        // inserts in this order; the cache is only read by key.
        let mut cache: Vec<(&Node, &Vec<Node>)> = jsdoc_cache.iter().collect();
        cache.sort_by_key(|(node, _)| **node);
        self.count(cache.len());
        for (node, jsdocs) in cache {
            self.node(*node)?;
            self.nodes(jsdocs)?;
        }
        self.u8(u8::from(*has_lazy_js_doc));
        self.nodes(reparsed_clones)?;
        self.count(pragmas.len());
        for pragma in pragmas {
            self.str(&pragma.name);
            self.count(pragma.args.len());
            for (key, arg) in &pragma.args {
                self.str(key);
                self.str(&arg.name);
                self.str(&arg.value);
                self.range(arg.range);
            }
            self.range(pragma.range);
            self.kind(&pragma.kind)?;
        }
        for references in [
            referenced_files,
            type_reference_directives,
            lib_reference_directives,
        ] {
            self.count(references.len());
            for reference in references {
                self.range(reference.range);
                self.str(&reference.file_name);
                self.zz(i64::from(reference.resolution_mode.0));
                self.u8(u8::from(reference.preserve));
            }
        }
        match check_js_directive {
            None => self.u8(0),
            Some(directive) => {
                self.u8(1);
                self.u8(u8::from(directive.enabled));
                self.range(directive.range);
            }
        }
        self.count(*node_count);
        self.count(*text_count);
        self.node(*common_js_module_indicator)?;
        self.node(*external_module_indicator)?;
        Ok(())
    }
}

/// The section of a finished parse of `text` with `opts` and
/// `script_kind`: `file` and its store (a store of this thread that is not
/// published), and the two `DetachedParse` fields. `load_section` reads it
/// back.
#[cfg(test)]
fn encode_section(
    opts: &SourceFileParseOptions,
    text: &str,
    script_kind: ScriptKind,
    file: &ParsedSourceFile,
    read_module_indicator_options: bool,
    import_specifiers: &[String],
) -> Result<Vec<u8>, String> {
    let views = lib_parse_slot_views(file.store)?;
    let mut body = Encoder {
        out: Vec::new(),
        store: file.store,
        index: 0,
        pos: 0,
    };
    let mut palette: Vec<(SyntaxKind, NodeFlags)> = Vec::new();
    let mut palette_index: FxHashMap<(SyntaxKind, NodeFlags), u64> = FxHashMap::default();
    let mut names: Vec<Name> = Vec::new();
    let mut name_index: FxHashMap<Name, u64> = FxHashMap::default();
    for (i, view) in views.iter().enumerate() {
        body.index = u32::try_from(i + 1).map_err(|_| "too many slots".to_string())?;
        let pair = *palette_index
            .entry((view.kind, view.flags))
            .or_insert_with(|| {
                palette.push((view.kind, view.flags));
                palette.len() as u64 - 1
            });
        body.var(pair);
        let pos = i64::from(view.loc.pos());
        let end = i64::from(view.loc.end());
        body.zz(pos - body.pos);
        body.zz(end - pos);
        body.pos = pos;
        body.zz(body.rel(view.parent));
        body.link(view.first_child);
        body.link(view.next_sibling);
        if matches!(
            view.kind,
            SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier
        ) {
            let keyword = get_identifier_token(view.name.as_str()) != SyntaxKind::Identifier;
            if keyword != view.text_is_keyword {
                return Err(format!("slot {}: the keyword bit differs", i + 1));
            }
            let index = *name_index.entry(view.name.clone()).or_insert_with(|| {
                names.push(view.name.clone());
                names.len() as u64 - 1
            });
            body.var(index);
        } else if view.name != Name::default() || view.text_is_keyword {
            return Err(format!("slot {}: a name on a {:?} slot", i + 1, view.kind));
        }
        if view.shared_name {
            body.u8(SHARED_NAME);
        } else {
            encode_payload(&mut body, view.data)?;
        }
    }
    let mut out = Vec::with_capacity(body.out.len() + 16 * names.len() + 1024);
    ParseKey::of(opts, text, script_kind).write(&mut out);
    out.push(u8::from(read_module_indicator_options));
    let mut e = Encoder {
        out,
        store: file.store,
        index: 0,
        pos: 0,
    };
    e.count(views.len());
    e.count(palette.len());
    for (kind, flags) in &palette {
        e.kind(kind)?;
        e.num(&flags.0)?;
    }
    e.count(names.len());
    for name in &names {
        match name.stable_id() {
            Some(id) if id & TEXT_BIT == 0 => e.out.extend_from_slice(&id.to_le_bytes()),
            _ => {
                let text = name.as_str();
                let len = u32::try_from(text.len())
                    .ok()
                    .filter(|&len| len & TEXT_BIT == 0)
                    .ok_or_else(|| format!("name of {} bytes", text.len()))?;
                e.out.extend_from_slice(&(TEXT_BIT | len).to_le_bytes());
                e.out.extend_from_slice(text.as_bytes());
            }
        }
    }
    e.out.extend_from_slice(&body.out);
    e.parsed_file(file)?;
    e.count(import_specifiers.len());
    for text in import_specifiers {
        e.str(text);
    }
    Ok(e.out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binder::lib_snapshot::{lib_text, snapshot_libs, write_blob};
    use crate::frontend::parser::utilities::{
        module_indicator_options_read, reset_module_indicator_options_read,
    };

    /// How to write the blob again.
    const REGENERATE: &str = "run `cargo test -p ts_goport --lib \
        frontend::parser::lib_parse_snapshot::tests::generate_lib_parse_snapshot -- --ignored --exact` \
        (through scripts/run-cargo-capped.sh)";

    /// Snapshot libs (`snapshot_libs`) that get no section. No perf9 project
    /// uses lib.webworker (the default libs take lib.dom), so its section
    /// would only add binary size.
    const SKIPPED_LIBS: &[&str] = &["lib.webworker.d.ts"];

    /// The libs that get a section, largest first.
    fn parse_snapshot_libs() -> Vec<&'static str> {
        snapshot_libs()
            .into_iter()
            .filter(|lib| !SKIPPED_LIBS.contains(lib))
            .collect()
    }

    /// The parse options and the text of bundled lib `lib`, as the program
    /// loader gives them (the path is not read by the parse).
    fn lib_input(lib: &str) -> (SourceFileParseOptions, &'static str) {
        let file_name = format!("{}/{lib}", crate::frontend::bundled::lib_path());
        let text = lib_text(lib).unwrap_or_else(|| panic!("no bundled {lib}"));
        let opts = SourceFileParseOptions {
            file_name,
            ..Default::default()
        };
        (opts, text)
    }

    /// The state of this thread that a lib parse must not change.
    fn thread_state() -> (usize, (u64, u64)) {
        (synthetic_slot_count(), next_ids())
    }

    /// A parse and the two `DetachedParse` fields.
    struct Parse {
        file: ParsedSourceFile,
        read_module_indicator_options: bool,
        import_specifiers: Vec<String>,
    }

    /// A live serial parse (no snapshot). It fails when the parse changes
    /// state of this thread that a load would not change.
    fn live_parse(opts: &SourceFileParseOptions, text: &'static str) -> Result<Parse, String> {
        let before = thread_state();
        reset_module_indicator_options_read();
        LIVE_ONLY.set(true);
        let file = parse_source_file(opts, text, ScriptKind::TS);
        LIVE_ONLY.set(false);
        let read_module_indicator_options = module_indicator_options_read();
        if thread_state() != before {
            return Err("the live parse made synthetic nodes or ids".into());
        }
        let import_specifiers = file.imports.iter().map(|n| n.text().to_string()).collect();
        Ok(Parse {
            file,
            read_module_indicator_options,
            import_specifiers,
        })
    }

    /// The section of `parse`.
    fn section(opts: &SourceFileParseOptions, text: &str, parse: &Parse) -> Vec<u8> {
        encode_section(
            opts,
            text,
            ScriptKind::TS,
            &parse.file,
            parse.read_module_indicator_options,
            &parse.import_specifiers,
        )
        .unwrap_or_else(|e| panic!("{}: {e}", opts.file_name))
    }

    /// Asserts that `loaded` equals the live parse whose section is
    /// `expected`: the same section (every slot and every file field) and
    /// the same store tables.
    fn assert_same_parse(
        lib: &str,
        opts: &SourceFileParseOptions,
        text: &'static str,
        live: &Parse,
        expected: &[u8],
        loaded: &Parse,
    ) {
        assert_eq!(loaded.file.parse_options, live.file.parse_options, "{lib}");
        assert!(
            std::ptr::eq(
                loaded.file.text.as_static().expect("a lib text is static"),
                live.file.text.as_static().expect("a lib text is static")
            ),
            "{lib}: text"
        );
        assert!(
            section(opts, text, loaded) == expected,
            "{lib}: the loaded parse differs from the live parse"
        );
        let a = lib_parse_store_dump(live.file.store);
        let b = lib_parse_store_dump(loaded.file.store);
        if let Some(i) = (0..a.len().max(b.len())).find(|&i| a.get(i) != b.get(i)) {
            panic!(
                "{lib}: store line {i} differs:\n  live:   {:?}\n  loaded: {:?}",
                a.get(i),
                b.get(i)
            );
        }
    }

    /// Writes `lib_parse.bin` from a live parse of each lib, and prints
    /// the sizes and times (times mean something only in an optimized test
    /// build).
    #[test]
    #[ignore = "writes src/frontend/parser/lib_parse.bin"]
    fn generate_lib_parse_snapshot() {
        let mut sections = Vec::new();
        for lib in parse_snapshot_libs() {
            let (opts, text) = lib_input(lib);
            // The process-wide unported counts: run this test alone.
            let unported = unported_report();
            let start = std::time::Instant::now();
            let live = live_parse(&opts, text).unwrap_or_else(|e| panic!("{lib}: {e}"));
            let parse_time = start.elapsed();
            assert_eq!(
                unported_report(),
                unported,
                "{lib}: the live parse hit unported code, which a load would skip"
            );
            let section = section(&opts, text, &live);
            // Load it into a new store, as `parse_source_file` does.
            let before = thread_state();
            let file_name: &'static str = Box::leak(opts.file_name.clone().into_boxed_str());
            let store = new_file_store(file_name, text);
            let start = std::time::Instant::now();
            let loaded = load_section(store, &section, &opts, text, ScriptKind::TS)
                .unwrap_or_else(|part| panic!("{lib}: the new section does not load ({part})"));
            let load_time = start.elapsed();
            assert_eq!(
                thread_state(),
                before,
                "{lib}: the load made synthetic nodes or ids"
            );
            let loaded = Parse {
                file: loaded.file,
                read_module_indicator_options: false,
                import_specifiers: loaded.import_specifiers,
            };
            assert_same_parse(lib, &opts, text, &live, &section, &loaded);
            eprintln!(
                "{lib}: text {} bytes, {} slots, section {} bytes, parse {parse_time:?}, load {load_time:?}",
                text.len(),
                file_store_slot_count(live.file.store),
                section.len()
            );
            sections.push((lib, section));
        }
        let blob = write_blob(MAGIC, &sections);
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/frontend/parser/lib_parse.bin"
        );
        std::fs::write(path, &blob).unwrap_or_else(|e| panic!("cannot write {path}: {e}"));
        eprintln!("wrote {path}: {} bytes", blob.len());
    }

    /// The embedded blob holds a section for each lib that equals a live
    /// parse of it now, and `parse_source_file` and
    /// `parse_source_file_detached` load it into a store that equals the
    /// live one, with no change to the state of the thread.
    #[test]
    fn snapshot_matches_live_parse() {
        for lib in parse_snapshot_libs() {
            let (opts, text) = lib_input(lib);
            let live = live_parse(&opts, text).unwrap_or_else(|e| panic!("{lib}: {e}"));
            let expected = section(&opts, text, &live);
            let entry = entries()
                .iter()
                .find(|entry| entry.name == lib)
                .unwrap_or_else(|| panic!("lib_parse.bin has no {lib}: {REGENERATE}"));
            if entry.section != &expected[..] {
                let stored = ParseKey::read(&mut SnapshotReader::new(entry.section));
                let now = ParseKey::of(&opts, text, ScriptKind::TS);
                panic!(
                    "lib_parse.bin is stale for {lib} (stored key {stored:?}, key now {now:?}): \
                     {REGENERATE}"
                );
            }

            let before = thread_state();
            let loads = LOADS.get();
            let serial = parse_source_file(&opts, text, ScriptKind::TS);
            assert_eq!(
                LOADS.get(),
                loads + 1,
                "{lib}: not loaded from the snapshot"
            );
            assert_eq!(
                thread_state(),
                before,
                "{lib}: the load changed the thread state"
            );
            assert_eq!(
                parsed_source_file_diagnostics(serial.root).len(),
                serial.diagnostics.len(),
                "{lib}: set_source_file_diagnostics"
            );
            let serial = Parse {
                file: serial,
                read_module_indicator_options: false,
                import_specifiers: live.import_specifiers.clone(),
            };
            assert_same_parse(lib, &opts, text, &live, &expected, &serial);

            // A parse worker loads it too, and the loader adopts the store.
            let worker_opts = opts.clone();
            let (detached, worker_loads) = std::thread::spawn(move || {
                let parse = parse_source_file_detached(1, &worker_opts, text, ScriptKind::TS);
                (parse, LOADS.get())
            })
            .join()
            .unwrap();
            assert_eq!(
                worker_loads, 1,
                "{lib}: the worker did not load the snapshot"
            );
            assert!(detached.store.is_self_contained(), "{lib}");
            assert_eq!(
                detached.read_module_indicator_options, live.read_module_indicator_options,
                "{lib}"
            );
            assert_eq!(detached.import_specifiers, live.import_specifiers, "{lib}");
            let import_specifiers = detached.import_specifiers.clone();
            let adopted = Parse {
                file: adopt_detached_parse(detached, &opts),
                read_module_indicator_options: false,
                import_specifiers,
            };
            assert_same_parse(lib, &opts, text, &live, &expected, &adopted);
        }
    }

    /// A section that does not decode leaves the store empty, so the file
    /// is parsed live into the same store.
    #[test]
    fn bad_section_parses_live() {
        let lib = parse_snapshot_libs()[0];
        let (opts, text) = lib_input(lib);
        let live = live_parse(&opts, text).unwrap_or_else(|e| panic!("{lib}: {e}"));
        let mut section = section(&opts, text, &live);
        // Cut the file fields: the slots load, then the decode fails.
        section.truncate(section.len() - 8);
        let file_name: &'static str = Box::leak(opts.file_name.clone().into_boxed_str());
        let store = new_file_store(file_name, text);
        assert_eq!(
            load_section(store, &section, &opts, text, ScriptKind::TS).err(),
            Some("load")
        );
        assert_eq!(file_store_slot_count(store), 1, "the store is not empty");
        assert!(!is_file_store_frozen(store));
    }
}
