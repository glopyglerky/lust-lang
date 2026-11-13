//! Go `binder/binder.go` lines 911-1816 (bindFunctionExpression .. bindIterativeStatement).
//!
//! PORT: Go writes binder results straight onto AST nodes and the `SourceFile`.
//! Our AST is read-only while binding, so this file assumes these `Binder`
//! fields (defined in binder_p1.rs) hold the per-file results until
//! `bind_source_file` stores them in the `GoFile` OnceCells:
//! - `node_bind: NodeBindBuilder` indexed by `NodeId::index()` of `self.file` nodes
//!   (Go `node.Symbol`, `node.Locals`, `FlowNodeData().FlowNode`, `EndFlowNode`, and
//!   the binder-added bits of `node.Flags` in `added_flags`).
//! - `flow_node_arena: Vec<FlowNode>` indexed by `FlowNodeId::local_index()`.
//! - `symbol_arena` (Go `symbolArena`): the `SymbolArena` (owned or `&mut`).
//! - `common_js_module_indicator: Node` (Go `file.CommonJSModuleIndicator`, set by the binder).
//! The small module-private helpers below read and write that state.

use crate::astdata::NodeData;
use crate::prelude::*;

// ---------------------------------------------------------------------------
// Module-private access helpers (not Go functions).
// ---------------------------------------------------------------------------













// Go: binder/binder.go:1078 getParentOfPropertyAssignment


// Go: binder/binder.go:1096 getInitializerSymbol
// PORT: Go reads `initializer.Symbol()` from the node; during binding that lives in
// the binder, so this takes the binder instead of only the symbol arena.


// Go: binder/binder.go:1353 isUseStrictPrologueDirective
// Should be called only on prologue directives (ast.IsPrologueDirective(node) should be true)

// Go: binder/binder.go:1360 FindUseStrictPrologue



// Go: binder/binder.go:1449 isEvalOrArgumentsIdentifier
// PERF: U1 (a). Compares name ids (`Node::text_is`), with no text load.
// PERF: binderview1. A published identifier reads its name once for both
// compares (`frozen_store_text_name`).


