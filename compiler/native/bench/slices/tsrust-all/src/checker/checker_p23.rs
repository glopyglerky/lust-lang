//! Port of `checker/checker.go` lines 20351-21263 (reportErrorsFromWidening
//! through appendIndexInfo).

use crate::prelude::*;
use crate::smallvec::SmallVec;


// A variable-like declaration is free of this references if it has a type annotation
// that is thisless, or if it has no type annotation and no initializer (and is thus of type any).
// Go: checker/checker.go:21154 isThislessVariableLikeDeclaration

// A type is free of this references if it's the any, string, number, boolean, symbol, or void keyword, a string
// literal type, an array with an element type that is free of this references, or a type reference that is
// free of this references.
// Go: checker/checker.go:21165 isThislessType

// A function-like declaration is considered free of `this` references if it has a return type
// annotation that is free of this references and if each parameter is thisless and if
// each type parameter (if present) is thisless.
// Go: checker/checker.go:21181 isThislessFunctionLikeDeclaration

// A type parameter is thisless if its constraint is thisless, or if it has no constraint. */
// Go: checker/checker.go:21189 isThislessTypeParameter

