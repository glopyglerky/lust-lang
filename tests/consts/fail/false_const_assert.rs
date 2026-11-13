// rustc-error: E0080
// ARCH 2.13: a false assert in a const item is a compile
// error, as in rustc. The panic message is part of the error.
const _: () = assert!(std::mem::size_of::<u64>() == 4, "u64 must be 4 bytes");
fn main() {}
