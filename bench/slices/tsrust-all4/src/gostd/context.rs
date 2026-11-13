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

/// A Context carries a deadline, a cancellation signal, and other values
/// across API boundaries.
///
/// Context's methods may be called by multiple goroutines simultaneously.
#[derive(Clone)]
pub struct Context(Ctx);

#[derive(Clone)]
enum Ctx {
    /// Go `backgroundCtx`.
    Background,
    /// Go `todoCtx`.
    Todo,
    /// Go `*cancelCtx`.
    Cancel(Arc<CancelCtx>),
    /// Go `*timerCtx`.
    Timer(Arc<TimerCtx>),
    /// Go `*valueCtx`.
    Value(Arc<ValueCtx>),
    /// Go `withoutCancelCtx`.
    WithoutCancel(Arc<WithoutCancelCtx>),
}

impl Context {
    /// Go `ctx.Deadline()`: the time when work done on behalf of this
    /// context should be canceled. `None` when no deadline is set.
    pub fn deadline(&self) -> Option<Instant> {
        match &self.0 {
            // Go: context/context.go:184 emptyCtx.Deadline
            Ctx::Background | Ctx::Todo => None,
            // Go: the embedded parent Context.
            Ctx::Cancel(c) => c.context.deadline(),
            // Go: context/context.go:670 timerCtx.Deadline
            Ctx::Timer(c) => Some(c.deadline),
            // Go: the embedded parent Context.
            Ctx::Value(c) => c.context.deadline(),
            // Go: context/context.go:597 withoutCancelCtx.Deadline
            Ctx::WithoutCancel(_) => None,
        }
    }

    /// Go `ctx.Done()`: a channel that's closed when work done on behalf of
    /// this context should be canceled. `None` is Go's nil channel: this
    /// context can never be canceled. Successive calls return the same
    /// channel.
    pub fn done(&self) -> Option<Done> {
        match &self.0 {
            // Go: context/context.go:188 emptyCtx.Done
            Ctx::Background | Ctx::Todo => None,
            Ctx::Cancel(c) => Some(c.done()),
            Ctx::Timer(c) => Some(c.cancel_ctx.done()),
            // Go: the embedded parent Context.
            Ctx::Value(c) => c.context.done(),
            // Go: context/context.go:601 withoutCancelCtx.Done
            Ctx::WithoutCancel(_) => None,
        }
    }

    /// Go `ctx.Err()`. If Done is not yet closed, Err returns nil. If Done
    /// is closed, Err returns a non-nil error explaining why:
    /// DeadlineExceeded if the context's deadline passed, or Canceled if the
    /// context was canceled for some other reason.
    pub fn err(&self) -> Option<GoError> {
        match &self.0 {
            // Go: context/context.go:192 emptyCtx.Err
            Ctx::Background | Ctx::Todo => None,
            Ctx::Cancel(c) => c.err(),
            Ctx::Timer(c) => c.cancel_ctx.err(),
            // Go: the embedded parent Context.
            Ctx::Value(c) => c.context.err(),
            // Go: context/context.go:605 withoutCancelCtx.Err
            Ctx::WithoutCancel(_) => None,
        }
    }


}


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

    fn id(&'static self) -> KeyId {
        KeyId {
            addr: self as *const ContextKey<T> as usize,
            type_id: TypeId::of::<T>(),
        }
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

/// A result of Go `value`.
enum ValueRef {
    /// The `*cancelCtx` found for `&cancelCtxKey`.
    CancelCtx(CancelCtxRef),
    User(Arc<dyn Any + Send + Sync>),
}

/// A Go `*cancelCtx`: a plain cancelCtx or the one embedded in a timerCtx.
#[derive(Clone)]
enum CancelCtxRef {
    Cancel(Arc<CancelCtx>),
    Timer(Arc<TimerCtx>),
}

impl CancelCtxRef {
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
/// Background returns a non-nil, empty [Context]. It is never canceled, has no
/// values, and has no deadline. It is typically used by the main function,
/// initialization, and tests, and as the top-level Context for incoming
/// requests.
pub fn background() -> Context {
    Context(Ctx::Background)
}

// Go: context/context.go:224 TODO
/// TODO returns a non-nil, empty [Context]. Code should use context.TODO when
/// it's unclear which Context to use or it is not yet available.
pub fn todo() -> Context {
    Context(Ctx::Todo)
}

// Go: context/context.go:232 CancelFunc
/// A CancelFunc tells an operation to abandon its work.
/// A CancelFunc does not wait for the work to stop.
/// A CancelFunc may be called by multiple goroutines simultaneously.
/// After the first call, subsequent calls to a CancelFunc do nothing.
pub type CancelFunc = Arc<dyn Fn() + Send + Sync>;

// Go: context/context.go:241 WithCancel

// Go: context/context.go:256 CancelCauseFunc
/// A CancelCauseFunc behaves like a [CancelFunc] but additionally sets the
/// cancellation cause. `None` is a nil cause.
pub type CancelCauseFunc = Arc<dyn Fn(Option<GoError>) + Send + Sync>;

// Go: context/context.go:269 WithCancelCause

// Go: context/context.go:274 withCancel
// PORT: named `with_cancel_unexported` because Go `WithCancel` is
// `with_cancel`. Go panics on a nil parent, which cannot occur here.

// Go: context/context.go:289 Cause

/// The `stop` function returned by `after_func`.
pub type AfterFuncStop = Arc<dyn Fn() -> bool + Send + Sync>;

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

/// Go `map[canceler]struct{}` key: the canceler's address.
fn canceler_key<T: ?Sized>(c: &T) -> usize {
    c as *const T as *const () as usize
}

// Go: context/context.go:424 closedchan

// Go: context/context.go:432 cancelCtx
/// A cancelCtx can be canceled. When canceled, it also cancels any children
/// that implement canceler.
struct CancelCtx {
    /// Go embedded `Context` (the parent).
    context: Context,

    mu: Mutex<CancelCtxFields>, // protects following fields
    /// Go `done atomic.Value`: of chan struct{}, created lazily, closed by first cancel call.
    done: OnceLock<Done>,
    /// Go `err atomic.Value`: set to non-nil by the first cancel call.
    err: OnceLock<GoError>,
}

struct CancelCtxFields {
    /// set to nil by the first cancel call.
    /// PORT: Go map order is random; the port cancels children in insertion
    /// order.
    children: Option<IndexMap<usize, Arc<dyn Canceler>>>,
    /// set to non-nil by the first cancel call.
    cause: Option<GoError>,
    /// Go `timerCtx.timer` ("Under cancelCtx.mu."); always `None` for a
    /// plain cancelCtx or an afterFuncCtx.
    timer: Option<timer::Timer>,
}

impl CancelCtx {
    // PORT: Go makes `&cancelCtx{}` and sets the embedded parent in
    // propagateCancel; the port sets it here.
    fn new(parent: Context) -> CancelCtx {
        CancelCtx {
            context: parent,
            mu: Mutex::new(CancelCtxFields {
                children: None,
                cause: None,
                timer: None,
            }),
            done: OnceLock::new(),
            err: OnceLock::new(),
        }
    }

    // Go: context/context.go:449 Done
    fn done(&self) -> Done {
        if let Some(d) = self.done.get() {
            return d.clone();
        }
        let _fields = lock(&self.mu);
        self.done.get_or_init(Done::new).clone()
    }

    // Go: context/context.go:464 Err
    fn err(&self) -> Option<GoError> {
        // An atomic load is ~5x faster than a mutex, which can matter in tight loops.
        if let Some(err) = self.err.get() {
            // Ensure the done channel has been closed before returning a non-nil error.
            self.done().wait();
            return Some(err.clone());
        }
        None
    }

    // Go: context/context.go:476 propagateCancel

    // Go: context/context.go:543 String
}


// Go: context/context.go:536 contextName

// Go: context/context.go:586 WithoutCancel
/// WithoutCancel returns a derived context that points to the parent context
/// and is not canceled when parent is canceled.
/// The returned context returns no Deadline or Err, and its Done channel is nil.
/// Calling [Cause] on the returned context returns nil.
pub fn without_cancel(parent: &Context) -> Context {
    Context(Ctx::WithoutCancel(Arc::new(WithoutCancelCtx {
        c: parent.clone(),
    })))
}

// Go: context/context.go:593 withoutCancelCtx
struct WithoutCancelCtx {
    c: Context,
}

impl WithoutCancelCtx {
    // Go: context/context.go:613 String
}

// Go: context/context.go:626 WithDeadline

// Go: context/context.go:633 WithDeadlineCause

// Go: context/context.go:663 timerCtx
/// A timerCtx carries a timer and a deadline. It embeds a cancelCtx to
/// implement Done and Err. It implements cancel by stopping its timer then
/// delegating to cancelCtx.cancel.
struct TimerCtx {
    cancel_ctx: CancelCtx,
    // Go `timer *time.Timer` is `cancel_ctx.mu`'s `timer` field.
    deadline: Instant,
}

impl TimerCtx {
    // Go: context/context.go:674 String
    // PORT: Go prints `deadline.String()` (wall clock) and the Go duration
    // text; an Instant has no wall clock, so this uses Rust debug text.
}


// Go: context/context.go:704 WithTimeout

// Go: context/context.go:711 WithTimeoutCause

// Go: context/context.go:728 WithValue
/// WithValue returns a derived context that points to the parent Context.
/// In the derived context, the value associated with key is val.
///
/// Use context Values only for request-scoped data that transits processes and
/// APIs, not for passing optional parameters to functions.
///
/// PORT: Go panics on a nil parent, a nil key or a key that is not
/// comparable; none can occur here.
pub fn with_value<T: Send + Sync + 'static>(
    parent: &Context,
    key: &'static ContextKey<T>,
    val: T,
) -> Context {
    Context(Ctx::Value(Arc::new(ValueCtx {
        context: parent.clone(),
        key: key.id(),
        key_name: key.name,
        val: Arc::new(val),
        val_type_name: std::any::type_name::<T>(),
    })))
}

// Go: context/context.go:743 valueCtx
/// A valueCtx carries a key-value pair. It implements Value for that key and
/// delegates all other calls to the embedded Context.
struct ValueCtx {
    context: Context,
    key: KeyId,
    key_name: &'static str,
    val: Arc<dyn Any + Send + Sync>,
    val_type_name: &'static str,
}

impl ValueCtx {
    // Go: context/context.go:763 String

    // Go: context/context.go:751 stringify
    // PORT: Go prints a key with its type name (for example `core.key`); the
    // port prints the key's Go name. A value that is not a string prints its
    // Rust type name where Go prints its Go type name.
}

// Go: context/context.go:776 value
// PORT: Go's `default` case (`c.Value(key)` of a custom Context) cannot
// occur.
fn value(c: &Context, key: &KeyRef) -> Option<ValueRef> {
    let mut c = c.clone();
    loop {
        let next = match &c.0 {
            Ctx::Value(ctx) => {
                if let KeyRef::User(k) = key {
                    if *k == ctx.key {
                        return Some(ValueRef::User(ctx.val.clone()));
                    }
                }
                ctx.context.clone()
            }
            Ctx::Cancel(ctx) => {
                if let KeyRef::CancelCtx = key {
                    return Some(ValueRef::CancelCtx(CancelCtxRef::Cancel(ctx.clone())));
                }
                ctx.context.clone()
            }
            Ctx::WithoutCancel(ctx) => {
                if let KeyRef::CancelCtx = key {
                    // This implements Cause(ctx) == nil
                    // when ctx is created using WithoutCancel.
                    return None;
                }
                ctx.c.clone()
            }
            Ctx::Timer(ctx) => {
                if let KeyRef::CancelCtx = key {
                    return Some(ValueRef::CancelCtx(CancelCtxRef::Timer(ctx.clone())));
                }
                ctx.cancel_ctx.context.clone()
            }
            Ctx::Background | Ctx::Todo => return None,
        };
        c = next;
    }
}
