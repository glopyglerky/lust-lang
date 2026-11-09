//! Port of `transformers/declarations/diagnostics.go`.

use crate::prelude::*;
use crate::printer::{SymbolAccessibility, SymbolAccessibilityResult};

// Go: transformers/declarations/diagnostics.go:10 GetSymbolAccessibilityDiagnostic
// PORT: a Go func value is an `Rc<dyn Fn>` so the transformer can save and
// restore it. A nil Go func is `None` at the use sites.

// PORT: Go passes the selectors as func values; they are plain fns here.

// Go: transformers/declarations/diagnostics.go:12 SymbolAccessibilityDiagnostic

// Go: transformers/declarations/diagnostics.go:18 wrapSimpleDiagnosticSelector

// Go: transformers/declarations/diagnostics.go:32 wrapNamedDiagnosticSelector

// Go: transformers/declarations/diagnostics.go:47 wrapFallbackErrorDiagnosticSelector

// Go: transformers/declarations/diagnostics.go:64 selectDiagnosticBasedOnModuleName

// Go: transformers/declarations/diagnostics.go:74 selectDiagnosticBasedOnModuleNameNoNameCheck

// Go: transformers/declarations/diagnostics.go:81 createGetSymbolAccessibilityDiagnosticForNodeName
// PERF: the Go `ast.Is*` tests are pure kind compares, so one match on the kind
// picks the same branch.

// Go: transformers/declarations/diagnostics.go:91 getAccessorNameVisibilityDiagnosticMessage

// Go: transformers/declarations/diagnostics.go:115 getMethodNameVisibilityDiagnosticMessage

// Go: transformers/declarations/diagnostics.go:139 createGetSymbolAccessibilityDiagnosticForNode
// PERF: Go chains up to 23 `ast.Is*` tests. Each one is a pure kind compare
// and the branches test disjoint kinds, so one match on the kind picks the same
// branch. The arms keep the Go order.

// Go: transformers/declarations/diagnostics.go:223 getVariableDeclarationTypeVisibilityDiagnosticMessage

// Go: transformers/declarations/diagnostics.go:263 getAccessorDeclarationTypeVisibilityDiagnosticMessage

// Go: transformers/declarations/diagnostics.go:299 getReturnTypeVisibilityDiagnosticMessage

// Go: transformers/declarations/diagnostics.go:358 getParameterDeclarationTypeVisibilityDiagnosticMessage

// Go: transformers/declarations/diagnostics.go:436 getTypeParameterConstraintVisibilityDiagnosticMessage

// Go: transformers/declarations/diagnostics.go:471 getRelatedSuggestionByDeclarationKind

// Go: transformers/declarations/diagnostics.go:502 getErrorByDeclarationKind

// Go: transformers/declarations/diagnostics.go:543 isDeclarationEnoughForErrors

// Go: transformers/declarations/diagnostics.go:547 isFunctionLikeAndNotConstructor

// Go: transformers/declarations/diagnostics.go:551 findNearestDeclaration

// PORT: Go `createDiagnosticForNode` (tracker.go:209) takes a
// `*diagnostics.Message` that can be nil when a kind lookup misses. Here the
// lookups return `Option`, and a nil message would crash in Go when the
// diagnostic is formatted, so `None` panics at creation.

// Go: transformers/declarations/diagnostics.go:568 createEntityInTypeNodeError

// Go: transformers/declarations/diagnostics.go:574 addParentDeclarationRelatedInfo

// Go: transformers/declarations/diagnostics.go:586 createAccessorTypeError
// PORT: Go reads `node.Symbol().Declarations`. The declarations module has no
// checker, so it reads the binder symbol from the program's binder symbols
// (`program::bound_symbols`).


// Go: transformers/declarations/diagnostics.go:604 createObjectLiteralError

// Go: transformers/declarations/diagnostics.go:610 createArrayLiteralError

// Go: transformers/declarations/diagnostics.go:616 createReturnTypeError

// Go: transformers/declarations/diagnostics.go:623 createBindingElementError

// Go: transformers/declarations/diagnostics.go:627 createVariableOrPropertyError

// Go: transformers/declarations/diagnostics.go:633 createExpressionError

// Go: transformers/declarations/diagnostics.go:637 createClassExpressionError

// Go: transformers/declarations/diagnostics.go:641 isParentForIDDIagnostic

// Go: transformers/declarations/diagnostics.go:651 createExpressionErrorEx


// Go: transformers/declarations/diagnostics.go:683 createGetIsolatedDeclarationErrors

// Go: transformers/declarations/tracker.go:235 createDiagnosticForNode
