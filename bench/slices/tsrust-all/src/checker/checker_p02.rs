//! Port of typescript-go `internal/checker/checker.go` lines 1116-2055.
//!
//! PORT: Go `c.error(...)` returns the `*ast.Diagnostic` it already added, and
//! some Go callers mutate it afterwards (`AddRelatedInfo`). Our `Diagnostic`
//! is owned, so those callers build the diagnostic with
//! `new_diagnostic_for_node`, finish it, then call `add_diagnostic`. That is
//! exactly what Go `c.error` does, in the same order of effects.

use crate::prelude::*;
use std::cell::Cell;

// Go: checker/checker.go:1128 createFileIndexMap
pub fn create_file_index_map(files: &[Node]) -> FxHashMap<Node, i32> {
    let mut result: FxHashMap<Node, i32> = FxHashMap::default();
    result.reserve(files.len());
    for (i, file) in files.iter().enumerate() {
        result.insert(*file, i as i32);
    }
    result
}


// Go: checker/checker.go:1678 isPrimitiveTypeName
pub fn is_primitive_type_name(s: &str) -> bool {
    s == "any" || s == "string" || s == "number" || s == "boolean" || s == "never" || s == "unknown"
}


// Go: checker/checker.go:1745 isES2015OrLaterConstructorName
pub fn is_es2015_or_later_constructor_name(s: &str) -> bool {
    s == "Promise" || s == "Symbol" || s == "Map" || s == "WeakMap" || s == "Set" || s == "WeakSet"
}


/// A spelling suggestion from the globals table, kept by
/// `get_suggestion_for_symbol_name_lookup`.
#[derive(Clone, Copy, Debug)]
pub enum GlobalSpellingSuggestion {
    /// No candidate is close enough.
    NotFound,
    /// A symbol of the globals table.
    Table(SymbolId),
    /// The symbol at this index of `get_primitive_type_alias_suggestions`,
    /// which makes new symbols on each call.
    Extra(usize),
}
