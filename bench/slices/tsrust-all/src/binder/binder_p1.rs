//! Port of typescript-go `internal/binder/binder.go` lines 1-910.
//!
//! PORT: Go stores binder output on AST nodes and on `ast.SourceFile`. Here
//! the AST is immutable, so the binder keeps that output in `Binder`
//! (`node_bind`, `file_bind`, `flow_nodes`) while it runs, and
//! `bind_source_file` moves it into the node records (`ast/store.rs`) and
//! the `GoFile` `OnceCell`s at the end (`BoundFile::install`).
//! Binder code must read and write node binder data through the `Binder`
//! helpers below (`node_symbol`, `set_node_symbol`, `node_flags`,
//! `get_locals`, `flow`, `flow_mut`, ...), not through `Node::symbol()` and
//! friends, which only see the data after binding finishes.

use crate::astdata::NodeData;
use crate::prelude::*;

// Go: binder/binder.go:45 ExpandoAssignmentInfo
#[derive(Clone, Copy, Debug, Default)]
pub struct ExpandoAssignmentInfo {
    pub node: Node,
    pub container: Node,
    pub block_scope_container: Node,
}

// Go: binder/binder.go:51 Binder
//
// PORT: `bindFunc` (a cached closure over `b.bind`) is not stored; callers
// use a closure over `self.bind` at the call site. `symbolArena` becomes
// `symbols` (the shared `SymbolArena`, moved in for the duration of
// binding). `flowNodeArena` becomes `flow_nodes`. `flowListArena` and
// `singleDeclarationsArena` are not needed: `FlowList` is a
// `Vec<FlowNodeId>` and declaration lists are `Vec<Node>`. The last five
// fields are Rust-only storage for data Go writes onto nodes and the file.

// Go: binder/binder.go:84 ActiveLabel
#[derive(Clone, Debug, Default)]
pub struct ActiveLabel {
    pub next: Option<Rc<RefCell<ActiveLabel>>>,
    pub break_target: FlowNodeId,
    pub continue_target: FlowNodeId,
    pub name: String,
    pub referenced: bool,
}

impl ActiveLabel {
    // Go: binder/binder.go:92 BreakTarget
    pub fn break_target_exported(&self) -> FlowNodeId {
        self.break_target
    }

    // Go: binder/binder.go:93 ContinueTarget
    pub fn continue_target_exported(&self) -> FlowNodeId {
        self.continue_target
    }
}

impl FlowNode {
    /// Go `flow.Node.AsFlowSwitchClauseData()`.
    pub fn as_flow_switch_clause_data(&self) -> FlowSwitchClauseData {
        assert!(
            self.flags.intersects(FlowFlags::SWITCH_CLAUSE),
            "not a switch clause flow node"
        );
        FlowSwitchClauseData {
            switch_statement: self.node,
            clause_start: self.antecedents[0].0 as u32 as i32,
            clause_end: self.antecedents[1].0 as u32 as i32,
        }
    }

    /// Go `flow.Node.AsFlowReduceLabelData()`.
    pub fn as_flow_reduce_label_data(&self) -> FlowReduceLabelData {
        assert!(
            self.flags.intersects(FlowFlags::REDUCE_LABEL),
            "not a reduce label flow node"
        );
        FlowReduceLabelData {
            target: self.antecedents[0],
            antecedents: self.antecedents[1..].to_vec(),
        }
    }
}

// Go: binder/binder.go:95 BindSourceFile
// Go: binder/binder.go:120 bindSourceFile
//
// PORT: `getBinder`/`putBinder` (a sync.Pool) are not needed. `symbols` is
// the program-wide arena; it is moved into the binder and moved back when
// binding ends. The binder output is stored in the file's `GoFile`
// `OnceCell`s (Go `file.BindOnce`). A file that is already bound is skipped.
// The caller publishes what the file added to `symbols` for the checkers'
// copies (`Lineage::add_file`).

/// Binder data per node of the file being bound. Most nodes get no data, so
/// a node keeps a 4-byte slot into `entries`, and `entries[0]` is the empty
/// data, which is never written. The flow node of a node is kept in `flows`,
/// not in `entries`. The install writes both into the node records
/// (`nodes`, `ast::bind_store_records`); `compact` merges them into the
/// compact form of the lib bind snapshot.
// PERF: bind C. The binder gives a flow node to every identifier (Go
// binder.go `bind`), so a flow node in `entries` made a 40-byte entry for
// each of them. Now it is one 4-byte write.
#[derive(Default)]
pub struct NodeBindBuilder {
    /// Per node, by `NodeId::index()`: index into `entries`, 0 for none.
    slots: Vec<u32>,
    entries: Vec<NodeBindData>,
    /// Per node, by `NodeId::index()`: the low half of its flow node id
    /// (index + 1), 0 for nil.
    flows: Vec<u32>,
    /// The high half of every flow node id of the file: its file index
    /// (`FlowNodeId::new`).
    flow_file: u64,
    /// Nodes that got a flow node, counted when a flow goes from nil to
    /// set. With `entries` it bounds the nodes that have data.
    flow_count: usize,
}

impl NodeBindBuilder {

    /// The data of node `index` (`NodeId::index()`), without its flow node.
    /// Binder code never reads a flow node back from a node.
    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> &NodeBindData {
        &self.entries[self.slots[index] as usize]
    }

    /// The data of node `index`, made on first write. Set the flow node
    /// with `set_flow_node`, not through this reference.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> &mut NodeBindData {
        let mut slot = self.slots[index];
        if slot == 0 {
            slot = u32::try_from(self.entries.len()).expect("node bind overflow");
            self.entries.push(NodeBindData::default());
            self.slots[index] = slot;
        }
        &mut self.entries[slot as usize]
    }

    /// Go `node.FlowNodeData().FlowNode = flow` for node `index`.
    #[inline]
    pub fn set_flow_node(&mut self, index: usize, flow: FlowNodeId) {
        debug_assert!(
            flow.is_nil() || (flow.0 & !0xffff_ffff) == self.flow_file,
            "flow node from another file"
        );
        let low = flow.0 as u32;
        let old = std::mem::replace(&mut self.flows[index], low);
        self.flow_count += usize::from(old == 0 && low != 0);
    }


}

/// The binder data of the nodes of one file as the bind hands it over
/// (`BoundFile`): the builder of a live bind, or the compact form that the
/// lib bind snapshot keeps. The install writes either into the node records
/// (`ast::bind_store_records`).
// PERF: AST node records, step 2. A live bind hands over its builder: a
// compact form made on the bind thread would be read again by the install,
// and the compaction (`NodeBindParts::new`) was about 0.35% of the
// instructions of `goport -p` on effect.
pub enum BoundNodes {
    Built(NodeBindBuilder),
    Parts(NodeBindParts),
}

impl BoundNodes {


}







// Go: binder/binder.go:374 GetSymbolNameForPrivateIdentifier
//
// PORT: Go `ast.GetSymbolId` hands out lazy global ids. Here the arena index
// is the symbol id. It is stable because every checker arena starts as a
// clone of the binder arena, and it is unique, which is all the name needs
// inside the process. Go binds files in parallel, and each file into its own
// arena here, so a bind cannot give the Go id. Where Go shows `symbol.Name`
// outside the process (the API), `go_symbol_name` puts the id back.




// PERF: size hints for a new members, exports or locals table of a
// container, from its member or statement count, so that filling the table
// does not grow it or rebuild its index. A hint only sizes the table: entry
// order does not change. 0 gives an unsized table.





// Go: core/pattern.go:5 Pattern
//
// PORT: private copy of `core.Pattern` / `core.TryParsePattern` for
// `bindModuleDeclaration`; the contract names no crate module for Go `core`
// helpers. `PatternAmbientModule` stores the prefix and suffix instead.
struct Pattern {
    text: String,
    star_index: i32, // -1 for exact match
}

impl Pattern {
    // Go: core/pattern.go:18 IsValid
    fn is_valid(&self) -> bool {
        self.star_index == -1 || (self.star_index as usize) < self.text.len()
    }
}

// Go: core/pattern.go:10 TryParsePattern

// Go: tspath/extension.go:45 RemoveFileExtension
//
// PORT: private copy; the contract names no crate module for Go `tspath`.
fn remove_file_extension(path: &str) -> &str {
    // Go: tspath/extension.go:43 extensionsToRemove
    const EXTENSIONS_TO_REMOVE: [&str; 12] = [
        ".d.ts", ".d.mts", ".d.cts", ".mjs", ".mts", ".cjs", ".cts", ".ts", ".js", ".tsx", ".jsx",
        ".json",
    ];
    // Remove any known extension even if it has more than one dot
    for ext in EXTENSIONS_TO_REMOVE {
        if let Some(stripped) = path.strip_suffix(ext) {
            return stripped;
        }
    }
    path
}
