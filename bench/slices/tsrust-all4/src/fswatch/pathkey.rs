//! Go: internal/fswatch/pathkey.go (ts#64210).

use crate::fswatch::prelude::*;

use crate::fswatch::pathcompare::PathComparer;

// Go: pathkey.go:10 NativePathComparisonAvailable
/// NativePathComparisonAvailable reports whether this platform implements the
/// native watch-name comparison. Volume sensitivity is queried separately.
pub const NATIVE_PATH_COMPARISON_AVAILABLE: bool = NATIVE_PATH_FOLDING;

// Go: pathkey.go:14 PathComparer
/// PathComparer describes the filename equivalence of a watched volume. Its zero
/// value compares bytes exactly. It is immutable and safe to share.
// PORT: named `PathComparerExported` because Go also has the unexported
// `pathComparer` (pathcompare.go), which is `PathComparer` (PORTING "Names").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PathComparerExported {
    pub comparer: PathComparer,
}

impl PathComparerExported {
    // Go: pathkey.go:21 PathComparer.Key
    /// Key returns a watch-only comparison key, not a filesystem path or a compiler
    /// identity. Native Darwin comparers use the same CoreFoundation folding as the
    /// watcher. Other comparers preserve bytes, including malformed UTF-8.
    pub fn key(&self, path: &str) -> String {
        if !self.comparer.ignore_case || !NATIVE_PATH_FOLDING {
            return path.to_string();
        }
        if path.contains('\0') {
            return path.to_string();
        }
        let folded = fold_native_path(path);
        if !folded.is_empty() {
            return folded;
        }
        // Invalid UTF-8 and NUL-containing names are opaque, not Unicode aliases.
        path.to_string()
    }

    // Go: pathkey.go:37 PathComparer.Rebase
}

