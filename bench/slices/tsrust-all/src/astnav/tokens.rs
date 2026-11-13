//! Port of Go `astnav/tokens.go`.
//!
//! PORT: Go closures that update captured locals become closures over
//! `Cell`/`RefCell` locals, because `ast::NodeVisitor` hooks are `Fn`.
//! Recursive Go closures (`find`) become nested `fn`s that take the captured
//! values as parameters. Scanning uses the literal Go scanner through
//! `scanner_ls::get_scanner_for_source_file`.

use crate::astnav::prelude::*;
use crate::frontend::core_binarysearch::binary_search_unique_func;
use crate::frontend::scanner::scanner_ls;
use std::cell::Cell;

// Go: astnav/tokens.go:12 shouldRescanLessThanLessThanToken

// Go: astnav/tokens.go:16 scanNavigationToken

// Go: astnav/tokens.go:24 GetTouchingPropertyName

// Go: astnav/tokens.go:30 GetTouchingToken

// Go: astnav/tokens.go:34 GetTokenAtPosition


// PORT: Go `getTokenAtPosition` has the same snake name as the exported
// `GetTokenAtPosition`. The exported one keeps the plain name because other
// packages call it; this private one gets the `_unexported` suffix.
// Go: astnav/tokens.go:38 getTokenAtPosition

// Go: astnav/tokens.go:276 getPosition

// Go: astnav/tokens.go:283 findRightmostNode

// Go: astnav/tokens.go:312 VisitEachChildAndJSDoc
// PORT: Go `func(*ast.Node, *ast.NodeVisitor) *ast.Node` callbacks (nil
// allowed) are `Option<&dyn Fn>`. Keep callback state in `Cell`/`RefCell`.

// Go: astnav/tokens.go:330 comparisonLessThan
const COMPARISON_LESS_THAN: i32 = -1;
// Go: astnav/tokens.go:331 comparisonEqualTo
const COMPARISON_EQUAL_TO: i32 = 0;
// Go: astnav/tokens.go:332 comparisonGreaterThan
const COMPARISON_GREATER_THAN: i32 = 1;

// Go: astnav/tokens.go:339 FindPrecedingToken

// Go: astnav/tokens.go:343 FindPrecedingTokenEx

// Go: astnav/tokens.go:465 isValidPrecedingNode

// Go: astnav/tokens.go:474 GetStartOfNode

// Go: astnav/tokens.go:480 findRightmostValidToken

// Go: astnav/tokens.go:622 FindNextToken

// Go: astnav/tokens.go:691 getNodeVisitor
// PORT: Go `ast.NewNodeVisitor(core.Identity, nil, hooks)`: the visit
// callback is the identity and the factory is the visitor's default one.

// Go: astnav/tokens.go:728 shouldSkipChild

// Go: astnav/tokens.go:739 FindChildOfKind
