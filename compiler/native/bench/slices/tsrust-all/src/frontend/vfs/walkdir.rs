//! Go: internal/vfs/walkdir.go (ts#64277).

use crate::frontend::prelude::*;

// Go: vfs/walkdir.go:18 WalkDir
// WalkDir calls walkFn for root and each accessible descendant in lexical order.
// Symbolic links are reported but not followed. Directory read failures are
// indistinguishable from empty directories because [FS.GetAccessibleEntries]
// does not return errors. The FileInfo returned by DirEntry.Info for a symbolic
// link contains only its name and mode because [FS] does not provide lstat.
// Using DirEntry.Info() requires the FS to implement Stat().
// PORT: Go `walkDirEntry.Info` calls `Stat` when the caller asks. A Rust
// `DirEntry` cannot hold the file system, so the walk calls `Stat` for each
// entry that is not a symbolic link and keeps the result
// (`new_walk_dir_entry`).

// PORT: the Go closures of `WalkDir` (`sameRoot`, `equivalent`,
// `canonicalize`, `visit`) and the variables they share.


// Go: vfs/walkdir.go:143 normalizeWalkDirError
fn normalize_walk_dir_error(err: Result<(), FsError>) -> Result<(), FsError> {
    match err {
        Err(err) if err.is_skip_dir() || err.is_skip_all() => Ok(()),
        err => err,
    }
}

// Go: vfs/walkdir.go:150 walkDirEntry
// PORT: a `DirEntry` value. `Info` (Go walkdir.go:165): a symbolic link has
// the Go `walkDirFileInfo` (its name and mode, size 0, the zero time);
// another entry has the `Stat` of its path, or `ErrNotExist`
// (`DirEntryInfo::NotExist`). See `walk_dir` for when `Stat` runs.
