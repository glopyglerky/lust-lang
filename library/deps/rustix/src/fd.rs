// `rustix::fd`: std's fd types, re-exported as upstream does
// with the `std` feature (tf-adapters F1: lust std now has
// `std::os::fd`). Code that stores a rustix `OwnedFd` in a
// `std::os::fd::OwnedFd`, or passes a `std::fs::File` to a
// rustix call, works as it does under rustc. `borrow_raw` and
// `from_raw_fd` are safe fns in lust std (it cannot spell
// `unsafe fn`); see library/std/src/os.rs.

pub use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
