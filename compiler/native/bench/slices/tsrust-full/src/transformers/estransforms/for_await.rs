//! Port of Go `transformers/estransforms/forawait.go`.

use super::contract::{};
use crate::prelude::*;
use crate::printer::{
    AutoGenerateOptions, EmitFlags,
    GeneratedIdentifierFlags,
};

// Go: transformers/estransforms/forawait.go:12 forAwaitHierarchyFacts
/// Facts we track as we traverse the tree
type ForAwaitHierarchyFacts = i32;

const FOR_AWAIT_HIERARCHY_FACTS_NONE: ForAwaitHierarchyFacts = 0;

//
// Ancestor facts
//

const FOR_AWAIT_HIERARCHY_FACTS_HAS_LEXICAL_THIS: ForAwaitHierarchyFacts = 1 << 0;
const FOR_AWAIT_HIERARCHY_FACTS_ITERATION_CONTAINER: ForAwaitHierarchyFacts = 1 << 1;

//
// Ancestor masks
//

// PORT: Go `1<<iota - 1` at iota 2.
const FOR_AWAIT_HIERARCHY_FACTS_ANCESTOR_FACTS_MASK: ForAwaitHierarchyFacts = (1 << 2) - 1;

const FOR_AWAIT_HIERARCHY_FACTS_SOURCE_FILE_EXCLUDES: ForAwaitHierarchyFacts =
    FOR_AWAIT_HIERARCHY_FACTS_ITERATION_CONTAINER;
const FOR_AWAIT_HIERARCHY_FACTS_STRICT_MODE_SOURCE_FILE_INCLUDES: ForAwaitHierarchyFacts =
    FOR_AWAIT_HIERARCHY_FACTS_NONE;

const FOR_AWAIT_HIERARCHY_FACTS_CLASS_OR_FUNCTION_INCLUDES: ForAwaitHierarchyFacts =
    FOR_AWAIT_HIERARCHY_FACTS_HAS_LEXICAL_THIS;
const FOR_AWAIT_HIERARCHY_FACTS_CLASS_OR_FUNCTION_EXCLUDES: ForAwaitHierarchyFacts =
    FOR_AWAIT_HIERARCHY_FACTS_ITERATION_CONTAINER;

const FOR_AWAIT_HIERARCHY_FACTS_ARROW_FUNCTION_INCLUDES: ForAwaitHierarchyFacts =
    FOR_AWAIT_HIERARCHY_FACTS_NONE;
const FOR_AWAIT_HIERARCHY_FACTS_ARROW_FUNCTION_EXCLUDES: ForAwaitHierarchyFacts =
    FOR_AWAIT_HIERARCHY_FACTS_CLASS_OR_FUNCTION_EXCLUDES;

const FOR_AWAIT_HIERARCHY_FACTS_ITERATION_STATEMENT_INCLUDES: ForAwaitHierarchyFacts =
    FOR_AWAIT_HIERARCHY_FACTS_ITERATION_CONTAINER;
const FOR_AWAIT_HIERARCHY_FACTS_ITERATION_STATEMENT_EXCLUDES: ForAwaitHierarchyFacts =
    FOR_AWAIT_HIERARCHY_FACTS_NONE;

// Go: transformers/estransforms/forawait.go:43 forawaitTransformer


// Go: transformers/estransforms/forawait.go:56 newforawaitTransformer

/// Go `printer.AutoGenerateOptions{Flags: GeneratedIdentifierFlagsReservedInNestedScopes}`.
fn reserved_in_nested_scopes() -> AutoGenerateOptions {
    AutoGenerateOptions {
        flags: GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES,
        ..Default::default()
    }
}


// Go: transformers/estransforms/forawait.go:308 unwrapInnermostStatementOfLabel
