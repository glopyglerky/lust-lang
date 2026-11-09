//! Go: tsoptions/contentmappers.go (tsgo#4712).

use crate::contentmapper::Manifest;
use crate::frontend::prelude::*;

// Go: tsoptions/contentmappers.go:17 resolveContentMapperManifest
// resolveContentMapperManifest locates packageName in node_modules (walking up from the directory of
// containingFile via node module resolution) and reads its package.json to produce the mapper's manifest
// and package directory. It never executes the package. On failure it returns a diagnostic describing why
// the mapper could not be resolved; on success the diagnostic is nil.
// PORT: Go nil `*ast.Diagnostic` is `None`. Go passes the `ParseConfigHost`
// as the resolution host; `ResolverHost` keeps its `FS()` and
// `GetCurrentDirectory()` values, as for `extends`.
