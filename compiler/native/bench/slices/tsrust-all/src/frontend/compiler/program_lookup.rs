//! Port of compiler/program.go lines 1788 to 2190: file lookup, include
//! explanations, resolution lookups, package name collection, the symlink
//! cache and the `plainJSErrors` set.

use crate::frontend::prelude::*;
use std::sync::{Arc, OnceLock};

// Go: program.go:79 packageNamesInfo
// PORT: U21 dropped the `packageNames` field and this type. It is here
// because only `collectPackageNames` uses it. Go `*collections.Set[string]`
// is an `FxHashSet<String>`.
#[derive(Clone, Debug, Default)]
pub struct PackageNamesInfo {
    pub resolved: FxHashSet<String>,
    pub unresolved: FxHashSet<String>,
    pub deep_import_packages: FxHashSet<String>,
}

/// Go callback `func(resolution T, moduleName string, mode core.ResolutionMode, filePath tspath.Path)`.
pub type ResolutionCallback<'f, T> = dyn FnMut(&T, &str, ResolutionMode, &Path) + 'f;


// Go: program.go:2357 forEachResolution

// Go: program.go:2373 plainJSErrors
// PORT: Go package-level set; built once on first use.

// Go: symlinks/knownsymlinks.go:81 (*KnownSymlinks).SetSymlinksFromResolutions
// PORT: the method lives here because it needs frontend resolution types.


// Go: emitter.go:493 sourceFileMayBeEmitted
// PORT: the Go host is `SourceFileMayBeEmittedHost`; the program is the only
// host the frontend uses.

// Go: emitter.go:553 getSourceFilesToEmit
// PORT: Go nil `targetSourceFiles` is `None` (#4699: a slice of targets and
// `forceJsEmit`).

// Go: `*Program` implements `outputpaths.OutputPathsHost`.

