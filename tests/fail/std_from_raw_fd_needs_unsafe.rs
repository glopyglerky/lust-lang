// expect: L0404
// rustc E0133: `FromRawFd::from_raw_fd` is an `unsafe fn` in std.
use std::os::fd::{FromRawFd, OwnedFd};

fn main() {
    let fd = OwnedFd::from_raw_fd(0);
    std::mem::forget(fd);
}
