//! Port of typescript-go `internal/checker/flow.go` lines 953-1854
//! (`getInstanceType` through `getReferenceCandidate`).
//!
//! PORT: Go `*FlowState` is `Rc<RefCell<FlowState>>` (see `Checker::free_flow_state`),
//! so flow functions take `f: &Rc<RefCell<FlowState>>`. Borrows of `f` are never
//! held across calls into the checker.

use crate::prelude::*;
use std::borrow::Cow;
use std::sync::LazyLock;


// Go: checker/flow.go:1496 getCandidateVariableDeclarationInitializer


/// PERF: not in Go. What `is_matching_reference` read about its last
/// identifier `source`. Flow analysis compares one reference with each flow
/// node on its path, so the same `source` comes again and again.
///
/// Each value is stable once read: `is_this_in_type_query` only reads the
/// tree, and `get_resolved_symbol` stores its answer in the node links, which
/// nothing overwrites later. The export symbol depends on the merged symbol
/// table and on the flags a merge can add, so it is read again after any
/// merge (`Checker::merge_version`). A nested `is_matching_reference` call
/// (during the first resolution of `node`) can replace the memo, so a value
/// is stored only while the memo still holds its node.
#[derive(Clone, Copy, Debug, Default)]
pub struct MatchingReferenceMemo {
    /// The identifier; nil when the memo is empty.
    pub node: Node,
    /// Go `ast.IsThisInTypeQuery(node)`.
    pub this_in_type_query: bool,
    /// Go `c.getResolvedSymbol(node)`; nil until read.
    pub resolved: SymbolId,
    /// Go `c.getExportSymbolOfValueSymbolIfExported(resolved)`; nil until read.
    pub export: SymbolId,
    /// `Checker::merge_version` when `export` was read.
    pub export_merge_version: u64,
}

// Go: checker/flow.go:1651 nonDottedNameCacheKey
// PORT: Go package var `CacheHashKey(xxh3.HashString128("?"))`, computed through
// `KeyBuilder` like `SIGNATURE_KEY_*` in checker_p01. Read with `*NON_DOTTED_NAME_CACHE_KEY`.


// Go: checker/flow.go:1782 tryGetNameFromType
// PORT: Go package function that reads type data. The contract makes such
// functions `Checker` methods, but `Checker::try_get_name_from_type` already
// exists (Go method `(*Checker).tryGetNameFromType`, checker.go:18621, which
// behaves differently). This stays a free function that takes the checker.

