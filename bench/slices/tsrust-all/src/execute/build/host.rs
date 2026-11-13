//! Go: execute/build/host.go, execute/build/compilerHost.go, and the
//! `ExtendedConfigCache` of execute/tsc/extendedconfigcache.go.
//!
//! PORT: Go `host` keeps a pointer to its `*Orchestrator` and reads
//! `opts.Sys`, `opts.Command` and `toPath` through it. Here the host keeps
//! those values itself, so it needs no reference back to the orchestrator.
//! The orchestrator owns the host as `Rc<BuildHost>` and passes clones
//! where Go passes `o.host`.
//!
//! PORT: Go `time.Time` is `Option<SystemTime>` (`None` = zero) and
//! `time.Duration` is `Duration`, as in build_task.rs.

use crate::contentmapper::{self, Mapper, };
use crate::execute::build::parse_cache::ParseCache;
use crate::execute::incremental::incremental;
use crate::frontend::prelude::*;
use crate::gostd::GoError;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, SystemTime};

// Go: tsc/extendedconfigcache.go:15 ExtendedConfigCache
// PORT: the Go type is `tsc.ExtendedConfigCache`; the Rust name adds `Tsc`
// because the `tsoptions.ExtendedConfigCache` interface already has the
// plain name. The per-entry mutex is dropped (one thread, see
// parse_cache.rs). The map borrow is not held while the config parses, so
// a nested `extends` can use the cache.



// PORT: Go keys the source file cache by `ast.SourceFileParseOptions`, a
// comparable struct. The Rust struct has no `Hash`, so this key hashes the
// same fields.
#[derive(Clone, PartialEq, Eq)]
pub struct SourceFileCacheKey(pub SourceFileParseOptions);


/// PORT: not in Go. What the module indicator options of a parse take from
/// the compiler options (Go `GetExternalModuleIndicatorOptions`,
/// parseoptions.go:19; the rest is the file name and its package.json):
/// the detection kind, and with `auto` only, whether `jsx` is react-jsx or
/// react-jsxdev and whether the module resolution is in Node16..NodeNext,
/// which reads the package.json `type` (`loadSourceFileMetaData`,
/// fileloader.go:384). `module` has an effect only through the two
/// default kinds here: with no `moduleDetection`, a module kind in
/// Node16..NodeNext gives `force` (`GetEmitModuleDetectionKind`,
/// compileroptions.go:243), and with no `moduleResolution` (or classic or
/// node10) the module kind picks the resolution kind
/// (`GetModuleResolutionKind`, compileroptions.go:227). The module kind
/// that `isFileForcedToBeModuleByFormat` passes on is not an input: once
/// the `type` is known, `GetImpliedNodeFormatForEmitWorker`
/// (utilities.go:2622) gives ESNext to the same files with any module
/// kind. Two configs with the same inputs
/// give each file the same module indicator options. See
/// `BuildHost::drop_kept_parses_whose_module_indicator_options_change`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ModuleIndicatorInputs {
    detection: ModuleDetectionKind,
    jsx: bool,
    node_resolution: bool,
}

impl ModuleIndicatorInputs {

}

impl Hash for SourceFileCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.file_name.hash(state);
        self.0.path.hash(state);
        self.0.external_module_indicator_options.jsx.hash(state);
        self.0.external_module_indicator_options.force.hash(state);
    }
}

// Go: vfs/cachedvfs/cachedvfs.go FS, the file system of the build host
// (`cachedvfs.From(sys.FS())`, orchestrator.go:1004).
// PORT: the same cache as `CachedFs` (always enabled: the build host
// never disables it), but the `FileExists`, `DirectoryExists`,
// `Realpath` and `GetAccessibleEntries` lookups live in a
// `BuildStatCache` that the parse workers of each program load read too
// (`CompilerHost::stat_cache`), as Go parse tasks share the host's
// cachedvfs. This cache lasts for the whole build, and a write does not
// update it (cachedvfs.go:144), so a later program can find a lookup here
// that an earlier program made before the build wrote that path. It holds
// only the lookups that Go makes: the workers' own lookups stay out of it
// unless the loader uses them (see `BuildStatCache`).



// Go: build/host.go:18 host

/// PORT: not in Go (perf). A set of paths (`BuildHost::written`), with a
/// check that takes no lock while the set is empty (a build that writes
/// nothing, such as a noop build).
#[derive(Default)]
pub struct WrittenPaths {
    any: std::sync::atomic::AtomicBool,
    paths: Mutex<FxHashSet<Path>>,
}

impl WrittenPaths {


}



// PORT: Go passes the `*host` as a `tsoptions.ParseConfigHost` (it has
// `FS()` and `GetCurrentDirectory()`). Rust needs the explicit impl.

// Go: build/host.go:35 `_ incremental.Host = (*host)(nil)`

// Go: build/compilerHost.go:13 compilerHost
// PORT: the host that the build task gives `compiler.NewProgram`: the
// build host with the task's trace writer. Go nil `contentMapperProject`
// is `None`.

