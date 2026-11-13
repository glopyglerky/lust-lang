//! Port of compiler/projectreferenceparser.go, projectreferencefilemapper.go
//! and projectreferencedtsfakinghost.go: loading referenced projects, the
//! source <-> output .d.ts maps, and the resolution host that fakes the
//! output .d.ts files of referenced projects.

use crate::frontend::prelude::*;
use std::time::SystemTime;

// ---------------------------------------------------------------------------
// projectreferenceparser.go
// ---------------------------------------------------------------------------


// Go: projectreferenceparser.go:13 projectReferenceParseTask


// Go: projectreferenceparser.go:34 createProjectReferenceParseTasks

// Go: projectreferenceparser.go:42 projectReferenceParser
// PORT: single-threaded. Go `core.WorkGroup` in single-threaded mode is a
// LIFO stack of closures; `queue` holds the queued tasks in the same order.
// Go `collections.SyncMap` is a plain map.


// ---------------------------------------------------------------------------
// projectreferencefilemapper.go
// ---------------------------------------------------------------------------


/// The part of Go `*fileLoader` that the mapper uses: the non-nil check and
/// `loader.toPath`.
// PORT: Go keeps a `*fileLoader` in the mapper, and the loader keeps the
// mapper. This keeps only the `toPath` inputs, so there is no cycle.
// `fileLoader.toPath` reads `opts.Host` each time; the host values do not
// change while the loader is present.
#[derive(Clone, Debug)]
pub struct MapperLoader {
    pub current_directory: String,
    pub use_case_sensitive_file_names: bool,
}

impl MapperLoader {

    // Go: fileloader.go:187 (*fileLoader).toPath
}

// Go: projectreferencefilemapper.go:14 projectReferenceFileMapper
// PORT: Go `loader *fileLoader` is only used as a non-nil flag and for
// `loader.toPath`; see `MapperLoader`.
// PORT: the Go nil `*ParsedCommandLine` map values are `None`. Go
// `collections.SyncMap` is a `RefCell` map, so lookups take `&self`.
// PORT: `host` is `None` for the Go nil. Go starts it as `opts.Host` (a
// `CompilerHost`, which is a `module.ResolutionHost`);
// `CompilerResolutionHost` wraps it.


// ---------------------------------------------------------------------------
// projectreferencedtsfakinghost.go
// ---------------------------------------------------------------------------




// Go: projectreferencedtsfakinghost.go:16 projectReferenceDtsFakingHost
// PORT: `current_directory` caches Go `h.host.GetCurrentDirectory()`,
// because `ResolutionHost` returns `&str`. The host value does not change.

// Go: projectreferencedtsfakinghost.go:23 newProjectReferenceDtsFakingHost
// PORT: Go copies `loader.dtsDirectories` by value. The set is complete
// when this runs (end of `initMapper`), so a clone is the same.


// Go: projectreferencedtsfakinghost.go:46 projectReferenceDtsFakingVfs
// PORT: the mapper is shared (`Rc<RefCell<..>>`). The loader clears
// `mapper.host` after loading, which breaks the mapper -> host -> vfs ->
// mapper cycle as in Go. `known_symlinks` is mutated from `&self` methods,
// so it is a `RefCell`.


