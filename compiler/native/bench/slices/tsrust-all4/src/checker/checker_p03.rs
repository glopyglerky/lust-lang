//! Port of Go `checker/checker.go` lines 2056-2967.

use crate::gostd::Context;
use crate::prelude::*;

// Go: checker/checker.go:2113 isImmediatelyUsedInInitializerOfBlockScopedVariable

// Go: checker/checker.go:2131 isSameScopeDescendentOf
// Starting from 'initial' node walk up the parent chain until 'stopAt' node is reached.
// If at any point current node is equal to 'parent' node - return true.
// If current node is an IIFE, continue walking up.
// Return false if 'stopAt' node is reached or isFunctionLike(current) === true.
// PERF: chkA. One walk up the store tables (`find_ancestor_with_kind`,
// which gives each kind), in place of a `parent()` and a `kind()` lookup per
// step. The tests and their order are Go's.

// Go: checker/checker.go:2147 isPropertyImmediatelyReferencedWithinDeclaration
// isPropertyImmediatelyReferencedWithinDeclaration is used for detecting ES-standard class field use-before-def errors

// PORT: Go `node.AsTypeParameterDeclaration().Expression`. The Go `Node.Expression()`
// method panics on type parameters, and fields.rs skips the clashing field name, so
// the field is read from the astdata data directly.


// Go: checker/checker.go:2952 isInstancePropertyWithInitializerOrPrivateIdentifierProperty

// Go: checker/checker.go:2956 superCallIsRootLevelInConstructor

// Go: checker/checker.go:2961 nodeImmediatelyReferencesSuperOrThis


// Go: ast/utilities.go:4271 IsPotentiallyExecutableNode
