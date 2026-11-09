// expect: L0612
// rustc E0277: Vec<T>: Tr needs T: Copy, which the fn does not bound.
trait Tr { type Out; }
impl<T: Copy> Tr for Vec<T> { type Out = T; }
fn f<T>(x: <Vec<T> as Tr>::Out) -> usize { let _ = x; 1 }
fn main() { println!("{}", f::<u8>(3)); }
