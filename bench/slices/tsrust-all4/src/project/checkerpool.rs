//! Go `internal/project/checkerpool.go`.
//!
//! PORT: the pool lives on the LSP dispatch thread (PORTING.md "Threads").
//! `p.mu` is dropped: every Go `p.mu.Lock()` is `self.mu_lock()`, which
//! only runs the request cleanups whose context is done (see
//! `register_request_cleanup`). Fields that Go changes under the lock are
//! `Cell`/`RefCell`. Go `*checker.Checker` is `Rc<RefCell<Checker>>` from
//! `ls_program::new_checker`; the caller's `borrow_mut` is the exclusive
//! use. The semaphores (`chan struct{}`) are counters: on one thread no
//! other goroutine can release a slot, so a send on a full semaphore is
//! `unreachable!`. The idle-cleanup timer is `gostd::local::after_func`.
//! Go `func()` release results are `ls_program::Release` (once).

use crate::project::prelude::*;

use crate::frontend::core_context::{self, CheckerLifetime};
use crate::program::ls_program::{self, Release};
use std::cell::Cell;
use std::rc::Weak;
use std::time::{Duration, Instant};

// Go: project/checkerpool.go:20 checkerHeldAnonymous
// checkerHeldAnonymous is a sentinel stored in heldBy when a checker is held
// by a caller that has no request ID (e.g., context.Background()). This
// distinguishes "held without ID" from "not held" (empty string).
pub const CHECKER_HELD_ANONYMOUS: &str = "<anonymous>";

// Go: project/checkerpool.go:22 CheckerPoolOptions
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckerPoolOptions {
    // MaxCheckers controls the total number of checker slots per project
    // (1 dedicated diagnostics checker + N-1 query checkers). Minimum 2.
    // Zero uses the default (4).
    pub max_checkers: i32,
    // IdleTimeout controls how long an idle checker is kept
    // before being disposed. Zero uses the default (30s).
    pub idle_timeout: Duration,
}

/// Go `chan struct{}` with a buffer, used as a counting semaphore
/// (`sem <- struct{}{}` claims a slot, `<-sem` releases one).
// PORT: on the dispatch thread nobody else can release a slot while a send
// waits, so Go would block forever there; that is `unreachable!`.
struct Semaphore {
    len: Cell<i32>,
    cap: i32,
}

impl Semaphore {
    fn new(cap: i32) -> Semaphore {
        Semaphore {
            len: Cell::new(0),
            cap,
        }
    }

    /// Go `sem <- struct{}{}`.
    fn send(&self) {
        if self.len.get() >= self.cap {
            unreachable!(
                "checkerpool: semaphore full on the dispatch thread (Go would block forever)"
            );
        }
        self.len.set(self.len.get() + 1);
    }

    /// Go `<-sem`.
    fn recv(&self) {
        if self.len.get() <= 0 {
            unreachable!(
                "checkerpool: semaphore empty on the dispatch thread (Go would block forever)"
            );
        }
        self.len.set(self.len.get() - 1);
    }
}

// Go: project/checkerpool.go:41 checkerPool
// checkerPool manages three categories of type checkers for a project:
//
//   - Diagnostics (index 0): A single checker for LSP diagnostics, providing
//     consistent walk order. Idle-cleaned.
//   - Temporary (indices 1+): Ephemeral query checkers for LSP operations.
//     Idle-cleaned after a configurable timeout.
//   - API: A single checker for API operations, providing stable
//     instance identity for reference equality on type/symbol handles.
//     Never idle-cleaned.
// PORT: Go `time.Time` zero values are `None`. `fileAssociations` is keyed
// by the file root `Node` (Go `*ast.SourceFile`).

// Go: project/checkerpool.go:84 newCheckerPool
// PORT: Go `log func(msg string)` (nil-able) is `Option<Rc<dyn Fn(&str)>>`.

// Go: project/checkerpool.go:116 holdTag
// holdTag returns the value to store in heldBy for the given request ID.
pub fn hold_tag(request_id: &str) -> String {
    if request_id.is_empty() {
        return CHECKER_HELD_ANONYMOUS.to_string();
    }
    request_id.to_string()
}

// Go: project/checkerpool.go:82 `var _ compiler.CheckerPool = (*checkerPool)(nil)`


// Go: project/checkerpool.go:533 noop
pub fn noop() {}

// Not in Go: the GC frees the checkers of a released pool in the background.
// PERF (freecheck1): a language server edit frees the old program's pool on
// the first request after the edit, before its answer: 1 or 2 checkers (hono
// 36 MiB each; per edit hono 2.7 ms when the memory is hot and 6.5 ms after a
// client pause, query-core 0.3 ms). In the first release after a client pause
// each checker waits for `drop_garbage` (`gostd::local::drop_after_pause`).
// One value per checker, so a message that comes during the frees waits for
// one checker at most. A checker that is still held elsewhere is only
// decremented later.
