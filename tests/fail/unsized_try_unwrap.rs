// expect: L0110
// rustc E0277: `Rc::try_unwrap` needs `T: Sized`; `str` is not.
use std::rc::Rc;
fn main() { let a: Rc<str> = Rc::from("x"); let s = Rc::try_unwrap(a); println!("{:?}", s.is_ok()); }
