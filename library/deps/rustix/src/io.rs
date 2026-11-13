// `rustix::io`: the Errno type and the plain read and write
// calls. Errno's Debug and Display are std's io::Error forms, as
// upstream delegates to them.

use crate::buffer::Buffer;
use crate::buffer::private::Sealed as _;
use crate::fd::{AsFd, AsRawFd};
use core::fmt;

/// A specialized `Result` type for rustix APIs.
pub type Result<T> = core::result::Result<T, Errno>;

/// An error number. The constants are the Linux values.
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Errno(i32);

/// Upstream derives `Hash` over its raw `u16`, the negated code
/// (`ENOENT` hashes as `0xfffe`).
impl core::hash::Hash for Errno {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (self.0.wrapping_neg() as u16).hash(state);
    }
}

impl Errno {
    /// `EPERM`
    pub const PERM: Errno = Errno(1);
    /// `ENOENT`
    pub const NOENT: Errno = Errno(2);
    /// `ESRCH`
    pub const SRCH: Errno = Errno(3);
    /// `EINTR`
    pub const INTR: Errno = Errno(4);
    /// `EIO`
    pub const IO: Errno = Errno(5);
    /// `ENXIO`
    pub const NXIO: Errno = Errno(6);
    /// `E2BIG`
    pub const TOOBIG: Errno = Errno(7);
    /// `ENOEXEC`
    pub const NOEXEC: Errno = Errno(8);
    /// `EBADF`
    pub const BADF: Errno = Errno(9);
    /// `ECHILD`
    pub const CHILD: Errno = Errno(10);
    /// `EAGAIN`
    pub const AGAIN: Errno = Errno(11);
    /// `EWOULDBLOCK` (same number as `EAGAIN` on Linux)
    pub const WOULDBLOCK: Errno = Errno(11);
    /// `ENOMEM`
    pub const NOMEM: Errno = Errno(12);
    /// `EACCES`
    pub const ACCESS: Errno = Errno(13);
    /// `EFAULT`
    pub const FAULT: Errno = Errno(14);
    /// `EBUSY`
    pub const BUSY: Errno = Errno(16);
    /// `EEXIST`
    pub const EXIST: Errno = Errno(17);
    /// `EXDEV`
    pub const XDEV: Errno = Errno(18);
    /// `ENODEV`
    pub const NODEV: Errno = Errno(19);
    /// `ENOTDIR`
    pub const NOTDIR: Errno = Errno(20);
    /// `EISDIR`
    pub const ISDIR: Errno = Errno(21);
    /// `EINVAL`
    pub const INVAL: Errno = Errno(22);
    /// `ENFILE`
    pub const NFILE: Errno = Errno(23);
    /// `EMFILE`
    pub const MFILE: Errno = Errno(24);
    /// `ENOTTY`
    pub const NOTTY: Errno = Errno(25);
    /// `EFBIG`
    pub const FBIG: Errno = Errno(27);
    /// `ENOSPC`
    pub const NOSPC: Errno = Errno(28);
    /// `ESPIPE`
    pub const SPIPE: Errno = Errno(29);
    /// `EROFS`
    pub const ROFS: Errno = Errno(30);
    /// `EMLINK`
    pub const MLINK: Errno = Errno(31);
    /// `EPIPE`
    pub const PIPE: Errno = Errno(32);
    /// `ERANGE`
    pub const RANGE: Errno = Errno(34);
    /// `ENAMETOOLONG`
    pub const NAMETOOLONG: Errno = Errno(36);
    /// `ENOSYS`
    pub const NOSYS: Errno = Errno(38);
    /// `ENOTEMPTY`
    pub const NOTEMPTY: Errno = Errno(39);
    /// `ELOOP`
    pub const LOOP: Errno = Errno(40);
    /// `ENOTSOCK`
    pub const NOTSOCK: Errno = Errno(88);
    /// `EOPNOTSUPP`
    pub const OPNOTSUPP: Errno = Errno(95);
    /// `ETIMEDOUT`
    pub const TIMEDOUT: Errno = Errno(110);
    /// `ECONNREFUSED`
    pub const CONNREFUSED: Errno = Errno(111);

    /// The error from a raw OS error number.
    pub const fn from_raw_os_error(code: i32) -> Errno {
        Errno(code)
    }

    /// The raw OS error number.
    pub const fn raw_os_error(self) -> i32 {
        self.0
    }

    #[cfg(feature = "std")]
    /// The `Errno` of an `io::Error`, when it holds a raw OS error.
    pub fn from_io_error(err: &std::io::Error) -> Option<Errno> {
        err.raw_os_error().map(Errno)
    }

    #[cfg(feature = "std")]
    /// Shorthand for `std::io::Error::from(self).kind()`.
    pub fn kind(self) -> std::io::ErrorKind {
        std::io::Error::from(self).kind()
    }
}

impl fmt::Display for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::io::Error::from(*self).fmt(f)
    }
}

impl fmt::Debug for Errno {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::io::Error::from(*self).fmt(f)
    }
}

// No `impl Error for Errno` here: the core::error overlay is
// spliced only when user code names it, which an adapter cannot
// rely on. Programs that need Errno as a std error reject
// (README); the io::Error conversion below is the form ts-rust
// uses.

#[cfg(feature = "std")]
impl From<Errno> for std::io::Error {
    fn from(err: Errno) -> std::io::Error {
        std::io::Error::from_raw_os_error(err.0)
    }
}

/// One `read` of up to `buf.len()` bytes (sys_fd_read fills a Vec,
/// copied here; the count and the bytes are the syscall's).
pub fn read<Fd: AsFd, Buf: Buffer<u8>>(fd: Fd, mut buf: Buf) -> Result<usize> {
    read_into(fd, buf.__lust_slice())
}

/// The fill of `read` over a plain slice.
fn read_into<Fd: AsFd>(fd: Fd, buf: &mut [u8]) -> Result<usize> {
    let mut out: Vec<u8> = Vec::new();
    let n = core::intrinsics::sys_fd_read(fd.as_fd().as_raw_fd(), buf.len() as u64, &mut out);
    if n < 0 {
        return Err(Errno((-n) as i32));
    }
    let n = n as usize;
    for i in 0..n {
        buf[i] = out[i];
    }
    Ok(n)
}

/// One `write` of `buf`.
pub fn write<Fd: AsFd>(fd: Fd, buf: &[u8]) -> Result<usize> {
    let n = core::intrinsics::sys_fd_write(fd.as_fd().as_raw_fd(), buf);
    if n < 0 {
        Err(Errno((-n) as i32))
    } else {
        Ok(n as usize)
    }
}
