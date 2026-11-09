//! Port of Go `ls/sourcedefinition.go`.
//!
//! PORT: Go `*compiler.Program` is `&compiler::NewProgram`.
//! `getOrParseSourceFile` parses and binds a file outside the program as Go
//! does; the file is published with no program first (node reads of a file
//! need its published Go file).

use crate::ls::prelude::*;

use crate::spanmap::Feature;


// Go: ls/sourcedefinition.go:132 sourceDefResolver
// sourceDefResolver resolves source definitions by mapping .d.ts declarations
// to their implementation files (.js/.ts). It uses the NoDts module resolver
// and file parsing for resolution, but never acquires the type checker or
// the original program; all checker-dependent work is done before results
// are passed in.
// PORT: Go `*sourceDefResolver` is owned by its one request; methods that
// fill `parsedFiles` take `&mut self`. Go `parsedFiles` is a nil map until
// the first store, so it is an `Option`.



// Go: ls/sourcedefinition.go:203 getSourceDefCheckerInfo
// getSourceDefCheckerInfo acquires the type checker for the given file and
// returns the definition declarations for node along with the module specifier
// of the import that brought the symbol into scope (empty if not applicable).


// Go: ls/sourcedefinition.go:317 isDefaultImportName

// Go: ls/sourcedefinition.go:324 getSourceDefinitionEntryNode

// Go: ls/sourcedefinition.go:331 getSourceDefinitionEntryDeclarations


// Go: ls/sourcedefinition.go:475 findContainingModuleSpecifier


// Go: ls/sourcedefinition.go:534 getCandidateSourceDeclarationNames

// Go: ls/sourcedefinition.go:572 findDeclarationNodesByName

// Go: ls/sourcedefinition.go:634 getContainerDepth
// getContainerDepth counts the number of container nodes above a declaration,
// matching the behavior of getDepth in getTopMostDeclarationNamesInFile.

// Go: ls/sourcedefinition.go:644 filterPreferredSourceDeclarations

// Go: ls/sourcedefinition.go:657 getPropertyLikeSourceDeclarations

// Go: ls/sourcedefinition.go:679 hasConcreteSourceDeclarations

// Go: ls/sourcedefinition.go:683 isConcreteSourceDeclaration

// Go: ls/sourcedefinition.go:706 uniqueDeclarationNodes

// Go: ls/sourcedefinition.go:727 findClosestDeclarationNode
