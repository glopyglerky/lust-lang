//! Go `internal/binder/binder.go` lines 1817-2795: statement and expression
//! control flow binding, narrowing predicates, declaration helpers and binder
//! diagnostics.

use crate::astdata::NodeData;
use crate::prelude::*;

// PORT: Binder storage used by this file. Go stores binder output on the AST
// nodes and flow nodes directly. The Rust `Binder` (defined in binder_p1)
// holds that output until `bind_source_file` fills the `GoFile` cells:
// - `self.flow_nodes: Vec<FlowNode>`, indexed by `FlowNodeId::local_index()`.
// - `self.node_bind: NodeBindBuilder`, indexed by `node.node_id().index()`.
// - `self.file_bind: FileBindData` (bind diagnostics and the other file data).
// - `self.symbols: SymbolArena` (Go `symbolArena`).
// - `self.active_label_list: Option<Rc<RefCell<ActiveLabel>>>`.
// All direct access goes through the `p3_*` helpers below so a storage
// mismatch is fixed in one place.

// PORT: Go `node.FlowNodeData() != nil`. True for node kinds whose Go data
// embeds `FlowNodeBase` (StatementBase, IterationStatementBase,
// AccessorDeclarationBase, Identifier, QualifiedName, BindingElement,
// MethodDeclaration, KeywordExpression, ArrowFunction, FunctionExpression,
// PropertyAccessExpression, ElementAccessExpression, MetaProperty).

// Go: binder/binder.go:1818 isLogicalAssignmentExpression


// Go: binder/binder.go:2531 isGeneratorFunctionExpression


// Go: binder/binder.go:2560 SetValueDeclaration

// Go: binder/binder.go:2580 GetContainerFlags


// Go: binder/binder.go:2624 isNarrowingExpression

// Go: binder/binder.go:2642 containsNarrowableReference

// Go: binder/binder.go:2655 isNarrowableReference

// Go: binder/binder.go:2673 hasNarrowableArgument

// Go: binder/binder.go:2688 isNarrowingBinaryExpression
// PORT: Go takes `*ast.BinaryExpression`; this takes its fields, read once
// (`BinaryView`).

// Go: binder/binder.go:2708 isNarrowableOperand

// Go: binder/binder.go:2724 isNarrowingTypeOfOperands


// Go: binder/binder.go:2756 getOptionalSymbolFlagForNode
// PERF: query Q7-3. `d` is the data of `node`, which the caller already
// loaded with `parsed_node_data`.

// Go: binder/binder.go:2761 isFunctionSymbol

// Go: binder/binder.go:2777 isStatementCondition

// Go: binder/binder.go:2789 isTopLevelLogicalExpression

// Go: binder/binder.go:2796 isAssignmentDeclaration

// Go: binder/binder.go:2800 isEffectiveModuleDeclaration
