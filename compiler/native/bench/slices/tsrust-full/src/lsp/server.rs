//! Go `internal/lsp/server.go`.
//!
//! PORT: threads (PORTING.md "Threads", map-lsp-server.md section 3). Go
//! runs the read loop, the dispatch loop, the write loop, the async part of
//! each handler, the progress loop and the parent watchdog on goroutines.
//! Language-service state (`Rc`/`RefCell`) must stay on one thread, so:
//!
//! - The dispatch thread (the thread that calls `Server::run`, with the
//!   stack size of `gostd::stack::max_stack_size`) owns `Server`: the
//!   session, the file system, the handler table and the API sessions. It
//!   runs the sync part of each handler and then its async part inline (Go
//!   starts the async part on a goroutine, so Go can answer requests out of
//!   order; the port answers them in order). Between the two parts, after
//!   each message and after each wake-up it runs
//!   `gostd::local::run_pending()`. When no message waits, it runs
//!   `gostd::local::run_idle()`.
//! - The reader thread owns the `Reader`. It routes responses to
//!   `pending_server_requests`, handles `$/cancelRequest` and the first
//!   `initialize`, and queues all other messages (Go does the same on the
//!   read goroutine). It also cancels the auto-import warm when a file
//!   event arrives, or makes it yield (below).
//! - The writer thread owns the `Writer` and drains the outgoing queue.
//! - The progress thread (`progress.rs`) and the parent watchdog
//!   (`cmd/tsgo/lsp.rs`) touch only `Send` data.
//! - An API session (`custom/initializeAPISession`) has an accept thread
//!   and, once connected, a reader thread. They queue the connection and
//!   wake-ups in the request queue; the dispatch thread serves the
//!   connection (`ApiConnProtocol`).
//!
//! So Go `*Server` is split. `ServerShared` (`Arc`, `Send + Sync`) holds
//! the queues, the pending maps, the atomics and the state that
//! `handleInitialize` writes once on the reader thread (`OnceLock`).
//! `Server` (`Rc`, dispatch thread) holds the rest. Go methods that the
//! reader thread calls (`readLoop`, `sendError`, `handleInitialize`, the
//! send functions) are on `ServerShared`.
//!
//! Effects of the one dispatch thread (lsp-concurrency.md D1, D5, D7). The
//! LS oracle keeps one request in flight, so it sees none of them:
//!
//! - Answers come in arrival order (above). A fast request waits for a slow
//!   one; Go answers the fast one first.
//! - `gostd::local` timers (diagnostics refresh, snapshot update, idle cache
//!   clean) fire at the next `run_pending`: before the async part of the
//!   running request or after its answer. Go fires them on time.
//! - A background client request (`update_watches` registerCapability, 1 s
//!   timeout) blocks the dispatch thread until the client answers. Go waits
//!   on a goroutine.
//! - A request that arrives while the auto-import warm runs waits for it
//!   (below). Go runs the request at the same time.
//! - LSP messages wait while an API request runs, and API requests wait
//!   while an LSP message runs. Go runs them at the same time. An API
//!   connection that opens while another is connected holds the other's
//!   requests until it closes (`ApiConnProtocol`).
//!
//! Background tasks and timers stay on the dispatch thread. A task that the
//! sync part of a request queues (the snapshot update's logging, watch
//! updates and publishDiagnostics) runs before the async part: Go starts it
//! on a goroutine before the async part, and it usually ends before the
//! answer (the publishDiagnostics of a changed tsconfig.json comes before
//! the textDocument/diagnostic answer that picks up the change). Tasks
//! queued later run at the message boundary, after the answer. Do not move
//! them to another thread: the oracle compares where background
//! publishDiagnostics land. Do not hold them back while messages wait
//! either: a task queued by didOpen (the `update_watches`
//! registerCapability) must go out before the answer of the next request,
//! as Go's does.
//!
//! The one exception is idle work (`gostd::local::go_idle`): the clone of
//! the auto-import warm, which sends nothing to the client. It starts only
//! with an empty request queue (`queued_requests`). Go runs it on a
//! goroutine, at the same time, from before the answer of the request that
//! made the snapshot. So the first attempt of a warm starts at once, and a
//! message that comes during it waits only up to Go's head start (a few
//! ms). Then a didOpen, didChange, didClose or didChangeWatchedFiles
//! cancels the warm, as Go's dispatch goroutine does when it handles them,
//! and any other message makes the attempt yield. An attempt after a
//! yield, and each attempt while the session is marked slow (an eager
//! attempt did not end), starts after `IDLE_QUIET_PERIOD`, and only a file
//! event cancels it (`project::WarmAutoImportPreempt`,
//! `project::Session::run_pending_warm`).
//! The warm stops at its next context check.
//!
//! Large frees (`gostd::local::drop_later`: a released program, the parse
//! tasks of a load) wait until the message is done, and then run only
//! while no message waits. Go's garbage collector does them in the
//! background. The checkers and synthetic nodes of a released program wait
//! in the same way only for the first release after a client pause, and
//! only when the client has not sent its next edit yet
//! (`gostd::local::drop_after_pause`), so only the releases of one message
//! wait (on an API connection, of one pipelined burst). Other releases free
//! them at once, so that the next check reuses their memory.
//!
//! Cancellation is Go's: `$/cancelRequest` reaches only a request that the
//! dispatch loop took (`pending_client_requests`); a cancel for a queued
//! request is dropped. The LS loops check the request context. The checker
//! checks it at each top-level statement and deferred node, like Go.
//!
//! When the `run` context ends (Go `signal.NotifyContext` in
//! `cmd/tsc/lsp.go`), every loop returns `context canceled`, as in Go. A
//! request on the dispatch thread first runs to its next cancel check. Its
//! answer is not written, because `send` uses the ended group context.

use crate::lsp::prelude::*;

use crate::contentmapper;
use crate::emitter::program_emit;
use crate::frontend::compiler;
use crate::frontend::json_ext::{self, AnyValue};
use crate::frontend::tsoptions;
use crate::gostd::context::{self, };
use crate::gostd::errors;
use crate::ipc;
use crate::lsp::lsproto::{ErrorCode, HasTextDocumentPosition, HasTextDocumentURI};
use crate::program::ls_program;
use crate::project::logging::{self, Logger as _};
use crate::project::{ata};
use std::any::Any;
use std::cell::Cell;
use std::collections::VecDeque;
use std::io::{BufRead, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Weak;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::sync::{Arc, Condvar, LazyLock, Mutex, MutexGuard, OnceLock, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// PORT: Go mutexes do not poison.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// PORT: Go `sync.RWMutex` read lock (no poison).
fn read_lock<T: Clone>(m: &RwLock<T>) -> T {
    m.read().unwrap_or_else(|e| e.into_inner()).clone()
}

// PORT: Go `sync.RWMutex` write lock (no poison).
fn write_lock<T>(m: &RwLock<T>, value: T) {
    *m.write().unwrap_or_else(|e| e.into_inner()) = value;
}

// Go: server.go:42 ServerOptions
// PORT: Go `In`, `Out` and `Err` move to the reader, writer and logging
// threads, so they are `Send`. `NpmInstall` and `SetParentProcessID` are
// nil-able Go funcs (`None`). `NpmInstall` returns Go's `([]byte, error)`
// pair, as `ata::NpmExecutor` does, and is `Send`: ATA runs it on a helper
// thread (`ata::NpmExecutor::npm_install_func`).

// Go: server.go:60 NewServer

// Go: server.go:90 fileRenameFilters

// Go: server.go:98 `_ ata.NpmExecutor = (*Server)(nil)` and
// `_ project.Client = (*Server)(nil)`: the impls below.

// Go: server.go:102 pendingClientRequest
// PORT: Go keeps the `*lsproto.RequestMessage`, which no code reads. The
// request stays on the dispatch thread (its params are not `Sync`), so the
// entry keeps its method.

// Go: server.go:107 Reader
// PORT: Go `Read() (*lsproto.Message, error)` can return a message and an
// error together (invalid params), so the result is a pair.

// Go: server.go:111 Writer

// Go: server.go:115 lspReader
pub struct LspReader {
    pub r: lsproto::BaseReader,
}

// Go: server.go:119 lspWriter

// Go: server.go:123 messageMarshalError
// PORT: a value type made with `errors::from_value_with_unwrap`, so
// `errors::as_type` finds it. Go `Unwrap() []error` returns
// `{lsproto.ErrorCodeInternalError, e.err}`; the port's value types have
// only `Unwrap() error`, so the one unwrap result is an error that wraps
// both, in the same order (see `new_message_marshal_error`).
#[derive(Clone, Debug, PartialEq)]
pub struct MessageMarshalError {
    pub err: GoError,
}

// Go: server.go:127 messageMarshalError.Error
impl std::fmt::Display for MessageMarshalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to marshal message: {}", self.err.error())
    }
}


// Go: `fmt.Errorf("%w: %w", code, err)`.


// Go: server.go:150 ToReader
// PORT: `lsproto.NewBaseReader` takes a `BufRead` (the caller supplies the
// buffer that Go's bufio adds).


// Go: server.go:162 ToWriter

// Go: server.go:167 `_ Reader = (*lspReader)(nil)`, `_ Writer = (*lspWriter)(nil)`:
// the impls above.



// Go: server.go:171 Server (the fields that other threads use)

// Go: server.go:171 Server (the dispatch-thread fields)



// Go: server.go:323 content mapper registration IDs (tsgo#4712)
const CONTENT_MAPPER_DID_OPEN_REGISTRATION_ID: &str = "content-mapper-did-open";
const CONTENT_MAPPER_DID_CHANGE_REGISTRATION_ID: &str = "content-mapper-did-change";
const CONTENT_MAPPER_DID_CLOSE_REGISTRATION_ID: &str = "content-mapper-did-close";
const CONTENT_MAPPER_DIAGNOSTIC_REGISTRATION_ID: &str = "content-mapper-diagnostic";
const CONTENT_MAPPER_HOVER_REGISTRATION_ID: &str = "content-mapper-hover";
const CONTENT_MAPPER_SIGNATURE_HELP_REGISTRATION_ID: &str = "content-mapper-signature-help";
const CONTENT_MAPPER_DEFINITION_REGISTRATION_ID: &str = "content-mapper-definition";
const CONTENT_MAPPER_TYPE_DEFINITION_REGISTRATION_ID: &str = "content-mapper-type-definition";
const CONTENT_MAPPER_IMPLEMENTATION_REGISTRATION_ID: &str = "content-mapper-implementation";
const CONTENT_MAPPER_REFERENCES_REGISTRATION_ID: &str = "content-mapper-references";
const CONTENT_MAPPER_DOCUMENT_HIGHLIGHT_REGISTRATION_ID: &str = "content-mapper-document-highlight";
const CONTENT_MAPPER_COMPLETION_REGISTRATION_ID: &str = "content-mapper-completion";
const CONTENT_MAPPER_RENAME_REGISTRATION_ID: &str = "content-mapper-rename";
const CONTENT_MAPPER_SEMANTIC_TOKENS_REGISTRATION_ID: &str = "content-mapper-semantic-tokens";
const CONTENT_MAPPER_DOCUMENT_SYMBOL_REGISTRATION_ID: &str = "content-mapper-document-symbol";
const CONTENT_MAPPER_FOLDING_RANGE_REGISTRATION_ID: &str = "content-mapper-folding-range";
const CONTENT_MAPPER_SELECTION_RANGE_REGISTRATION_ID: &str = "content-mapper-selection-range";
const CONTENT_MAPPER_INLAY_HINT_REGISTRATION_ID: &str = "content-mapper-inlay-hint";
const CONTENT_MAPPER_CODE_LENS_REGISTRATION_ID: &str = "content-mapper-code-lens";
const CONTENT_MAPPER_CODE_ACTION_REGISTRATION_ID: &str = "content-mapper-code-action";
const CONTENT_MAPPER_FORMATTING_REGISTRATION_ID: &str = "content-mapper-formatting";
const CONTENT_MAPPER_RANGE_FORMATTING_REGISTRATION_ID: &str = "content-mapper-range-formatting";
const CONTENT_MAPPER_ON_TYPE_FORMATTING_REGISTRATION_ID: &str = "content-mapper-on-type-formatting";
const CONTENT_MAPPER_LINKED_EDITING_REGISTRATION_ID: &str = "content-mapper-linked-editing";
const CONTENT_MAPPER_CALL_HIERARCHY_REGISTRATION_ID: &str = "content-mapper-call-hierarchy";
const CONTENT_MAPPER_WILL_RENAME_FILES_REGISTRATION_ID: &str = "content-mapper-will-rename-files";

// Go: server.go:352 supportedCodeActionKinds (ts#63951)



// Go: server.go:1896 generateDiagnosticDiffString
// PORT: Go `[]*lsproto.Diagnostic` are the borrowed results of
// `lsproto::compare_diagnostics`.

/// Go `time.Now().UnixMilli()`.
fn unix_milli_now() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        Err(e) => -(e.duration().as_millis() as i64),
    }
}




/// The text of a panic value (Go `%v` of the recovered value).
pub fn panic_value_string(r: &(dyn Any + Send)) -> String {
    if let Some(panic) = r.downcast_ref::<crate::core::GoPanic>() {
        panic.message.clone()
    } else if let Some(message) = r.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = r.downcast_ref::<String>() {
        message.clone()
    } else {
        String::new()
    }
}








/// PORT: how long the dispatch loop waits with an empty request queue
/// before it starts idle work that asks for it (`IdleStart::AfterQuiet`: an
/// attempt of the auto-import warm that is not eager). No Go counterpart: Go runs the
/// warm on a goroutine at once, and a request never waits for it. Here a
/// request that arrives while such an attempt runs waits for it (a file
/// event only until the warm's next context check). Clients send the next
/// message within about a millisecond of an answer (fast typing: didChange
/// and a diagnostic pull), so the attempt starts only when they pause.
pub const IDLE_QUIET_PERIOD: Duration = Duration::from_millis(50);

/// The quiet period of a test process in microseconds
/// (`set_idle_quiet_period`), 0 for `IDLE_QUIET_PERIOD`.
static TEST_IDLE_QUIET_PERIOD_US: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Sets the quiet period that idle work waits for in this process, in
/// place of `IDLE_QUIET_PERIOD`. Only tests call it: a longer period lets
/// a test wait between its messages longer than `IDLE_QUIET_PERIOD`, so a
/// loaded host can finish an idle job that starts at once before the next
/// message, while one that waits for quiet still does not start.
#[doc(hidden)]
pub fn set_idle_quiet_period(period: Duration) {
    let us = u64::try_from(period.as_micros()).unwrap_or(u64::MAX).max(1);
    TEST_IDLE_QUIET_PERIOD_US.store(us, Ordering::Relaxed);
}

/// `IDLE_QUIET_PERIOD`, or the period of `set_idle_quiet_period`.
fn idle_quiet_period() -> Duration {
    match TEST_IDLE_QUIET_PERIOD_US.load(Ordering::Relaxed) {
        0 => IDLE_QUIET_PERIOD,
        us => Duration::from_micros(us),
    }
}


// Go: server.go:1053 sendClientRequest
// WARNING: this should only be called in the async portion of a request handler,
// otherwise a deadlock can occur.
// PORT: the reader thread delivers the response, so the dispatch thread can
// wait here in the sync portion too (Go's handleInitialized does).

// Go: server.go:1090 sendClientRequestFireAndForget
// sendClientRequestFireAndForget sends a request to the client without waiting for a response.
// The response, if any, will be silently ignored by the read loop since no pending channel is registered.
// This means any error returned by the client will not be observed. Use only for requests where the
// response value is not needed (e.g., the client always returns null).


// Go: server.go:1103 userFacingRequestFailedError
#[derive(Clone, Debug, PartialEq)]
pub struct UserFacingRequestFailedError(pub String);

impl std::fmt::Display for UserFacingRequestFailedError {
    // Go: server.go:1105 Error
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}



// Go: server.go:1129 sendNotification


/// PORT: the async part of a handler. Go `func() error`.
pub type AsyncWork = Box<dyn FnOnce() -> Result<(), GoError>>;


// Go: server.go:1198 contentMapperFallbackResponse (tsgo#4712)
// contentMapperFallbackResponse returns an empty response for requests made for
// unknown file types not handled by any content mapper. This typically serves a
// short window in time between when the server has unregistered content mapper
// extensions and when the client has stopped sending requests for those file types.
// PORT: Go returns `(any, bool)`; `None` is Go `false`.

// Go: server.go:1227 handlerMap
// handlerMap maps LSP method to a handler function. The handler function executes any work that must be done synchronously
// before other requests/notifications can be processed, and returns any additional work as a function to be executed
// asynchronously after the synchronous work is complete.
// PORT: Go `func(*Server, context.Context, *lsproto.RequestMessage) (func() error, error)`;
// a nil `func() error` is `None`.

// Go: server.go:1229 handlers


// Go: server.go:1300 registerNotificationHandler
// PORT: Go `fn func(*Server, context.Context, Req) error`. `Req` is a
// pointer type (or `NoParams`), so the handler gets `Option<&Req>` (`None`
// is a nil pointer). `lsproto.UnmarshalParams` never gives a nil pointer
// without an error, so the register helpers pass `Some`.

// Go: server.go:1317 registerRequestHandler
// PORT: `params` as in `register_notification_handler`.

// Go: server.go:1341 registerLanguageServiceDocumentRequestHandler
// PORT: Go calls `params.TextDocumentURI()` in the sync part, which
// dereferences the params pointer, so `fn` gets `&Req`. The async part
// owns the decoded params (Go captures the pointer).

// Go: server.go:1368 registerLanguageServiceWithAutoImportsRequestHandler
// PORT: the async part owns the decoded params (Go captures the pointer).

// Go: server.go:1403 registerMultiProjectReferenceRequestHandler
// PORT: the async part owns the decoded params (Go captures the pointer).

// Go: server.go:1431 crossProjectOrchestrator
// PORT: Go `defaultProject *project.Project` is the session's
// `Rc<RefCell<Project>>`. Go stores `req`, which no code reads.

// Go: server.go:1438 `var _ ls.CrossProjectOrchestrator = (*crossProjectOrchestrator)(nil)`: the impl below.










/// PORT: what an API connection of the LSP server is doing, for its
/// protocol (`ApiConnProtocol`) and its session connection
/// (`ApiSessionConn`).
#[derive(Default)]
struct ApiConnState {
    /// The calls to the client that wait for their answer.
    calls: Cell<u32>,
    /// Whether a read runs the dispatch loop.
    serving_lsp: Cell<bool>,
}



/// Counts one call to the client until it is dropped (also on a panic).
struct PendingCall<'a>(&'a Cell<u32>);

impl Drop for PendingCall<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}




// Developer/debugging command handlers


// Go: server.go:2461 parseContentMapperContributions (tsgo#4712)
// PORT: Go `json.Marshal` of the options map (`LSPObject`) writes the keys in Go map
// order (random); `IndexMap` writes them in the order the client sent them.

// Go: server.go:2524 isValidContributedContentMapperExtension (tsgo#4712)

#[cfg(test)]
mod tests {
    use super::*;

    /// A queued LSP message: a request with `id`, or a notification.
    fn message(method: lsproto::Method, id: Option<i32>) -> QueuedRequest {
        QueuedRequest::Request(lsproto::RequestMessage {
            id: id.map(crate::jsonrpc::new_id_int),
            method,
            ..lsproto::RequestMessage::default()
        })
    }

    // followups21 (R168 reviewer): the stream check of `drop_after_pause`
    // (freecheck2) sees only a waiting notification. A waiting request or
    // wake-up keeps the release after a pause; the next didChange drops it
    // at once.
    #[test]
    fn stream_check_sees_only_a_waiting_notification() {
        let ctx = context::background();
        let queue: DynamicQueue<QueuedRequest> = new_dynamic_queue();
        assert!(!next_edit_waits(&queue), "empty queue");
        queue.put(&ctx, QueuedRequest::Wake).unwrap();
        queue
            .put(&ctx, message(lsproto::Method::TEXT_DOCUMENT_HOVER, Some(1)))
            .unwrap();
        assert!(!next_edit_waits(&queue), "a request and a wake-up");
        queue
            .put(
                &ctx,
                message(lsproto::Method::TEXT_DOCUMENT_DID_CHANGE, None),
            )
            .unwrap();
        assert!(next_edit_waits(&queue), "the next didChange waits");
    }

    /// Logs its name when it is dropped.
    struct Freed(&'static str, Rc<RefCell<Vec<&'static str>>>);

    impl Drop for Freed {
        fn drop(&mut self) {
            self.1.borrow_mut().push(self.0);
        }
    }

    // PORT: no Go counterpart (lswarm1). An idle job that starts at once
    // (the eager auto-import warm) runs before the frees that the last
    // message left, and a job that waits for a quiet period runs after
    // them. On the bitecs T1 repro the frees took 0.2 to 0.35 ms, and the
    // next didChange came 0.3 to 0.4 ms after the answer.
    #[test]
    fn an_eager_idle_job_runs_before_the_frees() {
        std::thread::spawn(|| {
            let ctx = context::background();
            gostd::local::keep_garbage();
            for (start, want) in [
                (gostd::local::IdleStart::AtOnce, ["warm", "free"]),
                (gostd::local::IdleStart::AfterQuiet, ["free", "warm"]),
            ] {
                let log = Rc::new(RefCell::new(Vec::new()));
                gostd::local::drop_later(Box::new(Freed("free", log.clone())));
                let job_log = log.clone();
                gostd::local::go_idle(start, Box::new(move || job_log.borrow_mut().push("warm")));
                let mut quiet = Vec::new();
                run_idle_work(
                    &ctx,
                    || false,
                    |period| {
                        quiet.push(period);
                        true
                    },
                );
                assert_eq!(*log.borrow(), want, "{start:?}");
                let period = match start {
                    gostd::local::IdleStart::AtOnce => Duration::ZERO,
                    gostd::local::IdleStart::AfterQuiet => IDLE_QUIET_PERIOD,
                };
                assert_eq!(quiet, [period], "{start:?}");
            }
        })
        .join()
        .expect("idle test thread");
    }
}
