// expect: L0112
// rustc E0277: `Cell<T>: Debug` needs `T: Copy`, which the derive's
// `T: Debug` bound does not give.
use std::cell::Cell;
#[derive(Debug)]
struct S<T> {
    c: Cell<T>,
}
fn main() {
    let s = S { c: Cell::new(1u8) };
    println!("{}", s.c.get());
}
