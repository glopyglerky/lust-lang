//! Port of Go `internal/symlinks/knownsymlinks.go`.

use crate::prelude::*;

use super::tspath;
use super::tspath::Path;

// Go: symlinks/knownsymlinks.go:13 KnownDirectoryLink
#[derive(Clone, Debug, Default)]
pub struct KnownDirectoryLink {
    /// Matches the casing returned by `realpath`. Used to compute the `realpath` of children.
    /// Always has trailing directory separator
    pub real: String,
    /// toPath(real). Stored to avoid repeated recomputation.
    /// Always has trailing directory separator
    pub real_path: Path,
}

// Go: symlinks/knownsymlinks.go:22 KnownSymlinks
// PORT: Go uses SyncMap and SyncSet, whose iteration order is random. The
// cache is filled once and then only read, so plain ordered maps are used.

// PORT: `Path` needs `Hash` and `Eq` as a map key. The frontend `Path` type
// derives them.

