//! Go `internal/lsp/logger.go`.
//!
//! PORT: Go `Log(msg ...any)` joins its arguments with `fmt.Sprint` and
//! `Logf(format, args...)` formats with `fmt.Sprintf`. As in
//! `project/logging`, callers pass the finished text, so every method takes
//! `&str`. Log text is not compared with Go.

use crate::lsp::prelude::*;

use crate::project::logging;

use std::io::Write as _;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

// PORT: Go mutexes do not poison.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// Go: lsp/logger.go:11 `var _ logging.Logger = (*logger)(nil)`: the
// `logging::Logger` impl below.

// Go: lsp/logger.go:13 logger
// PORT: Go `server *Server`. The logger runs on the reader thread and the
// dispatch thread, so it points at the `Send` half of the server
// (`ServerShared`). The server owns the logger, so the pointer is `Weak`.
// Go `mu` guards `verbosity`; here the mutex holds it.

// Go: lsp/logger.go:19 newLogger

// Go: lsp/logger.go:28 maxVerbosityForMessageType
// maxVerbosityForMessageType returns the least-verbose log level at which
// messages of the given LSP MessageType should still be sent.
pub fn max_verbosity_for_message_type(msg_type: lsproto::MessageType) -> lsproto::LogVerbosity {
    if msg_type == lsproto::MessageType::ERROR {
        lsproto::LogVerbosity::ERROR
    } else if msg_type == lsproto::MessageType::WARNING {
        lsproto::LogVerbosity::WARNING
    } else if msg_type == lsproto::MessageType::INFO {
        lsproto::LogVerbosity::INFO
    } else if msg_type == lsproto::MessageType::DEBUG {
        lsproto::LogVerbosity::DEBUG
    } else {
        lsproto::LogVerbosity::INFO
    }
}

// Go: lsp/logger.go:44 isValidLogVerbosity
// isValidLogVerbosity reports whether v is one of the defined LogVerbosity values.
pub fn is_valid_log_verbosity(v: lsproto::LogVerbosity) -> bool {
    v >= lsproto::LogVerbosity::OFF && v <= lsproto::LogVerbosity::ERROR
}

// PORT: Go checks `l == nil` in every method. The server always has a
// logger, so the checks are dropped.


// PORT: Go hands the same `*logger` to the session and the lspwatcher as a
// `logging.Logger`. The logger is shared with the reader thread through an
// `Arc`, and those consumers hold `Rc<dyn logging::Logger>`, so they get
// `Rc::new(arc_logger)`; this impl forwards to the logger.
