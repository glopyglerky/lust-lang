//! Port of Effect-TS/tsgo `internal/rules/prefer_unsafe_constructor.go`.

use crate::effect::diag;
use crate::effect::etscore::*;
use crate::effect::rule::*;
use crate::effect::typeparser::*;
use crate::prelude::*;



/// PreferUnsafeConstructorMatch holds the nodes needed by the diagnostic and quick fix.
// Go: rules/prefer_unsafe_constructor.go PreferUnsafeConstructorMatch
#[derive(Clone, Debug)]
pub struct PreferUnsafeConstructorMatch {
    /// The source file where this match was found
    pub source_file: Node,
    /// Error range on the outer Effect.runSync call
    pub location: TextRange,
    /// The Effect.runSync(...) call expression
    pub outer_call: Node,
    /// The Effect.runSync callee expression
    pub outer_callee: Node,
    /// The constructor call expression (e.g. Scope.make())
    pub inner_call: Node,
    /// The constructor callee expression (e.g. Scope.make)
    pub inner_callee: Node,
    /// The identifier holding the constructor name (e.g. make)
    pub inner_callee_name: Node,
    /// The sibling export name (e.g. makeUnsafe)
    pub unsafe_name: String,
    /// The callee text with the name replaced (e.g. Scope.makeUnsafe)
    pub unsafe_callee_text: String,
}







