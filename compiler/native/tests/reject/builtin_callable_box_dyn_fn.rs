// expect: L0641
// Valid Rust. lust's builtin calls (`catch_unwind`, the thread starts,
// `get_or_init`) call closures, fn items, fn pointers, `Box<dyn FnOnce>`,
// and references to closures or to `dyn Fn`; a `Box<dyn Fn>` is rejected
// at check (through a generic wrapper, at the build's instance check).
use std::panic::{AssertUnwindSafe, catch_unwind};

fn main() {
    let b: Box<dyn Fn() -> i32> = Box::new(|| 4);
    println!("{:?}", catch_unwind(AssertUnwindSafe(b)).ok());
}
