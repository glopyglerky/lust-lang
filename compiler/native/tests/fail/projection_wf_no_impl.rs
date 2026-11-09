// expect: L0612
// rustc E0277: B does not implement Tr, so the projection is not well-formed.
trait Tr { type Out; }
struct B;
fn f(_x: <B as Tr>::Out) {}
fn main() { println!("hi"); }
