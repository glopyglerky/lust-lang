//! Per-file Go node stores (the nodes the ported Go parser creates) and the
//! file registry (the published stores and the `GoFile` of each file id).
//!
//! Go parser nodes are ordinary `*ast.Node` values made by `ast.NodeFactory`.
//! Here each parsed file gets one store. The store id is the file id, so a
//! store node is a normal `Node` handle: high 32 bits are the file id, low 32
//! bits are the slot index + 1.
//!
//! Each slot has a header (Go kind plus the mutable Go `NodeBase` fields:
//! parent, flags, loc) and, for a node slot, a leaked `NodeData` (the Go
//! node data; the kind is in the header). A freeable parse owns the data
//! instead (`OwnedAst`).
//! Child ids inside that data are slot indexes of the same store:
//! - a child in the same file uses its own slot index;
//! - a child from another file or a synthetic child (Go shares the pointer)
//!   uses an alias slot, which `Node::new` resolves to that node;
//! - Go `nil` in a field that astdata stores as a required `NodeId` uses
//!   slot 0, which resolves to `Node::NIL`.
//!
//! `node.rs` reads kind, loc, flags and parent from the header of every
//! parsed node. Only synthetic nodes have no store.
//!
//! Phases of a store:
//! - Build: the parser runs on one thread and writes the stores of that
//!   thread (`BUILD`). The header and the data can change until the parser
//!   finishes the file (`finishNode`, parent setting, JSDoc flags,
//!   reparser.go writes). Then the parser freezes the file, which also
//!   builds the per-store tables that a publish puts in the registry. The
//!   build stores of a thread get consecutive ids from `BuildStores::base`,
//!   the published count when its first store was made. The last store
//!   made on a thread is its `ACTIVE` store: the parser reads and writes it
//!   without a store lookup.
//! - Detached: a parse worker (`files_parser.rs` prefetch) parses one file
//!   into a store with a provisional id (`DETACHED_STORE_BASE` + job) that
//!   only its thread sees (`DETACHED`). The loading thread adopts the
//!   finished store when the loader asks for that file
//!   (`adopt_detached_store`). The store then gets the next real id, so ids
//!   still follow the serial parse order.
//! - Published: the loader calls `publish_file_stores` with the `GoFile` of
//!   each build store before it installs the program. The stores and their
//!   `GoFile`s move into the process-wide, read-only registry. Node reads
//!   then need no thread-local and no `RefCell` borrow, and any thread can
//!   read them. Writes to a published store panic. A table that needs the
//!   real id of an adopted store is built then, on scoped threads for a
//!   large publish (`publish_stores`).
//!
//! The registry:
//! - One file id is one file version. Ids only grow (`PUBLISHED` is the
//!   next unused id), and a published file is never changed. A static
//!   published file is never freed; a freeable file version is freed with
//!   its last holder (lsshells M3b, `ast/file_version.rs`), and a read of
//!   its id after that panics. A new program version shares the ids of its
//!   unchanged files.
//! - AST node records, step 3: each published file id has one entry, its
//!   block (`FileBlock`, `file_block`): a static array for the ids below
//!   `LOW_BLOCKS` (`FILE_BLOCKS`), chunks made on demand above
//!   (`HIGH_BLOCKS`). It holds the per-slot columns of the file (records,
//!   kids, node column), and the rest of the file one load away
//!   (`BlockFile`: facts, root, foreign parents, links, and the store and
//!   `GoFile` of a static publish). The first program and every later one
//!   (edited files, other programs) read the same way.
//! - A freeable file version (an edited file in a language server or API
//!   process): its `FileVersion` owns its store and `GoFile`
//!   (`VersionStore`), and the block of its id is its node shell, with only
//!   its node columns leaked (`node_shell`). With owned nodes on
//!   (the default there, `owned_nodes_enabled`) its parse is a freeable
//!   parse (`enter_freeable_parse`, lsshells M3c), so its store also owns
//!   its astdata nodes, its pending lists and its parse lists (`OwnedAst`):
//!   the shell has no node column, and a node data read of the file is a
//!   scoped read (`read_store_node_miss`). With `GOPORT_OWNED_NODES=0` the
//!   parse is a static parse, whose node column is in the shell, as before
//!   M3c.
//! - Every read of a published store goes through one inline lookup
//!   (`file_block`). An id below `LOW_BLOCKS` reads with no call; a higher
//!   id reads in a cold block that calls the empty `high_block_path`. So
//!   the hot node reads in `node.rs`, the node records (`NodeRecord`,
//!   `NodeKids`) and the perf columns (links, facts) answer for the nodes
//!   of every program, and return `None` on a miss. A read of the store or
//!   the `GoFile` of a node shell reads the freeable version, out of line
//!   and pinned while the read runs (`with_published_store`,
//!   `try_with_go_file`); the accessors that return a borrow return a
//!   `FileRef` guard.
//! - After a registry miss, a synthetic id has no store (a few compares, no
//!   call). Any other id takes one cold call: the detached store, then the
//!   build stores of this thread.
//!
//! AST node records, step 2: the install of the binder output writes the
//! symbol, the flow node and the added flags of each node into its record
//! (`bind_store_records`). The other binder fields stay in
//! `GoFile::node_bind`, by the index in the record.

use crate::astdata::NodeData;
use crate::frontend::parser::SourceFileParseOptions;
use crate::prelude::*;
use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// Slot 0: Go `nil` stored in a astdata field that has no `Option`.
const NIL_SLOT: u32 = 0;

/// Position that marks a Go `nil` list in a astdata list field that has no
/// `Option`. Go positions are byte offsets, so they never reach it, and
/// `u32::MAX` is already the undefined position `-1`.
// PORT: astdata cannot change under the R97 rules. For store nodes the factory
// stores Go `nil` in a required list field as an empty list at this position,
// and `NodeList::is_nil` reads it back as nil (plan risk 1).
pub const NIL_LIST_POS: u32 = u32::MAX - 1;

/// The Go kind and the mutable Go `NodeBase` fields of a store node, as the
/// node reads see them: the value form of its `NodeRecord`
/// (`NodeRecord::header`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeHeader {
    /// Go `node.Parent`.
    pub parent: Node,
    pub loc: TextRange,
    pub flags: NodeFlags,
    pub kind: SyntaxKind,
    /// Set by `mark_source_file_roots`: Go `GetSourceFileOfNode(node)` is the
    /// root of this store (`FileStore::root`). False means "walk the parents".
    source_file_is_root: bool,
    /// Set when the slot is made or its data is replaced, for an Identifier
    /// or PrivateIdentifier slot: Go `scanner.GetIdentifierToken(node.Text())
    /// != KindIdentifier` (`frozen_store_text_is_keyword`). False for other
    /// slots.
    text_is_keyword: bool,
}

impl NodeHeader {
    /// The stored form of `parent` for a node of store `file`.
    #[inline]
    fn stored_parent(file: usize, parent: Node) -> Node {
        if parent.is_some() && parent.file_index() == file {
            handle(LOCAL_STORE, slot_index(parent) as u32)
        } else {
            parent
        }
    }
}


/// Facts about the slots of one finished store, made in one pass over its
/// records (`FileStore::make_facts`). They do not depend on the store id, so a detached
/// store gets them when its parse ends.
#[derive(Clone, Copy, Debug, Default)]
pub struct StoreFacts {
    /// Every slot after slot 0 holds a node (the store has no alias slot),
    /// so `record_resolve(file, i) == handle(file, i)` for every `i >= 1`,
    /// and a child walk from a node of this store stays in this store.
    pub alias_free: bool,
    /// The parent of every node slot is nil or a node of this store, so a
    /// parent walk from a node of this store stays in this store.
    pub parents_local: bool,
    /// Some node slot has kind `ExportAssignment` or `ExportSpecifier`.
    pub has_export_alias_kind: bool,
    /// Some node slot has kind `ConditionalType` or `MappedType`.
    pub has_flow_constraint_kind: bool,
    /// U4 (CH7): some node slot has the parser flag
    /// `POSSIBLY_CONTAINS_DEPRECATED_TAG`. The parse of a frozen store is
    /// over, and the bind never adds that bit (`BINDER_ADDED_FLAGS` in
    /// node.rs), so without it no node of the store has the bit in Go
    /// `node.Flags` (`frozen_store_lacks_deprecated_tag`).
    pub has_deprecated_tag: bool,
}

impl StoreFacts {
    /// The facts of a store that has only slot 0. `add` adds the others.
    const ONLY_NIL_SLOT: Self = Self {
        alias_free: true,
        parents_local: true,
        has_export_alias_kind: false,
        has_flow_constraint_kind: false,
        has_deprecated_tag: false,
    };

    /// Adds the slot of `record`, after slot 0.
    #[inline]
    fn add(&mut self, record: &NodeRecord, kind: SyntaxKind) {
        if !record.is_node() {
            self.alias_free = false;
            return;
        }
        if record.has_foreign_parent() {
            self.parents_local = false;
        }
        if record
            .flags()
            .intersects(NodeFlags::POSSIBLY_CONTAINS_DEPRECATED_TAG)
        {
            self.has_deprecated_tag = true;
        }
        match kind {
            SyntaxKind::ExportAssignment | SyntaxKind::ExportSpecifier => {
                self.has_export_alias_kind = true;
            }
            SyntaxKind::ConditionalType | SyntaxKind::MappedType => {
                self.has_flow_constraint_kind = true;
            }
            _ => {}
        }
    }
}

/// U4 (CH6, bind A): two child ids of a node slot, so that `Node::name`,
/// `Node::expression`, `Node::postfix_token` and `Node::question_token` of a
/// published store node need no load of its astdata node and data
/// (`NodeKids`, `frozen_store_child`). C2 adds a third field for
/// `Node::type_`, `Node::initializer`, `Node::type_name` and
/// `Node::type_argument_list`. The ids are store-local
/// slot indexes, like the child ids in the node data, so they stay valid
/// when a detached store gets its real id. `node.rs`
/// (`store_node_children`) makes the value from the node data with the arm
/// lists of those accessors.
///
/// - `name`: the id of the Go `Name()` child, 0 for nil or for a kind
///   without the field, or `UNKNOWN_ID`.
/// - `other`: a tag in the top two bits and an id below. Each kind has at
///   most one of these fields: `TAG_EXPRESSION` (Go `Expression()`, also
///   tag 0 with id 0 for a kind with none of the fields), `TAG_POSTFIX` (Go
///   `PostfixToken()`) or `TAG_QUESTION` (the node's own Go
///   `QuestionToken` field). All ones (`UNKNOWN_ID`) is unknown.
/// - `typed` (C2): a tag in the top three bits, the `NO_TYPE_ARGUMENTS` bit
///   and an id below, for Go `Type()`, `Initializer()` and
///   `AsTypeReference().TypeName`. `TYPED_TYPE`: `Type()` is the id and
///   `Initializer()` is nil (also id 0 for a node where both are nil, as for
///   a kind with neither field). `TYPED_INITIALIZER`: `Type()` is nil and
///   `Initializer()` is the id. `TYPED_TYPE_WITH_INITIALIZER`: `Type()` is
///   the id, and `Initializer()` is set too but not held. `TYPED_TYPE_NAME`
///   (TypeReference): `TypeName` is the id, `Type()` and `Initializer()`
///   are nil. `NO_TYPE_ARGUMENTS` is set when Go `TypeArgumentList()` is nil
///   and the node data has no list (`Node::type_argument_list` then gives
///   `NodeList::NIL`).
///
/// Unknown means "read the node data": nil and alias slots, the kinds whose
/// accessor arm is not a plain field read (QualifiedName,
/// CaseOrDefaultClause) and an id that does not fit.
// PERF: 12 bytes of the `NodeKids` of the slot. A read is one kids word
// and the per-store record of `Node::new`, not the chain kind table, node
// pointer, node tag, data box, field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotChildren {
    name: u32,
    other: u32,
    typed: u32,
}

impl SlotChildren {
    /// A field value that means "read the node data".
    const UNKNOWN_ID: u32 = u32::MAX;
    const TAG_SHIFT: u32 = 30;
    /// The largest id that `other` can hold.
    const MAX_TAGGED_ID: u32 = (1 << Self::TAG_SHIFT) - 1;
    pub(crate) const TAG_EXPRESSION: u32 = 0;
    pub(crate) const TAG_POSTFIX: u32 = 1;
    pub(crate) const TAG_QUESTION: u32 = 2;
    const TAG_UNKNOWN: u32 = 3;

    const TYPED_TAG_SHIFT: u32 = 29;
    /// C2: Go `TypeArgumentList()` is nil (see the type doc).
    const NO_TYPE_ARGUMENTS: u32 = 1 << 28;
    /// The largest id that `typed` can hold.
    const MAX_TYPED_ID: u32 = Self::NO_TYPE_ARGUMENTS - 1;
    pub(crate) const TYPED_TYPE: u32 = 0;
    pub(crate) const TYPED_INITIALIZER: u32 = 1;
    pub(crate) const TYPED_TYPE_WITH_INITIALIZER: u32 = 2;
    pub(crate) const TYPED_TYPE_NAME: u32 = 3;
    /// Tags 4 to 6 are not used.
    const TYPED_UNKNOWN: u32 = 7;
    /// `typed` when unknown: every read takes the node data. The
    /// `NO_TYPE_ARGUMENTS` bit is clear.
    const TYPED_UNKNOWN_VALUE: u32 = Self::TYPED_UNKNOWN << Self::TYPED_TAG_SHIFT;

    /// Every read takes the node data.
    pub(crate) const UNKNOWN: Self = Self {
        name: Self::UNKNOWN_ID,
        other: Self::UNKNOWN_ID,
        typed: Self::TYPED_UNKNOWN_VALUE,
    };

    /// `name` is the name child id (0 for nil or no field, `None` for
    /// unknown); `other` is `(tag, id)` (`None` for unknown). The `typed`
    /// field is unknown (see `with_typed`).
    #[inline]
    pub(crate) fn new(name: Option<u32>, other: Option<(u32, u32)>) -> Self {
        let other = match other {
            Some((tag, id)) if tag < Self::TAG_UNKNOWN && id <= Self::MAX_TAGGED_ID => {
                (tag << Self::TAG_SHIFT) | id
            }
            _ => Self::UNKNOWN_ID,
        };
        Self {
            // `UNKNOWN_ID` itself is not a slot id a store can reach, and
            // it reads as unknown, which is always exact.
            name: name.unwrap_or(Self::UNKNOWN_ID),
            other,
            typed: Self::TYPED_UNKNOWN_VALUE,
        }
    }

    /// C2: this entry with `typed` set from `typed` (`(tag, id)`, `None`
    /// for unknown) and `no_type_arguments`.
    #[inline]
    pub(crate) fn with_typed(self, typed: Option<(u32, u32)>, no_type_arguments: bool) -> Self {
        let typed = match typed {
            Some((tag, id)) if tag <= Self::TYPED_TYPE_NAME && id <= Self::MAX_TYPED_ID => {
                (tag << Self::TYPED_TAG_SHIFT) | id
            }
            _ => Self::TYPED_UNKNOWN_VALUE,
        };
        let no_type_arguments = if no_type_arguments {
            Self::NO_TYPE_ARGUMENTS
        } else {
            0
        };
        Self {
            typed: typed | no_type_arguments,
            ..self
        }
    }

    /// The tag and id of an `other` field value.
    #[inline]
    fn other_parts_of(other: u32) -> (u32, u32) {
        (other >> Self::TAG_SHIFT, other & Self::MAX_TAGGED_ID)
    }

    /// C2: the tag and id of a `typed` field value.
    #[inline]
    fn typed_parts_of(typed: u32) -> (u32, u32) {
        (typed >> Self::TYPED_TAG_SHIFT, typed & Self::MAX_TYPED_ID)
    }

    /// C2: a `typed` field value says Go `TypeArgumentList()` is nil.
    #[inline]
    fn has_no_type_arguments_in(typed: u32) -> bool {
        typed & Self::NO_TYPE_ARGUMENTS != 0
    }

    /// The tag and id of `other`.
    #[cfg(test)]
    fn other_parts(self) -> (u32, u32) {
        Self::other_parts_of(self.other)
    }

    /// C2: the tag and id of `typed`.
    #[cfg(test)]
    fn typed_parts(self) -> (u32, u32) {
        Self::typed_parts_of(self.typed)
    }

    /// C2: Go `TypeArgumentList()` is nil.
    #[cfg(test)]
    fn has_no_type_arguments(self) -> bool {
        Self::has_no_type_arguments_in(self.typed)
    }
}

/// AST node records (`ast-design/study.md`): the Go `NodeBase` fields and
/// the hot binder fields of one slot in 24 bytes (`FileStore::records`).
/// `NodeHeader` is the value form of its parse fields. The words are
/// atomics that the reads load with `Relaxed` (a plain load on x86-64 and
/// aarch64). The parse writes them through `get_mut`; the binder writes its
/// fields into the published record (`bind_store_records`).
///
/// - The Go `node.Kind` of the slot is not in its record but in the kind
///   column (`FileStore::kinds`, `FileBlock::kinds`).
/// - `flags`: Go `node.Flags` below `RECORD_BITS`: the parser flags, and
///   after the bind also the bits the binder added (`BINDER_ADDED_FLAGS` in
///   node.rs). In `RECORD_BITS` (the top 3 bits, which no Go `NodeFlags`
///   value uses): `SOURCE_FILE_ROOT`, `TEXT_IS_KEYWORD` and `NO_NODE`.
/// - `loc`: Go `node.Loc`, pos in the low half and end in the high half.
/// - `up` of a node slot: the parent code in the low half (`ParentCode`),
///   and the Go symbol (`SymbolId`) in the high half, 0 before the bind.
///   `up` of the nil slot or an alias slot: the whole target `Node`.
/// - `bind` without `BIND_EXTRA`: the low half of the Go
///   `FlowNodeData().FlowNode` (the flow node is in the same file; 0 is
///   nil). With `BIND_EXTRA`: the other bits are the index + 1 of the
///   binder fields of the node in `FileNodeBind` (`NodeBindExtra`), which
///   hold its flow node too. AST node records, step 4: `bind` of the nil
///   slot (slot 0) is the owner, the file id of the block
///   (`block_is_owned`). The publish writes it; the bind never writes the
///   nil slot.
///
/// Only the parse and the bind write a record. The bind writes after the
/// publish, and changes only `flags` (it ORs in the bits it adds, so the
/// record bits stay), the high half of `up` and `bind`. `header` reads each
/// word once and takes only
/// the low half of `up` of a node slot (the bind never writes `up` of the
/// nil slot or an alias slot), so it sees the parse fields and the flags
/// from before or after the bind, never a mix of one word. The bind of a
/// file ends before any reader of its binder fields starts: the checker
/// threads start after the bind, or get their work through a lock or a
/// channel, which orders the writes before their reads.
// PORT: AST node records, step 4: every word is an atomic, so a pooled
// block (`BlockPool`) can take the records of another file through a shared
// borrow, with no `unsafe`. The kind of a slot never changes after the slot
// is made (`replace_store_node_data` keeps it), so it is in a plain column
// of its own (`FileStore::kinds`), which a pooled block does not hold.
// PERF: step 4 first had an `AtomicU16` kind in the record, read through a
// 512-entry table (safe Rust has no cheaper `u16` to `SyntaxKind`):
// `goport -p` +1.3% to +2.1% instructions against step 7, and a plain
// `transmute` of the atomic load still +0.6% to +1.1%
// (ast-design/step4b). The column adds 2 bytes per slot.
// PERF: astmem1 P3. 24 bytes, not 32: the record bits are in the top of
// `flags`, and the `bind` word is 32 bits. The extras index of the 60% of
// node slots with no binder data took 4 bytes, and `bits` 4 bytes with its
// padding. A slot with extras (locals containers, exported declarations,
// function-like nodes: about 6% of slots) keeps its flow node in its
// extras entry, so the flow read of every other node is one load, as before.
#[repr(C)]
#[derive(Debug)]
pub struct NodeRecord {
    flags: AtomicU32,
    bind: AtomicU32,
    loc: AtomicU64,
    up: AtomicU64,
}

const _: () = assert!(std::mem::size_of::<NodeRecord>() == 24);

// The bind ORs only `BINDER_ADDED_FLAGS` into a record's `flags`
// (`bind_store_records` checks it), so it keeps the record bits.

/// The bit of a `NodeRecord::bind` word that holds an extras index
/// (`NodeBindExtra`), not a flow node.
pub const BIND_EXTRA: u32 = 1 << 31;

// The owner word (`NodeRecord::owner_word`) is a file id.
const _: () = assert!(FILE_ID_LIMIT <= 1 << 31);

/// AST node records, step 2: the low half of `NodeRecord::up` of a node
/// slot. 0 is a nil parent. `1..FOREIGN_PARENT` is a parent in the same
/// store: its slot index + 1, so the handle is `(file << 32) | code` and
/// needs no rewrite when the store id changes (`adopt_detached_store`).
/// `FOREIGN_PARENT | i` is a parent in another store (or a synthetic
/// parent): entry `i` of the foreign parents of the store
/// (`FileStore::foreign_parents`, `BlockFile::foreign`).
type ParentCode = u32;

/// The `ParentCode` bit of a parent in another store.
const FOREIGN_PARENT: ParentCode = 1 << 31;

impl NodeRecord {
    /// Go `GetSourceFileOfNode(node)` is the root of the store
    /// (`mark_source_file_roots`).
    const SOURCE_FILE_ROOT: u32 = 1 << 29;
    /// U1 (a): Go `scanner.GetIdentifierToken(node.Text()) !=
    /// KindIdentifier` of an Identifier or PrivateIdentifier slot.
    const TEXT_IS_KEYWORD: u32 = 1 << 30;
    /// The slot holds no node: the nil slot or an alias slot. Its target is
    /// in `up`.
    const NO_NODE: u32 = 1 << 31;
    /// The record bits in the `flags` word. Go `NodeFlags` ends at bit 28
    /// (`REPARSER_TRANSFORMED_LITERAL`); a flags write checks it: the parse
    /// writes in `checked_flags`, the bind writes in `bind_store_records`
    /// (`BINDER_ADDED_FLAGS`, which has no record bit).
    const RECORD_BITS: u32 = Self::SOURCE_FILE_ROOT | Self::TEXT_IS_KEYWORD | Self::NO_NODE;

    /// The record of a new node slot: Go `newNode` (undefined loc, nil
    /// parent, no flags).
    #[inline]
    fn node(text_is_keyword: bool) -> Self {
        Self::new(
            if text_is_keyword {
                Self::TEXT_IS_KEYWORD
            } else {
                0
            },
            NodeFlags::NONE,
            TextRange::undefined(),
            0,
        )
    }

    /// The nil slot (`target` nil) or an alias slot.
    #[inline]
    fn target(target: Node) -> Self {
        Self::new(
            Self::NO_NODE,
            NodeFlags::NONE,
            TextRange::undefined(),
            target.0,
        )
    }

    /// A slot with word `up` (see `NodeRecord`) and no binder fields.
    #[inline]
    fn new(bits: u32, flags: NodeFlags, loc: TextRange, up: u64) -> Self {
        debug_assert_eq!(bits & !Self::RECORD_BITS, 0);
        Self {
            flags: AtomicU32::new(Self::checked_flags(flags) | bits),
            bind: AtomicU32::new(0),
            loc: AtomicU64::new(Self::loc_word(loc)),
            up: AtomicU64::new(up),
        }
    }

    /// `flags.0`, which must not use `RECORD_BITS`.
    #[inline]
    fn checked_flags(flags: NodeFlags) -> u32 {
        assert_eq!(
            flags.0 & Self::RECORD_BITS,
            0,
            "Go NodeFlags {:#x} in the node record bits",
            flags.0
        );
        flags.0
    }

    #[inline]
    fn loc_word(loc: TextRange) -> u64 {
        u64::from(loc.pos() as u32) | (u64::from(loc.end() as u32) << 32)
    }

    /// A record of a fresh pool block (`BlockPool`): all words 0.
    fn zero() -> Self {
        Self {
            flags: AtomicU32::new(0),
            bind: AtomicU32::new(0),
            loc: AtomicU64::new(0),
            up: AtomicU64::new(0),
        }
    }

    /// The record bits (`RECORD_BITS`).
    #[inline]
    fn bits(&self) -> u32 {
        self.flags.load(Ordering::Relaxed) & Self::RECORD_BITS
    }

    #[inline]
    fn has_bit(&self, bit: u32) -> bool {
        self.bits() & bit != 0
    }

    /// True for a node slot, false for the nil slot and alias slots.
    #[inline]
    fn is_node(&self) -> bool {
        !self.has_bit(Self::NO_NODE)
    }

    #[inline]
    fn flags(&self) -> NodeFlags {
        NodeFlags(self.flags.load(Ordering::Relaxed) & !Self::RECORD_BITS)
    }

    #[inline]
    fn loc(&self) -> TextRange {
        let word = self.loc.load(Ordering::Relaxed);
        TextRange::new(word as u32 as i32, (word >> 32) as u32 as i32)
    }

    /// The target of the nil slot or an alias slot (`NO_NODE`).
    #[inline]
    fn target_node(&self) -> Node {
        Node(self.up.load(Ordering::Relaxed))
    }

    /// The `ParentCode` of a node slot.
    #[inline]
    fn parent_code(&self) -> ParentCode {
        self.up.load(Ordering::Relaxed) as u32
    }


    /// True when the parent of a node slot is in another store.
    #[inline]
    fn has_foreign_parent(&self) -> bool {
        self.parent_code() & FOREIGN_PARENT != 0
    }

    /// The header of this slot of store `file`, whose kind is `kind` and
    /// whose foreign parents are `foreign`, as the node reads see it. The
    /// parent of the nil slot or an alias slot is its target.
    #[inline]
    fn header(&self, kind: SyntaxKind, file: usize, foreign: &[Node]) -> NodeHeader {
        let word = self.flags.load(Ordering::Relaxed);
        let bits = word & Self::RECORD_BITS;
        let up = self.up.load(Ordering::Relaxed);
        let code = up as ParentCode;
        NodeHeader {
            parent: if bits & Self::NO_NODE != 0 {
                Node(up)
            } else if code & FOREIGN_PARENT != 0 {
                // No call and no panic, so a reader that does not use the
                // parent loses this code after inlining.
                let parent = foreign.get((code & !FOREIGN_PARENT) as usize).copied();
                debug_assert!(parent.is_some(), "foreign parent {code:#x}");
                parent.unwrap_or_default()
            } else {
                local_parent_of_code(code, file)
            },
            loc: self.loc(),
            flags: NodeFlags(word & !Self::RECORD_BITS),
            kind,
            source_file_is_root: bits & Self::SOURCE_FILE_ROOT != 0,
            text_is_keyword: bits & Self::TEXT_IS_KEYWORD != 0,
        }
    }

    /// AST node records, step 2: Go `node.Symbol()` of a node slot (nil
    /// before the bind).
    #[inline]
    fn symbol(&self) -> SymbolId {
        SymbolId((self.up.load(Ordering::Relaxed) >> 32) as u32)
    }

    /// AST node records, step 2: the `bind` word (see `NodeRecord`).
    #[inline]
    fn bind_word(&self) -> u32 {
        self.bind.load(Ordering::Relaxed)
    }

    /// AST node records, step 4: writes every word of `from` into this
    /// record of a pooled block (`node_shell`), with `Relaxed` stores. The
    /// publish of the block (`set_file_block`) orders them before the reads
    /// of the new owner.
    #[inline]
    fn store_from(&self, from: &NodeRecord) {
        let relaxed = Ordering::Relaxed;
        self.flags.store(from.flags.load(relaxed), relaxed);
        self.bind.store(from.bind.load(relaxed), relaxed);
        self.loc.store(from.loc.load(relaxed), relaxed);
        self.up.store(from.up.load(relaxed), relaxed);
    }

    /// AST node records, step 4: the owner word of the nil slot of the
    /// block of file `file` (see `NodeRecord`, `bind`).
    // PERF: ownercheck1. The file id itself, not id + 1: the owner check
    // (`check_block_owner`) compares it with the file id of the handle, which
    // the read has in a register already, with no add. A fresh pool block
    // is all zero, but no read reaches it before `node_shell` writes its
    // owner.
    #[inline]
    fn owner_word(file: usize) -> u32 {
        // `FILE_ID_LIMIT` fits (see the assert after `BIND_EXTRA`).
        file as u32
    }

    /// AST node records, step 4: true when this nil slot record is the one
    /// of the block of file `file` (`block_is_owned`).
    #[inline]
    fn is_owned_by(&self, file: usize) -> bool {
        self.bind_word() == Self::owner_word(file)
    }

    /// Writes the parent code of a node slot.
    #[inline]
    fn set_parent_code(&mut self, code: ParentCode) {
        let up = self.up.get_mut();
        *up = (*up & !0xffff_ffff) | u64::from(code);
    }

    #[inline]
    fn set_loc(&mut self, loc: TextRange) {
        *self.loc.get_mut() = Self::loc_word(loc);
    }

    /// Writes the Go flags and keeps the record bits.
    #[inline]
    fn set_flags(&mut self, flags: NodeFlags) {
        let word = self.flags.get_mut();
        *word = (*word & Self::RECORD_BITS) | Self::checked_flags(flags);
    }

    #[inline]
    fn set_bit(&mut self, bit: u32, on: bool) {
        debug_assert_eq!(bit & !Self::RECORD_BITS, 0);
        let word = self.flags.get_mut();
        if on {
            *word |= bit;
        } else {
            *word &= !bit;
        }
    }

    /// AST node records, step 2: writes the binder fields of a node slot of
    /// a published store (`bind_store_records`): the symbol into `up`, the
    /// added flags into `flags` (the record bits stay), and the `bind` word.
    /// The bind is the only writer of a published record, and it writes
    /// each record once. `added` has no record bit: `bind_store_records`
    /// checks each data entry against `BINDER_ADDED_FLAGS` in release
    /// builds too, and those flags have none (const assert after
    /// `NodeRecord`).
    #[inline]
    fn write_bind(&self, symbol: SymbolId, added: NodeFlags, bind: u32) {
        debug_assert_eq!(added.0 & Self::RECORD_BITS, 0);
        if symbol.is_some() {
            let up = self.up.load(Ordering::Relaxed);
            self.up.store(
                (up & 0xffff_ffff) | (u64::from(symbol.0) << 32),
                Ordering::Relaxed,
            );
        }
        if !added.is_empty() {
            let flags = self.flags.load(Ordering::Relaxed);
            self.flags.store(flags | added.0, Ordering::Relaxed);
        }
        if bind != 0 {
            self.bind.store(bind, Ordering::Relaxed);
        }
    }
}

/// The `ParentCode` of `stored`, nil or a `LOCAL_STORE` handle
/// (`NodeHeader::stored_parent`).
#[inline]
fn local_parent_code(stored: Node) -> ParentCode {
    debug_assert!(stored.is_nil() || stored.file_index() == LOCAL_STORE);
    let code = stored.0 as u32;
    assert!(code < FOREIGN_PARENT, "too many slots in a store");
    code
}

/// Go `node.Parent` of a node slot of store `file` whose parent code
/// `code` is nil or a slot of the store (not `FOREIGN_PARENT`).
#[inline]
fn local_parent_of_code(code: ParentCode, file: usize) -> Node {
    debug_assert_eq!(code & FOREIGN_PARENT, 0);
    if code == 0 {
        Node::NIL
    } else {
        Node(((file as u64) << 32) | u64::from(code))
    }
}

/// AST node records, step 1: the child ids and the name or modifier word of
/// one slot in 16 bytes (`FileStore::kids`), next to its `NodeRecord`.
/// `name`, `other` and `typed` are the fields of `SlotChildren` (U4, C2).
/// `word` is the interned Go `node.Text()` (`Name::id`) of an Identifier or
/// PrivateIdentifier slot (U1 (a) (d)), and the U1 (b) modifier bits
/// (`ModifierList::modifier_flags` of the node's own list) of any other
/// slot. The parse writes them through `get_mut`.
#[derive(Debug)]
pub struct NodeKids {
    name: AtomicU32,
    other: AtomicU32,
    typed: AtomicU32,
    word: AtomicU32,
}

const _: () = assert!(std::mem::size_of::<NodeKids>() == 16);

impl NodeKids {
    /// Every child read takes the node data; no name, no modifier bits. The
    /// nil slot and alias slots.
    #[inline]
    fn unknown() -> Self {
        Self::new(SlotChildren::UNKNOWN, 0)
    }

    #[inline]
    fn new(children: SlotChildren, word: u32) -> Self {
        Self {
            name: AtomicU32::new(children.name),
            other: AtomicU32::new(children.other),
            typed: AtomicU32::new(children.typed),
            word: AtomicU32::new(word),
        }
    }


    #[inline]
    fn children(&self) -> SlotChildren {
        SlotChildren {
            name: self.name.load(Ordering::Relaxed),
            other: self.other.load(Ordering::Relaxed),
            typed: self.typed.load(Ordering::Relaxed),
        }
    }

    #[inline]
    fn name_id(&self) -> u32 {
        self.name.load(Ordering::Relaxed)
    }

    #[inline]
    fn other(&self) -> u32 {
        self.other.load(Ordering::Relaxed)
    }

    #[inline]
    fn typed(&self) -> u32 {
        self.typed.load(Ordering::Relaxed)
    }

    #[inline]
    fn word(&self) -> u32 {
        self.word.load(Ordering::Relaxed)
    }

    /// U1 (a): the name of a slot of name kind `kind`; `Name::default()`
    /// for any other kind.
    #[inline]
    fn text_name(&self, kind: SyntaxKind) -> Name {
        if is_name_kind(kind) {
            Name::from_id(self.word())
        } else {
            Name::default()
        }
    }

    /// U1 (b): the modifier bits of a slot of kind `kind`; 0 for a name
    /// kind.
    #[inline]
    fn modifier_bits(&self, kind: SyntaxKind) -> u32 {
        if is_name_kind(kind) { 0 } else { self.word() }
    }

    #[inline]
    fn set_children(&mut self, children: SlotChildren) {
        *self.name.get_mut() = children.name;
        *self.other.get_mut() = children.other;
        *self.typed.get_mut() = children.typed;
    }

    #[inline]
    fn set_word(&mut self, word: u32) {
        *self.word.get_mut() = word;
    }

    /// AST node records, step 4: writes every word of `from` into this
    /// entry of a pooled block (`NodeRecord::store_from`).
    #[inline]
    fn store_from(&self, from: &NodeKids) {
        let relaxed = Ordering::Relaxed;
        self.name.store(from.name.load(relaxed), relaxed);
        self.other.store(from.other.load(relaxed), relaxed);
        self.typed.store(from.typed.load(relaxed), relaxed);
        self.word.store(from.word.load(relaxed), relaxed);
    }
}

/// Identifier or PrivateIdentifier: a kind whose slot has a U1 (a) name.
#[inline]
fn is_name_kind(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::Identifier | SyntaxKind::PrivateIdentifier)
}

/// Which `SlotChildren` field `frozen_store_child` reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreChild {
    /// Go `node.Name()`.
    Name,
    /// Go `node.Expression()`.
    Expression,
    /// Go `node.PostfixToken()`.
    PostfixToken,
    /// Go `node.QuestionToken()`: the own field, else the postfix token when
    /// it is a `?` token.
    QuestionToken,
    /// C2: Go `node.Type()`.
    Type,
    /// C2: Go `node.Initializer()`.
    Initializer,
    /// C2: Go `node.AsTypeReference().TypeName`. `None` (read the data, which
    /// panics like Go) for other kinds.
    TypeName,
}

/// R2-5: the binder child links of one slot (`FileStore::links`): the
/// children of a node in Go `ForEachChild` order as a chain, first child
/// and next sibling, so `Binder::bind_each_child` walks them without the
/// node data. The ids are store-local slot indexes, so they stay valid when
/// a detached store gets its real id.
///
/// - `first_child`: the first child of this node, `LINK_END` for a node
///   with no child, `LINK_NONE` when the chain of this node is not known.
/// - `next_sibling`: the next child in the chain that holds this slot,
///   `LINK_END` for the last one, `LINK_NONE` when no chain holds it.
///
/// Rules that keep a known chain equal to Go `ForEachChild` of its node:
/// - The parser links a node when it sets the parents of its children
///   (`StoreChildLinks`), in the same visit. A chain holds only node slots
///   of the same store.
/// - A slot is in at most one chain. A child that another chain holds
///   (a node that two parents share, or a child twice in one node), a
///   child of another store and an alias child make the node unknown.
/// - A new data write to a node (`replace_store_node_data`) frees its chain
///   and makes it unknown until the parser links it again.
/// - A store whose parse did not finish gets no links (`publish`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SlotLinks {
    first_child: u32,
    next_sibling: u32,
}

/// R2-5: a `SlotLinks` field value: the chain ends (slot 0, the nil slot,
/// is never a child in a chain).
const LINK_END: u32 = NIL_SLOT;
/// R2-5: a `SlotLinks` field value: not known (`first_child`) or in no
/// chain (`next_sibling`).
const LINK_NONE: u32 = u32::MAX;

impl SlotLinks {
    /// A slot with no known chain that no chain holds.
    const NONE: Self = Self {
        first_child: LINK_NONE,
        next_sibling: LINK_NONE,
    };
}

/// U1 (e): capacity hints for the binder of a store file, counted from the
/// slot kinds when the store is frozen (`frozen_store_bind_estimate`).
/// Only capacities: the binder output does not depend on them.
#[derive(Clone, Copy, Debug, Default)]
struct BindEstimate {
    /// `NodeBindBuilder` entries: about the nodes that get binder data.
    entries: u32,
    /// `Binder::flow_nodes`: about the flow nodes the binder makes.
    flow_nodes: u32,
}

/// The slot kind counts of a `BindEstimate`, one `add` per slot kind.
// PERF: U1 (e). `make_facts` adds each kind in its pass over the records,
// so the estimate needs no second pass over the kinds.
#[derive(Default)]
struct BindCounts {
    identifiers: usize,
    others: usize,
    flow_nodes: usize,
}

impl BindCounts {
    #[inline]
    fn add(&mut self, kind: SyntaxKind) {
        match kind {
            // The nil slot and alias slots.
            SyntaxKind::Unknown => {}
            SyntaxKind::Identifier => self.identifiers += 1,
            _ => {
                self.others += 1;
                self.flow_nodes += flow_node_weight(kind);
            }
        }
    }

    fn estimate(&self) -> BindEstimate {
        // Every identifier gets its flow node (binder.go `bind`), and about
        // one other node in two is a declaration, a container, a statement
        // or a narrowable reference. An estimate above the count only costs
        // untouched capacity until the builder is dropped.
        let entries = self.identifiers + self.others / 2 + 1;
        // The unreachable flow node and the source file start node.
        let flow_nodes = self.flow_nodes + 2;
        BindEstimate {
            entries: u32::try_from(entries).unwrap_or(u32::MAX),
            flow_nodes: u32::try_from(flow_nodes).unwrap_or(u32::MAX),
        }
    }
}

/// About how many flow nodes the binder makes for a node of kind `kind`
/// (binder.go `bindContainer` and the `bind*Flow` functions).
fn flow_node_weight(kind: SyntaxKind) -> usize {
    match kind {
        // A start node for a control flow container, a flow call, a flow
        // assignment, or a logical or assignment operator.
        SyntaxKind::MethodSignature
        | SyntaxKind::CallSignature
        | SyntaxKind::ConstructSignature
        | SyntaxKind::FunctionType
        | SyntaxKind::ConstructorType
        | SyntaxKind::FunctionDeclaration
        | SyntaxKind::MethodDeclaration
        | SyntaxKind::GetAccessor
        | SyntaxKind::SetAccessor
        | SyntaxKind::FunctionExpression
        | SyntaxKind::ArrowFunction
        | SyntaxKind::ModuleBlock
        | SyntaxKind::CallExpression
        | SyntaxKind::VariableDeclaration
        | SyntaxKind::BindingElement
        | SyntaxKind::BinaryExpression
        | SyntaxKind::PostfixUnaryExpression
        | SyntaxKind::DeleteExpression => 1,
        // A start node and a return label, or a clause and its label.
        SyntaxKind::Constructor
        | SyntaxKind::ClassStaticBlockDeclaration
        | SyntaxKind::CaseClause
        | SyntaxKind::DefaultClause
        | SyntaxKind::LabeledStatement => 2,
        // Branch or loop labels and the true and false conditions.
        SyntaxKind::IfStatement
        | SyntaxKind::ConditionalExpression
        | SyntaxKind::WhileStatement
        | SyntaxKind::DoStatement
        | SyntaxKind::ForStatement
        | SyntaxKind::ForInStatement
        | SyntaxKind::ForOfStatement
        | SyntaxKind::SwitchStatement
        | SyntaxKind::TryStatement => 5,
        _ => 0,
    }
}








/// File index that marks a parent in the same store inside a stored
/// header. Reads give the handle of the store (`NodeHeader::read`), so a
/// store keeps its records when its id changes (`adopt_detached_store`).
const LOCAL_STORE: usize = 0x7fff_ffff;

/// First provisional store id. Real file ids stay below `FILE_ID_LIMIT`;
/// synthetic node and flow ids are above every provisional id.
pub const DETACHED_STORE_BASE: usize = 0x8000_0000;
/// Number of provisional ids.
pub const DETACHED_STORE_LIMIT: usize = 0x4000_0000;

/// Every real file id is below this (`file_block`). Ids are never used
/// again: each new file version takes one, and so does each file of a
/// whole parse again (a parse option change, a reopen, a `tsc -w` tsconfig
/// edit).
// fileid1: 2^22 ids took 116 hours at 10 language server edits per second
// (one id per edit). At 2^28 that is 7,456 hours, and the 64-byte entry of
// each id from `LOW_BLOCKS` on (16 GiB at the cap) plus the other memory
// of each edit runs out first.
const FILE_ID_LIMIT: usize = 1 << 28;
/// AST node records, step 3: a file id below this has its entry in
/// `FILE_BLOCKS`; a higher one in a chunk of `HIGH_BLOCKS`.
const LOW_BLOCKS: usize = 1 << 16;
/// Entries per `HIGH_BLOCKS` chunk: 1 MiB of entries per chunk, and 16,380
/// chunk cells (256 KiB of `.data`) up to `FILE_ID_LIMIT`.
const HIGH_CHUNK: usize = 1 << 14;

// A real file id is never `LOCAL_STORE`, and the chunks end at the limit.
const _: () = assert!(FILE_ID_LIMIT <= LOCAL_STORE);
const _: () = assert!((FILE_ID_LIMIT - LOW_BLOCKS) % HIGH_CHUNK == 0);

/// The end of the new file ids (`BuildStores::next_id`,
/// `publish_file_stores`).
#[cfg(not(test))]
#[inline]
fn file_id_cap() -> usize {
    FILE_ID_LIMIT
}

#[cfg(test)]
thread_local! {
    /// The `file_id_cap` of this test thread, so a test can reach it.
    static TEST_FILE_ID_CAP: std::cell::Cell<usize> = const { std::cell::Cell::new(FILE_ID_LIMIT) };
}

/// `file_id_cap` in a test: the lower cap of the thread
/// (`TEST_FILE_ID_CAP`).
#[cfg(test)]
fn file_id_cap() -> usize {
    TEST_FILE_ID_CAP.get().min(FILE_ID_LIMIT)
}

#[inline]
fn is_detached_id(file: usize) -> bool {
    (DETACHED_STORE_BASE..DETACHED_STORE_BASE + DETACHED_STORE_LIMIT).contains(&file)
}

/// True for an id that never has a store: a synthetic node or flow id, or
/// any other id at or above `FILE_ID_LIMIT` that is not a provisional id.
#[inline]
fn is_storeless_id(file: usize) -> bool {
    file >= FILE_ID_LIMIT && !is_detached_id(file)
}








/// The next unused file id. Only `publish_file_stores` changes it.
static PUBLISHED: AtomicUsize = AtomicUsize::new(0);

/// Marks the `HIGH_BLOCKS` part of `file_block` as cold. It does nothing.
// PERF: rustc gives a branch into a block that calls a `#[cold]` function
// a low weight (`find_cold_blocks`), and LLVM puts that block after the hot
// code. `std::hint::cold_path` does the same, but is stable only from Rust
// 1.95. `inline(never)` keeps the call in the MIR until codegen reads it.
// LLVM keeps the call too: each copy of `file_block` has a call to this
// empty function (through the GOT) in its cold block, so an out-of-line
// reader copy is not a leaf function. The hot path runs no call.
#[cold]
#[inline(never)]
fn high_block_path() {}











/// AST node records, step 4: the records and kids of one pooled block
/// (`BlockPool`), leaked once and then reused by the node shells of
/// freeable file versions. Not `Copy` and not `Clone`, so a block is in one
/// place only: a `VersionStore`, the quarantine or a free list. The
/// default is the empty block, which is not pooled.
#[derive(Default)]
struct PoolBlock {
    records: &'static [NodeRecord],
    kids: &'static [NodeKids],
}

impl PoolBlock {

    /// The number of slots.
    fn cap(&self) -> usize {
        self.records.len()
    }
}

/// AST node records, step 4: the blocks of the node shells of dead freeable
/// file versions, for the node shells of later versions. An edit then
/// reuses the 40 bytes per node (a record and its kids, 24 + 16) of an
/// older version of the file, where each shell leaked them before. The
/// blocks never go back to the allocator.
/// - A version's store gives its block back when it drops (`VersionStore`):
///   when the version dies, or a little after on a free thread when a pin
///   release or the build takes the store out of the dying version
///   (`FileVersion::take_data`). The pin epoch is read at that drop. The
///   block waits in `quarantine` until `QUARANTINE_RELEASES` more pin
///   releases (`file_version::pin_epoch`) have happened, then goes to the
///   free list of its size class (`pool_class`). A pin release is a
///   program release, or the release of a source file lease that held the
///   last holder of a freeable parse
///   (`file_version::release_file_version_pins_later`, apimem1).
/// - A node shell of `len` slots takes the first free block of `len` to
///   `2 * len + 64` slots (`take_pool_block`), or a fresh one of
///   `len + len / 8 + 64` slots, so a file that grows a little while it is
///   edited still fits its old block.
// PORT: Go frees a dead `*ast.SourceFile` with the GC. Every holder of a
// node of a version holds the version, so no read of it can follow its
// death; the quarantine is a margin for a thread that keeps a node handle
// and reads only its header, which does not pin the version. A client that
// releases leases fast makes the pin releases come sooner, so the margin is
// shorter in time. After a reuse such a read gives the new owner's data;
// with debug assertions it panics (`file_block`, `block_is_owned`).
struct BlockPool {
    /// The free blocks by size class.
    free: Vec<Vec<PoolBlock>>,
    /// The given-back blocks with the pin epoch from which they are free,
    /// in the order they came back (the epochs only grow).
    quarantine: VecDeque<(usize, PoolBlock)>,
    /// Each kind column that `shell_kinds` leaked, by the hash of its
    /// kinds.
    kind_columns: FxHashMap<u64, Vec<&'static [SyntaxKind]>>,
}

/// The number of pin releases a given-back block waits (`BlockPool`). A
/// version usually dies inside a release, after its epoch bump, and a node
/// shell takes its block before the release of its edit: in a language
/// server with no lease release between the edits, the version that dies
/// in the release of edit N gives its block to the version of edit N + 3.
const QUARANTINE_RELEASES: usize = 2;







/// Empties the pool (its blocks stay leaked), so a test sees only the
/// blocks it gave back.
#[cfg(test)]
fn clear_block_pool() {
    let mut pool = lock_pool();
    pool.free.clear();
    pool.quarantine.clear();
    pool.kind_columns.clear();
}












/// The handle of slot `index` in store `file`. Does not resolve aliases.
const fn handle(file: usize, index: u32) -> Node {
    Node(((file as u64) << 32) | (index as u64 + 1))
}

/// Slot index of a store handle.
#[inline]
fn slot_index(n: Node) -> usize {
    ((n.0 & 0xffff_ffff) - 1) as usize
}








// ──────────────────────────────────────────────────────────────────────
// Stores
// ──────────────────────────────────────────────────────────────────────



/// Text bytes per slot that a new store reserves for (`FileStore::new`).
// PERF: U1 (e). The TypeScript parser makes one node for about 6 to 8 text
// bytes of the Query, Zod and Hono sources, and one for about 14 to 21 bytes
// of the lib and `@types` declaration files and the Effect sources (long
// JSDoc comments). At 12 a declaration file, such as lib.dom, fills its
// vectors without a `realloc` copy, and a source file grows them once.
// `end_parse` gives the unused capacity back.
const STORE_TEXT_BYTES_PER_SLOT: usize = 12;


/// `FileStore::unlink_children` on the link column `links`.
fn unlink_children(links: &mut [SlotLinks], index: usize) {
    let mut child = std::mem::replace(&mut links[index].first_child, LINK_NONE);
    if child == LINK_NONE {
        return;
    }
    while child != LINK_END {
        let entry = &mut links[child as usize];
        child = std::mem::replace(&mut entry.next_sibling, LINK_NONE);
        debug_assert_ne!(child, LINK_NONE, "R2-5 chain without an end");
    }
}

/// `FileStore::link_child` on the link column `links`.
#[inline]
fn link_child(links: &mut [SlotLinks], parent: usize, last: &mut u32, child: usize) -> bool {
    if links[child].next_sibling != LINK_NONE {
        return false;
    }
    links[child].next_sibling = LINK_END;
    let child = child as u32;
    if *last == LINK_END {
        links[parent].first_child = child;
    } else {
        links[*last as usize].next_sibling = child;
    }
    *last = child;
    true
}

// ──────────────────────────────────────────────────────────────────────
// Owned nodes of a freeable parse (lsshells M3c)
// ──────────────────────────────────────────────────────────────────────

thread_local! {
    /// Set while a freeable parse with owned nodes runs on this thread
    /// (`enter_freeable_parse`, `owned_nodes_enabled`): `new_file_store`
    /// makes a store that owns its astdata nodes.
    static FREEABLE_PARSE: Cell<bool> = const { Cell::new(false) };
    /// Set while any freeable parse runs on this thread
    /// (`is_freeable_parse`).
    static FREEABLE_VERSION_PARSE: Cell<bool> = const { Cell::new(false) };
}

/// Starts a freeable parse on this thread (`is_freeable_parse`). With
/// owned nodes on (`owned_nodes_enabled`) the stores that
/// `new_file_store` makes until the scope ends own their astdata nodes,
/// pending lists and parse lists (`OwnedAst`), so they are freed with the
/// store, not leaked in the AST arena. The language server parse cache and
/// the compiler host (`tsc --watch`, `goport_multiprog`) open it for a new
/// version of a published path (`ast::freeable_path`), the version that
/// gets a `FileVersion`.
// PORT: Go nodes are heap objects that the GC frees with their file. A
// static parse keeps its nodes in the leaked AST arena, so its node reads
// stay `'static`.
#[must_use]
pub fn enter_freeable_parse() -> FreeableParseScope {
    FreeableParseScope {
        previous: FREEABLE_PARSE.replace(owned_nodes_enabled()),
        previous_version: FREEABLE_VERSION_PARSE.replace(true),
    }
}

/// True when a freeable parse owns its nodes (lsshells M3c). Off, it is a
/// static parse (its nodes stay in the leaked AST arena, and its node shell
/// has a node column), as before M3c. `GOPORT_OWNED_NODES` is read once:
/// `0` is off, `1` is on; else it is on in a language server, API or
/// `tsc --watch` process only (`ast::frees_file_versions_by_default`),
/// where the freeable versions are. Another CLI process makes freeable
/// parses only with `GOPORT_FREE_FILE_VERSIONS=1` (`goport_multiprog`), and
/// they stay static there by default.
// PERF: on by default since lsshells M3g (root decision
// m3-owned-default-2026-09-29). Only with it on do 1000-edit sessions pass
// memory (M3f, mini-abf9: effect 0.46 MiB/edit and +480 MiB, off 1.23 and
// +1216, limits 1.02 and 686). It costs edit time: every node data and list
// read of the edited file is a pinned read (about 150,000 per edit on
// effect). M3g against R139 (pin B): session instructions +6.40% on effect
// and +4.68% on query-core with it on, +2.2% with it off; edit median +0.5
// to +2.1 ms on effect and +0.8 to +1.7 ms on query-core in 200-edit
// sessions, +0.9 ms (query-core) and +1.0 to +1.2 ms (effect) in 1000-edit
// sessions. The CLI costs (instructions: query +1.38%, hono +0.48%, zod
// +0.18%, effect +0.19%) come from the M3 merge, not from this switch. The
// planned fix of the read cost is the AST node records plan
// (`target/continuation-r97-goport/ast-design/study.md`).
fn owned_nodes_enabled() -> bool {
    static FLAG: OnceLock<Option<bool>> = OnceLock::new();
    let flag = *FLAG.get_or_init(|| match std::env::var("GOPORT_OWNED_NODES").as_deref() {
        Ok("0") => Some(false),
        Ok("1") => Some(true),
        _ => None,
    });
    flag.unwrap_or_else(crate::ast::frees_file_versions_by_default)
}

/// `enter_freeable_parse` with owned nodes on, whatever the flag, for the
/// unit tests of the owned stores.
#[cfg(test)]
pub(crate) fn enter_owned_parse() -> FreeableParseScope {
    FreeableParseScope {
        previous: FREEABLE_PARSE.replace(true),
        previous_version: FREEABLE_VERSION_PARSE.replace(true),
    }
}

/// The scope of `enter_freeable_parse`.
pub struct FreeableParseScope {
    previous: bool,
    previous_version: bool,
}

impl Drop for FreeableParseScope {
    fn drop(&mut self) {
        FREEABLE_PARSE.set(self.previous);
        FREEABLE_VERSION_PARSE.set(self.previous_version);
    }
}

/// True while a freeable parse runs on this thread (`enter_freeable_parse`),
/// with or without owned nodes: the parse of a new version of an edited
/// file, which leaks nothing per version (`parse_source_file`).
#[must_use]
pub fn is_freeable_parse() -> bool {
    FREEABLE_VERSION_PARSE.get()
}

/// Cells per chunk of `OwnedAst::chunks`.
// PERF: 256 cells of 16 bytes (`NodeData`, astmem1 P1) is a 4 KiB chunk,
// one small size class of jemalloc.
const OWNED_CHUNK: usize = 256;

/// `OwnedAst::cell_of` of a slot that has no owned node: the nil slot, an
/// alias slot, or a node slot with a static node (a shared name node,
/// `alloc_store_shared_name_node`, or a lib snapshot slot).
const NO_CELL: u32 = u32::MAX;

/// A sealed chunk of `OwnedAst::chunks` (up to `OWNED_CHUNK` cells), or
/// the flat table of every cell after the parse (`OwnedAst::flat`). A held
/// read shares it (`HeldStoreNode::Owned`).
pub type OwnedChunk = Arc<Vec<NodeData>>;

/// The number of node datas that the live freeable stores own
/// (`OwnedAst`), for tests.
static OWNED_NODES: AtomicUsize = AtomicUsize::new(0);

/// The number of node datas that the live stores of freeable parses own
/// (lsshells M3c). A freeable file version frees its nodes when it dies, so
/// this does not grow with the edits. Tests use it.
#[must_use]
pub fn owned_node_count() -> usize {
    OWNED_NODES.load(Ordering::Relaxed)
}




/// The data in a cell that an early seal skipped (`OwnedAst::finish`). No
/// slot names it. Its data box has no size, so it allocates nothing.
fn hole_data() -> NodeData {
    NodeData::Token(Box::new(crate::astdata::TokenData))
}

/// The cell of node `index` of chunk `chunk` (`OwnedAst::chunks`).
fn owned_cell(chunk: usize, index: usize) -> u32 {
    let cell = chunk
        .checked_mul(OWNED_CHUNK)
        .and_then(|first| u32::try_from(first + index).ok())
        .expect("too many owned nodes in one store");
    assert_ne!(cell, NO_CELL, "too many owned nodes in one store");
    cell
}

/// The data that `FileStore::nodes` holds for a slot whose data a freeable
/// parse owns (`OwnedAst`). It is never read as data.
fn owned_marker() -> &'static NodeData {
    static MARKER: OnceLock<&'static NodeData> = OnceLock::new();
    MARKER.get_or_init(|| Box::leak(Box::new(hole_data())))
}

/// U1 (e) for a freeable parse: a pending list that the store owns (what
/// `PendingList` and `PendingModifierList` are for a static parse).
#[derive(Debug)]
pub struct OwnedPending {
    /// Go `list.Loc` in astdata form (`ts_range`).
    range: crate::astdata::text::TextRange,
    /// The store ids of the nodes (`store_child_id`).
    nodes: Box<[crate::astdata::NodeId]>,
    /// The astdata bit (`NodeList::stored_trailing_comma`).
    has_trailing_comma: bool,
    /// Go `ModifiersToFlags(nodes)` of a modifier list; `None` for a node
    /// list.
    flags: Option<crate::astdata::ModifierFlags>,
}

/// The selector id of a pending list (`StoreList::sel_id`). The selector
/// ids of the list sites are below it (`ast::synthetic::SELECTOR_LIMIT`).
pub(crate) const PENDING_SEL: u32 = (1 << StoreList::SEL_BITS) - 1;

/// A list of a store that owns its astdata nodes (a freeable parse,
/// lsshells M3c), as a `NodeList`, `ModifierList` or `NodeSlice` names it:
/// the list field that list selector `sel` (`list_selector`) finds in the
/// data of cell `key` of store `file`, or pending list `key` of that store
/// when `sel` is `PENDING_SEL`. It is read at each use (`with_store_list`),
/// so it keeps no borrow. 12 bytes, so the list handles stay 16 bytes.
// PERF: lsshells M3f. The handle also keeps what the hot list reads need
// and a list never changes (`sel`: the Go nil marker, the astdata
// `has_trailing_comma` bit and the length), so `NodeList::is_nil`,
// `NodeList::nodes` and `NodeSlice::len` read no store.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreList {
    file: u32,
    key: u32,
    /// Bits 0 to 9: the selector id, or `PENDING_SEL`. Bit 10: the range
    /// is the Go nil marker (`is_nil_list_range`). Bit 11: the astdata
    /// `has_trailing_comma` bit. Bits 12 to 31: the length, or `LEN_MAX`
    /// for a longer list (then read at each use).
    sel: u32,
}

impl StoreList {
    const SEL_BITS: u32 = 10;
    const NIL_BIT: u32 = 1 << 10;
    const COMMA_BIT: u32 = 1 << 11;
    const LEN_SHIFT: u32 = 12;
    const LEN_MAX: u32 = u32::MAX >> Self::LEN_SHIFT;


    /// The store id of the list.
    #[inline]
    #[must_use]
    pub fn file(self) -> usize {
        self.file as usize
    }

    /// The selector id, or `PENDING_SEL` for a pending list.
    #[inline]
    fn sel_id(self) -> u32 {
        self.sel & PENDING_SEL
    }

    /// True when the list is the Go `nil` marker (`NIL_LIST_POS`): Go
    /// `list == nil`.
    #[inline]
    #[must_use]
    pub fn is_nil_marker(self) -> bool {
        self.sel & Self::NIL_BIT != 0
    }

    /// The astdata `has_trailing_comma` bit.
    #[inline]
    #[must_use]
    pub fn stored_trailing_comma(self) -> bool {
        self.sel & Self::COMMA_BIT != 0
    }



    /// True when the list has no nodes.
    #[inline]
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.sel >> Self::LEN_SHIFT == 0
    }

    /// A slot of a direct-mapped cache of `slots` entries (a power of two)
    /// for this handle.
    #[inline]
    #[must_use]
    pub fn hash_slot(self, slots: usize) -> usize {
        let key = (u64::from(self.file) << 32 | u64::from(self.key)) ^ u64::from(self.sel) << 20;
        (key.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 48) as usize & (slots - 1)
    }
}

/// A list of a store that owns its nodes, read in place
/// (`with_store_list`): a list field of node data (`AnyList`) or a pending
/// list.
#[derive(Clone, Copy)]
pub enum StoreListView<'a> {
    Data(AnyList<'a>),
    Pending(&'a OwnedPending),
}

impl<'a> StoreListView<'a> {
    /// Go `list.Loc` in astdata form.
    #[must_use]
    pub fn range(self) -> crate::astdata::text::TextRange {
        match self {
            Self::Data(l) => l.nodes().range,
            Self::Pending(p) => p.range,
        }
    }


    /// The astdata `has_trailing_comma` bit.
    #[must_use]
    pub fn has_trailing_comma(self) -> bool {
        match self {
            Self::Data(l) => l.nodes().has_trailing_comma,
            Self::Pending(p) => p.has_trailing_comma,
        }
    }

    /// Go `modifiers.ModifierFlags` of a modifier list. Panics on a node
    /// list.
    #[must_use]
    pub fn modifier_flags(self) -> ModifierFlags {
        match self {
            Self::Data(l) => ModifierFlags(l.modifiers().flags.0),
            Self::Pending(p) => ModifierFlags(p.flags.expect("a pending modifier list").0),
        }
    }

    /// The address of the list, for Go pointer compares and keys
    /// (`NodeList::list_ptr`). It stays the same while the store lives.
    #[must_use]
    pub fn ptr(self) -> *const () {
        match self {
            Self::Data(l) => std::ptr::from_ref(l.nodes()).cast(),
            Self::Pending(p) => std::ptr::from_ref(p).cast(),
        }
    }


}






/// What `static_store_node` found for a store node.
pub enum StaticNode {
    /// The node has `'static` data.
    Static(&'static NodeData),
    /// The node has no `'static` data: a node of a freeable file version or
    /// an owned node of an unpublished store. Read it with
    /// `with_scoped_store_node` (lsshells M3c).
    Scoped,
    /// No store of this thread or of the registry has the node.
    NoStore,
}










/// A list field that a list selector found in the data of a store node
/// with no `'static` node (`scoped_store_list_of`).
#[derive(Clone, Copy)]
pub enum ScopedList {
    /// The node is static after all (a shared name node, or a freeable
    /// file version whose parse was not freeable): a `'static` list.
    Static(AnyList<'static>),
    /// A list that the store owns.
    Store(StoreList),
}

/// Entries of `HOT_LISTS`.
const HOT_LIST_SLOTS: usize = 256;

/// One entry of `HOT_LISTS`: a node, a selector id and what
/// `scoped_store_list_of` gave for them.
#[derive(Clone, Copy)]
struct HotListEntry {
    node: Node,
    sel_id: u32,
    found: Option<Option<ScopedList>>,
}

thread_local! {
    /// The last `scoped_store_list_of` answers for nodes of hot file
    /// versions (`file_version::is_hot`), by node and selector id, direct
    /// mapped (lsshells M3f). A published version never changes and a file
    /// id is never reused, so an entry stays true; a node of a dead
    /// version is never hot, so its entries are not read.
    // PERF: the checker asks for the same lists of the edited file many
    // times (parameters, type parameters, members).
    static HOT_LISTS: RefCell<[HotListEntry; HOT_LIST_SLOTS]> = const {
        RefCell::new(
            [HotListEntry {
                node: Node::NIL,
                sel_id: 0,
                found: None,
            }; HOT_LIST_SLOTS],
        )
    };
}






// ──────────────────────────────────────────────────────────────────────
// Detached stores (parse workers)
// ──────────────────────────────────────────────────────────────────────





/// Maps the node handles of an adopted detached store to its real id.
#[derive(Clone, Copy, Debug)]
pub struct StoreRemap {
    from: usize,
    to: usize,
}

impl StoreRemap {
    /// The real store id.
    #[must_use]
    pub fn store(self) -> usize {
        self.to
    }

    /// `n` with the provisional store id replaced by the real one. Other
    /// handles (nil, other stores) do not change.
    #[inline]
    #[must_use]
    pub fn node(self, n: Node) -> Node {
        if n.is_some() && n.file_index() == self.from {
            handle(self.to, slot_index(n) as u32)
        } else {
            n
        }
    }
}


// ──────────────────────────────────────────────────────────────────────
// File registry
// ──────────────────────────────────────────────────────────────────────


#[cold]
#[inline(never)]
fn not_published(file: usize) -> ! {
    panic!("file {file} is not published")
}



















// Go: ast/ast.go:2745 (*SourceFile).resolveJSDoc




/// `FileStore::publish_work` of one store without tables to make: about
/// the cost of its map drops, in slots.
const PUBLISH_WORK_PER_STORE: usize = 32;
/// `publish_stores` uses scoped threads from this much work on. Below it
/// the thread starts cost more than they save.
const PARALLEL_PUBLISH_WORK: usize = 1 << 17;
/// Threads for `publish_stores`, the calling thread included.
const PUBLISH_THREADS: usize = 4;











// ──────────────────────────────────────────────────────────────────────
// Read hooks for node.rs and core.rs
// ──────────────────────────────────────────────────────────────────────
























/// The node data of a node slot. Panics on the nil slot and alias slots.
#[inline]
fn slot_node(slot: Option<&'static NodeData>) -> &'static NodeData {
    slot.expect("store handle does not name a node slot")
}







/// The node for slot `index` of store `file`, whose records are `records`:
/// the slot itself when it holds a node, else the target in its record
/// (`NodeRecord::NO_NODE`).
#[inline]
fn record_resolve(file: usize, index: usize, records: &[NodeRecord]) -> Node {
    let record = &records[index];
    if record.is_node() {
        handle(file, index as u32)
    } else {
        record.target_node()
    }
}


















/// R2-5: the iterator of `frozen_store_children` and
/// `VersionChildLinks::children`. A chain holds only node slots of its
/// store, so each child is the handle of its slot.
#[derive(Clone, Copy, Debug)]
pub struct StoreChildren<'a> {
    file: usize,
    links: &'a [SlotLinks],
    /// The next child, `LINK_END` at the end.
    next: u32,
}

impl Iterator for StoreChildren<'_> {
    type Item = Node;

    #[inline]
    fn next(&mut self) -> Option<Node> {
        if self.next == LINK_END {
            return None;
        }
        let index = self.next;
        self.next = self.links[index as usize].next_sibling;
        Some(handle(self.file, index))
    }
}


/// U4 (CH7): what `frozen_find_ancestor` found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AncestorWalk {
    /// The first node for which the callback was true, or nil when the walk
    /// passed the root.
    Found(Node),
    /// The walk reached this parent, which is not in the store. Walk on
    /// from it.
    Next(Node),
}





/// U1 (c): how `Node::new(file, id)` maps the child ids of one published
/// store, read once for a loop over many ids (`NodeSliceIter`).
#[derive(Clone, Copy, Debug)]
pub enum FrozenIds {
    /// A store with alias slots: its records (`record_resolve`), and `base`
    /// as for `Direct`.
    Records {
        base: u64,
        records: &'static [NodeRecord],
    },
    /// An alias-free store: id 0 is nil (what slot 0 resolves to), and
    /// any other id is the handle `base | (id + 1)`, `base` = `file << 32`.
    Direct { base: u64 },
}

impl FrozenIds {
    /// `Node::new(file, id)` for the store this value was read for.
    #[inline]
    #[must_use]
    pub fn node(self, id: crate::astdata::NodeId) -> Node {
        match self {
            FrozenIds::Records { base, records } => {
                let record = &records[id.index()];
                if record.is_node() {
                    Node(base | (id.index() as u64 + 1))
                } else {
                    record.target_node()
                }
            }
            FrozenIds::Direct { base } => {
                if id.index() == NIL_SLOT as usize {
                    Node::NIL
                } else {
                    Node(base | (id.index() as u64 + 1))
                }
            }
        }
    }
}



// ──────────────────────────────────────────────────────────────────────
// Go field writes during the parse
// ──────────────────────────────────────────────────────────────────────





/// R2-5: Go `child.Parent = parent` for each child of `parent` in Go
/// `ForEachChild` order (parser.go `overrideParentInImmediateChildren`),
/// which also makes the chain of `parent` in the link column (`SlotLinks`).
/// `new` starts the chain; `set_parent` sets the parent of one child and
/// links it, in the same store access.
pub struct StoreChildLinks {
    parent: Node,
    /// The last child linked so far, `LINK_END` before the first.
    last: u32,
    /// True while the chain of `parent` is made. False when `parent` is not
    /// a node of an unfinished store of this thread, or a child could not be
    /// linked (the chain of `parent` is then unknown).
    linking: bool,
}

impl StoreChildLinks {

}








// ──────────────────────────────────────────────────────────────────────
// Lib parse snapshot (R3-1, frontend/parser/lib_parse_snapshot.rs)
// ──────────────────────────────────────────────────────────────────────

/// R3-1: one node slot of a lib parse snapshot: what the parse left in the
/// slot when it froze the file, besides what the freeze and the U1 (b) and
/// U4 kids entries make from it. A snapshot store has no alias slot, and slot
/// 0 (nil) is not in the snapshot.
pub(crate) struct LibParseSlot {
    pub(crate) kind: SyntaxKind,
    pub(crate) flags: NodeFlags,
    pub(crate) loc: TextRange,
    /// The slot of the Go parent, 0 for nil.
    pub(crate) parent: u32,
    /// The R2-5 links (`SlotLinks`): a slot, `Self::LINK_END` or
    /// `Self::LINK_NONE`.
    pub(crate) first_child: u32,
    pub(crate) next_sibling: u32,
    /// The node data. `None`: the slot points at the shared name data of
    /// `kind` (S1, `alloc_store_shared_name_node`).
    pub(crate) data: Option<NodeData>,
    /// The name word and the keyword bit (`slot_text_name`):
    /// `Name::default()` and false for a kind other than Identifier and
    /// PrivateIdentifier.
    pub(crate) name: Name,
    pub(crate) text_is_keyword: bool,
}

impl LibParseSlot {
    pub(crate) const LINK_END: u32 = LINK_END;
    pub(crate) const LINK_NONE: u32 = LINK_NONE;
}




/// R3-1, tests: one slot of a finished store as a snapshot keeps it
/// (`LibParseSlot`), with the data by reference.
#[cfg(test)]
pub(crate) struct LibParseSlotView {
    pub(crate) kind: SyntaxKind,
    pub(crate) flags: NodeFlags,
    pub(crate) loc: TextRange,
    pub(crate) parent: u32,
    pub(crate) first_child: u32,
    pub(crate) next_sibling: u32,
    pub(crate) data: &'static NodeData,
    /// The slot points at the shared name data of its kind (S1).
    pub(crate) shared_name: bool,
    pub(crate) name: Name,
    pub(crate) text_is_keyword: bool,
}

/// R3-1, tests: the slots after slot 0 of store `file`, a store of this
/// thread whose parse is finished and that is not published. An error when
/// a snapshot cannot keep the store: an alias slot, a parent in another
/// store, or node data that does not fit the slot kind.
#[cfg(test)]
pub(crate) fn lib_parse_slot_views(file: usize) -> Result<Vec<LibParseSlotView>, String> {
    with_store(file, |s| {
        if !s.frozen || !s.facts_made || s.links.len() != s.records.len() {
            return Err("the store is not frozen".into());
        }
        if !s.aliases.is_empty() {
            return Err("the store has alias slots".into());
        }
        let mut views = Vec::with_capacity(s.records.len());
        for index in 1..s.records.len() {
            let record = &s.records[index];
            let kind = s.kinds[index];
            let data = s.nodes[index].ok_or_else(|| format!("slot {index} is not a node slot"))?;
            if !data.matches_syntax_kind(kind) {
                return Err(format!("slot {index}: the node data does not fit {kind:?}"));
            }
            let parent = match (record.local_parent(), record.parent_code()) {
                (_, 0) => NIL_SLOT,
                (Some(parent), _) => parent as u32,
                _ => return Err(format!("slot {index}: a parent in another store")),
            };
            let shared_name = is_name_kind(kind) && std::ptr::eq(data, shared_name_node(kind));
            let links = s.links[index];
            views.push(LibParseSlotView {
                kind,
                flags: record.flags(),
                loc: record.loc(),
                parent,
                first_child: links.first_child,
                next_sibling: links.next_sibling,
                data,
                shared_name,
                name: s.kids[index].text_name(kind),
                text_is_keyword: record.has_bit(NodeRecord::TEXT_IS_KEYWORD),
            });
        }
        Ok(views)
    })
}

/// R3-1, tests: one line for each part of store `file` (a store of this
/// thread whose parse is finished) that the snapshot load and the freeze
/// make, in a form that does not depend on the store id, so the store of a
/// live parse and of a snapshot load can be compared. The node data is not
/// in it (the snapshot test compares it field by field).
#[cfg(test)]
pub(crate) fn lib_parse_store_dump(file: usize) -> Vec<String> {
    let local = |n: Node| {
        if n.is_nil() {
            "nil".to_string()
        } else if n.file_index() == file {
            format!("#{}", slot_index(n))
        } else {
            format!("{n:?}")
        }
    };
    with_store(file, |s| {
        let mut jsdoc: Vec<String> = s
            .jsdoc_cache
            .iter()
            .map(|(&n, jsdocs)| {
                let list: Vec<String> = jsdocs.iter().map(|&j| local(j)).collect();
                format!("{} {list:?}", local(n))
            })
            .collect();
        jsdoc.sort();
        let diagnostics: Vec<String> = s
            .diagnostics
            .iter()
            .map(|d| {
                format!(
                    "{} {} {} {} {} {:?}",
                    local(d.file),
                    d.pos,
                    d.end,
                    d.code,
                    d.message.key(),
                    d.message_args
                )
            })
            .collect();
        let mut lines = vec![
            format!("file {} text {}", s.file_name, s.text.len()),
            format!(
                "frozen {} facts {} root_slot {} root {} aliases {}",
                s.frozen,
                s.facts_made,
                s.root_slot,
                local(s.root),
                s.aliases.len(),
            ),
            format!("parser_flags {:?}", s.parser_flags),
            format!("jsdoc {jsdoc:?}"),
            format!(
                "lazy {:?} lazy_cache {}",
                s.lazy_js_doc,
                s.lazy_jsdoc_cache.len()
            ),
            format!(
                "variant {:?} diagnostics {diagnostics:?}",
                s.language_variant
            ),
            format!("facts {:?} bind {:?}", s.facts, s.bind_estimate),
            format!(
                "build {} {} names {}",
                s.build_links.len(),
                s.identifier_names.len(),
                s.ecma_line_starts.get().is_some()
            ),
            format!(
                "columns {} {} {} {}",
                s.records.len(),
                s.kids.len(),
                s.nodes.len(),
                s.links.len()
            ),
        ];
        for index in 0..s.records.len() {
            let kind = s.kinds[index];
            let shared = is_name_kind(kind)
                && s.nodes[index].is_some_and(|data| std::ptr::eq(data, shared_name_node(kind)));
            let record = &s.records[index];
            let kids = &s.kids[index];
            lines.push(format!(
                "{index}: record {:?} {} {:?} {:?} {} bind {} node {} shared {shared} kids {:?} {:?} {} links {:?}",
                kind,
                record.bits(),
                record.flags(),
                record.loc(),
                local(record.header(kind, file, &s.foreign_parents).parent),
                record.bind.load(Ordering::Relaxed),
                s.nodes[index].is_some(),
                kids.children(),
                kids.text_name(kind).as_str(),
                kids.modifier_bits(kind),
                s.links.get(index),
            ));
        }
        lines
    })
}

// ──────────────────────────────────────────────────────────────────────
// Allocation (used by factory.rs through `NodeFactory::for_file`)
// ──────────────────────────────────────────────────────────────────────



/// S1: the one node data that every store Identifier (or
/// PrivateIdentifier, by `kind`) made by `alloc_store_shared_name_node`
/// points at. Its data is the Go factory payload with the default fields:
/// no flow node (the binder keeps flow nodes in its tables) and an empty
/// text (the name column holds the text).
fn shared_name_node(kind: SyntaxKind) -> &'static NodeData {
    static IDENTIFIER: OnceLock<&'static NodeData> = OnceLock::new();
    static PRIVATE_IDENTIFIER: OnceLock<&'static NodeData> = OnceLock::new();
    let leak = |data| -> &'static NodeData { Box::leak(Box::new(data)) };
    match kind {
        SyntaxKind::Identifier => *IDENTIFIER.get_or_init(|| {
            leak(NodeData::Identifier(Box::new(
                crate::astdata::IdentifierData {
                    flow_node: None,
                    text: String::new(),
                },
            )))
        }),
        SyntaxKind::PrivateIdentifier => *PRIVATE_IDENTIFIER.get_or_init(|| {
            leak(NodeData::PrivateIdentifier(Box::new(
                crate::astdata::PrivateIdentifierData {
                    text: String::new(),
                },
            )))
        }),
        _ => panic!("{kind:?} is not a name kind"),
    }
}

/// S1, debug builds: panics unless `data` (the payload the factory builds
/// for a store name node of kind `kind`) equals the data of the shared node
/// of `kind`. The struct patterns name every field, so a new astdata field
/// does not compile here until it is checked.
#[cfg(debug_assertions)]
pub fn debug_assert_shared_name_data(kind: SyntaxKind, data: &NodeData) {
    let same = match (data, shared_name_node(kind)) {
        (NodeData::Identifier(a), NodeData::Identifier(b)) => {
            let crate::astdata::IdentifierData { flow_node, text } = &**a;
            *flow_node == b.flow_node && *text == b.text
        }
        (NodeData::PrivateIdentifier(a), NodeData::PrivateIdentifier(b)) => {
            let crate::astdata::PrivateIdentifierData { text } = &**a;
            *text == b.text
        }
        _ => false,
    };
    assert!(
        same,
        "S1: the factory payload of a store {kind:?} differs from the shared node"
    );
}











/// A Go `core.TextRange` in astdata form (`-1` is stored as `u32::MAX`).
fn ts_range(loc: TextRange) -> crate::astdata::text::TextRange {
    crate::astdata::text::TextRange {
        start: crate::astdata::text::TextPos::new(loc.pos() as u32),
        end: crate::astdata::text::TextPos::new(loc.end() as u32),
    }
}


/// U1 (e): a list that the parser made in a store and that no node data
/// holds yet: what a pending `NodeList` handle names. Its ids live in the
/// AST bump arena. `store_list_value` builds the astdata list from it, once
/// for each node data that stores it.
#[derive(Debug)]
pub struct PendingList {
    /// Go `list.Loc` in astdata form (`ts_range`).
    pub(crate) range: crate::astdata::text::TextRange,
    /// The store ids of the nodes (`store_child_id`).
    pub(crate) nodes: &'static [crate::astdata::NodeId],
    /// The astdata bit (`NodeList::stored_trailing_comma`).
    pub(crate) has_trailing_comma: bool,
}

impl PendingList {

    /// The astdata list with the same range, ids and bit.
    fn to_ts(&self) -> crate::astdata::NodeList {
        crate::astdata::NodeList {
            range: self.range,
            nodes: self.nodes.to_vec(),
            has_trailing_comma: self.has_trailing_comma,
        }
    }
}

/// U1 (e): the `PendingList` of a modifier list, with Go
/// `ModifiersToFlags(nodes)`.
#[derive(Debug)]
pub struct PendingModifierList {
    pub(crate) list: PendingList,
    pub(crate) flags: crate::astdata::ModifierFlags,
}

impl PendingModifierList {
    /// The astdata modifier list with the same list and flags.
    fn to_ts(&self) -> crate::astdata::ModifierList {
        crate::astdata::ModifierList {
            list: self.list.to_ts(),
            flags: self.flags,
        }
    }
}







/// True when `l` is the Go `nil` marker of a required list field.
#[inline]
#[must_use]
pub fn is_nil_list_marker(l: &crate::astdata::NodeList) -> bool {
    is_nil_list_range(&l.range)
}

/// True when `range` is the range of a Go `nil` marker list
/// (`NIL_LIST_POS`).
#[inline]
#[must_use]
pub fn is_nil_list_range(range: &crate::astdata::text::TextRange) -> bool {
    range.start.get() == NIL_LIST_POS && range.end.get() == NIL_LIST_POS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_factory_writes_into_the_file_store() {
        let file = new_file_store("/a.ts", "a.b");
        let f = NodeFactory::for_file(file);
        let a = f.new_identifier("a");
        let b = f.new_identifier("b");
        set_node_loc(a, TextRange::new(0, 1));
        let q = f.new_qualified_name(a, b);
        set_node_parent(a, q);
        set_node_flags(q, NodeFlags::AMBIENT);

        assert_eq!(q.file_index(), file);
        assert_eq!(q.kind(), SyntaxKind::QualifiedName);
        assert_eq!(q.left(), a);
        assert_eq!(a.parent(), q);
        assert_eq!(a.loc(), TextRange::new(0, 1));
        assert_eq!(q.loc(), TextRange::undefined());
        assert_eq!(q.flags(), NodeFlags::AMBIENT);
        assert_eq!(source_file_text(q), "a.b");

        // A synthetic child is an alias slot that resolves to the same node.
        let s = NodeFactory::new().new_identifier("s");
        let q2 = f.new_qualified_name(q, s);
        assert_eq!(q2.right(), s);

        // Go nil in a required list field reads back as nil.
        let decls = f.new_variable_declaration_list(NodeList::NIL, NodeFlags::NONE);
        assert!(decls.declarations().is_nil());
        let list = f.new_node_list_with_loc(&[a], TextRange::new(0, 3));
        assert_eq!(list.loc(), TextRange::new(0, 3));
        assert_eq!(list.nodes().get(0), a);

        freeze_file_store(file);
        assert!(std::panic::catch_unwind(|| set_node_parent(b, q)).is_err());
    }

    #[test]
    fn freeze_makes_the_name_and_modifier_columns() {
        let file = new_file_store("/c.ts", "");
        let f = NodeFactory::for_file(file);
        let keyword = f.new_identifier("await");
        let plain = f.new_identifier("b");
        let private = f.new_private_identifier("#c");
        let export = f.new_modifier(SyntaxKind::ExportKeyword);
        let declare = f.new_modifier(SyntaxKind::DeclareKeyword);
        let modifiers = f.new_modifier_list(&[export, declare]);
        let list = f.new_variable_declaration_list(NodeList::NIL, NodeFlags::NONE);
        let statement = f.new_variable_statement(modifiers, list);
        freeze_file_store(file);

        with_store(file, |s| {
            let names = |n: Node| s.slot_text_name_of(slot_index(n));
            assert_eq!(names(keyword), Name::from("await"));
            assert_eq!(names(plain), Name::from("b"));
            assert_eq!(names(private), Name::from("#c"));
            assert_eq!(names(statement), Name::default());
            let is_keyword =
                |n: Node| s.records[slot_index(n)].has_bit(NodeRecord::TEXT_IS_KEYWORD);
            assert!(is_keyword(keyword));
            assert!(!is_keyword(plain));
            assert!(!is_keyword(private));
            let bits = |n: Node| {
                let i = slot_index(n);
                s.kids[i].modifier_bits(s.kinds[i])
            };
            assert_eq!(
                bits(statement),
                (ModifierFlags::EXPORT | ModifierFlags::AMBIENT).0
            );
            assert_eq!(bits(list), 0);
            assert_eq!(bits(export), 0);
        });
    }

    #[test]
    fn replaced_data_updates_the_name_and_modifier_columns() {
        let file = new_file_store("/d.ts", "");
        let f = NodeFactory::for_file(file);
        let id = f.new_identifier("a");
        let export = f.new_modifier(SyntaxKind::ExportKeyword);
        let modifiers = f.new_modifier_list(&[export]);
        let list = f.new_variable_declaration_list(NodeList::NIL, NodeFlags::NONE);
        let statement = f.new_variable_statement(modifiers, list);
        with_store(file, |s| {
            assert_eq!(s.slot_text_name_of(slot_index(id)), Name::from("a"));
            assert_ne!(
                s.kids[slot_index(statement)].modifier_bits(SyntaxKind::VariableStatement),
                0
            );
        });

        let mut data = store_ast_node(id).clone();
        if let NodeData::Identifier(d) = &mut data {
            d.text = "await".to_string();
        }
        replace_store_node_data(id, data);
        let mut data = store_ast_node(statement).clone();
        if let NodeData::VariableStatement(d) = &mut data {
            d.modifiers = None;
        }
        replace_store_node_data(statement, data);
        freeze_file_store(file);

        with_store(file, |s| {
            assert_eq!(s.slot_text_name_of(slot_index(id)), Name::from("await"));
            assert!(s.records[slot_index(id)].has_bit(NodeRecord::TEXT_IS_KEYWORD));
            assert_eq!(
                s.kids[slot_index(statement)].modifier_bits(SyntaxKind::VariableStatement),
                0
            );
        });
    }

    #[test]
    fn slots_get_the_children_column() {
        let file = new_file_store("/g.ts", "");
        let f = NodeFactory::for_file(file);
        let id = |n: Node| slot_index(n) as u32;
        let a = f.new_identifier("a");
        let b = f.new_identifier("b");
        let access = f.new_property_access_expression(a, Node::NIL, b, NodeFlags::NONE);
        let statement = f.new_expression_statement(access);
        let question = f.new_token(SyntaxKind::QuestionToken);
        let p = f.new_identifier("p");
        let signature = f.new_property_signature_declaration(
            ModifierList::NIL,
            p,
            question,
            Node::NIL,
            Node::NIL,
        );
        let x = f.new_identifier("x");
        let parameter = f.new_parameter_declaration(
            ModifierList::NIL,
            Node::NIL,
            x,
            Node::NIL,
            Node::NIL,
            Node::NIL,
        );
        let qualified = f.new_qualified_name(a, b);
        // A synthetic child gets an alias slot.
        let s = NodeFactory::new().new_identifier("s");
        let aliased = f.new_property_access_expression(s, Node::NIL, b, NodeFlags::NONE);
        // A reparser write replaces a child that the column holds.
        let c = f.new_identifier("c");
        let mut data = store_ast_node(statement).clone();
        if let NodeData::ExpressionStatement(d) = &mut data {
            d.expression = store_child_id(file, c);
        }
        replace_store_node_data(statement, data);
        freeze_file_store(file);

        with_store(file, |st| {
            let entry = |n: Node| st.kids[slot_index(n)].children();
            let expression = SlotChildren::TAG_EXPRESSION;
            // C2: none of these nodes has a type, an initializer or type
            // arguments.
            let untyped = |e: SlotChildren| e.with_typed(Some((SlotChildren::TYPED_TYPE, 0)), true);
            let no_type =
                |e: SlotChildren| e.with_typed(Some((SlotChildren::TYPED_INITIALIZER, 0)), true);
            assert_eq!(
                entry(access),
                untyped(SlotChildren::new(Some(id(b)), Some((expression, id(a)))))
            );
            assert_eq!(
                entry(statement),
                untyped(SlotChildren::new(Some(0), Some((expression, id(c)))))
            );
            assert_eq!(
                entry(signature),
                no_type(SlotChildren::new(
                    Some(id(p)),
                    Some((SlotChildren::TAG_POSTFIX, id(question)))
                ))
            );
            assert_eq!(
                entry(parameter),
                no_type(SlotChildren::new(
                    Some(id(x)),
                    Some((SlotChildren::TAG_QUESTION, 0))
                ))
            );
            assert_eq!(
                entry(a),
                untyped(SlotChildren::new(Some(0), Some((expression, 0))))
            );
            assert_eq!(entry(qualified), untyped(SlotChildren::UNKNOWN));
            // The nil slot and alias slots are unknown; an aliased child has
            // its alias slot id, as in the node data.
            assert_eq!(st.kids[NIL_SLOT as usize].children(), SlotChildren::UNKNOWN);
            let alias = st.aliases[&s];
            assert_eq!(st.kids[alias as usize].children(), SlotChildren::UNKNOWN);
            assert_eq!(
                entry(aliased),
                untyped(SlotChildren::new(Some(id(b)), Some((expression, alias))))
            );
            st.debug_check_kids();
        });
    }

    #[test]
    fn slots_get_the_typed_column() {
        let file = new_file_store("/i.ts", "");
        let f = NodeFactory::for_file(file);
        let id = |n: Node| slot_index(n) as u32;
        let x = f.new_identifier("x");
        let t = f.new_keyword_type_node(SyntaxKind::NumberKeyword);
        let one = f.new_numeric_literal("1", TokenFlags::NONE);
        let both = f.new_variable_declaration(x, Node::NIL, t, one);
        let only_initializer = f.new_variable_declaration(x, Node::NIL, Node::NIL, one);
        let only_type = f.new_variable_declaration(x, Node::NIL, t, Node::NIL);
        let name = f.new_identifier("T");
        let plain_reference = f.new_type_reference_node(name, NodeList::NIL);
        let generic_reference = f.new_type_reference_node(name, f.new_node_list(&[t]));
        let call = f.new_call_expression(
            x,
            Node::NIL,
            NodeList::NIL,
            f.new_node_list(&[]),
            NodeFlags::NONE,
        );
        freeze_file_store(file);

        with_store(file, |st| {
            let typed = |n: Node| {
                let entry = st.kids[slot_index(n)].children();
                (entry.typed_parts(), entry.has_no_type_arguments())
            };
            // Both set: the column holds the type only.
            assert_eq!(
                typed(both),
                ((SlotChildren::TYPED_TYPE_WITH_INITIALIZER, id(t)), true)
            );
            // One set: the column holds it, and the other reads as nil.
            assert_eq!(
                typed(only_initializer),
                ((SlotChildren::TYPED_INITIALIZER, id(one)), true)
            );
            assert_eq!(typed(only_type), ((SlotChildren::TYPED_TYPE, id(t)), true));
            assert_eq!(
                typed(plain_reference),
                ((SlotChildren::TYPED_TYPE_NAME, id(name)), true)
            );
            assert_eq!(
                typed(generic_reference),
                ((SlotChildren::TYPED_TYPE_NAME, id(name)), false)
            );
            assert_eq!(typed(call), ((SlotChildren::TYPED_TYPE, 0), true));
            assert_eq!(typed(x), ((SlotChildren::TYPED_TYPE, 0), true));
            // Unknown never claims a nil type argument list.
            assert!(!SlotChildren::UNKNOWN.has_no_type_arguments());
            st.debug_check_kids();
        });
    }

    /// R2-5: the chain of `n` in the build links of its store, or `None`
    /// when it is not known.
    fn build_chain(n: Node) -> Option<Vec<Node>> {
        let file = n.file_index();
        with_store(file, |s| {
            let mut next = s.build_links[slot_index(n)].first_child;
            if next == LINK_NONE {
                return None;
            }
            let mut chain = Vec::new();
            while next != LINK_END {
                chain.push(handle(file, next));
                next = s.build_links[next as usize].next_sibling;
            }
            Some(chain)
        })
    }

    /// R2-5: what `override_parent_in_immediate_children` does for `parent`.
    fn link_children(parent: Node) {
        let mut links = StoreChildLinks::new(parent);
        parent.for_each_child(|child| {
            if !links.set_parent(child) {
                set_node_parent(child, parent);
            }
            false
        });
    }

    #[test]
    fn child_links_follow_the_parent_writes() {
        let file = new_file_store("/h.ts", "");
        let f = NodeFactory::for_file(file);
        let a = f.new_identifier("a");
        let b = f.new_identifier("b");
        let q = f.new_qualified_name(a, b);
        link_children(q);
        assert_eq!(build_chain(q), Some(vec![a, b]));
        assert_eq!(a.parent(), q);
        // Setting the parents again makes the same chain.
        link_children(q);
        assert_eq!(build_chain(q), Some(vec![a, b]));
        // A leaf has an empty chain; a node the parser did not finish has
        // none.
        link_children(a);
        assert_eq!(build_chain(a), Some(Vec::new()));
        assert_eq!(build_chain(b), None);
        // A second parent of linked children is unknown; the first keeps
        // its chain, as Go keeps them in its fields.
        let q2 = f.new_qualified_name(a, b);
        link_children(q2);
        assert_eq!(build_chain(q2), None);
        assert_eq!(build_chain(q), Some(vec![a, b]));
        assert_eq!(a.parent(), q2);
        // A data write frees the chain, so the children can be linked again.
        replace_store_node_data(q, store_ast_node(q).clone());
        assert_eq!(build_chain(q), None);
        link_children(q2);
        assert_eq!(build_chain(q2), Some(vec![a, b]));
        // A child twice in one node, or a child of another store, makes the
        // chain unknown and frees the children linked before it.
        let twice = f.new_qualified_name(q, q);
        link_children(twice);
        assert_eq!(build_chain(twice), None);
        let synthetic = NodeFactory::new().new_identifier("s");
        let mixed = f.new_qualified_name(q, synthetic);
        link_children(mixed);
        assert_eq!(build_chain(mixed), None);
        assert_eq!(synthetic.parent(), mixed);
        link_children(twice);
        assert_eq!(build_chain(twice), None);
        freeze_file_store(file);
        with_store(file, |s| {
            assert_eq!(s.links.len(), s.records.len());
            assert_eq!(s.links[slot_index(q2)].first_child, slot_index(a) as u32);
            assert_eq!(s.links[slot_index(a)].next_sibling, slot_index(b) as u32);
            assert_eq!(s.links[slot_index(b)].next_sibling, LINK_END);
        });
    }

    #[test]
    fn parsed_chains_equal_for_each_child() {
        use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
        let text = "function f(a: number, b = 1) { return a + b; }\n\
            let x: Array<string> = [1, 2].map(y => `${y}`);\n\
            type T = { a?: string } | number;\n\
            class C<U> extends Object { m(): U | undefined { return undefined; } }\n";
        let opts = SourceFileParseOptions {
            file_name: "/links.ts".to_string(),
            ..Default::default()
        };
        let parsed = parse_source_file(&opts, text, ScriptKind::TS);
        let file = parsed.store;
        let links = with_store(file, |s| s.links.clone());
        assert_eq!(links.len(), file_store_slot_count(file));
        let mut known = 0;
        for index in 1..links.len() as u32 {
            let mut next = links[index as usize].first_child;
            if next == LINK_NONE {
                continue;
            }
            known += 1;
            let mut chain = Vec::new();
            while next != LINK_END {
                chain.push(handle(file, next));
                next = links[next as usize].next_sibling;
            }
            let n = handle(file, index);
            assert_eq!(
                chain,
                n.iter_children().collect::<Vec<_>>(),
                "{:?}",
                n.kind()
            );
        }
        assert!(known > 0);
        // The parser links the root too.
        assert_ne!(links[slot_index(parsed.root)].first_child, LINK_NONE);
    }

    #[test]
    fn slot_children_tags_round_trip() {
        let entry = SlotChildren::new(Some(7), Some((SlotChildren::TAG_QUESTION, 9)));
        assert_eq!(entry.name, 7);
        assert_eq!(entry.other_parts(), (SlotChildren::TAG_QUESTION, 9));
        // An id that does not fit next to the tag is unknown.
        let big = SlotChildren::new(Some(1), Some((SlotChildren::TAG_POSTFIX, 1 << 30)));
        assert_eq!(big.other, SlotChildren::UNKNOWN_ID);
        assert_eq!(big.other_parts().0, SlotChildren::TAG_UNKNOWN);
        assert_eq!(SlotChildren::new(None, None), SlotChildren::UNKNOWN);
        // C2: the tag, the id and the type arguments bit round trip, and an
        // id that does not fit is unknown.
        let typed = entry.with_typed(Some((SlotChildren::TYPED_TYPE_NAME, 11)), true);
        assert_eq!(typed.typed_parts(), (SlotChildren::TYPED_TYPE_NAME, 11));
        assert!(typed.has_no_type_arguments());
        let big = entry.with_typed(Some((SlotChildren::TYPED_TYPE, 1 << 28)), false);
        assert_eq!(big.typed_parts().0, SlotChildren::TYPED_UNKNOWN);
        assert_eq!(std::mem::size_of::<SlotChildren>(), 12);
    }

    #[test]
    fn store_identifier_text_lives_in_the_name_column() {
        let file = new_file_store("/e.ts", "");
        let f = NodeFactory::for_file(file);
        let id = f.new_identifier("abc");
        let private = f.new_private_identifier("#p");
        let missing = f.new_identifier("");
        assert!(identifier_text(store_ast_node(id)).is_empty());
        // S1: every store identifier points at one shared astdata node.
        assert!(std::ptr::eq(store_ast_node(id), store_ast_node(missing)));
        assert!(std::ptr::eq(
            store_ast_node(id),
            shared_name_node(SyntaxKind::Identifier)
        ));
        assert!(std::ptr::eq(
            store_ast_node(private),
            shared_name_node(SyntaxKind::PrivateIdentifier)
        ));
        assert_eq!(id.text(), "abc");
        assert_eq!(private.text(), "#p");
        assert_eq!(missing.text(), "");
        // Data cloned from a store identifier has no text; the name stays.
        replace_store_node_data(id, store_ast_node(id).clone());
        assert_eq!(id.text(), "abc");
        // S1: the write gives the slot its own node; the shared one stays.
        assert!(!std::ptr::eq(store_ast_node(id), store_ast_node(missing)));
        assert_eq!(missing.text(), "");
        // A synthetic identifier keeps its text in the data.
        assert_eq!(NodeFactory::new().new_identifier("syn").text(), "syn");
        freeze_file_store(file);
        assert_eq!(id.text(), "abc");
        assert_eq!(missing.text(), "");
    }

    #[test]
    fn pending_lists_are_built_once_into_node_data() {
        assert_eq!(std::mem::size_of::<NodeList>(), 16);
        assert_eq!(std::mem::size_of::<ModifierList>(), 16);
        let file = new_file_store("/f.ts", "");
        let f = NodeFactory::for_file(file);
        let a = f.new_identifier("a");
        let list = f.new_node_list_with_loc(&[a], TextRange::new(1, 4));
        assert!(list.pending_list().is_some());
        assert_eq!(list, list);
        assert_eq!(list.nodes().get(0), a);
        assert_eq!(list.loc(), TextRange::new(1, 4));
        let decls = f.new_variable_declaration_list(list, NodeFlags::NONE);
        let stored = decls.declarations();
        assert!(stored.ts_list().is_some());
        assert_eq!(stored, decls.declarations());
        assert_ne!(stored, list);
        assert_eq!(stored.nodes().get(0), a);
        assert_eq!(stored.loc(), TextRange::new(1, 4));

        let export = f.new_modifier(SyntaxKind::ExportKeyword);
        let modifiers = f.new_modifier_list(&[export]);
        assert!(modifiers.pending_list().is_some());
        assert_eq!(modifiers.modifier_flags(), ModifierFlags::EXPORT);
        let statement = f.new_variable_statement(modifiers, decls);
        assert_eq!(
            statement.modifiers().modifier_flags(),
            ModifierFlags::EXPORT
        );
        assert_ne!(statement.modifiers(), modifiers);

        let empty = f.new_node_list_with_loc(&[], TextRange::new(2, 2));
        let missing = empty.with_missing_marker();
        assert!(!crate::frontend::parser::parser_p1::is_missing_node_list(
            empty
        ));
        assert!(crate::frontend::parser::parser_p1::is_missing_node_list(
            missing
        ));
        let holder = f.new_variable_declaration_list(missing, NodeFlags::NONE);
        assert!(crate::frontend::parser::parser_p1::is_missing_node_list(
            holder.declarations()
        ));
    }

    #[test]
    fn adopted_detached_parse_equals_a_serial_parse() {
        use crate::frontend::parser::{
            ParsedSourceFile, SourceFileParseOptions, adopt_detached_parse, parse_source_file,
            parse_source_file_detached,
        };
        let text =
            "/** doc */\nexport function f(a: number) { return a + 1; }\nlet x = <T>(y: T) => y;\n";
        let opts = SourceFileParseOptions {
            file_name: "/a.ts".to_string(),
            ..Default::default()
        };
        let serial = parse_source_file(&opts, text, ScriptKind::TS);
        let worker_opts = opts.clone();
        let detached = std::thread::spawn(move || {
            parse_source_file_detached(7, &worker_opts, text, ScriptKind::TS)
        })
        .join()
        .unwrap();
        assert_eq!(detached.store.id(), DETACHED_STORE_BASE + 7);
        let adopted: ParsedSourceFile = adopt_detached_parse(detached, &opts);

        assert_eq!(adopted.store, serial.store + 1);
        let slots = file_store_slot_count(serial.store);
        assert_eq!(file_store_slot_count(adopted.store), slots);
        let to_serial = |n: Node| {
            if n.is_some() && n.file_index() == adopted.store {
                handle(serial.store, slot_index(n) as u32)
            } else {
                n
            }
        };
        for index in 1..slots as u32 {
            let (a, b) = (handle(serial.store, index), handle(adopted.store, index));
            let (ha, hb) = (store_header(a), store_header(b));
            assert_eq!(ha.kind, hb.kind);
            assert_eq!(ha.flags, hb.flags);
            assert_eq!(ha.loc, hb.loc);
            assert_eq!(ha.parent, to_serial(hb.parent));
        }
        assert_eq!(serial.root, to_serial(adopted.root));
        assert_eq!(serial.imports.len(), adopted.imports.len());
        assert_eq!(serial.jsdoc_cache.len(), adopted.jsdoc_cache.len());
        for (node, jsdocs) in &adopted.jsdoc_cache {
            let expected: Vec<Node> = jsdocs.iter().map(|&n| to_serial(n)).collect();
            assert_eq!(serial.jsdoc_cache[&to_serial(*node)], expected);
            assert_eq!(
                file_store_js_doc(adopted.store, *node).map(NodeSlice::to_vec),
                Some(jsdocs.clone())
            );
        }
    }

    // watchcfg1 round c: a parse worker's parse in a freeable parse scope
    // owns its nodes (`new_detached_file_store`), the adopted parse reads
    // like a static parse of the same text with its JSDoc cache on the real
    // store id, and a worker parse that the loader does not take frees its
    // store and text when it is dropped.
    #[test]
    fn owned_detached_parse_reads_like_a_static_parse() {
        use crate::frontend::parser::{
            SourceFileParseOptions, adopt_detached_parse, parse_source_file,
            parse_source_file_detached,
        };
        // A JS file: its parse makes the JSDoc cache (a TS file's is lazy).
        let text = "/** @param {number} a */\nexport async function f(a, b = 1) { return a + b; }\n\
            /** @type {string[]} */\nlet x = [1, 2].map(y => `${y}`);\n";
        let opts = |name: &str| SourceFileParseOptions {
            file_name: name.to_string(),
            ..Default::default()
        };
        let fixed = parse_source_file(&opts("/static.js"), text, ScriptKind::JS);
        let worker = |job: usize, name: &'static str| {
            let worker_opts = opts(name);
            let text = FileText::Shared(Arc::from(text));
            let weak = text.weak().expect("a shared text");
            let parse = std::thread::spawn(move || {
                let _scope = enter_owned_parse();
                parse_source_file_detached(job, &worker_opts, text, ScriptKind::JS)
            })
            .join()
            .unwrap();
            (parse, weak)
        };

        let (detached, _) = worker(8, "/owned.js");
        assert!(detached.freeable);
        assert!(detached.store.is_self_contained());
        assert!(
            detached.store.store.owned.is_some(),
            "the worker parse owns its nodes"
        );
        let owned = adopt_detached_parse(detached, &opts("/owned.js"));
        assert!(with_store(owned.store, |s| s.owned.is_some()));
        let (a, b) = (tree(fixed.root), tree(owned.root));
        assert_eq!(a.len(), b.len());
        for (&a, &b) in a.iter().zip(&b) {
            assert_eq!(twin_facts(a), twin_facts(b), "{:?}", a.kind());
        }
        assert_eq!(owned.jsdoc_cache.len(), 2);
        for (node, jsdocs) in &owned.jsdoc_cache {
            assert_eq!(node.file_index(), owned.store);
            assert_eq!(
                file_store_js_doc(owned.store, *node).map(NodeSlice::to_vec),
                Some(jsdocs.clone())
            );
        }

        let (untaken, text) = worker(9, "/untaken.js");
        assert!(text.upgrade().is_some());
        drop(untaken);
        assert!(
            text.upgrade().is_none(),
            "a worker parse that the loader does not take is freed"
        );
    }

    /// The nodes of the tree of `root` in `for_each_child` order.
    fn tree(root: Node) -> Vec<Node> {
        fn walk(n: Node, out: &mut Vec<Node>) {
            out.push(n);
            n.for_each_child(|child| {
                walk(child, out);
                false
            });
        }
        let mut out = Vec::new();
        walk(root, &mut out);
        out
    }

    /// What the node reads give for `n` (lsshells M3c twin parses): header,
    /// text, modifiers, JSDoc and the lists of its data.
    fn twin_facts(n: Node) -> String {
        let mut lists = Vec::new();
        n.for_each_child_and_lists(&mut |_| false, &mut |l, is_mod| {
            lists.push((l.nodes().len(), l.pos(), l.end(), is_mod));
        });
        format!(
            "{:?} {:?} {:?} {:?} {:?} {:?} {} {:?}",
            n.kind(),
            n.loc(),
            n.flags(),
            n.text(),
            n.modifier_flags(),
            n.modifiers().nodes().to_vec().len(),
            n.js_doc(Node::NIL).len(),
            lists
        )
    }

    // lsshells M3c: a freeable parse keeps its astdata nodes in its store
    // and answers every node read as a static parse of the same text.
    #[test]
    fn owned_parse_reads_like_a_static_parse() {
        use crate::frontend::parser::{SourceFileParseOptions, parse_source_file};
        let text = "/** doc */\nexport async function f(a: number, b = 1) { return a + b; }\n\
            let x: Array<string> = [1, 2].map(y => `${y}`);\n\
            type T = { a?: string } | number;\n\
            @dec class C<U> extends Object { private m(): U | undefined { return undefined; } }\n\
            const re = /ab+c/g, s = 'str', n = 0x10, big = 10n;\n";
        let opts = |name: &str| SourceFileParseOptions {
            file_name: name.to_string(),
            ..Default::default()
        };
        let fixed = parse_source_file(&opts("/static.ts"), text, ScriptKind::TS);
        let before = owned_node_count();
        let owned = {
            let _scope = enter_owned_parse();
            parse_source_file(&opts("/owned.ts"), text, ScriptKind::TS)
        };
        assert!(!is_freeable_parse(), "the scope ends");
        assert!(
            owned_node_count() > before,
            "the freeable parse owns its nodes"
        );
        assert!(with_store(owned.store, |s| s.owned.is_some()));
        assert!(with_store(fixed.store, |s| s.owned.is_none()));
        assert!(try_store_ast_node(fixed.root).is_some());
        assert!(
            try_store_ast_node(owned.root).is_none(),
            "an owned node has no 'static data"
        );
        let (a, b) = (tree(fixed.root), tree(owned.root));
        assert_eq!(a.len(), b.len());
        for (&a, &b) in a.iter().zip(&b) {
            assert_eq!(twin_facts(a), twin_facts(b), "{:?}", a.kind());
        }
        assert!(owned.root.statement_list().store_list().is_some());
        assert_eq!(fixed.root.statements().len(), owned.root.statements().len());
    }

    // lsshells M3c: a data write in a store that owns its nodes fills a new
    // cell, so a list handle of the old data still reads the old list (a Go
    // `*NodeList` pointer), and the pending lists are the store's.
    #[test]
    fn owned_data_write_keeps_old_list_handles() {
        let _scope = enter_owned_parse();
        let file = new_file_store("/owned-write.ts", "");
        let f = NodeFactory::for_file(file);
        let export = f.new_modifier(SyntaxKind::ExportKeyword);
        let modifiers = f.new_modifier_list(&[export]);
        assert!(modifiers.store_list().is_some(), "an owned pending list");
        let list = f.new_variable_declaration_list(NodeList::NIL, NodeFlags::NONE);
        let statement = f.new_variable_statement(modifiers, list);
        let old = statement.modifiers();
        assert!(old.store_list().is_some());
        assert_eq!(old.nodes().to_vec(), vec![export]);
        assert_eq!(old.modifier_flags(), ModifierFlags::EXPORT);
        assert_eq!(old, statement.modifiers(), "one list, one handle identity");
        assert!(list.declarations().is_nil());

        let mut data = with_ast_data(statement, Clone::clone);
        if let NodeData::VariableStatement(d) = &mut data {
            d.modifiers = None;
        }
        replace_store_node_data(statement, data);
        assert!(statement.modifiers().is_nil());
        assert_eq!(old.nodes().to_vec(), vec![export], "the old list stays");
        assert_ne!(old, statement.modifiers());
        freeze_file_store(file);
        assert_eq!(statement.kind(), SyntaxKind::VariableStatement);
        assert_eq!(statement.declaration_list(), list);
        assert!(with_store(file, |s| s.owned_ref().len >= 4));
    }

    // AST node records, steps 2 and 3: a node whose Go parent is a node of
    // another store keeps that parent in the foreign parent table of its
    // store (`ParentCode`, `FOREIGN_PARENT`). The protected tests never make
    // one (step 2 skeptic), so this test does, and reads the parents before
    // the publish (the store), after it (the block of a static file and the
    // node shell of a freeable file version), and after that version dies
    // (its records stay leaked). It publishes, so no other test may build
    // or publish stores while it runs (the runner uses one thread).
    #[test]
    fn foreign_parents_read_through_the_file_block() {
        let a = new_file_store("/foreign/a.ts", "x;");
        let b = new_file_store("/foreign/b.ts", "y; w;");
        let c = new_file_store("/foreign/c.ts", "v;");
        let (fa, fb, fc) = (
            NodeFactory::for_file(a),
            NodeFactory::for_file(b),
            NodeFactory::for_file(c),
        );
        let x = fa.new_identifier("x");
        let statement = fa.new_expression_statement(x);
        let z = fa.new_identifier("z");
        let y = fb.new_identifier("y");
        let w = fb.new_identifier("w");
        let v = fc.new_identifier("v");
        set_node_parent(x, statement);
        set_node_parent(statement, y);
        set_node_parent(z, w);
        set_node_parent(v, x);
        let check = |when: &str| {
            assert_eq!(x.parent(), statement, "{when}");
            assert_eq!(statement.parent(), y, "{when}");
            assert_eq!(z.parent(), w, "{when}");
            assert_eq!(v.parent(), x, "{when}");
            assert_eq!(y.parent(), Node::NIL, "{when}");
            assert_eq!(store_header(statement).parent, y, "{when}");
            assert_eq!(store_header(v).parent, x, "{when}");
        };
        check("built");
        for file in [a, b, c] {
            freeze_file_store(file);
        }
        with_store(a, |s| {
            assert_eq!(s.foreign_parents, [y, w]);
            assert!(!s.facts.parents_local);
        });
        check("frozen");

        // c is a freeable file version: the block of its id is its node
        // shell, and the version owns its store.
        let version = super::super::file_version::FileVersion::new(c);
        crate::program::publish_parsed_files("/");
        assert!(file_block(a).is_some_and(|block| block.file.store.is_some()));
        assert!(file_block(c).is_some_and(|block| block.file.store.is_none()
            && block.file.go_file.is_none()
            && *block.file.foreign == [x]));
        check("published");
        assert_eq!(frozen_store_parent(x), Some(statement));
        assert_eq!(frozen_store_parent(statement), None, "the header has it");
        assert_eq!(frozen_store_parent(v), None, "the header has it");
        assert_eq!(
            frozen_store_parent_kind(x),
            Some((statement, SyntaxKind::ExpressionStatement))
        );
        assert_eq!(frozen_store_parent_kind(statement), None);
        assert_eq!(
            frozen_find_ancestor(x, |_, _| false),
            Some(AncestorWalk::Next(y))
        );
        assert_eq!(
            frozen_find_ancestor(v, |_, _| false),
            Some(AncestorWalk::Next(x))
        );
        let parents_local = |file| frozen_store_facts(file).map(|facts| facts.parents_local);
        assert_eq!(parents_local(a), Some(false));
        assert_eq!(parents_local(b), Some(true));
        assert_eq!(parents_local(c), Some(false));
        assert!(
            !frozen_store_lacks_deprecated_tag(x),
            "parents leave store a"
        );

        // A record read of a dead version gives the values of that node.
        assert_eq!(Arc::strong_count(&version), 1, "no pin holds the version");
        drop(version);
        assert_eq!(v.parent(), x);
        assert_eq!(
            frozen_find_ancestor(v, |_, _| false),
            Some(AncestorWalk::Next(x))
        );
    }

    // AST node records, step 3: a file id from `LOW_BLOCKS` on has its
    // block in a chunk of `HIGH_BLOCKS` (a process with many programs).
    // This test process never gets that many file ids, so the test sets
    // the block of the last file id, which no publish reaches, and reads
    // its node through the node reads.
    #[test]
    fn high_file_ids_read_their_block_from_a_chunk() {
        set_and_read_high_file_block(FILE_ID_LIMIT - 1);
    }

    // fileid1: ids are never used again, so a long session (one id per
    // edit) or a long watch could reach the old limit of 2^22 ids and
    // panic ("too many file ids"). The id at the old limit has a block now.
    #[test]
    fn file_ids_past_the_old_limit_read_their_block() {
        set_and_read_high_file_block(1 << 22);
    }

    // fileid1: with a test cap, the last id below the cap is given and
    // published, and the next new store stops at the cap. It publishes, so
    // no other test may build or publish stores while it runs (the runner
    // uses one thread).
    #[test]
    fn new_file_ids_stop_at_the_cap() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        let message = std::thread::spawn(|| {
            let cap = PUBLISHED.load(Ordering::Acquire) + 1;
            TEST_FILE_ID_CAP.set(cap);
            let last = new_file_store("/fileidcap/a.ts", "a;");
            assert_eq!(last, cap - 1);
            NodeFactory::for_file(last).new_identifier("a");
            freeze_file_store(last);
            crate::program::publish_parsed_files("/");
            assert_eq!(PUBLISHED.load(Ordering::Acquire), cap);
            assert!(file_block(last).is_some(), "the last id is published");
            let next = catch_unwind(AssertUnwindSafe(|| new_file_store("/fileidcap/b.ts", "b;")));
            let payload = next.expect_err("a new store at the cap must panic");
            payload
                .downcast_ref::<&str>()
                .map(|m| (*m).to_owned())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_default()
        })
        .join()
        .unwrap();
        assert_eq!(message, "too many file ids");
    }

    // factscol1b: a static publish gets a facts column at its first facts
    // read, one word per slot, shared by all threads. A node shell (a
    // freeable file version) gets none, so its facts stay in the
    // thread-local map that forgets them with the version. The test sets
    // the blocks of two file ids that no publish reaches.
    #[test]
    fn static_files_keep_subtree_facts_in_a_column() {
        fn block(file: usize, store: Option<&'static FileStore>) -> FileBlock {
            let mut records = vec![NodeRecord::target(Node::NIL), NodeRecord::node(false)];
            *records[0].bind.get_mut() = NodeRecord::owner_word(file);
            FileBlock {
                kinds: Vec::leak(vec![SyntaxKind::Unknown, SyntaxKind::Block]),
                records: Vec::leak(records),
                kids: Vec::leak(vec![NodeKids::unknown(), NodeKids::unknown()]),
                file: Box::leak(Box::new(BlockFile {
                    nodes: &[],
                    facts: StoreFacts::ONLY_NIL_SLOT,
                    root: Node::NIL,
                    foreign: &[],
                    links: &[],
                    store,
                    go_file: None,
                    subtree_facts: OnceLock::new(),
                })),
            }
        }
        let (static_file, shell_file) = (FILE_ID_LIMIT - 5, FILE_ID_LIMIT - 6);
        let store: &'static FileStore = Box::leak(Box::default());
        set_file_block(static_file, block(static_file, Some(store)));
        set_file_block(shell_file, block(shell_file, None));
        let n = handle(static_file, 1);
        let word = static_facts_word(n).expect("a static file has a column");
        assert_eq!(word.load(Ordering::Relaxed), 0, "no facts computed yet");
        word.store(7, Ordering::Relaxed);
        let column = file_block(static_file).unwrap().file.subtree_facts.get();
        assert_eq!(column.map(|c| c.len()), Some(2), "one word per slot");
        let other_thread =
            std::thread::spawn(move || static_facts_word(n).map(|w| w.load(Ordering::Relaxed)));
        assert_eq!(other_thread.join().unwrap(), Some(7));
        assert!(static_facts_word(handle(shell_file, 1)).is_none());
        assert!(static_facts_word(Node::NIL).is_none());
    }

    /// Sets the block of unpublished file id `file` (from `LOW_BLOCKS` on)
    /// and reads its node through the node reads.
    fn set_and_read_high_file_block(file: usize) {
        let mut records = vec![NodeRecord::target(Node::NIL), NodeRecord::node(false)];
        records[1].set_flags(NodeFlags::AMBIENT);
        records[1].set_loc(TextRange::new(3, 7));
        // AST node records, step 4: the owner word (`block_is_owned`).
        *records[0].bind.get_mut() = NodeRecord::owner_word(file);
        let block = FileBlock {
            kinds: Vec::leak(vec![SyntaxKind::Unknown, SyntaxKind::Identifier]),
            records: Vec::leak(records),
            kids: Vec::leak(vec![NodeKids::unknown(), NodeKids::unknown()]),
            file: Box::leak(Box::new(BlockFile {
                nodes: &[],
                facts: StoreFacts::ONLY_NIL_SLOT,
                root: Node::NIL,
                foreign: &[],
                links: &[],
                store: None,
                go_file: None,
                subtree_facts: OnceLock::new(),
            })),
        };
        assert!(file_block(file).is_none());
        set_file_block(file, block);
        let n = handle(file, 1);
        assert_eq!(n.kind(), SyntaxKind::Identifier);
        assert_eq!(n.flags(), NodeFlags::AMBIENT);
        assert_eq!(n.loc(), TextRange::new(3, 7));
        assert_eq!(n.parent(), Node::NIL);
        assert_eq!(
            resolve_store_id(file, crate::astdata::NodeId::new(0)),
            Node::NIL
        );
        assert!(file_block(file - 1).is_none(), "the id before");
        assert!(file_block(file - HIGH_CHUNK).is_none(), "another chunk");
        assert!(file_block(LOW_BLOCKS).is_none(), "the first chunk");
        assert!(file_block(FILE_ID_LIMIT).is_none(), "no file id");
    }

    // textleak1 A1: a publish keeps the emptied cells of its build stores,
    // and the next build store of the thread reuses one, so a language
    // server edit leaks no store cell in the AST arena.
    #[test]
    fn next_build_stores_reuse_published_cells() {
        std::thread::spawn(|| {
            let a = new_file_store("/cells/a.ts", "a;");
            let a_cell = active_store(a).expect("a is the active store");
            NodeFactory::for_file(a).new_identifier("a");
            freeze_file_store(a);
            crate::program::publish_parsed_files("/");
            let b = new_file_store("/cells/b.ts", "b;");
            let b_cell = active_store(b).expect("b is the active store");
            assert!(
                std::ptr::eq(a_cell, b_cell),
                "b does not reuse the cell of a"
            );
            assert_eq!(file_store_text(b), "b;");
            assert_eq!(file_store_text(a), "a;");
        })
        .join()
        .unwrap();
    }

    // astmem1 P3: the bind ORs its added flags into the `flags` word of a
    // record, whose top bits are the record bits (`RECORD_BITS`). A flag
    // outside `BINDER_ADDED_FLAGS` (a binder bug) stops the bind in every
    // build, before it writes the record. It publishes, so no other test
    // may build or publish stores while it runs (the runner uses one
    // thread).
    #[test]
    fn a_bind_flag_outside_binder_added_flags_panics_in_every_build() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        let file = new_file_store("/bindflags/a.ts", "x;");
        let x = NodeFactory::for_file(file).new_identifier("x");
        freeze_file_store(file);
        crate::program::publish_parsed_files("/");
        let data = NodeBindData {
            added_flags: NodeFlags(NodeRecord::SOURCE_FILE_ROOT),
            ..NodeBindData::default()
        };
        let record = || &file_block(file).expect("a is published").records[slot_index(x)];
        let bits = record().bits();
        let bind = || {
            bind_store_records(
                file,
                std::iter::once((slot_index(x), 0, Some(&data), FlowNodeId::NIL)),
            )
        };
        let payload = catch_unwind(AssertUnwindSafe(bind)).expect_err("the bind must panic");
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .unwrap_or_default();
        assert!(
            message.contains("outside BINDER_ADDED_FLAGS"),
            "{message:?}"
        );
        assert_eq!(record().bits(), bits, "the bind changed the record bits");
    }

    // AST node records, step 4: the node shell of a freeable file version
    // takes a pooled block (`BlockPool`). A dead version gives its block
    // back, the block waits for two pin releases, and then the next
    // node shell that fits it takes it. A stale read of the dead version
    // then fails the owner check, and panics as a read of a dead version's
    // store does: a binder field read (the symbol, the flags, the flow node
    // and the extras) in every build (`check_block_owner`), every other
    // read with debug assertions. Without them a header read gives the new
    // owner's data (the kind column is not pooled; shells with equal kinds
    // share one). It publishes, so no other test may build or publish
    // stores while it runs (the runner uses one thread).
    #[test]
    fn pooled_node_blocks_wait_two_releases_and_check_their_owner() {
        use super::super::file_version::{FileVersion, release_file_version_pins};
        use std::panic::{AssertUnwindSafe, catch_unwind};
        clear_block_pool();
        // Publishes a freeable version of one identifier per name.
        let identifiers = |name: &'static str, names: &[&str]| {
            let text: &'static str = Box::leak(names.join(";").into_boxed_str());
            let file = new_file_store(name, text);
            let factory = NodeFactory::for_file(file);
            let nodes: Vec<Node> = names.iter().map(|n| factory.new_identifier(*n)).collect();
            freeze_file_store(file);
            let version = FileVersion::new(file);
            crate::program::publish_parsed_files("/");
            (version, nodes)
        };
        let addr = |n: Node| node_block_addr(n).expect("a published store node");

        // c, bound: its second node has a symbol and a flow node.
        let (c, c_nodes) = identifiers("/pool/c.ts", &["x", "y"]);
        let (c_file, cy) = (c.file(), c_nodes[1]);
        let c_block = addr(cy);
        assert!(node_block_is_owned(cy));
        file_block(c_file).expect("c is published").records[slot_index(cy)].write_bind(
            SymbolId(7),
            NodeFlags::NONE,
            3,
        );
        assert_eq!(frozen_store_symbol(cy), Some(SymbolId(7)));
        assert_eq!(Arc::strong_count(&c), 1, "no pin holds the version");
        drop(c);
        assert!(node_block_is_owned(cy), "the block of a dead version waits");
        assert_eq!(cy.kind(), SyntaxKind::Identifier);

        // Its block waits for two pin releases.
        let (d, d_nodes) = identifiers("/pool/d.ts", &["z", "w"]);
        assert_ne!(addr(d_nodes[0]), c_block, "no release yet");
        // Shells with equal kinds share one kind column (`shell_kinds`).
        let kinds_of = |n: Node| {
            registry_block(n.file_index())
                .expect("a published store node")
                .kinds
                .as_ptr()
        };
        assert_eq!(kinds_of(d_nodes[0]), kinds_of(cy), "one kind column");
        release_file_version_pins();
        let (d1, d1_nodes) = identifiers("/pool/d1.ts", &["u", "t"]);
        assert_ne!(addr(d1_nodes[0]), c_block, "one release");
        release_file_version_pins();

        // e has as many slots as c, so it takes c's block.
        let e_file = new_file_store("/pool/e.ts", "q;");
        let factory = NodeFactory::for_file(e_file);
        let q = factory.new_identifier("q");
        let statement = factory.new_expression_statement(q);
        set_node_parent(q, statement);
        freeze_file_store(e_file);
        let e = FileVersion::new(e_file);
        crate::program::publish_parsed_files("/");
        assert_eq!(slot_index(statement), slot_index(cy));
        assert_eq!(addr(statement), c_block, "two releases");
        assert!(node_block_is_owned(statement));
        assert!(!node_block_is_owned(cy), "c's block has a new owner");
        assert_eq!(statement.kind(), SyntaxKind::ExpressionStatement);
        assert_eq!(q.parent(), statement);
        assert_eq!(q.kind(), SyntaxKind::Identifier);
        assert_eq!(
            frozen_store_symbol(statement),
            Some(SymbolId::NIL),
            "not bound"
        );
        assert_eq!(frozen_store_bind_word(statement), Some(0), "not bound");

        assert_ne!(kinds_of(statement), kinds_of(cy), "other kinds");

        // A stale read of c.
        let released = format!("file version {c_file} is released");
        let panic_message = |read: &dyn Fn()| {
            catch_unwind(AssertUnwindSafe(read)).err().map(|payload| {
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_default()
            })
        };
        let binder_reads: [(&str, &dyn Fn()); 6] = [
            ("symbol", &|| {
                let _ = cy.symbol();
            }),
            ("flags", &|| {
                let _ = cy.flags();
            }),
            ("parser flags", &|| {
                let _ = cy.parser_flags(NodeFlags::AMBIENT);
            }),
            ("flow node", &|| {
                let _ = cy.flow_node();
            }),
            ("extras", &|| {
                let _ = cy.local_symbol();
            }),
            ("any symbol", &|| {
                let _ = frozen_store_any_symbol(c_file, |_| false);
            }),
        ];
        for (name, read) in binder_reads {
            assert_eq!(
                panic_message(read).as_deref(),
                Some(released.as_str()),
                "a stale {name} read of a reused block"
            );
        }
        let kind = panic_message(&|| {
            let _ = cy.kind();
        });
        if cfg!(debug_assertions) {
            assert_eq!(kind.as_deref(), Some(released.as_str()));
        } else {
            // The kind column is not pooled, so the kind is c's. A header
            // read gives the record of e: x has the parent of q, in c.
            assert_eq!(kind, None);
            assert_eq!(cy.kind(), SyntaxKind::Identifier);
            assert_eq!(slot_index(c_nodes[0]), slot_index(q));
            assert_eq!(c_nodes[0].parent(), cy);
        }

        // A version with more slots than a free block has gets a fresh
        // block; one that fits takes it.
        let (d_block, d1_block) = (addr(d_nodes[0]), addr(d1_nodes[0]));
        drop((d, d1));
        release_file_version_pins();
        release_file_version_pins();
        let names: Vec<String> = (0..80).map(|i| format!("n{i}")).collect();
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let (f, f_nodes) = identifiers("/pool/f.ts", &names);
        assert!(![c_block, d_block, d1_block].contains(&addr(f_nodes[0])));
        assert_eq!(f_nodes[79].kind(), SyntaxKind::Identifier);
        let (g, g_nodes) = identifiers("/pool/g.ts", &["s"]);
        assert!([d_block, d1_block].contains(&addr(g_nodes[0])));
        assert_eq!(g_nodes[0].kind(), SyntaxKind::Identifier);
        drop((e, f, g));
    }

    // watchfree1: a node shell shares the kind column of any earlier shell
    // with equal kinds, not only of the last few (`shell_kinds`): a
    // `tsc --watch` build after a config change parses every file again.
    // It publishes, so no other test may build or publish stores while it
    // runs (the runner uses one thread).
    #[test]
    fn node_shells_share_any_equal_kind_column() {
        use super::super::file_version::FileVersion;
        // Publishes a freeable version of `count` identifiers, and gives
        // the address of its kind column.
        let column = |count: usize| {
            let names: Vec<String> = (0..count).map(|i| format!("n{i}")).collect();
            let text: &'static str = Box::leak(names.join(";").into_boxed_str());
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            let name: &'static str = Box::leak(format!("/kinds/{id}.ts").into_boxed_str());
            let file = new_file_store(name, text);
            let factory = NodeFactory::for_file(file);
            let node = names
                .iter()
                .map(|n| factory.new_identifier(n.as_str()))
                .last()
                .expect("one name");
            freeze_file_store(file);
            let version = FileVersion::new(file);
            crate::program::publish_parsed_files("/");
            let kinds = registry_block(node.file_index())
                .expect("a published store node")
                .kinds
                .as_ptr();
            (version, kinds)
        };
        let (_first, first) = column(1);
        let others: Vec<_> = (2..9).map(column).collect();
        assert!(
            others.iter().all(|(_, kinds)| *kinds != first),
            "other kinds, other columns"
        );
        let (_again, again) = column(1);
        assert_eq!(again, first, "equal kinds share the first column");
    }
}
