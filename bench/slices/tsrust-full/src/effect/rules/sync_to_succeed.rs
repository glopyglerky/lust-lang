//! Port of Effect-TS/tsgo `internal/rules/sync_to_succeed.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// SyncToSucceedMatch holds the nodes needed by the diagnostic and quick fix.
// Go: rules/sync_to_succeed.go SyncToSucceedMatch
#[derive(Clone, Copy)]
pub struct SyncToSucceedMatch {
    pub source_file: Node,
    pub location: TextRange,
    pub callee: Node,
    /// Replaceable property name; nil for named imports and local aliases.
    pub callee_name: Node,
    pub thunk: Node,
    pub constant_value: Node,
}

