// `rustix::termios`: only the window size query ts-rust uses.

use crate::fd::{AsFd, AsRawFd};
use crate::io::{Errno, Result};

/// A terminal window size.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Winsize {
    /// The number of rows the terminal has.
    pub ws_row: u16,
    /// The number of columns the terminal has.
    pub ws_col: u16,
    /// The width in pixels (often unused).
    pub ws_xpixel: u16,
    /// The height in pixels (often unused).
    pub ws_ypixel: u16,
}

/// `ioctl(fd, TIOCGWINSZ)`: the terminal window size of `fd`.
pub fn tcgetwinsize<Fd: AsFd>(fd: Fd) -> Result<Winsize> {
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_winsize(fd.as_fd().as_raw_fd(), &mut rec);
    if e != 0 || rec.len() < 8 {
        return Err(Errno::from_raw_os_error(e));
    }
    let at = |i: usize| (rec[2 * i] as u16) | ((rec[2 * i + 1] as u16) << 8);
    Ok(Winsize { ws_row: at(0), ws_col: at(1), ws_xpixel: at(2), ws_ypixel: at(3) })
}
