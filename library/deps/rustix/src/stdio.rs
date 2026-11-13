// `rustix::stdio`: the standard stream fds and dup2 onto them.

use crate::fd::{AsFd, AsRawFd, BorrowedFd};
use crate::io::{Errno, Result};

#[cfg(feature = "std")]
/// `STDIN_FILENO` as a borrowed fd.
pub const fn stdin() -> BorrowedFd<'static> {
    BorrowedFd::borrow_raw(0)
}

#[cfg(feature = "std")]
/// `STDOUT_FILENO` as a borrowed fd.
pub const fn stdout() -> BorrowedFd<'static> {
    BorrowedFd::borrow_raw(1)
}

#[cfg(feature = "std")]
/// `STDERR_FILENO` as a borrowed fd.
pub const fn stderr() -> BorrowedFd<'static> {
    BorrowedFd::borrow_raw(2)
}

/// One dup2 onto a standard fd.
fn dup2_to<Fd: AsFd>(fd: Fd, to: i32) -> Result<()> {
    let r = core::intrinsics::sys_x_dup2(fd.as_fd().as_raw_fd(), to);
    if r < 0 {
        return Err(Errno::from_raw_os_error(-r));
    }
    Ok(())
}

/// `dup2(fd, STDIN_FILENO)`.
pub fn dup2_stdin<Fd: AsFd>(fd: Fd) -> Result<()> {
    dup2_to(fd, 0)
}

/// `dup2(fd, STDOUT_FILENO)`.
pub fn dup2_stdout<Fd: AsFd>(fd: Fd) -> Result<()> {
    dup2_to(fd, 1)
}

/// `dup2(fd, STDERR_FILENO)`.
pub fn dup2_stderr<Fd: AsFd>(fd: Fd) -> Result<()> {
    dup2_to(fd, 2)
}
