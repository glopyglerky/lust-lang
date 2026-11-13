// expect: L0612
// rustc E0277: f now returns Rc<u32>, so the unchanged g() -> impl Debug is not Send (sigedit: Sol's RPIT chain).
use std::fmt::Debug;
fn f() -> std::rc::Rc<u32> { std::rc::Rc::new(7) }
fn g() -> impl Debug { f() }
fn need_send<T: Send>(t: T) -> T { t }
fn main() { println!("{:?}", need_send(g())); }
