// The `std::sync` and `std::thread` surface in body checking (W-THREADS,
// ARCH 7.3). The types are builtin Adts (ty/sync.rs); their associated
// fns, methods, and free fns type here as `Builtin::Sync` calls, and
// codegen emits them (cgen/sync.rs) over the runtime (runtime/sync.c).
//
// Every name here is std's exact stable surface. A method lust lacks on
// a known type rejects with L0640 (rule zero: never fall through to a
// user method of the same name on a std type).
use super::*;
use crate::ty::sync::{AtomicVal, SyncTy, atomic_of, sync_ty};

/// An atomic read-modify-write method (`fetch_add` and kin).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FetchOp {
    Add,
    Sub,
    And,
    Nand,
    Or,
    Xor,
    Max,
    Min,
}

/// One sync or thread builtin call (`Builtin::Sync`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SyncOp {
    /// `AtomicX::new(v)` (a `const fn`).
    AtomicNew(SyncTy),
    /// `AtomicX::from(v)` (not const).
    AtomicFrom(SyncTy),
    /// `AtomicX::default()`: the zero value.
    AtomicDefault(SyncTy),
    AtomicLoad,
    AtomicStore,
    AtomicSwap,
    AtomicCompareExchange,
    AtomicCompareExchangeWeak,
    AtomicFetch(FetchOp),
    /// `AtomicBool::fetch_not`.
    AtomicFetchNot,
    /// `fetch_update(set_order, fetch_order, f)`: std's load, then
    /// compare-exchange loop while `f` gives `Some`.
    AtomicFetchUpdate,
    AtomicIntoInner,
    AtomicGetMut,
    /// `atomic::fence(order)`.
    Fence,
    /// `atomic::compiler_fence(order)`.
    CompilerFence,
    /// `hint::spin_loop()`.
    SpinLoop,
    /// `Arc::new(v)`.
    ArcNew,
    ArcStrongCount,
    ArcWeakCount,
    ArcPtrEq,
    /// `Arc::as_ptr` (w-cinterop): the payload address.
    ArcAsPtr,
    ArcDowngrade,
    ArcGetMut,
    ArcMakeMut,
    ArcTryUnwrap,
    ArcIntoInner,
    ArcUnwrapOrClone,
    /// `sync::Weak::new()`.
    ArcWeakNew,
    ArcWeakUpgrade,
    ArcWeakStrongCount,
    ArcWeakWeakCount,
    ArcWeakPtrEq,
    /// `Mutex::new(v)` or `RwLock::new(v)` (a `const fn`).
    LockNew(SyncTy),
    /// `Mutex::from(v)` or `RwLock::from(v)`.
    LockFrom(SyncTy),
    /// `Mutex::default()` or `RwLock::default()`.
    LockDefault(SyncTy),
    /// `Mutex::lock`.
    MutexLock,
    /// `RwLock::read`.
    RwLockRead,
    /// `RwLock::write`.
    RwLockWrite,
    /// `get_mut(&mut self) -> LockResult<&mut T>` on a lock.
    LockGetMut,
    /// `into_inner(self) -> LockResult<T>` on a lock.
    LockIntoInner,
    LockIsPoisoned,
    LockClearPoison,
    /// `PoisonError::new(v)`.
    PoisonNew,
    PoisonIntoInner,
    PoisonGetRef,
    PoisonGetMut,
    /// `thread::spawn(f)`.
    ThreadSpawn,
    /// `panic::catch_unwind(f)`, `panic::resume_unwind(payload)`, and
    /// `panic::panic_any(msg)` (W-UNWIND; typed in check/unwind.rs).
    CatchUnwind,
    ResumeUnwind,
    PanicAny,
    /// `core::intrinsics::panic_hook_swap(new)` and `panic_hook_ref()`:
    /// the runtime's hook slot, for std's `set_hook`/`take_hook` source.
    PanicHookSwap,
    PanicHookRef,
    /// `core::intrinsics::panic_hook_payload()` and
    /// `panic_hook_location()`: the panic the hook runs for.
    PanicHookPayload,
    PanicHookLocation,
    /// `thread::current()`.
    ThreadCurrent,
    /// `thread::yield_now()`.
    ThreadYieldNow,
    /// `thread::panicking()` (runtime/panic.c: unwinding, or in a hook).
    ThreadPanicking,
    /// `JoinHandle::join(self)`.
    JoinHandleJoin,
    /// `JoinHandle::thread(&self) -> &Thread`.
    JoinHandleThread,
    JoinHandleIsFinished,
    /// `Thread::id(&self) -> ThreadId`.
    ThreadGetId,
    /// `Thread::name(&self) -> Option<&str>`.
    ThreadGetName,
    /// `thread::scope(f)`.
    ThreadScope,
    /// `Scope::spawn(&self, f)`.
    ScopeSpawn,
    /// `mpsc::channel()`.
    Channel,
    /// `mpsc::sync_channel(bound)`.
    SyncChannel,
    /// `Sender::send` and `SyncSender::send`.
    ChanSend,
    /// `Receiver::recv`.
    ChanRecv,
    /// `Receiver::try_recv`.
    ChanTryRecv,
    /// `Receiver::recv_timeout(dur)` (tf-p6 R09).
    ChanRecvTimeout,
    /// `SyncSender::try_send` (TRIAGE T5).
    ChanTrySend,
    /// `Arc::new_cyclic(data_fn)` (TRIAGE T5).
    ArcNewCyclic,
    /// `Condvar::wait_timeout` (TRIAGE T5).
    CondvarWaitTimeout,
    /// `Condvar::wait_timeout_ms` (deprecated in std, same form).
    CondvarWaitTimeoutMs,
    /// `WaitTimeoutResult::timed_out`.
    WaitTimedOut,
    /// `thread::Builder::new` (TRIAGE T5).
    BuilderNew,
    /// `Builder::name(self, name)`.
    BuilderName,
    /// `Builder::stack_size(self, size)`.
    BuilderStackSize,
    /// `Builder::spawn(self, f)`.
    BuilderSpawn,
    /// `Builder::spawn_scoped(self, scope, f)`.
    BuilderSpawnScoped,
    /// `thread::available_parallelism`.
    AvailPar,
    /// `Condvar::new()`, `Barrier::new(n)`, `OnceLock::new()` (const fns).
    SyncNew(SyncTy),
    CondvarWait,
    /// `Condvar::wait_while(guard, condition)`.
    CondvarWaitWhile,
    /// `Condvar::wait_timeout_while(guard, dur, condition)`.
    CondvarWaitTimeoutWhile,
    CondvarNotifyOne,
    CondvarNotifyAll,
    BarrierWait,
    BarrierIsLeader,
    OnceGet,
    OnceGetMut,
    OnceSet,
    OnceGetOrInit,
    OnceIntoInner,
    OnceTake,
    /// `Duration::new(secs, nanos)`.
    DurNew,
    /// `Duration::from_secs` (0), `from_millis` (1), `from_micros` (2),
    /// `from_nanos` (3).
    DurFrom(u8),
    /// `as_secs` (0), `as_millis` (1), `as_micros` (2), `as_nanos` (3).
    DurAs(u8),
    /// `subsec_millis` (1), `subsec_micros` (2), `subsec_nanos` (3).
    DurSubsec(u8),
    DurIsZero,
    /// `thread::sleep(dur)`.
    ThreadSleep,
    /// `Mutex::try_lock`, `RwLock::try_read` (false), `try_write` (true).
    TryLock(bool),
    /// `rx.iter()` (0), `rx.try_iter()` (1), `rx.into_iter()` (2), and
    /// the `for x in &rx` / `for x in rx` conversions.
    RecvMakeIter(u8),
    /// One `next` of an mpsc iterator: `recv` (true) or `try_recv`, as
    /// an `Option<T>` (lower/iter.rs makes these calls).
    RecvNext(bool),
}

impl SyncOp {
    /// One of each op, for the capabilities table.
    pub const ALL: [SyncOp; 121] = [
        SyncOp::AtomicNew(SyncTy::AtomicUsize),
        SyncOp::AtomicFrom(SyncTy::AtomicUsize),
        SyncOp::AtomicDefault(SyncTy::AtomicUsize),
        SyncOp::AtomicLoad,
        SyncOp::AtomicStore,
        SyncOp::AtomicSwap,
        SyncOp::AtomicCompareExchange,
        SyncOp::AtomicCompareExchangeWeak,
        SyncOp::AtomicFetch(FetchOp::Add),
        SyncOp::AtomicFetch(FetchOp::Sub),
        SyncOp::AtomicFetch(FetchOp::And),
        SyncOp::AtomicFetch(FetchOp::Nand),
        SyncOp::AtomicFetch(FetchOp::Or),
        SyncOp::AtomicFetch(FetchOp::Xor),
        SyncOp::AtomicFetch(FetchOp::Max),
        SyncOp::AtomicFetch(FetchOp::Min),
        SyncOp::AtomicFetchNot,
        SyncOp::AtomicFetchUpdate,
        SyncOp::AtomicIntoInner,
        SyncOp::AtomicGetMut,
        SyncOp::Fence,
        SyncOp::CompilerFence,
        SyncOp::SpinLoop,
        SyncOp::ArcNew,
        SyncOp::ArcStrongCount,
        SyncOp::ArcWeakCount,
        SyncOp::ArcPtrEq,
        SyncOp::ArcAsPtr,
        SyncOp::ArcDowngrade,
        SyncOp::ArcGetMut,
        SyncOp::ArcMakeMut,
        SyncOp::ArcTryUnwrap,
        SyncOp::ArcIntoInner,
        SyncOp::ArcUnwrapOrClone,
        SyncOp::ArcWeakNew,
        SyncOp::ArcWeakUpgrade,
        SyncOp::ArcWeakStrongCount,
        SyncOp::ArcWeakWeakCount,
        SyncOp::ArcWeakPtrEq,
        SyncOp::LockNew(SyncTy::Mutex),
        SyncOp::LockFrom(SyncTy::Mutex),
        SyncOp::LockDefault(SyncTy::Mutex),
        SyncOp::MutexLock,
        SyncOp::RwLockRead,
        SyncOp::RwLockWrite,
        SyncOp::LockGetMut,
        SyncOp::LockIntoInner,
        SyncOp::LockIsPoisoned,
        SyncOp::LockClearPoison,
        SyncOp::PoisonNew,
        SyncOp::PoisonIntoInner,
        SyncOp::PoisonGetRef,
        SyncOp::PoisonGetMut,
        SyncOp::ThreadSpawn,
        SyncOp::CatchUnwind,
        SyncOp::ResumeUnwind,
        SyncOp::PanicAny,
        SyncOp::PanicHookSwap,
        SyncOp::PanicHookRef,
        SyncOp::PanicHookPayload,
        SyncOp::PanicHookLocation,
        SyncOp::ThreadCurrent,
        SyncOp::ThreadYieldNow,
        SyncOp::ThreadPanicking,
        SyncOp::JoinHandleJoin,
        SyncOp::JoinHandleThread,
        SyncOp::JoinHandleIsFinished,
        SyncOp::ThreadGetId,
        SyncOp::ThreadGetName,
        SyncOp::ThreadScope,
        SyncOp::ScopeSpawn,
        SyncOp::Channel,
        SyncOp::SyncChannel,
        SyncOp::ChanSend,
        SyncOp::ChanRecv,
        SyncOp::ChanTryRecv,
        SyncOp::ChanRecvTimeout,
        SyncOp::ChanTrySend,
        SyncOp::ArcNewCyclic,
        SyncOp::CondvarWaitTimeout,
        SyncOp::CondvarWaitTimeoutMs,
        SyncOp::WaitTimedOut,
        SyncOp::BuilderNew,
        SyncOp::BuilderName,
        SyncOp::BuilderStackSize,
        SyncOp::BuilderSpawn,
        SyncOp::BuilderSpawnScoped,
        SyncOp::AvailPar,
        SyncOp::SyncNew(SyncTy::Condvar),
        SyncOp::CondvarWait,
        SyncOp::CondvarWaitWhile,
        SyncOp::CondvarWaitTimeoutWhile,
        SyncOp::CondvarNotifyOne,
        SyncOp::CondvarNotifyAll,
        SyncOp::BarrierWait,
        SyncOp::BarrierIsLeader,
        SyncOp::OnceGet,
        SyncOp::OnceGetMut,
        SyncOp::OnceSet,
        SyncOp::OnceGetOrInit,
        SyncOp::OnceIntoInner,
        SyncOp::OnceTake,
        SyncOp::DurNew,
        SyncOp::DurFrom(0),
        SyncOp::DurFrom(1),
        SyncOp::DurFrom(2),
        SyncOp::DurFrom(3),
        SyncOp::DurAs(0),
        SyncOp::DurAs(1),
        SyncOp::DurAs(2),
        SyncOp::DurAs(3),
        SyncOp::DurSubsec(1),
        SyncOp::DurSubsec(2),
        SyncOp::DurSubsec(3),
        SyncOp::DurIsZero,
        SyncOp::ThreadSleep,
        SyncOp::TryLock(false),
        SyncOp::TryLock(true),
        SyncOp::RecvMakeIter(0),
        SyncOp::RecvMakeIter(1),
        SyncOp::RecvMakeIter(2),
    ];

    /// (types, names, how it is called) for `lust capabilities`.
    pub fn info(self) -> (&'static [&'static str], &'static [&'static str], u8) {
        const ATOMICS: &[&str] = &["std::sync::atomic::Atomic{Bool,I8,U8,I16,U16,I32,U32,I64,U64,Isize,Usize}"];
        const INTS: &[&str] = &["std::sync::atomic::Atomic{I8,U8,I16,U16,I32,U32,I64,U64,Isize,Usize}"];
        const LOCKS: &[&str] = &["std::sync::Mutex<T>", "std::sync::RwLock<T>"];
        let (m, a, f) = (0u8, 1u8, 2u8);
        match self {
            SyncOp::AtomicNew(_) => (ATOMICS, &["new"], a),
            SyncOp::AtomicFrom(_) => (ATOMICS, &["from"], a),
            SyncOp::AtomicDefault(_) => (ATOMICS, &["default"], a),
            SyncOp::AtomicLoad => (ATOMICS, &["load"], m),
            SyncOp::AtomicStore => (ATOMICS, &["store"], m),
            SyncOp::AtomicSwap => (ATOMICS, &["swap"], m),
            SyncOp::AtomicCompareExchange => (ATOMICS, &["compare_exchange"], m),
            SyncOp::AtomicCompareExchangeWeak => (ATOMICS, &["compare_exchange_weak"], m),
            SyncOp::AtomicFetch(FetchOp::Add) => (INTS, &["fetch_add"], m),
            SyncOp::AtomicFetch(FetchOp::Sub) => (INTS, &["fetch_sub"], m),
            SyncOp::AtomicFetch(FetchOp::And) => (ATOMICS, &["fetch_and"], m),
            SyncOp::AtomicFetch(FetchOp::Nand) => (ATOMICS, &["fetch_nand"], m),
            SyncOp::AtomicFetch(FetchOp::Or) => (ATOMICS, &["fetch_or"], m),
            SyncOp::AtomicFetch(FetchOp::Xor) => (ATOMICS, &["fetch_xor"], m),
            SyncOp::AtomicFetch(FetchOp::Max) => (INTS, &["fetch_max"], m),
            SyncOp::AtomicFetch(FetchOp::Min) => (INTS, &["fetch_min"], m),
            SyncOp::AtomicFetchNot => (&["std::sync::atomic::AtomicBool"], &["fetch_not"], m),
            SyncOp::AtomicFetchUpdate => (ATOMICS, &["fetch_update"], m),
            SyncOp::AtomicIntoInner => (ATOMICS, &["into_inner"], m),
            SyncOp::AtomicGetMut => (ATOMICS, &["get_mut"], m),
            SyncOp::Fence => (&["std::sync::atomic"], &["fence"], f),
            SyncOp::CompilerFence => (&["std::sync::atomic"], &["compiler_fence"], f),
            SyncOp::SpinLoop => (&["std::hint"], &["spin_loop"], f),
            SyncOp::ArcNew => (&["std::sync::Arc<T>"], &["new"], a),
            SyncOp::ArcStrongCount => (&["std::sync::Arc<T>"], &["strong_count"], a),
            SyncOp::ArcWeakCount => (&["std::sync::Arc<T>"], &["weak_count"], a),
            SyncOp::ArcPtrEq => (&["std::sync::Arc<T>"], &["ptr_eq"], a),
            SyncOp::ArcAsPtr => (&["std::sync::Arc<T>"], &["as_ptr"], a),
            SyncOp::ArcDowngrade => (&["std::sync::Arc<T>"], &["downgrade"], a),
            SyncOp::ArcGetMut => (&["std::sync::Arc<T>"], &["get_mut"], a),
            SyncOp::ArcMakeMut => (&["std::sync::Arc<T>"], &["make_mut"], a),
            SyncOp::ArcTryUnwrap => (&["std::sync::Arc<T>"], &["try_unwrap"], a),
            SyncOp::ArcIntoInner => (&["std::sync::Arc<T>"], &["into_inner"], a),
            SyncOp::ArcUnwrapOrClone => (&["std::sync::Arc<T>"], &["unwrap_or_clone"], a),
            SyncOp::ArcWeakNew => (&["std::sync::Weak<T>"], &["new"], a),
            SyncOp::ArcWeakUpgrade => (&["std::sync::Weak<T>"], &["upgrade"], m),
            SyncOp::ArcWeakStrongCount => (&["std::sync::Weak<T>"], &["strong_count"], m),
            SyncOp::ArcWeakWeakCount => (&["std::sync::Weak<T>"], &["weak_count"], m),
            SyncOp::ArcWeakPtrEq => (&["std::sync::Weak<T>"], &["ptr_eq"], m),
            SyncOp::LockNew(_) => (LOCKS, &["new"], a),
            SyncOp::LockFrom(_) => (LOCKS, &["from"], a),
            SyncOp::LockDefault(_) => (LOCKS, &["default"], a),
            SyncOp::MutexLock => (&["std::sync::Mutex<T>"], &["lock"], m),
            SyncOp::RwLockRead => (&["std::sync::RwLock<T>"], &["read"], m),
            SyncOp::RwLockWrite => (&["std::sync::RwLock<T>"], &["write"], m),
            SyncOp::LockGetMut => (LOCKS, &["get_mut"], m),
            SyncOp::LockIntoInner => (LOCKS, &["into_inner"], m),
            SyncOp::LockIsPoisoned => (LOCKS, &["is_poisoned"], m),
            SyncOp::LockClearPoison => (LOCKS, &["clear_poison"], m),
            SyncOp::PoisonNew => (&["std::sync::PoisonError<T>"], &["new"], a),
            SyncOp::PoisonIntoInner => (&["std::sync::PoisonError<T>"], &["into_inner"], m),
            SyncOp::PoisonGetRef => (&["std::sync::PoisonError<T>"], &["get_ref"], m),
            SyncOp::PoisonGetMut => (&["std::sync::PoisonError<T>"], &["get_mut"], m),
            SyncOp::ThreadSpawn => (&["std::thread"], &["spawn"], f),
            SyncOp::ThreadCurrent => (&["std::thread"], &["current"], f),
            SyncOp::ThreadYieldNow => (&["std::thread"], &["yield_now"], f),
            SyncOp::CatchUnwind => (&["std::panic"], &["catch_unwind"], f),
            SyncOp::ResumeUnwind => (&["std::panic"], &["resume_unwind"], f),
            SyncOp::PanicAny => (&["std::panic"], &["panic_any"], f),
            SyncOp::PanicHookSwap => (&["core::intrinsics (std source only)"], &["panic_hook_swap"], f),
            SyncOp::PanicHookRef => (&["core::intrinsics (std source only)"], &["panic_hook_ref"], f),
            SyncOp::PanicHookPayload => (&["core::intrinsics (std source only)"], &["panic_hook_payload"], f),
            SyncOp::PanicHookLocation => (&["core::intrinsics (std source only)"], &["panic_hook_location"], f),
            SyncOp::ThreadPanicking => (&["std::thread"], &["panicking"], f),
            SyncOp::JoinHandleJoin => (&["std::thread::JoinHandle<T>"], &["join"], m),
            SyncOp::JoinHandleThread => (&["std::thread::JoinHandle<T>"], &["thread"], m),
            SyncOp::JoinHandleIsFinished => (&["std::thread::JoinHandle<T>"], &["is_finished"], m),
            SyncOp::ThreadGetId => (&["std::thread::Thread"], &["id"], m),
            SyncOp::ThreadGetName => (&["std::thread::Thread"], &["name"], m),
            SyncOp::ThreadScope => (&["std::thread"], &["scope"], f),
            SyncOp::ScopeSpawn => (&["std::thread::Scope"], &["spawn"], m),
            SyncOp::Channel => (&["std::sync::mpsc"], &["channel"], f),
            SyncOp::SyncChannel => (&["std::sync::mpsc"], &["sync_channel"], f),
            SyncOp::ChanSend => (&["std::sync::mpsc::Sender<T>", "std::sync::mpsc::SyncSender<T>"], &["send"], m),
            SyncOp::ChanRecv => (&["std::sync::mpsc::Receiver<T>"], &["recv"], m),
            SyncOp::ChanTryRecv => (&["std::sync::mpsc::Receiver<T>"], &["try_recv"], m),
            SyncOp::ChanRecvTimeout => (&["std::sync::mpsc::Receiver<T>"], &["recv_timeout"], m),
            SyncOp::ChanTrySend => (&["std::sync::mpsc::SyncSender<T>"], &["try_send"], m),
            SyncOp::ArcNewCyclic => (&["alloc::sync::Arc<T>"], &["new_cyclic"], a),
            SyncOp::CondvarWaitTimeout => (&["std::sync::Condvar"], &["wait_timeout"], m),
            SyncOp::CondvarWaitTimeoutMs => (&["std::sync::Condvar"], &["wait_timeout_ms"], m),
            SyncOp::WaitTimedOut => (&["std::sync::WaitTimeoutResult"], &["timed_out"], m),
            SyncOp::BuilderNew => (&["std::thread::Builder"], &["new"], a),
            SyncOp::BuilderName => (&["std::thread::Builder"], &["name"], m),
            SyncOp::BuilderStackSize => (&["std::thread::Builder"], &["stack_size"], m),
            SyncOp::BuilderSpawn => (&["std::thread::Builder"], &["spawn"], m),
            SyncOp::BuilderSpawnScoped => (&["std::thread::Builder"], &["spawn_scoped"], m),
            SyncOp::AvailPar => (&["std::thread"], &["available_parallelism"], f),
            SyncOp::SyncNew(_) => (&["std::sync::Condvar", "std::sync::Barrier", "std::sync::OnceLock<T>"], &["new"], a),
            SyncOp::CondvarWait => (&["std::sync::Condvar"], &["wait"], m),
            SyncOp::CondvarWaitWhile => (&["std::sync::Condvar"], &["wait_while"], m),
            SyncOp::CondvarWaitTimeoutWhile => (&["std::sync::Condvar"], &["wait_timeout_while"], m),
            SyncOp::CondvarNotifyOne => (&["std::sync::Condvar"], &["notify_one"], m),
            SyncOp::CondvarNotifyAll => (&["std::sync::Condvar"], &["notify_all"], m),
            SyncOp::BarrierWait => (&["std::sync::Barrier"], &["wait"], m),
            SyncOp::BarrierIsLeader => (&["std::sync::BarrierWaitResult"], &["is_leader"], m),
            SyncOp::OnceGet => (&["std::sync::OnceLock<T>"], &["get"], m),
            SyncOp::OnceGetMut => (&["std::sync::OnceLock<T>"], &["get_mut"], m),
            SyncOp::OnceSet => (&["std::sync::OnceLock<T>"], &["set"], m),
            SyncOp::OnceGetOrInit => (&["std::sync::OnceLock<T>"], &["get_or_init"], m),
            SyncOp::OnceIntoInner => (&["std::sync::OnceLock<T>"], &["into_inner"], m),
            SyncOp::OnceTake => (&["std::sync::OnceLock<T>"], &["take"], m),
            SyncOp::DurNew => (&["std::time::Duration"], &["new"], a),
            SyncOp::DurFrom(u) => (&["std::time::Duration"], [&["from_secs"], &["from_millis"], &["from_micros"], &["from_nanos"]][u as usize & 3], a),
            SyncOp::DurAs(u) => (&["std::time::Duration"], [&["as_secs"], &["as_millis"], &["as_micros"], &["as_nanos"]][u as usize & 3], m),
            SyncOp::DurSubsec(u) => (&["std::time::Duration"], [&["subsec_millis"], &["subsec_millis"], &["subsec_micros"], &["subsec_nanos"]][u as usize & 3], m),
            SyncOp::DurIsZero => (&["std::time::Duration"], &["is_zero"], m),
            SyncOp::ThreadSleep => (&["std::thread"], &["sleep"], f),
            SyncOp::TryLock(false) => (&["std::sync::Mutex<T>", "std::sync::RwLock<T>"], &["try_lock", "try_read"], m),
            SyncOp::TryLock(true) => (&["std::sync::RwLock<T>"], &["try_write"], m),
            SyncOp::RecvMakeIter(u) => (&["std::sync::mpsc::Receiver<T>"], [&["iter"], &["try_iter"], &["into_iter"], &["iter"]][u as usize & 3], m),
            SyncOp::RecvNext(_) => (&[], &[], m),
        }
    }
}

/// A std trait a builtin answers by structure (the collect glue walks).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncGlue {
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
}

/// Whether a sync type has the std impl of `g`: Ok(answer), or
/// Err(T) when the impl holds exactly when `T` has it (`Mutex<T>:
/// Debug` needs `T: Debug`). None when `t` is not a sync type the table
/// knows (`Arc` and `sync::Weak` answer through the smart-pointer and
/// weak paths of the walks, like `Rc` and `rc::Weak`).
pub fn sync_glue(tys: &impl TyIntern, t: Ty, g: SyncGlue) -> Option<Result<bool, Ty>> {
    use SyncGlue as G;
    let k = crate::ty::sync::sync_kind(tys, t)?;
    let elem = tys.adt_first_ty_arg(t);
    Some(match (k, g) {
        (SyncTy::Arc | SyncTy::ArcWeak, _) => return None,
        // `Debug` for `T: ?Sized + Debug`, `Default` for `T: Default`.
        (SyncTy::Mutex | SyncTy::RwLock, G::Debug | G::Default) => Err(elem?),
        // A guard is `Debug` (and `Display`) through `T`.
        (SyncTy::MutexGuard | SyncTy::RwLockReadGuard | SyncTy::RwLockWriteGuard, G::Debug) => Err(elem?),
        // Every atomic is `Debug` and `Default`.
        (_, G::Debug | G::Default) if k.atomic_value().is_some() => Ok(true),
        // `PoisonError`, `JoinHandle`, `Thread`, `ThreadId`, and the join
        // error are `Debug` for every `T`.
        (SyncTy::PoisonError | SyncTy::JoinHandle | SyncTy::ScopedJoinHandle | SyncTy::Thread | SyncTy::ThreadId | SyncTy::JoinError, G::Debug) => Ok(true),
        // The channel ends and `SendError` are Debug for every `T`;
        // the senders are Clone.
        (SyncTy::Sender | SyncTy::SyncSender | SyncTy::Receiver | SyncTy::SendError, G::Debug) => Ok(true),
        (SyncTy::Sender | SyncTy::SyncSender, G::Clone) => Ok(true),
        // `Condvar`, `Barrier`, and `BarrierWaitResult` are Debug;
        // `Condvar` and `OnceLock<T>` are Default; `OnceLock<T>` is Debug
        // for `T: Debug`.
        (SyncTy::Condvar | SyncTy::Barrier | SyncTy::BarrierWaitResult, G::Debug) => Ok(true),
        (SyncTy::Condvar | SyncTy::OnceLock, G::Default) => Ok(true),
        // `Duration` is Copy, Eq, and Default (its Debug, Ord, and
        // arithmetic are not in lust yet: they reject).
        (SyncTy::Duration, G::Default | G::Clone | G::Copy | G::PartialEq) => Ok(true),
        (SyncTy::OnceLock, G::Debug) => Err(elem?),
        // `thread::Builder` is Debug (TRIAGE T5).
        (SyncTy::ThreadBuilder, G::Debug) => Ok(true),
        // `Thread` is `Clone`; `ThreadId` is `Copy` and `Eq`.
        (SyncTy::Thread, G::Clone) => Ok(true),
        (SyncTy::ThreadId, G::Clone | G::Copy | G::PartialEq) => Ok(true),
        _ => Ok(false),
    })
}

/// The sync type a written type path names: its full std path
/// (`std::sync::atomic::AtomicUsize`). A bare name never reaches check:
/// the resolver rewrites an imported one (a `use`, a fn-body glob, or a
/// module glob) to its full path, and one with no import is L0102.
pub fn sync_type_named(interner: &Interner, segs: &[ast::PathSeg]) -> Option<SyncTy> {
    if segs.len() < 2 {
        return None;
    }
    let path: Vec<&str> = segs.iter().map(|s| interner.get(s.name)).collect();
    SyncTy::by_path(&path)
}

/// Rejects a type argument lust cannot give a sync type yet: `str` or
/// `[T]` in a `Mutex`. `Arc<str>`, `Arc<[T]>`, `Arc<dyn Trait>`,
/// `Mutex<dyn Trait>` (check/own.rs), and `sync::Weak` of `dyn Trait`,
/// `str` and `[T]` (cgen/own.rs) are fat forms.
pub fn check_sync_arg(tys: &impl TyIntern, k: SyncTy, e: Ty, span: Span) -> Result<(), Diag> {
    if matches!(k, SyncTy::Mutex | SyncTy::MutexGuard) && matches!(tys.kind(e), TyKind::Slice(_) | TyKind::Str) {
        return Err(crate::diag::not_yet("L0641", span, "a Mutex of an unsized type"));
    }
    let unsized_ok = matches!(k, SyncTy::Arc | SyncTy::ArcWeak) || (k == SyncTy::Mutex && matches!(tys.kind(e), TyKind::Dyn(..)));
    if !unsized_ok && matches!(k, SyncTy::Arc | SyncTy::ArcWeak) && matches!(tys.kind(e), TyKind::Slice(_) | TyKind::Str | TyKind::Dyn(..)) {
        return Err(crate::diag::not_yet("L0641", span, &format!("an {} of an unsized type", k.name())));
    }
    Ok(())
}

/// The free fn a std path names, if it is one of the sync free fns.
pub fn sync_free_fn(path: &[&str]) -> Option<SyncOp> {
    match path {
        ["std" | "core", "sync", "atomic", "fence"] => Some(SyncOp::Fence),
        ["std" | "core", "sync", "atomic", "compiler_fence"] => Some(SyncOp::CompilerFence),
        ["std" | "core", "hint", "spin_loop"] => Some(SyncOp::SpinLoop),
        ["std", "thread", "spawn"] => Some(SyncOp::ThreadSpawn),
        ["std", "panic", "catch_unwind"] => Some(SyncOp::CatchUnwind),
        ["std", "panic", "resume_unwind"] => Some(SyncOp::ResumeUnwind),
        ["std", "panic", "panic_any"] => Some(SyncOp::PanicAny),
        ["std", "thread", "current"] => Some(SyncOp::ThreadCurrent),
        ["std", "thread", "yield_now"] => Some(SyncOp::ThreadYieldNow),
        ["std", "thread", "panicking"] => Some(SyncOp::ThreadPanicking),
        ["std", "thread", "scope"] => Some(SyncOp::ThreadScope),
        ["std", "thread", "sleep"] => Some(SyncOp::ThreadSleep),
        ["std", "thread", "available_parallelism"] => Some(SyncOp::AvailPar),
        ["std", "sync", "mpsc", "channel"] => Some(SyncOp::Channel),
        ["std", "sync", "mpsc", "sync_channel"] => Some(SyncOp::SyncChannel),
        _ => None,
    }
}

impl<'p> Ctx<'p> {
    /// The associated fns of the sync types (`AtomicUsize::new`). None
    /// when the path's type is none of these (the caller goes on); a
    /// known type with an item lust lacks rejects here. A user type of
    /// the same name wins, except on a std path (`std_ty`).
    pub(super) fn sync_assoc(&mut self, names: &[Sym], std_ty: bool, span: Span) -> Option<Option<Res>> {
        let (&item, ty_path) = names.split_last()?;
        if !std_ty {
            return None;
        }
        let path: Vec<&str> = ty_path.iter().map(|n| self.name(*n)).collect();
        let k = SyncTy::by_path(&path)?;
        let i = self.name(item).to_string();
        let op = match (k, k.atomic_value(), i.as_str()) {
            (_, Some(_), "new") => SyncOp::AtomicNew(k),
            (_, Some(_), "from") => SyncOp::AtomicFrom(k),
            (_, Some(_), "default") => SyncOp::AtomicDefault(k),
            (SyncTy::Arc | SyncTy::ArcWeak, _, "clone") => return Some(Some(Res::Builtin(Builtin::Clone))),
            (SyncTy::Arc, _, "new") => SyncOp::ArcNew,
            (SyncTy::Arc, _, "new_cyclic") => SyncOp::ArcNewCyclic,
            // `Arc::default()`: the default glue, as for `Rc` and `Box`.
            (SyncTy::Arc, _, "default") => return Some(Some(Res::Builtin(Builtin::DefaultGlue))),
            (SyncTy::Arc, _, "from") => return Some(Some(Res::Builtin(Builtin::OwnFrom(crate::check::own::Owner::Arc)))),
            (SyncTy::Arc, _, "strong_count") => SyncOp::ArcStrongCount,
            (SyncTy::Arc, _, "weak_count") => SyncOp::ArcWeakCount,
            (SyncTy::Arc, _, "ptr_eq") => SyncOp::ArcPtrEq,
            (SyncTy::Arc, _, "as_ptr") => SyncOp::ArcAsPtr,
            (SyncTy::Arc, _, "downgrade") => SyncOp::ArcDowngrade,
            (SyncTy::Arc, _, "get_mut") => SyncOp::ArcGetMut,
            (SyncTy::Arc, _, "make_mut") => SyncOp::ArcMakeMut,
            (SyncTy::Arc, _, "try_unwrap") => SyncOp::ArcTryUnwrap,
            (SyncTy::Arc, _, "into_inner") => SyncOp::ArcIntoInner,
            (SyncTy::Arc, _, "unwrap_or_clone") => SyncOp::ArcUnwrapOrClone,
            (SyncTy::ArcWeak, _, "new") => SyncOp::ArcWeakNew,
            // `sync::Weak`'s methods called by path (`Weak::upgrade(&w)`).
            (SyncTy::ArcWeak, _, "upgrade") => {
                // By path and as a value (`.and_then(Weak::upgrade)`,
                // TRIAGE T5): a PrimAssoc over the SyncOp.
                let elem = self.new_var(false);
                let self_ty = sync_ty(&mut self.tys, SyncTy::ArcWeak, &[elem]);
                return Some(Some(Res::PrimAssoc { b: Builtin::Sync(SyncOp::ArcWeakUpgrade), self_ty, name: item }));
            }
            // `Receiver::recv`/`try_recv`/`recv_timeout` and
            // `JoinHandle::join` by path and as values
            // (`.map(JoinHandle::join)`, tf-p6 R05/R09): a PrimAssoc
            // over the method, as `Weak::upgrade`.
            (SyncTy::Receiver, _, "recv" | "try_recv" | "recv_timeout") | (SyncTy::JoinHandle, _, "join") => {
                let op = match i.as_str() {
                    "recv" => SyncOp::ChanRecv,
                    "try_recv" => SyncOp::ChanTryRecv,
                    "recv_timeout" => SyncOp::ChanRecvTimeout,
                    _ => SyncOp::JoinHandleJoin,
                };
                let elem = self.new_var(false);
                let self_ty = sync_ty(&mut self.tys, k, &[elem]);
                return Some(Some(Res::PrimAssoc { b: Builtin::Sync(op), self_ty, name: item }));
            }
            (SyncTy::ArcWeak, _, "strong_count") => SyncOp::ArcWeakStrongCount,
            (SyncTy::ArcWeak, _, "weak_count") => SyncOp::ArcWeakWeakCount,
            (SyncTy::ArcWeak, _, "ptr_eq") => SyncOp::ArcWeakPtrEq,
            (SyncTy::Mutex | SyncTy::RwLock, _, "new") => SyncOp::LockNew(k),
            (SyncTy::Mutex | SyncTy::RwLock, _, "from") => SyncOp::LockFrom(k),
            (SyncTy::Mutex | SyncTy::RwLock | SyncTy::Condvar | SyncTy::OnceLock, _, "default") => SyncOp::LockDefault(k),
            (SyncTy::Condvar | SyncTy::Barrier | SyncTy::OnceLock, _, "new") => SyncOp::SyncNew(k),
            (SyncTy::OnceLock, _, "from") => SyncOp::LockFrom(k),
            (SyncTy::Duration, _, "new") => SyncOp::DurNew,
            (SyncTy::Duration, _, "default") => SyncOp::LockDefault(k),
            (SyncTy::Duration, _, "from_secs") => SyncOp::DurFrom(0),
            (SyncTy::Duration, _, "from_millis") => SyncOp::DurFrom(1),
            (SyncTy::Duration, _, "from_micros") => SyncOp::DurFrom(2),
            (SyncTy::Duration, _, "from_nanos") => SyncOp::DurFrom(3),
            (SyncTy::PoisonError, _, "new") => SyncOp::PoisonNew,
            (SyncTy::ThreadBuilder, _, "new") => SyncOp::BuilderNew,
            // `PoisonError::into_inner` and kin by path, as a
            // value too (`unwrap_or_else(PoisonError::into_inner)`,
            // TRIAGE T5): a PrimAssoc over the method's SyncOp.
            (SyncTy::PoisonError, _, "into_inner" | "get_ref" | "get_mut") => {
                let op = match i.as_str() {
                    "into_inner" => SyncOp::PoisonIntoInner,
                    "get_ref" => SyncOp::PoisonGetRef,
                    _ => SyncOp::PoisonGetMut,
                };
                let elem = self.new_var(false);
                let self_ty = sync_ty(&mut self.tys, SyncTy::PoisonError, &[elem]);
                return Some(Some(Res::PrimAssoc { b: Builtin::Sync(op), self_ty, name: item }));
            }
            _ => {
                self.err(
                    "L0640",
                    span,
                    format!("`{}::{i}` is not implemented in lust yet", k.name()),
                    "Lust supports `new`, `from`, and `default` on the atomics and locks, and `Arc`'s and `Weak`'s counting fns; call lock methods with method syntax.",
                );
                return Some(None);
            }
        };
        Some(Some(Res::Builtin(Builtin::Sync(op))))
    }

    /// The receiver form, other parameters, and return of a sync method
    /// a path names (`PoisonError::into_inner`, `Weak::upgrade`,
    /// `Receiver::recv_timeout`, `JoinHandle::join`): the method table's
    /// signatures for `prim_assoc_sig`'s by-path calls and values.
    pub(super) fn sync_path_method_sig(&mut self, k: SyncTy, self_ty: Ty, name: Sym) -> Option<(SyncOp, Autoref, Vec<Ty>, Ty)> {
        let elem = self.tys.adt_first_ty_arg(self_ty).unwrap_or(ERROR);
        let n = self.name(name).to_string();
        Some(match (k, n.as_str()) {
            (SyncTy::PoisonError, "into_inner") => (SyncOp::PoisonIntoInner, Autoref::None, vec![], elem),
            (SyncTy::PoisonError, "get_ref") => (SyncOp::PoisonGetRef, Autoref::Ref, vec![], self.tys.mk_ref(false, elem)),
            (SyncTy::PoisonError, "get_mut") => (SyncOp::PoisonGetMut, Autoref::RefMut, vec![], self.tys.mk_ref(true, elem)),
            (SyncTy::ArcWeak, "upgrade") => {
                let arc = sync_ty(&mut self.tys, SyncTy::Arc, &[elem]);
                (SyncOp::ArcWeakUpgrade, Autoref::Ref, vec![], self.tys.mk_option(arc))
            }
            (SyncTy::Receiver, "recv" | "try_recv" | "recv_timeout") => {
                let (op, def) = match n.as_str() {
                    "recv" => (SyncOp::ChanRecv, generics::recv_error_def()),
                    "try_recv" => (SyncOp::ChanTryRecv, generics::try_recv_error_def()),
                    _ => (SyncOp::ChanRecvTimeout, generics::recv_timeout_error_def()),
                };
                let er = self.tys.intern(TyKind::Adt(def, EMPTY_ARGS));
                let rest = if op == SyncOp::ChanRecvTimeout { vec![self.duration_ty()] } else { vec![] };
                (op, Autoref::Ref, rest, self.tys.mk_result(elem, er))
            }
            (SyncTy::JoinHandle, "join") => {
                let err = self.box_any_send();
                (SyncOp::JoinHandleJoin, Autoref::None, vec![], self.tys.mk_result(elem, err))
            }
            _ => return None,
        })
    }

    /// The `Duration` type a sync call takes: the std source
    /// `core::time::Duration` Adt when spliced, else the builtin.
    fn duration_ty(&mut self) -> Ty {
        let p = self.p;
        crate::check::spec::read_all(crate::check::spec::tab::ADTS);
        let src = (0..p.adts.len() as u32)
            .map(AdtId)
            .find(|&a| p.is_std_adt(a) && p.display_path(p.adt(a).name) == "core::time::Duration");
        match src {
            Some(a) => self.tys.adt_ty(a),
            None => sync_ty(&mut self.tys, SyncTy::Duration, &[]),
        }
    }

    /// The platform `std::io::Error` type, when the platform
    /// crate is in the session (its mention triggers load it).
    fn io_error_ty(&mut self) -> Ty {
        let p = self.p;
        crate::check::spec::read_all(crate::check::spec::tab::ADTS);
        (0..p.adts.len() as u32)
            .map(AdtId)
            .find(|&a| p.shipped_crate_at(p.adt(a).span) == Some(crate::ty::lang::ShippedCrate::Std) && p.display_path(p.adt(a).name) == "std::io::Error")
            .map(|a| self.tys.adt_ty(a))
            .unwrap_or(ERROR)
    }

    /// Parameter and return types of a `Builtin::Sync` associated or free
    /// fn call.
    pub(super) fn sync_call_sig(&mut self, op: SyncOp, callee: ExprId, span: Span) -> (Vec<Ty>, Ty) {
        let ord = crate::check::generics::atomic_ordering_ty(&mut self.tys);
        // `Arc<T>` and `sync::Weak<T>` ops: the element type comes from
        // the written Self type (`Arc::<u8>::new`, an alias;
        // `unify_builtin_self` in `call`), else inference.
        let elem = self.new_var(false);
        let arc = sync_ty(&mut self.tys, SyncTy::Arc, &[elem]);
        let weak = sync_ty(&mut self.tys, SyncTy::ArcWeak, &[elem]);
        let arc_ref = self.tys.mk_ref(false, arc);
        let arc_mut = self.tys.mk_ref(true, arc);
        let weak_ref = self.tys.mk_ref(false, weak);
        match op {
            SyncOp::ArcNew => (vec![elem], arc),
            // `Arc::new_cyclic<F: FnOnce(&Weak<T>) -> T>(data_fn)`.
            SyncOp::ArcNewCyclic => {
                let wref = self.tys.mk_ref(false, weak);
                let f = self.fn_once_param(&[wref], elem, span);
                (vec![f], arc)
            }
            SyncOp::ArcStrongCount | SyncOp::ArcWeakCount => (vec![arc_ref], USIZE),
            SyncOp::ArcPtrEq => (vec![arc_ref, arc_ref], BOOL),
            SyncOp::ArcAsPtr => {
                self.deferred_raw.push((elem, span, super::RawUse::Pointee));
                (vec![arc_ref], self.tys.intern(TyKind::RawPtr(crate::ty::Mut::Shared, elem)))
            }
            SyncOp::ArcDowngrade => (vec![arc_ref], weak),
            SyncOp::ArcGetMut => {
                let r = self.tys.mk_ref(true, elem);
                (vec![arc_mut], self.tys.mk_option(r))
            }
            SyncOp::ArcMakeMut => {
                self.deferred_make_mut.push((elem, span, true));
                (vec![arc_mut], self.tys.mk_ref(true, elem))
            }
            SyncOp::ArcTryUnwrap => (vec![arc], self.tys.mk_result(elem, arc)),
            SyncOp::ArcIntoInner => (vec![arc], self.tys.mk_option(elem)),
            SyncOp::ArcUnwrapOrClone => {
                self.deferred_clone.push((elem, span));
                (vec![arc], elem)
            }
            // std's `impl<T> Weak<T>`: `new` needs a sized `T`.
            SyncOp::ArcWeakNew => {
                self.deferred_sized.push((elem, span));
                (vec![], weak)
            }
            SyncOp::ArcWeakUpgrade => (vec![weak_ref], self.tys.mk_option(arc)),
            SyncOp::ArcWeakStrongCount | SyncOp::ArcWeakWeakCount => (vec![weak_ref], USIZE),
            SyncOp::ArcWeakPtrEq => (vec![weak_ref, weak_ref], BOOL),
            SyncOp::AtomicNew(k) | SyncOp::AtomicFrom(k) => {
                let v = k.atomic_value().expect("an atomic").ty();
                (vec![v], sync_ty(&mut self.tys, k, &[]))
            }
            SyncOp::AtomicDefault(k) => (vec![], sync_ty(&mut self.tys, k, &[])),
            SyncOp::LockNew(k) | SyncOp::LockFrom(k) => (vec![elem], sync_ty(&mut self.tys, k, &[elem])),
            SyncOp::LockDefault(k) => {
                if k.is_lock() {
                    self.deferred_default.push((elem, span));
                }
                let args: &[Ty] = if k.arity() == 0 { &[] } else { &[elem] };
                (vec![], sync_ty(&mut self.tys, k, args))
            }
            SyncOp::SyncNew(SyncTy::Barrier) => (vec![USIZE], sync_ty(&mut self.tys, SyncTy::Barrier, &[])),
            SyncOp::DurNew => (vec![int(IntTy::U64), int(IntTy::U32)], sync_ty(&mut self.tys, SyncTy::Duration, &[])),
            SyncOp::DurFrom(_) => (vec![int(IntTy::U64)], sync_ty(&mut self.tys, SyncTy::Duration, &[])),
            // With the std source `core::time::Duration` spliced (W-STD-C),
            // `std::time::Duration` names it, so `sleep` takes it.
            SyncOp::ThreadSleep => {
                let p = self.p;
                crate::check::spec::read_all(crate::check::spec::tab::ADTS);
                let src = (0..p.adts.len() as u32)
                    .map(AdtId)
                    .find(|&a| p.is_std_adt(a) && p.display_path(p.adt(a).name) == "core::time::Duration");
                let d = match src {
                    Some(a) => self.tys.adt_ty(a),
                    None => sync_ty(&mut self.tys, SyncTy::Duration, &[]),
                };
                (vec![d], UNIT)
            }
            SyncOp::SyncNew(k) => {
                let args: &[Ty] = if k.arity() == 0 { &[] } else { &[elem] };
                (vec![], sync_ty(&mut self.tys, k, args))
            }
            SyncOp::PoisonNew => (vec![elem], sync_ty(&mut self.tys, SyncTy::PoisonError, &[elem])),
            SyncOp::BuilderNew => (vec![], sync_ty(&mut self.tys, SyncTy::ThreadBuilder, &[])),
            SyncOp::AvailPar => {
                let nz = generics::nonzero_ty(&mut self.tys, int(IntTy::Usize));
                let err = self.io_error_ty();
                (vec![], self.tys.mk_result(nz, err))
            }
            // `spawn<F, T>(f: F) -> JoinHandle<T>` where `F: FnOnce() -> T
            // + Send + 'static, T: Send + 'static` (a generic parameter
            // needs the bound here; the borrow checker proves concrete
            // borrows, borrowck/regions.rs).
            SyncOp::ThreadSpawn => {
                let ret = self.new_var(false);
                let f = self.fn_once_param(&[], ret, span);
                self.require_send(f, span);
                self.require_send(ret, span);
                self.require_spawn_static(f, span);
                self.require_static(ret, span);
                (vec![f], sync_ty(&mut self.tys, SyncTy::JoinHandle, &[ret]))
            }
            SyncOp::ThreadCurrent => (vec![], sync_ty(&mut self.tys, SyncTy::Thread, &[])),
            SyncOp::CatchUnwind
            | SyncOp::ResumeUnwind
            | SyncOp::PanicAny
            | SyncOp::PanicHookSwap
            | SyncOp::PanicHookRef
            | SyncOp::PanicHookPayload
            | SyncOp::PanicHookLocation => {
                self.unwind_call_sig(op, span)
            }
            // `scope<'env, F, T>(f: F) -> T` where `F: for<'scope>
            // FnOnce(&'scope Scope<'scope, 'env>) -> T`: the closure's
            // `&Scope` is higher-ranked.
            SyncOp::ThreadScope => {
                let ret = self.new_var(false);
                let sc = sync_ty(&mut self.tys, SyncTy::Scope, &[]);
                let sr = self.tys.mk_ref(false, sc);
                let f = self.fn_once_param(&[sr], ret, span);
                if let TyKind::Infer(v) = self.kind(f) {
                    self.scope_closure_vars.push(v.0);
                }
                (vec![f], ret)
            }
            SyncOp::ThreadYieldNow => (vec![], UNIT),
            // `channel<T>() -> (Sender<T>, Receiver<T>)`; the turbofish
            // is on the fn (`channel::<u8>()`).
            SyncOp::Channel | SyncOp::SyncChannel => {
                let t = match self.p.ast.expr(callee) {
                    Expr::Path(segs) => match segs.last().map(|s| s.type_args()).unwrap_or_default().first() {
                        Some(&g) => self.resolve_written(g),
                        None => self.new_var(false),
                    },
                    _ => self.new_var(false),
                };
                let tx = sync_ty(&mut self.tys, if op == SyncOp::Channel { SyncTy::Sender } else { SyncTy::SyncSender }, &[t]);
                let rx = sync_ty(&mut self.tys, SyncTy::Receiver, &[t]);
                let ts = self.tys.intern_tys(&[tx, rx]);
                let pair = self.tys.intern(TyKind::Tuple(ts));
                (if op == SyncOp::Channel { vec![] } else { vec![USIZE] }, pair)
            }
            SyncOp::ThreadPanicking => (vec![], BOOL),
            SyncOp::Fence | SyncOp::CompilerFence => (vec![ord], UNIT),
            SyncOp::SpinLoop => (vec![], UNIT),
            _ => unreachable!("sync methods type in sync_method"),
        }
    }

    /// Methods on the sync types (after autoderef). None when the
    /// receiver is none of these.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn sync_method(&mut self, e: ExprId, recv: ExprId, base: Ty, derefs: u8, name: Sym, span: Span, args: &[ExprId]) -> Option<Ty> {
        if let Some(el) = self.tys.arc_weak_elem(base) {
            let w_ref = self.tys.mk_ref(false, base);
            let (op, params, ret) = match self.name(name) {
                "upgrade" => {
                    let arc = sync_ty(&mut self.tys, SyncTy::Arc, &[el]);
                    (SyncOp::ArcWeakUpgrade, vec![], self.tys.mk_option(arc))
                }
                "strong_count" => (SyncOp::ArcWeakStrongCount, vec![], USIZE),
                "weak_count" => (SyncOp::ArcWeakWeakCount, vec![], USIZE),
                "ptr_eq" => (SyncOp::ArcWeakPtrEq, vec![w_ref], BOOL),
                "as_ptr" | "into_raw" => {
                    let n = self.name(name).to_string();
                    self.err("L0640", span, format!("`Weak::{n}` is not implemented in lust yet"), "Use `upgrade` and the counts.");
                    return Some(self.iter_bail(args));
                }
                _ => return None,
            };
            self.check_packed_borrow(recv, span);
            self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(op), derefs, autoref: Autoref::Ref });
            self.check_args(&params, args, span);
            return Some(ret);
        }
        if let Some(ret) = self.lock_method(e, recv, base, derefs, name, span, args) {
            return Some(ret);
        }
        let (k, v) = atomic_of(&self.tys, base)?;
        let n = self.name(name).to_string();
        let vt = v.ty();
        let ord = crate::check::generics::atomic_ordering_ty(&mut self.tys);
        let is_int = matches!(v, AtomicVal::Int(_));
        let fetch = |f| (SyncOp::AtomicFetch(f), Autoref::Ref, vec![vt, ord], vt);
        let (op, autoref, params, ret) = match n.as_str() {
            "load" => (SyncOp::AtomicLoad, Autoref::Ref, vec![ord], vt),
            "store" => (SyncOp::AtomicStore, Autoref::Ref, vec![vt, ord], UNIT),
            "swap" => (SyncOp::AtomicSwap, Autoref::Ref, vec![vt, ord], vt),
            "compare_exchange" | "compare_exchange_weak" => {
                let op = if n == "compare_exchange" { SyncOp::AtomicCompareExchange } else { SyncOp::AtomicCompareExchangeWeak };
                let res = self.tys.mk_result(vt, vt);
                (op, Autoref::Ref, vec![vt, vt, ord, ord], res)
            }
            "fetch_add" if is_int => fetch(FetchOp::Add),
            "fetch_sub" if is_int => fetch(FetchOp::Sub),
            "fetch_max" if is_int => fetch(FetchOp::Max),
            "fetch_min" if is_int => fetch(FetchOp::Min),
            "fetch_and" => fetch(FetchOp::And),
            "fetch_nand" => fetch(FetchOp::Nand),
            "fetch_or" => fetch(FetchOp::Or),
            "fetch_xor" => fetch(FetchOp::Xor),
            "fetch_not" if !is_int => (SyncOp::AtomicFetchNot, Autoref::Ref, vec![ord], vt),
            "fetch_update" => {
                let opt = self.tys.mk_option(vt);
                let f = self.sync_fn_mut_param(&[vt], opt, span);
                let res = self.tys.mk_result(vt, vt);
                (SyncOp::AtomicFetchUpdate, Autoref::Ref, vec![ord, ord, f], res)
            }
            "into_inner" => (SyncOp::AtomicIntoInner, Autoref::None, vec![], vt),
            "get_mut" => {
                let r = self.tys.mk_ref(true, vt);
                (SyncOp::AtomicGetMut, Autoref::RefMut, vec![], r)
            }
            // Stable std methods lust does not have yet.
            "as_ptr" | "compare_and_swap" | "from_ptr" | "update" | "try_update" => {
                self.err("L0640", span, format!("`{}::{n}` is not implemented in lust yet", k.name()), "Use `load`, `store`, or `compare_exchange` in a loop.");
                return Some(self.iter_bail(args));
            }
            _ => return None,
        };
        if autoref != Autoref::None {
            self.check_packed_borrow(recv, span);
        }
        self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(op), derefs, autoref });
        self.check_args(&params, args, span);
        self.lint_atomic_ordering(op, args);
        // cgen calls a closure, a fn item of this program, or a fn pointer
        // there (`Callee`); a builtin item (`Some`) is not one yet.
        if op == SyncOp::AtomicFetchUpdate
            && let Some(&fa) = args.get(2)
        {
            let mut ft = self.deep(self.out.expr_ty[self.ei(fa)]);
            if let TyKind::Ref(_, _, i) = self.kind(ft) {
                ft = self.deep(i);
            }
            let ok = match self.kind(ft) {
                TyKind::Closure(..) | TyKind::FnPtr(_) | TyKind::Error => true,
                TyKind::FnDef(d, _) => self.p.local_def(d.hash).is_some(),
                _ => false,
            };
            if !ok {
                self.errs.push(not_yet("L0641", self.span(fa), "this callable as `fetch_update`'s function (use a closure)"));
            }
        }
        Some(ret)
    }

    /// A fresh closure-argument type with the pending obligations of
    /// `F: FnOnce(inputs..) -> output` (also `LocalKey::with`, check/tls.rs).
    pub(super) fn fn_once_param(&mut self, inputs: &[Ty], output: Ty, span: Span) -> Ty {
        let f = self.new_var(false);
        let lang = self.p.gx.lang;
        let ts = self.tys.intern_tys(inputs);
        let tuple = self.tys.intern(TyKind::Tuple(ts));
        let args = self.tys.intern_args(&[GenericArg::Ty(f), GenericArg::Ty(tuple)]);
        self.obl.push(obligations::Obligation::Trait { trait_ref: TraitRef { def: lang.fn_once, args, bindings: EMPTY_BINDINGS }, span, depth: 0 });
        if let Some(out) = lang.output_of(lang.fn_once) {
            self.obl.push(obligations::Obligation::AssocEq { eq: AssocEqData { assoc: out, trait_args: args, ty: output }, span });
        }
        f
    }

    /// A fresh closure-argument type with the pending obligations of
    /// `F: FnMut(inputs..) -> output`; each input's outer reference is
    /// higher-ranked (`wait_while`'s `&mut T`).
    fn sync_fn_mut_param(&mut self, inputs: &[Ty], output: Ty, span: Span) -> Ty {
        let f = self.new_var(false);
        if let TyKind::Infer(v) = self.kind(f) {
            self.iter_closure_args.insert(v.0, vec![true; inputs.len()]);
        }
        let lang = self.p.gx.lang;
        let ts = self.tys.intern_tys(inputs);
        let tuple = self.tys.intern(TyKind::Tuple(ts));
        let args = self.tys.intern_args(&[GenericArg::Ty(f), GenericArg::Ty(tuple)]);
        self.obl.push(obligations::Obligation::Trait { trait_ref: TraitRef { def: lang.fn_mut, args, bindings: EMPTY_BINDINGS }, span, depth: 0 });
        if let Some(out) = lang.output_of(lang.fn_once) {
            self.obl.push(obligations::Obligation::AssocEq { eq: AssocEqData { assoc: out, trait_args: args, ty: output }, span });
        }
        f
    }

    /// The element of an mpsc `Receiver<T>` or `&Receiver<T>` (a `for`
    /// loop's iterable).
    pub(super) fn recv_elem(&mut self, t: Ty) -> Option<Ty> {
        let t = self.resolve(t);
        let r = match self.kind(t) {
            TyKind::Ref(_, m, inner) if !m.is_mut() => self.resolve(inner),
            _ => t,
        };
        (crate::ty::sync::sync_kind(&self.tys, r) == Some(SyncTy::Receiver)).then(|| self.tys.adt_first_ty_arg(r)).flatten()
    }

    /// The pending obligation `t: 'static` (a generic parameter needs
    /// the bound in the env, rustc E0310; concrete borrows are the
    /// borrow checker's part).
    fn require_static(&mut self, t: Ty, span: Span) {
        self.obl.push(obligations::Obligation::TypeStatic { ty: t, span });
    }

    /// `require_static` for a thread spawn's function (`F: 'static`): the
    /// borrow checker proves an inferred `dyn` object lifetime in it
    /// (`Ctx::spawn_static`).
    fn require_spawn_static(&mut self, t: Ty, span: Span) {
        self.spawn_static.push(span);
        self.require_static(t, span);
    }

    /// The pending obligation `t: Send`.
    pub(super) fn require_send(&mut self, t: Ty, span: Span) {
        let send = self.p.gx.lang.send;
        let args = self.tys.intern_args(&[GenericArg::Ty(t)]);
        self.obl.push(obligations::Obligation::Trait { trait_ref: TraitRef { def: send, args, bindings: EMPTY_BINDINGS }, span, depth: 0 });
    }

    /// `LockResult<G>`: `Result<G, PoisonError<G>>`.
    fn lock_result(&mut self, g: Ty) -> Ty {
        let pe = sync_ty(&mut self.tys, SyncTy::PoisonError, &[g]);
        self.tys.mk_result(g, pe)
    }

    /// Methods on `Mutex<T>`, `RwLock<T>`, and `PoisonError<T>`. The
    /// guards have none of their own: method calls deref through them.
    #[allow(clippy::too_many_arguments)]
    fn lock_method(&mut self, e: ExprId, recv: ExprId, base: Ty, derefs: u8, name: Sym, span: Span, args: &[ExprId]) -> Option<Ty> {
        let n = self.name(name).to_string();
        let (op, autoref, ret) = if let Some((k, el)) = self.tys.lock_elem(base) {
            // `Mutex<dyn Tr>` (check/own.rs): `lock` and `is_poisoned`.
            if matches!(self.kind(el), TyKind::Dyn(..)) && !matches!(n.as_str(), "lock" | "is_poisoned") {
                return Some(self.dyn_cell_method_reject(&n, span, args));
            }
            let guard = |this: &mut Self, g: SyncTy| {
                let t = sync_ty(&mut this.tys, g, &[el]);
                this.lock_result(t)
            };
            match (k, n.as_str()) {
                (SyncTy::Mutex, "lock") => (SyncOp::MutexLock, Autoref::Ref, guard(self, SyncTy::MutexGuard)),
                (SyncTy::RwLock, "read") => (SyncOp::RwLockRead, Autoref::Ref, guard(self, SyncTy::RwLockReadGuard)),
                (SyncTy::RwLock, "write") => (SyncOp::RwLockWrite, Autoref::Ref, guard(self, SyncTy::RwLockWriteGuard)),
                (_, "get_mut") => {
                    let r = self.tys.mk_ref(true, el);
                    (SyncOp::LockGetMut, Autoref::RefMut, self.lock_result(r))
                }
                (_, "into_inner") => (SyncOp::LockIntoInner, Autoref::None, self.lock_result(el)),
                (_, "is_poisoned") => (SyncOp::LockIsPoisoned, Autoref::Ref, BOOL),
                (_, "clear_poison") => (SyncOp::LockClearPoison, Autoref::Ref, UNIT),
                // `TryLockResult<G>`: `Result<G, TryLockError<G>>`.
                (SyncTy::Mutex, "try_lock") | (SyncTy::RwLock, "try_read" | "try_write") => {
                    let gk = match n.as_str() {
                        "try_lock" => SyncTy::MutexGuard,
                        "try_read" => SyncTy::RwLockReadGuard,
                        _ => SyncTy::RwLockWriteGuard,
                    };
                    let g = sync_ty(&mut self.tys, gk, &[el]);
                    let te = generics::try_lock_error_ty(&mut self.tys, g);
                    (SyncOp::TryLock(n == "try_write"), Autoref::Ref, self.tys.mk_result(g, te))
                }
                _ => return None,
            }
        } else if let Some(k @ (SyncTy::JoinHandle | SyncTy::ScopedJoinHandle | SyncTy::Thread | SyncTy::Scope | SyncTy::ThreadBuilder)) = crate::ty::sync::sync_kind(&self.tys, base) {
            match (k, n.as_str()) {
                // `spawn<F, T>(&'scope self, f: F) -> ScopedJoinHandle<'scope,
                // T>` where `F: FnOnce() -> T + Send + 'scope, T: Send +
                // 'scope` (the regions in borrowck/regions.rs).
                // `Builder::name(self, name) -> Builder` and
                // `stack_size(self, size) -> Builder`: by value.
                (SyncTy::ThreadBuilder, "name") => {
                    self.check_packed_borrow(recv, span);
                    self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(SyncOp::BuilderName), derefs, autoref: Autoref::None });
                    self.check_args(&[STRING], args, span);
                    return Some(sync_ty(&mut self.tys, SyncTy::ThreadBuilder, &[]));
                }
                (SyncTy::ThreadBuilder, "stack_size") => {
                    self.check_packed_borrow(recv, span);
                    self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(SyncOp::BuilderStackSize), derefs, autoref: Autoref::None });
                    self.check_args(&[USIZE], args, span);
                    return Some(sync_ty(&mut self.tys, SyncTy::ThreadBuilder, &[]));
                }
                // `Builder::spawn_scoped<'scope, 'env, F, T>(self, scope:
                // &'scope Scope<'scope, 'env>, f: F) ->
                // io::Result<ScopedJoinHandle<'scope, T>>`, `F: FnOnce() -> T
                // + Send + 'scope, T: Send + 'scope` (the borrow checker's
                // scope rule, as for `scope.spawn`).
                (SyncTy::ThreadBuilder, "spawn_scoped") => {
                    let ret = self.new_var(false);
                    let f = self.fn_once_param(&[], ret, span);
                    self.require_send(f, span);
                    self.require_send(ret, span);
                    let sc = sync_ty(&mut self.tys, SyncTy::Scope, &[]);
                    let sr = self.tys.mk_ref(false, sc);
                    let h = sync_ty(&mut self.tys, SyncTy::ScopedJoinHandle, &[ret]);
                    let err = self.io_error_ty();
                    self.check_packed_borrow(recv, span);
                    self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(SyncOp::BuilderSpawnScoped), derefs, autoref: Autoref::None });
                    self.check_args(&[sr, f], args, span);
                    return Some(self.tys.mk_result(h, err));
                }
                // `Builder::spawn<F, T>(self, f) -> io::Result<JoinHandle<T>>`.
                (SyncTy::ThreadBuilder, "spawn") => {
                    let ret = self.new_var(false);
                    let f = self.fn_once_param(&[], ret, span);
                    self.require_send(f, span);
                    self.require_send(ret, span);
                    self.require_spawn_static(f, span);
                    self.require_static(ret, span);
                    let h = sync_ty(&mut self.tys, SyncTy::JoinHandle, &[ret]);
                    let err = self.io_error_ty();
                    self.check_packed_borrow(recv, span);
                    self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(SyncOp::BuilderSpawn), derefs, autoref: Autoref::None });
                    self.check_args(&[f], args, span);
                    return Some(self.tys.mk_result(h, err));
                }
                (SyncTy::Scope, "spawn") => {
                    let ret = self.new_var(false);
                    let f = self.fn_once_param(&[], ret, span);
                    self.require_send(f, span);
                    self.require_send(ret, span);
                    let h = sync_ty(&mut self.tys, SyncTy::ScopedJoinHandle, &[ret]);
                    self.check_packed_borrow(recv, span);
                    self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(SyncOp::ScopeSpawn), derefs, autoref: Autoref::Ref });
                    self.check_args(&[f], args, span);
                    return Some(h);
                }
                (SyncTy::JoinHandle | SyncTy::ScopedJoinHandle, "join") => {
                    let el = self.tys.adt_first_ty_arg(base).unwrap_or(ERROR);
                    // `thread::Result<T>`: a panicked thread's payload.
                    let err = self.box_any_send();
                    (SyncOp::JoinHandleJoin, Autoref::None, self.tys.mk_result(el, err))
                }
                (SyncTy::JoinHandle | SyncTy::ScopedJoinHandle, "thread") => {
                    let th = sync_ty(&mut self.tys, SyncTy::Thread, &[]);
                    (SyncOp::JoinHandleThread, Autoref::Ref, self.tys.mk_ref(false, th))
                }
                (SyncTy::JoinHandle | SyncTy::ScopedJoinHandle, "is_finished") => (SyncOp::JoinHandleIsFinished, Autoref::Ref, BOOL),
                (SyncTy::Thread, "id") => (SyncOp::ThreadGetId, Autoref::Ref, sync_ty(&mut self.tys, SyncTy::ThreadId, &[])),
                (SyncTy::Thread, "name") => {
                    let s = self.tys.mk_ref(false, STR);
                    (SyncOp::ThreadGetName, Autoref::Ref, self.tys.mk_option(s))
                }
                (_, "unpark" | "as_u64" | "into_raw" | "as_ptr") => {
                    self.err("L0640", span, format!("`{}::{n}` is not implemented in lust yet", k.name()), "Use `join`, `id`, or `name`.");
                    return Some(self.iter_bail(args));
                }
                _ => return None,
            }
        } else if let Some(k @ (SyncTy::Sender | SyncTy::SyncSender | SyncTy::Receiver)) = crate::ty::sync::sync_kind(&self.tys, base) {
            let el = self.tys.adt_first_ty_arg(base).unwrap_or(ERROR);
            match (k, n.as_str()) {
                (SyncTy::Sender | SyncTy::SyncSender, "send") => {
                    let se = sync_ty(&mut self.tys, SyncTy::SendError, &[el]);
                    let r = self.tys.mk_result(UNIT, se);
                    self.check_packed_borrow(recv, span);
                    self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(SyncOp::ChanSend), derefs, autoref: Autoref::Ref });
                    self.check_args(&[el], args, span);
                    return Some(r);
                }
                (SyncTy::Receiver, "recv") => {
                    let er = self.tys.intern(TyKind::Adt(crate::check::generics::recv_error_def(), EMPTY_ARGS));
                    (SyncOp::ChanRecv, Autoref::Ref, self.tys.mk_result(el, er))
                }
                (SyncTy::Receiver, "try_recv") => {
                    let er = self.tys.intern(TyKind::Adt(crate::check::generics::try_recv_error_def(), EMPTY_ARGS));
                    (SyncOp::ChanTryRecv, Autoref::Ref, self.tys.mk_result(el, er))
                }
                // `recv_timeout(&self, timeout: Duration) -> Result<T,
                // RecvTimeoutError>` (tf-p6 R09).
                (SyncTy::Receiver, "recv_timeout") => {
                    let er = self.tys.intern(TyKind::Adt(crate::check::generics::recv_timeout_error_def(), EMPTY_ARGS));
                    let r = self.tys.mk_result(el, er);
                    let d = self.duration_ty();
                    self.check_packed_borrow(recv, span);
                    self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(SyncOp::ChanRecvTimeout), derefs, autoref: Autoref::Ref });
                    self.check_args(&[d], args, span);
                    return Some(r);
                }
                (SyncTy::Receiver, "iter" | "try_iter" | "into_iter") => {
                    let (u, kind, autoref) = match n.as_str() {
                        "iter" => (0, crate::ty::iter::IterKind::RecvIter, Autoref::Ref),
                        "try_iter" => (1, crate::ty::iter::IterKind::RecvTryIter, Autoref::Ref),
                        _ => (2, crate::ty::iter::IterKind::RecvIntoIter, Autoref::None),
                    };
                    (SyncOp::RecvMakeIter(u), autoref, crate::ty::iter::mk_iter(&mut self.tys, kind, &[el]))
                }
                (SyncTy::SyncSender, "try_send") => {
                    let te = generics::try_send_error_ty(&mut self.tys, el);
                    let r = self.tys.mk_result(UNIT, te);
                    self.check_packed_borrow(recv, span);
                    self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(SyncOp::ChanTrySend), derefs, autoref: Autoref::Ref });
                    self.check_args(&[el], args, span);
                    return Some(r);
                }
                (_, "send_timeout" | "recv_deadline") => {
                    self.err("L0640", span, format!("`{}::{n}` is not implemented in lust yet", k.name()), "Use `send`, `recv`, or `try_recv`.");
                    return Some(self.iter_bail(args));
                }
                _ => return None,
            }
        } else if let Some(k @ (SyncTy::Condvar | SyncTy::Barrier | SyncTy::BarrierWaitResult | SyncTy::OnceLock)) = crate::ty::sync::sync_kind(&self.tys, base) {
            let el = self.tys.adt_first_ty_arg(base).unwrap_or(ERROR);
            let (op, autoref, params, ret) = match (k, n.as_str()) {
                // `wait<'a, T>(&self, guard: MutexGuard<'a, T>) ->
                // LockResult<MutexGuard<'a, T>>`.
                (SyncTy::Condvar, "wait" | "wait_while") => {
                    let t = self.new_var(false);
                    let g = sync_ty(&mut self.tys, SyncTy::MutexGuard, &[t]);
                    let ret = self.lock_result(g);
                    if n == "wait" {
                        (SyncOp::CondvarWait, Autoref::Ref, vec![g], ret)
                    } else {
                        let tm = self.tys.mk_ref(true, t);
                        let f = self.sync_fn_mut_param(&[tm], BOOL, span);
                        (SyncOp::CondvarWaitWhile, Autoref::Ref, vec![g, f], ret)
                    }
                }
                // `wait_timeout_while(guard, dur, condition) ->
                // LockResult<(MutexGuard<T>, WaitTimeoutResult)>`.
                (SyncTy::Condvar, "wait_timeout_while") => {
                    let t = self.new_var(false);
                    let g = sync_ty(&mut self.tys, SyncTy::MutexGuard, &[t]);
                    let wtr = generics::wait_timeout_result_ty(&mut self.tys);
                    let ts = self.tys.intern_tys(&[g, wtr]);
                    let pair = self.tys.intern(TyKind::Tuple(ts));
                    let ret = self.lock_result(pair);
                    let d = self.duration_ty();
                    let tm = self.tys.mk_ref(true, t);
                    let f = self.sync_fn_mut_param(&[tm], BOOL, span);
                    (SyncOp::CondvarWaitTimeoutWhile, Autoref::Ref, vec![g, d, f], ret)
                }
                // `wait_timeout(guard, dur) -> LockResult<(MutexGuard<T>,
                // WaitTimeoutResult)>` (TRIAGE T5); the ms form takes u64.
                (SyncTy::Condvar, "wait_timeout" | "wait_timeout_ms") => {
                    let t = self.new_var(false);
                    let g = sync_ty(&mut self.tys, SyncTy::MutexGuard, &[t]);
                    let pair = {
                        let wtr = generics::wait_timeout_result_ty(&mut self.tys);
                        let ts = self.tys.intern_tys(&[g, wtr]);
                        self.tys.intern(TyKind::Tuple(ts))
                    };
                    if n == "wait_timeout" {
                        let ret = self.lock_result(pair);
                        let d = self.duration_ty();
                        (SyncOp::CondvarWaitTimeout, Autoref::Ref, vec![g, d], ret)
                    } else {
                        // The deprecated ms form pairs the guard
                        // with a plain bool, not a WaitTimeoutResult.
                        let ts = self.tys.intern_tys(&[g, BOOL]);
                        let pair = self.tys.intern(TyKind::Tuple(ts));
                        let ret = self.lock_result(pair);
                        (SyncOp::CondvarWaitTimeoutMs, Autoref::Ref, vec![g, int(IntTy::U64)], ret)
                    }
                }
                (SyncTy::Condvar, "notify_one") => (SyncOp::CondvarNotifyOne, Autoref::Ref, vec![], UNIT),
                (SyncTy::Condvar, "notify_all") => (SyncOp::CondvarNotifyAll, Autoref::Ref, vec![], UNIT),
                (SyncTy::Barrier, "wait") => (SyncOp::BarrierWait, Autoref::Ref, vec![], sync_ty(&mut self.tys, SyncTy::BarrierWaitResult, &[])),
                (SyncTy::BarrierWaitResult, "is_leader") => (SyncOp::BarrierIsLeader, Autoref::Ref, vec![], BOOL),
                (SyncTy::OnceLock, "get") => {
                    let r = self.tys.mk_ref(false, el);
                    (SyncOp::OnceGet, Autoref::Ref, vec![], self.tys.mk_option(r))
                }
                (SyncTy::OnceLock, "get_mut") => {
                    let r = self.tys.mk_ref(true, el);
                    (SyncOp::OnceGetMut, Autoref::RefMut, vec![], self.tys.mk_option(r))
                }
                (SyncTy::OnceLock, "set") => (SyncOp::OnceSet, Autoref::Ref, vec![el], self.tys.mk_result(UNIT, el)),
                (SyncTy::OnceLock, "get_or_init") => {
                    let f = self.fn_once_param(&[], el, span);
                    (SyncOp::OnceGetOrInit, Autoref::Ref, vec![f], self.tys.mk_ref(false, el))
                }
                (SyncTy::OnceLock, "into_inner") => (SyncOp::OnceIntoInner, Autoref::None, vec![], self.tys.mk_option(el)),
                (SyncTy::OnceLock, "take") => (SyncOp::OnceTake, Autoref::RefMut, vec![], self.tys.mk_option(el)),
                (_, "wait_timeout_while" | "get_mut_or_init" | "get_or_try_init" | "try_insert" | "wait_with" | "get_or_init_with") => {
                    self.err("L0640", span, format!("`{}::{n}` is not implemented in lust yet", k.name()), "Use the methods without timeouts.");
                    return Some(self.iter_bail(args));
                }
                _ => return None,
            };
            if autoref != Autoref::None {
                self.check_packed_borrow(recv, span);
            }
            self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(op), derefs, autoref });
            self.check_args(&params, args, span);
            return Some(ret);
        } else if crate::ty::sync::sync_kind(&self.tys, base) == Some(SyncTy::Duration) {
            let (op, ret) = match n.as_str() {
                "as_secs" => (SyncOp::DurAs(0), int(IntTy::U64)),
                "as_millis" => (SyncOp::DurAs(1), int(IntTy::U128)),
                "as_micros" => (SyncOp::DurAs(2), int(IntTy::U128)),
                "as_nanos" => (SyncOp::DurAs(3), int(IntTy::U128)),
                "subsec_millis" => (SyncOp::DurSubsec(1), int(IntTy::U32)),
                "subsec_micros" => (SyncOp::DurSubsec(2), int(IntTy::U32)),
                "subsec_nanos" => (SyncOp::DurSubsec(3), int(IntTy::U32)),
                "is_zero" => (SyncOp::DurIsZero, BOOL),
                _ => return None,
            };
            self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(op), derefs, autoref: Autoref::Ref });
            self.check_args(&[], args, span);
            return Some(ret);
        } else if let Some(el) = self.tys.poison_elem(base) {
            match n.as_str() {
                "into_inner" => (SyncOp::PoisonIntoInner, Autoref::None, el),
                "get_ref" => (SyncOp::PoisonGetRef, Autoref::Ref, self.tys.mk_ref(false, el)),
                "get_mut" => (SyncOp::PoisonGetMut, Autoref::RefMut, self.tys.mk_ref(true, el)),
                _ => return None,
            }
        } else {
            return None;
        };
        if autoref != Autoref::None {
            self.check_packed_borrow(recv, span);
        }
        self.set_res(e, Res::BuiltinMethod { b: Builtin::Sync(op), derefs, autoref });
        self.check_args(&[], args, span);
        Some(ret)
    }

    /// The `atomic::Ordering` variant an argument names directly
    /// (`Ordering::Relaxed`, its full path, or an imported variant), as
    /// its discriminant: 0 Relaxed, 1 Release, 2 Acquire, 3 AcqRel,
    /// 4 SeqCst. None for any other expression (a variable, a const).
    fn literal_atomic_ordering(&self, arg: ExprId) -> Option<u32> {
        if !matches!(self.p.ast.expr(arg), Expr::Path(_)) {
            return None;
        }
        let Res::Variant(a, v) = self.out.expr_res[self.ei(arg)] else { return None };
        (self.p.tys.local_adt_of(crate::check::generics::atomic_ordering_def()) == Some(a)).then_some(v)
    }

    /// rustc's deny-by-default `invalid_atomic_ordering` lint: a load
    /// with `Release` or `AcqRel`, a store with `Acquire` or `AcqRel`, a
    /// fence with `Relaxed`, and a compare-exchange failure ordering of
    /// `Release` or `AcqRel`, each written as the variant itself, do
    /// not compile. Through a variable they panic at run time instead.
    pub(super) fn lint_atomic_ordering(&mut self, op: SyncOp, args: &[ExprId]) {
        let (at, bad, msg): (usize, &[u32], &str) = match op {
            SyncOp::AtomicLoad => (0, &[1, 3], "atomic loads cannot have `Release` or `AcqRel` ordering"),
            SyncOp::AtomicStore => (1, &[2, 3], "atomic stores cannot have `Acquire` or `AcqRel` ordering"),
            SyncOp::AtomicCompareExchange | SyncOp::AtomicCompareExchangeWeak => {
                (3, &[1, 3], "`compare_exchange`'s failure ordering may not be `Release` or `AcqRel`")
            }
            SyncOp::AtomicFetchUpdate => {
                (1, &[1, 3], "`fetch_update`'s failure ordering may not be `Release` or `AcqRel`, since a failed `fetch_update` does not result in a write")
            }
            SyncOp::Fence | SyncOp::CompilerFence => (0, &[0], "memory fences cannot have `Relaxed` ordering"),
            _ => return,
        };
        if let Some(&arg) = args.get(at)
            && let Some(v) = self.literal_atomic_ordering(arg)
            && bad.contains(&v)
        {
            let span = self.span(arg);
            self.err("L0542", span, msg, "Use an ordering this operation allows (rustc `invalid_atomic_ordering`).");
        }
    }
}

/// The std impls of the sync types, as synthetic impl records for trait
/// selection (`T: Default` bounds, derives over sync fields): every
/// atomic is `Debug`, `Default`, and `From<V>` of its value.
pub fn add_sync_impls(p: &mut crate::check::collect::Program, ordinal: &mut u32, span: Span) {
    use crate::check::generics::insert_synthetic;
    use crate::traits::index::Synthetic;
    let lang = p.gx.lang;
    let g0 = Generics::default();
    // `Arc<T>` as alloc writes it, like `Rc<T>`: Clone with no bound on
    // `T`, the value traits through `T`, `Deref<Target = T>`, `AsRef<T>`,
    // `Borrow<T>`. `sync::Weak<T>`: Clone, Debug, Default.
    let p0 = p.tys.intern(TyKind::Param(crate::ty::ParamId(0)));
    let one = Generics { parent: None, params: vec![ParamDef { name: p.interner.intern("T"), kind: ParamKind::Type, span }], predicates: Vec::new() };
    let arc = sync_ty(&mut p.tys, SyncTy::Arc, &[p0]);
    let weak = sync_ty(&mut p.tys, SyncTy::ArcWeak, &[p0]);
    let bound = |p: &mut crate::check::collect::Program, tr: DefId| -> Generics {
        let mut a = vec![GenericArg::Ty(p0)];
        if tr == lang.partial_eq || tr == lang.partial_ord {
            a.push(GenericArg::Ty(p0));
        }
        let args = p.tys.intern_args(&a);
        let mut g = one.clone();
        g.predicates = vec![Predicate::Trait(Binder::plain(TraitRef { def: tr, args, bindings: EMPTY_BINDINGS }))];
        g
    };
    insert_synthetic(p, ordinal, lang.clone, arc, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
    for tr in [lang.debug, lang.display, lang.eq, lang.hash, lang.default, lang.partial_eq, lang.partial_ord, lang.ord] {
        let g = bound(p, tr);
        let rhs: Vec<Ty> = if tr == lang.partial_eq || tr == lang.partial_ord { vec![arc] } else { Vec::new() };
        insert_synthetic(p, ordinal, tr, arc, &rhs, g, Vec::new(), Synthetic::Lang, false, span);
    }
    insert_synthetic(p, ordinal, lang.deref, arc, &[], one.clone(), vec![(lang.assoc_target, p0)], Synthetic::Lang, false, span);
    for tr in [lang.as_ref, lang.borrow] {
        insert_synthetic(p, ordinal, tr, arc, &[p0], one.clone(), Vec::new(), Synthetic::Lang, false, span);
    }
    for tr in [lang.clone, lang.debug] {
        insert_synthetic(p, ordinal, tr, weak, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
    }
    // `impl<T> Default for Weak<T>`: `T: Sized` (`Weak<dyn Fn()>` has
    // no default; R26 sweep).
    let g = bound(p, lang.sized);
    insert_synthetic(p, ordinal, lang.default, weak, &[], g, Vec::new(), Synthetic::Lang, false, span);
    // `Mutex<T>` and `RwLock<T>`: Debug and Default through `T`,
    // `From<T>`. Their guards: `Deref<Target = T>` (and `DerefMut` but
    // for the read guard), Debug and Display through `T`. `PoisonError`
    // is Debug and Display for every `T`.
    let bounded = |p: &mut crate::check::collect::Program, tr: DefId| -> Generics {
        let args = p.tys.intern_args(&[GenericArg::Ty(p0)]);
        let mut g = one.clone();
        g.predicates = vec![Predicate::Trait(Binder::plain(TraitRef { def: tr, args, bindings: EMPTY_BINDINGS }))];
        g
    };
    for k in [SyncTy::Mutex, SyncTy::RwLock] {
        let t = sync_ty(&mut p.tys, k, &[p0]);
        for tr in [lang.debug, lang.default] {
            let g = bounded(p, tr);
            insert_synthetic(p, ordinal, tr, t, &[], g, Vec::new(), Synthetic::Lang, false, span);
        }
        insert_synthetic(p, ordinal, lang.from, t, &[p0], one.clone(), Vec::new(), Synthetic::Lang, false, span);
    }
    for k in [SyncTy::MutexGuard, SyncTy::RwLockReadGuard, SyncTy::RwLockWriteGuard] {
        let t = sync_ty(&mut p.tys, k, &[p0]);
        insert_synthetic(p, ordinal, lang.deref, t, &[], one.clone(), vec![(lang.assoc_target, p0)], Synthetic::Lang, false, span);
        if k.guard().is_some_and(|g| g.1) {
            insert_synthetic(p, ordinal, lang.deref_mut, t, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
        for tr in [lang.debug, lang.display] {
            let g = bounded(p, tr);
            insert_synthetic(p, ordinal, tr, t, &[], g, Vec::new(), Synthetic::Lang, false, span);
        }
    }
    // The thread types: `JoinHandle<T>` and the join error are Debug;
    // `Thread` is Clone and Debug; `ThreadId` Copy, Eq, and Debug.
    let jh = sync_ty(&mut p.tys, SyncTy::JoinHandle, &[p0]);
    insert_synthetic(p, ordinal, lang.debug, jh, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
    for k in [SyncTy::Sender, SyncTy::SyncSender, SyncTy::Receiver, SyncTy::SendError] {
        let t = sync_ty(&mut p.tys, k, &[p0]);
        insert_synthetic(p, ordinal, lang.debug, t, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
        if k != SyncTy::Receiver && k != SyncTy::SendError {
            insert_synthetic(p, ordinal, lang.clone, t, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
        if k == SyncTy::SendError {
            insert_synthetic(p, ordinal, lang.display, t, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
    }
    for k in [SyncTy::Condvar, SyncTy::Barrier, SyncTy::BarrierWaitResult] {
        let t = sync_ty(&mut p.tys, k, &[]);
        insert_synthetic(p, ordinal, lang.debug, t, &[], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
        if k == SyncTy::Condvar {
            insert_synthetic(p, ordinal, lang.default, t, &[], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
    }
    let du = sync_ty(&mut p.tys, SyncTy::Duration, &[]);
    for tr in [lang.clone, lang.copy, lang.eq, lang.default] {
        insert_synthetic(p, ordinal, tr, du, &[], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
    }
    insert_synthetic(p, ordinal, lang.partial_eq, du, &[du], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
    let ol = sync_ty(&mut p.tys, SyncTy::OnceLock, &[p0]);
    insert_synthetic(p, ordinal, lang.default, ol, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
    insert_synthetic(p, ordinal, lang.from, ol, &[p0], one.clone(), Vec::new(), Synthetic::Lang, false, span);
    let g = bounded(p, lang.debug);
    insert_synthetic(p, ordinal, lang.debug, ol, &[], g, Vec::new(), Synthetic::Lang, false, span);
    let sjh = sync_ty(&mut p.tys, SyncTy::ScopedJoinHandle, &[p0]);
    insert_synthetic(p, ordinal, lang.debug, sjh, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
    let je = sync_ty(&mut p.tys, SyncTy::JoinError, &[]);
    insert_synthetic(p, ordinal, lang.debug, je, &[], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
    let th = sync_ty(&mut p.tys, SyncTy::Thread, &[]);
    for tr in [lang.clone, lang.debug] {
        insert_synthetic(p, ordinal, tr, th, &[], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
    }
    let tid = sync_ty(&mut p.tys, SyncTy::ThreadId, &[]);
    for tr in [lang.clone, lang.copy, lang.debug, lang.eq] {
        insert_synthetic(p, ordinal, tr, tid, &[], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
    }
    insert_synthetic(p, ordinal, lang.partial_eq, tid, &[tid], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
    let pe = sync_ty(&mut p.tys, SyncTy::PoisonError, &[p0]);
    for tr in [lang.debug, lang.display] {
        insert_synthetic(p, ordinal, tr, pe, &[], one.clone(), Vec::new(), Synthetic::Lang, false, span);
    }
    for k in SyncTy::ALL {
        let Some(v) = k.atomic_value() else { continue };
        let t = sync_ty(&mut p.tys, k, &[]);
        for tr in [lang.debug, lang.default] {
            insert_synthetic(p, ordinal, tr, t, &[], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
        }
        insert_synthetic(p, ordinal, lang.from, t, &[v.ty()], g0.clone(), Vec::new(), Synthetic::Lang, false, span);
    }
}

/// A written impl of an auto trait for a user Adt (`unsafe impl Send for
/// S {}`): rustc then uses only the written impls, not the structural
/// rule. Some(true) when an impl holds for every instantiation (no bound
/// beyond `Sized`); Some(false) when only conditional impls exist (lust
/// does not prove their bounds yet: a safe reject); None with no written
/// impl (the caller walks the fields).
pub fn explicit_auto_impl(p: &crate::check::collect::Program, tys: &impl TyIntern, t: Ty, trait_def: DefId) -> Option<bool> {
    use crate::traits::index::{SelfHead, Synthetic};
    let (def, _) = tys.adt_parts(t)?;
    let any = p.gx.index.impls_of_trait(trait_def).iter().any(|&id| {
        let rec = p.gx.index.peek(id);
        rec.synthetic == Synthetic::None && rec.head == SelfHead::Adt(def)
    });
    any.then(|| crate::check::fmt::user_impl_holds(p, tys, t, trait_def))
}

/// The auto trait facts (`Send`, `Sync`, `Unpin`) of the std builtin
/// types, as std's own impls give them (ARCH 2.7): Some(answer) for a
/// builtin with a fixed rule, None for a type the caller walks by
/// structure (`Vec`, `Option`, `Box`, `Result`, user Adts). Every
/// builtin is `Unpin`. Checked against rustc 1.99 by the probes in
/// tests/pass and tests/fail (`send_sync_*`).
pub fn builtin_auto(
    p: &crate::check::collect::Program,
    tys: &mut impl TyIntern,
    env: &[Predicate],
    trait_def: DefId,
    t: Ty,
    seen: &mut crate::check::collect::AutoSeen,
) -> Option<Option<bool>> {
    use crate::check::collect::auto_structural;
    use crate::ty::BuiltinTy as B;
    let lang = &p.gx.lang;
    let b = tys.builtin_of(t)?;
    let structural_unpin = matches!(b, B::Vec | B::Option | B::Boxed | B::Result | B::Sync(SyncTy::Mutex | SyncTy::RwLock | SyncTy::PoisonError));
    if trait_def == lang.unpin && !structural_unpin {
        return Some(Some(true));
    }
    let send = trait_def == lang.send;
    let elem = tys.adt_first_ty_arg(t);
    let each = |tys: &mut _, parts: Vec<Ty>, seen: &mut crate::check::collect::AutoSeen| -> Option<bool> {
        seen.push((trait_def, t));
        let mut unknown = false;
        for (part, tr) in parts.into_iter().map(|x| (x, trait_def)) {
            match auto_structural(p, tys, env, tr, part, seen) {
                Some(false) => return Some(false),
                None => unknown = true,
                Some(true) => {}
            }
        }
        if unknown { None } else { Some(true) }
    };
    Some(match b {
        // `LocalKey` holds a fn pointer; `AccessError` is a unit struct.
        B::String | B::Chars | B::ExitCode | B::LocalKey | B::AccessError => Some(true),
        // The std hashers are plain values (BuildHasherDefault<H> holds a
        // PhantomData<fn() -> H>); the tables and Entry follow their type
        // arguments, as hashbrown's auto impls do (W-STD-B).
        B::Hash(k) => {
            use crate::ty::hash::HashTy as H;
            if matches!(k, H::RandomState | H::DefaultHasher | H::BuildHasherDefault) {
                Some(true)
            } else {
                let args: Vec<Ty> = tys.adt_parts(t).map(|(_, a)| tys.args(a).iter().filter_map(|g| g.as_ty()).collect()).unwrap_or_default();
                each(tys, args, seen)
            }
        }
        // `env::Args`, `Rc`, `rc::Weak`, the `RefCell` guards, and the
        // `fmt` types are neither `Send` nor `Sync`.
        B::Iter(crate::ty::iter::IterKind::Args) | B::Rc | B::Weak | B::RefGuard | B::RefMutGuard | B::Fmt(_) => Some(false),
        // `Cell<T>` and `RefCell<T>` are `Send` when `T` is; never `Sync`.
        B::Cell | B::RefCell => {
            if send {
                each(tys, elem.into_iter().collect(), seen)
            } else {
                Some(false)
            }
        }
        // `Arc<T>` and `sync::Weak<T>` are `Send` and `Sync` exactly when
        // `T` is both (std's unsafe impls).
        B::Sync(SyncTy::Arc | SyncTy::ArcWeak) => {
            let Some(e) = elem else { return Some(None) };
            seen.push((trait_def, t));
            let mut unknown = false;
            for tr in [lang.send, lang.sync] {
                match auto_structural(p, tys, env, tr, e, seen) {
                    Some(false) => return Some(Some(false)),
                    None => unknown = true,
                    Some(true) => {}
                }
            }
            if unknown { None } else { Some(true) }
        }
        // std's impls: `Mutex<T>` is Send and Sync for `T: Send`;
        // `RwLock<T>` is Send for `T: Send`, Sync for `T: Send + Sync`;
        // the guards are never Send and are Sync for `T: Sync`;
        // `PoisonError<T>` follows `T`.
        B::Sync(SyncTy::Mutex | SyncTy::RwLock | SyncTy::MutexGuard | SyncTy::RwLockReadGuard | SyncTy::RwLockWriteGuard | SyncTy::PoisonError) => {
            let Some(e) = elem else { return Some(None) };
            let k = crate::ty::sync::sync_kind(tys, t).expect("a sync type");
            let needs: &[DefId] = match k {
                _ if trait_def == lang.unpin => &[lang.unpin],
                SyncTy::Mutex => &[lang.send],
                SyncTy::RwLock if send => &[lang.send],
                SyncTy::RwLock => &[lang.send, lang.sync],
                SyncTy::PoisonError => &[trait_def],
                _ if send => return Some(Some(false)),
                _ => &[lang.sync],
            };
            seen.push((trait_def, t));
            let mut unknown = false;
            for &tr in needs {
                match auto_structural(p, tys, env, tr, e, seen) {
                    Some(false) => return Some(Some(false)),
                    None => unknown = true,
                    Some(true) => {}
                }
            }
            if unknown { None } else { Some(true) }
        }
        // std's impls: `JoinHandle<T>`, `Thread`, and `ThreadId` are Send
        // and Sync for every `T`; `Box<dyn Any + Send>` is Send only.
        B::Sync(SyncTy::JoinHandle | SyncTy::ScopedJoinHandle | SyncTy::Scope | SyncTy::Thread | SyncTy::ThreadId) => Some(true),
        B::Sync(SyncTy::JoinError) => Some(send),
        B::Sync(SyncTy::Condvar | SyncTy::Barrier | SyncTy::BarrierWaitResult | SyncTy::Duration) => Some(true),
        // `OnceLock<T>` is Send for `T: Send`, Sync for `T: Send + Sync`.
        B::Sync(SyncTy::OnceLock) => {
            let Some(e) = elem else { return Some(None) };
            seen.push((trait_def, t));
            let mut unknown = false;
            let needs: &[DefId] = if send { &[lang.send] } else { &[lang.send, lang.sync] };
            for &tr in needs {
                match auto_structural(p, tys, env, tr, e, seen) {
                    Some(false) => return Some(Some(false)),
                    None => unknown = true,
                    Some(true) => {}
                }
            }
            if unknown { None } else { Some(true) }
        }
        // `Sender<T>` and `SyncSender<T>` are Send and Sync for `T:
        // Send`; `Receiver<T>` is Send for `T: Send`, never Sync;
        // `SendError<T>` follows `T`.
        B::Sync(SyncTy::Sender | SyncTy::SyncSender | SyncTy::Receiver | SyncTy::SendError) => {
            let k = crate::ty::sync::sync_kind(tys, t).expect("a sync type");
            if k == SyncTy::Receiver && !send {
                return Some(Some(false));
            }
            let Some(e) = elem else { return Some(None) };
            let tr = if k == SyncTy::SendError { trait_def } else { lang.send };
            seen.push((trait_def, t));
            auto_structural(p, tys, env, tr, e, seen)
        }
        B::Sync(k) => match k.atomic_value() {
            Some(_) => Some(true),
            None => None,
        },
        // The hash iterators hold their table's state, not fields this
        // walk sees: not proven either way (W-STD-B).
        B::Iter(k) if k.is_hash() => return Some(None),
        B::Iter(k) => {
            let Some((_, args)) = crate::ty::iter::iter_parts(tys, t) else { return Some(None) };
            let mut parts = Vec::new();
            for f in k.fields() {
                use crate::ty::iter::Field as F;
                match *f {
                    F::Arg(i) | F::OptArg(i) | F::VecArg(i) => parts.push(args[i as usize]),
                    // `Chars` and `&str` are always Send and Sync.
                    F::Usize | F::Bool | F::Chars | F::Str => {}
                    F::SliceRef(i, m) => {
                        let sl = tys.intern(TyKind::Slice(args[i as usize]));
                        parts.push(tys.mk_ref(m, sl));
                    }
                    F::Recv(..) | F::VecRefMut(_) | F::StringRefMut | F::OptRef(..) => parts.push(crate::ty::iter::field_ty(tys, *f, &args)),
                    // The peeked item: not modelled, so not proven.
                    F::PeekSlot(_) => return Some(None),
                }
            }
            each(tys, parts, seen)
        }
        B::Vec | B::Option | B::Boxed | B::Result => return None,
    })
}
