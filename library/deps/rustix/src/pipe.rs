// `rustix::pipe`: pipe creation.

use crate::fd::{FromRawFd, OwnedFd};
use crate::io::{Errno, Result};
use crate::rec_u64;

/// Flags for `pipe_with` (`pipe2` flags, Linux values).
// No Debug (upstream prints the bitflags form, which this adapter lacks)
// and no Default (upstream has none).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PipeFlags(u32);

impl PipeFlags {
    /// `O_CLOEXEC`
    pub const CLOEXEC: PipeFlags = PipeFlags(0o2000000);
    /// `O_DIRECT`
    pub const DIRECT: PipeFlags = PipeFlags(0o40000);
    /// `O_NONBLOCK`
    pub const NONBLOCK: PipeFlags = PipeFlags(0o4000);

    pub const fn empty() -> PipeFlags {
        PipeFlags(0)
    }
    pub const fn bits(&self) -> u32 {
        self.0
    }
    pub const fn from_bits_retain(bits: u32) -> PipeFlags {
        PipeFlags(bits)
    }
    pub const fn contains(&self, other: PipeFlags) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for PipeFlags {
    type Output = PipeFlags;
    fn bitor(self, other: PipeFlags) -> PipeFlags {
        PipeFlags(self.0 | other.0)
    }
}

/// `pipe2(flags)`: the pair (read end, write end).
pub fn pipe_with(flags: PipeFlags) -> Result<(OwnedFd, OwnedFd)> {
    let mut rec: Vec<u8> = Vec::new();
    let e = core::intrinsics::sys_x_pipe(flags.bits() as i32, &mut rec);
    if e != 0 || rec.len() < 16 {
        return Err(Errno::from_raw_os_error(e));
    }
    Ok((OwnedFd::from_raw_fd(rec_u64(&rec, 0) as i32), OwnedFd::from_raw_fd(rec_u64(&rec, 1) as i32)))
}

/// `pipe()`: a pipe with no flags.
pub fn pipe() -> Result<(OwnedFd, OwnedFd)> {
    pipe_with(PipeFlags::empty())
}
