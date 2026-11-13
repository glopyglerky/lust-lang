//! Go `internal/lsp/dynamic_queue.go`.

use crate::lsp::prelude::*;

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

// PORT: Go mutexes do not poison.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// Inspired by Brian C. Mills' "Rethinking Classical Concurrency Patterns" talk:
// https://www.youtube.com/watch?v=5zXAHh5tJqQ
//
// This queue is a state machine, where each state is a channel, "idle" or "ready".
// Only one caller ever has the actual state struct at a time. The Get function
// will wait until the "ready" channel holds the state. Putting an item
// means grabbing the state from any channel, modifying it, and putting it
// back on the "ready" channel. Since this is all managed via contexts, any method
// can be cancelled while waiting for the state.
//
// PORT: the two one-slot channels ("idle" and "ready") become one mutex over
// the state and a condition variable. Holding the mutex is holding the state.
// The state is "ready" when it has items and "idle" when it has none. A wait
// on the condition variable is a wait on the "ready" channel; a waker on
// `ctx.Done()` wakes it, like the `ctx.Done()` case of Go's `select`. The
// order (FIFO) and the errors (`ctx.Err()`) are Go's.

// Go: lsp/dynamic_queue.go:17 dynamicQueue
pub struct DynamicQueue<T> {
    // PORT: Go `idle` and `ready` channels; see the file comment.
    inner: Arc<DynamicQueueInner<T>>,
}

struct DynamicQueueInner<T> {
    state: Mutex<DynamicQueueState<T>>,
    ready: Condvar,
}

// Go: lsp/dynamic_queue.go:22 dynamicQueueState
pub struct DynamicQueueState<T> {
    pub items: VecDeque<T>,
}

// Go: lsp/dynamic_queue.go:26 newDynamicQueue
pub fn new_dynamic_queue<T: Send + 'static>() -> DynamicQueue<T> {
    DynamicQueue {
        inner: Arc::new(DynamicQueueInner {
            state: Mutex::new(DynamicQueueState {
                items: VecDeque::new(),
            }),
            ready: Condvar::new(),
        }),
    }
}

impl<T: Send + 'static> DynamicQueue<T> {
    // Go: lsp/dynamic_queue.go:35 Put

    // Go: lsp/dynamic_queue.go:50 Get

    /// PORT: no Go counterpart. Runs `f` on the queued items, in order,
    /// while it holds the state. The LSP server takes an LSP `shutdown` or
    /// `exit` out of the queue with it while an API request waits for the
    /// client (`ServerShared::wait_during_api_call`).
    pub fn with_items<R>(&self, f: impl FnOnce(&mut VecDeque<T>) -> R) -> R {
        f(&mut self.get_any().items)
    }

    // Go: lsp/dynamic_queue.go:76 getAny
    // PORT: the state is always available (see `put`), so this only takes
    // the mutex. Go also returns `ctx.Err()` when the context is done while
    // it waits; `put` checks the context first, and taking the mutex never
    // waits for long.
    fn get_any(&self) -> MutexGuard<'_, DynamicQueueState<T>> {
        lock(&self.inner.state)
    }

    // Go: lsp/dynamic_queue.go:87 getReady
    // PORT: Go `select`s on the "ready" channel and `ctx.Done()`; when both
    // are ready Go picks one at random. The port returns the item.
}
