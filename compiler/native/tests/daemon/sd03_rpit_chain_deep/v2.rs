use std::fmt::Debug;
use std::rc::Rc;
fn f() -> Rc<u32> { Rc::new(7) }
fn g() -> impl Debug { f() }
fn h() -> impl Debug { g() }
fn need<T: Send>(t: T) -> T { t }
fn main() { println!("{:?}", need(h())); }
