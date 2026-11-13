// `nix::sys::event`: kqueue and kevent over sysx (rel-mac), the
// forms ts-rust's fswatch/unix_bsd.rs uses. Upstream's module is
// BSD-only; this one exists on macOS only, as the calls do
// (sysx returns ENOSYS elsewhere). Flag and filter values are
// Darwin's (sys/event.h), as nix passes libc's. KEvent here is a
// plain record; upstream wraps libc's kevent struct, so its Debug
// form differs (README). `kevent64` and the free `kevent`
// functions are not here.

use crate::errno::Errno;
use crate::sys::time::Timespec;
use crate::Result;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};

/// Kqueue filter types (Darwin EVFILT_* values).
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(i16)]
#[non_exhaustive]
pub enum EventFilter {
    /// `EVFILT_READ`
    EVFILT_READ = -1,
    /// `EVFILT_WRITE`
    EVFILT_WRITE = -2,
    /// `EVFILT_AIO`
    EVFILT_AIO = -3,
    /// `EVFILT_VNODE`
    EVFILT_VNODE = -4,
    /// `EVFILT_PROC`
    EVFILT_PROC = -5,
    /// `EVFILT_SIGNAL`
    EVFILT_SIGNAL = -6,
    /// `EVFILT_TIMER`
    EVFILT_TIMER = -7,
    /// `EVFILT_MACHPORT`
    EVFILT_MACHPORT = -8,
    /// `EVFILT_FS`
    EVFILT_FS = -9,
    /// `EVFILT_USER`
    EVFILT_USER = -10,
    /// `EVFILT_VM`
    EVFILT_VM = -12,
    /// `EVFILT_EXCEPT`
    EVFILT_EXCEPT = -15,
}

impl TryFrom<i16> for EventFilter {
    type Error = Errno;

    fn try_from(v: i16) -> Result<EventFilter> {
        Ok(match v {
            -1 => EventFilter::EVFILT_READ,
            -2 => EventFilter::EVFILT_WRITE,
            -3 => EventFilter::EVFILT_AIO,
            -4 => EventFilter::EVFILT_VNODE,
            -5 => EventFilter::EVFILT_PROC,
            -6 => EventFilter::EVFILT_SIGNAL,
            -7 => EventFilter::EVFILT_TIMER,
            -8 => EventFilter::EVFILT_MACHPORT,
            -9 => EventFilter::EVFILT_FS,
            -10 => EventFilter::EVFILT_USER,
            -12 => EventFilter::EVFILT_VM,
            -15 => EventFilter::EVFILT_EXCEPT,
            _ => return Err(Errno::EINVAL),
        })
    }
}

/// Event flags (Darwin EV_* values).
/// No Debug name list, as in the other flag sets here (README).
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct EvFlags(u16);

impl EvFlags {
    /// `EV_ADD`
    pub const EV_ADD: EvFlags = EvFlags(0x0001);
    /// `EV_DELETE`
    pub const EV_DELETE: EvFlags = EvFlags(0x0002);
    /// `EV_ENABLE`
    pub const EV_ENABLE: EvFlags = EvFlags(0x0004);
    /// `EV_DISABLE`
    pub const EV_DISABLE: EvFlags = EvFlags(0x0008);
    /// `EV_ONESHOT`
    pub const EV_ONESHOT: EvFlags = EvFlags(0x0010);
    /// `EV_CLEAR`
    pub const EV_CLEAR: EvFlags = EvFlags(0x0020);
    /// `EV_RECEIPT`
    pub const EV_RECEIPT: EvFlags = EvFlags(0x0040);
    /// `EV_DISPATCH`
    pub const EV_DISPATCH: EvFlags = EvFlags(0x0080);
    /// `EV_FLAG0`
    pub const EV_FLAG0: EvFlags = EvFlags(0x1000);
    /// `EV_FLAG1`
    pub const EV_FLAG1: EvFlags = EvFlags(0x2000);
    /// `EV_EOF`
    pub const EV_EOF: EvFlags = EvFlags(0x8000);
    /// `EV_ERROR`
    pub const EV_ERROR: EvFlags = EvFlags(0x4000);
    /// `EV_OOBAND`
    pub const EV_OOBAND: EvFlags = EvFlags(0x2000);
    /// `EV_POLL`
    pub const EV_POLL: EvFlags = EvFlags(0x1000);

    /// No flags.
    pub const fn empty() -> EvFlags {
        EvFlags(0)
    }
    /// The raw bits.
    pub const fn bits(&self) -> u16 {
        self.0
    }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u16) -> EvFlags {
        EvFlags(bits)
    }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: EvFlags) -> bool {
        self.0 & other.0 == other.0
    }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl core::fmt::Debug for EvFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("EvFlags").field(&self.0).finish()
    }
}

impl core::ops::BitOr for EvFlags {
    type Output = EvFlags;
    fn bitor(self, other: EvFlags) -> EvFlags {
        EvFlags(self.0 | other.0)
    }
}

impl core::ops::BitOrAssign for EvFlags {
    fn bitor_assign(&mut self, other: EvFlags) {
        self.0 |= other.0;
    }
}

impl core::ops::BitAnd for EvFlags {
    type Output = EvFlags;
    fn bitand(self, other: EvFlags) -> EvFlags {
        EvFlags(self.0 & other.0)
    }
}

/// Filter-specific flags (Darwin NOTE_* values).
#[derive(Copy, Clone, Eq, Hash, PartialEq)]
pub struct FilterFlag(u32);

impl FilterFlag {
    /// `NOTE_TRIGGER`
    pub const NOTE_TRIGGER: FilterFlag = FilterFlag(0x0100_0000);
    /// `NOTE_FFNOP`
    pub const NOTE_FFNOP: FilterFlag = FilterFlag(0x0000_0000);
    /// `NOTE_FFAND`
    pub const NOTE_FFAND: FilterFlag = FilterFlag(0x4000_0000);
    /// `NOTE_FFOR`
    pub const NOTE_FFOR: FilterFlag = FilterFlag(0x8000_0000);
    /// `NOTE_FFCOPY`
    pub const NOTE_FFCOPY: FilterFlag = FilterFlag(0xc000_0000);
    /// `NOTE_FFCTRLMASK`
    pub const NOTE_FFCTRLMASK: FilterFlag = FilterFlag(0xc000_0000);
    /// `NOTE_FFLAGSMASK`
    pub const NOTE_FFLAGSMASK: FilterFlag = FilterFlag(0x00ff_ffff);
    /// `NOTE_LOWAT`
    pub const NOTE_LOWAT: FilterFlag = FilterFlag(0x0000_0001);
    /// `NOTE_DELETE`
    pub const NOTE_DELETE: FilterFlag = FilterFlag(0x0000_0001);
    /// `NOTE_WRITE`
    pub const NOTE_WRITE: FilterFlag = FilterFlag(0x0000_0002);
    /// `NOTE_OOB`
    pub const NOTE_OOB: FilterFlag = FilterFlag(0x0000_0002);
    /// `NOTE_EXTEND`
    pub const NOTE_EXTEND: FilterFlag = FilterFlag(0x0000_0004);
    /// `NOTE_ATTRIB`
    pub const NOTE_ATTRIB: FilterFlag = FilterFlag(0x0000_0008);
    /// `NOTE_LINK`
    pub const NOTE_LINK: FilterFlag = FilterFlag(0x0000_0010);
    /// `NOTE_RENAME`
    pub const NOTE_RENAME: FilterFlag = FilterFlag(0x0000_0020);
    /// `NOTE_REVOKE`
    pub const NOTE_REVOKE: FilterFlag = FilterFlag(0x0000_0040);
    /// `NOTE_NONE`
    pub const NOTE_NONE: FilterFlag = FilterFlag(0x0000_0080);
    /// `NOTE_EXIT`
    pub const NOTE_EXIT: FilterFlag = FilterFlag(0x8000_0000);
    /// `NOTE_FORK`
    pub const NOTE_FORK: FilterFlag = FilterFlag(0x4000_0000);
    /// `NOTE_EXEC`
    pub const NOTE_EXEC: FilterFlag = FilterFlag(0x2000_0000);
    /// `NOTE_SIGNAL`
    pub const NOTE_SIGNAL: FilterFlag = FilterFlag(0x0800_0000);
    /// `NOTE_EXITSTATUS`
    pub const NOTE_EXITSTATUS: FilterFlag = FilterFlag(0x0400_0000);
    /// `NOTE_PDATAMASK`
    pub const NOTE_PDATAMASK: FilterFlag = FilterFlag(0x000f_ffff);
    /// `NOTE_PCTRLMASK`
    pub const NOTE_PCTRLMASK: FilterFlag = FilterFlag(0xfff0_0000);
    /// `NOTE_SECONDS`
    pub const NOTE_SECONDS: FilterFlag = FilterFlag(0x0000_0001);
    /// `NOTE_USECONDS`
    pub const NOTE_USECONDS: FilterFlag = FilterFlag(0x0000_0002);
    /// `NOTE_NSECONDS`
    pub const NOTE_NSECONDS: FilterFlag = FilterFlag(0x0000_0004);
    /// `NOTE_ABSOLUTE`
    pub const NOTE_ABSOLUTE: FilterFlag = FilterFlag(0x0000_0008);
    /// `NOTE_LEEWAY`
    pub const NOTE_LEEWAY: FilterFlag = FilterFlag(0x0000_0010);
    /// `NOTE_CRITICAL`
    pub const NOTE_CRITICAL: FilterFlag = FilterFlag(0x0000_0020);
    /// `NOTE_BACKGROUND`
    pub const NOTE_BACKGROUND: FilterFlag = FilterFlag(0x0000_0040);

    /// No flags.
    pub const fn empty() -> FilterFlag {
        FilterFlag(0)
    }
    /// The raw bits.
    pub const fn bits(&self) -> u32 {
        self.0
    }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> FilterFlag {
        FilterFlag(bits)
    }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: FilterFlag) -> bool {
        self.0 & other.0 == other.0
    }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl core::fmt::Debug for FilterFlag {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("FilterFlag").field(&self.0).finish()
    }
}

impl core::ops::BitOr for FilterFlag {
    type Output = FilterFlag;
    fn bitor(self, other: FilterFlag) -> FilterFlag {
        FilterFlag(self.0 | other.0)
    }
}

impl core::ops::BitOrAssign for FilterFlag {
    fn bitor_assign(&mut self, other: FilterFlag) {
        self.0 |= other.0;
    }
}

impl core::ops::BitAnd for FilterFlag {
    type Output = FilterFlag;
    fn bitand(self, other: FilterFlag) -> FilterFlag {
        FilterFlag(self.0 & other.0)
    }
}

/// One kevent record (48 bytes on the sysx wire, sysx.h).
const REC: usize = 48;

/// The u64 at byte `at` of a record (little-endian, native here).
fn rec_u64(rec: &[u8], at: usize) -> u64 {
    let mut v: u64 = 0;
    for k in 0..8 {
        v |= (rec[at + k] as u64) << (8 * k);
    }
    v
}

/// An event for use with `Kqueue::kevent`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct KEvent {
    ident: usize,
    filter: i16,
    flags: EvFlags,
    fflags: FilterFlag,
    data: isize,
    udata: isize,
}

impl KEvent {
    /// A new event for the changelist of `Kqueue::kevent`.
    pub fn new(ident: usize, filter: EventFilter, flags: EvFlags, fflags: FilterFlag, data: isize, udata: isize) -> KEvent {
        KEvent { ident, filter: filter as i16, flags, fflags, data, udata }
    }

    /// The identifier (often a file descriptor).
    pub fn ident(&self) -> usize {
        self.ident
    }

    /// The filter, or an error for one nix does not name.
    pub fn filter(&self) -> Result<EventFilter> {
        EventFilter::try_from(self.filter)
    }

    /// The event flags.
    pub fn flags(&self) -> EvFlags {
        self.flags
    }

    /// The filter-specific flags.
    pub fn fflags(&self) -> FilterFlag {
        self.fflags
    }

    /// The filter-specific data value.
    pub fn data(&self) -> isize {
        self.data
    }

    /// The user value passed through the kernel unchanged.
    pub fn udata(&self) -> isize {
        self.udata
    }

    /// This event as one sysx record appended to `out`.
    fn pack(&self, out: &mut Vec<u8>) {
        for v in [self.ident as u64, self.filter as i64 as u64, self.flags.bits() as u64, self.fflags.bits() as u64, self.data as u64, self.udata as u64] {
            out.extend_from_slice(&v.to_ne_bytes());
        }
    }

    /// The event in one sysx record.
    fn unpack(rec: &[u8]) -> KEvent {
        KEvent {
            ident: rec_u64(rec, 0) as usize,
            filter: rec_u64(rec, 8) as i64 as i16,
            flags: EvFlags::from_bits_retain(rec_u64(rec, 16) as u16),
            fflags: FilterFlag::from_bits_retain(rec_u64(rec, 24) as u32),
            data: rec_u64(rec, 32) as i64 as isize,
            udata: rec_u64(rec, 40) as i64 as isize,
        }
    }
}

/// A kernel event queue.
#[derive(Debug)]
pub struct Kqueue(OwnedFd);

impl AsFd for Kqueue {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl From<Kqueue> for OwnedFd {
    fn from(value: Kqueue) -> OwnedFd {
        value.0
    }
}

impl Kqueue {
    /// Create a new kernel event queue.
    pub fn new() -> Result<Kqueue> {
        let fd = core::intrinsics::sys_x_kqueue();
        if fd < 0 {
            Err(Errno::from_raw(-fd))
        } else {
            Ok(Kqueue(OwnedFd::from_raw_fd(fd)))
        }
    }

    /// Register the changelist and return pending events into the
    /// eventlist, as kevent(2) does: the count placed there, or the
    /// error. A changelist error with room in the eventlist comes
    /// back as an EV_ERROR event, as the kernel reports it.
    pub fn kevent(&self, changelist: &[KEvent], eventlist: &mut [KEvent], timeout_opt: Option<Timespec>) -> Result<usize> {
        let mut changes: Vec<u8> = Vec::with_capacity(changelist.len() * REC);
        for ev in changelist {
            ev.pack(&mut changes);
        }
        let mut events: Vec<u8> = vec![0u8; eventlist.len() * REC];
        let (has_timeout, sec, nsec) = match timeout_opt {
            Some(ts) => (1, ts.tv_sec, ts.tv_nsec),
            None => (0, 0, 0),
        };
        let n = core::intrinsics::sys_x_kevent(self.0.as_fd().as_raw_fd(), &changes, &mut events, has_timeout, sec, nsec);
        if n < 0 {
            return Err(Errno::from_raw(-n));
        }
        for (i, slot) in eventlist.iter_mut().take(n as usize).enumerate() {
            *slot = KEvent::unpack(&events[i * REC..(i + 1) * REC]);
        }
        Ok(n as usize)
    }
}
