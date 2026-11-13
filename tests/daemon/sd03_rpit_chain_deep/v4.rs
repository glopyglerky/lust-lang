use std::fmt::Debug;
use std::rc::Rc;
fn f() -> std::cell::Cell<u32> { std::cell::Cell::new(7) }
fn g() -> impl Debug { f() }
fn h() -> impl Debug { g() }
fn need<T: Sync>(t: T) -> T { t }
fn main() { println!("{:?}", need(h())); }
