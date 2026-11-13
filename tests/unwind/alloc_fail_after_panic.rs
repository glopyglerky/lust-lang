// std's alloc error hook prints the RUST_BACKTRACE note on every call:
// a caught panic before it (which took the panic hook's first-panic
// note) does not remove it. handle_alloc_error is the same hook.
use std::alloc::{handle_alloc_error, Layout};

fn main() {
    let r = std::panic::catch_unwind(|| {
        let v: Vec<u8> = Vec::with_capacity(usize::MAX);
        v.len()
    });
    println!("caught={}", r.is_err());
    let r = std::panic::catch_unwind(|| {
        let v: Vec<u16> = Vec::with_capacity(usize::MAX / 2);
        v.len()
    });
    println!("caught={}", r.is_err());
    handle_alloc_error(Layout::from_size_align(40, 8).unwrap());
}
