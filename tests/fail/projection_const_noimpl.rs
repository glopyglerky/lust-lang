// expect: L0110
// rustc E0277: B does not implement Tr.
trait Tr { type Out; }
struct B;
const C: <B as Tr>::Out = 200;
fn main() { println!("hi"); }
