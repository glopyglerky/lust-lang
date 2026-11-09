//! Go `internal/project/background/queue.go`.
//!
//! PORT: background tasks touch dispatch-thread state, so `wg.Go` posts the
//! task to `gostd::local::go` (PORTING "Go runtime"). The dispatch loop runs
//! it later, in enqueue order. A Go task that sleeps (`time.After`) goes on
//! in a `gostd::local::after_func` timer; it takes a `TaskHold` so that it
//! still counts as running until then. `Wait` runs
//! `gostd::local::run_pending`, and waits for due timers, until this queue's
//! tasks have finished. `mu` is dropped (one thread).

use crate::project::background::prelude::*;
use std::cell::Cell;

// Go: project/background/queue.go:9 Queue
// Queue manages background tasks execution
pub struct Queue {
    // PORT: Go `wg sync.WaitGroup`: the number of enqueued tasks that have
    // not finished. Shared with the posted tasks.
    wg: Rc<Cell<i32>>,
    closed: Cell<bool>,
}

// Go: project/background/queue.go:16 NewQueue
// NewQueue creates a new background queue for managing background tasks execution.
pub fn new_queue() -> Rc<Queue> {
    Rc::new(Queue {
        wg: Rc::new(Cell::new(0)),
        closed: Cell::new(false),
    })
}

// PORT: Go `defer wg.Done()` inside `wg.Go`.
struct WaitGroupDone(Rc<Cell<i32>>);

impl Drop for WaitGroupDone {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}

/// PORT: the rest of a running task that goes on later on the dispatch
/// thread (Go: the same goroutine after a sleep). A task takes it with
/// `Queue::hold` and moves it into the timer function; the task counts as
/// running until the hold is dropped. The timer function runs the rest
/// under `core::go_wait_group_task`, as `Enqueue` runs the task.
pub struct TaskHold {
    _done: WaitGroupDone,
}

impl Queue {
    // Go: project/background/queue.go:20 Enqueue

    /// PORT: called by a running task whose rest runs later (see
    /// `TaskHold`). No Go counterpart: the Go task is one goroutine.
    pub fn hold(&self) -> TaskHold {
        self.wg.set(self.wg.get() + 1);
        TaskHold {
            _done: WaitGroupDone(self.wg.clone()),
        }
    }

    // Go: project/background/queue.go:44 Wait
    // Wait waits for all active tasks to complete.
    // It does not prevent new tasks from being enqueued while waiting.

    // Go: project/background/queue.go:48 Close
    pub fn close(&self) {
        self.closed.set(true);
    }
}
