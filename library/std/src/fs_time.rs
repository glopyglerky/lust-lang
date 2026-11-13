// `std::fs::Metadata::modified` and `accessed` (W-STD-C overlay of
// `std::fs`; it defines no items, only the impl).
//
// They are not in the platform crate's fs.rs because they return a
// `std::time::SystemTime`: the platform crate resolves before the std
// splice that defines `std::time`, so it cannot name it. A std splice
// may hold inherent impls of the platform crate's types (both are the
// `std` shipped crate). The times come from the public `MetadataExt`
// accessors (the stat fields), as std reads `st_mtime`/`st_mtime_nsec`.

use std::fs::Metadata;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

impl Metadata {
    /// The last modification time, as std's on Linux.
    pub fn modified(&self) -> io::Result<SystemTime> {
        system_time(self.mtime(), self.mtime_nsec())
    }

    /// The last access time, as std's on Linux.
    pub fn accessed(&self) -> io::Result<SystemTime> {
        system_time(self.atime(), self.atime_nsec())
    }

    /// The creation (birth) time: the statx btime on Linux, the
    /// stat birth time on macOS, as std's. No birth time from the
    /// filesystem is std's Unsupported error.
    pub fn created(&self) -> io::Result<SystemTime> {
        let (sec, nsec, has) = self.__lust_btime();
        if !has {
            return Err(io::Error::__lust_adapter_simple_message(
                io::ErrorKind::Unsupported,
                "creation time is not available on this platform currently",
            ));
        }
        system_time(sec, nsec)
    }
}

/// std's `SystemTime::new(tv_sec, tv_nsec)` on Linux (sys/time/unix.rs
/// and `Timespec::new`): `tv_sec` seconds plus `tv_nsec` nanoseconds
/// after the epoch, or an InvalidData "invalid timestamp" error when
/// `tv_nsec` is outside `0..1_000_000_000`. The epoch arithmetic gives
/// std's (seconds, nanoseconds) pair for every i64 `tv_sec`.
fn system_time(tv_sec: i64, tv_nsec: i64) -> io::Result<SystemTime> {
    // On Apple OS a pre-epoch time is tv_sec <= 0 with a negative
    // tv_nsec (std's Timespec::new compensates to the common form).
    let (tv_sec, tv_nsec) = if cfg!(target_os = "macos") && tv_sec <= 0 && tv_sec > i64::MIN && tv_nsec < 0 && tv_nsec > -1_000_000_000 {
        (tv_sec - 1, tv_nsec + 1_000_000_000)
    } else {
        (tv_sec, tv_nsec)
    };
    if !(0..1_000_000_000).contains(&tv_nsec) {
        return Err(io::Error::__lust_adapter_simple_message(io::ErrorKind::InvalidData, "invalid timestamp"));
    }
    // No overflow: any i64 seconds plus less than one second fits.
    let whole = if tv_sec >= 0 {
        UNIX_EPOCH + Duration::from_secs(tv_sec as u64)
    } else {
        UNIX_EPOCH - Duration::from_secs(tv_sec.unsigned_abs())
    };
    Ok(whole + Duration::from_nanos(tv_nsec as u64))
}
