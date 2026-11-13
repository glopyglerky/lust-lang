// expect: L0110
// rustc E0599: `RefCell::into_inner` moves the value out, so it needs `T: Sized`.
use std::cell::RefCell;
trait T { fn f(&self) -> u8; }
struct S;
impl T for S { fn f(&self) -> u8 { 1 } }
fn main() { let a: Box<RefCell<dyn T>> = Box::new(RefCell::new(S)); let v = a.into_inner(); }
