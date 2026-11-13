//! Go `internal/project/projectcollectionbuilder.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). The builder is shared
//! (`Rc<ProjectCollectionBuilder>`: projects and compiler hosts keep a
//! pointer to it until `freeze`), so its methods take `self: &Rc<Self>` and
//! the fields Go writes after construction are `Cell` / `RefCell`. Go
//! `dirty.Value[*Project]` is `&dyn dirty::Value<Rc<RefCell<Project>>>`
//! (a returned one is `Rc<dyn ..>`). Go `collections.Set` and
//! `map[K]struct{}` are `FxHashSet`; Go `collections.SyncSet` is a
//! `RefCell<FxHashSet>`. `core.BreadthFirstSearchParallelEx` and the
//! parallel `core.WorkGroup` run serially in queue order. Log text uses
//! `{:?}` for Go `%v` (log only).

use crate::project::prelude::*;

use crate::contentmapper;
use crate::frontend::core_ext::{ProjectReference, };
use crate::frontend::{core_bfs, core_ls_ext, core_workgroup};
use std::cell::Cell;
use std::collections::VecDeque;
use std::time::Instant;

// Go: project/projectcollectionbuilder.go:26 projectLoadKind
// PORT: Go `type projectLoadKind int` with iota consts; Go
// `projectLoadKindFind` is `ProjectLoadKind::FIND` (same values).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectLoadKind(pub i32);

impl ProjectLoadKind {
    // Project is not created or updated, only looked up in cache
    pub const FIND: ProjectLoadKind = ProjectLoadKind(0);
    // Project is created and then its graph is updated
    pub const CREATE: ProjectLoadKind = ProjectLoadKind(1);
}

// Go: project/projectcollectionbuilder.go:35 ProjectCollectionBuilder
// PORT: tsgo#4712 adds the content-mapped parse cache, the content mapper
// host (Go nil interface is `None`) and the inferred project mappers.


// Go: project/projectcollectionbuilder.go:68 newProjectCollectionBuilder
// PORT: Go `oldAPIState.clone()` copies the state, so the caller passes it by
// reference.

// PORT: Go `ensureCloned` (closure in Finalize). `None` is Go
// `changed == false` (the base collection is still the result).


// Go: project/projectcollectionbuilder.go:1419 projectReferencesEqual (ts#63950)
// PORT: Go `[]*core.ProjectReference` elements are never nil here.
pub fn project_references_equal(a: &[ProjectReference], b: &[ProjectReference]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(a, b)| a.path == b.path && a.circular == b.circular)
}

/// Go `slices.Equal` on `[]*contentmapper.Mapper` (pointer compare).
fn mappers_equal(a: &[Rc<contentmapper::Mapper>], b: &[Rc<contentmapper::Mapper>]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| Rc::ptr_eq(a, b))
}


// Go: project/projectcollectionbuilder.go:271 isReferencedBy (closure in DidChangeFiles)
// PORT: Go defines this closure and never calls it (it only calls itself);
// it is kept for the literal port. Go `*collections.Set[*Project]` is a set
// of project addresses.

// Go: project/projectcollectionbuilder.go:581 logChangeFileResult
// PORT: Go passes the result by value; here by reference.
pub fn log_change_file_result(result: &ChangeFileResult, logger: &Option<Rc<logging::LogTree>>) {
    if !result.affected_projects.is_empty() {
        logger.logf(&format!(
            "Config file change affected projects: {:?}",
            result.affected_projects.iter().collect::<Vec<_>>()
        ));
    }
    if !result.affected_files.is_empty() {
        logger.logf(&format!(
            "Config file change affected config file lookups for {} files",
            result.affected_files.len()
        ));
    }
}

// Go: project/projectcollectionbuilder.go:1042 searchNode
#[derive(Clone)]
pub struct SearchNode {
    pub config_file_name: String,
    pub load_kind: ProjectLoadKind,
    pub logger: Option<Rc<logging::LogTree>>,
}

// Go: project/projectcollectionbuilder.go:1048 searchNodeKey
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SearchNodeKey {
    pub config_file_name: String,
    pub load_kind: ProjectLoadKind,
}

// Go: project/projectcollectionbuilder.go:1053 searchResult
