// `rustix::event`: poll. The shim uses poll(2) with a
// millisecond timeout rounded up; upstream uses ppoll, so a
// timed wait can end up to a millisecond later here (a wait
// with no ready fd, the only form whose length a program can
// print). README lists this.

use crate::fd::{AsFd, AsRawFd, BorrowedFd};
pub use crate::timespec::{Nsecs, Secs, Timespec};
use crate::io::{Errno, Result};

/// Poll event flags (`POLL*`, Linux values).
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PollFlags(u16);

impl PollFlags {
    /// `POLLIN`
    pub const IN: PollFlags = PollFlags(0x001);
    /// `POLLPRI`
    pub const PRI: PollFlags = PollFlags(0x002);
    /// `POLLOUT`
    pub const OUT: PollFlags = PollFlags(0x004);
    /// `POLLERR`
    pub const ERR: PollFlags = PollFlags(0x008);
    /// `POLLHUP`
    pub const HUP: PollFlags = PollFlags(0x010);
    /// `POLLNVAL`
    pub const NVAL: PollFlags = PollFlags(0x020);
    /// `POLLRDNORM`
    pub const RDNORM: PollFlags = PollFlags(0x040);
    /// `POLLRDBAND`
    pub const RDBAND: PollFlags = PollFlags(0x080);
    /// `POLLWRNORM`
    pub const WRNORM: PollFlags = PollFlags(0x100);
    /// `POLLWRBAND`
    pub const WRBAND: PollFlags = PollFlags(0x200);
    /// `POLLRDHUP`
    pub const RDHUP: PollFlags = PollFlags(0x2000);

    pub const fn empty() -> PollFlags {
        PollFlags(0)
    }
    pub const fn bits(&self) -> u16 {
        self.0
    }
    pub const fn from_bits_retain(bits: u16) -> PollFlags {
        PollFlags(bits)
    }
    pub const fn contains(&self, other: PollFlags) -> bool {
        self.0 & other.0 == other.0
    }
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl core::ops::BitOr for PollFlags {
    type Output = PollFlags;
    fn bitor(self, other: PollFlags) -> PollFlags {
        PollFlags(self.0 | other.0)
    }
}

impl core::ops::BitOrAssign for PollFlags {
    fn bitor_assign(&mut self, other: PollFlags) {
        self.0 |= other.0;
    }
}

/// One fd to poll, with its requested and returned events.
// Upstream: `Debug, Clone` (no Copy); its Debug prints the raw `u16`
// events, which this adapter does not, so there is none here.
#[derive(Clone)]
pub struct PollFd<'fd> {
    fd: BorrowedFd<'fd>,
    events: PollFlags,
    revents: PollFlags,
}

impl<'fd> PollFd<'fd> {
    /// A poll entry for `fd` (anything with an fd) and `events`.
    pub fn new<Fd: AsFd>(fd: &'fd Fd, events: PollFlags) -> PollFd<'fd> {
        PollFd { fd: fd.as_fd(), events, revents: PollFlags::empty() }
    }

    /// A poll entry from a borrowed fd.
    pub fn from_borrowed_fd(fd: BorrowedFd<'fd>, events: PollFlags) -> PollFd<'fd> {
        PollFd { fd, events, revents: PollFlags::empty() }
    }

    /// The events `poll` returned for this fd.
    pub fn revents(&self) -> PollFlags {
        self.revents
    }
}

/// `poll(fds, timeout)`: waits and fills each entry's revents.
/// Returns the number of ready entries.
pub fn poll(fds: &mut [PollFd<'_>], timeout: Option<&Timespec>) -> Result<usize> {
    let mut recs: Vec<u8> = Vec::with_capacity(fds.len() * 8);
    for f in fds.iter() {
        let v = (f.fd.as_raw_fd() as u32 as u64) | ((f.events.bits() as u64) << 32);
        for k in 0..8 {
            recs.push((v >> (8 * k)) as u8);
        }
    }
    let (sec, nsec) = match timeout {
        Some(t) => (t.tv_sec, t.tv_nsec),
        None => (-1, 0),
    };
    let r = core::intrinsics::sys_x_poll(&mut recs, sec, nsec);
    if r < 0 {
        return Err(Errno::from_raw_os_error(-r));
    }
    for (i, f) in fds.iter_mut().enumerate() {
        let rev = (recs[i * 8 + 6] as u16) | ((recs[i * 8 + 7] as u16) << 8);
        f.revents = PollFlags::from_bits_retain(rev);
    }
    Ok(r as usize)
}
