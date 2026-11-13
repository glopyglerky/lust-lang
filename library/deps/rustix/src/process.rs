// `rustix::process`: pids, signals, kill, parent death signal,
// resource limits, and `waitid`.

use crate::io::{Errno, Result};
use crate::rec_u64;
use core::fmt;

/// A process identifier as a raw integer.
pub type RawPid = i32;

/// A non-zero Unix process ID.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Pid(i32);

impl Pid {
    /// The init process (pid 1).
    pub const INIT: Pid = Pid(1);

    /// `Some` for a positive value, `None` for zero, as upstream.
    /// A negative value is kept (upstream only debug-asserts it).
    pub const fn from_raw(raw: RawPid) -> Option<Pid> {
        if raw == 0 { None } else { Some(Pid(raw)) }
    }

    #[cfg(feature = "std")]
    /// The pid of a child process.
    pub fn from_child(child: &std::process::Child) -> Pid {
        Pid(child.id() as i32)
    }

    /// The raw pid.
    pub const fn as_raw_pid(self) -> RawPid {
        self.0
    }

    /// The raw pid of an optional pid (0 for None).
    pub const fn as_raw(pid: Option<Pid>) -> RawPid {
        match pid {
            Some(pid) => pid.0,
            None => 0,
        }
    }

    /// True for the init process.
    pub const fn is_init(self) -> bool {
        self.0 == 1
    }
}

impl fmt::Display for Pid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A signal number.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Signal(i32);

impl Signal {
    pub const HUP: Signal = Signal::named(1);
    pub const INT: Signal = Signal::named(2);
    pub const QUIT: Signal = Signal::named(3);
    pub const ILL: Signal = Signal::named(4);
    pub const TRAP: Signal = Signal::named(5);
    pub const ABORT: Signal = Signal::named(6);
    pub const BUS: Signal = Signal::named(7);
    pub const FPE: Signal = Signal::named(8);
    pub const KILL: Signal = Signal::named(9);
    pub const USR1: Signal = Signal::named(10);
    pub const SEGV: Signal = Signal::named(11);
    pub const USR2: Signal = Signal::named(12);
    pub const PIPE: Signal = Signal::named(13);
    pub const ALARM: Signal = Signal::named(14);
    pub const TERM: Signal = Signal::named(15);
    pub const STKFLT: Signal = Signal::named(16);
    pub const CHILD: Signal = Signal::named(17);
    pub const CONT: Signal = Signal::named(18);
    pub const STOP: Signal = Signal::named(19);
    pub const TSTP: Signal = Signal::named(20);
    pub const TTIN: Signal = Signal::named(21);
    pub const TTOU: Signal = Signal::named(22);
    pub const URG: Signal = Signal::named(23);
    pub const XCPU: Signal = Signal::named(24);
    pub const XFSZ: Signal = Signal::named(25);
    pub const VTALARM: Signal = Signal::named(26);
    pub const PROF: Signal = Signal::named(27);
    pub const WINCH: Signal = Signal::named(28);
    pub const IO: Signal = Signal::named(29);
    pub const POWER: Signal = Signal::named(30);
    pub const SYS: Signal = Signal::named(31);

    /// A const from a known non-zero number.
    const fn named(n: i32) -> Signal {
        Signal(n)
    }

    /// The signal of a raw named (non-real-time) number, if any.
    pub const fn from_named_raw(sig: i32) -> Option<Signal> {
        match sig {
            1 => Some(Signal::HUP),
            2 => Some(Signal::INT),
            3 => Some(Signal::QUIT),
            4 => Some(Signal::ILL),
            5 => Some(Signal::TRAP),
            6 => Some(Signal::ABORT),
            7 => Some(Signal::BUS),
            8 => Some(Signal::FPE),
            9 => Some(Signal::KILL),
            10 => Some(Signal::USR1),
            11 => Some(Signal::SEGV),
            12 => Some(Signal::USR2),
            13 => Some(Signal::PIPE),
            14 => Some(Signal::ALARM),
            15 => Some(Signal::TERM),
            16 => Some(Signal::STKFLT),
            17 => Some(Signal::CHILD),
            18 => Some(Signal::CONT),
            19 => Some(Signal::STOP),
            20 => Some(Signal::TSTP),
            21 => Some(Signal::TTIN),
            22 => Some(Signal::TTOU),
            23 => Some(Signal::URG),
            24 => Some(Signal::XCPU),
            25 => Some(Signal::XFSZ),
            26 => Some(Signal::VTALARM),
            27 => Some(Signal::PROF),
            28 => Some(Signal::WINCH),
            29 => Some(Signal::IO),
            30 => Some(Signal::POWER),
            31 => Some(Signal::SYS),
            _ => None,
        }
    }

    /// The raw signal number.
    pub const fn as_raw(self) -> i32 {
        self.0
    }
}

impl fmt::Debug for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self.0 {
            1 => "Signal::HUP",
            2 => "Signal::INT",
            3 => "Signal::QUIT",
            4 => "Signal::ILL",
            5 => "Signal::TRAP",
            6 => "Signal::ABORT",
            7 => "Signal::BUS",
            8 => "Signal::FPE",
            9 => "Signal::KILL",
            10 => "Signal::USR1",
            11 => "Signal::SEGV",
            12 => "Signal::USR2",
            13 => "Signal::PIPE",
            14 => "Signal::ALARM",
            15 => "Signal::TERM",
            16 => "Signal::STKFLT",
            17 => "Signal::CHILD",
            18 => "Signal::CONT",
            19 => "Signal::STOP",
            20 => "Signal::TSTP",
            21 => "Signal::TTIN",
            22 => "Signal::TTOU",
            23 => "Signal::URG",
            24 => "Signal::XCPU",
            25 => "Signal::XFSZ",
            26 => "Signal::VTALARM",
            27 => "Signal::PROF",
            28 => "Signal::WINCH",
            29 => "Signal::IO",
            30 => "Signal::POWER",
            31 => "Signal::SYS",
            // Upstream formats the name str with Debug (quoted),
            // and the fallback the same way in three pieces.
            _ => return write!(f, "{:?}{:?}{:?}", "Signal::from_raw(", self.0, ")"),
        };
        write!(f, "{name:?}")
    }
}

/// `getpid()`.
pub fn getpid() -> Pid {
    let raw = core::intrinsics::sys_getpid() as i32;
    match Pid::from_raw(raw) {
        Some(p) => p,
        None => Pid::INIT,
    }
}

/// `getppid()`.
pub fn getppid() -> Option<Pid> {
    Pid::from_raw(core::intrinsics::sys_x_getppid())
}

/// `kill(pid, sig)`.
pub fn kill_process(pid: Pid, signal: Signal) -> Result<()> {
    kill_raw(pid.as_raw_pid(), signal.as_raw())
}

/// `kill(pid, 0)`: tests whether a process exists.
pub fn test_kill_process(pid: Pid) -> Result<()> {
    kill_raw(pid.as_raw_pid(), 0)
}

/// One kill call.
fn kill_raw(pid: i32, sig: i32) -> Result<()> {
    let e = core::intrinsics::sys_x_kill(pid, sig);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(())
}

/// `prctl(PR_SET_PDEATHSIG, sig)`.
pub fn set_parent_process_death_signal(signal: Option<Signal>) -> Result<()> {
    let sig = match signal {
        Some(s) => s.as_raw(),
        None => 0,
    };
    let e = core::intrinsics::sys_x_prctl_deathsig(sig);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(())
}

/// A resource limit kind (`RLIMIT_*`, Linux numbers).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
#[non_exhaustive]
pub enum Resource {
    Cpu = 0,
    Fsize = 1,
    Data = 2,
    Stack = 3,
    Core = 4,
    Rss = 5,
    Nproc = 6,
    Nofile = 7,
    Memlock = 8,
    As = 9,
    Locks = 10,
    Sigpending = 11,
    Msgqueue = 12,
    Nice = 13,
    Rtprio = 14,
    Rttime = 15,
}

/// Resource limits: `None` is no limit (infinity).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rlimit {
    pub current: Option<u64>,
    pub maximum: Option<u64>,
}

/// `getrlimit(resource)`. This never fails for a valid resource,
/// as upstream's signature has it.
pub fn getrlimit(resource: Resource) -> Rlimit {
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_getrlimit(resource as u32 as i32, &mut rec);
    if e != 0 || rec.len() < 16 {
        return Rlimit { current: None, maximum: None };
    }
    let lim = |v: u64| if v == u64::MAX { None } else { Some(v) };
    Rlimit { current: lim(rec_u64(&rec, 0)), maximum: lim(rec_u64(&rec, 1)) }
}

/// `setrlimit(resource, limits)`.
pub fn setrlimit(resource: Resource, limits: Rlimit) -> Result<()> {
    let raw = |v: Option<u64>| v.unwrap_or(u64::MAX);
    let e = core::intrinsics::sys_x_setrlimit(resource as u32 as i32, raw(limits.current), raw(limits.maximum));
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(())
}

/// Options for modifying the behavior of `waitid` (Linux values).
// No Debug (upstream prints the bitflags form, which this adapter lacks).
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct WaitIdOptions(u32);

impl WaitIdOptions {
    /// Return immediately if no child has exited.
    pub const NOHANG: WaitIdOptions = WaitIdOptions(1);
    /// Return if a stopped child has been resumed by delivery of `Signal::Cont`.
    pub const CONTINUED: WaitIdOptions = WaitIdOptions(8);
    /// Wait for processed that have exited.
    pub const EXITED: WaitIdOptions = WaitIdOptions(4);
    /// Keep processed in a waitable state.
    pub const NOWAIT: WaitIdOptions = WaitIdOptions(0x0100_0000);
    /// Wait for processes that have been stopped.
    pub const STOPPED: WaitIdOptions = WaitIdOptions(2);

    pub const fn empty() -> WaitIdOptions {
        WaitIdOptions(0)
    }
    pub const fn bits(&self) -> u32 {
        self.0
    }
    pub const fn from_bits_retain(bits: u32) -> WaitIdOptions {
        WaitIdOptions(bits)
    }
    pub const fn contains(&self, other: WaitIdOptions) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for WaitIdOptions {
    type Output = WaitIdOptions;
    fn bitor(self, other: WaitIdOptions) -> WaitIdOptions {
        WaitIdOptions(self.0 | other.0)
    }
}

/// The status of a process after calling `waitid`. Upstream holds the
/// raw `siginfo_t`; its accessors and Debug are not here yet.
#[derive(Clone, Copy)]
pub struct WaitIdStatus {
    #[allow(dead_code)]
    signo: i32,
    #[allow(dead_code)]
    status: i32,
    #[allow(dead_code)]
    code: i32,
}

/// The identifier to wait on in a call to `waitid`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WaitId<'a> {
    /// Wait on all processes.
    All,
    /// Wait for a specific process ID.
    Pid(Pid),
    /// Wait for a specific process group ID, or the calling process' group ID.
    Pgid(Option<Pid>),
    /// Wait for a specific process file descriptor.
    PidFd(crate::fd::BorrowedFd<'a>),
}

/// `waitid(_, _, _, opts)`: waits for the specified child process to
/// change state. `None` when `NOHANG` found no child that changed (the
/// `siginfo_t`'s pid stays 0), as upstream.
pub fn waitid<'a, Id: Into<WaitId<'a>>>(id: Id, options: WaitIdOptions) -> Result<Option<WaitIdStatus>> {
    use crate::fd::AsRawFd;
    // P_ALL, P_PID, P_PGID, P_PIDFD
    let (idtype, raw) = match id.into() {
        WaitId::All => (0, 0),
        WaitId::Pid(pid) => (1, pid.as_raw_pid()),
        WaitId::Pgid(pgid) => (2, Pid::as_raw(pgid)),
        WaitId::PidFd(fd) => (3, fd.as_raw_fd()),
    };
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_waitid(idtype, raw, options.bits() as i32, &mut rec);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    if rec_u64(&rec, 0) == 0 {
        return Ok(None);
    }
    Ok(Some(WaitIdStatus { signo: rec_u64(&rec, 2) as i32, status: rec_u64(&rec, 3) as i32, code: rec_u64(&rec, 4) as i32 }))
}
