//! Port of Go `internal/api/requestfilesystem/filechanges.go` (ts#64115,
//! ts#64291).

use crate::prelude::*;

use super::requestfilesystem::{RequestFileSystem, RequestFileSystemImpl, get_request_file_system};
use crate::frontend::tspath;
use crate::frontend::vfs;
use crate::ls::lsconv;
use crate::lsp::lsproto;
use crate::project::{self, FileHandle as _};

// Go: api/requestfilesystem/filechanges.go requestFileSystem.ExpandFileChanges
// (project.FileChangeExpander)

// Go: api/requestfilesystem/filechanges.go addFileChanges
// PORT: Go map order is random; the names are walked in sorted order.
