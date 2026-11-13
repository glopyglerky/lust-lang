// expect: L0612
// rustc E0277: the projection needs T: Tr, and T has no bound.
trait Tr { type Out; }
fn f<T>(_x: <T as Tr>::Out) {}
fn main() { println!("hi"); }
