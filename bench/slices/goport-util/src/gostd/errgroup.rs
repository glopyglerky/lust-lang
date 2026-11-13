//! Go `golang.org/x/sync/errgroup` (v0.21.0 `errgroup/errgroup.go`) on
//! threads.
//!
//! Package errgroup provides synchronization, error propagation, and Context
//! cancellation for groups of goroutines working on subtasks of a common task.
//!
//! PORT: use this only for `Send` work (PORTING.md "Go runtime"). An errgroup
//! over dispatch-thread state runs serially in Go start order instead.

use crate::prelude::*;

use crate::gostd::context::{self, };
use crate::gostd::errors::GoError;
use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Once};

// PORT: Go mutexes do not poison.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// Go: errgroup/errgroup.go:18 token
// PORT: Go `chan token` with capacity n is a counting semaphore here.
struct TokenChan {
    cap: usize,
    len: Mutex<usize>,
    cond: Condvar,
}

impl TokenChan {
    fn new(cap: usize) -> TokenChan {
        TokenChan {
            cap,
            len: Mutex::new(0),
            cond: Condvar::new(),
        }
    }

    /// Go `sem <- token{}` (blocks while the channel is full).
    fn send(&self) {
        let mut len = lock(&self.len);
        while *len >= self.cap {
            len = self.cond.wait(len).unwrap_or_else(|e| e.into_inner());
        }
        *len += 1;
        self.cond.notify_all();
    }

    /// Go `select { case sem <- token{}: return true; default: return false }`.
    fn try_send(&self) -> bool {
        let mut len = lock(&self.len);
        if *len >= self.cap {
            return false;
        }
        *len += 1;
        self.cond.notify_all();
        true
    }

    /// Go `<-sem`.
    fn recv(&self) {
        let mut len = lock(&self.len);
        while *len == 0 {
            len = self.cond.wait(len).unwrap_or_else(|e| e.into_inner());
        }
        *len -= 1;
        self.cond.notify_all();
    }

    /// Go `len(sem)`.
    fn len(&self) -> usize {
        *lock(&self.len)
    }
}

/// Go `sync.WaitGroup`.
struct WaitGroup {
    count: Mutex<i64>,
    cond: Condvar,
}

impl WaitGroup {
    fn new() -> WaitGroup {
        WaitGroup {
            count: Mutex::new(0),
            cond: Condvar::new(),
        }
    }

    fn add(&self, delta: i64) {
        let mut count = lock(&self.count);
        *count += delta;
        if *count < 0 {
            panic!("sync: negative WaitGroup counter");
        }
        if *count == 0 {
            self.cond.notify_all();
        }
    }

    fn done(&self) {
        self.add(-1);
    }

    fn wait(&self) {
        let mut count = lock(&self.count);
        while *count != 0 {
            count = self.cond.wait(count).unwrap_or_else(|e| e.into_inner());
        }
    }
}

// Go: errgroup/errgroup.go:25 Group





// Go: errgroup/errgroup.go:48 WithContext
