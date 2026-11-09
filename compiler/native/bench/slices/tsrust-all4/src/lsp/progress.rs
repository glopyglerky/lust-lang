//! Go `internal/lsp/progress.go`.
//!
//! PORT: the progress goroutine becomes a thread. It touches only `Send`
//! data: the outgoing queue and the write-once server state behind
//! `ServerShared`. Go `...any` message arguments are `Vec<String>` (built
//! with `args![..]`), as in `project::Client`.

use crate::lsp::prelude::*;

use crate::gostd::context::Done;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TrySendError, sync_channel};
use std::time::{Duration, Instant};

// Go: lsp/progress.go:13 progressEvent

// Go: lsp/progress.go:22 progressReporter
// progressReporter abstracts the LSP transport operations needed by
// projectLoadingProgress so the progress logic can be tested without a
// full Server instance.

// Go: lsp/progress.go:34 serverProgressReporter
// serverProgressReporter adapts *Server to the progressReporter interface.


// Go: lsp/progress.go:70 projectLoadingProgress
// projectLoadingProgress manages LSP WorkDoneProgress indicators for
// long-running operations. A single persistent goroutine processes
// start/finish events, maintains a ref-counted map of active operations,
// and sends progress messages in order.
//
// To avoid flickering on fast operations, the indicator is not shown
// until progressDelay has elapsed since the first start event. If all
// operations complete before then, no progress UI is displayed.
//
// start/finish may block if the internal buffer (64 events) is full,
// but will bail out if the server's background context is cancelled.
//
// PORT: Go `ch chan progressEvent`. The channel carries `Option`: `None`
// is a wake-up that a waker on `reporter.done()` sends, so `run` can wait
// on the channel alone (Go `select`s on the channel, the delay timer and
// `done()`). The receiving end goes to the `run` thread.

// Go: lsp/progress.go:76 newProjectLoadingProgress

// Go: lsp/progress.go:80 newProjectLoadingProgressFromReporter

