//! Port of Go `internal/api/requestfilesystem/requestfilesystem.go`
//! (ts#64115, ts#64277, ts#64291, ts#64391).
//!
//! PORT: Go has the exported request type `RequestFileSystem` and the
//! unexported file system `requestFileSystem`. The file system is
//! `RequestFileSystemImpl` here. Its methods have Go value receivers, so it
//! is `Clone` (the fields are shared handles).
//!
//! PORT: Go type assertions on a `vfs.FS` (`fs.(*requestFileSystem)`,
//! `fs.(project.LayeredFileSystem)`, `fs.(project.FileHandleSource)`) use
//! `vfs::Fs::as_any` (a downcast to the concrete type) and the
//! `project::FsLayer` queries (the server lane design).

use crate::prelude::*;

use super::pathtree::{
    RequestDirectory, RequestEntry, RequestFallback, RequestFile, RequestInfo, RequestPathNode,
    RequestPathNodeRef, RequestSymlinkEntry, 
    request_path_contains, walk_symlinks,
};
use crate::frontend::json::{JsonDecoder, JsonError, UnmarshalerFrom, };
use crate::frontend::json_ext::{self, };
use crate::frontend::tspath;
use crate::frontend::vfs::{self, };
use crate::gostd::{GoError, errors, strconv};
use crate::project::{self, };
use std::borrow::Cow;
use std::time::SystemTime;

// Go: api/requestfilesystem/requestfilesystem.go Kind
// Kind controls how a request filesystem is used.




// Go: api/requestfilesystem/requestfilesystem.go RequestDirectoryEntries
// RequestDirectoryEntries is a cached directory listing. Entry names are
// relative to the directory, matching vfs.GetAccessibleEntries.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RequestDirectoryEntries {
    pub files: Vec<String>,
    pub directories: Vec<String>,
}

// Go: api/requestfilesystem/requestfilesystem.go RequestSymlink
// RequestSymlink describes a symbolic link in a request filesystem.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RequestSymlink {
    // Target is resolved relative to the directory containing the link, matching
    // native symbolic-link semantics.
    pub target: String,
    // Host routes the target through the host filesystem. This is the only way a
    // full filesystem can access paths not supplied in the request filesystem.
    pub host: bool,
}

// Go: api/requestfilesystem/requestfilesystem.go RequestFileSystem
// RequestFileSystem supplies file contents and, optionally, directory listings
// for a request that creates a snapshot.
// PORT: Go maps are `FxHashMap`; the code that walks them walks the names in
// sorted order (Go order is random).

// PORT: Go v2 default struct unmarshal of the tagged request structs. The api
// only decodes them, so no marshalers are written.



/// Go map keys in sorted order (see `RequestFileSystem`).
fn sorted_keys<V>(map: &FxHashMap<String, V>) -> Vec<&String> {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    keys
}

// Go: api/requestfilesystem/requestfilesystem.go requestFileSystem
// requestFileSystem is either a full filesystem or a layer over the session
// host filesystem. Its base is always the host, which may be a callback filesystem;
// inherited request entries are compacted into paths.

// Go: api/requestfilesystem/requestfilesystem.go resolvedRequestPath
#[derive(Clone, Debug, Default)]
struct ResolvedRequestPath {
    path: String,
    followed_symlink: bool,
    host: bool,
    ok: bool,
}

// Go: api/requestfilesystem/requestfilesystem.go requestPathLookup

// Go: api/requestfilesystem/requestfilesystem.go getRequestFileSystem
// PORT: Go returns the `*requestFileSystem` pointer; the port returns a copy
// (the fields are shared handles).

// Go: api/requestfilesystem/requestfilesystem.go NewForUpdate
// NewForUpdate creates a request filesystem for a snapshot update. Layers over
// request filesystems are compacted eagerly so the result does not retain its
// base snapshot's filesystem.
// PORT: Go nil `params` returns `base`; that is `None`.

// Go: api/requestfilesystem/requestfilesystem.go HasFullFileSystem
// HasFullFileSystem reports whether fileSystem contains a complete request filesystem.
// PORT: Go nil `fileSystem` is `None`.

// Go: api/requestfilesystem/requestfilesystem.go newRequestFileSystemWorker


// Go: api/requestfilesystem/requestfilesystem.go cloneEntries
pub fn clone_entries(entries: &vfs::Entries) -> vfs::Entries {
    vfs::Entries {
        files: entries.files.clone(),
        directories: entries.directories.clone(),
        symlinks: entries.symlinks.clone(),
    }
}

// Go: api/requestfilesystem/requestfilesystem.go mergeEntries

// Go: vfs.FS methods of requestFileSystem.

// PORT: the Go interfaces and the concrete type that `*requestFileSystem`
// satisfies (`project::FsLayer`, server lane).

// Go: api/requestfilesystem/requestfilesystem.go requestFileSystem.GetFile,
// GetFileByPath (project.FileHandleSource)

// Go: api/requestfilesystem/requestfilesystem.go requestFileSystem.Overlays
// (project.LayeredFileSystem)

// Go: api/requestfilesystem/requestfilesystem.go requestFileSystem.BaseFileSystem,
// WithBaseFileSystem (project.RebasableFileSystem)

// Go: api/requestfilesystem/requestfilesystem.go statFileSystem
