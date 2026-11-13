// expect: L0612
// rustc E0277: the impl that gives Out needs String: Copy.
trait Tr { type Out; }
impl<T: Copy> Tr for Vec<T> { type Out = T; }
fn f(x: <Vec<String> as Tr>::Out) -> usize { x.len() }
fn main() { println!("{}", f(String::new())); }
