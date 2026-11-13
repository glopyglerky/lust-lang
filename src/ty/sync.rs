// The `std::sync` and `std::thread` types lust knows by def (W-THREADS,
// ARCH 7.3). Each is a builtin Adt with its own def; values live in the
// C runtime (src/runtime/sync.c) and codegen (src/cgen/sync.rs). Type
// paths reach them only by their full std path: none is in the prelude,
// so a bare name needs a `use`, which the resolver rewrites to the path.
//
// `atomic::Ordering` is not here: it is an ordinary fieldless enum Adt
// (`check::generics::atomic_ordering_def`), like `cmp::Ordering`.
use super::*;

/// One sync or thread type lust knows by def.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SyncTy {
    /// `alloc::sync::Arc<T>`: the `Rc` header with atomic counts.
    Arc,
    /// `alloc::sync::Weak<T>`.
    ArcWeak,
    AtomicBool,
    AtomicI8,
    AtomicU8,
    AtomicI16,
    AtomicU16,
    AtomicI32,
    AtomicU32,
    AtomicI64,
    AtomicU64,
    AtomicIsize,
    AtomicUsize,
    /// `std::sync::Mutex<T>`: a futex word, the poison flag, the value.
    Mutex,
    /// `std::sync::MutexGuard<'a, T>`: unlocks on drop.
    MutexGuard,
    /// `std::sync::RwLock<T>`.
    RwLock,
    /// `std::sync::RwLockReadGuard<'a, T>`.
    RwLockReadGuard,
    /// `std::sync::RwLockWriteGuard<'a, T>`.
    RwLockWriteGuard,
    /// `std::sync::PoisonError<T>`: the `Err` of a `LockResult`. Under
    /// `panic=abort` no lock is ever poisoned, so none is ever built.
    PoisonError,
    /// `std::thread::JoinHandle<T>`.
    JoinHandle,
    /// `std::thread::Thread`: a handle to a thread (`thread::current()`).
    Thread,
    /// `std::thread::ThreadId`.
    ThreadId,
    /// The `Err` of `thread::Result<T>`: `Box<dyn Any + Send>`. Lust has
    /// no `Any`, so this stands for that one type and no path names it.
    /// Under `panic=abort` no join ever returns it.
    JoinError,
    /// `std::thread::Scope<'scope, 'env>`.
    Scope,
    /// `std::thread::ScopedJoinHandle<'scope, T>`.
    ScopedJoinHandle,
    /// `std::sync::mpsc::Sender<T>`.
    Sender,
    /// `std::sync::mpsc::SyncSender<T>`.
    SyncSender,
    /// `std::sync::mpsc::Receiver<T>`.
    Receiver,
    /// `std::sync::mpsc::SendError<T>`: the value a send gives back.
    SendError,
    /// `std::sync::Condvar`.
    Condvar,
    /// `std::sync::Barrier`.
    Barrier,
    /// `std::sync::BarrierWaitResult`.
    BarrierWaitResult,
    /// `std::sync::OnceLock<T>`.
    OnceLock,
    /// `core::time::Duration` (for `thread::sleep`).
    Duration,
    /// `std::thread::Builder` (TRIAGE T5): name and stack size.
    ThreadBuilder,
}

impl SyncTy {
    pub const ALL: [SyncTy; 35] = [
        SyncTy::Arc,
        SyncTy::ArcWeak,
        SyncTy::AtomicBool,
        SyncTy::AtomicI8,
        SyncTy::AtomicU8,
        SyncTy::AtomicI16,
        SyncTy::AtomicU16,
        SyncTy::AtomicI32,
        SyncTy::AtomicU32,
        SyncTy::AtomicI64,
        SyncTy::AtomicU64,
        SyncTy::AtomicIsize,
        SyncTy::AtomicUsize,
        SyncTy::Mutex,
        SyncTy::MutexGuard,
        SyncTy::RwLock,
        SyncTy::RwLockReadGuard,
        SyncTy::RwLockWriteGuard,
        SyncTy::PoisonError,
        SyncTy::JoinHandle,
        SyncTy::Thread,
        SyncTy::ThreadId,
        SyncTy::JoinError,
        SyncTy::Scope,
        SyncTy::ScopedJoinHandle,
        SyncTy::Sender,
        SyncTy::SyncSender,
        SyncTy::Receiver,
        SyncTy::SendError,
        SyncTy::Condvar,
        SyncTy::Barrier,
        SyncTy::BarrierWaitResult,
        SyncTy::OnceLock,
        SyncTy::Duration,
        SyncTy::ThreadBuilder,
    ];

    /// The struct name (the def hash input and the printed name).
    pub fn name(self) -> &'static str {
        match self {
            SyncTy::Arc => "Arc",
            SyncTy::ArcWeak => "Weak",
            SyncTy::AtomicBool => "AtomicBool",
            SyncTy::AtomicI8 => "AtomicI8",
            SyncTy::AtomicU8 => "AtomicU8",
            SyncTy::AtomicI16 => "AtomicI16",
            SyncTy::AtomicU16 => "AtomicU16",
            SyncTy::AtomicI32 => "AtomicI32",
            SyncTy::AtomicU32 => "AtomicU32",
            SyncTy::AtomicI64 => "AtomicI64",
            SyncTy::AtomicU64 => "AtomicU64",
            SyncTy::AtomicIsize => "AtomicIsize",
            SyncTy::AtomicUsize => "AtomicUsize",
            SyncTy::Mutex => "Mutex",
            SyncTy::MutexGuard => "MutexGuard",
            SyncTy::RwLock => "RwLock",
            SyncTy::RwLockReadGuard => "RwLockReadGuard",
            SyncTy::RwLockWriteGuard => "RwLockWriteGuard",
            SyncTy::PoisonError => "PoisonError",
            SyncTy::JoinHandle => "JoinHandle",
            SyncTy::Thread => "Thread",
            SyncTy::ThreadId => "ThreadId",
            SyncTy::JoinError => "Box<dyn Any + Send>",
            SyncTy::Scope => "Scope",
            SyncTy::ScopedJoinHandle => "ScopedJoinHandle",
            SyncTy::Sender => "Sender",
            SyncTy::SyncSender => "SyncSender",
            SyncTy::Receiver => "Receiver",
            SyncTy::SendError => "SendError",
            SyncTy::Condvar => "Condvar",
            SyncTy::Barrier => "Barrier",
            SyncTy::BarrierWaitResult => "BarrierWaitResult",
            SyncTy::OnceLock => "OnceLock",
            SyncTy::Duration => "Duration",
            SyncTy::ThreadBuilder => "Builder",
        }
    }

    /// The std module path the type lives in, under `std` (and under
    /// `core` or `alloc` for the types those crates define).
    pub fn module(self) -> &'static [&'static str] {
        match self {
            _ if k_is_atomic(self) => &["sync", "atomic"],
            SyncTy::JoinHandle | SyncTy::Thread | SyncTy::ThreadId | SyncTy::JoinError | SyncTy::Scope | SyncTy::ScopedJoinHandle | SyncTy::ThreadBuilder => &["thread"],
            SyncTy::Sender | SyncTy::SyncSender | SyncTy::Receiver | SyncTy::SendError => &["sync", "mpsc"],
            SyncTy::Duration => &["time"],
            _ => &["sync"],
        }
    }

    /// The crates whose module path also names the type (`std` always;
    /// the def hashes under the first).
    pub fn crates(self) -> &'static [&'static str] {
        match self {
            SyncTy::Arc | SyncTy::ArcWeak => &["alloc", "std"],
            _ if k_is_atomic(self) => &["core", "std"],
            SyncTy::Duration => &["core", "std"],
            _ => &["std"],
        }
    }

    /// The number of type parameters.
    pub fn arity(self) -> usize {
        match self {
            _ if k_is_atomic(self) => 0,
            SyncTy::Thread
            | SyncTy::ThreadId
            | SyncTy::JoinError
            | SyncTy::Scope
            | SyncTy::Condvar
            | SyncTy::Barrier
            | SyncTy::BarrierWaitResult
            | SyncTy::ThreadBuilder
            | SyncTy::Duration => 0,
            _ => 1,
        }
    }

    /// The name rustc prints for the type (`std::sync::Weak` keeps its
    /// module, as rustc tells it from `rc::Weak`).
    pub fn display(self) -> String {
        match self {
            SyncTy::ArcWeak => "std::sync::Weak".into(),
            SyncTy::JoinError => "Box<dyn Any + Send>".into(),
            k => k.name().into(),
        }
    }

    /// The value type of an atomic: `bool` or the integer type.
    pub fn atomic_value(self) -> Option<AtomicVal> {
        Some(match self {
            SyncTy::Arc
            | SyncTy::ArcWeak
            | SyncTy::Mutex
            | SyncTy::MutexGuard
            | SyncTy::RwLock
            | SyncTy::RwLockReadGuard
            | SyncTy::RwLockWriteGuard
            | SyncTy::PoisonError
            | SyncTy::JoinHandle
            | SyncTy::Thread
            | SyncTy::ThreadId
            | SyncTy::JoinError
            | SyncTy::Scope
            | SyncTy::ScopedJoinHandle
            | SyncTy::Sender
            | SyncTy::SyncSender
            | SyncTy::Receiver
            | SyncTy::SendError
            | SyncTy::Condvar
            | SyncTy::Barrier
            | SyncTy::BarrierWaitResult
            | SyncTy::OnceLock
            | SyncTy::Duration
            | SyncTy::ThreadBuilder => return None,
            SyncTy::AtomicBool => AtomicVal::Bool,
            SyncTy::AtomicI8 => AtomicVal::Int(IntTy::I8),
            SyncTy::AtomicU8 => AtomicVal::Int(IntTy::U8),
            SyncTy::AtomicI16 => AtomicVal::Int(IntTy::I16),
            SyncTy::AtomicU16 => AtomicVal::Int(IntTy::U16),
            SyncTy::AtomicI32 => AtomicVal::Int(IntTy::I32),
            SyncTy::AtomicU32 => AtomicVal::Int(IntTy::U32),
            SyncTy::AtomicI64 => AtomicVal::Int(IntTy::I64),
            SyncTy::AtomicU64 => AtomicVal::Int(IntTy::U64),
            SyncTy::AtomicIsize => AtomicVal::Int(IntTy::Isize),
            SyncTy::AtomicUsize => AtomicVal::Int(IntTy::Usize),
        })
    }

    /// For a lock guard: the lock kind it borrows and whether it gives
    /// `&mut T` (`DerefMut`). The guards deref like `RefCell`'s `Ref`
    /// and `RefMut` (TyIntern::guard_elem).
    pub fn guard(self) -> Option<(SyncTy, bool)> {
        match self {
            SyncTy::MutexGuard => Some((SyncTy::Mutex, true)),
            SyncTy::RwLockReadGuard => Some((SyncTy::RwLock, false)),
            SyncTy::RwLockWriteGuard => Some((SyncTy::RwLock, true)),
            _ => None,
        }
    }

    /// The number of lifetime params, ahead of the type params: one for
    /// a lock guard and a `ScopedJoinHandle`, two for a `Scope`.
    pub fn regions(self) -> u32 {
        match self {
            SyncTy::Scope => 2,
            SyncTy::ScopedJoinHandle => 1,
            _ => u32::from(self.guard().is_some()),
        }
    }

    /// True for the types whose C form is a struct of state words and
    /// the value: `Mutex`, `RwLock`, and `OnceLock`.
    pub fn is_cell_struct(self) -> bool {
        matches!(self, SyncTy::Mutex | SyncTy::RwLock | SyncTy::OnceLock)
    }

    /// True for `Mutex` and `RwLock`.
    pub fn is_lock(self) -> bool {
        matches!(self, SyncTy::Mutex | SyncTy::RwLock)
    }

    /// The kind a std type path names (`std::sync::atomic::AtomicUsize`,
    /// `core::sync::atomic::AtomicBool`), if it is one of these.
    pub fn by_path(path: &[&str]) -> Option<SyncTy> {
        let (&name, rest) = path.split_last()?;
        let (&krate, module) = rest.split_first()?;
        SyncTy::ALL.iter().copied().find(|k| k.name() == name && k.module() == module && k.crates().contains(&krate))
    }
}

/// True for the eleven atomics.
fn k_is_atomic(k: SyncTy) -> bool {
    matches!(
        k,
        SyncTy::AtomicBool
            | SyncTy::AtomicI8
            | SyncTy::AtomicU8
            | SyncTy::AtomicI16
            | SyncTy::AtomicU16
            | SyncTy::AtomicI32
            | SyncTy::AtomicU32
            | SyncTy::AtomicI64
            | SyncTy::AtomicU64
            | SyncTy::AtomicIsize
            | SyncTy::AtomicUsize
    )
}

/// The value an atomic holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AtomicVal {
    Bool,
    Int(IntTy),
}

impl AtomicVal {
    /// The value type.
    pub fn ty(self) -> Ty {
        match self {
            AtomicVal::Bool => BOOL,
            AtomicVal::Int(i) => int(i),
        }
    }
    /// The width in bytes (an `isize`/`usize` is 8, x86_64 and aarch64).
    pub fn bytes(self) -> u32 {
        match self {
            AtomicVal::Bool => 1,
            AtomicVal::Int(i) => i.bits() / 8,
        }
    }
}

/// The def of one sync type (see `BuiltinDefs::sync_tys`).
pub(super) fn sync_def(k: SyncTy) -> DefId {
    let mut module = vec![k.crates()[0]];
    module.extend_from_slice(k.module());
    builtin_def(&module, k.name())
}

/// The sync kind of a type, when it is one of these.
pub fn sync_kind(tys: &impl TyIntern, t: Ty) -> Option<SyncTy> {
    match tys.builtin_of(t) {
        Some(BuiltinTy::Sync(k)) => Some(k),
        _ => None,
    }
}

/// The type of one sync struct applied to `args` (empty for the
/// atomics).
pub fn sync_ty(tys: &mut impl TyIntern, k: SyncTy, args: &[Ty]) -> Ty {
    let def = tys.builtins().sync_tys[k as usize];
    let a: Vec<GenericArg> = args.iter().map(|&t| GenericArg::Ty(t)).collect();
    tys.mk_adt(def, &a)
}

/// The atomic value of a type, when it is an atomic.
pub fn atomic_of(tys: &impl TyIntern, t: Ty) -> Option<(SyncTy, AtomicVal)> {
    let k = sync_kind(tys, t)?;
    Some((k, k.atomic_value()?))
}

/// The std rows for `ty::lang`'s path tables: every crate and module
/// spelling of every sync type, as `crate/module/../Name`.
pub fn type_rows() -> Vec<String> {
    let mut out = Vec::new();
    for k in SyncTy::ALL {
        for c in k.crates() {
            let mut parts = vec![*c];
            parts.extend_from_slice(k.module());
            parts.push(k.name());
            out.push(parts.join("/"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_name_their_types() {
        assert_eq!(SyncTy::by_path(&["std", "sync", "atomic", "AtomicUsize"]), Some(SyncTy::AtomicUsize));
        assert_eq!(SyncTy::by_path(&["core", "sync", "atomic", "AtomicBool"]), Some(SyncTy::AtomicBool));
        assert_eq!(SyncTy::by_path(&["std", "sync", "AtomicBool"]), None);
        assert_eq!(SyncTy::by_path(&["alloc", "sync", "atomic", "AtomicBool"]), None);
        assert!(type_rows().contains(&"std/sync/atomic/AtomicI32".to_string()));
        assert_eq!(SyncTy::by_path(&["std", "sync", "MutexGuard"]), Some(SyncTy::MutexGuard));
        assert_eq!(SyncTy::by_path(&["core", "sync", "Mutex"]), None);
    }
}
