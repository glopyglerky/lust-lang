// expect: L0612
use std::cell::Cell;
struct S { a: Cell<u8>, b: &'static Cell<u8> }
fn need<T: Send>(_: &T) {}
fn main() { let b: &'static Cell<u8> = Box::leak(Box::new(Cell::new(2))); let s = S { a: Cell::new(1), b }; need(&s); println!("{}", s.a.get() + s.b.get()); }
