//! Port of Go `internal/api/requestfilesystem/pathtree.go` (ts#64115,
//! ts#64291).
//!
//! PORT: Go `*requestPathNode` is shared between trees (`composeRequestPaths`
//! copies a node and keeps its children), so a node is
//! `Rc<RefCell<RequestPathNode>>`. Nodes are only written while
//! `newRequestFileSystemWorker` builds a tree; later trees share them.
//! Go map order is random; the code that returns results sorts them, as Go
//! does.

use crate::prelude::*;

use crate::frontend::tspath;
use crate::frontend::vfs;

// Go: api/requestfilesystem/pathtree.go requestFallback
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RequestFallback {
    #[default]
    Inherit,
    Allowed,
    Missing,
}

// Go: api/requestfilesystem/pathtree.go requestEntry
// PORT: the Go interface is an enum of its three pointer types. `None` is
// the nil interface.
#[derive(Clone, Debug)]
pub enum RequestEntry {
    File(Rc<RequestFile>),
    Symlink(Rc<RequestSymlinkEntry>),
    Directory(Rc<RequestDirectory>),
}

// Go: api/requestfilesystem/pathtree.go requestFile
#[derive(Clone, Debug, Default)]
pub struct RequestFile {
    pub file_name: String,
    pub content: String,
}

// Go: api/requestfilesystem/pathtree.go requestSymlink
// PORT: named `RequestSymlinkEntry`; `RequestSymlink` is the exported
// request type (requestfilesystem.go).
#[derive(Clone, Debug, Default)]
pub struct RequestSymlinkEntry {
    pub link_name: String,
    pub target: String,
    pub host: bool,
}

// Go: api/requestfilesystem/pathtree.go requestDirectory
#[derive(Clone, Debug, Default)]
pub struct RequestDirectory {
    pub directory_name: String,
    pub listing: Option<vfs::Entries>,
}

// Go: api/requestfilesystem/pathtree.go requestFile (vfs.FileInfo methods)
// PORT: Go `*requestFile` is a `vfs.FileInfo`; the port makes the value.
impl RequestFile {
}

// Go: api/requestfilesystem/pathtree.go requestDirectory (vfs.FileInfo methods)
impl RequestDirectory {
}

/// Go `node.entry.(vfs.FileInfo)`: a file or a directory entry is a
/// `vfs.FileInfo`; a symlink entry is not.
// PORT: the entry stays an entry, so a caller can still take the file
// content (Go `info.(*requestFile)`).
#[derive(Clone, Debug)]
pub enum RequestInfo {
    File(Rc<RequestFile>),
    Directory(Rc<RequestDirectory>),
}

impl RequestInfo {
    pub fn is_dir(&self) -> bool {
        matches!(self, RequestInfo::Directory(_))
    }

}

pub type RequestPathNodeRef = Rc<RefCell<RequestPathNode>>;

// Go: api/requestfilesystem/pathtree.go requestPathNode
#[derive(Clone, Debug, Default)]
pub struct RequestPathNode {
    pub entry: Option<RequestEntry>,
    pub fallback: RequestFallback,
    // PORT: Go nil map is `None`.
    pub children: Option<FxHashMap<tspath::Path, RequestPathNodeRef>>,
    pub has_symlinks: bool,
}

impl RequestPathNode {
    // Go: api/requestfilesystem/pathtree.go requestPathNode.replacesSubtree
    pub fn replaces_subtree(&self) -> bool {
        matches!(
            self.entry,
            Some(RequestEntry::File(_)) | Some(RequestEntry::Symlink(_))
        )
    }

    fn child(&self, path: &tspath::Path) -> Option<RequestPathNodeRef> {
        self.children.as_ref().and_then(|c| c.get(path)).cloned()
    }
}

// Go: api/requestfilesystem/pathtree.go requestPathAncestors

// Go: api/requestfilesystem/pathtree.go requestPathNode.ensure

// Go: api/requestfilesystem/pathtree.go requestPathNode.lookup
// PORT: Go nil `node` is `None`.

// Go: api/requestfilesystem/pathtree.go requestPathNode.walkSymlinks
pub fn walk_symlinks(
    node: Option<&RequestPathNodeRef>,
    visit: &mut dyn FnMut(&tspath::Path, &Rc<RequestSymlinkEntry>),
) {
    let Some(node) = node else {
        return;
    };
    let node = node.borrow();
    if !node.has_symlinks {
        return;
    }
    let Some(children) = &node.children else {
        return;
    };
    for (path, child) in children {
        if let Some(RequestEntry::Symlink(symlink)) = &child.borrow().entry {
            visit(path, symlink);
        }
        walk_symlinks(Some(child), visit);
    }
}

// Go: api/requestfilesystem/pathtree.go requestPathNode.entries

// Go: api/requestfilesystem/pathtree.go composeRequestPaths

// Go: api/requestfilesystem/pathtree.go requestPathNode.firstSymlink

// Go: api/requestfilesystem/pathtree.go requestPathNode.containsFileAncestor

// Go: api/requestfilesystem/pathtree.go requestPathContains
pub fn request_path_contains(parent: &tspath::Path, path: &tspath::Path) -> bool {
    *path == *parent
        || path
            .as_str()
            .starts_with(&tspath::ensure_trailing_directory_separator(
                parent.as_str(),
            ))
}
