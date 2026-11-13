//! Port of Go `ls/callhierarchy.go`.
//!
//! PORT: Go `*compiler.Program` is `&compiler::NewProgram`. Go
//! `program.GetTypeChecker(ctx)` and `GetTypeCheckerForFile(ctx, file)` lease
//! a checker through `ls_program`; the `Release` guard stays alive to the end
//! of the Go function, as Go's `defer done()`.

use crate::ls::prelude::*;

use crate::spanmap::Feature;
use std::cell::OnceCell;

// Go: ls/callhierarchy.go:23 CallHierarchyDeclaration
pub type CallHierarchyDeclaration = Node;

// PORT: Go `findImplementationOrAllInitialDeclarations` and
// `resolveCallHierarchyDeclaration` return `any` holding either a
// `*ast.Node` or a `[]*ast.Node`, and callers switch on the dynamic type.
// This enum holds the two cases. A Go nil `any` is `None` where it can occur.
#[derive(Clone, Debug)]
pub enum CallHierarchyDeclarationOrDeclarations {
    Node(Node),
    Nodes(Vec<Node>),
}

// Go: ls/callhierarchy.go:26 isNamedExpression
// Indictates whether a node is named function or class expression.

// Go: ls/callhierarchy.go:37 isVariableLike

// Go: ls/callhierarchy.go:45 isAssignedExpression
// Indicates whether a node is a function, arrow, or class expression assigned to a constant variable or class property.

// Go: ls/callhierarchy.go:75 isPossibleCallHierarchyDeclaration
// Indicates whether a node could possibly be a call hierarchy declaration.
//
// See `resolveCallHierarchyDeclaration` for the specific rules.

// Go: ls/callhierarchy.go:95 isValidCallHierarchyDeclaration
// Indicates whether a node is a valid a call hierarchy declaration.
//
// See `resolveCallHierarchyDeclaration` for the specific rules.

// Go: ls/callhierarchy.go:120 getCallHierarchyDeclarationReferenceNode
// Gets the node that can be used as a reference to a call hierarchy declaration.

// Go: ls/callhierarchy.go:149 getSymbolOfCallHierarchyDeclaration
// Gets the symbol for a call hierarchy declaration.

// Go: ls/callhierarchy.go:161 getCallHierarchyItemName
// Gets the text and range for the name of a call hierarchy declaration.
// PORT: Go named results `(text string, pos int, end int)` are a tuple.

// Go: ls/callhierarchy.go:221 getTextOfCallHierarchyName

// Go: ls/callhierarchy.go:250 getCallHierarchyItemContainerName

// Go: ls/callhierarchy.go:298 moveRangePastModifiers

// Go: ls/callhierarchy.go:307 findImplementation
// Finds the implementation of a function-like declaration, if one exists.

// Go: ls/callhierarchy.go:337 findAllInitialDeclarations
// PORT: Go returns nil or a non-empty slice; nil is an empty `Vec`.

// Go: ls/callhierarchy.go:388 findImplementationOrAllInitialDeclarations
// Find the implementation or the first declaration for a call hierarchy declaration.
// PORT: Go returns `any` (see `CallHierarchyDeclarationOrDeclarations`).

// Go: ls/callhierarchy.go:410 resolveCallHierarchyDeclaration
// Resolves the call hierarchy declaration for a node.
// PORT: Go returns `any`; a nil result is `None`.


// Go: ls/callhierarchy.go:533 callSite
#[derive(Clone, Copy, Debug)]
pub struct CallSite {
    pub declaration: Node,
    pub text_range: TextRange,
    pub source_file: Node,
}

// Go: ls/callhierarchy.go:539 convertEntryToCallSite

// Go: ls/callhierarchy.go:568 getCallSiteGroupKey
// PORT: Go `ast.NodeId` is the `u64` from `get_node_id`.


// Go: ls/callhierarchy.go:593 incomingEntry
// PORT: the three `sync.Once` + value pairs are `OnceCell` fields filled on
// first use.


// Go: ls/callhierarchy.go:607 var _ lsproto.HasTextDocumentPosition = (*incomingEntry)(nil)



// Go: ls/callhierarchy.go:711 callSiteCollector


// Go: ls/callhierarchy.go:869 collectCallSites
// PORT: Go passes `c *checker.Checker`, leased by `getOutgoingCalls`. The
// collector calls `resolveCallHierarchyDeclaration`, which leases the program
// checker again (Go shares one checker between both leases). A Rust
// `RefCell` borrow held across that call would be a second borrow, so this
// takes the leased `Rc` and borrows it only for `findImplementation`.

