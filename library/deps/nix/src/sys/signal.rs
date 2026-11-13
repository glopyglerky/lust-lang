// `nix::sys::signal`: the `Signal` enum (features process or signal)
// and, with `signal`, `SigSet` with its add/remove/contains forms and
// the thread mask calls tsgo's `unblock_go_signals` uses. The set is a
// bit mask of the signals 1..64 (Linux has no signal above 64); the
// mask calls go to glibc's `pthread_sigmask` through sysx, as upstream
// does through libc. Everything else in the module is absent and
// rejects in lust (README).

#[cfg(feature = "signal")]
use crate::errno::Errno;
#[cfg(feature = "signal")]
use crate::Result;

/// Types of operating system signals (Linux x86_64 numbers).
#[cfg(any(feature = "process", feature = "signal"))]
#[repr(i32)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Signal {
    /// Hangup
    SIGHUP = 1,
    /// Interrupt
    SIGINT = 2,
    /// Quit
    SIGQUIT = 3,
    /// Illegal instruction (not reset when caught)
    SIGILL = 4,
    /// Trace trap (not reset when caught)
    SIGTRAP = 5,
    /// Abort
    SIGABRT = 6,
    /// Bus error
    SIGBUS = 7,
    /// Floating point exception
    SIGFPE = 8,
    /// Kill (cannot be caught or ignored)
    SIGKILL = 9,
    /// User defined signal 1
    SIGUSR1 = 10,
    /// Segmentation violation
    SIGSEGV = 11,
    /// User defined signal 2
    SIGUSR2 = 12,
    /// Write on a pipe with no one to read it
    SIGPIPE = 13,
    /// Alarm clock
    SIGALRM = 14,
    /// Software termination signal from kill
    SIGTERM = 15,
    /// Stack fault (obsolete)
    SIGSTKFLT = 16,
    /// To parent on child stop or exit
    SIGCHLD = 17,
    /// Continue a stopped process
    SIGCONT = 18,
    /// Sendable stop signal not from tty
    SIGSTOP = 19,
    /// Stop signal from tty
    SIGTSTP = 20,
    /// To readers pgrp upon background tty read
    SIGTTIN = 21,
    /// Like TTIN if (tp->t_local&LTOSTOP)
    SIGTTOU = 22,
    /// Urgent condition on IO channel
    SIGURG = 23,
    /// Exceeded CPU time limit
    SIGXCPU = 24,
    /// Exceeded file size limit
    SIGXFSZ = 25,
    /// Virtual time alarm
    SIGVTALRM = 26,
    /// Profiling time alarm
    SIGPROF = 27,
    /// Window size changes
    SIGWINCH = 28,
    /// Input/output possible signal
    SIGIO = 29,
    /// Power failure imminent.
    SIGPWR = 30,
    /// Bad system call
    SIGSYS = 31,
}

#[cfg(feature = "signal")]
pub use self::Signal::*;

/// Specifies a set of `Signal`s that may be blocked, waited for, etc.
// No Debug (upstream prints libc's `sigset_t`), and no Eq or Hash yet.
#[cfg(feature = "signal")]
#[derive(Clone, Copy)]
pub struct SigSet {
    bits: u64,
}

/// `pthread_sigmask` `how` values (Linux).
#[cfg(feature = "signal")]
const SIG_BLOCK: i32 = 0;
#[cfg(feature = "signal")]
const SIG_UNBLOCK: i32 = 1;

#[cfg(feature = "signal")]
impl SigSet {
    /// Initialize to include nothing.
    pub fn empty() -> SigSet {
        SigSet { bits: 0 }
    }

    /// Add the specified signal to the set.
    pub fn add(&mut self, signal: Signal) {
        self.bits |= 1 << (signal as i32 - 1);
    }

    /// Remove all signals from this set.
    pub fn clear(&mut self) {
        self.bits = 0;
    }

    /// Remove the specified signal from this set.
    pub fn remove(&mut self, signal: Signal) {
        self.bits &= !(1 << (signal as i32 - 1));
    }

    /// Return whether this set includes the specified signal.
    pub fn contains(&self, signal: Signal) -> bool {
        self.bits & (1 << (signal as i32 - 1)) != 0
    }

    /// Adds the set of signals to the signal mask for the calling thread.
    pub fn thread_block(&self) -> Result<()> {
        sigmask(SIG_BLOCK, self.bits)
    }

    /// Removes the set of signals from the signal mask for the calling thread.
    pub fn thread_unblock(&self) -> Result<()> {
        sigmask(SIG_UNBLOCK, self.bits)
    }
}

/// One `pthread_sigmask(how, set, NULL)` call.
#[cfg(feature = "signal")]
fn sigmask(how: i32, bits: u64) -> Result<()> {
    let e = core::intrinsics::sys_x_sigmask(how, bits);
    if e != 0 {
        return Err(Errno::from_raw(e));
    }
    Ok(())
}

#[cfg(feature = "signal")]
impl Extend<Signal> for SigSet {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Signal>,
    {
        for signal in iter {
            self.add(signal);
        }
    }
}

#[cfg(feature = "signal")]
impl FromIterator<Signal> for SigSet {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Signal>,
    {
        let mut sigset = SigSet::empty();
        sigset.extend(iter);
        sigset
    }
}
