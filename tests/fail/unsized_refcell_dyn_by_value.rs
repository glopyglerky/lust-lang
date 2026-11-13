// expect: L0110
// rustc E0277: a `RefCell<dyn T>` has no size, so it cannot be a by-value parameter.
use std::cell::RefCell;
trait T { fn f(&self) -> u8; }
fn g(x: RefCell<dyn T>) {}
fn main() {}
