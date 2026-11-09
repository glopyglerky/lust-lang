//! Port of Go `printer/emitcontext.go`.
//!
//! `EmitContext` stores side tables that transforms write and the printer
//! reads: emit flags, comment and source map ranges, original node links,
//! auto-generated name info and emit helpers.

use crate::flags_macros::go_enum;
use crate::prelude::*;

use super::types::{EmitFlags, GeneratedIdentifierFlags};
use super::utilities::{find_span_end, };
use std::cell::OnceCell;
use std::sync::atomic::{AtomicU32, Ordering};


// Stores side-table information used during transformation that can be read by the printer to customize emit
//
// NOTE: EmitContext is not guaranteed to be thread-safe.
// PORT: Go mutates the context through a pointer that many objects share.
// Here the context is shared as `Rc<EmitContext>`, so each field is a
// `RefCell`. Methods never hold a borrow across a factory or visitor call.

/// Go `environmentFlags`.
pub(crate) type EnvironmentFlags = i32;

pub(crate) const ENVIRONMENT_FLAGS_NONE: EnvironmentFlags = 0;
pub(crate) const ENVIRONMENT_FLAGS_IN_PARAMETERS: EnvironmentFlags = 1 << 0; // currently visiting a parameter list
pub(crate) const ENVIRONMENT_FLAGS_VARIABLES_HOISTED_IN_PARAMETERS: EnvironmentFlags = 1 << 1; // a temp variable was hoisted while visiting a parameter list

/// Go `varScope`.
#[derive(Default, Debug)]
pub(crate) struct VarScope {
    pub(crate) variables: Vec<Node>,
    pub(crate) functions: Vec<Node>,
    pub(crate) flags: EnvironmentFlags,
    pub(crate) initialization_statements: Vec<Node>,
}

/// Go `EmitContext.Factory`. Set once by `new_emit_context`; it derefs to the
/// printer `NodeFactory`, so `c.factory.new_x(..)` reads like Go.
#[derive(Default)]
pub struct EmitContextFactory(OnceCell<NodeFactory>);

impl std::ops::Deref for EmitContextFactory {
    type Target = NodeFactory;

    fn deref(&self) -> &NodeFactory {
        self.0.get().expect("EmitContext has no Factory")
    }
}

// Go: printer/emitcontext.go:45 NewEmitContext




// Go: printer/emitcontext.go:57 GetEmitContext
// PORT: Go takes a context from a `sync.Pool`. Pooling is a concurrency
// optimization, so this makes a new context. The returned function resets
// it, as Go does before returning it to the pool.


// Go: printer/emitcontext.go:362 isHoistedVariable

/// Go `AutoGenerateOptions`.
#[derive(Clone, Debug, Default)]
pub struct AutoGenerateOptions {
    pub flags: GeneratedIdentifierFlags,
    pub prefix: String,
    pub suffix: String,
}

/// Go `nextAutoGenerateId atomic.Uint32`. Use `next_auto_generate_id()` for
/// Go `AutoGenerateId(nextAutoGenerateId.Add(1))`.
pub static NEXT_AUTO_GENERATE_ID: AtomicU32 = AtomicU32::new(0);

/// Go `AutoGenerateId(nextAutoGenerateId.Add(1))`.
#[must_use]
pub fn next_auto_generate_id() -> AutoGenerateId {
    AutoGenerateId(NEXT_AUTO_GENERATE_ID.fetch_add(1, Ordering::Relaxed) + 1)
}

/// Go `AutoGenerateId`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AutoGenerateId(pub u32);

/// Go `AutoGenerateInfo`.
#[derive(Clone, Debug, Default)]
pub struct AutoGenerateInfo {
    pub flags: GeneratedIdentifierFlags, // Specifies whether to auto-generate the text for an identifier.
    pub id: AutoGenerateId, // Ensures unique generated identifiers get unique names, but clones get the same name.
    pub prefix: String,     // Optional prefix to apply to the start of the generated name
    pub suffix: String,     // Optional suffix to apply to the end of the generated name
    pub node: Node, // For a GeneratedIdentifierFlagsNode, the node from which to generate an identifier
}

//
// Emit-related Data
//

/// Go `emitNodeFlags`.
pub(crate) type EmitNodeFlags = u32;

pub(crate) const HAS_COMMENT_RANGE: EmitNodeFlags = 1 << 0;
pub(crate) const HAS_SOURCE_MAP_RANGE: EmitNodeFlags = 1 << 1;

// Go: printer/emitcontext.go:528 SnippetKind
go_enum!(SnippetKind, i32 {
    TAB_STOP = 0; // SnippetKindTabStop
});

// Go: printer/emitcontext.go:534 SnippetElement
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnippetElement {
    pub kind: SnippetKind,
    pub order: i32,
}

/// Go `SynthesizedComment`.
#[derive(Clone, Debug)]
pub struct SynthesizedComment {
    pub kind: SyntaxKind,
    pub loc: TextRange,
    pub has_leading_new_line: bool,
    pub has_trailing_new_line: bool,
    pub text: String,
}





// ──────────────────────────────────────────────────────────────────────
// Node-keyed side tables
// ──────────────────────────────────────────────────────────────────────

/// Keys per page of a `SyntheticPages` table.
const NODE_PAGE: usize = 64;

/// The most pages in the window of a `SyntheticPages` table: 2M handles,
/// so the page list is at most 256 KiB.
const NODE_WINDOW_PAGES: usize = 1 << 15;

/// `SyntheticPages::first` of a table with no window yet. A handle page
/// minus it (wrapping) is never in the window.
const NO_WINDOW: usize = usize::MAX / 2;

/// The records of an `EmitContext` table whose keys are synthetic (factory)
/// nodes, in pages by handle number. A synthetic handle is the slot number
/// of its node in the arena of its thread, and the nodes that a context
/// sees are mostly the ones its transforms make, which have near numbers.
/// The window has `NODE_WINDOW_PAGES` pages from the page of the first key
/// written. A key outside it is not here: the owner table keeps it in its
/// other store, with the parsed keys.
// PERF: emitast2. A read is a few loads, not a hash lookup: emit flags,
// comment ranges and original links are read for each node that a
// transform makes or the printer prints.
#[derive(Clone)]
struct SyntheticPages<V> {
    /// The handle page of `pages[0]`, or `NO_WINDOW`.
    first: usize,
    pages: Vec<Option<Box<[Option<V>; NODE_PAGE]>>>,
}

impl<V> Default for SyntheticPages<V> {
    fn default() -> Self {
        Self {
            first: NO_WINDOW,
            pages: Vec::new(),
        }
    }
}

impl<V> SyntheticPages<V> {




    fn values(&self) -> impl Iterator<Item = &V> {
        self.pages.iter().flatten().flat_map(|p| p.iter().flatten())
    }
}

/// Go `EmitContext.original`: the original node of each node that has one.
// PORT: Go `map[*ast.Node]*ast.Node`; see `SyntheticPages`.
#[derive(Clone, Default)]
pub(crate) struct Originals {
    synthetic: SyntheticPages<Node>,
    /// Parsed keys, and synthetic keys outside the window.
    other: FxHashMap<Node, Node>,
}

impl Originals {


    fn check_overwrite(old: Node, original: Node, allow_overwrite: bool) {
        if !allow_overwrite && old != original {
            panic!("Original node already set.");
        }
    }


    fn clear(&mut self) {
        self.synthetic = SyntheticPages::default();
        self.other.clear();
    }

    fn values(&self) -> impl Iterator<Item = Node> {
        self.synthetic.values().chain(self.other.values()).copied()
    }
}


