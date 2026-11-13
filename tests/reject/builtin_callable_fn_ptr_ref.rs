// expect: L0641
// Valid Rust. A reference to a fn pointer as `catch_unwind`'s function:
// lust's builtin call has no deref for it, so check rejects it.
use std::panic::catch_unwind;

fn seven() -> i32 {
    7
}

fn main() {
    let fp: fn() -> i32 = seven;
    println!("{:?}", catch_unwind(&fp).ok());
}
