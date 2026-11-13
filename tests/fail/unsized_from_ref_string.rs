// expect: L0612
// rustc E0277: `Rc<str>` has no `From<&String>` (a generic `From` argument does not deref-coerce).
use std::rc::Rc;
fn main() { let s = String::from("x"); let a: Rc<str> = Rc::from(&s); println!("{}", a); }
