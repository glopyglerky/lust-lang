use std::fmt::Debug;
fn f(x: u8) -> u32 where u8: Copy { x as u32 }
fn g() -> impl Debug { f(1) }
fn h() -> impl Debug + Clone { (g(), f(2)).1 }
fn need<T: Send + Debug>(t: T) -> T { t }
fn main() { println!("{:?} {:?}", need(g()), h().clone()); }
