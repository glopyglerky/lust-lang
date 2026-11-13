//! Go `internal/project/snapshot.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). Go `*Snapshot` is
//! `Rc<Snapshot>`; the Go `atomic.Int32` ref count is a manual `Cell<i32>`
//! because `dispose` releases parse cache and extended config cache
//! entries. Go `*Project` is `Rc<RefCell<Project>>`. Go
//! `collections.Set` is `FxHashSet`. Log text uses `{:?}` for Go `%v`
//! (log only).

use crate::project::prelude::*;

use crate::contentmapper;
use crate::frontend::core_ext::ProjectReference;
use std::cell::{Cell, OnceCell};
use std::panic::AssertUnwindSafe;
use std::time::Instant;

// Go: project/snapshot.go:31 Snapshot


// Go: project/snapshot.go:218 (*Snapshot).LSPLineMap, as a function of the
// snapshot's file system.
// PORT: Go gives NewConverters the method value `s.LSPLineMap`, which keeps
// the snapshot alive. `s.fs` never changes after NewSnapshot, so the port's
// closure holds `s.fs` instead (no `Rc` cycle, same results).

// Go: project/snapshot.go:84 SnapshotHost.newSnapshot (ts#64163)
// PORT: Go `NewSnapshot` became this host method; the snapshot starts with
// refCount 1 and keeps the host.

// PORT: the body of Go `SnapshotHost.newSnapshot`.

// Go: project/snapshot.go:193 overlayFileHandles (ts#64291)
// PORT: Go `map[tspath.Path]FileHandle` keeps the overlay map's order here.

// Go: project/snapshot.go:111 snapshotOverlays (ts#64291)


// Go: project/snapshot.go:13 (import of ls; Snapshot is the ls.Host of a
// language service)

// Go: project/snapshot.go:322 APICreateProgramRequest (ts#64204)
// PORT: Go `*core.CompilerOptions` is `Rc<CompilerOptions>`: every Go
// caller (the API session) sets it. A Go nil `[]*core.ProjectReference` is
// an empty `Vec`.

// Go: project/snapshot.go:331 ModuleResolverFactory (ts#64299)

// Go: project/snapshot.go:335 APIReconfigureProgramRequest (ts#64204)
// PORT: Go embeds `APICreateProgramRequest`; here it is the field
// `api_create_program_request`.

// Go: project/snapshot.go:340 APISnapshotRequest
// PORT: Go `*collections.Set[T]` is `Option<FxHashSet<T>>` (nil is `None`).
// Go `map[tspath.Path]string` (`open_files`, `ensure_files`, ts#64391) is
// `Option<IndexMap>`, so API-opened files enter the API state in request
// order (Go map order is random).
// PORT: Go nil `vfs.FS` is `None`. `Debug` skips the file system.


// Go: project/snapshot.go:357 ProjectTreeRequest
#[derive(Clone, Debug, Default)]
pub struct ProjectTreeRequest {
    // If null, all project trees need to be loaded, otherwise only those that are referenced
    pub referenced_projects: Option<FxHashSet<tspath::Path>>,
}

impl ProjectTreeRequest {
    // Go: project/snapshot.go:362 IsAllProjects
    pub fn is_all_projects(&self) -> bool {
        self.referenced_projects.is_none()
    }

    // Go: project/snapshot.go:366 IsProjectReferenced
    // PORT: Go `Set.Has` returns false on a nil set.
    pub fn is_project_referenced(&self, project_id: &tspath::Path) -> bool {
        self.referenced_projects
            .as_ref()
            .is_some_and(|referenced_projects| referenced_projects.contains(project_id))
    }

    // Go: project/snapshot.go:370 Projects
    // PORT: a Go nil slice is empty. Go map order is random; FxHashSet order here.
    pub fn projects(&self) -> Vec<tspath::Path> {
        let Some(referenced_projects) = &self.referenced_projects else {
            return Vec::new();
        };
        referenced_projects.iter().cloned().collect()
    }
}

// Go: project/snapshot.go:377 ResourceRequest
// PORT: Go `*ProjectTreeRequest` is `Option<ProjectTreeRequest>` (nil is `None`).
#[derive(Clone, Debug, Default)]
pub struct ResourceRequest {
    // Documents are URIs that were requested by the client.
    // The new snapshot should ensure projects for these URIs have loaded programs.
    pub documents: Vec<lsproto::DocumentUri>,
    // ConfiguredProjectDocuments are URIs for which configured projects should be loaded
    // (if disableSolutionSearching/disableReferencedProjectLoad settings allow),
    // but no inferred project should be created if no configured project is found.
    // This is used by cross-project operations like find-all-references.
    pub configured_project_documents: Vec<lsproto::DocumentUri>,
    // Update requested Projects.
    // this is used when we want to get LS and from all the Projects the file can be part of
    // ts#64319: project IDs.
    pub projects: Vec<ID>,
    // Update and ensure project trees that reference the projects
    // This is used to compute the solution and project tree so that
    // we can find references across all the projects in the solution irrespective of which project is open
    pub project_tree: Option<ProjectTreeRequest>,
    // AutoImports is the document URI for which auto imports should be prepared.
    pub auto_imports: lsproto::DocumentUri,
}

// Go: project/snapshot.go:397 SnapshotChange
// PORT: Go embeds `ResourceRequest`; here it is the field
// `resource_request`. Go `*core.CompilerOptions` is
// `Option<Rc<CompilerOptions>>`, Go `*lsutil.UserPreferences` is
// `Option<lsutil::UserPreferences>`, Go `*APISnapshotRequest` is
// `Option<APISnapshotRequest>` (nil is `None`).

// Go: project/snapshot.go:421 ATAStateChange
// ATAStateChange represents a change to a project's ATA state.
// PORT: Go `*ata.TypingsInfo` is `Option<Rc<ata::TypingsInfo>>`.



