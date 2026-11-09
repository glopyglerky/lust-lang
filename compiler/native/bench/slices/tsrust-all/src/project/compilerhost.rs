//! Go `internal/project/compilerhost.go`.
//!
//! PORT: Go `compilerHost` is the struct `CompilerHost`; the Go interface
//! `compiler.CompilerHost` is always written `compiler::CompilerHost`
//! (map-project.md section 4). The host is shared (`Rc<CompilerHost>` in
//! the project and `Rc<dyn compiler::CompilerHost>` in the program), and
//! `freeze` writes it after sharing, so the fields that `freeze` clears are
//! `RefCell`s. `freeze` drops the `builder` and `project` references, which
//! breaks the `Project -> host -> builder -> Project` cycle.

use crate::project::prelude::*;

use crate::contentmapper;
use crate::frontend::core_ext::{
    
};
use crate::frontend::module::{KeyList};
use crate::frontend::parser;
use std::cell::Cell;
use std::sync::Arc;
use crate::xxhash_rust::xxh3::xxh3_128;

// Go: project/compilerhost.go:21 compilerHost

/// The resolve-ahead keys and kept share of the projects that a snapshot
/// clone made and deleted, by config file path. The first host of a later
/// project of the same config takes them (`new_compiler_host`), so its load
/// resolves ahead the keys of the deleted project's last load. The project
/// search of each hono file open makes and deletes tsconfig.spec.json; with
/// no keys, its load resolves its 1338 module keys on the loading thread.
/// The keys are hints only: the loader checks each answer
/// (`accept_ahead_answer`), and resolves itself a key that it does not
/// find. One per session (`SnapshotHost`), with at most `CAPACITY`
/// projects; a new one drops the least recently deleted (a project
/// deleted again moves to the end, and `take` removes its entry).
///
/// Rule: only `Snapshot::clone` puts, for a program that it made for a
/// project that its new collection does not have. A host that `release`
/// frees (the old host of a live project, or the last host of a project
/// whose files closed) drops its keys and share as before, so the kept
/// state of a closed project goes as `KeptShare` says.
// PORT: not in Go (perf; Go has no resolve ahead).
#[derive(Default)]
pub struct ResolveAheadStash {
    projects: RefCell<Vec<StashedProject>>,
}

struct StashedProject {
    config_file_path: tspath::Path,
    keys: Arc<KeyList>,
    share: Rc<compiler::resolve_ahead::KeptShare>,
}

impl ResolveAheadStash {
    const CAPACITY: usize = 8;


    /// Takes the keys and the share of the deleted project of
    /// `config_file_path`.
    fn take(
        &self,
        config_file_path: &tspath::Path,
    ) -> Option<(Arc<KeyList>, Rc<compiler::resolve_ahead::KeptShare>)> {
        let mut projects = self.projects.borrow_mut();
        let index = projects
            .iter()
            .position(|project| project.config_file_path == *config_file_path)?;
        let project = projects.remove(index);
        Some((project.keys, project.share))
    }
}

// Go: project/compilerhost.go:36 newCompilerHost
// PORT: reads `project.configFilePath`, so the caller must not hold a
// mutable borrow of `project` during this call. The host keeps its own
// `Rc`s of `project` and `builder` until `freeze`.

thread_local! {
    /// The hosts of this thread that are neither frozen nor dropped
    /// (`unfrozen_compiler_hosts`).
    static UNFROZEN_HOSTS: Cell<usize> = const { Cell::new(0) };
}

/// Not in Go: the number of hosts that `new_compiler_host` made on this
/// thread and that are neither frozen nor dropped. A host keeps its project
/// and the builder until `freeze`. Go's GC frees them with the host; the
/// port keeps the host of a released program (`release`), so every host
/// that a snapshot clone made must be frozen when the clone ends. Tests
/// check it.
#[must_use]
pub fn unfrozen_compiler_hosts() -> usize {
    UNFROZEN_HOSTS.with(Cell::get)
}



// Go: project/compilerhost.go:14 `var _ compiler.CompilerHost = (*compilerHost)(nil)`







