// expect: L0111
// rustc E0283: `Rc::from("x")` with no target type matches `From<&str> for Rc<str>` and `From<T> for Rc<T>`.
use std::rc::Rc;
fn main() { let a = Rc::from("x"); println!("{a}"); }
