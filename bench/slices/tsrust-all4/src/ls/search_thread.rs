//! Search threads of cross-project requests (`crossproject.rs`).
//!
//! PORT: Go runs the search of each project of a references,
//! implementations or rename request on its own goroutine, over the shared
//! program, snapshot and checker pools. Those Rust values use `Rc` and live
//! on the dispatch thread. Here each program that such a search reaches gets
//! one long-lived search thread, as the compile path gives each checker its
//! own thread (`program::create_checkers`):
//! - The thread starts from a copy of the dispatch thread's thread-local
//!   state (`program::WorkerSeed`), made after the program is bound. It
//!   makes its own checker when a job first needs one, and keeps it between
//!   jobs. It drops the checker after `CHECKER_IDLE_TIMEOUT` with no job,
//!   and ends when the program is released (`release_search_thread`).
//! - A job gets plain data (`SearchJob`) and returns plain data
//!   (`ItemOutcome`). The thread makes its own language service over a copy
//!   of the program data (`SearchProgramData`, `SearchView`) and a host
//!   (`WorkerHost`) that reads program files from the AST store. The text in
//!   the store is the text the program was parsed from, which is the
//!   snapshot's text of that file.
//! - The reads that the copy does not hold (files outside the program,
//!   directories, triple-slash resolutions) go to the dispatch thread
//!   (`HostQuery`). It answers them from the item's language service while it
//!   waits for the results (`answer_query`).
//! - No `Rc` value and no checker crosses threads.

use crate::ls::prelude::*;

use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

/// A search thread drops its checker after this long with no job.
// Go: project/checkerpool.go newCheckerPool (the default idle timeout of a
// checker).
const CHECKER_IDLE_TIMEOUT: Duration = Duration::from_secs(30);

/// The index of the next search thread checker. Search checkers get their
/// own id range, so that no id of a dispatch thread checker repeats.
static NEXT_SEARCH_CHECKER_INDEX: AtomicUsize = AtomicUsize::new(1 << 20);


// ---------------------------------------------------------------------------
// Jobs and messages
// ---------------------------------------------------------------------------

/// The item data of phase 2 that `handle_cross_project` has.
pub struct SearchItem {
    pub ctx: Context,
    pub uri: lsproto::DocumentUri,
    pub position: lsproto::Position,
    pub is_rename: bool,
    pub implementations: bool,
    pub options: SymbolEntryTransformOptions,
}

/// What phase 2 of an item needs on a search thread (plain data).
struct SearchJob<Req> {
    item: SearchItem,
    params: Req,
    project_id: crate::ls::autoimport::ProjectID,
    preferences: lsutil::UserPreferences,
    use_case_sensitive_file_names: bool,
    position_encoding: lsproto::PositionEncodingKind,
}

/// The result of phases 1 and 2 of an item, for its commit.
pub struct ItemOutcome<Resp> {
    /// The original definition locations, in search order.
    pub locations: Vec<(lsproto::DocumentUri, lsproto::Position)>,
    /// The response or error; None where Go returns with neither (no
    /// language service, a canceled request, or a panic).
    pub result: Option<Result<Resp, GoError>>,
    /// Go `panicOccured` of the item.
    pub panic: Option<String>,
}

impl<Resp> ItemOutcome<Resp> {
    /// An item that returned before its search.
    pub fn skipped() -> Self {
        ItemOutcome {
            locations: Vec::new(),
            result: None,
            panic: None,
        }
    }

    /// An item that panicked before its search.
    pub fn panicked(panic_occured: String) -> Self {
        ItemOutcome {
            locations: Vec::new(),
            result: None,
            panic: Some(panic_occured),
        }
    }
}



/// A read that a search thread asks the dispatch thread for.
pub enum HostQuery {
    ReadFile(String),
    FileExists(String),
    DirectoryExists(String),
    GetDirectories(String),
    ReadDirectory {
        current_dir: String,
        path: String,
        extensions: Vec<String>,
        excludes: Vec<String>,
        includes: Vec<String>,
        depth: i32,
    },
    IsSourceFromProjectReference(tspath::Path),
    ReferencesToFile {
        referencing_file: Node,
        target: Node,
    },
    ReferenceAtPosition {
        source_file: Node,
        position: i32,
    },
}

/// The dispatch thread's answer to a `HostQuery`.
pub enum HostAnswer {
    /// The file text is shared, not copied (`FileText`).
    File(FileText, bool),
    Bool(bool),
    Strings(Vec<String>),
    FileReferences(Vec<FileReference>),
    RefInfo(Option<RefInfo>),
    /// The read panicked on the dispatch thread; the search panics again
    /// with the same payload, so a Go panic (`core::GoPanic`) stays one.
    Panic(Box<dyn std::any::Any + Send>),
    /// The request no longer waits (it panicked); the search result is
    /// dropped.
    Gone,
}

/// A message from a search thread to the dispatch thread.
pub enum ToDispatch<Resp> {
    Query {
        item: usize,
        query: HostQuery,
        reply: mpsc::Sender<HostAnswer>,
    },
    Done {
        item: usize,
        outcome: ItemOutcome<Resp>,
    },
}

/// Sends a `HostQuery` of the running job and waits for the answer.
type QueryFn = Box<dyn Fn(HostQuery) -> HostAnswer + Send>;


/// Sends the result of a job once. If the job is dropped before it runs (its
/// thread stopped), it sends a panic, so that the dispatch thread never waits
/// for it forever.
struct JobReply<Resp> {
    sender: Option<mpsc::Sender<ToDispatch<Resp>>>,
    item: usize,
}

impl<Resp> JobReply<Resp> {
    fn send(mut self, outcome: ItemOutcome<Resp>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(ToDispatch::Done {
                item: self.item,
                outcome,
            });
        }
    }
}

impl<Resp> Drop for JobReply<Resp> {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(ToDispatch::Done {
                item: self.item,
                outcome: ItemOutcome::panicked(
                    "panic handling request: the search thread stopped".to_string(),
                ),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Dispatch thread side
// ---------------------------------------------------------------------------







// ---------------------------------------------------------------------------
// Search thread side
// ---------------------------------------------------------------------------






/// The dispatch thread link of the running job.
struct JobHost {
    query: QueryFn,
    /// Files outside the program that the job read through the dispatch
    /// thread (the item's snapshot does not change during the job).
    files: RefCell<FxHashMap<String, (FileText, bool)>>,
}





