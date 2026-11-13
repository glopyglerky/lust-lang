// Lust adapter for rustix 1.1.5 (docs/ARCH.md section 3.6): the
// subset ts-rust's core files use, over the sys_x intrinsics
// (src/runtime/sysx.c) and the shared sys_fd intrinsics. Here:
// `io` (Errno with std's error texts, read, write), `fd`
// (std's BorrowedFd, OwnedFd, AsFd, AsRawFd, IntoRawFd,
// FromRawFd, re-exported as upstream), `fs` (the
// flag types, CWD, open, openat, fstat, statat, statx, accessat,
// utimensat, mknodat, fcntl_getfl and fcntl_setfl, statvfs,
// FileType, Stat, Statx, StatVfs, Timespec and Timestamps,
// and fs::inotify init and watches), `process` (Pid, Signal,
// kill and rlimit forms), `termios` (tcgetwinsize), and `path`
// (the Arg trait of the path-taking calls).
//
// Not here (see README.md): RawDir and inotify's Reader
// (their buffer type, core::mem::MaybeUninit, is not in lust
// yet), the net, mount, thread, time, and other modules, the
// `fs` items beyond the list above, and every `unsafe` form.
// Such code rejects in lust.
//
// Linux x86_64 values throughout, as the lust target is.

// Gated by upstream's features, so a unit with fewer features sees
// upstream's surface.
#[cfg(feature = "event")]
pub mod event;
pub mod buffer;
pub mod fd;
#[cfg(feature = "fs")]
pub mod fs;
pub mod io;
#[cfg(feature = "fs")]
pub mod path;
#[cfg(feature = "pipe")]
pub mod pipe;
#[cfg(feature = "process")]
pub mod process;
#[cfg(feature = "stdio")]
pub mod stdio;
#[cfg(feature = "termios")]
pub mod termios;

/// `struct timespec`, re-exported by `fs` and `event` as upstream.
#[allow(dead_code)]
mod timespec {
    /// Seconds.
    pub type Secs = i64;
    /// Nanoseconds.
    pub type Nsecs = i64;

    /// A quantity of time in seconds plus nanoseconds.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Timespec {
        /// Seconds.
        pub tv_sec: Secs,
        /// Nanoseconds.
        pub tv_nsec: Nsecs,
    }
}

/// Little-endian u64 field `i` of a sysx record.
fn rec_u64(rec: &[u8], i: usize) -> u64 {
    let mut v = 0u64;
    let base = i * 8;
    for k in 0..8 {
        v |= (rec[base + k] as u64) << (8 * k);
    }
    v
}

/// Little-endian i64 field `i` of a sysx record.
fn rec_i64(rec: &[u8], i: usize) -> i64 {
    rec_u64(rec, i) as i64
}
