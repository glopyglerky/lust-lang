// expect: L0612
// ARCH 2.8, rustc E0277: `dyn A` is not Send.
trait A { fn a(&self) -> u8; }
struct S;
impl A for S { fn a(&self) -> u8 { 1 } }
fn need_send<T: Send + ?Sized>(_x: &T) {}
fn main() { let s = S; let d: &dyn A = &s; need_send(d); }
