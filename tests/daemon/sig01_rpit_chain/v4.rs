use std::fmt::Debug;
fn f() -> std::rc::Rc<u32> { std::rc::Rc::new(7) }
fn g() -> impl Debug { f() }
fn need_send<T: Sync>(t: T) -> T { t }
fn main() { println!("{:?}", need_send(g())); }
