// expect: L0404
// rustc E0133: the trait method is `unsafe` through a bound too.
use std::os::fd::{FromRawFd, OwnedFd};

fn make<T: FromRawFd>() -> T {
    T::from_raw_fd(0)
}

fn main() {
    let fd: OwnedFd = make();
    std::mem::forget(fd);
}
