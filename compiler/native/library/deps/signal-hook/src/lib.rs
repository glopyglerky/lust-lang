// Lust adapter for signal-hook 0.4.4 (docs/ARCH.md section
// 3.6): the consts, flag, iterator, and low_level forms
// ts-rust uses, over the sysx signal registry. Not here: the
// exfiltrator generics (SignalsInfo with custom exfiltrators),
// the async-signal-safe channel types beyond what iterator
// needs, and handler chaining to a handler installed before
// registration (the registry installs its own; README).

pub mod consts {
    /// The signal numbers upstream re-exports from libc on Linux
    /// (not SIGPWR, SIGSTKFLT, or the aliases).
    pub mod signal {
        pub const SIGHUP: i32 = 1;
        pub const SIGINT: i32 = 2;
        pub const SIGQUIT: i32 = 3;
        pub const SIGILL: i32 = 4;
        pub const SIGTRAP: i32 = 5;
        pub const SIGABRT: i32 = 6;
        pub const SIGBUS: i32 = 7;
        pub const SIGFPE: i32 = 8;
        pub const SIGKILL: i32 = 9;
        pub const SIGUSR1: i32 = 10;
        pub const SIGSEGV: i32 = 11;
        pub const SIGUSR2: i32 = 12;
        pub const SIGPIPE: i32 = 13;
        pub const SIGALRM: i32 = 14;
        pub const SIGTERM: i32 = 15;
        pub const SIGCHLD: i32 = 17;
        pub const SIGCONT: i32 = 18;
        pub const SIGSTOP: i32 = 19;
        pub const SIGTSTP: i32 = 20;
        pub const SIGTTIN: i32 = 21;
        pub const SIGTTOU: i32 = 22;
        pub const SIGURG: i32 = 23;
        pub const SIGXCPU: i32 = 24;
        pub const SIGXFSZ: i32 = 25;
        pub const SIGVTALRM: i32 = 26;
        pub const SIGPROF: i32 = 27;
        pub const SIGWINCH: i32 = 28;
        pub const SIGIO: i32 = 29;
        pub const SIGSYS: i32 = 31;
    }

    pub use self::signal::*;

    /// Signals commonly requesting shutdown.
    pub const TERM_SIGNALS: &[i32] = &[SIGTERM, SIGQUIT, SIGINT];

    /// Signals a handler may not be registered for (registering
    /// one panics, as upstream).
    pub const FORBIDDEN: &[i32] = &[signal::SIGKILL, signal::SIGSTOP, signal::SIGILL, signal::SIGFPE, signal::SIGSEGV];
}

pub mod flag;
#[cfg(feature = "iterator")]
pub mod iterator;
pub mod low_level;

/// An id of a registered action (upstream's `SigId`; unregister
/// by id is not in this adapter).
/// Upstream's is `SigId { signal, action }` with its own Debug, Hash,
/// and Ord; this one keeps only the identity (`PartialEq`).
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct SigId(usize);

/// The next SigId number.
fn next_sig_id() -> SigId {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    SigId(NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
}

/// Panics on a forbidden signal, as upstream's register does.
fn check_forbidden(signal: i32) {
    assert!(!consts::FORBIDDEN.contains(&signal), "Attempted to register forbidden signal {signal}");
}

/// An io error from a registry errno (0 is Ok).
fn reg_status(code: i32) -> std::io::Result<()> {
    if code == 0 { Ok(()) } else { Err(std::io::Error::from_raw_os_error(code)) }
}
