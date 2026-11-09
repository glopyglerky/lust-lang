//! Synthetic AST nodes: the nodes Go creates with `ast.NodeFactory`.
//!
//! Go factory nodes are ordinary `*ast.Node` values. Here a synthetic node is
//! a `Node` handle whose file index is `SYNTHETIC_NODE_FILE`. Its low 32 bits
//! index a thread-local slot list (like `SYNTHETIC_FLOW_FILE` for flow nodes).
//!
//! Each node slot names a `crate::astdata::Node` (kind and `NodeData`) that the
//! thread owns, so the accessors in `node.rs` and `fields.rs` that match on
//! `NodeData` work on synthetic nodes too. Child ids inside that data are ids
//! in the synthetic id space:
//! - a synthetic child uses its own slot index;
//! - a parsed child (Go shares the pointer) uses an alias slot. `Node::new`
//!   resolves an alias slot to the parsed node, so identity is kept:
//!   `synthetic.name() == parsed_name` is true, like Go pointer equality;
//! - Go `nil` in a field that astdata stores as a required `NodeId` uses slot 0,
//!   which resolves to `Node::NIL`.
//!
//! Go mutates factory nodes after creation (`node.Parent = p`, `node.Loc = l`,
//! `node.Flags |= f`, `node.FlowNodeData().FlowNode = f`,
//! `node.AsX().Symbol = s`). Those fields live in the slot, not in the
//! `crate::astdata::Node`, and `node.rs` reads them through the hooks below.
//!
//! Ownership: the slots, the node data, the factory lists and the node
//! slices that reads make belong to the thread's arena. Nothing hands out a
//! reference into them that outlives a read. Data is read inside a closure
//! (`with_ast_data`, or the `with_data!` macro in hot code),
//! and a list field through a selector (`list_of!`, `modifiers_of!`). A list
//! of synthetic data is a `SyntheticList` handle (an index), not a pointer,
//! and a text is interned (`synthetic_text`). Only a parsed node of a
//! static parse gives `&'static` data (`static_ast_node`). A node that a
//! freeable parse owns (lsshells M3c, `ast/store.rs` `OwnedAst`) is read the
//! same way as a synthetic node: in a scope (`with_scoped_ast_node`,
//! `read_scoped_ast_node`), with lists as handles (`ast::StoreList`, whose
//! selector has a small id, `SelectorSite`). A checker worker of a released
//! program frees all of it (`free_synthetic_nodes`, called by
//! `program::release_program`). A one-program process leaks it at the end
//! instead (`forget_synthetic_nodes`), like the checker itself.
//!
//! Owners: the language server makes the nodes of every program version on
//! its dispatch thread. There each program version owns the entries made
//! while it is current (`open_synthetic_owner`), and its release frees them
//! (`free_synthetic_owner`, called by `ls_program`). Other entries belong to
//! the thread (the base owner). The tables are in chunks, and each chunk
//! has one owner, so a handle stays an index and keeps Go pointer identity
//! while its owner lives. A freed chunk leaves a hole that is never used
//! again: a read of a freed entry panics, and a handle never names another
//! node. The rules:
//! - a new node, a factory list and a list copy go to the current program
//!   version when it is an owner on this thread and no base scope
//!   (`enter_base_synthetic_owner`) is open, else to the base owner;
//! - a data write (`replace_node_data`, `set_node_kind`) goes to the owner
//!   of the node, so a base node never points into a program version;
//! - an alias slot goes to the file version of its parsed node when that is
//!   a freeable version (`ast/file_version.rs`), else to the base owner. An
//!   alias lives as long as the node it names: after the version dies, the
//!   next new alias or file scope frees the entries of that version
//!   (`free_dead_aliases`). A read of such an entry panics, as a read of
//!   the parsed node does;
//! - in a file scope (`enter_file_synthetic_owner`), a new node, factory
//!   list or list copy goes to the file version of a parsed node in the
//!   same way. Lazy JSDoc opens one: its cache keeps the JSDoc nodes of a
//!   parsed node for as long as that node lives;
//! - node slices always go to the base owner.
//!
//! Code whose nodes a cache keeps across program versions (the token cache,
//! parses) opens a base scope.
//!
//! d.ts twins: a checker thread and the thread that prints its d.ts files
//! (`program::send_dts_twin_job`) take new chunk numbers from one counter
//! per table (`share_synthetic_chunks`), so a chunk number names a chunk of
//! one of the two threads only. The twin starts with no entries
//! (`install_twin_synthetic_arena`) and gets a copy of the checker entries
//! that each tree it prints reaches, at their indexes (`print_pack`). A
//! read of an entry that no pack copied panics (`Slot::Absent`).
//!
//! PORT: the Go GC frees request garbage at once and checker nodes with
//! their checker. Here they stay until their program version is released.
//!
//! PORT: parsed nodes are immutable. The setters panic on a parsed node,
//! except on a node of a ported-parser file that the parser has not finished
//! (`ast/store.rs`). Go code that writes a field of a parsed node needs its
//! own port decision.

use crate::astdata::NodeData;
use crate::prelude::*;
use std::cell::{Cell, OnceCell};
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};

mod print_pack;
pub use print_pack::{};

/// File index of synthetic nodes. `SYNTHETIC_FLOW_FILE` is `0xffff_ffff`.
pub const SYNTHETIC_NODE_FILE: usize = 0xffff_fffe;

/// Slot 0: Go `nil` stored in a astdata field that has no `Option`.
const NIL_SLOT: u32 = 0;



/// The cached `Node::subtree_facts` of a factory node: `COMPUTED` is set
/// once they are cached. A copy (`Clone`) has none cached: each thread
/// computes the facts of its nodes, as with the per-thread map of parsed
/// nodes (`ast::SUBTREE_FACTS`), so a node that a seed or a print pack
/// copies gets the facts of its data on the new thread.
#[derive(Default)]
struct SlotFacts(SubtreeFacts);

impl Clone for SlotFacts {
    fn clone(&self) -> Self {
        Self::default()
    }
}


/// A list that the factory made (`new_synthetic_node_list`,
/// `new_synthetic_modifier_list`) or that a read copied
/// (`copy_synthetic_list`).
#[derive(Clone)]
enum OwnList {
    Nodes(crate::astdata::NodeList),
    Modifiers(crate::astdata::ModifierList),
}

impl OwnList {
    fn get(&self) -> AnyList<'_> {
        match self {
            Self::Nodes(l) => AnyList::Nodes(l),
            Self::Modifiers(m) => AnyList::Modifiers(m),
        }
    }
}

/// Bytes per chunk of `SyntheticArena::slots`, `datas` and `lists`.
// PERF: 14 KiB is jemalloc's largest small size class, and small classes
// pack densely in slabs. A larger chunk is a large allocation, which starts
// at a random cache line offset and so touches one page more (10% for a
// 40 KiB chunk).
const CHUNK_BYTES: usize = 14 * 1024;



/// Lists per chunk of `SyntheticArena::lists`.
const LIST_CHUNK: usize = CHUNK_BYTES / size_of::<OwnList>();

/// A chunk of `SyntheticArena::datas`: cells that fill in order.
type DataChunk = Rc<[OnceCell<crate::astdata::Node>]>;

/// The panic of a read of an entry whose owner was freed.
const FREED: &str = "synthetic entry of a released program or file version is read";

/// The panic of a read on a d.ts twin of a checker entry that no print pack
/// copied (`Slot::Absent`).
const ABSENT: &str = "a d.ts twin reads a synthetic node that its print pack does not hold";

/// The tables of `SyntheticArena` that have chunk numbers, as indexes of
/// `SharedChunks::next`.
const SLOT_TABLE: usize = 0;
const DATA_TABLE: usize = 1;
const LIST_TABLE: usize = 2;

/// The next chunk number of each table (`SLOT_TABLE`, `DATA_TABLE`,
/// `LIST_TABLE`), shared by a checker thread and its d.ts twin
/// (`share_synthetic_chunks`).
pub struct SharedChunks {
    next: [AtomicU32; 3],
}

/// The owner of an arena chunk (see "Owners" in the module comment).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OwnerKey {
    /// The thread. Its chunks go only with the whole arena.
    Base,
    /// A language server program version (`GoProgram::id`).
    Program(u32),
    /// The alias slots of the nodes of a freeable file version (its file
    /// id) and the entries made in its file scopes, in chunks of their own.
    File(u32),
}

/// The chunk numbers of one owner, oldest first. New entries of the owner
/// fill the last chunk of each table until it is full.
#[derive(Default)]
struct OwnerChunks {
    slots: Vec<u32>,
    datas: Vec<u32>,
    /// The number of filled cells in the last chunk of `datas`.
    data_fill: u32,
    lists: Vec<u32>,
}

impl OwnerChunks {
    const EMPTY: Self = Self {
        slots: Vec::new(),
        datas: Vec::new(),
        data_fill: 0,
        lists: Vec::new(),
    };
}



/// The number of a new chunk of `table`, which has `len` chunks with
/// `per_chunk` entries each: `len`, or the next number of the shared
/// counter (`SyntheticArena::shared`). Entry ids are `u32` and are not used
/// again, so a thread that makes about 4 billion entries of one kind runs
/// out of them.
fn new_chunk_number(
    shared: &Option<Arc<SharedChunks>>,
    table: usize,
    len: usize,
    per_chunk: usize,
) -> u32 {
    let c = match shared {
        None => len,
        Some(shared) => shared.next[table].fetch_add(1, Ordering::Relaxed) as usize,
    };
    assert!(
        (c + 1) * per_chunk < u32::MAX as usize,
        "synthetic entry ids exhausted"
    );
    c as u32
}

/// The chunks of `owner`, which must be open (a file owner opens with
/// `SyntheticArena::alias_owner`), from the owner fields of an arena, so the
/// caller can still borrow its tables.
fn owner_chunks<'a>(
    base: &'a mut OwnerChunks,
    owners: &'a mut FxHashMap<u32, OwnerChunks>,
    alias_files: &'a mut FxHashMap<u32, OwnerChunks>,
    owner: OwnerKey,
) -> &'a mut OwnerChunks {
    match owner {
        OwnerKey::Base => base,
        OwnerKey::Program(id) => owners.get_mut(&id).expect("synthetic owner is not open"),
        OwnerKey::File(file) => alias_files
            .get_mut(&file)
            .expect("synthetic owner is not open"),
    }
}






static EMPTY_BIND: NodeBindData = NodeBindData {
    symbol: SymbolId::NIL,
    local_symbol: SymbolId::NIL,
    locals: SymbolTable::NIL,
    next_container: Node::NIL,
    flow_node: FlowNodeId::NIL,
    end_flow_node: FlowNodeId::NIL,
    return_flow_node: FlowNodeId::NIL,
    added_flags: NodeFlags::NONE,
};


/// False when `GOPORT_SYNTHETIC_OWNERS=0`: then no program version becomes
/// an owner, and every entry belongs to its thread, as before owners (for
/// A/B runs, and as a fallback).
fn owners_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| !matches!(std::env::var("GOPORT_SYNTHETIC_OWNERS").as_deref(), Ok("0")))
}






/// From `enter_base_synthetic_owner` or `enter_file_synthetic_owner`. It
/// is `!Send`, so it drops on the thread that made it.
pub struct SyntheticOwnerScope {
    /// The owner of the scope that was open around this one.
    outer: Option<OwnerKey>,
    _not_send: std::marker::PhantomData<*const ()>,
}











/// The handle of slot `index`. Does not resolve aliases.
const fn handle(index: u32) -> Node {
    Node(((SYNTHETIC_NODE_FILE as u64) << 32) | (index as u64 + 1))
}

/// Slot index of a synthetic handle.
fn slot_index(n: Node) -> usize {
    ((n.0 & 0xffff_ffff) - 1) as usize
}

/// True when `n` is a node that the factory created.
#[must_use]
pub fn is_synthetic_node(n: Node) -> bool {
    n.is_some() && n.file_index() == SYNTHETIC_NODE_FILE
}




// ──────────────────────────────────────────────────────────────────────
// Node data reads
// ──────────────────────────────────────────────────────────────────────

/// The astdata node of a synthetic node, held apart from the arena by its
/// chunk (see `SyntheticArena::datas`).
struct HeldNode {
    chunk: DataChunk,
    cell: usize,
}

impl std::ops::Deref for HeldNode {
    type Target = crate::astdata::Node;

    #[inline]
    fn deref(&self) -> &crate::astdata::Node {
        self.chunk[self.cell]
            .get()
            .expect("synthetic node data entry is not filled")
    }
}




/// The data of a parsed node that a caller loads once for several field
/// reads (`parsed_node_data`, the `_in` accessors of `node.rs`): `Some` for
/// a node with `'static` data, `None` for a node that a freeable parse owns
/// (lsshells M3c). With `None` an `_in` accessor reads the node again, as
/// its plain accessor does (most of them read a store column).
pub type LoadedData = Option<&'static NodeData>;












/// wasm: `read_ast_node_miss` for a node that is not of the hot file
/// version, with the miss code once (`read_ast_node_cold_dyn`): `read` runs
/// through `&mut dyn FnMut`. One copy per reader type was 119 KB of the
/// module (101 copies).
#[cfg(target_family = "wasm")]
#[cold]
#[inline(never)]
fn read_ast_node_cold<R>(n: Node, read: impl FnOnce(&NodeData) -> R) -> R {
    let mut read = Some(read);
    let mut result = None;
    read_ast_node_cold_dyn(n, &mut |node| {
        result = Some((read.take().expect("the node is read once"))(node));
    });
    result.expect("the node is read once")
}

/// wasm: `read_ast_node_cold` with one copy for all readers.
#[cfg(target_family = "wasm")]
#[cold]
#[inline(never)]
fn read_ast_node_cold_dyn(n: Node, read: &mut dyn FnMut(&NodeData)) {
    if n.file_index() == SYNTHETIC_NODE_FILE {
        return read(&synthetic_ast_node(n).data);
    }
    crate::ast::store::read_store_node_miss(n, read)
}

/// `$body` with `$d` bound to the astdata data (`&NodeData`) of node `$n`,
/// parsed or synthetic, like `with_ast_data`. Go dereferences the pointer, so
/// nil panics. `$body` cannot keep a reference into the data, and it cannot
/// `return` from the caller. It is a field read: it must not make or change
/// nodes of the store of `$n` (`read_scoped_ast_node`).
// PERF: the macro writes `$body` twice: inline for a node of a static tier
// (the `&'static` read, with no closure call) and in a closure that runs
// out of line for any other node (`read_ast_node_miss`): a synthetic node,
// an unpublished store node, or a node that a freeable parse owns (lsshells
// M3c). That closure holds the synthetic node data apart
// from the arena, so `$body` can make synthetic nodes. A closure with two
// call sites is not inlined, which cost about 1% of instructions in
// multiprog. To write `$body` once instead, make this
// `with_ast_data($n, |$d| $body)`; no call site changes.
macro_rules! with_data {
    ($n:expr, |$d:ident| $body:expr) => {{
        let n__: $crate::core::Node = $n;
        match $crate::ast::synthetic::static_tier_ast_node(n__) {
            Some(data__) => {
                let $d: &crate::astdata::NodeData = data__;
                $body
            }
            None => $crate::ast::synthetic::read_ast_node_miss(n__, |data__| {
                let $d: &crate::astdata::NodeData = data__;
                $body
            }),
        }
    }};
}

/// The `NodeList` that the selector `|$d| $body` (a `ListSel` body) finds in
/// the data of node `$n` (nil for a Go `nil` field), or `None` when the kind
/// of `$n` has no such field. Like `with_data!`, the selector runs inline for
/// a node of a static tier; any other node (a synthetic node, an unpublished
/// store node, a node that a freeable parse owns) gets it as a `ListSel`,
/// with the id of this site (`SelectorSite`).
macro_rules! list_of {
    ($n:expr, |$d:ident| $body:expr) => {{
        let n__: $crate::core::Node = $n;
        match $crate::ast::synthetic::static_tier_ast_node(n__) {
            Some(data__) => {
                let $d: &'static crate::astdata::NodeData = data__;
                let found: Option<Option<$crate::ast::synthetic::AnyList<'static>>> = $body;
                found.map(|l| {
                    $crate::ast::NodeList::from_ts(
                        n__.file_index(),
                        l.map($crate::ast::synthetic::AnyList::nodes),
                    )
                })
            }
            None => {
                static SITE: $crate::ast::synthetic::SelectorSite =
                    $crate::ast::synthetic::SelectorSite::new();
                $crate::ast::synthetic::scoped_node_list_of(n__, &SITE, |$d| $body)
            }
        }
    }};
}

/// `list_of!` for a modifier list field.
macro_rules! modifiers_of {
    ($n:expr, |$d:ident| $body:expr) => {{
        let n__: $crate::core::Node = $n;
        match $crate::ast::synthetic::static_tier_ast_node(n__) {
            Some(data__) => {
                let $d: &'static crate::astdata::NodeData = data__;
                let found: Option<Option<$crate::ast::synthetic::AnyList<'static>>> = $body;
                found.map(|m| {
                    $crate::ast::ModifierList::from_ts(
                        n__.file_index(),
                        m.map($crate::ast::synthetic::AnyList::modifiers),
                    )
                })
            }
            None => {
                static SITE: $crate::ast::synthetic::SelectorSite =
                    $crate::ast::synthetic::SelectorSite::new();
                $crate::ast::synthetic::scoped_modifiers_of(n__, &SITE, |$d| $body)
            }
        }
    }};
}

/// A list or modifier list read from node data.
#[derive(Clone, Copy)]
pub enum AnyList<'a> {
    Nodes(&'a crate::astdata::NodeList),
    Modifiers(&'a crate::astdata::ModifierList),
}

impl<'a> AnyList<'a> {
    /// The node list (for a modifier list, its `NodeList`).
    #[must_use]
    pub fn nodes(self) -> &'a crate::astdata::NodeList {
        match self {
            Self::Nodes(l) => l,
            Self::Modifiers(m) => &m.list,
        }
    }

    /// The modifier list. Panics on a node list.
    #[must_use]
    pub fn modifiers(self) -> &'a crate::astdata::ModifierList {
        match self {
            Self::Modifiers(m) => m,
            Self::Nodes(_) => panic!("a NodeList is not a ModifierList"),
        }
    }

    /// The address of the node list, for Go pointer equality.
    fn ptr(self) -> *const () {
        std::ptr::from_ref(self.nodes()).cast()
    }
}

/// Finds a list field in node data. `None`: the data has no such field (a
/// kind without it). `Some(None)`: the field is Go `nil`.
pub type ListSel = for<'a> fn(&'a NodeData) -> Option<Option<AnyList<'a>>>;

/// The most list selector sites (`SelectorSite`) in the process.
const SELECTOR_LIMIT: usize = 1024;

/// The selectors that `SelectorSite::id` gave ids, by id.
static SELECTORS: [OnceLock<ListSel>; SELECTOR_LIMIT] = [const { OnceLock::new() }; SELECTOR_LIMIT];

/// The number of ids that `SelectorSite::id` gave.
static SELECTOR_COUNT: AtomicUsize = AtomicUsize::new(0);

/// The list selector of one `list_of!` or `modifiers_of!` site, with a
/// small process-wide id (lsshells M3c): a list handle of a store that owns
/// its nodes (`ast::StoreList`) names its selector with 4 bytes, so
/// `NodeList` and `ModifierList` stay 16 bytes. Each site gets its id at its
/// first scoped read.
// PORT: Go keeps a `*NodeList` pointer. A list of a freeable file version
// cannot be borrowed past its read, so the handle names the node data cell
// and the selector, and the list is found again at each use.
pub struct SelectorSite(AtomicU32);

impl SelectorSite {
    #[must_use]
    pub const fn new() -> Self {
        Self(AtomicU32::new(0))
    }

    /// The id of `sel`, the selector of this site (`list_selector`).
    #[inline]
    pub fn id(&self, sel: ListSel) -> u32 {
        match self.0.load(Ordering::Acquire) {
            0 => self.new_id(sel),
            id => id - 1,
        }
    }

    /// The first `id` of this site.
    #[cold]
    #[inline(never)]
    fn new_id(&self, sel: ListSel) -> u32 {
        let id = SELECTOR_COUNT.fetch_add(1, Ordering::Relaxed);
        // The last id is `ast::store::PENDING_SEL`.
        assert!(
            id < SELECTOR_LIMIT - 1,
            "more than {} list selector sites",
            SELECTOR_LIMIT - 1
        );
        // A new id: this is its only write.
        let _ = SELECTORS[id].set(sel);
        let id = id as u32;
        // Two threads can give one site an id at once. The first id stays;
        // the other one is not used again.
        match self
            .0
            .compare_exchange(0, id + 1, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => id,
            Err(first) => first - 1,
        }
    }
}

impl Default for SelectorSite {
    fn default() -> Self {
        Self::new()
    }
}

/// The list selector with id `id` (`SelectorSite::id`).
#[inline]
#[must_use]
pub fn list_selector(id: u32) -> ListSel {
    *SELECTORS[id as usize]
        .get()
        .expect("a list selector id has its selector")
}














// ──────────────────────────────────────────────────────────────────────
// Lists of synthetic data
// ──────────────────────────────────────────────────────────────────────

/// A list of synthetic data: the `NodeList`, `ModifierList` and `NodeSlice`
/// form of a list that this thread's arena owns. It is an index, so it is
/// only valid on the thread that made it (like a synthetic `Node`).
#[derive(Clone, Copy)]
pub enum SyntheticList {
    /// The list field that `sel` finds in node data `data`
    /// (`SyntheticNode::data`).
    Field { data: u32, sel: ListSel },
    /// A factory list.
    Own { index: u32 },
    /// A node slice that a read made (`NodeSlice` only).
    Slice { index: u32 },
}

impl std::fmt::Debug for SyntheticList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Field { data, .. } => write!(f, "Field({data})"),
            Self::Own { index } => write!(f, "Own({index})"),
            Self::Slice { index } => write!(f, "Slice({index})"),
        }
    }
}











// ──────────────────────────────────────────────────────────────────────
// Go field writes on factory nodes
// ──────────────────────────────────────────────────────────────────────








/// A astdata node for synthetic data. Only kind and data are read; the
/// header lives in the slot.
fn new_ts_node(kind: SyntaxKind, data: NodeData) -> crate::astdata::Node {
    crate::astdata::Node {
        kind,
        flags: crate::astdata::NodeFlags(0),
        range: undefined_ts_range(),
        parent: None,
        data,
    }
}







// ──────────────────────────────────────────────────────────────────────
// Allocation (used by factory.rs)
// ──────────────────────────────────────────────────────────────────────



// ──────────────────────────────────────────────────────────────────────
// Factory SourceFile fields
// ──────────────────────────────────────────────────────────────────────








// Go: ast/ast.go:2805 (node *SourceFile) copyFrom



/// Go `core.UndefinedTextRange()` in astdata form. `TextPos` is `u32`; the
/// `as i32` in `node.rs` `text_range_of` turns `u32::MAX` back into `-1`.
fn undefined_ts_range() -> crate::astdata::text::TextRange {
    ts_range(TextRange::undefined())
}

/// A Go `core.TextRange` in astdata form (`-1` is stored as `u32::MAX`).
fn ts_range(loc: TextRange) -> crate::astdata::text::TextRange {
    crate::astdata::text::TextRange {
        start: crate::astdata::text::TextPos::new(loc.pos() as u32),
        end: crate::astdata::text::TextPos::new(loc.end() as u32),
    }
}







#[cfg(test)]
mod tests {
    use super::*;

    // The test runs on its own thread, so `free_synthetic_nodes` frees only
    // the nodes it made.
    #[test]
    fn synthetic_lists_keep_go_pointer_identity_until_freed() {
        std::thread::spawn(|| {
            let f = NodeFactory::new();
            let a = f.new_identifier("a");
            let statements = f.new_node_list(&[a]);
            let block = f.new_block(statements, false);

            // Reads of one list field give one list. The node data holds a
            // copy of the factory list (`synthetic_list_value`).
            let read = block.statement_list();
            assert_eq!(read, block.statement_list());
            assert_eq!(read.list_ptr(), block.statement_list().list_ptr());
            assert_ne!(read, statements);
            assert_eq!(read.nodes().to_vec(), vec![a]);
            assert_eq!(a.text(), "a");

            // A data write gives the field a new list. The old handle still
            // reads the old list, like a Go pointer.
            replace_node_data(block, with_ast_data(block, Clone::clone));
            assert_ne!(block.statement_list(), read);
            assert_eq!(read.nodes().get(0), a);

            assert!(synthetic_slot_count() > 1);
            free_synthetic_nodes();
            assert_eq!(synthetic_slot_count(), 1);
        })
        .join()
        .expect("test thread panicked");
    }

    // Node data and factory lists fill chunks. A new chunk does not move an
    // earlier entry, and a read that holds a node can make more nodes.
    #[test]
    fn synthetic_entries_stay_in_place_across_chunks() {
        std::thread::spawn(|| {
            let f = NodeFactory::new();
            let first = f.new_identifier("first");
            let list = f.new_node_list(&[first]);
            let ptr = list.list_ptr();
            let last = with_ast_data(first, |_| {
                let mut last = NodeList::NIL;
                for i in 0..2 * DATA_CHUNK.max(LIST_CHUNK) {
                    last = f.new_node_list(&[f.new_identifier(format!("n{i}"))]);
                }
                last
            });
            assert_eq!(list.list_ptr(), ptr);
            assert_eq!(list.nodes().to_vec(), vec![first]);
            assert_eq!(first.text(), "first");
            let last_text = format!("n{}", 2 * DATA_CHUNK.max(LIST_CHUNK) - 1);
            assert_eq!(last.nodes().get(0).text(), last_text);

            // The seed copy keeps every index.
            let seed = synthetic_seed();
            std::thread::spawn(move || {
                install_synthetic_seed(seed);
                assert_eq!(list.nodes().to_vec(), vec![first]);
                assert_eq!(first.text(), "first");
                assert_eq!(last.nodes().get(0).text(), last_text);
            })
            .join()
            .expect("seed thread panicked");
            free_synthetic_nodes();
        })
        .join()
        .expect("test thread panicked");
    }

    /// The message of a caught panic.
    fn panic_message(payload: &(dyn std::any::Any + Send)) -> Option<&str> {
        payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
    }

    // A program version that is an owner owns the entries made while it is
    // current, and keeps their identity while it lives. Its free drops them
    // and keeps the base entries. A freed handle panics, and a new node gets
    // a new index.
    #[test]
    fn program_owner_frees_its_entries() {
        std::thread::spawn(|| {
            let program: &'static GoProgram = Box::leak(Box::new(GoProgram {
                id: next_program_id(),
                options: Box::leak(Box::default()),
                state: std::sync::OnceLock::new(),
            }));
            let f = NodeFactory::new();
            let base = f.new_identifier("base");
            let base_slots = synthetic_live_slot_count();

            open_synthetic_owner(program.id);
            let (statement, list, block) = {
                let _program = enter_program(Some(program));
                let statement = f.new_expression_statement(base);
                let list = f.new_node_list(&[statement]);
                let block = f.new_block(list, false);
                assert_eq!(block.statement_list(), block.statement_list());
                assert_eq!(block.statement_list().nodes().to_vec(), vec![statement]);
                assert_eq!(list.nodes().to_vec(), vec![statement]);
                // A data write goes to the owner of the node: the base.
                replace_node_data(base, with_ast_data(base, Clone::clone));
                assert!(synthetic_live_slot_count() > base_slots);
                (statement, list, block)
            };
            free_synthetic_owner(program.id);

            assert_eq!(synthetic_live_slot_count(), base_slots);
            assert_eq!(base.text(), "base");
            let freed = std::panic::catch_unwind(|| block.kind())
                .expect_err("a node of a freed owner is read");
            assert_eq!(panic_message(&*freed), Some(FREED));
            let freed =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| list.nodes().to_vec()))
                    .expect_err("a list of a freed owner is read");
            assert_eq!(panic_message(&*freed), Some(FREED));
            let fresh = f.new_identifier("fresh");
            assert!(fresh != statement && fresh != block);
            assert_eq!(fresh.text(), "fresh");

            // A seed keeps the base indexes and the holes.
            let seed = synthetic_seed();
            std::thread::spawn(move || {
                install_synthetic_seed(seed);
                assert_eq!(base.text(), "base");
                assert_eq!(fresh.text(), "fresh");
                assert!(std::panic::catch_unwind(|| block.kind()).is_err());
            })
            .join()
            .expect("seed thread panicked");
            free_synthetic_nodes();
        })
        .join()
        .expect("test thread panicked");
    }

    // After a client pause the free of an owner takes its entries out of the
    // tables at once (a read panics), but its chunks wait for
    // `drop_garbage` (`gostd::local::drop_after_pause`).
    #[test]
    fn owner_chunks_wait_for_drop_garbage_after_a_client_pause() {
        use crate::gostd::local;
        std::thread::spawn(|| {
            let program: &'static GoProgram = Box::leak(Box::new(GoProgram {
                id: next_program_id(),
                options: Box::leak(Box::default()),
                state: std::sync::OnceLock::new(),
            }));
            local::keep_garbage();
            local::note_message_gap(std::time::Duration::from_millis(20));
            let f = NodeFactory::new();
            let base = f.new_identifier("base");
            let base_slots = synthetic_live_slot_count();

            open_synthetic_owner(program.id);
            let owned = {
                let _program = enter_program(Some(program));
                f.new_identifier("owned")
            };
            free_synthetic_owner(program.id);

            assert_eq!(synthetic_live_slot_count(), base_slots);
            let freed = std::panic::catch_unwind(|| owned.kind())
                .expect_err("a node of a freed owner is read");
            assert_eq!(panic_message(&*freed), Some(FREED));
            assert_eq!(base.text(), "base");
            assert_eq!(local::garbage_len(), 1, "the chunks do not wait");
            local::drop_garbage(|| false);
            assert_eq!(local::garbage_len(), 0);
            free_synthetic_nodes();
        })
        .join()
        .expect("test thread panicked");
    }
}
