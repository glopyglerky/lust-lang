use std::fmt::Debug;
use std::rc::Rc;
fn f() -> u64 { 7 }
fn g() -> impl Debug { f() }
fn h() -> impl Debug { g() }
fn need<T: Send>(t: T) -> T { t }
fn main() { println!("{:?}", need(h())); }
