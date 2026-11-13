// `rustix::fs`: the flag types and the file calls ts-rust's
// core files use, over sysx. Flag Debug forms are the derived
// tuple forms, not upstream's name lists (README).

use crate::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};
use crate::io::{Errno, Result};
use crate::path::Arg;
use crate::{rec_i64, rec_u64};

/// `AT_FDCWD`: the current working directory as a dirfd.
pub const CWD: BorrowedFd<'static> = BorrowedFd::borrow_raw(-100);

/// The path bytes of `path`, or EINVAL for an interior NUL.
fn path_bytes<P: Arg>(path: &P) -> Result<Vec<u8>> {
    let bytes = path.__lust_path_bytes();
    if bytes.contains(&0) {
        return Err(Errno::INVAL);
    }
    Ok(bytes)
}

/// `OFlags` flags.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct OFlags(u32);

impl OFlags {
    pub const RDONLY: OFlags = OFlags(0);
    pub const WRONLY: OFlags = OFlags(1);
    pub const RDWR: OFlags = OFlags(2);
    pub const ACCMODE: OFlags = OFlags(3);
    pub const CREATE: OFlags = OFlags(0o100);
    pub const EXCL: OFlags = OFlags(0o200);
    pub const NOCTTY: OFlags = OFlags(0o400);
    pub const TRUNC: OFlags = OFlags(0o1000);
    pub const APPEND: OFlags = OFlags(0o2000);
    pub const NONBLOCK: OFlags = OFlags(0o4000);
    pub const DSYNC: OFlags = OFlags(0o10000);
    pub const SYNC: OFlags = OFlags(0o4010000);
    // O_DIRECTORY and O_NOFOLLOW differ between x86_64 and the
    // asm-generic arches (aarch64 here); the bits go to openat raw.
    // Another arch gets neither const, so its use rejects.
    #[cfg(target_arch = "x86_64")]
    pub const DIRECTORY: OFlags = OFlags(0o200000);
    #[cfg(target_arch = "aarch64")]
    pub const DIRECTORY: OFlags = OFlags(0o40000);
    #[cfg(target_arch = "x86_64")]
    pub const NOFOLLOW: OFlags = OFlags(0o400000);
    #[cfg(target_arch = "aarch64")]
    pub const NOFOLLOW: OFlags = OFlags(0o100000);
    pub const CLOEXEC: OFlags = OFlags(0o2000000);
    pub const PATH: OFlags = OFlags(0o10000000);

    /// No flags.
    pub const fn empty() -> OFlags { OFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> OFlags { OFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: OFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for OFlags {
    type Output = OFlags;
    fn bitor(self, other: OFlags) -> OFlags { OFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for OFlags {
    fn bitor_assign(&mut self, other: OFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for OFlags {
    type Output = OFlags;
    fn bitand(self, other: OFlags) -> OFlags { OFlags(self.0 & other.0) }
}

/// `Mode` flags.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Mode(u32);

impl Mode {
    pub const RWXU: Mode = Mode(0o700);
    pub const RUSR: Mode = Mode(0o400);
    pub const WUSR: Mode = Mode(0o200);
    pub const XUSR: Mode = Mode(0o100);
    pub const RWXG: Mode = Mode(0o070);
    pub const RGRP: Mode = Mode(0o040);
    pub const WGRP: Mode = Mode(0o020);
    pub const XGRP: Mode = Mode(0o010);
    pub const RWXO: Mode = Mode(0o007);
    pub const ROTH: Mode = Mode(0o004);
    pub const WOTH: Mode = Mode(0o002);
    pub const XOTH: Mode = Mode(0o001);
    pub const SUID: Mode = Mode(0o4000);
    pub const SGID: Mode = Mode(0o2000);
    pub const SVTX: Mode = Mode(0o1000);

    /// No flags.
    pub const fn empty() -> Mode { Mode(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Mode { Mode(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: Mode) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for Mode {
    type Output = Mode;
    fn bitor(self, other: Mode) -> Mode { Mode(self.0 | other.0) }
}

impl core::ops::BitOrAssign for Mode {
    fn bitor_assign(&mut self, other: Mode) { self.0 |= other.0; }
}

impl core::ops::BitAnd for Mode {
    type Output = Mode;
    fn bitand(self, other: Mode) -> Mode { Mode(self.0 & other.0) }
}

/// `AtFlags` flags.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct AtFlags(u32);

impl AtFlags {
    pub const SYMLINK_NOFOLLOW: AtFlags = AtFlags(0x100);
    pub const EACCESS: AtFlags = AtFlags(0x200);
    pub const REMOVEDIR: AtFlags = AtFlags(0x200);
    pub const SYMLINK_FOLLOW: AtFlags = AtFlags(0x400);
    pub const NO_AUTOMOUNT: AtFlags = AtFlags(0x800);
    pub const EMPTY_PATH: AtFlags = AtFlags(0x1000);
    pub const STATX_FORCE_SYNC: AtFlags = AtFlags(0x2000);
    pub const STATX_DONT_SYNC: AtFlags = AtFlags(0x4000);
    pub const STATX_SYNC_AS_STAT: AtFlags = AtFlags(0);

    /// No flags.
    pub const fn empty() -> AtFlags { AtFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> AtFlags { AtFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: AtFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for AtFlags {
    type Output = AtFlags;
    fn bitor(self, other: AtFlags) -> AtFlags { AtFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for AtFlags {
    fn bitor_assign(&mut self, other: AtFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for AtFlags {
    type Output = AtFlags;
    fn bitand(self, other: AtFlags) -> AtFlags { AtFlags(self.0 & other.0) }
}

/// `Access` flags.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Access(u32);

impl Access {
    pub const EXISTS: Access = Access(0);
    pub const EXEC_OK: Access = Access(1);
    pub const WRITE_OK: Access = Access(2);
    pub const READ_OK: Access = Access(4);

    /// No flags.
    pub const fn empty() -> Access { Access(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Access { Access(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: Access) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for Access {
    type Output = Access;
    fn bitor(self, other: Access) -> Access { Access(self.0 | other.0) }
}

impl core::ops::BitOrAssign for Access {
    fn bitor_assign(&mut self, other: Access) { self.0 |= other.0; }
}

impl core::ops::BitAnd for Access {
    type Output = Access;
    fn bitand(self, other: Access) -> Access { Access(self.0 & other.0) }
}

/// `StatxFlags` flags.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct StatxFlags(u32);

impl StatxFlags {
    pub const TYPE: StatxFlags = StatxFlags(0x001);
    pub const MODE: StatxFlags = StatxFlags(0x002);
    pub const NLINK: StatxFlags = StatxFlags(0x004);
    pub const UID: StatxFlags = StatxFlags(0x008);
    pub const GID: StatxFlags = StatxFlags(0x010);
    pub const ATIME: StatxFlags = StatxFlags(0x020);
    pub const MTIME: StatxFlags = StatxFlags(0x040);
    pub const CTIME: StatxFlags = StatxFlags(0x080);
    pub const INO: StatxFlags = StatxFlags(0x100);
    pub const SIZE: StatxFlags = StatxFlags(0x200);
    pub const BLOCKS: StatxFlags = StatxFlags(0x400);
    pub const BASIC_STATS: StatxFlags = StatxFlags(0x7ff);
    pub const BTIME: StatxFlags = StatxFlags(0x800);
    pub const MNT_ID: StatxFlags = StatxFlags(0x1000);
    pub const DIOALIGN: StatxFlags = StatxFlags(0x2000);
    pub const ALL: StatxFlags = StatxFlags(0x3fff);

    /// No flags.
    pub const fn empty() -> StatxFlags { StatxFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> StatxFlags { StatxFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: StatxFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for StatxFlags {
    type Output = StatxFlags;
    fn bitor(self, other: StatxFlags) -> StatxFlags { StatxFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for StatxFlags {
    fn bitor_assign(&mut self, other: StatxFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for StatxFlags {
    type Output = StatxFlags;
    fn bitand(self, other: StatxFlags) -> StatxFlags { StatxFlags(self.0 & other.0) }
}


impl Mode {
    /// The permission bits of a raw mode.
    pub const fn from_raw_mode(mode: u32) -> Mode {
        Mode(mode & 0o7777)
    }

    /// The raw permission bits.
    pub const fn as_raw_mode(self) -> u32 {
        self.0
    }
}

/// A file type, from the mode bits or a directory entry. Upstream has
/// no `Hash`; its discriminants are the `S_IF*` bits (an `as` cast
/// reads them).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FileType {
    /// A regular file.
    RegularFile = 0o100000,
    /// A directory.
    Directory = 0o040000,
    /// A symbolic link.
    Symlink = 0o120000,
    /// A FIFO (named pipe).
    Fifo = 0o010000,
    /// A socket.
    Socket = 0o140000,
    /// A character device.
    CharacterDevice = 0o020000,
    /// A block device.
    BlockDevice = 0o060000,
    /// Unknown (the file system gave no type).
    Unknown,
}

impl FileType {
    /// The file type of a raw `st_mode`.
    pub const fn from_raw_mode(mode: u32) -> FileType {
        match mode & 0o170000 {
            0o100000 => FileType::RegularFile,
            0o040000 => FileType::Directory,
            0o120000 => FileType::Symlink,
            0o010000 => FileType::Fifo,
            0o140000 => FileType::Socket,
            0o020000 => FileType::CharacterDevice,
            0o060000 => FileType::BlockDevice,
            _ => FileType::Unknown,
        }
    }

    /// The `S_IF*` raw mode bits of this type (0 for Unknown).
    pub const fn as_raw_mode(self) -> u32 {
        match self {
            FileType::RegularFile => 0o100000,
            FileType::Directory => 0o040000,
            FileType::Symlink => 0o120000,
            FileType::Fifo => 0o010000,
            FileType::Socket => 0o140000,
            FileType::CharacterDevice => 0o020000,
            FileType::BlockDevice => 0o060000,
            FileType::Unknown => 0,
        }
    }
}

/// The result of `stat` and kin (Linux fields).
// Upstream: `Debug, Copy, Clone`, with spare and newer fields this
// adapter does not carry, so its Debug differs: none here.
#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: u64,
    pub st_mtime: i64,
    pub st_mtime_nsec: u64,
    pub st_ctime: i64,
    pub st_ctime_nsec: u64,
}

/// Decodes one sysx stat record.
fn stat_of(rec: &[u8]) -> Stat {
    Stat {
        st_dev: rec_u64(rec, 0),
        st_ino: rec_u64(rec, 1),
        st_nlink: rec_u64(rec, 2),
        st_mode: rec_u64(rec, 3) as u32,
        st_uid: rec_u64(rec, 4) as u32,
        st_gid: rec_u64(rec, 5) as u32,
        st_rdev: rec_u64(rec, 6),
        st_size: rec_i64(rec, 7),
        st_blksize: rec_i64(rec, 8),
        st_blocks: rec_i64(rec, 9),
        st_atime: rec_i64(rec, 10),
        st_atime_nsec: rec_u64(rec, 11),
        st_mtime: rec_i64(rec, 12),
        st_mtime_nsec: rec_u64(rec, 13),
        st_ctime: rec_i64(rec, 14),
        st_ctime_nsec: rec_u64(rec, 15),
    }
}

/// A timestamp within `Statx`.
// Upstream: `Debug, Copy, Clone`, with spare and newer fields this
// adapter does not carry, so its Debug differs: none here.
#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct StatxTimestamp {
    pub tv_sec: i64,
    pub tv_nsec: u32,
}

/// The result of `statx`.
// Upstream: `Debug, Copy, Clone`, with spare and newer fields this
// adapter does not carry, so its Debug differs: none here.
#[derive(Copy, Clone)]
#[non_exhaustive]
pub struct Statx {
    pub stx_mask: u32,
    pub stx_blksize: u32,
    pub stx_nlink: u32,
    pub stx_uid: u32,
    pub stx_gid: u32,
    pub stx_mode: u16,
    pub stx_ino: u64,
    pub stx_size: u64,
    pub stx_blocks: u64,
    pub stx_atime: StatxTimestamp,
    pub stx_mtime: StatxTimestamp,
    pub stx_ctime: StatxTimestamp,
    pub stx_btime: StatxTimestamp,
    pub stx_rdev_major: u32,
    pub stx_rdev_minor: u32,
    pub stx_dev_major: u32,
    pub stx_dev_minor: u32,
}

/// Decodes one sysx statx record.
fn statx_of(rec: &[u8]) -> Statx {
    let ts = |sec: usize, nsec: usize| StatxTimestamp { tv_sec: rec_i64(rec, sec), tv_nsec: rec_u64(rec, nsec) as u32 };
    Statx {
        stx_mask: rec_u64(rec, 0) as u32,
        stx_blksize: rec_u64(rec, 1) as u32,
        stx_nlink: rec_u64(rec, 3) as u32,
        stx_uid: rec_u64(rec, 4) as u32,
        stx_gid: rec_u64(rec, 5) as u32,
        stx_mode: rec_u64(rec, 6) as u16,
        stx_ino: rec_u64(rec, 7),
        stx_size: rec_u64(rec, 8),
        stx_blocks: rec_u64(rec, 9),
        stx_atime: ts(11, 12),
        stx_mtime: ts(13, 14),
        stx_ctime: ts(15, 16),
        stx_btime: ts(17, 18),
        stx_rdev_major: rec_u64(rec, 19) as u32,
        stx_rdev_minor: rec_u64(rec, 20) as u32,
        stx_dev_major: rec_u64(rec, 21) as u32,
        stx_dev_minor: rec_u64(rec, 22) as u32,
    }
}

pub use crate::timespec::{Nsecs, Secs, Timespec};

/// `UTIME_NOW` for a `Timespec` nanosecond field.
pub const UTIME_NOW: i64 = (1 << 30) - 1;
/// `UTIME_OMIT` for a `Timespec` nanosecond field.
pub const UTIME_OMIT: i64 = (1 << 30) - 2;

/// The two times of `utimensat`.
// Upstream: `Debug, Clone` only.
#[derive(Clone, Debug)]
pub struct Timestamps {
    pub last_access: Timespec,
    pub last_modification: Timespec,
}

/// `open(path, oflags, mode)`.
pub fn open<P: Arg>(path: P, flags: OFlags, mode: Mode) -> Result<OwnedFd> {
    openat(CWD, path, flags, mode)
}

/// `openat(dirfd, path, oflags, mode)`.
pub fn openat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, flags: OFlags, mode: Mode) -> Result<OwnedFd> {
    let bytes = path_bytes(&path)?;
    let fd = core::intrinsics::sys_x_openat(dirfd.as_fd().as_raw_fd(), &bytes, flags.bits() as i32, mode.as_raw_mode());
    if fd < 0 {
        Err(Errno::from_raw_os_error(-fd))
    } else {
        Ok(OwnedFd::from_raw_fd(fd))
    }
}

/// `fstat(fd)`.
pub fn fstat<Fd: AsFd>(fd: Fd) -> Result<Stat> {
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_fstat(fd.as_fd().as_raw_fd(), &mut rec);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(stat_of(&rec))
}

/// `fstatat(dirfd, path, flags)`.
pub fn statat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, flags: AtFlags) -> Result<Stat> {
    let bytes = path_bytes(&path)?;
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_statat(dirfd.as_fd().as_raw_fd(), &bytes, flags.bits() as i32, &mut rec);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(stat_of(&rec))
}

/// `statx(dirfd, path, flags, mask)`.
pub fn statx<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, flags: AtFlags, mask: StatxFlags) -> Result<Statx> {
    let bytes = path_bytes(&path)?;
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_statx(dirfd.as_fd().as_raw_fd(), &bytes, flags.bits() as i32, mask.bits(), &mut rec);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(statx_of(&rec))
}

/// `faccessat(dirfd, path, access, flags)`.
pub fn accessat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, access: Access, flags: AtFlags) -> Result<()> {
    let bytes = path_bytes(&path)?;
    let e = core::intrinsics::sys_x_accessat(dirfd.as_fd().as_raw_fd(), &bytes, access.bits() as i32, flags.bits() as i32);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(())
}

/// `utimensat(dirfd, path, times, flags)`.
pub fn utimensat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, times: &Timestamps, flags: AtFlags) -> Result<()> {
    let bytes = path_bytes(&path)?;
    let e = core::intrinsics::sys_x_utimensat(
        dirfd.as_fd().as_raw_fd(),
        &bytes,
        times.last_access.tv_sec,
        times.last_access.tv_nsec,
        times.last_modification.tv_sec,
        times.last_modification.tv_nsec,
        flags.bits() as i32,
    );
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(())
}

/// `mknodat(dirfd, path, kind, mode, dev)`.
pub fn mknodat<P: Arg, Fd: AsFd>(dirfd: Fd, path: P, kind: FileType, mode: Mode, dev: u64) -> Result<()> {
    let bytes = path_bytes(&path)?;
    let e = core::intrinsics::sys_x_mknodat(dirfd.as_fd().as_raw_fd(), &bytes, kind.as_raw_mode() | mode.as_raw_mode(), dev);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(())
}

/// `fcntl(fd, F_GETFL)`.
pub fn fcntl_getfl<Fd: AsFd>(fd: Fd) -> Result<OFlags> {
    let r = core::intrinsics::sys_x_fcntl_getfl(fd.as_fd().as_raw_fd());
    if r < 0 {
        Err(Errno::from_raw_os_error(-r))
    } else {
        Ok(OFlags::from_bits_retain(r as u32))
    }
}

/// `fcntl(fd, F_SETFL, flags)`.
pub fn fcntl_setfl<Fd: AsFd>(fd: Fd, flags: OFlags) -> Result<()> {
    let e = core::intrinsics::sys_x_fcntl_setfl(fd.as_fd().as_raw_fd(), flags.bits() as i32);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(())
}

/// Mount flags of a `StatVfs` (`ST_*`, Linux values).
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct StatVfsMountFlags(u64);

impl StatVfsMountFlags {
    /// `ST_RDONLY`
    pub const RDONLY: StatVfsMountFlags = StatVfsMountFlags(1);
    /// `ST_NOSUID`
    pub const NOSUID: StatVfsMountFlags = StatVfsMountFlags(2);
    /// `ST_NODEV`
    pub const NODEV: StatVfsMountFlags = StatVfsMountFlags(4);
    /// `ST_NOEXEC`
    pub const NOEXEC: StatVfsMountFlags = StatVfsMountFlags(8);
    /// `ST_SYNCHRONOUS`
    pub const SYNCHRONOUS: StatVfsMountFlags = StatVfsMountFlags(16);
    /// `ST_MANDLOCK`
    pub const MANDLOCK: StatVfsMountFlags = StatVfsMountFlags(64);
    /// `ST_NOATIME`
    pub const NOATIME: StatVfsMountFlags = StatVfsMountFlags(1024);
    /// `ST_NODIRATIME`
    pub const NODIRATIME: StatVfsMountFlags = StatVfsMountFlags(2048);
    /// `ST_RELATIME`
    pub const RELATIME: StatVfsMountFlags = StatVfsMountFlags(4096);

    /// No flags.
    pub const fn empty() -> StatVfsMountFlags { StatVfsMountFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u64 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u64) -> StatVfsMountFlags { StatVfsMountFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: StatVfsMountFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for StatVfsMountFlags {
    type Output = StatVfsMountFlags;
    fn bitor(self, other: StatVfsMountFlags) -> StatVfsMountFlags { StatVfsMountFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for StatVfsMountFlags {
    fn bitor_assign(&mut self, other: StatVfsMountFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for StatVfsMountFlags {
    type Output = StatVfsMountFlags;
    fn bitand(self, other: StatVfsMountFlags) -> StatVfsMountFlags { StatVfsMountFlags(self.0 & other.0) }
}

/// The result of `statvfs` and `fstatvfs` (no derives, as upstream).
pub struct StatVfs {
    pub f_bsize: u64,
    pub f_frsize: u64,
    pub f_blocks: u64,
    pub f_bfree: u64,
    pub f_bavail: u64,
    pub f_files: u64,
    pub f_ffree: u64,
    pub f_favail: u64,
    pub f_fsid: u64,
    pub f_flag: StatVfsMountFlags,
    pub f_namemax: u64,
}

/// Decodes one sysx statvfs record.
fn statvfs_of(rec: &[u8]) -> StatVfs {
    StatVfs {
        f_bsize: rec_u64(rec, 0),
        f_frsize: rec_u64(rec, 1),
        f_blocks: rec_u64(rec, 2),
        f_bfree: rec_u64(rec, 3),
        f_bavail: rec_u64(rec, 4),
        f_files: rec_u64(rec, 5),
        f_ffree: rec_u64(rec, 6),
        f_favail: rec_u64(rec, 7),
        f_fsid: rec_u64(rec, 8),
        f_flag: StatVfsMountFlags::from_bits_retain(rec_u64(rec, 9)),
        f_namemax: rec_u64(rec, 10),
    }
}

/// `statvfs(path)`.
pub fn statvfs<P: Arg>(path: P) -> Result<StatVfs> {
    let bytes = path_bytes(&path)?;
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_statvfs(&bytes, &mut rec);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(statvfs_of(&rec))
}

/// `fstatvfs(fd)`.
pub fn fstatvfs<Fd: AsFd>(fd: Fd) -> Result<StatVfs> {
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_fstatvfs(fd.as_fd().as_raw_fd(), &mut rec);
    if e != 0 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok(statvfs_of(&rec))
}

/// A directory iterator implemented with getdents (`rustix::fs::RawDir`).
/// The records are read through the `sys_x_getdents` intrinsic into a
/// scratch Vec and mirrored into the caller's buffer, so the buffer
/// holds the kernel bytes as upstream leaves them; entries parse from
/// the scratch copy and borrow their names from it.
pub struct RawDir<'buf, Fd: AsFd> {
    fd: Fd,
    buf: &'buf mut [std::mem::MaybeUninit<u8>],
    raw: Vec<u8>,
    initialized: usize,
    offset: usize,
}

impl<'buf, Fd: AsFd> RawDir<'buf, Fd> {
    /// A new iterator from the given file descriptor and buffer. The
    /// buffer size may be trimmed to accommodate alignment (the
    /// kernel's `linux_dirent64` is 8-byte aligned).
    pub fn new(fd: Fd, buf: &'buf mut [std::mem::MaybeUninit<u8>]) -> Self {
        let addr = buf.as_mut_ptr() as usize;
        let offset = (8 - addr % 8) % 8;
        let buf = if offset < buf.len() { &mut buf[offset..] } else { &mut [] };
        RawDir { fd, buf, raw: Vec::new(), initialized: 0, offset: 0 }
    }

    /// Identical to `Iterator::next` except that the item borrows
    /// from self (upstream's own words): the next record, refilling
    /// the buffer with one getdents64 call when it is empty.
    pub fn next(&mut self) -> Option<Result<RawDirEntry<'_>>> {
        if self.is_buffer_empty() {
            self.raw.clear();
            let n = core::intrinsics::sys_x_getdents(self.fd.as_fd().as_raw_fd(), self.buf.len() as u64, &mut self.raw);
            if n < 0 {
                return Some(Err(Errno::from_raw_os_error((-n) as i32)));
            }
            if n == 0 {
                return None;
            }
            let n = n as usize;
            for i in 0..n {
                self.buf[i].write(self.raw[i]);
            }
            self.initialized = n;
            self.offset = 0;
        }
        // linux_dirent64: ino u64, off i64, reclen u16, type u8, name.
        let at = &self.raw[self.offset..self.initialized];
        let word = |i: usize| u64::from_le_bytes([at[i], at[i + 1], at[i + 2], at[i + 3], at[i + 4], at[i + 5], at[i + 6], at[i + 7]]);
        let reclen = u16::from_le_bytes([at[16], at[17]]) as usize;
        let name_end = at[19..reclen].iter().position(|&b| b == 0).map(|p| 19 + p + 1).unwrap_or(reclen);
        let name = std::ffi::CStr::from_bytes_with_nul(&at[19..name_end]).expect("the kernel ends each name with a NUL");
        let ent = RawDirEntry { file_name: name, file_type: at[18], inode_number: word(0), next_entry_cookie: word(8) as i64 };
        self.offset += reclen;
        Some(Ok(ent))
    }

    /// Returns true if the internal buffer is empty and will be
    /// refilled when calling `next`.
    pub fn is_buffer_empty(&self) -> bool {
        self.offset >= self.initialized
    }
}

/// A raw directory entry, similar to `std::fs::DirEntry`.
pub struct RawDirEntry<'a> {
    file_name: &'a std::ffi::CStr,
    file_type: u8,
    inode_number: u64,
    next_entry_cookie: i64,
}

impl<'a> RawDirEntry<'a> {
    /// Returns the file name of this directory entry.
    pub fn file_name(&self) -> &std::ffi::CStr {
        self.file_name
    }

    /// Returns the type of this directory entry.
    pub fn file_type(&self) -> FileType {
        FileType::from_dirent_d_type(self.file_type)
    }

    /// Returns the inode number of this directory entry.
    pub fn ino(&self) -> u64 {
        self.inode_number
    }

    /// Returns the seek cookie to the next directory entry.
    pub fn next_entry_cookie(&self) -> u64 {
        self.next_entry_cookie as u64
    }
}

impl core::fmt::Debug for RawDirEntry<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RawDirEntry")
            .field("file_name", &self.file_name())
            .field("file_type", &self.file_type())
            .field("ino", &self.ino())
            .field("next_entry_cookie", &self.next_entry_cookie())
            .finish()
    }
}

impl FileType {
    /// The type of a directory entry's `d_type` byte (crate-private,
    /// as upstream).
    pub(crate) const fn from_dirent_d_type(d_type: u8) -> FileType {
        match d_type {
            1 => FileType::Fifo,
            2 => FileType::CharacterDevice,
            4 => FileType::Directory,
            6 => FileType::BlockDevice,
            8 => FileType::RegularFile,
            10 => FileType::Symlink,
            12 => FileType::Socket,
            _ => FileType::Unknown,
        }
    }
}

/// `rustix::fs::inotify`: inotify creation and watches, over
/// sysx. Events are read with `rustix::io::read` on the fd, as
/// ts-rust does. The `Reader` iterator at the end reads events the same way.
pub mod inotify {
    use crate::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};
    use crate::fs::path_bytes;
    use crate::io::{Errno, Result};
    use crate::path::Arg;

/// `inotify_init1` flags.
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CreateFlags(u32);

impl CreateFlags {
    /// `IN_CLOEXEC`
    pub const CLOEXEC: CreateFlags = CreateFlags(0o2000000);
    /// `IN_NONBLOCK`
    pub const NONBLOCK: CreateFlags = CreateFlags(0o4000);

    /// No flags.
    pub const fn empty() -> CreateFlags { CreateFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> CreateFlags { CreateFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: CreateFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for CreateFlags {
    type Output = CreateFlags;
    fn bitor(self, other: CreateFlags) -> CreateFlags { CreateFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for CreateFlags {
    fn bitor_assign(&mut self, other: CreateFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for CreateFlags {
    type Output = CreateFlags;
    fn bitand(self, other: CreateFlags) -> CreateFlags { CreateFlags(self.0 & other.0) }
}

/// `inotify_add_watch` mask bits.
// No Debug: upstream prints the bitflags form, which this adapter lacks.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct WatchFlags(u32);

impl WatchFlags {
    /// `IN_ACCESS`
    pub const ACCESS: WatchFlags = WatchFlags(0x0000_0001);
    /// `IN_MODIFY`
    pub const MODIFY: WatchFlags = WatchFlags(0x0000_0002);
    /// `IN_ATTRIB`
    pub const ATTRIB: WatchFlags = WatchFlags(0x0000_0004);
    /// `IN_CLOSE_WRITE`
    pub const CLOSE_WRITE: WatchFlags = WatchFlags(0x0000_0008);
    /// `IN_CLOSE_NOWRITE`
    pub const CLOSE_NOWRITE: WatchFlags = WatchFlags(0x0000_0010);
    /// `IN_OPEN`
    pub const OPEN: WatchFlags = WatchFlags(0x0000_0020);
    /// `IN_MOVED_FROM`
    pub const MOVED_FROM: WatchFlags = WatchFlags(0x0000_0040);
    /// `IN_MOVED_TO`
    pub const MOVED_TO: WatchFlags = WatchFlags(0x0000_0080);
    /// `IN_CREATE`
    pub const CREATE: WatchFlags = WatchFlags(0x0000_0100);
    /// `IN_DELETE`
    pub const DELETE: WatchFlags = WatchFlags(0x0000_0200);
    /// `IN_DELETE_SELF`
    pub const DELETE_SELF: WatchFlags = WatchFlags(0x0000_0400);
    /// `IN_MOVE_SELF`
    pub const MOVE_SELF: WatchFlags = WatchFlags(0x0000_0800);
    /// `IN_CLOSE`
    pub const CLOSE: WatchFlags = WatchFlags(0x0000_0018);
    /// `IN_MOVE`
    pub const MOVE: WatchFlags = WatchFlags(0x0000_00c0);
    /// `IN_ALL_EVENTS`
    pub const ALL_EVENTS: WatchFlags = WatchFlags(0x0000_0fff);
    /// `IN_ONLYDIR`
    pub const ONLYDIR: WatchFlags = WatchFlags(0x0100_0000);
    /// `IN_DONT_FOLLOW`
    pub const DONT_FOLLOW: WatchFlags = WatchFlags(0x0400_0000);
    /// `IN_EXCL_UNLINK`
    pub const EXCL_UNLINK: WatchFlags = WatchFlags(0x0800_0000);
    /// `IN_MASK_CREATE`
    pub const MASK_CREATE: WatchFlags = WatchFlags(0x1000_0000);
    /// `IN_MASK_ADD`
    pub const MASK_ADD: WatchFlags = WatchFlags(0x2000_0000);
    /// `IN_ONESHOT`
    pub const ONESHOT: WatchFlags = WatchFlags(0x8000_0000);

    /// No flags.
    pub const fn empty() -> WatchFlags { WatchFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> WatchFlags { WatchFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: WatchFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

impl core::ops::BitOr for WatchFlags {
    type Output = WatchFlags;
    fn bitor(self, other: WatchFlags) -> WatchFlags { WatchFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for WatchFlags {
    fn bitor_assign(&mut self, other: WatchFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for WatchFlags {
    type Output = WatchFlags;
    fn bitand(self, other: WatchFlags) -> WatchFlags { WatchFlags(self.0 & other.0) }
}

/// inotify event mask bits, as read from an event record.
// No Debug: upstream prints the bitflags form, which this adapter lacks.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct ReadFlags(u32);

impl ReadFlags {
    /// `IN_ACCESS`
    pub const ACCESS: ReadFlags = ReadFlags(0x0000_0001);
    /// `IN_MODIFY`
    pub const MODIFY: ReadFlags = ReadFlags(0x0000_0002);
    /// `IN_ATTRIB`
    pub const ATTRIB: ReadFlags = ReadFlags(0x0000_0004);
    /// `IN_CLOSE_WRITE`
    pub const CLOSE_WRITE: ReadFlags = ReadFlags(0x0000_0008);
    /// `IN_CLOSE_NOWRITE`
    pub const CLOSE_NOWRITE: ReadFlags = ReadFlags(0x0000_0010);
    /// `IN_OPEN`
    pub const OPEN: ReadFlags = ReadFlags(0x0000_0020);
    /// `IN_MOVED_FROM`
    pub const MOVED_FROM: ReadFlags = ReadFlags(0x0000_0040);
    /// `IN_MOVED_TO`
    pub const MOVED_TO: ReadFlags = ReadFlags(0x0000_0080);
    /// `IN_CREATE`
    pub const CREATE: ReadFlags = ReadFlags(0x0000_0100);
    /// `IN_DELETE`
    pub const DELETE: ReadFlags = ReadFlags(0x0000_0200);
    /// `IN_DELETE_SELF`
    pub const DELETE_SELF: ReadFlags = ReadFlags(0x0000_0400);
    /// `IN_MOVE_SELF`
    pub const MOVE_SELF: ReadFlags = ReadFlags(0x0000_0800);
    /// `IN_UNMOUNT`
    pub const UNMOUNT: ReadFlags = ReadFlags(0x0000_2000);
    /// `IN_Q_OVERFLOW`
    pub const QUEUE_OVERFLOW: ReadFlags = ReadFlags(0x0000_4000);
    /// `IN_IGNORED`
    pub const IGNORED: ReadFlags = ReadFlags(0x0000_8000);
    /// `IN_ISDIR`
    pub const ISDIR: ReadFlags = ReadFlags(0x4000_0000);

    /// No flags.
    pub const fn empty() -> ReadFlags { ReadFlags(0) }
    /// The raw bits.
    pub const fn bits(&self) -> u32 { self.0 }
    /// Flags from raw bits, keeping unknown bits.
    pub const fn from_bits_retain(bits: u32) -> ReadFlags { ReadFlags(bits) }
    /// True when every bit of `other` is set in `self`.
    pub const fn contains(&self, other: ReadFlags) -> bool { self.0 & other.0 == other.0 }
    /// True when no bit is set.
    pub const fn is_empty(&self) -> bool { self.0 == 0 }
}

/// The Debug form bitflags 2 derives for `ReadFlags`: a tuple
/// struct around the flag names (`ReadFlags(ACCESS | 0x8)`,
/// `ReadFlags(0x0)` when empty), so `{:#?}` breaks lines as
/// upstream. Other flag types in this adapter stay without Debug
/// (README).
impl core::fmt::Debug for ReadFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("ReadFlags").field(&ReadFlagNames(self.0)).finish()
    }
}

/// The inner bitflags value of `ReadFlags` Debug: the names of the
/// set bits in declaration order, then the unnamed rest as hex
/// (bitflags' `parser::to_writer`; every ReadFlags bit is one bit).
struct ReadFlagNames(u32);

impl core::fmt::Debug for ReadFlagNames {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        const NAMES: &[(u32, &str)] = &[
            (0x0000_0001, "ACCESS"),
            (0x0000_0004, "ATTRIB"),
            (0x0000_0010, "CLOSE_NOWRITE"),
            (0x0000_0008, "CLOSE_WRITE"),
            (0x0000_0100, "CREATE"),
            (0x0000_0200, "DELETE"),
            (0x0000_0400, "DELETE_SELF"),
            (0x0000_0002, "MODIFY"),
            (0x0000_0800, "MOVE_SELF"),
            (0x0000_0040, "MOVED_FROM"),
            (0x0000_0080, "MOVED_TO"),
            (0x0000_0020, "OPEN"),
            (0x0000_8000, "IGNORED"),
            (0x4000_0000, "ISDIR"),
            (0x0000_4000, "QUEUE_OVERFLOW"),
            (0x0000_2000, "UNMOUNT"),
        ];
        if self.0 == 0 {
            return write!(f, "{:#x}", 0u32);
        }
        let mut rest = self.0;
        let mut first = true;
        for &(bit, name) in NAMES {
            if rest & bit != 0 {
                if !first {
                    f.write_str(" | ")?;
                }
                f.write_str(name)?;
                first = false;
                rest &= !bit;
            }
        }
        if rest != 0 {
            if !first {
                f.write_str(" | ")?;
            }
            write!(f, "{:#x}", rest)?;
        }
        Ok(())
    }
}

impl core::ops::BitOr for ReadFlags {
    type Output = ReadFlags;
    fn bitor(self, other: ReadFlags) -> ReadFlags { ReadFlags(self.0 | other.0) }
}

impl core::ops::BitOrAssign for ReadFlags {
    fn bitor_assign(&mut self, other: ReadFlags) { self.0 |= other.0; }
}

impl core::ops::BitAnd for ReadFlags {
    type Output = ReadFlags;
    fn bitand(self, other: ReadFlags) -> ReadFlags { ReadFlags(self.0 & other.0) }
}

    /// `inotify_init1(flags)`.
    pub fn init(flags: CreateFlags) -> Result<OwnedFd> {
        let fd = core::intrinsics::sys_x_inotify_init1(flags.bits() as i32);
        if fd < 0 {
            Err(Errno::from_raw_os_error(-fd))
        } else {
            Ok(OwnedFd::from_raw_fd(fd))
        }
    }

    /// `inotify_add_watch(fd, path, flags)`: the watch descriptor.
    pub fn add_watch<P: Arg, Fd: AsFd>(fd: Fd, path: P, flags: WatchFlags) -> Result<i32> {
        let bytes = path_bytes(&path)?;
        let wd = core::intrinsics::sys_x_inotify_add_watch(fd.as_fd().as_raw_fd(), &bytes, flags.bits());
        if wd < 0 {
            Err(Errno::from_raw_os_error(-wd))
        } else {
            Ok(wd)
        }
    }

    /// `inotify_rm_watch(fd, wd)`.
    pub fn remove_watch<Fd: AsFd>(fd: Fd, wd: i32) -> Result<()> {
        let e = core::intrinsics::sys_x_inotify_rm_watch(fd.as_fd().as_raw_fd(), wd);
        if e != 0 {
            return Err(Errno::from_raw_os_error(e));
        }
        Ok(())
    }

    /// An inotify event iterator implemented with the read syscall
    /// (upstream's `Reader`, based on `RawDir`'s API). Records are
    /// read through a scratch Vec and mirrored into the caller's
    /// buffer; events parse from the scratch copy.
    pub struct Reader<'buf, Fd: AsFd> {
        fd: Fd,
        buf: &'buf mut [std::mem::MaybeUninit<u8>],
        raw: Vec<u8>,
        initialized: usize,
        offset: usize,
    }

    impl<'buf, Fd: AsFd> Reader<'buf, Fd> {
        /// A new iterator from the given file descriptor and buffer.
        pub fn new(fd: Fd, buf: &'buf mut [std::mem::MaybeUninit<u8>]) -> Self {
            let addr = buf.as_mut_ptr() as usize;
            let offset = (4 - addr % 4) % 4;
            let buf = if offset < buf.len() { &mut buf[offset..] } else { &mut [] };
            Reader { fd, buf, raw: Vec::new(), initialized: 0, offset: 0 }
        }

        /// Read the next inotify event (or fail with the read's
        /// error, `Errno::AGAIN` included for a nonblocking fd).
        pub fn next(&mut self) -> Result<Event<'_>> {
            if self.is_buffer_empty() {
                self.raw.clear();
                let n = core::intrinsics::sys_fd_read(self.fd.as_fd().as_raw_fd(), self.buf.len() as u64, &mut self.raw);
                if n < 0 {
                    return Err(Errno::from_raw_os_error((-n) as i32));
                }
                if n == 0 {
                    return Err(Errno::INVAL);
                }
                let n = n as usize;
                for i in 0..n {
                    self.buf[i].write(self.raw[i]);
                }
                self.initialized = n;
                self.offset = 0;
            }
            // inotify_event: wd i32, mask u32, cookie u32, len u32, name.
            let at = &self.raw[self.offset..self.initialized];
            let wd = i32::from_le_bytes([at[0], at[1], at[2], at[3]]);
            let mask = u32::from_le_bytes([at[4], at[5], at[6], at[7]]);
            let cookie = u32::from_le_bytes([at[8], at[9], at[10], at[11]]);
            let len = u32::from_le_bytes([at[12], at[13], at[14], at[15]]) as usize;
            let file_name = if len > 0 {
                let end = at[16..16 + len].iter().position(|&b| b == 0).map(|p| 16 + p + 1).unwrap_or(16 + len);
                Some(std::ffi::CStr::from_bytes_with_nul(&at[16..end]).expect("the kernel ends each name with a NUL"))
            } else {
                None
            };
            self.offset += 16 + len;
            Ok(Event { wd, events: ReadFlags::from_bits_retain(mask), cookie, file_name })
        }

        /// Returns true if the internal buffer is empty and will be
        /// refilled when calling `next`.
        pub fn is_buffer_empty(&self) -> bool {
            self.offset >= self.initialized
        }
    }

    /// An inotify event.
    #[derive(Debug)]
    pub struct Event<'a> {
        wd: i32,
        events: ReadFlags,
        cookie: u32,
        file_name: Option<&'a std::ffi::CStr>,
    }

    impl<'a> Event<'a> {
        /// Returns the watch for which this event occurs.
        pub fn wd(&self) -> i32 {
            self.wd
        }

        /// Returns a description of the events.
        pub fn events(&self) -> ReadFlags {
            self.events
        }

        /// Returns the unique cookie associating related events.
        pub fn cookie(&self) -> u32 {
            self.cookie
        }

        /// Returns the file name of this event, if any.
        pub fn file_name(&self) -> Option<&std::ffi::CStr> {
            self.file_name
        }
    }
}
