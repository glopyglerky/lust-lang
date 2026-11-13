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
use crate::frontend::module::{AheadCall, KeyList};
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

/// Checks the file system calls of a resolve-ahead answer on this host's
/// file system, as the loader's own resolution of the key would make them
/// at this point of the load, and replays their side effects
/// (`compiler::CompilerHost::resolve_ahead`). False: a call would give
/// another answer here, and the loader resolves the key itself.
///
/// - `file_exists`: the snapshot's cached file of the path decides, if it
///   has one (`SnapshotFSBuilder::cached_file_state`). A cached file that
///   needs a reload fails the check: the loader's lookup would read it.
///   Else the snapshot's lookup cache decides, if it has an answer (Go
///   `cachedvfs` of the layered file system, which the worker read the same
///   way: open files over the OS), and else the lookups of an earlier
///   resolve-ahead job of the snapshot, if one has an answer
///   (`AheadLookupLayer`). With neither, the answer passes: the cache takes
///   this load's job answer, which is the worker's, when it is asked
///   (`ResolveAheadHost::attach`). A `known` answer (not found in this load)
///   is asked of the layered file system through the cache.
/// - `directory_exists` and `realpath`: the same, and a call that the job's
///   answers share (`AheadCall::Shared`) is checked once per load.
/// - a read is the loader's own read (`SourceFS::get_file`: it tracks the
///   file, caches it and notes a `node_modules` realpath alias), made at
///   the moment the loader would make it, since every call before it gave
///   the same answer. Its text must have the worker's hash. `reads` keeps
///   the reads of this load, so a later answer that reads the file again
///   only compares the hash. A worker read that failed fails the check.
/// - then each `file_exists` path becomes a seen file and each missing
///   directory a missing directory, as the loader's calls would note them.
/// - the calls of a package.json cache entry (`AheadCall::PackageJson`)
///   are checked and replayed once per load: the snapshot does not change
///   its answers during the load, and the replay notes the same paths
///   again.
/// What `accept_ahead_answer` keeps during one load.
struct AheadCheck {
    /// The snapshot's lookup cache (`SnapshotFSBuilder::cached_fs`).
    lookups: Rc<vfs::CachedFs>,
    /// The index of this load's job in the layer under `lookups`
    /// (`AheadLookupLayer`; `usize::MAX` until the load gives the workers a
    /// job).
    job: Rc<Cell<usize>>,
    /// The hash of each file that the check read, by name.
    reads: RefCell<FxHashMap<String, Option<u128>>>,
    /// The shared calls (`AheadCall::PackageJson`, `AheadCall::Shared`)
    /// that passed the check and were replayed, by address (the answers
    /// keep them for the whole load).
    groups: RefCell<FxHashSet<usize>>,
}

impl AheadCheck {
    /// True when `answer`, a worker's answer for a lookup, is the answer of
    /// the snapshot: `cached`, the answer of its lookup cache, or else the
    /// answer of an earlier job (`before`), if it has one.
    fn agrees<T: PartialEq>(
        &self,
        cached: Option<T>,
        before: impl Fn(&dyn compiler::resolve_ahead::AheadLookups) -> Option<T>,
        answer: &T,
    ) -> bool {
        cached
            .or_else(|| ahead_lookup_layer(&self.lookups)?.before(self.job.get(), before))
            .is_none_or(|snapshot| snapshot == *answer)
    }
}





fn shared_key(call: &Arc<AheadCall>) -> usize {
    Arc::as_ptr(call) as usize
}
