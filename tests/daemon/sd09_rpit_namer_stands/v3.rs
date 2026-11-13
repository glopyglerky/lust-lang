use std::fmt::Debug;
fn f(x: u64) -> u32  { x as u32 }
fn g() -> impl Debug { f(1) }
fn h() -> impl Debug + Clone { (g(), f(2)).1 }
fn need<T: Send + Debug>(t: T) -> T { t }
fn main() { println!("{:?} {:?}", need(g()), h().clone()); }
