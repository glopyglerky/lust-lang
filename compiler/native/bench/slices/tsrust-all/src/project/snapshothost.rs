//! Go `internal/project/snapshothost.go` (ts#64163).
//!
//! PORT: one thread (project/dirty/interfaces.rs). Go `*SnapshotHost` is
//! `Rc<SnapshotHost>`: every snapshot keeps its host so `Deref` can release
//! the host's caches. The Go `atomic.Uint64` snapshot id is a `Cell`. Go
//! `Session` embeds `*SnapshotHost`; the Rust `Session` holds it in
//! `snapshot_host` and derefs to it (session.rs).

use crate::project::prelude::*;

use crate::contentmapper;
use crate::frontend::parser;
use std::cell::Cell;

// Go: project/snapshothost.go:20 SnapshotHost
// SnapshotHost owns the services shared by a collection of immutable snapshots.

// Go: project/snapshothost.go:34 SourceFileLease (ts#64434)
// PORT: Go `releaseOnce sync.Once` is a `Cell<bool>` (one thread).



/// Go `logging.Logger` as the session logger argument of `Snapshot.Clone`,
/// `cloneForProgram` and `CloneSnapshotWithAutoImports`.
// PORT: Go passes the session's logger (a non-nil interface, also for the nop
// logger) or nil. `None` is the Go nil interface; `Some(logger)` is the
// session logger, which is itself `None` for the nop logger (see
// `logging::new_nop_logger`).
pub type SessionLogger<'a> = Option<&'a Option<Rc<dyn logging::Logger>>>;


// Go: project/snapshothost.go:65 NewSnapshotHost

