use std::fmt::Debug;
fn f(x: u8) -> u32  { x as u32 }
fn t<T: Debug>(v: T) -> String { format!("{:?} {}", v, f(3)) }
fn u<T: Clone + Debug>(v: T) -> Vec<T> { let n = f(1) as usize; vec![v; n] }
struct W<T>(T);
impl<T: Debug> W<T> { fn show(&self) -> String { format!("{:?}{}", self.0, f(2)) } }
fn main() { println!("{} {:?} {}", t(1u8), u("a"), W(2i16).show()); }
