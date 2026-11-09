use std::fmt::Debug;
trait Tr { type Out: Debug; fn get(&self) -> Self::Out; }
impl Tr for u8 { type Out = u32; fn get(&self) -> u32 { *self as u32 + 1 } }
impl<T: Tr> Tr for Vec<T> { type Out = Vec<T::Out>; fn get(&self) -> Self::Out { self.iter().map(|x| x.get()).collect() } }
fn f<T: Tr>(x: T) -> T::Out { x.get() }
fn main() { println!("{:?}", f(vec![1u8, 2])); }
