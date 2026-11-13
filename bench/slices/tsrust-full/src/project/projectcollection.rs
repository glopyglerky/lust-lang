//! Go `internal/project/projectcollection.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). Go `*Project` is
//! `Rc<RefCell<Project>>` (nil is `None`); Go `*ConfigFileRegistry` is
//! `Rc<ConfigFileRegistry>`. Go `collections.Set` is `FxHashSet`. Go
//! `map[K]struct{}` is `FxHashSet<K>`. A Go map that the builder shares
//! with the collection is copied here (the collection never writes it).

use crate::project::prelude::*;

use crate::frontend::{core_bfs, core_ls_ext};
use std::cell::OnceCell;

// Go: project/projectcollection.go:15 ProjectCollection

// Go: project/projectcollection.go:45 APIState
// APIState tracks the projects and files that API clients have explicitly opened.
// Opens and closes are ref-counted so multiple API clients don't clobber each
// other, and it is carried across snapshots so API-opened resources stay loaded.
// PORT: Go maps iterate in random order; `IndexMap` keeps insertion order.
// Go `clone` (projectcollection.go:55) and `equals` (projectcollection.go:62)
// are the derived `Clone` and `PartialEq`; `IndexMap` equality ignores order,
// like `maps.Equal`.

// Go: project/projectcollection.go:67 apiOpenedFile
// apiOpenedFile tracks a file kept open by API clients along with its ref count.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct APIOpenedFile {
    pub file_name: String,
    pub ref_count: i32,
}


// Go: project/projectcollection.go:202 openFilePaths
// PORT: Go `map[tspath.Path]*Overlay` is the `IndexMap` of overlayfs.rs.

// Go: project/projectcollection.go:348 findDefaultConfiguredProjectFromProgramInclusion
// findDefaultConfiguredProjectFromProgramInclusion finds the default configured project for a file
// based on the file's inclusion in existing projects. The projects should be sorted, as ties will
// be broken by slice order. `getProject` should return a project with an up-to-date program.
// Along with the resulting project path, a boolean is returned indicating whether there were multiple
// direct inclusions of the file in different projects, indicating that the caller may want to perform
// additional logic to determine the best project.
// PORT: `get_project` returns `None` for Go nil; Go then dereferences it.
