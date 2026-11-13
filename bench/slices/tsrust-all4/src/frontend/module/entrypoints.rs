//! Go `internal/module/resolver.go` lines 2127 to 2366 (pin N): `Ending`,
//! `ResolvedEntrypoint`, `GetEntrypointsFromPackageJsonInfo`,
//! `createResolvedEntrypointHandlingSymlink`, `loadEntrypointsFromExportMap`
//! and `getMatchedStarForPatternEntrypoint`. Only `ls/autoimport` and
//! `modulespecifiers.ProcessEntrypointEnding` (language service) use them.
//!
//! PORT: shapes follow `resolver_p1.rs` and `resolver_p2.rs`: Go
//! `*packagejson.InfoCacheEntry` is `Option<Rc<InfoCacheEntry>>`, Go
//! `*resolved` is `Option<Resolved>`. Go `*collections.Set[string]` is
//! `Option<FxHashSet<String>>` (`None` is nil). Go `*ResolvedEntrypoint`
//! values are shared (`Rc<ResolvedEntrypoint>`).

use crate::frontend::prelude::*;

use crate::flags_macros::go_enum;

/// Go `(*resolved).isResolved` on a possibly nil `*resolved`.
fn is_resolved(resolved: &Option<Resolved>) -> bool {
    resolved.as_ref().is_some_and(|r| !r.path.is_empty())
}

// Go: module/resolver.go:2127 Ending
go_enum!(Ending, i32 {
    // EndingFixed indicates that the module specifier cannot be changed without changing its resolution.
    FIXED = 0; // EndingFixed
    // EndingExtensionChangeable indicates that the module specifier's extension portion was inferred from a
    // file on disk, so an interchangeable one could be used instead (e.g. replacing .d.ts with .js).
    EXTENSION_CHANGEABLE = 1; // EndingExtensionChangeable
    // EndingChangeable indicates that the module specifier's file name and extension portion were inferred
    // from a file on disk without being matched as part of an 'exports' pattern, so can be changed according
    // to the importer's module resolution rules (e.g. an /index.d.ts may be dropped entirely in CommonJS settings).
    CHANGEABLE = 2; // EndingChangeable
});

// Go: module/resolver.go:2141 ResolvedEntrypoint
#[derive(Clone, Debug, Default)]
pub struct ResolvedEntrypoint {
    // OriginalFileName is the symlink path if the entrypoint was discovered at a symlink. Empty otherwise.
    pub original_file_name: String,
    // ResolvedFileName is the real path to the entrypoint file.
    pub resolved_file_name: String,
    pub module_specifier: String,
    // Ending indicates whether the file name and extension portion of ModuleSpecifier is fixed or can be changed.
    pub ending: Ending,
    // IncludeConditions are the conditions that a resolver must have to reach this entrypoint.
    pub include_conditions: Option<FxHashSet<String>>,
    // ExcludeConditions are the conditions that a resolver must not have to reach this entrypoint.
    pub exclude_conditions: Option<FxHashSet<String>>,
}

impl ResolvedEntrypoint {
    // Go: module/resolver.go:2155 SymlinkOrRealpath
    #[must_use]
    pub fn symlink_or_realpath(&self) -> String {
        if !self.original_file_name.is_empty() {
            return self.original_file_name.clone();
        }
        self.resolved_file_name.clone()
    }
}


