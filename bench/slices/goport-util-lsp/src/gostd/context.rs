//! Go `context` (go1.27.1 `src/context/context.go`; it differs from go1.26.8
//! only in the package comment).
//!
//! PORT: Go `Context` is an interface. The port has only the standard
//! implementations (background, TODO, cancel, timer, value, withoutCancel),
//! so `Context` is a closed enum of shared handles. Code paths that exist in
//! Go only for custom `Context` implementations (the `afterFuncer` parent,
//! `stopCtx`, the watcher goroutine in `propagateCancel`, the `default`
//! branch of `value`) cannot run and are marked where they would be.
//!
//! A Go channel `<-chan struct{}` from `Done()` is `Done`. Go `time.Time` is
//! `std::time::Instant`.

use crate::prelude::*;

use crate::gostd::errors::{self, GoError};
use crate::gostd::timer;
use std::any::{Any, TypeId};
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, LazyLock, Mutex, MutexGuard, Once, OnceLock};
use std::time::{Duration, Instant};

// PORT: Go mutexes do not poison. A panic while a lock is held leaves the
// data as it is, as in Go.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

// PORT: Go `go f()`.





/// A typed key for `with_value` and `Context::value`. Declare one static per
/// Go key: `pub static KEY: ContextKey<T> = ContextKey::new("goName");`.
/// Two keys are equal only when they are the same static.
pub struct ContextKey<T> {
    name: &'static str,
    _value: PhantomData<fn() -> T>,
}

impl<T: 'static> ContextKey<T> {
    pub const fn new(name: &'static str) -> ContextKey<T> {
        ContextKey {
            name,
            _value: PhantomData,
        }
    }

    /// The Go name of the key (used by `String()`).
    pub fn name(&self) -> &'static str {
        self.name
    }

}

impl<T> fmt::Debug for ContextKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)
    }
}

/// Go `key == ctx.key` for a user key: the same static.
#[derive(Clone, Copy, PartialEq, Eq)]
struct KeyId {
    addr: usize,
    type_id: TypeId,
}

/// A key passed to Go `value`: a user key or `&cancelCtxKey`.
enum KeyRef {
    /// Go `&cancelCtxKey`.
    CancelCtx,
    User(KeyId),
}




/// Go `<-chan struct{}` returned by `Context::done`.
///
/// PORT: a Go channel is waited on with `select`. `Done` offers the waits
/// the port needs: `is_closed` (a `select` with `default`), `wait`,
/// `wait_timeout`, and `register_waker` so that code blocked on its own
/// condition variable wakes when the channel closes.
#[derive(Clone)]
pub struct Done(Arc<DoneInner>);

impl fmt::Debug for Done {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Done(closed: {})", self.is_closed())
    }
}

struct DoneInner {
    state: Mutex<DoneState>,
    cond: Condvar,
}

struct DoneState {
    closed: bool,
    next_waker: u64,
    wakers: Vec<(u64, Box<dyn FnOnce() + Send>)>,
}

impl Done {
    /// Go `make(chan struct{})`.
    fn new() -> Done {
        Done(Arc::new(DoneInner {
            state: Mutex::new(DoneState {
                closed: false,
                next_waker: 0,
                wakers: Vec::new(),
            }),
            cond: Condvar::new(),
        }))
    }

    /// Whether the channel is closed (Go `select { case <-done: ...; default: }`).
    pub fn is_closed(&self) -> bool {
        lock(&self.0.state).closed
    }

    /// Go `<-done`: blocks until the channel is closed.
    pub fn wait(&self) {
        let mut state = lock(&self.0.state);
        while !state.closed {
            state = self.0.cond.wait(state).unwrap_or_else(|e| e.into_inner());
        }
    }



    /// Registers `f` to run once when the channel closes. Returns an id for
    /// `unregister_waker`, or `None` when the channel is already closed (then
    /// `f` is dropped without running; the caller checks `ctx.err()`).
    ///
    /// `f` runs on the thread that cancels the context, after the channel is
    /// marked closed and the context locks are released. It should only
    /// signal (for example lock a mutex and notify a condition variable).
    pub fn register_waker(&self, f: impl FnOnce() + Send + 'static) -> Option<u64> {
        let mut state = lock(&self.0.state);
        if state.closed {
            return None;
        }
        let id = state.next_waker;
        state.next_waker += 1;
        state.wakers.push((id, Box::new(f)));
        Some(id)
    }


    /// Go channel equality (`done == closedchan`).
    pub fn ptr_eq(&self, other: &Done) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

}




// Go: context/context.go:168 Canceled

// Go: context/context.go:172 DeadlineExceeded

// Go: context/context.go:174 deadlineExceededError
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeadlineExceededError;

impl fmt::Display for DeadlineExceededError {
    // Go: context/context.go:176 Error
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("context deadline exceeded")
    }
}

impl DeadlineExceededError {
    // Go: context/context.go:177 Timeout
    pub fn timeout(&self) -> bool {
        true
    }

    // Go: context/context.go:178 Temporary
    pub fn temporary(&self) -> bool {
        true
    }
}

// Go: context/context.go:216 Background

// Go: context/context.go:224 TODO

// Go: context/context.go:232 CancelFunc

// Go: context/context.go:241 WithCancel

// Go: context/context.go:256 CancelCauseFunc

// Go: context/context.go:269 WithCancelCause

// Go: context/context.go:274 withCancel
// PORT: named `with_cancel_unexported` because Go `WithCancel` is
// `with_cancel`. Go panics on a nil parent, which cannot occur here.

// Go: context/context.go:289 Cause


// Go: context/context.go:326 AfterFunc

/// The number of threads that `after_func` callbacks have started in this
/// process. Tests read it to check that a code path starts no thread.
pub static AFTER_FUNC_GOROUTINES: AtomicUsize = AtomicUsize::new(0);

// Go: context/context.go:343 afterFuncer
// PORT: only custom Context implementations have an AfterFunc method; the
// port has none, so the interface is not needed.

// Go: context/context.go:347 afterFuncCtx


// Go: context/context.go:366 stopCtx
// PORT: a stopCtx replaces the parent of a cancelCtx whose parent has an
// AfterFunc method (a custom Context). The port has none.

// Go: context/context.go:372 goroutines
// PORT: a test counter for the watcher goroutine of propagateCancel, which
// the port never starts.

// Go: context/context.go:383 parentCancelCtx

// Go: context/context.go:400 removeChild

// Go: context/context.go:418 canceler
/// A canceler is a context type that can be canceled directly. The
/// implementations are *cancelCtx and *timerCtx.
trait Canceler: Send + Sync {
    fn cancel(&self, remove_from_parent: bool, err: GoError, cause: Option<GoError>);
    fn done(&self) -> Done;
}


// Go: context/context.go:424 closedchan

// Go: context/context.go:432 cancelCtx




// Go: context/context.go:536 contextName

// Go: context/context.go:586 WithoutCancel

// Go: context/context.go:593 withoutCancelCtx


// Go: context/context.go:626 WithDeadline

// Go: context/context.go:633 WithDeadlineCause

// Go: context/context.go:663 timerCtx



// Go: context/context.go:704 WithTimeout

// Go: context/context.go:711 WithTimeoutCause

// Go: context/context.go:728 WithValue

// Go: context/context.go:743 valueCtx


// Go: context/context.go:776 value
// PORT: Go's `default` case (`c.Value(key)` of a custom Context) cannot
// occur.
