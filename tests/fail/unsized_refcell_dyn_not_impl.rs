// expect: L0612
// rustc E0277: `S` does not implement `T`, so `Rc<RefCell<S>>` does not coerce to `Rc<RefCell<dyn T>>`.
use std::cell::RefCell; use std::rc::Rc;
trait T { fn f(&self) -> u8; }
struct S;
fn main() { let a: Rc<RefCell<dyn T>> = Rc::new(RefCell::new(S)); }
