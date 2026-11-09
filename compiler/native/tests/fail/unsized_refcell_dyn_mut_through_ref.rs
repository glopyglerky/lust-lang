// expect: L0308
// rustc E0596: a `Ref<dyn T>` gives shared access only; `f` takes `&mut self`.
use std::cell::RefCell; use std::rc::Rc;
trait T { fn f(&mut self) -> u8; }
struct S;
impl T for S { fn f(&mut self) -> u8 { 1 } }
fn main() { let a: Rc<RefCell<dyn T>> = Rc::new(RefCell::new(S)); let r = a.borrow(); println!("{}", r.f()); }
