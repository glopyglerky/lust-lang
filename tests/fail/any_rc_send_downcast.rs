// expect: L0104
// rustc E0599: std has `Rc<dyn Any>::downcast` only, not for `dyn Any + Send` (W-CHECK-G3).
use std::any::Any;
use std::rc::Rc;
fn main() { let r: Rc<dyn Any + Send> = Rc::new(1); println!("{}", r.downcast::<i32>().is_ok()); }
