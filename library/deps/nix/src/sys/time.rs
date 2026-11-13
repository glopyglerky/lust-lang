// `nix::sys::time`: the TimeSpec form the event module's caller
// (ts-rust's fswatch/unix_bsd.rs) uses, over a plain (sec, nsec)
// pair. Upstream's TimeSpec wraps libc's timespec and has the full
// arithmetic around it; only the construction and the AsRef form
// are here (README). UTIME_NOW and UTIME_OMIT are the target's
// utimensat special values (Darwin spells them -1 and -2).

use core::time::Duration;

/// A timespec value, as `AsRef` yields it (upstream's is libc's).
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Timespec {
    /// Seconds.
    pub tv_sec: i64,
    /// Nanoseconds.
    pub tv_nsec: i64,
}

/// A timestamp in seconds and nanoseconds.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct TimeSpec(Timespec);

impl TimeSpec {
    /// Leave the timestamp unchanged (utimensat).
    #[cfg(target_os = "macos")]
    pub const UTIME_OMIT: TimeSpec = TimeSpec::new(0, -2);
    /// Leave the timestamp unchanged (utimensat).
    #[cfg(target_os = "linux")]
    pub const UTIME_OMIT: TimeSpec = TimeSpec::new(0, 1073741822);
    /// Update the timestamp to now (utimensat).
    #[cfg(target_os = "macos")]
    pub const UTIME_NOW: TimeSpec = TimeSpec::new(0, -1);
    /// Update the timestamp to now (utimensat).
    #[cfg(target_os = "linux")]
    pub const UTIME_NOW: TimeSpec = TimeSpec::new(0, 1073741823);

    /// A TimeSpec from seconds and nanoseconds.
    pub const fn new(seconds: i64, nanoseconds: i64) -> TimeSpec {
        TimeSpec(Timespec { tv_sec: seconds, tv_nsec: nanoseconds })
    }

    /// A TimeSpec from a Duration.
    pub fn from_duration(d: Duration) -> TimeSpec {
        TimeSpec::new(d.as_secs() as i64, d.subsec_nanos() as i64)
    }

    /// The seconds part.
    pub const fn tv_sec(&self) -> i64 {
        self.0.tv_sec
    }

    /// The nanoseconds part.
    pub const fn tv_nsec(&self) -> i64 {
        self.0.tv_nsec
    }
}

impl AsRef<Timespec> for TimeSpec {
    fn as_ref(&self) -> &Timespec {
        &self.0
    }
}

impl AsMut<Timespec> for TimeSpec {
    fn as_mut(&mut self) -> &mut Timespec {
        &mut self.0
    }
}

impl From<Duration> for TimeSpec {
    fn from(d: Duration) -> TimeSpec {
        TimeSpec::from_duration(d)
    }
}

impl From<TimeSpec> for Duration {
    fn from(t: TimeSpec) -> Duration {
        Duration::new(t.0.tv_sec as u64, t.0.tv_nsec as u32)
    }
}
