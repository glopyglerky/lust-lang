//! Port of Go `internal/transformers/declarations/supplementalreferences.go`
//! (tsgo#4712).

use crate::frontend::tspath;
use crate::prelude::*;

// Go: transformers/declarations/supplementalreferences.go:12 SupplementalReferencesTransformer

// Go: transformers/declarations/supplementalreferences.go:19 NewSupplementalReferencesTransformer
// PORT: Go takes the source file and reads `sourceFile.SupplementalSourceFiles()`.
// The emitter holds the source file as a node, and the content mapper info is
// on the parsed file, so the caller passes the supplemental files (their
// `SourceFile` nodes, in Go order) instead of the source file.

