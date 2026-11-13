//! Port of Go `checker/checker.go` lines 15735-16703: qualified name
//! resolution, module exports and late binding, alias resolution, symbol
//! flags, and the type of a symbol (`getTypeOfSymbol` and its helpers,
//! `getTypeForVariableLikeDeclaration`).
//!
//! PORT: Go symbol tables are maps with reference semantics; here they are
//! `SymbolTable` handles into `self.symbols`. `make(ast.SymbolTable)` is
//! `self.symbols.new_table()`, `t[k]` is `self.symbols.get(t, k)` and
//! `t[k] = v` is `self.symbols.set(t, k, v)`.

use crate::prelude::*;

// Go: checker/checker.go:16460 ExportCollision
// PERF: Go stores `specifierText`, the text of the first export node's
// module specifier. We store that export node and make the text only when a
// TS2308 diagnostic uses it. Most entries never report, so this saves one
// String per re-exported name.
#[derive(Clone, Debug, Default)]
pub struct ExportCollision {
    pub export_node: Node,
    pub exports_with_duplicate: Vec<Node>,
}

// Go: checker/checker.go:16465 ExportCollisionTable
// PORT: Go `map[string]*ExportCollision`. Go iterates this map to report
// diagnostics; Go map order is random, so we keep insertion order
// (`IndexMap`) to be deterministic. Diagnostics are sorted later.
// PERF: keyed by interned `Name`, so an insert copies no text. The Fx hasher
// does not change the order: an `IndexMap` iterates in insertion order.
pub type ExportCollisionTable = FxIndexMap<Name, ExportCollision>;

// PORT: state captured by the Go `visit` closure in `getExportsOfModuleWorker`.

// Go: checker/utilities.go:217 entityNameToString
// PORT: the checker wrapper `entityNameToString(name)` has the same snake
// name as the ast function `EntityNameToString(name, getTextOfNode)` that is
// already ported as `entity_name_to_string(name, Option<..>)`. To avoid the
// glob-import clash, this file calls the ast function with
// `scanner.GetTextOfNode`, which is exactly what the checker wrapper does.

