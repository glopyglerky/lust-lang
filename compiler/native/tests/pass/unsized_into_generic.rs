use std::rc::Rc;
fn mk<T: Into<Rc<str>>>(t: T) -> Rc<str> { t.into() }
fn main() { let a = mk("lit"); let b = mk(String::from("own")); println!("{a} {b}"); }
