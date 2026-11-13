// expect: L0404
// rustc E0053: std's `from_raw_fd` is an `unsafe fn`; a safe one does not
// match the trait (and lust cannot spell `unsafe fn`).
use std::os::fd::FromRawFd;

struct X;

impl FromRawFd for X {
    fn from_raw_fd(_fd: i32) -> X {
        X
    }
}

fn main() {
    println!("x");
}
