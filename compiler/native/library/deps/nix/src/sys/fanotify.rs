// `nix::sys::fanotify`: fanotify group creation and marks,
// over sysx (tf-adapters F2). Flag values are the Linux
// kernel values nix passes to libc. Not here: `read_events`,
// `FanotifyEvent`, `write_response` and `FanotifyResponse`
// (ts-rust reads raw event bytes with rustix::io::read);
// `Response` consts are here for the mask arithmetic callers
// do. Flag Debug forms are the derived tuple forms, not
// upstream's name lists (README).

use crate::errno::Errno;
use crate::{NixPath, Result};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};

/// Mask for defining which events shall be listened with
/// `Fanotify::mark` and for querying notifications.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct MaskFlags(u64);

impl MaskFlags {
    /// `FAN_ACCESS`
    pub const FAN_ACCESS: MaskFlags = MaskFlags(0x0000_0001);
    /// `FAN_MODIFY`
    pub const FAN_MODIFY: MaskFlags = MaskFlags(0x0000_0002);
    /// `FAN_ATTRIB`
    pub const FAN_ATTRIB: MaskFlags = MaskFlags(0x0000_0004);
    /// `FAN_CLOSE_WRITE`
    pub const FAN_CLOSE_WRITE: MaskFlags = MaskFlags(0x0000_0008);
    /// `FAN_CLOSE_NOWRITE`
    pub const FAN_CLOSE_NOWRITE: MaskFlags = MaskFlags(0x0000_0010);
    /// `FAN_OPEN`
    pub const FAN_OPEN: MaskFlags = MaskFlags(0x0000_0020);
    /// `FAN_MOVED_FROM`
    pub const FAN_MOVED_FROM: MaskFlags = MaskFlags(0x0000_0040);
    /// `FAN_MOVED_TO`
    pub const FAN_MOVED_TO: MaskFlags = MaskFlags(0x0000_0080);
    /// `FAN_CREATE`
    pub const FAN_CREATE: MaskFlags = MaskFlags(0x0000_0100);
    /// `FAN_DELETE`
    pub const FAN_DELETE: MaskFlags = MaskFlags(0x0000_0200);
    /// `FAN_DELETE_SELF`
    pub const FAN_DELETE_SELF: MaskFlags = MaskFlags(0x0000_0400);
    /// `FAN_MOVE_SELF`
    pub const FAN_MOVE_SELF: MaskFlags = MaskFlags(0x0000_0800);
    /// `FAN_OPEN_EXEC`
    pub const FAN_OPEN_EXEC: MaskFlags = MaskFlags(0x0000_1000);
    /// `FAN_Q_OVERFLOW`
    pub const FAN_Q_OVERFLOW: MaskFlags = MaskFlags(0x0000_4000);
    /// `FAN_FS_ERROR`
    pub const FAN_FS_ERROR: MaskFlags = MaskFlags(0x0000_8000);
    /// `FAN_OPEN_PERM`
    pub const FAN_OPEN_PERM: MaskFlags = MaskFlags(0x0001_0000);
    /// `FAN_ACCESS_PERM`
    pub const FAN_ACCESS_PERM: MaskFlags = MaskFlags(0x0002_0000);
    /// `FAN_OPEN_EXEC_PERM`
    pub const FAN_OPEN_EXEC_PERM: MaskFlags = MaskFlags(0x0004_0000);
    /// `FAN_EVENT_ON_CHILD`
    pub const FAN_EVENT_ON_CHILD: MaskFlags = MaskFlags(0x0800_0000);
    /// `FAN_RENAME`
    pub const FAN_RENAME: MaskFlags = MaskFlags(0x1000_0000);
    /// `FAN_ONDIR`
    pub const FAN_ONDIR: MaskFlags = MaskFlags(0x4000_0000);
    /// `FAN_CLOSE`
    pub const FAN_CLOSE: MaskFlags = MaskFlags(0x0000_0018);
    /// `FAN_MOVE`
    pub const FAN_MOVE: MaskFlags = MaskFlags(0x0000_00c0);

    /// No flags.
    pub const fn empty() -> MaskFlags { MaskFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u64 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u64) -> MaskFlags { MaskFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: MaskFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for MaskFlags {
    type Output = MaskFlags;
    fn bitor(self, other: MaskFlags) -> MaskFlags { MaskFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for MaskFlags {
    fn bitor_assign(&mut self, other: MaskFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for MaskFlags {
    type Output = MaskFlags;
    fn bitand(self, other: MaskFlags) -> MaskFlags { MaskFlags(self.0 & other.0) }
}

/// Configuration options for `Fanotify::init`.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct InitFlags(u32);

impl InitFlags {
    /// `FAN_CLOEXEC`
    pub const FAN_CLOEXEC: InitFlags = InitFlags(0x0000_0001);
    /// `FAN_NONBLOCK`
    pub const FAN_NONBLOCK: InitFlags = InitFlags(0x0000_0002);
    /// `FAN_CLASS_NOTIF`
    pub const FAN_CLASS_NOTIF: InitFlags = InitFlags(0x0000_0000);
    /// `FAN_CLASS_CONTENT`
    pub const FAN_CLASS_CONTENT: InitFlags = InitFlags(0x0000_0004);
    /// `FAN_CLASS_PRE_CONTENT`
    pub const FAN_CLASS_PRE_CONTENT: InitFlags = InitFlags(0x0000_0008);
    /// `FAN_UNLIMITED_QUEUE`
    pub const FAN_UNLIMITED_QUEUE: InitFlags = InitFlags(0x0000_0010);
    /// `FAN_UNLIMITED_MARKS`
    pub const FAN_UNLIMITED_MARKS: InitFlags = InitFlags(0x0000_0020);
    /// `FAN_REPORT_PIDFD`
    pub const FAN_REPORT_PIDFD: InitFlags = InitFlags(0x0000_0080);
    /// `FAN_REPORT_TID`
    pub const FAN_REPORT_TID: InitFlags = InitFlags(0x0000_0100);

    /// No flags.
    pub const fn empty() -> InitFlags { InitFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> InitFlags { InitFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: InitFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for InitFlags {
    type Output = InitFlags;
    fn bitor(self, other: InitFlags) -> InitFlags { InitFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for InitFlags {
    fn bitor_assign(&mut self, other: InitFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for InitFlags {
    type Output = InitFlags;
    fn bitand(self, other: InitFlags) -> InitFlags { InitFlags(self.0 & other.0) }
}

/// File status flags for fanotify events file descriptors.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EventFFlags(u32);

impl EventFFlags {
    /// `O_RDONLY`
    pub const O_RDONLY: EventFFlags = EventFFlags(0);
    /// `O_WRONLY`
    pub const O_WRONLY: EventFFlags = EventFFlags(1);
    /// `O_RDWR`
    pub const O_RDWR: EventFFlags = EventFFlags(2);
    /// `O_LARGEFILE`
    pub const O_LARGEFILE: EventFFlags = EventFFlags(0o100000);
    /// `O_CLOEXEC`
    pub const O_CLOEXEC: EventFFlags = EventFFlags(0o2000000);
    /// `O_APPEND`
    pub const O_APPEND: EventFFlags = EventFFlags(0o2000);
    /// `O_DSYNC`
    pub const O_DSYNC: EventFFlags = EventFFlags(0o10000);
    /// `O_NOATIME`
    pub const O_NOATIME: EventFFlags = EventFFlags(0o1000000);
    /// `O_NONBLOCK`
    pub const O_NONBLOCK: EventFFlags = EventFFlags(0o4000);
    /// `O_SYNC`
    pub const O_SYNC: EventFFlags = EventFFlags(0o4010000);

    /// No flags.
    pub const fn empty() -> EventFFlags { EventFFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> EventFFlags { EventFFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: EventFFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for EventFFlags {
    type Output = EventFFlags;
    fn bitor(self, other: EventFFlags) -> EventFFlags { EventFFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for EventFFlags {
    fn bitor_assign(&mut self, other: EventFFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for EventFFlags {
    type Output = EventFFlags;
    fn bitand(self, other: EventFFlags) -> EventFFlags { EventFFlags(self.0 & other.0) }
}

/// Configuration options for `Fanotify::mark`.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct MarkFlags(u32);

impl MarkFlags {
    /// `FAN_MARK_ADD`
    pub const FAN_MARK_ADD: MarkFlags = MarkFlags(0x0000_0001);
    /// `FAN_MARK_REMOVE`
    pub const FAN_MARK_REMOVE: MarkFlags = MarkFlags(0x0000_0002);
    /// `FAN_MARK_DONT_FOLLOW`
    pub const FAN_MARK_DONT_FOLLOW: MarkFlags = MarkFlags(0x0000_0004);
    /// `FAN_MARK_ONLYDIR`
    pub const FAN_MARK_ONLYDIR: MarkFlags = MarkFlags(0x0000_0008);
    /// `FAN_MARK_IGNORED_MASK`
    pub const FAN_MARK_IGNORED_MASK: MarkFlags = MarkFlags(0x0000_0020);
    /// `FAN_MARK_IGNORED_SURV_MODIFY`
    pub const FAN_MARK_IGNORED_SURV_MODIFY: MarkFlags = MarkFlags(0x0000_0040);
    /// `FAN_MARK_FLUSH`
    pub const FAN_MARK_FLUSH: MarkFlags = MarkFlags(0x0000_0080);
    /// `FAN_MARK_EVICTABLE`
    pub const FAN_MARK_EVICTABLE: MarkFlags = MarkFlags(0x0000_0200);
    /// `FAN_MARK_IGNORE`
    pub const FAN_MARK_IGNORE: MarkFlags = MarkFlags(0x0000_0400);
    /// `FAN_MARK_INODE`
    pub const FAN_MARK_INODE: MarkFlags = MarkFlags(0x0000_0000);
    /// `FAN_MARK_MOUNT`
    pub const FAN_MARK_MOUNT: MarkFlags = MarkFlags(0x0000_0010);
    /// `FAN_MARK_FILESYSTEM`
    pub const FAN_MARK_FILESYSTEM: MarkFlags = MarkFlags(0x0000_0100);
    /// `FAN_MARK_IGNORE_SURV`
    pub const FAN_MARK_IGNORE_SURV: MarkFlags = MarkFlags(0x0000_0440);

    /// No flags.
    pub const fn empty() -> MarkFlags { MarkFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> MarkFlags { MarkFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: MarkFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for MarkFlags {
    type Output = MarkFlags;
    fn bitor(self, other: MarkFlags) -> MarkFlags { MarkFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for MarkFlags {
    fn bitor_assign(&mut self, other: MarkFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for MarkFlags {
    type Output = MarkFlags;
    fn bitand(self, other: MarkFlags) -> MarkFlags { MarkFlags(self.0 & other.0) }
}

/// Compile version number of fanotify API.
pub const FANOTIFY_METADATA_VERSION: u8 = 3;

/// A fanotify group.
#[derive(Debug)]
pub struct Fanotify {
    fd: OwnedFd,
}

impl Fanotify {
    /// Initialize a new fanotify group (`fanotify_init(2)`).
    pub fn init(flags: InitFlags, event_f_flags: EventFFlags) -> Result<Fanotify> {
        let fd = core::intrinsics::sys_x_fanotify_init(flags.bits(), event_f_flags.bits());
        if fd < 0 {
            Err(Errno::from_raw(-fd))
        } else {
            Ok(Fanotify { fd: OwnedFd::from_raw_fd(fd) })
        }
    }

    /// Add, remove, or modify a fanotify mark (`fanotify_mark(2)`).
    /// A path with an interior NUL is EINVAL, as upstream's
    /// `NixPath` conversion gives.
    pub fn mark<Fd: AsFd, P: ?Sized + NixPath>(&self, flags: MarkFlags, mask: MaskFlags, dirfd: Fd, path: Option<&P>) -> Result<()> {
        let dir_raw = dirfd.as_fd().as_raw_fd();
        let e = match path {
            Some(path) => path.with_nix_path(|cstr| {
                core::intrinsics::sys_x_fanotify_mark(self.fd.as_raw_fd(), flags.bits(), mask.bits(), dir_raw, 1, cstr.to_bytes())
            })?,
            None => core::intrinsics::sys_x_fanotify_mark(self.fd.as_raw_fd(), flags.bits(), mask.bits(), dir_raw, 0, &[]),
        };
        if e != 0 {
            return Err(Errno::from_raw(e));
        }
        Ok(())
    }
}

impl FromRawFd for Fanotify {
    fn from_raw_fd(fd: RawFd) -> Fanotify {
        Fanotify { fd: OwnedFd::from_raw_fd(fd) }
    }
}

impl AsFd for Fanotify {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl AsRawFd for Fanotify {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl From<Fanotify> for OwnedFd {
    fn from(value: Fanotify) -> OwnedFd {
        value.fd
    }
}
