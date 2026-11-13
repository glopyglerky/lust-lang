// `Rc::downgrade` of an `Rc<str>` gives a `Weak<str>` (the Rc's fat value, cgen/own.rs).
use std::rc::Rc;
fn main() { let a: Rc<str> = Rc::from("x"); let w = Rc::downgrade(&a); println!("{:?}", w.upgrade()); }
