// expect: L0110
// rustc E0596: an `Rc<dyn FnMut>` gives no mutable access.
use std::rc::Rc;
fn main() { let f: Rc<dyn FnMut()> = Rc::new(|| ()); f(); }
