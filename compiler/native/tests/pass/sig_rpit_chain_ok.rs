// An unchanged `g() -> impl Debug { f() }` hides f's return type; a caller needs Send from g() (sigedit: Sol's RPIT chain).
use std::fmt::Debug;
fn f() -> u32 { 7 }
fn g() -> impl Debug { f() }
fn need_send<T: Send>(t: T) -> T { t }
fn main() { println!("{:?}", need_send(g())); }
